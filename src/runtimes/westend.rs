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
    get_from_seed, random_wait, try_fetch_stashes_from_remote_url, Crunch, NominatorsAmount,
    ValidatorAmount, ValidatorIndex,
};
use crate::errors::CrunchError;
use crate::report::{
    EraIndex, Network, Payout, Points, RawData, Report, Signer, Validator, Validators,
};
use crate::stats;
use async_recursion::async_recursion;
use codec::Decode;
use log::{debug, info, warn};
use std::{
    cmp, convert::TryFrom, convert::TryInto, fs, result::Result, str::FromStr, thread, time,
};
use subxt::{
    sp_core::{sr25519, Pair as PairT},
    sp_runtime::AccountId32,
    DefaultConfig, DefaultExtra, EventSubscription, PairSigner, RawEvent,
};

#[subxt::subxt(
    runtime_metadata_path = "metadata/westend_metadata.scale",
    generated_type_derives = "Clone, Debug"
)]
mod api {}

pub type WestendApi = api::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>;

type Call = api::runtime_types::westend_runtime::Call;
type StakingCall = api::runtime_types::pallet_staking::pallet::pallet::Call;

pub async fn run_and_subscribe_era_paid_events(crunch: &Crunch) -> Result<(), CrunchError> {
    info!("Inspect and `crunch` unclaimed payout rewards");
    // Run once before start subscription
    try_run_batch(&crunch, None).await?;
    info!("Subscribe 'EraPaid' on-chain finalized event");
    let client = crunch.client().clone();
    let sub = client.rpc().subscribe_finalized_events().await?;
    let decoder = client.events_decoder();
    let mut sub = EventSubscription::<DefaultConfig>::new(sub, &decoder);
    sub.filter_event::<api::staking::events::EraPaid>();
    while let Some(result) = sub.next().await {
        if let Ok(raw) = result {
            match api::staking::events::EraPaid::decode(&mut &raw.data[..]) {
                Ok(event) => {
                    info!("Successfully decoded event {:?}", event);
                    let wait: u64 = random_wait(120);
                    info!("Waiting {} seconds before run batch", wait);
                    thread::sleep(time::Duration::from_secs(wait));
                    try_run_batch(&crunch, None).await?;
                }
                Err(e) => return Err(CrunchError::CodecError(e)),
            }
        }
    }
    // If subscription has closed for some reason await and subscribe again
    Err(CrunchError::SubscriptionFinished)
}

#[async_recursion]
pub async fn try_run_batch(crunch: &Crunch, next_attempt: Option<u8>) -> Result<(), CrunchError> {
    // Skip run if it's the 3rd or more attempt
    let mut attempt = match next_attempt {
        Some(na) => {
            if na >= 3 {
                return Ok(());
            } else {
                next_attempt
            }
        }
        _ => None,
    };
    //
    let client = crunch.client();
    let api = client.clone().to_runtime_api::<WestendApi>();
    let properties = client.properties();
    let config = CONFIG.clone();

    // Get Network name
    let chain_name = client.rpc().system_chain().await?;

    // Get Era index
    let active_era_index = match api.storage().staking().active_era(None).await? {
        Some(info) => info.index,
        None => return Err(CrunchError::Other("Active era not available".into())),
    };

    // Get Token symbol
    let token_symbol: String = if let Some(token_symbol) = properties.get("tokenSymbol") {
        token_symbol.as_str().unwrap_or_default().to_string()
    } else {
        "ND".to_string()
    };

    // Get Token decimals
    let token_decimals: u8 = if let Some(token_decimals) = properties.get("tokenDecimals") {
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
        token_symbol: token_symbol,
        token_decimals: token_decimals,
    };
    debug!("network {:?}", network);

    // Load seed account
    let seed =
        fs::read_to_string(config.seed_path).expect("Something went wrong reading the seed file");
    let seed_account: sr25519::Pair = get_from_seed(&seed, None);
    let seed_account_signer =
        PairSigner::<DefaultConfig, DefaultExtra<DefaultConfig>, sr25519::Pair>::new(
            seed_account.clone(),
        );
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
    let ed = get_existential_deposit(&crunch)?;
    let seed_account_info = api
        .storage()
        .system()
        .account(seed_account_id, None)
        .await?;
    if seed_account_info.data.free <= (2 * ed) {
        signer
            .warnings
            .push("⚡ Signer account is running low on funds ⚡".to_string());
    }

    // Add unclaimed eras into payout staker calls
    let mut calls_for_batch: Vec<Call> = vec![];
    let mut validators = collect_validators_data(&crunch, active_era_index).await?;
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
                    }
                    maximum_payouts = Some(i - 1);
                }
            }
        }
    }

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

                let call_start_index: usize = (x * config.maximum_calls).try_into().unwrap();
                let call_end_index: usize = if config.maximum_calls
                    > calls_for_batch[call_start_index..].len() as u32
                {
                    ((x * config.maximum_calls) + calls_for_batch[call_start_index..].len() as u32)
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
                let batch_response = api
                    .tx()
                    .utility()
                    .batch(calls_for_batch_clipped.clone())
                    .sign_and_submit_then_watch(&seed_account_signer)
                    .await?
                    .wait_for_finalized()
                    .await?;
                debug!("batch_response {:?}", batch_response);

                // Alternately, we could just `fetch_events`, which grabs all of the events like
                // the above, but does not check for success, and leaves it up to you:
                let tx_events = batch_response.fetch_events().await?;

                // Get block number
                let block_number = if let Some(header) =
                    client.rpc().header(Some(tx_events.block_hash())).await?
                {
                    header.number
                } else {
                    0
                };

                let failed_event =
                    tx_events.find_first_event::<api::system::events::ExtrinsicFailed>()?;

                if let Some(ev) = failed_event {
                    // TODO: repeat the batch call?  Or just log an error
                    return Err(CrunchError::Other(format!("Extrinsic failed: {:?}", ev)));
                } else {
                    // Iterate over events to calculate respective reward amounts
                    for event in tx_events.as_slice() {
                        debug!("{:?}", event);
                        match event {
                            RawEvent {
                                ref pallet,
                                ref variant,
                                data,
                                ..
                            } if pallet == "Staking" && variant == "PayoutStarted" => {
                                // https://polkadot.js.org/docs/substrate/events#payoutstartedu32-accountid32
                                // PayoutStarted(u32, AccountId32)
                                // summary: The stakers' rewards are getting paid. [era_index, validator_stash]
                                //
                                let event_decoded =
                                    api::staking::events::PayoutStarted::decode(&mut &data[..])?;
                                debug!("{:?}", event_decoded);
                                let validator_index_ref =
                                    &mut validators.iter().position(|v| v.stash == event_decoded.1);
                                era_index = event_decoded.0;
                                validator_index = *validator_index_ref;
                                validator_amount_value = 0;
                                nominators_amount_value = 0;
                                nominators_quantity = 0;
                            }
                            RawEvent {
                                ref pallet,
                                ref variant,
                                data,
                                ..
                            } if pallet == "Staking" && variant == "Rewarded" => {
                                // https://polkadot.js.org/docs/substrate/events#rewardedaccountid32-u128
                                // Rewarded(AccountId32, u128)
                                // summary: An account has been rewarded for their signed submission being finalized
                                //
                                let event_decoded =
                                    api::staking::events::Rewarded::decode(&mut &data[..])?;
                                debug!("{:?}", event_decoded);
                                if let Some(i) = validator_index {
                                    let validator = &mut validators[i];
                                    if event_decoded.0 == validator.stash {
                                        validator_amount_value = event_decoded.1;
                                    } else {
                                        nominators_amount_value += event_decoded.1;
                                        nominators_quantity += 1;
                                    }
                                }
                            }
                            RawEvent {
                                ref pallet,
                                ref variant,
                                ..
                            } if pallet == "Utility" && variant == "ItemCompleted" => {
                                // https://polkadot.js.org/docs/substrate/events#itemcompleted
                                // summary: A single item within a Batch of dispatches has completed with no error.
                                if let Some(i) = validator_index {
                                    let validator = &mut validators[i];
                                    // Add era to claimed vec
                                    validator.claimed.push(era_index);
                                    // Fetch stash points
                                    let points = get_validator_points_info(
                                        &crunch,
                                        era_index,
                                        validator.stash.clone(),
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
                                }
                            }
                            RawEvent {
                                ref pallet,
                                ref variant,
                                ..
                            } if pallet == "Utility" && variant == "BatchCompleted" => {
                                // https://polkadot.js.org/docs/substrate/events#batchcompleted
                                // summary: Batch of dispatches completed fully with no error.
                                info!("Batch Completed for Era {}", network.active_era);
                                attempt = None;
                            }
                            RawEvent {
                                ref pallet,
                                ref variant,
                                data,
                                ..
                            } if pallet == "Utility" && variant == "BatchInterrupted" => {
                                // https://polkadot.js.org/docs/substrate/events#batchinterruptedu32-spruntimedispatcherror
                                // summary: Batch of dispatches did not complete fully. Index of first failing dispatch given, as well as the error.
                                //
                                // Fix: https://github.com/turboflakes/crunch/issues/4
                                // Most likely the batch was interrupted because of an AlreadyClaimed era
                                // BatchInterrupted { index: 0, error: Module { index: 6, error: 14 } }
                                let event_decoded =
                                    api::utility::events::BatchInterrupted::decode(&mut &data[..])?;
                                warn!("{:?}", event_decoded);

                                if let Call::Staking(call) = &calls_for_batch_clipped
                                    [usize::try_from(event_decoded.index).unwrap()]
                                {
                                    match &call {
                                        StakingCall::payout_stakers {
                                            validator_stash, ..
                                        } => {
                                            warn!("validator_stash: {:?}", validator_stash);
                                            let validator_index = &mut validators
                                                .iter()
                                                .position(|v| v.stash == *validator_stash);

                                            if let Some(i) = *validator_index {
                                                let validator = &mut validators[i];
                                                // TODO: decode DispatchError to a readable format
                                                validator
                                                    .warnings
                                                    .push("⚡ Batch interrupted ⚡".to_string());
                                            }
                                        }
                                        _ => unreachable!(),
                                    };
                                }
                                // Attempt to run one more time
                                attempt = if let Some(na) = attempt {
                                    Some(na + 1)
                                } else {
                                    Some(1)
                                };
                            }
                            _ => (),
                        };
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
    let client = crunch.client();
    let api = client.clone().to_runtime_api::<WestendApi>();
    let config = CONFIG.clone();

    // Get unclaimed eras for the stash addresses
    let active_validators = api.storage().session().validators(None).await?;
    debug!("active_validators {:?}", active_validators);
    let mut validators: Validators = Vec::new();

    let stashes: Vec<String> = match try_fetch_stashes_from_remote_url().await? {
        Some(stashes) => stashes,
        None => config.stashes,
    };

    for (_i, stash_str) in stashes.iter().enumerate() {
        let stash = AccountId32::from_str(stash_str)?;

        // Check if stash has bonded controller
        let controller = match api.storage().staking().bonded(stash.clone(), None).await? {
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
        v.is_active = active_validators.contains(&stash);

        // Look for unclaimed eras, starting on current_era - maximum_eras
        let start_index = get_era_index_start(&crunch, era_index).await?;

        // Get staking info from ledger
        if let Some(staking_ledger) = api
            .storage()
            .staking()
            .ledger(controller.clone(), None)
            .await?
        {
            debug!(
                "{} * claimed_rewards: {:?}",
                stash, staking_ledger.claimed_rewards
            );

            // Find unclaimed eras in previous 84 eras (reverse order)
            for e in (start_index..era_index).rev() {
                // If reward was already claimed skip it
                if staking_ledger.claimed_rewards.contains(&e) {
                    v.claimed.push(e);
                    continue;
                }
                // Verify if stash was active in set
                let exposure = api
                    .storage()
                    .staking()
                    .eras_stakers(e, stash.clone(), None)
                    .await?;
                if exposure.total > 0 {
                    v.unclaimed.push(e)
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
    let client = crunch.client();
    let api = client.clone().to_runtime_api::<WestendApi>();
    let config = CONFIG.clone();

    let history_depth: u32 = api.storage().staking().history_depth(None).await?;

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

fn get_existential_deposit(crunch: &Crunch) -> Result<u128, CrunchError> {
    let client = crunch.client();
    let balances_metadata = client.metadata().pallet("Balances")?;
    let constant_metadata = balances_metadata.constant("ExistentialDeposit")?;
    let ed = u128::decode(&mut &constant_metadata.value[..])?;
    Ok(ed)
}

async fn get_validator_points_info(
    crunch: &Crunch,
    era_index: EraIndex,
    stash: AccountId32,
) -> Result<Points, CrunchError> {
    let client = crunch.client();
    let api = client.clone().to_runtime_api::<WestendApi>();
    // Get era reward points
    let era_reward_points = api
        .storage()
        .staking()
        .eras_reward_points(era_index, None)
        .await?;
    let stash_points = match era_reward_points
        .individual
        .iter()
        .find(|(s, _)| *s == &stash)
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
}

#[async_recursion]
async fn get_display_name(
    crunch: &Crunch,
    stash: &AccountId32,
    sub_account_name: Option<String>,
) -> Result<String, CrunchError> {
    let client = crunch.client();
    let api = client.clone().to_runtime_api::<WestendApi>();

    match api
        .storage()
        .identity()
        .identity_of(stash.clone(), None)
        .await?
    {
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
            if let Some((parent_account, data)) = api
                .storage()
                .identity()
                .super_of(stash.clone(), None)
                .await?
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
fn parse_identity_data(data: api::runtime_types::pallet_identity::types::Data) -> String {
    match data {
        api::runtime_types::pallet_identity::types::Data::Raw0(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw1(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw2(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw3(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw4(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw5(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw6(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw7(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw8(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw9(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw10(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw11(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw12(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw13(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw14(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw15(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw16(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw17(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw18(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw19(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw20(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw21(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw22(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw23(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw24(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw25(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw26(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw27(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw28(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw29(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw30(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw31(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw32(bytes) => str(bytes.to_vec()),
        _ => format!("???"),
    }
}

fn str(bytes: Vec<u8>) -> String {
    format!("{}", String::from_utf8(bytes).expect("Identity not utf-8"))
}

pub async fn inspect(crunch: &Crunch) -> Result<(), CrunchError> {
    let client = crunch.client();
    let api = client.clone().to_runtime_api::<WestendApi>();
    let config = CONFIG.clone();

    info!("Inspect stashes -> {}", config.stashes.join(","));
    let history_depth: u32 = api.storage().staking().history_depth(None).await?;
    let active_era_index = match api.storage().staking().active_era(None).await? {
        Some(active_era_info) => active_era_info.index,
        None => return Err(CrunchError::Other("Active era not available".into())),
    };
    for stash_str in config.stashes.iter() {
        let stash = AccountId32::from_str(stash_str)?;
        info!("{} * Stash account", stash);

        let start_index = active_era_index - history_depth;
        let mut unclaimed: Vec<u32> = Vec::new();
        let mut claimed: Vec<u32> = Vec::new();

        if let Some(controller) = api.storage().staking().bonded(stash.clone(), None).await? {
            if let Some(ledger_response) = api
                .storage()
                .staking()
                .ledger(controller.clone(), None)
                .await?
            {
                // Find unclaimed eras in previous 84 eras
                for era_index in start_index..active_era_index {
                    // If reward was already claimed skip it
                    if ledger_response.claimed_rewards.contains(&era_index) {
                        claimed.push(era_index);
                        continue;
                    }
                    // Verify if stash was active in set
                    let exposure = api
                        .storage()
                        .staking()
                        .eras_stakers(era_index, stash.clone(), None)
                        .await?;
                    if exposure.total > 0 {
                        unclaimed.push(era_index)
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
