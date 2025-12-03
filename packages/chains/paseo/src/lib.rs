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

use crunch_asset_hub_paseo::{
    ah_metadata::staking::events::EraPaid, ah_metadata::system::events::CodeUpdated,
    fetch_active_era_index, fetch_claimed_or_unclaimed_pages_per_era, fetch_controller,
    get_era_index_start, get_signer_details, get_stashes, try_run_batch_payouts,
    try_run_batch_pool_members,
};
use crunch_config::CONFIG;
use crunch_core::{get_keypair_from_seed_file, random_wait, try_fetch_onet_data, Crunch};
use crunch_error::CrunchError;
use crunch_people_paseo::{get_display_name, get_distinct_parent_identites};
use crunch_relay_chain_paseo::fetch_authorities;
use crunch_report::{
    replace_emoji_lowercase, EraIndex, Network, NominationPoolsSummary, RawData, Report,
    Validator, Validators,
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
    let api = crunch
        .asset_hub_client()
        .as_ref()
        .expect("AH API to be available");
    let rpc = crunch
        .asset_hub_rpc()
        .as_ref()
        .expect("AH Legacy API to be available");

    // Keep track of the last known runtime version
    let last_spec_version = api.runtime_version().spec_version;

    let mut block_sub = api.blocks().subscribe_finalized().await?;
    while let Some(block) = block_sub.next().await {
        // Fetch current runtime version before trying to decode anything
        let current_spec_version = api.runtime_version().spec_version;

        // If a runtime upgrade occurred, raise known error so all clients could be
        // gracefully recreated
        if current_spec_version != last_spec_version {
            return Err(CrunchError::RuntimeUpgradeDetected(
                last_spec_version,
                current_spec_version,
            ));
        }

        // Silently handle RPC disconnection and wait for the next block as soon as
        // reconnection is available
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
                    if let Some(block_hash) =
                        rpc.chain_get_block_hash(Some(block_number.into())).await?
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

        // Event --> system::CodeUpdated
        if let Some(_event) = events.find_first::<CodeUpdated>()? {
            return Err(CrunchError::RuntimeUpgradeDetected(
                last_spec_version,
                current_spec_version,
            ));
        }

        latest_block_number_processed = Some(block.number());
    }
    // If subscription has closed for some reason await and subscribe again
    Err(CrunchError::SubscriptionFinished)
}

async fn collect_validators_data(
    crunch: &Crunch,
    era_index: EraIndex,
) -> Result<Validators, CrunchError> {
    // Get unclaimed eras for the stash addresses
    let active_validators = fetch_authorities(&crunch).await?;
    debug!("active_validators {:?}", active_validators);

    let mut validators: Validators = Vec::new();

    let stashes = get_stashes(&crunch).await?;

    for (_i, stash_str) in stashes.iter().enumerate() {
        let stash = AccountId32::from_str(stash_str).map_err(|e| {
            CrunchError::Other(format!("Invalid account: {stash_str} error: {e:?}"))
        })?;

        let controller = fetch_controller(&crunch, &stash, &mut validators).await?;

        if controller.is_none() {
            continue;
        }

        // Instantiates a new validator struct
        let mut v = Validator::new(stash.clone());

        // Set controller
        v.controller = controller.clone();

        // Get validator name
        (v.name, v.parent_identity, v.has_identity) =
            get_display_name(&crunch, &stash, None).await?;

        // Check if validator is in active set
        v.is_active = active_validators.contains(&stash);

        // Look for unclaimed eras, starting on current_era - maximum_eras
        let start_index = get_era_index_start(&crunch, era_index).await?;

        // Find unclaimed eras in previous 84 eras (reverse order)
        for era in (start_index..era_index).rev() {
            fetch_claimed_or_unclaimed_pages_per_era(&crunch, &stash, era, &mut v)
                .await?;
        }
        validators.push(v);
    }

    // Sort validators by identity, than by non-identity and push the stashes
    // with warnings to bottom
    let mut validators_with_warnings = validators
        .clone()
        .into_iter()
        .filter(|v| v.warnings.len() > 0)
        .collect::<Vec<Validator>>();

    validators_with_warnings.sort_by(|a, b| {
        replace_emoji_lowercase(&a.name)
            .partial_cmp(&replace_emoji_lowercase(&b.name))
            .unwrap()
    });

    let validators_with_no_identity = validators
        .clone()
        .into_iter()
        .filter(|v| v.warnings.len() == 0 && !v.has_identity)
        .collect::<Vec<Validator>>();

    let mut validators = validators
        .into_iter()
        .filter(|v| v.warnings.len() == 0 && v.has_identity)
        .collect::<Vec<Validator>>();

    validators.sort_by(|a, b| {
        replace_emoji_lowercase(&a.name)
            .partial_cmp(&replace_emoji_lowercase(&b.name))
            .unwrap()
    });
    validators.extend(validators_with_no_identity);
    validators.extend(validators_with_warnings);

    debug!("validators {:?}", validators);
    Ok(validators)
}

pub async fn try_crunch(crunch: &Crunch) -> Result<(), CrunchError> {
    let config = CONFIG.clone();

    let signer_keypair: Keypair = get_keypair_from_seed_file()?;
    let seed_account_id: AccountId32 = signer_keypair.public_key().into();

    let signer_details = get_signer_details(&crunch, &seed_account_id).await?;

    // Get Network name
    let chain_name = crunch.rpc().system_chain().await?;

    // Get Era index
    let active_era_index = fetch_active_era_index(&crunch).await?;

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

    let subdomain = crunch.subdomain(true);

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
    crunch_asset_hub_paseo::inspect(crunch).await
}
