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

use crate::config::{Config, CONFIG};
use crate::errors::CrunchError;
use async_std::task;
use chrono::Utc;
use codec::Encode;
use log::{debug, error, info, warn};
use regex::Regex;
use std::fs;
use std::{
  collections::BTreeMap, convert::TryInto, marker::PhantomData, result::Result, str::FromStr,
};
use std::{thread, time};
use substrate_subxt::{
  identity::{IdentityOfStoreExt, Judgement, SubsOfStoreExt, SuperOfStoreExt},
  session::{NewSessionEvent, ValidatorsStore},
  sp_core::storage::StorageKey,
  sp_core::Decode,
  sp_core::{sr25519, Pair as PairT},
  sp_runtime::AccountId32,
  staking::{
    ActiveEraStoreExt, BondedStoreExt, EraIndex, EraPayoutEvent, ErasRewardPointsStoreExt, PayoutStakersCallExt,
    ErasStakersClippedStoreExt, ErasStakersStoreExt, ErasTotalStakeStoreExt,
    ErasValidatorPrefsStoreExt, ErasValidatorRewardStoreExt, HistoryDepthStoreExt, LedgerStoreExt,
    NominatorsStoreExt, PayeeStoreExt, RewardDestination, RewardPoint, ValidatorsStoreExt,
  },
  Client, ClientBuilder, DefaultNodeRuntime, PairSigner,
};

pub async fn create_substrate_node_client(
  config: Config,
) -> Result<Client<DefaultNodeRuntime>, substrate_subxt::Error> {
  ClientBuilder::<DefaultNodeRuntime>::new()
    .set_url(config.substrate_ws_url)
    .skip_type_sizes_check()
    .build()
    .await
}

pub async fn create_or_await_substrate_node_client(config: Config) -> Client<DefaultNodeRuntime> {
  loop {
    match create_substrate_node_client(config.clone()).await {
      Ok(client) => break client,
      Err(e) => {
        error!("{}", e);
        info!("Awaiting for Substrate node client to be ready");
        thread::sleep(time::Duration::from_secs(6));
      }
    }
  }
}

fn get_account_id_from_storage_key(key: StorageKey) -> AccountId32 {
  let s = &key.0[key.0.len() - 32..];
  let v: [u8; 32] = s.try_into().expect("slice with incorrect length");
  AccountId32::new(v)
}

/// Helper function to generate a crypto pair from seed
fn get_from_seed(seed: &str, pass: Option<&str>) -> sr25519::Pair {
  sr25519::Pair::from_string(seed, pass).expect("constructed from known-good static value; qed")
}

pub struct Crunch {
  pub client: substrate_subxt::Client<DefaultNodeRuntime>,
}

impl Crunch {
  async fn new() -> Crunch {
    Crunch {
      client: create_or_await_substrate_node_client(CONFIG.clone()).await,
    }
  }

  /// Spawn and restart crunch task on error
  pub fn it() {
    spawn_and_restart_crunch_on_error();
  }

  async fn run(&self) -> Result<(), CrunchError> {
    let client = self.client.clone();
    let config = CONFIG.clone();

    let history_depth: u32 = client.history_depth(None).await?;
    let active_era = client.active_era(None).await?;
    


    info!("Looking for flakes in stashes: {:?}", config.stashes);
    let seed = fs::read_to_string(config.seed_phrase_filename)
      .expect("Something went wrong reading the file");

    let seed_account: sr25519::Pair = get_from_seed(&seed, None);
    let seed_account_signer = PairSigner::<DefaultNodeRuntime, sr25519::Pair>::new(seed_account);
    for stash_str in config.stashes.iter() {
      let stash = AccountId32::from_str(stash_str)?;

      let start_index = active_era.index - history_depth;
      let mut unclaimed: Vec<u32> = Vec::new();
      let mut max_unclaimed = Some(config.max_unclaimed);

      if let Some(controller) = client.bonded(stash.clone(), None).await? {
        if let Some(ledger_response) = client.ledger(controller.clone(), None).await? {
          debug!(
            "stash {} already claimed rewards: {:?}",
            stash, ledger_response.claimed_rewards
          );
          // Find unclaimed eras in previous 84 eras
          for era_index in start_index..active_era.index {
            // If reward was already claimed skip it
            if ledger_response.claimed_rewards.contains(&era_index) {
              info!(
                "Stash {} crunched already some tasty flakes at era: {:?}",
                stash, era_index
              );
              continue;
            }
            // Verify if stash was active in set
            let exposure = client.eras_stakers(era_index, stash.clone(), None).await?;
            if exposure.total > 0 {
              unclaimed.push(era_index)
            }
          }

          info!("unclaimed eras: {:?}", unclaimed);

          while let Some(i) = max_unclaimed {
            if i == 0 {
              println!("Equal 0, quit!");
              max_unclaimed = None;
            } else {
              if let Some(claim_era) = unclaimed.pop() {
                info!("CLaim era {}", claim_era);
                // Payout unclaimed eras
                let response = client.payout_stakers_and_watch(&seed_account_signer, stash.clone(), claim_era).await?;
                println!("response: {:?}", response);
              }
              max_unclaimed = Some(i - 1);
            }
          }
          if unclaimed.len() > 0 {
            warn!(
              "Stash {} still has {} bowls full of delicious flakes to go!",
              stash,
              unclaimed.len()
            );
          } else {
            info!("Stash {} run out of flakes!", stash);
          }
        }
      };
    }

    info!("There's no flakes to take in - shelves are empty.");
    Ok(())
  }
}

fn spawn_and_restart_crunch_on_error() {
  let crunch_task = task::spawn(async {
    let config = CONFIG.clone();
    loop {
      let c: Crunch = Crunch::new().await;
      if let Err(e) = c.run().await {
        error!("{}", e);
        thread::sleep(time::Duration::from_millis(500));
        continue;
      };
      thread::sleep(time::Duration::from_secs(config.interval));
    }
  });
  task::block_on(crunch_task);
}
