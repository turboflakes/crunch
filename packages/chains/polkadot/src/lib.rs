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

use crunch_config::CONFIG;
use crunch_core::{get_keypair_from_seed_file, random_wait, try_fetch_onet_data, Crunch};
use crunch_error::CrunchError;
use crunch_people_polkadot::{get_display_name, get_distinct_parent_identites};
use crunch_relay_chain_polkadot::{
    collect_validators_data, get_stashes, rc_metadata,
    rc_metadata::staking::events::EraPaid, try_run_batch_payouts,
    try_run_batch_pool_members,
};
use crunch_report::{
    replace_emoji_lowercase, EraIndex, Network, NominationPoolsSummary, PageIndex,
    RawData, Report, SignerDetails, Validators,
};
use log::{debug, info, warn};
use std::{convert::TryInto, result::Result, str::FromStr, thread, time};
use subxt::utils::AccountId32;
use subxt_signer::sr25519::Keypair;

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
    let (signer_name, _, _) = get_display_name(&crunch, &seed_account_id, None).await?;
    let mut signer_details = SignerDetails {
        account: seed_account_id.clone(),
        name: signer_name,
        warnings: Vec::new(),
    };
    debug!("signer_details {:?}", signer_details);

    // Warn if signer account is running low on funds (if lower than 2x Existential Deposit)
    let ed_addr = rc_metadata::constants().balances().existential_deposit();
    let ed = api.constants().at(&ed_addr)?;

    let seed_account_info_addr = rc_metadata::storage()
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

    // Get Network name
    let chain_name = crunch.rpc().system_chain().await?;

    // Get Era index
    let active_era_addr = rc_metadata::storage().staking().active_era();
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

    let subdomain = crunch.subdomain(false);

    // Set network info
    let network = Network {
        name: chain_name.clone(),
        subdomain,
        active_era: active_era_index,
        token_symbol,
        token_decimals,
    };
    debug!("network {:?}", network);

    // Check if group by identity is enabled by user to change the behaviour of how stashes are processed
    if config.group_identity_enabled {
        // Try run payouts in batches
        let mut all_validators =
            collect_validators_data(&crunch, active_era_index).await?;

        let parent_identities: Vec<String> =
            get_distinct_parent_identites(all_validators.clone());

        for parent in parent_identities {
            // Filter validators by parent identity
            let mut validators = all_validators
                .clone()
                .into_iter()
                .filter(|v| replace_emoji_lowercase(&v.parent_identity) == parent)
                .collect::<Validators>();

            // Remove all processed validators from original vec so it don't get looked up again
            all_validators
                .retain(|v| replace_emoji_lowercase(&v.parent_identity) != parent);

            if validators.len() > 0 {
                // Try run payouts in batches
                let payout_summary =
                    try_run_batch_payouts(&crunch, &signer_keypair, &mut validators)
                        .await?;

                // Try fetch ONE-T grade data
                for v in &mut validators {
                    v.onet =
                        try_fetch_onet_data(chain_name.to_lowercase(), v.stash.clone())
                            .await?;
                }

                // NOTE: In the last iteration try to batch pools if any and include them in the report
                // TODO: Eventually we could do a separate message containing only the pools report
                let pools_summary: Option<NominationPoolsSummary> =
                    if all_validators.len() == 0 {
                        // Try run pool members in batches
                        Some(try_run_batch_pool_members(&crunch, &signer_keypair).await?)
                    } else {
                        None
                    };

                let data = RawData {
                    network: network.clone(),
                    signer_details: signer_details.clone(),
                    validators,
                    payout_summary,
                    pools_summary,
                };

                let report = Report::from(data);
                crunch
                    .send_message(&report.message(), &report.formatted_message())
                    .await?;
            }
            // NOTE: To prevent too many request from matrix API set a sleep here of 5 seconds before trying another identity payout
            thread::sleep(time::Duration::from_secs(5));
        }
    } else {
        let mut validators = collect_validators_data(&crunch, active_era_index).await?;

        // Try run payouts in batches
        let payout_summary =
            try_run_batch_payouts(&crunch, &signer_keypair, &mut validators).await?;

        // Try fetch ONE-T grade data
        for v in &mut validators {
            v.onet =
                try_fetch_onet_data(chain_name.to_lowercase(), v.stash.clone()).await?;
        }

        // Try run members in batches
        let pools_summary = try_run_batch_pool_members(&crunch, &signer_keypair).await?;

        let data = RawData {
            network,
            signer_details,
            validators,
            payout_summary,
            pools_summary: Some(pools_summary),
        };

        let report = Report::from(data);
        crunch
            .send_message(&report.message(), &report.formatted_message())
            .await?;
    }

    Ok(())
}

pub async fn inspect(crunch: &Crunch) -> Result<(), CrunchError> {
    let api = crunch.client().clone();

    let stashes = get_stashes(&crunch).await?;
    info!("Inspect {} stashes -> {}", stashes.len(), stashes.join(","));

    let history_depth_addr = rc_metadata::constants().staking().history_depth();
    let history_depth: u32 = api.constants().at(&history_depth_addr)?;

    let active_era_addr = rc_metadata::storage().staking().active_era();
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
            let claimed_rewards_addr = rc_metadata::storage()
                .staking()
                .claimed_rewards(era_index, stash.clone());
            if let Some(claimed_rewards) = api
                .storage()
                .at_latest()
                .await?
                .fetch(&claimed_rewards_addr)
                .await?
            {
                // Verify if there are more pages to claim than the ones already claimed
                let eras_stakers_overview_addr = rc_metadata::storage()
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
                let eras_stakers_paged_addr = rc_metadata::storage()
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
