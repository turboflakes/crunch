// The MIT License (MIT)
// Copyright © 2021 Aukbit Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::config::CONFIG;
use crate::crunch::{
    get_from_seed, random_wait, try_fetch_stashes_from_remote_url, Crunch,
    NominatorsAmount, ValidatorAmount, ValidatorIndex,
};
use crate::errors::CrunchError;
use crate::pools::{nomination_pool_account, AccountType};
use crate::report::{
    EraIndex, Network, Payout, PayoutSummary, Points, RawData, Report, Signer, Validator,
    Validators,
};
use crate::stats;
use async_recursion::async_recursion;
use futures::StreamExt;
use log::{debug, info, warn};
use std::{
    cmp, convert::TryFrom, convert::TryInto, fs, result::Result, str::FromStr, thread,
    time,
};
use subxt::{
    ext::sp_core::{sr25519, Pair as PairT},
    ext::sp_runtime::AccountId32,
    tx::PairSigner,
    PolkadotConfig,
};

#[subxt::subxt(
    runtime_metadata_path = "metadata/kusama_metadata.scale",
    derive_for_all_types = "Clone"
)]
mod node_runtime {}

use node_runtime::{
    runtime_types::sp_core::bounded::bounded_vec::BoundedVec, staking::events::EraPaid,
    staking::events::PayoutStarted, staking::events::Rewarded,
    system::events::ExtrinsicFailed, utility::events::BatchCompleted,
    utility::events::BatchInterrupted, utility::events::ItemCompleted,
};

type Call = node_runtime::runtime_types::kusama_runtime::RuntimeCall;
type StakingCall = node_runtime::runtime_types::pallet_staking::pallet::pallet::Call;

pub async fn run_and_subscribe_era_paid_events(
    crunch: &Crunch,
) -> Result<(), CrunchError> {
    info!("Inspect and `crunch` unclaimed payout rewards");
    // Run once before start subscription
    try_run_batch(&crunch, None).await?;
    info!("Subscribe 'EraPaid' on-chain finalized event");
    let api = crunch.client().clone();
    let mut block_sub = api.blocks().subscribe_finalized().await?;
    while let Some(block) = block_sub.next().await {
        let block = block?;

        let events = block.events().await?;

        // Event --> staking::EraPaid
        if let Some(_event) = events.find_first::<EraPaid>()? {
            let wait: u64 = random_wait(240);
            info!("Waiting {} seconds before run batch", wait);
            thread::sleep(time::Duration::from_secs(wait));
            try_run_batch(&crunch, None).await?;
        }
    }
    // If subscription has closed for some reason await and subscribe again
    Err(CrunchError::SubscriptionFinished)
}

#[async_recursion]
pub async fn try_run_batch(
    crunch: &Crunch,
    next_attempt: Option<u8>,
) -> Result<(), CrunchError> {
    let api = crunch.client().clone();

    // Warn if static metadata is no longer the same as the latest runtime version
    if node_runtime::validate_codegen(&api).is_err() {
        warn!("Crunch upgrade might be required soon. Local static metadata differs from current chain runtime version.");
    }

    // Skip run if it's the 2nd or more attempt
    let mut attempt = match next_attempt {
        Some(na) => {
            if na >= 1 {
                None
            } else {
                Some(na + 1)
            }
        }
        _ => None,
    };
    //

    let config = CONFIG.clone();

    // Get Network name
    let chain_name = api.rpc().system_chain().await?;

    // Get Era index
    let active_era_addr = node_runtime::storage().staking().active_era();
    let active_era_index = match api.storage().fetch(&active_era_addr, None).await? {
        Some(info) => info.index,
        None => return Err(CrunchError::Other("Active era not available".into())),
    };

    let properties = api.rpc().system_properties().await?;

    // Get Token symbol
    let token_symbol: String = if let Some(token_symbol) = properties.get("tokenSymbol") {
        token_symbol.as_str().unwrap_or_default().to_string()
    } else {
        "ND".to_string()
    };

    // Get Token decimals
    let token_decimals: u8 = if let Some(token_decimals) = properties.get("tokenDecimals")
    {
        token_decimals
            .as_u64()
            .unwrap_or_default()
            .try_into()
            .unwrap()
    } else {
        12
    };

    // Set network info
    let network = Network {
        name: chain_name,
        active_era: active_era_index,
        token_symbol,
        token_decimals,
    };
    debug!("network {:?}", network);

    // Load seed account
    let seed = fs::read_to_string(config.seed_path)
        .expect("Something went wrong reading the seed file");
    let seed_account: sr25519::Pair = get_from_seed(&seed, None);
    let seed_account_signer =
        PairSigner::<PolkadotConfig, sr25519::Pair>::new(seed_account.clone());
    let seed_account_id: AccountId32 = seed_account.public().into();

    // Get signer account identity
    let signer_name = get_display_name(&crunch, &seed_account_id, None).await?;
    let mut signer = Signer {
        account: seed_account_id.clone(),
        name: signer_name,
        warnings: Vec::new(),
    };
    debug!("signer {:?}", signer);

    // Warn if signer account is running low on funds (if lower than 2x Existential Deposit)
    let ed_addr = node_runtime::constants().balances().existential_deposit();
    let ed = api.constants().at(&ed_addr)?;

    let seed_account_info_addr =
        node_runtime::storage().system().account(&seed_account_id);
    if let Some(seed_account_info) =
        api.storage().fetch(&seed_account_info_addr, None).await?
    {
        if seed_account_info.data.free
            <= (config.existential_deposit_factor_warning as u128 * ed)
        {
            signer
                .warnings
                .push("⚡ Signer account is running low on funds ⚡".to_string());
        }
    }

    // Add unclaimed eras into payout staker calls
    let mut calls_for_batch: Vec<Call> = vec![];
    let mut validators = collect_validators_data(&crunch, active_era_index).await?;
    let mut summary: PayoutSummary = Default::default();

    for v in &mut validators {
        //
        if v.unclaimed.len() > 0 {
            let mut maximum_payouts = Some(config.maximum_payouts);
            // define extrinsic payout stakers calls as many as unclaimed eras or maximum_payouts reached
            while let Some(i) = maximum_payouts {
                if i == 0 {
                    maximum_payouts = None;
                } else {
                    if let Some(claim_era) = v.unclaimed.pop() {
                        let call = Call::Staking(StakingCall::payout_stakers {
                            validator_stash: v.stash.clone(),
                            era: claim_era,
                        });
                        calls_for_batch.push(call);
                        summary.calls += 1;
                    }
                    maximum_payouts = Some(i - 1);
                }
            }
        }
        if v.is_active {
            summary.next_minimum_expected += 1;
        }
    }
    summary.total_validators = validators.len() as u32;
    summary.total_validators_previous_era_already_claimed = validators
        .iter()
        .map(|v| v.claimed.contains(&(active_era_index - 1)) as u32)
        .reduce(|a, b| a + b)
        .unwrap_or_default();

    if calls_for_batch.len() > 0 {
        // TODO check batch call weight or maximum_calls [default: 8]
        //
        // Calculate the number of extrinsics (iteractions) based on the maximum number of calls per batch
        // and the number of calls to be sent
        //
        let maximum_batch_calls =
            (calls_for_batch.len() as f32 / config.maximum_calls as f32).ceil() as u32;
        let mut iteration = Some(0);
        while let Some(x) = iteration {
            if x == maximum_batch_calls {
                iteration = None;
            } else {
                let mut validator_index: ValidatorIndex = None;
                let mut era_index: EraIndex = 0;
                let mut validator_amount_value: ValidatorAmount = 0;
                let mut nominators_amount_value: NominatorsAmount = 0;
                let mut nominators_quantity = 0;

                let call_start_index: usize =
                    (x * config.maximum_calls).try_into().unwrap();
                let call_end_index: usize = if config.maximum_calls
                    > calls_for_batch[call_start_index..].len() as u32
                {
                    ((x * config.maximum_calls)
                        + calls_for_batch[call_start_index..].len() as u32)
                        .try_into()
                        .unwrap()
                } else {
                    ((x * config.maximum_calls) + config.maximum_calls)
                        .try_into()
                        .unwrap()
                };

                debug!(
                    "batch call indexes [{:?} : {:?}]",
                    call_start_index, call_end_index
                );

                let calls_for_batch_clipped =
                    calls_for_batch[call_start_index..call_end_index].to_vec();

                // Note: Unvalidated extrinsic. If it fails a static metadata file will need to be updated!
                let tx = node_runtime::tx()
                    .utility()
                    .batch(calls_for_batch_clipped.clone())
                    .unvalidated();

                let batch_response = api
                    .tx()
                    .sign_and_submit_then_watch_default(&tx, &seed_account_signer)
                    .await?
                    .wait_for_finalized()
                    .await?;

                // Alternately, we could just `fetch_events`, which grabs all of the events like
                // the above, but does not check for success, and leaves it up to you:
                let tx_events = batch_response.fetch_events().await?;

                // Get block number
                let block_number = if let Some(header) =
                    api.rpc().header(Some(tx_events.block_hash())).await?
                {
                    header.number
                } else {
                    0
                };

                let failed_event = tx_events.find_first::<ExtrinsicFailed>()?;

                if let Some(ev) = failed_event {
                    // TODO: repeat the batch call?  Or just log an error
                    return Err(CrunchError::Other(format!(
                        "Extrinsic failed: {:?}",
                        ev
                    )));
                } else {
                    // Iterate over events to calculate respective reward amounts
                    for event in tx_events.iter() {
                        let event = event?;
                        debug!("{:?}", event);
                        if let Some(ev) = event.as_event::<PayoutStarted>()? {
                            // https://polkadot.js.org/docs/substrate/events#payoutstartedu32-accountid32
                            // PayoutStarted(u32, AccountId32)
                            // summary: The stakers' rewards are getting paid. [era_index, validator_stash]
                            //
                            debug!("{:?}", ev);
                            let validator_index_ref = &mut validators
                                .iter()
                                .position(|v| v.stash == ev.validator_stash);
                            era_index = ev.era_index;
                            validator_index = *validator_index_ref;
                            validator_amount_value = 0;
                            nominators_amount_value = 0;
                            nominators_quantity = 0;
                        } else if let Some(ev) = event.as_event::<Rewarded>()? {
                            // https://polkadot.js.org/docs/substrate/events#rewardedaccountid32-u128
                            // Rewarded(AccountId32, u128)
                            // summary: An account has been rewarded for their signed submission being finalized
                            //
                            debug!("{:?}", ev);
                            if let Some(i) = validator_index {
                                let validator = &mut validators[i];
                                if ev.stash == validator.stash {
                                    validator_amount_value = ev.amount;
                                } else {
                                    nominators_amount_value += ev.amount;
                                    nominators_quantity += 1;
                                }
                            }
                        } else if let Some(_ev) = event.as_event::<ItemCompleted>()? {
                            // https://polkadot.js.org/docs/substrate/events#itemcompleted
                            // summary: A single item within a Batch of dispatches has completed with no error.
                            //
                            if let Some(i) = validator_index {
                                let validator = &mut validators[i];
                                // Add era to claimed vec
                                validator.claimed.push(era_index);
                                // Fetch stash points
                                let points = get_validator_points_info(
                                    &crunch,
                                    era_index,
                                    &validator.stash,
                                )
                                .await?;

                                let p = Payout {
                                    block_number,
                                    extrinsic: tx_events.extrinsic_hash(),
                                    era_index,
                                    validator_amount_value,
                                    nominators_amount_value,
                                    nominators_quantity,
                                    points,
                                };
                                validator.payouts.push(p);
                                summary.calls_succeeded += 1;
                            }
                        } else if let Some(_ev) = event.as_event::<BatchCompleted>()? {
                            // https://polkadot.js.org/docs/substrate/events#batchcompleted
                            // summary: Batch of dispatches completed fully with no error.
                            info!("Batch Completed for Era {}", network.active_era);
                            attempt = None;
                        } else if let Some(ev) = event.as_event::<BatchInterrupted>()? {
                            // https://polkadot.js.org/docs/substrate/events#batchinterruptedu32-spruntimedispatcherror
                            // summary: Batch of dispatches did not complete fully. Index of first failing dispatch given, as well as the error.
                            //
                            // Fix: https://github.com/turboflakes/crunch/issues/4
                            // Most likely the batch was interrupted because of an AlreadyClaimed era
                            // BatchInterrupted { index: 0, error: Module { index: 6, error: 14 } }
                            warn!("{:?}", ev);
                            if let Call::Staking(call) = &calls_for_batch_clipped
                                [usize::try_from(ev.index).unwrap()]
                            {
                                match &call {
                                    StakingCall::payout_stakers {
                                        validator_stash,
                                        ..
                                    } => {
                                        warn!("validator_stash: {:?}", validator_stash);
                                        let validator_index = &mut validators
                                            .iter()
                                            .position(|v| v.stash == *validator_stash);

                                        if let Some(i) = *validator_index {
                                            let validator = &mut validators[i];
                                            // TODO: decode DispatchError to a readable format
                                            validator.warnings.push(
                                                "⚡ Batch interrupted ⚡".to_string(),
                                            );
                                        }
                                    }
                                    _ => unreachable!(),
                                };
                            }
                            // Attempt to run batch once again
                            attempt = if attempt.is_none() { Some(1) } else { attempt };
                        }
                    }
                }
                iteration = Some(x + 1);
            }
        }
    }

    // Prepare notification report
    debug!("validators {:?}", validators);

    let data = RawData {
        network,
        signer,
        validators,
        summary,
    };

    let report = Report::from(data);
    crunch
        .send_message(&report.message(), &report.formatted_message())
        .await?;

    // Note: If there's anything to attempt, call recursively try_run_batch one more time
    if let None = attempt {
        Ok(())
    } else {
        return try_run_batch(&crunch, attempt).await;
    }
}

async fn collect_validators_data(
    crunch: &Crunch,
    era_index: EraIndex,
) -> Result<Validators, CrunchError> {
    let api = crunch.client().clone();

    // Get unclaimed eras for the stash addresses
    let active_validators_addr = node_runtime::storage().session().validators();
    let active_validators = api.storage().fetch(&active_validators_addr, None).await?;
    debug!("active_validators {:?}", active_validators);
    let mut validators: Validators = Vec::new();

    let stashes = get_stashes(&crunch).await?;

    for (_i, stash_str) in stashes.iter().enumerate() {
        let stash = AccountId32::from_str(stash_str)?;

        // Check if stash has bonded controller
        let controller_addr = node_runtime::storage().staking().bonded(&stash);
        let controller = match api.storage().fetch(&controller_addr, None).await? {
            Some(controller) => controller,
            None => {
                let mut v = Validator::new(stash.clone());
                v.warnings = vec![format!(
                    "Stash <code>{}</code> does not have a bonded Controller account!",
                    stash
                )];
                validators.push(v);
                continue;
            }
        };
        debug!("controller {:?}", controller);
        // Instantiates a new validator struct
        let mut v = Validator::new(stash.clone());

        // Set controller
        v.controller = Some(controller.clone());

        // Get validator name
        v.name = get_display_name(&crunch, &stash, None).await?;

        // Check if validator is in active set
        v.is_active = if let Some(ref av) = active_validators {
            av.contains(&stash)
        } else {
            false
        };

        // Look for unclaimed eras, starting on current_era - maximum_eras
        let start_index = get_era_index_start(&crunch, era_index).await?;

        // Get staking info from ledger
        let ledger_addr = node_runtime::storage().staking().ledger(&controller);
        if let Some(staking_ledger) = api.storage().fetch(&ledger_addr, None).await? {
            debug!(
                "{} * claimed_rewards: {:?}",
                stash, staking_ledger.claimed_rewards
            );

            // deconstruct claimed rewards
            let BoundedVec(claimed_rewards) = staking_ledger.claimed_rewards;
            // Find unclaimed eras in previous 84 eras (reverse order)
            for e in (start_index..era_index).rev() {
                // If reward was already claimed skip it
                if claimed_rewards.contains(&e) {
                    if e == era_index - 1 {
                        v.is_previous_era_already_claimed = true;
                    }
                    v.claimed.push(e);
                    continue;
                }
                // Verify if stash was active in set
                let eras_stakers_addr =
                    node_runtime::storage().staking().eras_stakers(&e, &stash);
                if let Some(exposure) =
                    api.storage().fetch(&eras_stakers_addr, None).await?
                {
                    if exposure.total > 0 {
                        v.unclaimed.push(e)
                    }
                }
            }
        }
        validators.push(v);
    }
    debug!("validators {:?}", validators);
    Ok(validators)
}

async fn get_era_index_start(
    crunch: &Crunch,
    era_index: EraIndex,
) -> Result<EraIndex, CrunchError> {
    let api = crunch.client().clone();
    let config = CONFIG.clone();

    let history_depth_addr = node_runtime::constants().staking().history_depth();
    let history_depth: u32 = api.constants().at(&history_depth_addr)?;

    if era_index < cmp::min(config.maximum_history_eras, history_depth) {
        return Ok(0);
    } else if config.is_short {
        return Ok(era_index - cmp::min(config.maximum_history_eras, history_depth));
    } else {
        // Note: If crunch is running in verbose mode, ignore MAXIMUM_ERAS
        // since we still want to show information about inclusion and eras crunched for all history_depth
        return Ok(era_index - history_depth);
    }
}

async fn get_validator_points_info(
    crunch: &Crunch,
    era_index: EraIndex,
    stash: &AccountId32,
) -> Result<Points, CrunchError> {
    let api = crunch.client().clone();
    // Get era reward points
    let era_reward_points_addr = node_runtime::storage()
        .staking()
        .eras_reward_points(&era_index);

    if let Some(era_reward_points) =
        api.storage().fetch(&era_reward_points_addr, None).await?
    {
        let stash_points = match era_reward_points
            .individual
            .iter()
            .find(|(s, _)| *s == *stash)
        {
            Some((_, p)) => *p,
            None => 0,
        };

        // Calculate average points
        let mut points: Vec<u32> = era_reward_points
            .individual
            .into_iter()
            .map(|(_, points)| points)
            .collect();

        let points_f64: Vec<f64> = points.iter().map(|points| *points as f64).collect();

        let points = Points {
            validator: stash_points,
            era_avg: stats::mean(&points_f64),
            ci99_9_interval: stats::confidence_interval_99_9(&points_f64),
            outlier_limits: stats::iqr_interval(&mut points),
        };

        Ok(points)
    } else {
        Ok(Points::default())
    }
}

#[async_recursion]
async fn get_display_name(
    crunch: &Crunch,
    stash: &AccountId32,
    sub_account_name: Option<String>,
) -> Result<String, CrunchError> {
    let api = crunch.client().clone();

    let identity_of_addr = node_runtime::storage().identity().identity_of(stash);
    match api.storage().fetch(&identity_of_addr, None).await? {
        Some(identity) => {
            debug!("identity {:?}", identity);
            let parent = parse_identity_data(identity.info.display);
            let name = match sub_account_name {
                Some(child) => format!("{}/{}", parent, child),
                None => parent,
            };
            Ok(name)
        }
        None => {
            let super_of_addr = node_runtime::storage().identity().super_of(stash);
            if let Some((parent_account, data)) =
                api.storage().fetch(&super_of_addr, None).await?
            {
                let sub_account_name = parse_identity_data(data);
                return get_display_name(
                    &crunch,
                    &parent_account,
                    Some(sub_account_name.to_string()),
                )
                .await;
            } else {
                let s = &stash.to_string();
                Ok(format!("{}...{}", &s[..6], &s[s.len() - 6..]))
            }
        }
    }
}

//
fn parse_identity_data(
    data: node_runtime::runtime_types::pallet_identity::types::Data,
) -> String {
    match data {
        node_runtime::runtime_types::pallet_identity::types::Data::Raw0(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw1(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw2(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw3(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw4(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw5(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw6(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw7(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw8(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw9(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw10(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw11(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw12(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw13(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw14(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw15(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw16(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw17(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw18(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw19(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw20(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw21(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw22(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw23(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw24(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw25(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw26(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw27(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw28(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw29(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw30(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw31(bytes) => {
            str(bytes.to_vec())
        }
        node_runtime::runtime_types::pallet_identity::types::Data::Raw32(bytes) => {
            str(bytes.to_vec())
        }
        _ => format!("???"),
    }
}

fn str(bytes: Vec<u8>) -> String {
    format!("{}", String::from_utf8(bytes).expect("Identity not utf-8"))
}

pub async fn inspect(crunch: &Crunch) -> Result<(), CrunchError> {
    let api = crunch.client().clone();

    let stashes = get_stashes(&crunch).await?;
    info!("Inspect {} stashes -> {}", stashes.len(), stashes.join(","));

    let history_depth_addr = node_runtime::constants().staking().history_depth();
    let history_depth: u32 = api.constants().at(&history_depth_addr)?;

    let active_era_addr = node_runtime::storage().staking().active_era();
    let active_era_index = match api.storage().fetch(&active_era_addr, None).await? {
        Some(info) => info.index,
        None => return Err(CrunchError::Other("Active era not available".into())),
    };

    for stash_str in stashes.iter() {
        let stash = AccountId32::from_str(stash_str)?;
        info!("{} * Stash account", stash);

        let start_index = active_era_index - history_depth;
        let mut unclaimed: Vec<u32> = Vec::new();
        let mut claimed: Vec<u32> = Vec::new();

        let bonded_addr = node_runtime::storage().staking().bonded(&stash);
        if let Some(controller) = api.storage().fetch(&bonded_addr, None).await? {
            let ledger_addr = node_runtime::storage().staking().ledger(&controller);
            if let Some(ledger_response) = api.storage().fetch(&ledger_addr, None).await?
            {
                // deconstruct claimed rewards
                let BoundedVec(claimed_rewards) = ledger_response.claimed_rewards;
                // Find unclaimed eras in previous 84 eras
                for era_index in start_index..active_era_index {
                    // If reward was already claimed skip it
                    if claimed_rewards.contains(&era_index) {
                        claimed.push(era_index);
                        continue;
                    }
                    // Verify if stash was active in set
                    let eras_stakers_addr = node_runtime::storage()
                        .staking()
                        .eras_stakers(&era_index, &stash);
                    if let Some(exposure) =
                        api.storage().fetch(&eras_stakers_addr, None).await?
                    {
                        if exposure.total > 0 {
                            unclaimed.push(era_index)
                        }
                    }
                }
            }
        }
        info!(
            "{} claimed eras in the last {} -> {:?}",
            claimed.len(),
            history_depth,
            claimed
        );
        info!(
            "{} unclaimed eras in the last {} -> {:?}",
            unclaimed.len(),
            history_depth,
            unclaimed
        );
    }
    info!("Job done!");
    Ok(())
}

pub async fn get_stashes(crunch: &Crunch) -> Result<Vec<String>, CrunchError> {
    let config = CONFIG.clone();

    let mut stashes: Vec<String> = config.stashes;
    info!("{} stashes loaded from 'config.stashes'", stashes.len());

    if let Some(remotes) = try_fetch_stashes_from_remote_url().await? {
        stashes.extend(remotes);
    };

    if let Some(nominees) = try_fetch_stashes_from_pool_ids(&crunch).await? {
        stashes.extend(nominees);
    }

    if config.unique_stashes_enabled {
        // sort and remove duplicates
        stashes.sort();
        stashes.dedup();
    }

    Ok(stashes)
}

pub async fn try_fetch_stashes_from_pool_ids(
    crunch: &Crunch,
) -> Result<Option<Vec<String>>, CrunchError> {
    let api = crunch.client().clone();
    let config = CONFIG.clone();
    if config.pool_ids.len() == 0 {
        return Ok(None);
    }

    let active_era_addr = node_runtime::storage().staking().active_era();
    let era_index = match api.storage().fetch(&active_era_addr, None).await? {
        Some(info) => info.index,
        None => return Err("Active era not defined".into()),
    };

    let mut all: Vec<String> = Vec::new();
    let mut active: Vec<String> = Vec::new();

    for pool_id in config.pool_ids.iter() {
        let pool_stash_account = nomination_pool_account(AccountType::Bonded, *pool_id);
        let nominators_addr = node_runtime::storage()
            .staking()
            .nominators(&pool_stash_account);
        if let Some(nominations) = api.storage().fetch(&nominators_addr, None).await? {
            // deconstruct targets
            let BoundedVec(targets) = nominations.targets;
            all.extend(
                targets
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            );

            // NOTE_1: Only check active nominees from previous era
            // By the end of current era crunch will trigger any payout left from previous eras if that is the case.
            // NOTE_2: Ideally nominees shouldn't have any pending payouts, but is in the best interest of the pool members
            // that pool operators trigger payouts as a backup at least for the active nominees.
            for stash in targets {
                let eras_stakers_addr = node_runtime::storage()
                    .staking()
                    .eras_stakers(era_index - 1, &stash);
                if let Some(exposure) =
                    api.storage().fetch(&eras_stakers_addr, None).await?
                {
                    if exposure.others.iter().any(|x| x.who == pool_stash_account) {
                        active.push(stash.to_string());
                    }
                }
            }
        }
    }
    if all.is_empty() && active.is_empty() {
        return Ok(None);
    }

    if config.all_nominees_payouts_enabled {
        info!(
            "{} stashes loaded from 'pool-ids': [{}]",
            all.len(),
            config
                .pool_ids
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );

        return Ok(Some(all));
    }

    // Note: by default only active nominees (stashes) are triggered
    info!(
        "{} active stashes loaded from 'pool-ids': [{}]",
        active.len(),
        config
            .pool_ids
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );

    Ok(Some(active))
}
