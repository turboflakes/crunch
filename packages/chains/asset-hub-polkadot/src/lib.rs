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

use async_recursion::async_recursion;
use crunch_config::CONFIG;
use crunch_core::{
    get_account_id_from_storage_key, to_hex, try_fetch_stashes_from_remote_url, Crunch,
    NominatorsAmount, ValidatorAmount, ValidatorIndex,
};
use crunch_error::CrunchError;
use crunch_people_polkadot::get_display_name;
use crunch_pools::{nomination_pool_account, AccountType};
use crunch_report::{
    Batch, EraIndex, NominationPoolCommission, NominationPoolsSummary, PageIndex, Payout,
    PayoutSummary, Points, SignerDetails, Validator, Validators,
};
use log::{debug, info, warn};
use std::{
    cmp,
    convert::{TryFrom, TryInto},
    result::Result,
    str::FromStr,
};
use subxt::{
    config::polkadot::PolkadotExtrinsicParamsBuilder as TxParams,
    error::DispatchError,
    tx::TxStatus,
    utils::{AccountId32, MultiAddress},
};
use subxt_signer::sr25519::Keypair;

#[subxt::subxt(
    runtime_metadata_path = "metadata/asset_hub_polkadot_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod ah_metadata {}

use ah_metadata::{
    nomination_pools::events::PoolCommissionClaimed,
    runtime_types::{
        asset_hub_polkadot_runtime::OriginCaller,
        bounded_collections::{
            bounded_btree_map::BoundedBTreeMap, bounded_vec::BoundedVec,
            weak_bounded_vec::WeakBoundedVec,
        },
        frame_support::dispatch::RawOrigin,
        pallet_nomination_pools::{BondExtra, ClaimPermission},
        xcm_runtime_apis::dry_run::CallDryRunEffects,
    },
    staking::events::{PayoutStarted, Rewarded},
    system::events::ExtrinsicFailed,
    utility::{
        calls::types::with_weight::Weight,
        events::{
            BatchCompleted, BatchCompletedWithErrors, BatchInterrupted, ItemCompleted,
            ItemFailed,
        },
    },
};

type Call = ah_metadata::runtime_types::asset_hub_polkadot_runtime::RuntimeCall;
type StakingCall = ah_metadata::runtime_types::pallet_staking_async::pallet::pallet::Call;
type NominationPoolsCall =
    ah_metadata::runtime_types::pallet_nomination_pools::pallet::Call;

pub async fn try_run_batch_pool_members(
    crunch: &Crunch,
    signer: &Keypair,
) -> Result<NominationPoolsSummary, CrunchError> {
    let config = CONFIG.clone();
    let ah_api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");

    let ah_rpc = crunch
        .asset_hub_rpc()
        .as_ref()
        .expect("AH Legacy API to be available");

    let mut calls_for_batch: Vec<Call> = vec![];
    let mut summary: NominationPoolsSummary = Default::default();

    // Fetch pool members and add member rewards calls to the batch
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

    // Add claim commission calls if enabled and pool ids are set
    if config.pool_claim_commission_enabled {
        for pool_id in config.pool_ids.clone() {
            let call = Call::NominationPools(NominationPoolsCall::claim_commission {
                pool_id: pool_id.clone(),
            });
            calls_for_batch.push(call);
            summary.calls += 1;
        }
    }

    if calls_for_batch.len() > 0 {
        // TODO check batch call weight or maximum_calls [default: 8]
        //
        // Calculate the number of extrinsics (iteractions) based on the maximum number of calls per batch
        // and the number of calls to be sent
        //
        let maximum_batch_calls = (calls_for_batch.len() as f32
            / config.maximum_pool_calls as f32)
            .ceil() as u32;
        let mut iteration = Some(0);
        while let Some(x) = iteration {
            if x == maximum_batch_calls {
                iteration = None;
            } else {
                let call_start_index: usize =
                    (x * config.maximum_pool_calls).try_into().unwrap();
                let call_end_index: usize = if config.maximum_pool_calls
                    > calls_for_batch[call_start_index..].len() as u32
                {
                    ((x * config.maximum_pool_calls)
                        + calls_for_batch[call_start_index..].len() as u32)
                        .try_into()
                        .unwrap()
                } else {
                    ((x * config.maximum_pool_calls) + config.maximum_pool_calls)
                        .try_into()
                        .unwrap()
                };

                debug!(
                    "batch pool_calls indexes [{:?} : {:?}]",
                    call_start_index, call_end_index
                );

                let calls_for_batch_clipped =
                    calls_for_batch[call_start_index..call_end_index].to_vec();

                // Note: Unvalidated extrinsic. If it fails a static metadata file will need to be updated!
                let tx = ah_metadata::tx()
                    .utility()
                    .force_batch(calls_for_batch_clipped.clone())
                    .unvalidated();

                // Configure the transaction parameters by defining `tip` and `tx_mortal` as per user config;
                let tx_params = if config.tx_mortal_period > 0 {
                    TxParams::new()
                        .tip(config.tx_tip.into())
                        .mortal(config.tx_mortal_period)
                        .build()
                } else {
                    TxParams::new().tip(config.tx_tip.into()).build()
                };

                // Log call data in debug mode
                if config.is_debug {
                    let call_data = ah_api.tx().call_data(&tx)?;
                    let hex_call_data = to_hex(&call_data);
                    debug!("call_data: {hex_call_data}");
                }

                let mut tx_progress = ah_api
                    .tx()
                    .sign_and_submit_then_watch(&tx, signer, tx_params)
                    .await?;

                while let Some(status) = tx_progress.next().await {
                    match status? {
                        TxStatus::InFinalizedBlock(in_block) => {
                            // Get block number
                            let block_number = if let Some(header) = ah_rpc
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
                                } else if let Some(ev) =
                                    event.as_event::<PoolCommissionClaimed>()?
                                {
                                    let p = NominationPoolCommission {
                                        pool_id: ev.pool_id,
                                        commission: ev.commission,
                                    };
                                    summary.pool_commissions.push(p);
                                } else if let Some(_ev) =
                                    event.as_event::<BatchCompleted>()?
                                {
                                    // https://polkadot.js.org/docs/substrate/events#batchcompleted
                                    // summary: Batch of dispatches completed fully with no error.
                                    info!(
                                        "Nomination Pools Batch Completed ({} calls)",
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
                            "Nomination Pools Batch Completed with errors ({} calls)",
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
    validators: &mut Validators,
) -> Result<PayoutSummary, CrunchError> {
    let ah_api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");

    // Get block weights
    let block_weights_addr = ah_metadata::constants().system().block_weights();
    let block_weights = ah_api.constants().at(&block_weights_addr)?;

    let max_extrinsic_weight = block_weights
        .per_class
        .normal
        .max_extrinsic
        .expect("Max extrinsic weights not found.");

    debug!("Max extrinsic weight: {:?}", max_extrinsic_weight);

    // Get Existential Deposit
    let ed_addr = ah_metadata::constants().balances().existential_deposit();
    let existencial_deposit = ah_api.constants().at(&ed_addr)?;

    // let mut validators = collect_validators_data(&crunch, active_era_index).await?;
    let mut summary: PayoutSummary = Default::default();

    // Add unclaimed eras into payout staker calls
    let mut calls_for_batch: Vec<Call> = build_calls_for_batch(validators, &mut summary)?;

    let mut iteration = Some(1);
    while let Some(x) = iteration {
        debug!("try_run_batch_payouts: {} {}", x, calls_for_batch.len());

        // Fetch signer free balance
        let signer_addr = ah_metadata::storage()
            .system()
            .account(signer.public_key().into());
        let available_balance = if let Some(signer_info) = ah_api
            .storage()
            .at_latest()
            .await?
            .fetch(&signer_addr)
            .await?
        {
            signer_info.data.free
        } else {
            0
        };

        //
        // validate_calls_for_batch
        //
        let (valid_calls, pending_calls) = validate_calls_for_batch(
            crunch,
            signer,
            calls_for_batch.clone(),
            available_balance,
            existencial_deposit,
            max_extrinsic_weight.clone(),
            None,
        )
        .await?;

        //
        // sign_and_submit_maximum_calls
        //
        if valid_calls.len() > 0 {
            sign_and_submit_maximum_calls(
                crunch,
                signer,
                valid_calls,
                validators,
                &mut summary,
            )
            .await?;
        }

        if let Some(next_calls) = pending_calls {
            calls_for_batch = next_calls;
            iteration = Some(x + 1);
        } else {
            iteration = None;
        }
    }

    debug!("validators {:?}", validators);

    // Prepare summary report
    summary.total_validators = validators.len() as u32;

    Ok(summary)
}

pub async fn sign_and_submit_maximum_calls(
    crunch: &Crunch,
    signer: &Keypair,
    calls: Vec<Call>,
    validators: &mut Validators,
    summary: &mut PayoutSummary,
) -> Result<(), CrunchError> {
    let config = CONFIG.clone();
    let ah_api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");

    let ah_rpc = crunch
        .asset_hub_rpc()
        .as_ref()
        .expect("AH Legacy API to be available");

    // Note: Unvalidated extrinsic. If it fails a static metadata file will need to be updated!
    let tx: subxt::tx::DefaultPayload<ah_metadata::utility::calls::types::ForceBatch> =
        ah_metadata::tx()
            .utility()
            .force_batch(calls.clone())
            .unvalidated();

    // Configure the transaction parameters by defining `tip` and `tx_mortal` as per user config;
    let tx_params = if config.tx_mortal_period > 0 {
        TxParams::new()
            .tip(config.tx_tip.into())
            .mortal(config.tx_mortal_period)
            .build()
    } else {
        TxParams::new().tip(config.tx_tip.into()).build()
    };

    // Log call data in debug mode
    if config.is_debug {
        let call_data = ah_api.tx().call_data(&tx)?;
        let hex_call_data = to_hex(&call_data);
        debug!("call_data: {hex_call_data}");
    }

    let mut tx_progress = ah_api
        .tx()
        .sign_and_submit_then_watch(&tx, signer, tx_params)
        .await?;

    let mut validator_index: ValidatorIndex = None;
    let mut era_index: EraIndex = 0;
    let mut validator_amount_value: ValidatorAmount = 0;
    let mut nominators_amount_value: NominatorsAmount = 0;
    let mut nominators_quantity = 0;

    while let Some(status) = tx_progress.next().await {
        match status? {
            TxStatus::InFinalizedBlock(in_block) => {
                // Get block number
                let block_number = if let Some(header) =
                    ah_rpc.chain_get_header(Some(in_block.block_hash())).await?
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
                            ah_api.metadata(),
                        )?;
                        return Err(dispatch_error.into());
                    } else if let Some(ev) = event.as_event::<PayoutStarted>()? {
                        // https://polkadot.js.org/docs/substrate/events#payoutstartedu32-accountid32
                        // PayoutStarted(u32, AccountId32)
                        // summary: The stakers' rewards are getting paid. [era_index, validator_stash]
                        //
                        debug!("{:?}", ev);
                        let validator_index_ref = validators
                            .iter()
                            .position(|v| v.stash == ev.validator_stash);
                        era_index = ev.era_index;
                        validator_index = validator_index_ref;
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
                    } else if let Some(_ev) = event.as_event::<ItemFailed>()? {
                        // https://polkadot.js.org/docs/substrate/events/#itemfailedspruntimedispatcherror
                        // summary: A single item within a Batch of dispatches has completed with error.
                        //
                        summary.calls_failed += 1;
                    } else if let Some(_ev) = event.as_event::<BatchCompleted>()? {
                        // https://polkadot.js.org/docs/substrate/events#batchcompleted
                        // summary: Batch of dispatches completed fully with no error.
                        info!("Batch Completed ({} calls)", calls.len());
                    } else if let Some(_ev) =
                        event.as_event::<BatchCompletedWithErrors>()?
                    {
                        // https://polkadot.js.org/docs/substrate/events/#batchcompletedwitherrors
                        // summary: Batch of dispatches completed but has errors.
                        info!("Batch Completed with errors ({} calls)", calls.len());
                    } else if let Some(ev) = event.as_event::<BatchInterrupted>()? {
                        // NOTE: Deprecate with force_batch
                        //
                        // https://polkadot.js.org/docs/substrate/events#batchinterruptedu32-spruntimedispatcherror
                        // summary: Batch of dispatches did not complete fully. Index of first failing dispatch given, as well as the error.
                        //
                        // Fix: https://github.com/turboflakes/crunch/issues/4
                        // Most likely the batch was interrupted because of an AlreadyClaimed era
                        // BatchInterrupted { index: 0, error: Module { index: 6, error: 14 } }
                        warn!("{:?}", ev);
                        if let Call::Staking(call) =
                            &calls[usize::try_from(ev.index).unwrap()]
                        {
                            match &call {
                                StakingCall::payout_stakers {
                                    validator_stash, ..
                                } => {
                                    warn!(
                                        "Batch interrupted at stash: {:?}",
                                        validator_stash
                                    );
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

    Ok(())
}

fn build_calls_for_batch(
    validators: &mut Validators,
    summary: &mut PayoutSummary,
) -> Result<Vec<Call>, CrunchError> {
    let config = CONFIG.clone();
    // Add unclaimed eras into payout staker calls
    let mut calls_for_batch: Vec<Call> = vec![];

    for v in validators.into_iter() {
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
    Ok(calls_for_batch)
}

#[async_recursion]
pub async fn validate_calls_for_batch(
    crunch: &Crunch,
    signer: &Keypair,
    calls: Vec<Call>,
    available_balance: u128,
    existencial_deposit: u128,
    max_weight: Weight,
    pending_calls: Option<Vec<Call>>,
) -> Result<(Vec<Call>, Option<Vec<Call>>), CrunchError> {
    type UtilityCall = ah_metadata::runtime_types::pallet_utility::pallet::Call;
    let batch_call = Call::Utility(UtilityCall::force_batch {
        calls: calls.clone(),
    });

    debug!("validate number of calls: {:?}", calls.len());

    match validate_call_via_tx_payment(
        &crunch,
        batch_call.clone(),
        available_balance,
        existencial_deposit,
        max_weight.clone(),
    )
    .await
    {
        Ok(_) => {
            if let Some(ref pending) = pending_calls {
                info!(
                    "Batch validated with {} calls successfully. Pending calls: {}",
                    calls.len(),
                    pending.len()
                );
            } else {
                info!("Batch validated with {} calls successfully", calls.len());
            }

            return Ok((calls.clone(), pending_calls));
        }
        Err(err) => match err {
            CrunchError::MaxWeightExceeded(e) => {
                debug!(
                    "Batch with {} calls got weight exceeded: {}",
                    calls.len(),
                    e
                );
                if calls.len() > 1 {
                    let new_calls = calls[..calls.len() - 1].to_vec();
                    let last_call = calls[calls.len() - 1..].to_vec();
                    let pending_calls = if let Some(mut pending) = pending_calls {
                        pending.extend(last_call);
                        Some(pending)
                    } else {
                        Some(last_call)
                    };

                    return validate_calls_for_batch(
                        crunch,
                        signer,
                        new_calls,
                        available_balance,
                        existencial_deposit,
                        max_weight.clone(),
                        pending_calls,
                    )
                    .await;
                }
                // NOTE: If there's only one call left, we can't split it further.
                // This should never happen, as a single payout should always be able to fit with the extrinsic weight limit.
                return Err(CrunchError::MaxWeightExceededForOneExtrinsic);
            }
            _ => {
                return Err(CrunchError::DryRunError(format!("{:?}", err)));
            }
        },
    }
}

async fn validate_call_via_tx_payment(
    crunch: &Crunch,
    call: Call,
    available_balance: u128,
    existencial_deposit: u128,
    max_weight: Weight,
) -> Result<(), CrunchError> {
    let ah_api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");

    let runtime_api_call = ah_metadata::apis()
        .transaction_payment_call_api()
        .query_call_info(call, 0);

    let result = ah_api
        .runtime_api()
        .at_latest()
        .await?
        .call(runtime_api_call)
        .await?;

    debug!("tx payment info: {:?}", result);

    if result.weight.ref_time > max_weight.ref_time
        || result.weight.proof_size > max_weight.proof_size
    {
        return Err(CrunchError::MaxWeightExceeded(format!(
            "Actual weight ({:?}) exceeds maximum weight ({:?})",
            result.weight, max_weight
        )));
    }

    if available_balance < result.partial_fee + existencial_deposit {
        return Err(CrunchError::InsufficientBalance(format!(
            "Available balance ({}) is less than fees ({} plancks) + existential deposit ({} plancks)",
            available_balance, result.partial_fee, existencial_deposit
        )));
    }

    Ok(())
}

async fn _validate_call_via_dry_run(
    crunch: &Crunch,
    signer: &Keypair,
    call: Call,
    max_weight: Weight,
) -> Result<(), CrunchError> {
    let api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");

    let origin: OriginCaller =
        OriginCaller::system(RawOrigin::Signed(signer.public_key().into()));

    let runtime_api_call = ah_metadata::apis()
        .dry_run_api()
        .dry_run_call(origin, call, 0);

    let result = api
        .runtime_api()
        .at_latest()
        .await?
        .call(runtime_api_call)
        .await?;

    match result {
        Ok(CallDryRunEffects {
            execution_result, ..
        }) => match execution_result {
            Ok(post_dispatch_info) => {
                debug!("Post dispatch info: {:?}", post_dispatch_info);
                if let Some(actual_weight) = post_dispatch_info.actual_weight {
                    if actual_weight.ref_time > max_weight.ref_time
                        || actual_weight.proof_size > max_weight.proof_size
                    {
                        return Err(CrunchError::MaxWeightExceeded(format!(
                            "Actual weight ({:?}) exceeds maximum weight ({:?})",
                            actual_weight, max_weight
                        )));
                    }
                }
            }
            Err(err) => {
                return Err(CrunchError::DryRunError(format!("{:?}", err.error)));
            }
        },
        Err(err) => {
            return Err(CrunchError::DryRunError(format!("{:?}", err)));
        }
    }

    Ok(())
}

pub async fn fetch_controller(
    crunch: &Crunch,
    stash: &AccountId32,
    validators: &mut Validators,
) -> Result<Option<AccountId32>, CrunchError> {
    let ah_api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");
    // Check if stash has bonded controller
    let controller_addr = ah_metadata::storage().staking().bonded(stash.clone());
    match ah_api
        .storage()
        .at_latest()
        .await?
        .fetch(&controller_addr)
        .await?
    {
        Some(ctrl) => Ok(Some(ctrl)),
        None => {
            let mut v = Validator::new(stash.clone());
            (v.name, v.parent_identity, v.has_identity) =
                get_display_name(&crunch, &stash, None).await?;
            v.warnings = vec![format!("No controller bonded!")];
            validators.push(v);
            Ok(None)
        }
    }
}

pub async fn get_era_index_start(
    crunch: &Crunch,
    era_index: EraIndex,
) -> Result<EraIndex, CrunchError> {
    let api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");
    let config = CONFIG.clone();

    let history_depth_addr = ah_metadata::constants().staking().history_depth();
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

pub async fn fetch_claimed_or_unclaimed_pages_per_era(
    crunch: &Crunch,
    stash: &AccountId32,
    era: EraIndex,
    validator: &mut Validator,
) -> Result<(), CrunchError> {
    let ah_api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");
    // Verify if stash has claimed/unclaimed pages per era by cross checking eras_stakers_overview with claimed_rewards
    let claimed_rewards_addr = ah_metadata::storage()
        .staking()
        .claimed_rewards(era, stash.clone());
    if let Some(WeakBoundedVec(claimed_rewards)) = ah_api
        .storage()
        .at_latest()
        .await?
        .fetch(&claimed_rewards_addr)
        .await?
    {
        // Verify if there are more pages to claim than the ones already claimed
        let eras_stakers_overview_addr = ah_metadata::storage()
            .staking()
            .eras_stakers_overview(era, stash.clone());
        if let Some(exposure) = ah_api
            .storage()
            .at_latest()
            .await?
            .fetch(&eras_stakers_overview_addr)
            .await?
        {
            // Check if all pages are claimed or not
            for page_index in 0..exposure.page_count {
                if claimed_rewards.contains(&page_index) {
                    validator.claimed.push((era, page_index));
                } else {
                    validator.unclaimed.push((era, page_index));
                }
            }
        } else {
            // If eras_stakers_overview is not available set all pages claimed
            for page_index in claimed_rewards {
                validator.claimed.push((era, page_index));
            }
        }
    } else {
        // Set all pages unclaimed in case there are no claimed rewards for the era and stash specified
        let eras_stakers_paged_addr = ah_metadata::storage()
            .staking()
            .eras_stakers_paged_iter2(era, stash.clone());
        let mut iter = ah_api
            .storage()
            .at_latest()
            .await?
            .iter(eras_stakers_paged_addr)
            .await?;

        let mut page_index = 0;
        while let Some(Ok(_)) = iter.next().await {
            validator.unclaimed.push((era, page_index));
            page_index += 1;
        }
    }
    Ok(())
}

pub async fn fetch_active_era_index(crunch: &Crunch) -> Result<u32, CrunchError> {
    let ah_api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");
    let active_era_addr = ah_metadata::storage().staking().active_era();
    match ah_api
        .storage()
        .at_latest()
        .await?
        .fetch(&active_era_addr)
        .await?
    {
        Some(info) => Ok(info.index),
        None => return Err(CrunchError::Other("Active era not available".into())),
    }
}

async fn get_validator_points_info(
    crunch: &Crunch,
    era_index: EraIndex,
    stash: &AccountId32,
) -> Result<Points, CrunchError> {
    let api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");
    // Get era reward points
    let era_reward_points_addr = ah_metadata::storage()
        .staking()
        .eras_reward_points(era_index);

    if let Some(era_reward_points) = api
        .storage()
        .at_latest()
        .await?
        .fetch(&era_reward_points_addr)
        .await?
    {
        let BoundedBTreeMap(individual) = era_reward_points.individual;

        let stash_points = match individual.iter().find(|(s, _)| *s == *stash) {
            Some((_, p)) => *p,
            None => 0,
        };

        // Calculate average points
        let mut points: Vec<u32> =
            individual.into_iter().map(|(_, points)| points).collect();

        let points_f64: Vec<f64> = points.iter().map(|points| *points as f64).collect();

        let points = Points {
            validator: stash_points,
            era_avg: crunch_stats::mean(&points_f64),
            ci99_9_interval: crunch_stats::confidence_interval_99_9(&points_f64),
            outlier_limits: crunch_stats::iqr_interval(&mut points),
        };

        Ok(points)
    } else {
        Ok(Points::default())
    }
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

    if config.unique_stashes_enabled || config.group_identity_enabled {
        // sort and remove duplicates
        stashes.sort();
        stashes.dedup();
    }

    Ok(stashes)
}

pub async fn get_signer_details(
    crunch: &Crunch,
    seed_account_id: &AccountId32,
) -> Result<SignerDetails, CrunchError> {
    let config = CONFIG.clone();
    let api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");

    // Get signer account identity
    let (signer_name, _, _) = get_display_name(&crunch, &seed_account_id, None).await?;
    let mut signer_details = SignerDetails {
        account: seed_account_id.clone(),
        name: signer_name,
        warnings: Vec::new(),
    };
    debug!("signer_details {:?}", signer_details);

    // Warn if signer account is running low on funds (if lower than 2x Existential Deposit)
    let ed_addr = ah_metadata::constants().balances().existential_deposit();
    let ed = api.constants().at(&ed_addr)?;

    let seed_account_info_addr = ah_metadata::storage()
        .system()
        .account(seed_account_id.clone());
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
            let warning = "⚡ Signer account is running low on funds ⚡";
            signer_details.warnings.push(warning.to_string());
            warn!("{warning}");
        }
        info!(
            "Signer {} has {:?} free plancks",
            seed_account_id.to_string(),
            seed_account_info.data.free
        );
    } else {
        let rpc = crunch.rpc().clone();
        let chain_name = rpc.system_chain().await?;
        warn!(
            "Signer {} not found on the {chain_name} network!",
            seed_account_id.to_string(),
        );
    }

    Ok(signer_details)
}

pub async fn try_fetch_pool_operators_for_compound(
    crunch: &Crunch,
) -> Result<Option<Vec<AccountId32>>, CrunchError> {
    let config = CONFIG.clone();

    if config.pool_ids.len() == 0 && !config.pool_only_operator_compound_enabled {
        return Ok(None);
    }

    let api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");

    let mut members: Vec<AccountId32> = Vec::new();

    for pool_id in &config.pool_ids {
        let bonded_pool_addr = ah_metadata::storage()
            .nomination_pools()
            .bonded_pools(*pool_id);
        if let Some(pool) = api
            .storage()
            .at_latest()
            .await?
            .fetch(&bonded_pool_addr)
            .await?
        {
            let permissions_addr = ah_metadata::storage()
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
                    // fetch pool owner pending rewards
                    let runtime_api_call = ah_metadata::apis()
                        .nomination_pools_api()
                        .pending_rewards(pool.roles.depositor.clone());

                    let claimable = api
                        .runtime_api()
                        .at_latest()
                        .await?
                        .call(runtime_api_call)
                        .await?;

                    if claimable > config.pool_compound_threshold as u128 {
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

    let api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");

    let mut members: Vec<AccountId32> = Vec::new();

    // 1. get all members with permissions set as [PermissionlessCompound, PermissionlessAll]
    let permissions_addr = ah_metadata::storage()
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
            let pool_member_addr = ah_metadata::storage()
                .nomination_pools()
                .pool_members(member.clone());
            if let Some(pool_member) = api
                .storage()
                .at_latest()
                .await?
                .fetch(&pool_member_addr)
                .await?
            {
                if config.pool_ids.contains(&pool_member.pool_id) {
                    // fetch pool member pending rewards
                    let runtime_api_call = ah_metadata::apis()
                        .nomination_pools_api()
                        .pending_rewards(member.clone());

                    let claimable = api
                        .runtime_api()
                        .at_latest()
                        .await?
                        .call(runtime_api_call)
                        .await?;

                    if claimable > config.pool_compound_threshold as u128 {
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
    let config = CONFIG.clone();
    if config.pool_ids.len() == 0
        || (!config.pool_active_nominees_payout_enabled
            && !config.pool_all_nominees_payout_enabled)
    {
        return Ok(None);
    }

    let api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");

    let active_era_addr = ah_metadata::storage().staking().active_era();
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
        let nominators_addr = ah_metadata::storage()
            .staking()
            .nominators(pool_stash_account.clone());
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
                let eras_stakers_paged_addr = ah_metadata::storage()
                    .staking()
                    .eras_stakers_paged_iter2(era_index - 1, stash.clone());
                let mut iter = api
                    .storage()
                    .at_latest()
                    .await?
                    .iter(eras_stakers_paged_addr)
                    .await?;

                while let Some(Ok(data)) = iter.next().await {
                    let exposure = data.value.0;
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

pub async fn inspect(crunch: &Crunch) -> Result<(), CrunchError> {
    let api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");

    let stashes = get_stashes(&crunch).await?;
    info!("Inspect {} stashes -> {}", stashes.len(), stashes.join(","));

    let history_depth_addr = ah_metadata::constants().staking().history_depth();
    let history_depth: u32 = api.constants().at(&history_depth_addr)?;

    let active_era_addr = ah_metadata::storage().staking().active_era();
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

    for stash_str in stashes.iter() {
        let stash = AccountId32::from_str(stash_str).map_err(|e| {
            CrunchError::Other(format!("Invalid account: {stash_str} error: {e:?}"))
        })?;
        info!("{} * Stash account", stash.clone());

        let start_index = active_era_index - history_depth;
        let mut unclaimed: Vec<(EraIndex, PageIndex)> = Vec::new();
        let mut claimed: Vec<(EraIndex, PageIndex)> = Vec::new();

        // Find unclaimed eras in previous 84 eras
        for era_index in start_index..active_era_index {
            // Verify if stash has claimed/unclaimed pages per era by cross checking eras_stakers_overview with claimed_rewards
            let claimed_rewards_addr = ah_metadata::storage()
                .staking()
                .claimed_rewards(era_index, stash.clone());
            if let Some(WeakBoundedVec(claimed_rewards)) = api
                .storage()
                .at_latest()
                .await?
                .fetch(&claimed_rewards_addr)
                .await?
            {
                // Verify if there are more pages to claim than the ones already claimed
                let eras_stakers_overview_addr = ah_metadata::storage()
                    .staking()
                    .eras_stakers_overview(era_index, stash.clone());
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
                let eras_stakers_paged_addr = ah_metadata::storage()
                    .staking()
                    .eras_stakers_paged_iter2(era_index, stash.clone());
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
