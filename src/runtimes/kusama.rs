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
    get_account_id_from_storage_key, get_keypair_from_seed_file, random_wait,
    try_fetch_onet_data, try_fetch_stashes_from_remote_url, Crunch, NominatorsAmount,
    ValidatorAmount, ValidatorIndex,
};
use crate::errors::CrunchError;
use crate::pools::{nomination_pool_account, AccountType};
use crate::report::{
    Batch, EraIndex, Network, NominationPoolsSummary, PageIndex, Payout, PayoutSummary,
    Points, RawData, Report, SignerDetails, Validator, Validators,
};
use crate::stats;
use async_recursion::async_recursion;
use log::{debug, info, warn};
use std::{
    cmp, convert::TryFrom, convert::TryInto, result::Result, str::FromStr, thread, time,
};
use subxt::{
    config::polkadot::PolkadotExtrinsicParamsBuilder as TxParams,
    error::DispatchError,
    ext::codec::{Decode, Encode},
    tx::TxStatus,
    utils::{AccountId32, MultiAddress},
};

use subxt_signer::sr25519::Keypair;

#[subxt::subxt(
    runtime_metadata_path = "metadata/kusama_metadata_small.scale",
    derive_for_all_types = "Clone, PartialEq"
)]
mod relay_runtime {}

use relay_runtime::{
    runtime_types::bounded_collections::bounded_vec::BoundedVec,
    runtime_types::pallet_nomination_pools::{BondExtra, ClaimPermission},
    staking::events::EraPaid,
    staking::events::PayoutStarted,
    staking::events::Rewarded,
    system::events::ExtrinsicFailed,
    utility::events::BatchCompleted,
    utility::events::BatchCompletedWithErrors,
    utility::events::BatchInterrupted,
    utility::events::ItemCompleted,
    utility::events::ItemFailed,
};

#[subxt::subxt(
    runtime_metadata_path = "metadata/people_kusama_metadata_small.scale",
    derive_for_all_types = "Clone, PartialEq"
)]
mod people_runtime {}

type Call = relay_runtime::runtime_types::staging_kusama_runtime::RuntimeCall;
type StakingCall = relay_runtime::runtime_types::pallet_staking::pallet::pallet::Call;
type NominationPoolsCall =
    relay_runtime::runtime_types::pallet_nomination_pools::pallet::Call;

pub async fn run_and_subscribe_era_paid_events(
    crunch: &Crunch,
) -> Result<(), CrunchError> {
    info!("Inspect and `crunch` unclaimed payout rewards");
    // Run once before start subscription
    try_crunch(&crunch).await?;
    let mut latest_block_number_processed: Option<u32> = Some(0);
    info!("Subscribe 'EraPaid' on-chain finalized event");
    let api = crunch.client().clone();
    let mut block_sub = api.blocks().subscribe_finalized().await?;
    while let Some(block) = block_sub.next().await {
        // let block = block?;

        // Silently handle RPC disconnection and wait for the next block as soon as reconnection is available
        let block = match block {
            Ok(b) => b,
            Err(e) => {
                if e.is_disconnected_will_reconnect() {
                    warn!("The RPC connection was dropped will try to reconnect.");
                    continue;
                }
                return Err(e.into());
            }
        };

        // Process blocks that might have been dropped while reconnecting
        while let Some(processed_block_number) = latest_block_number_processed {
            if block.number() == processed_block_number || processed_block_number == 0 {
                latest_block_number_processed = None;
            } else {
                let block_number = processed_block_number + 1;

                // Skip current block and fetch only blocks that have not yet been processed
                if block.number() - block_number > 0 {
                    if let Some(block_hash) = crunch
                        .rpc()
                        .chain_get_block_hash(Some(block_number.into()))
                        .await?
                    {
                        let events = api.events().at(block_hash).await?;

                        // Event --> staking::EraPaid
                        if let Some(_event) = events.find_first::<EraPaid>()? {
                            let wait: u64 = random_wait(240);
                            info!("Waiting {} seconds before run batch", wait);
                            thread::sleep(time::Duration::from_secs(wait));
                            try_crunch(&crunch).await?;
                        }
                    }
                }

                latest_block_number_processed = Some(block_number);
            }
        }

        let events = block.events().await?;

        // Event --> staking::EraPaid
        if let Some(_event) = events.find_first::<EraPaid>()? {
            let wait: u64 = random_wait(240);
            info!("Waiting {} seconds before run batch", wait);
            thread::sleep(time::Duration::from_secs(wait));
            try_crunch(&crunch).await?;
        }

        latest_block_number_processed = Some(block.number());
    }
    // If subscription has closed for some reason await and subscribe again
    Err(CrunchError::SubscriptionFinished)
}

pub async fn try_crunch(crunch: &Crunch) -> Result<(), CrunchError> {
    let config = CONFIG.clone();
    let api = crunch.client().clone();

    let signer_keypair: Keypair = get_keypair_from_seed_file()?;
    let seed_account_id: AccountId32 = signer_keypair.public_key().into();

    // Get signer account identity
    let (signer_name, _) = get_display_name(&crunch, &seed_account_id, None).await?;
    let mut signer_details = SignerDetails {
        account: seed_account_id.clone(),
        name: signer_name,
        warnings: Vec::new(),
    };
    debug!("signer_details {:?}", signer_details);

    // Warn if signer account is running low on funds (if lower than 2x Existential Deposit)
    let ed_addr = relay_runtime::constants().balances().existential_deposit();
    let ed = api.constants().at(&ed_addr)?;

    let seed_account_info_addr =
        relay_runtime::storage().system().account(&seed_account_id);
    if let Some(seed_account_info) = api
        .storage()
        .at_latest()
        .await?
        .fetch(&seed_account_info_addr)
        .await?
    {
        if seed_account_info.data.free
            <= (config.existential_deposit_factor_warning as u128 * ed)
        {
            signer_details
                .warnings
                .push("⚡ Signer account is running low on funds ⚡".to_string());
        }
    }

    // Try run payouts in batches
    let (mut validators, payout_summary) =
        try_run_batch_payouts(&crunch, &signer_keypair).await?;

    // Try run members in batches
    let pools_summary = try_run_batch_pool_members(&crunch, &signer_keypair).await?;

    // Get Network name
    let chain_name = crunch.rpc().system_chain().await?;

    // Try fetch ONE-T grade data
    for v in &mut validators {
        v.onet = try_fetch_onet_data(chain_name.to_lowercase(), v.stash.clone()).await?;
    }

    // Get Era index
    let active_era_addr = relay_runtime::storage().staking().active_era();
    let active_era_index = match api
        .storage()
        .at_latest()
        .await?
        .fetch(&active_era_addr)
        .await?
    {
        Some(info) => info.index,
        None => return Err(CrunchError::Other("Active era not available".into())),
    };

    let properties = crunch.rpc().system_properties().await?;

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

    let data = RawData {
        network,
        signer_details,
        validators,
        payout_summary,
        pools_summary,
    };

    let report = Report::from(data);
    crunch
        .send_message(&report.message(), &report.formatted_message())
        .await?;

    Ok(())
}

pub async fn try_run_batch_pool_members(
    crunch: &Crunch,
    signer: &Keypair,
) -> Result<NominationPoolsSummary, CrunchError> {
    let config = CONFIG.clone();
    let api = crunch.client().clone();

    let mut calls_for_batch: Vec<Call> = vec![];
    let mut summary: NominationPoolsSummary = Default::default();

    if let Some(members) = try_fetch_pool_members_for_compound(&crunch).await? {
        //
        for member in &members {
            //
            let call = Call::NominationPools(NominationPoolsCall::bond_extra_other {
                member: MultiAddress::Id(member.clone()),
                extra: BondExtra::Rewards,
            });
            calls_for_batch.push(call);
            summary.calls += 1;
        }
        summary.total_members = members.len() as u32;
    }

    if calls_for_batch.len() > 0 {
        // TODO check batch call weight or maximum_calls [default: 8]
        //
        // Calculate the number of extrinsics (iteractions) based on the maximum number of calls per batch
        // and the number of calls to be sent
        //
        let maximum_batch_calls = (calls_for_batch.len() as f32
            / config.maximum_pool_members_calls as f32)
            .ceil() as u32;
        let mut iteration = Some(0);
        while let Some(x) = iteration {
            if x == maximum_batch_calls {
                iteration = None;
            } else {
                let call_start_index: usize =
                    (x * config.maximum_pool_members_calls).try_into().unwrap();
                let call_end_index: usize = if config.maximum_pool_members_calls
                    > calls_for_batch[call_start_index..].len() as u32
                {
                    ((x * config.maximum_pool_members_calls)
                        + calls_for_batch[call_start_index..].len() as u32)
                        .try_into()
                        .unwrap()
                } else {
                    ((x * config.maximum_pool_members_calls)
                        + config.maximum_pool_members_calls)
                        .try_into()
                        .unwrap()
                };

                debug!(
                    "batch pool_members_calls indexes [{:?} : {:?}]",
                    call_start_index, call_end_index
                );

                let calls_for_batch_clipped =
                    calls_for_batch[call_start_index..call_end_index].to_vec();

                // Note: Unvalidated extrinsic. If it fails a static metadata file will need to be updated!
                let tx = relay_runtime::tx()
                    .utility()
                    .force_batch(calls_for_batch_clipped.clone())
                    .unvalidated();

                // Configure the transaction parameters by defining `tip` and `tx_mortal` as per user config;
                let tx_params = if config.tx_mortal_period > 0 {
                    // Get latest block to be submitted in tx params
                    let latest_block = api.blocks().at_latest().await?;
                    TxParams::new()
                        .tip(config.tx_tip.into())
                        .mortal(latest_block.header(), config.tx_mortal_period)
                        .build()
                } else {
                    TxParams::new().tip(config.tx_tip.into()).build()
                };

                let mut tx_progress = api
                    .tx()
                    .sign_and_submit_then_watch(&tx, signer, tx_params)
                    .await?;

                while let Some(status) = tx_progress.next().await {
                    match status? {
                        TxStatus::InFinalizedBlock(in_block) => {
                            // Get block number
                            let block_number = if let Some(header) = crunch
                                .rpc()
                                .chain_get_header(Some(in_block.block_hash()))
                                .await?
                            {
                                header.number
                            } else {
                                0
                            };

                            // Fetch events from block
                            let tx_events = in_block.fetch_events().await?;

                            // Iterate over events to calculate respective reward amounts
                            for event in tx_events.iter() {
                                let event = event?;
                                if let Some(_ev) = event.as_event::<ItemCompleted>()? {
                                    // https://polkadot.js.org/docs/substrate/events#itemcompleted
                                    // summary: A single item within a Batch of dispatches has completed with no error.
                                    //
                                    summary.calls_succeeded += 1;
                                } else if let Some(_ev) =
                                    event.as_event::<ItemFailed>()?
                                {
                                    // https://polkadot.js.org/docs/substrate/events/#itemfailedspruntimedispatcherror
                                    // summary: A single item within a Batch of dispatches has completed with error.
                                    //
                                    summary.calls_failed += 1;
                                } else if let Some(_ev) =
                                    event.as_event::<BatchCompleted>()?
                                {
                                    // https://polkadot.js.org/docs/substrate/events#batchcompleted
                                    // summary: Batch of dispatches completed fully with no error.
                                    info!(
                            "Nomination Pools Compound Batch Completed ({} calls)",
                            calls_for_batch_clipped.len()
                        );
                                    let b = Batch {
                                        block_number,
                                        extrinsic: tx_events.extrinsic_hash(),
                                    };
                                    summary.batches.push(b);
                                } else if let Some(_ev) =
                                    event.as_event::<BatchCompletedWithErrors>()?
                                {
                                    // https://polkadot.js.org/docs/substrate/events/#batchcompletedwitherrors
                                    // summary: Batch of dispatches completed but has errors.
                                    info!(
                            "Nomination Pools Compound Batch Completed with errors ({} calls)",
                            calls_for_batch_clipped.len()
                        );
                                    let b = Batch {
                                        block_number,
                                        extrinsic: tx_events.extrinsic_hash(),
                                    };
                                    summary.batches.push(b);
                                }
                            }
                        }
                        TxStatus::Error { message } => {
                            warn!("TxStatus: {message:?}");
                        }
                        TxStatus::Invalid { message } => {
                            warn!("TxStatus: {message:?}");
                        }
                        TxStatus::Dropped { message } => {
                            warn!("TxStatus: {message:?}");
                        }
                        _ => {}
                    }
                }
                iteration = Some(x + 1);
            }
        }
    }

    Ok(summary)
}

pub async fn try_run_batch_payouts(
    crunch: &Crunch,
    signer: &Keypair,
) -> Result<(Validators, PayoutSummary), CrunchError> {
    let config = CONFIG.clone();
    let api = crunch.client().clone();

    // Get Era index
    let active_era_addr = relay_runtime::storage().staking().active_era();
    let active_era_index = match api
        .storage()
        .at_latest()
        .await?
        .fetch(&active_era_addr)
        .await?
    {
        Some(info) => info.index,
        None => return Err(CrunchError::Other("Active era not available".into())),
    };

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
                    if let Some((claim_era, _page_index)) = v.unclaimed.pop() {
                        // TODO: After deprecated storage items going away we could consider
                        // using payout_stakers_by_page with the respective page_index.
                        // Until than lets just call payout_stakers x times based on
                        // the unclaimed pages previously checked.
                        //
                        // PR: https://github.com/paritytech/polkadot-sdk/pull/1189
                        //
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

    if calls_for_batch.len() > 0 {
        // TODO check batch call weight or maximum_calls [default: 4]
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
                let tx = relay_runtime::tx()
                    .utility()
                    .force_batch(calls_for_batch_clipped.clone())
                    .unvalidated();

                // Configure the transaction parameters by defining `tip` and `tx_mortal` as per user config;
                let tx_params = if config.tx_mortal_period > 0 {
                    // Get latest block to be submitted in tx params
                    let latest_block = api.blocks().at_latest().await?;
                    TxParams::new()
                        .tip(config.tx_tip.into())
                        .mortal(latest_block.header(), config.tx_mortal_period)
                        .build()
                } else {
                    TxParams::new().tip(config.tx_tip.into()).build()
                };

                let mut tx_progress = api
                    .tx()
                    .sign_and_submit_then_watch(&tx, signer, tx_params)
                    .await?;

                while let Some(status) = tx_progress.next().await {
                    match status? {
                        TxStatus::InFinalizedBlock(in_block) => {
                            // Get block number
                            let block_number = if let Some(header) = crunch
                                .rpc()
                                .chain_get_header(Some(in_block.block_hash()))
                                .await?
                            {
                                header.number
                            } else {
                                0
                            };

                            // Fetch events from block
                            let tx_events = in_block.fetch_events().await?;

                            // Iterate over events to calculate respective reward amounts
                            for event in tx_events.iter() {
                                let event = event?;
                                if let Some(_ev) = event.as_event::<ExtrinsicFailed>()? {
                                    let dispatch_error = DispatchError::decode_from(
                                        event.field_bytes(),
                                        api.metadata(),
                                    )?;
                                    return Err(dispatch_error.into());
                                } else if let Some(ev) =
                                    event.as_event::<PayoutStarted>()?
                                {
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
                                } else if let Some(_ev) =
                                    event.as_event::<ItemCompleted>()?
                                {
                                    // https://polkadot.js.org/docs/substrate/events#itemcompleted
                                    // summary: A single item within a Batch of dispatches has completed with no error.
                                    //
                                    if let Some(i) = validator_index {
                                        let validator = &mut validators[i];

                                        // NOTE: Currently we do not track which page is being payout here.
                                        // It should be changed when payout_stakers_by_page is in place
                                        validator.claimed.push((era_index, 0));
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
                                } else if let Some(_ev) =
                                    event.as_event::<ItemFailed>()?
                                {
                                    // https://polkadot.js.org/docs/substrate/events/#itemfailedspruntimedispatcherror
                                    // summary: A single item within a Batch of dispatches has completed with error.
                                    //
                                    summary.calls_failed += 1;
                                } else if let Some(_ev) =
                                    event.as_event::<BatchCompleted>()?
                                {
                                    // https://polkadot.js.org/docs/substrate/events#batchcompleted
                                    // summary: Batch of dispatches completed fully with no error.
                                    info!(
                                        "Batch Completed ({} calls)",
                                        calls_for_batch_clipped.len()
                                    );
                                } else if let Some(_ev) =
                                    event.as_event::<BatchCompletedWithErrors>()?
                                {
                                    // https://polkadot.js.org/docs/substrate/events/#batchcompletedwitherrors
                                    // summary: Batch of dispatches completed but has errors.
                                    info!(
                                        "Batch Completed with errors ({} calls)",
                                        calls_for_batch_clipped.len()
                                    );
                                } else if let Some(ev) =
                                    event.as_event::<BatchInterrupted>()?
                                {
                                    // NOTE: Deprecate with force_batch
                                    //
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
                                                warn!(
                                                    "Batch interrupted at stash: {:?}",
                                                    validator_stash
                                                );
                                                let validator_index =
                                                    &mut validators.iter().position(
                                                        |v| v.stash == *validator_stash,
                                                    );

                                                if let Some(i) = *validator_index {
                                                    let validator = &mut validators[i];
                                                    // TODO: decode DispatchError to a readable format
                                                    validator.warnings.push(
                                                        "⚡ Batch interrupted ⚡"
                                                            .to_string(),
                                                    );
                                                }
                                            }
                                            _ => unreachable!(),
                                        };
                                    }
                                }
                            }
                        }
                        TxStatus::Error { message } => {
                            warn!("TxStatus: {message:?}");
                        }
                        TxStatus::Invalid { message } => {
                            warn!("TxStatus: {message:?}");
                        }
                        TxStatus::Dropped { message } => {
                            warn!("TxStatus: {message:?}");
                        }
                        _ => {}
                    }
                }

                iteration = Some(x + 1);
            }
        }
    }

    debug!("validators {:?}", validators);

    // Prepare summary report
    summary.total_validators = validators.len() as u32;

    Ok((validators, summary))
}

async fn collect_validators_data(
    crunch: &Crunch,
    era_index: EraIndex,
) -> Result<Validators, CrunchError> {
    let api = crunch.client().clone();

    // Get unclaimed eras for the stash addresses
    let active_validators_addr = relay_runtime::storage().session().validators();
    let active_validators = api
        .storage()
        .at_latest()
        .await?
        .fetch(&active_validators_addr)
        .await?;
    debug!("active_validators {:?}", active_validators);
    let mut validators: Validators = Vec::new();

    let stashes = get_stashes(&crunch).await?;

    for (_i, stash_str) in stashes.iter().enumerate() {
        let stash = AccountId32::from_str(stash_str).map_err(|e| {
            CrunchError::Other(format!("Invalid account: {stash_str} error: {e:?}"))
        })?;

        // Check if stash has bonded controller
        let controller_addr = relay_runtime::storage().staking().bonded(&stash);
        let controller = match api
            .storage()
            .at_latest()
            .await?
            .fetch(&controller_addr)
            .await?
        {
            Some(controller) => controller,
            None => {
                let mut v = Validator::new(stash.clone());
                (v.name, v.has_identity) =
                    get_display_name(&crunch, &stash, None).await?;
                v.warnings = vec![format!("No controller bonded!")];
                validators.push(v);
                continue;
            }
        };

        // Instantiates a new validator struct
        let mut v = Validator::new(stash.clone());

        // Set controller
        v.controller = Some(controller.clone());

        // Get validator name
        (v.name, v.has_identity) = get_display_name(&crunch, &stash, None).await?;

        // Check if validator is in active set
        v.is_active = if let Some(ref av) = active_validators {
            av.contains(&stash)
        } else {
            false
        };

        // Look for unclaimed eras, starting on current_era - maximum_eras
        let start_index = get_era_index_start(&crunch, era_index).await?;

        // Get staking info from ledger
        let ledger_addr = relay_runtime::storage().staking().ledger(&controller);
        if let Some(staking_ledger) =
            api.storage().at_latest().await?.fetch(&ledger_addr).await?
        {
            debug!(
                "{} * claimed_rewards: {:?}",
                stash, staking_ledger.legacy_claimed_rewards
            );
            // deconstruct claimed rewards
            let BoundedVec(legacy_claimed_rewards) =
                staking_ledger.legacy_claimed_rewards;

            // Find unclaimed eras in previous 84 eras (reverse order)
            for e in (start_index..era_index).rev() {
                // TODO: legacy methods to be deprecated in the future
                // check https://github.com/paritytech/polkadot-sdk/pull/1189
                if legacy_claimed_rewards.contains(&e) {
                    v.claimed.push((e, 0));
                    continue;
                }

                // Verify if stash has claimed/unclaimed pages per era by cross checking eras_stakers_overview with claimed_rewards
                let claimed_rewards_addr = relay_runtime::storage()
                    .staking()
                    .claimed_rewards(&e, &stash);
                if let Some(claimed_rewards) = api
                    .storage()
                    .at_latest()
                    .await?
                    .fetch(&claimed_rewards_addr)
                    .await?
                {
                    // Verify if there are more pages to claim than the ones already claimed
                    let eras_stakers_overview_addr = relay_runtime::storage()
                        .staking()
                        .eras_stakers_overview(&e, &stash);
                    if let Some(exposure) = api
                        .storage()
                        .at_latest()
                        .await?
                        .fetch(&eras_stakers_overview_addr)
                        .await?
                    {
                        // Check if all pages are claimed or not
                        for page_index in 0..exposure.page_count {
                            if claimed_rewards.contains(&page_index) {
                                v.claimed.push((e, page_index));
                            } else {
                                v.unclaimed.push((e, page_index));
                            }
                        }
                    } else {
                        // If eras_stakers_overview is not available set all pages claimed
                        for page_index in claimed_rewards {
                            v.claimed.push((e, page_index));
                        }
                    }
                } else {
                    // Set all pages unclaimed in case there are no claimed rewards for the era and stash specified
                    let eras_stakers_paged_addr = relay_runtime::storage()
                        .staking()
                        .eras_stakers_paged_iter2(&e, &stash);
                    let mut iter = api
                        .storage()
                        .at_latest()
                        .await?
                        .iter(eras_stakers_paged_addr)
                        .await?;

                    let mut page_index = 0;
                    while let Some(Ok(_)) = iter.next().await {
                        v.unclaimed.push((e, page_index));
                        page_index += 1;
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

    let history_depth_addr = relay_runtime::constants().staking().history_depth();
    let history_depth: u32 = api.constants().at(&history_depth_addr)?;

    if era_index < cmp::min(config.maximum_history_eras, history_depth) {
        return Ok(0);
    }

    if config.is_short || config.is_medium {
        return Ok(era_index - cmp::min(config.maximum_history_eras, history_depth));
    }

    // Note: If crunch is running in verbose mode, ignore MAXIMUM_ERAS
    // since we still want to show information about inclusion and eras crunched for all history_depth
    Ok(era_index - history_depth)
}

async fn get_validator_points_info(
    crunch: &Crunch,
    era_index: EraIndex,
    stash: &AccountId32,
) -> Result<Points, CrunchError> {
    let api = crunch.client().clone();
    // Get era reward points
    let era_reward_points_addr = relay_runtime::storage()
        .staking()
        .eras_reward_points(&era_index);

    if let Some(era_reward_points) = api
        .storage()
        .at_latest()
        .await?
        .fetch(&era_reward_points_addr)
        .await?
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
) -> Result<(String, bool), CrunchError> {
    let api = crunch.people_client().clone();

    let identity_of_addr = people_runtime::storage().identity().identity_of(stash);
    match api
        .storage()
        .at_latest()
        .await?
        .fetch(&identity_of_addr)
        .await?
    {
        Some((identity, _)) => {
            debug!("identity {:?}", identity);
            let parent = parse_identity_data(identity.info.display);
            let name = match sub_account_name {
                Some(child) => format!("{}/{}", parent, child),
                None => parent,
            };
            Ok((name, true))
        }
        None => {
            let super_of_addr = people_runtime::storage().identity().super_of(stash);
            if let Some((parent_account, data)) = api
                .storage()
                .at_latest()
                .await?
                .fetch(&super_of_addr)
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
                Ok((format!("{}...{}", &s[..6], &s[s.len() - 6..]), false))
            }
        }
    }
}

//
fn parse_identity_data(
    data: people_runtime::runtime_types::pallet_identity::types::Data,
) -> String {
    match data {
        people_runtime::runtime_types::pallet_identity::types::Data::Raw0(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw1(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw2(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw3(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw4(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw5(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw6(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw7(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw8(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw9(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw10(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw11(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw12(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw13(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw14(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw15(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw16(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw17(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw18(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw19(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw20(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw21(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw22(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw23(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw24(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw25(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw26(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw27(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw28(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw29(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw30(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw31(bytes) => {
            str(bytes.to_vec())
        }
        people_runtime::runtime_types::pallet_identity::types::Data::Raw32(bytes) => {
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

    let history_depth_addr = relay_runtime::constants().staking().history_depth();
    let history_depth: u32 = api.constants().at(&history_depth_addr)?;

    let active_era_addr = relay_runtime::storage().staking().active_era();
    let active_era_index = match api
        .storage()
        .at_latest()
        .await?
        .fetch(&active_era_addr)
        .await?
    {
        Some(info) => info.index,
        None => return Err(CrunchError::Other("Active era not available".into())),
    };

    // try_fetch_pool_members_for_compound(&crunch).await?;

    for stash_str in stashes.iter() {
        let stash = AccountId32::from_str(stash_str).map_err(|e| {
            CrunchError::Other(format!("Invalid account: {stash_str} error: {e:?}"))
        })?;
        // fetch identity
        let (identity, has_identity) = get_display_name(&crunch, &stash, None).await?;
        if has_identity {
            info!("{} * Stash account for {}", stash, identity);
        } else {
            info!("{} * Stash account", stash);
        }

        let start_index = active_era_index - history_depth;
        let mut unclaimed: Vec<(EraIndex, PageIndex)> = Vec::new();
        let mut claimed: Vec<(EraIndex, PageIndex)> = Vec::new();

        let bonded_addr = relay_runtime::storage().staking().bonded(&stash);
        if let Some(controller) =
            api.storage().at_latest().await?.fetch(&bonded_addr).await?
        {
            let ledger_addr = relay_runtime::storage().staking().ledger(&controller);
            if let Some(ledger_response) =
                api.storage().at_latest().await?.fetch(&ledger_addr).await?
            {
                // deconstruct claimed rewards
                let BoundedVec(legacy_claimed_rewards) =
                    ledger_response.legacy_claimed_rewards;

                // Find unclaimed eras in previous 84 eras
                for era_index in start_index..active_era_index {
                    // TODO: legacy methods to be deprecated in the future
                    // check https://github.com/paritytech/polkadot-sdk/pull/1189
                    if legacy_claimed_rewards.contains(&era_index) {
                        claimed.push((era_index, 0));
                        continue;
                    }

                    // Verify if stash has claimed/unclaimed pages per era by cross checking eras_stakers_overview with claimed_rewards
                    let claimed_rewards_addr = relay_runtime::storage()
                        .staking()
                        .claimed_rewards(&era_index, &stash);
                    if let Some(claimed_rewards) = api
                        .storage()
                        .at_latest()
                        .await?
                        .fetch(&claimed_rewards_addr)
                        .await?
                    {
                        // Verify if there are more pages to claim than the ones already claimed
                        let eras_stakers_overview_addr = relay_runtime::storage()
                            .staking()
                            .eras_stakers_overview(&era_index, &stash);
                        if let Some(exposure) = api
                            .storage()
                            .at_latest()
                            .await?
                            .fetch(&eras_stakers_overview_addr)
                            .await?
                        {
                            // Check if all pages are claimed or not
                            for page_index in 0..exposure.page_count {
                                if claimed_rewards.contains(&page_index) {
                                    claimed.push((era_index, page_index));
                                } else {
                                    unclaimed.push((era_index, page_index));
                                }
                            }
                        } else {
                            // If eras_stakers_overview is not available set all pages claimed
                            for page_index in claimed_rewards {
                                claimed.push((era_index, page_index));
                            }
                        }
                    } else {
                        // Set all pages unclaimed in case there are no claimed rewards for the era and stash specified
                        let eras_stakers_paged_addr = relay_runtime::storage()
                            .staking()
                            .eras_stakers_paged_iter2(&era_index, &stash);
                        let mut iter = api
                            .storage()
                            .at_latest()
                            .await?
                            .iter(eras_stakers_paged_addr)
                            .await?;

                        let mut page_index = 0;
                        while let Some(Ok(_)) = iter.next().await {
                            unclaimed.push((era_index, page_index));
                            page_index += 1;
                        }
                    }
                }
            }
        }
        info!(
            "{} claimed pages in the last {} eras -> {:?}",
            claimed.len(),
            history_depth,
            claimed
        );
        info!(
            "{} unclaimed pages in the last {} eras -> {:?}",
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

pub async fn try_fetch_pool_operators_for_compound(
    crunch: &Crunch,
) -> Result<Option<Vec<AccountId32>>, CrunchError> {
    let config = CONFIG.clone();

    if config.pool_ids.len() == 0 && !config.pool_only_operator_compound_enabled {
        return Ok(None);
    }

    let api = crunch.client().clone();

    let mut members: Vec<AccountId32> = Vec::new();

    for pool_id in &config.pool_ids {
        let bonded_pool_addr = relay_runtime::storage()
            .nomination_pools()
            .bonded_pools(pool_id);
        if let Some(pool) = api
            .storage()
            .at_latest()
            .await?
            .fetch(&bonded_pool_addr)
            .await?
        {
            let permissions_addr = relay_runtime::storage()
                .nomination_pools()
                .claim_permissions(pool.roles.depositor.clone());

            if let Some(permissions) = api
                .storage()
                .at_latest()
                .await?
                .fetch(&permissions_addr)
                .await?
            {
                if [
                    ClaimPermission::PermissionlessCompound,
                    ClaimPermission::PermissionlessAll,
                ]
                .contains(&permissions)
                {
                    // fetch pending rewards
                    let call_name = format!("NominationPoolsApi_pending_rewards");
                    let bytes = crunch
                        .rpc()
                        .state_call(
                            &call_name,
                            Some(&pool.roles.depositor.clone().encode()),
                            None,
                        )
                        .await?;

                    let claimable: u128 = Decode::decode(&mut &*bytes)?;

                    if claimable > config.pool_compound_threshold.into() {
                        members.push(pool.roles.depositor.clone());
                    }
                }
            }
        }
    }
    Ok(Some(members))
}

pub async fn try_fetch_pool_members_for_compound(
    crunch: &Crunch,
) -> Result<Option<Vec<AccountId32>>, CrunchError> {
    let config = CONFIG.clone();
    if config.pool_ids.len() == 0
        && !config.pool_only_operator_compound_enabled
        && !config.pool_members_compound_enabled
    {
        return Ok(None);
    }

    if config.pool_only_operator_compound_enabled {
        return try_fetch_pool_operators_for_compound(&crunch).await;
    }

    let api = crunch.client().clone();

    let mut members: Vec<AccountId32> = Vec::new();

    // 1. get all members with permissions set as [PermissionlessCompound, PermissionlessAll]
    let permissions_addr = relay_runtime::storage()
        .nomination_pools()
        .claim_permissions_iter();

    let mut iter = api
        .storage()
        .at_latest()
        .await?
        .iter(permissions_addr)
        .await?;

    while let Some(Ok(storage)) = iter.next().await {
        if [
            ClaimPermission::PermissionlessCompound,
            ClaimPermission::PermissionlessAll,
        ]
        .contains(&storage.value)
        {
            let member = get_account_id_from_storage_key(storage.key_bytes);
            // debug!("member: {}", member);

            // 2 .Verify if member belongs to the pools configured
            let pool_member_addr = relay_runtime::storage()
                .nomination_pools()
                .pool_members(&member);
            if let Some(pool_member) = api
                .storage()
                .at_latest()
                .await?
                .fetch(&pool_member_addr)
                .await?
            {
                if config.pool_ids.contains(&pool_member.pool_id) {
                    // fetch pending rewards
                    let call_name = format!("NominationPoolsApi_pending_rewards");
                    let bytes = crunch
                        .rpc()
                        .state_call(&call_name, Some(&member.encode()), None)
                        .await?;

                    let claimable: u128 = Decode::decode(&mut &*bytes)?;

                    if claimable > config.pool_compound_threshold.into() {
                        members.push(member);
                    }
                }
            }
        }
    }

    Ok(Some(members))
}

pub async fn try_fetch_stashes_from_pool_ids(
    crunch: &Crunch,
) -> Result<Option<Vec<String>>, CrunchError> {
    let api = crunch.client().clone();
    let config = CONFIG.clone();
    if config.pool_ids.len() == 0
        || (!config.pool_active_nominees_payout_enabled
            && !config.pool_all_nominees_payout_enabled)
    {
        return Ok(None);
    }

    let active_era_addr = relay_runtime::storage().staking().active_era();
    let era_index = match api
        .storage()
        .at_latest()
        .await?
        .fetch(&active_era_addr)
        .await?
    {
        Some(info) => info.index,
        None => return Err("Active era not defined".into()),
    };

    let mut all: Vec<String> = Vec::new();
    let mut active: Vec<String> = Vec::new();

    for pool_id in config.pool_ids.iter() {
        let pool_stash_account = nomination_pool_account(AccountType::Bonded, *pool_id);
        let nominators_addr = relay_runtime::storage()
            .staking()
            .nominators(&pool_stash_account);
        if let Some(nominations) = api
            .storage()
            .at_latest()
            .await?
            .fetch(&nominators_addr)
            .await?
        {
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
                let eras_stakers_addr = relay_runtime::storage()
                    .staking()
                    .eras_stakers(era_index - 1, &stash);
                if let Some(exposure) = api
                    .storage()
                    .at_latest()
                    .await?
                    .fetch(&eras_stakers_addr)
                    .await?
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

    if config.pool_all_nominees_payout_enabled {
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
