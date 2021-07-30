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
use crate::matrix::Matrix;
use async_std::task;
use log::{debug, error, info, warn};
use std::fs;
use std::{result::Result, str::FromStr};
use std::{thread, time};
use substrate_subxt::{
  sp_core::{crypto, sr25519, Pair as PairT},
  sp_runtime::AccountId32,
  staking::{
    ActiveEraStoreExt, BondedStoreExt, ErasStakersStoreExt, HistoryDepthStoreExt, LedgerStoreExt,
    PayoutStakersCallExt, RewardEvent,
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
      Ok(client) => {
        info!(
          "Connected to {} network <> {} * Substrate node <> {} v{}",
          client.chain_name(),
          config.substrate_ws_url,
          client.node_name(),
          client.node_version()
        );
        break client;
      }
      Err(e) => {
        error!("{}", e);
        info!("Awaiting for connection <> {}", config.substrate_ws_url);
        thread::sleep(time::Duration::from_secs(6));
      }
    }
  }
}

/// Helper function to generate a crypto pair from seed
fn get_from_seed(seed: &str, pass: Option<&str>) -> sr25519::Pair {
  sr25519::Pair::from_string(seed, pass).expect("constructed from known-good static value; qed")
}

fn number_to_symbols(n: usize, symbol: &str, max: usize) -> String {
  let cap: usize = match n {
    n if n < (max / 4) as usize => 1,
    n if n < (max / 2) as usize => 2,
    n if n < max - (max / 4) as usize => 3,
    _ => 4,
  };
  let v = vec![""; cap + 1];
  v.join(symbol)
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

  /// Spawn and restart crunch flakes task on error
  pub fn flakes() {
    spawn_and_restart_crunch_flakes_on_error();
  }

  /// Spawn crunch view task
  pub fn view() {
    spawn_crunch_view();
  }

  //
  async fn run(&self) -> Result<(), CrunchError> {
    let client = self.client.clone();
    let config = CONFIG.clone();

    let properties = client.properties();
    // Display SS58 addresses based on the connected chain
    crypto::set_default_ss58_version(crypto::Ss58AddressFormat::Custom(
      properties.ss58_format.into(),
    ));

    // Load seed account
    let seed =
      fs::read_to_string(config.seed_path).expect("Something went wrong reading the seed file");
    let seed_account: sr25519::Pair = get_from_seed(&seed, None);
    let seed_account_signer =
      PairSigner::<DefaultNodeRuntime, sr25519::Pair>::new(seed_account.clone());
    let seed_account_id: AccountId32 = seed_account.public().into();

    // Matrix authentication
    let mut m: Matrix = Matrix::new(properties.ss58_format.into()).await;
    m.login().await?;

    let message = format!("Hey, it's crunch time!");
    let formatted_message = format!("⏰ Hey, it's crunch time 🦾");
    info!("{}", message);
    m.send_message(&message, &formatted_message).await?;

    let message = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let formatted_message = format!(
      "🤖 <code>{} v{}</code>",
      env!("CARGO_PKG_NAME"),
      env!("CARGO_PKG_VERSION")
    );
    m.send_message(&message, &formatted_message).await?;

    // log seed public account
    let message = format!("{} * Signer account", seed_account_id);
    let formatted_message = format!("✍️ Signer account 👉 <code>{}</code>", seed_account_id);

    info!("{}", message);
    m.send_message(&message, &formatted_message).await?;

    let history_depth: u32 = client.history_depth(None).await?;
    let active_era = client.active_era(None).await?;
    for (i, stash_str) in config.stashes.iter().enumerate() {
      let stash = AccountId32::from_str(stash_str)?;

      let message = format!("Task {} -> Go crunch it!", i + 1);
      let formatted_message = format!("🏁 {} --> Go crunch it!", i + 1);

      info!("{}", message);
      m.send_message(&message, &formatted_message).await?;

      let message = format!("{} * Stash account", stash);
      let formatted_message = format!("💰 Stash account 👉 <code>{}</code>", stash);

      info!("{}", message);
      m.send_message(&message, &formatted_message).await?;

      let start_index = active_era.index - history_depth;
      let mut unclaimed: Vec<u32> = Vec::new();
      let mut claimed: Vec<u32> = Vec::new();
      let mut maximum_payouts = Some(config.maximum_payouts);

      if let Some(controller) = client.bonded(stash.clone(), None).await? {
        if let Some(ledger_response) = client.ledger(controller.clone(), None).await? {
          // Find unclaimed eras in previous 84 eras
          for era_index in start_index..active_era.index {
            // If reward was already claimed skip it
            if ledger_response.claimed_rewards.contains(&era_index) {
              claimed.push(era_index);
              continue;
            }
            // Verify if stash was active in set
            let exposure = client.eras_stakers(era_index, stash.clone(), None).await?;
            if exposure.total > 0 {
              unclaimed.push(era_index)
            }
          }
          if claimed.len() > 0 {
            let message = format!(
              "In the last {} eras -> {} have already been crunched * {:?}",
              history_depth,
              claimed.len(),
              claimed
            );
            let formatted_message = format!(
              "📒 In the last {} eras --> {} have already been crunched 💨",
              history_depth,
              claimed.len()
            );
            info!("{}", message);
            m.send_message(&message, &formatted_message).await?;
          } else {
            let message = format!(
              "In the last {} eras -> There was nothing to crunch",
              history_depth
            );
            let formatted_message = format!(
              "📒 In the last {} eras --> There was nothing to crunch 😞",
              history_depth
            );
            info!("{}", message);
            m.send_message(&message, &formatted_message).await?;
          }
          debug!(
            "{} * Claimed rewards {:?}",
            stash, ledger_response.claimed_rewards
          );

          if unclaimed.len() > 0 {
            // Get how many eras will be claimed based on maximum_payouts
            let quantity = if unclaimed.len() >= config.maximum_payouts {
              config.maximum_payouts
            } else {
              unclaimed.len()
            };

            let symbols = number_to_symbols(unclaimed.len(), "!", history_depth as usize);
            let message = format!(
              "{} And {} eras still have delicious flakes to crunch {} So, let's crunch {}!",
              symbols,
              unclaimed.len(),
              symbols,
              quantity
            );
            let symbols = number_to_symbols(unclaimed.len(), "⚡", history_depth as usize);
            let formatted_message = format!(
              "{} And {} eras still have delicious flakes to crunch {} So, let's crunch {} 😋",
              symbols,
              unclaimed.len(),
              symbols,
              quantity
            );
            info!("{}", message);
            m.send_message(&message, &formatted_message).await?;

            debug!("{} * Unclaimed rewards {:?}", stash, unclaimed);

            // Call extrinsic payout stakers as many and unclaimed eras or maximum_payouts reached
            while let Some(i) = maximum_payouts {
              if i == 0 {
                maximum_payouts = None;
              } else {
                if let Some(claim_era) = unclaimed.pop() {
                  let message = format!("Crunch flakes in era {}", stash);
                  let formatted_message = format!("🥣 Crunch flakes in era {}", claim_era);
                  info!("{}", message);
                  m.send_message(&message, &formatted_message).await?;

                  // Call extrinsic payout stakers and wait for event
                  let result = client
                    .payout_stakers_and_watch(&seed_account_signer, stash.clone(), claim_era)
                    .await?;

                  debug!("{} * Result {:?}", stash, result);

                  // Calculate validator and nominators reward amounts
                  let mut stash_amount_value: u128 = 0;
                  let mut others_amount_value: u128 = 0;
                  for reward in result.find_events::<RewardEvent<_>>()? {
                    if reward.stash == stash {
                      stash_amount_value = reward.amount;
                    } else {
                      others_amount_value += reward.amount;
                    }
                  }

                  // Validator reward amount
                  let stash_amount = format!(
                    "{} {}",
                    stash_amount_value as f64 / 10f64.powi(properties.token_decimals.into()),
                    properties.token_symbol
                  );
                  let stash_amount_percentage = (stash_amount_value as f64
                    / (stash_amount_value + others_amount_value) as f64)
                    * 100.0;
                  let message = format!(
                    "Validator crunched tasty flakes worth of {} ({:.2}%)",
                    stash_amount, stash_amount_percentage,
                  );
                  let formatted_message = format!(
                    "🧑‍🚀 Validator crunched tasty flakes worth of {} ({:.2}%)",
                    stash_amount, stash_amount_percentage
                  );
                  info!("{}", message);
                  m.send_message(&message, &formatted_message).await?;

                  // Nominators reward amount
                  let others_amount = format!(
                    "{} {}",
                    others_amount_value as f64 / 10f64.powi(properties.token_decimals.into()),
                    properties.token_symbol
                  );
                  let others_amount_percentage = (others_amount_value as f64
                    / (stash_amount_value + others_amount_value) as f64)
                    * 100.0;

                  let message = format!(
                    "Nominators crunched tasty flakes worth of {} ({:.2}%)",
                    others_amount, others_amount_percentage,
                  );
                  let formatted_message = format!(
                    "🦸 Nominators crunched tasty flakes worth of {} ({:.2}%)",
                    others_amount, others_amount_percentage
                  );
                  info!("{}", message);
                  m.send_message(&message, &formatted_message).await?;
                }
                maximum_payouts = Some(i - 1);
              }
            }
            // Check if there are still eras left to claim
            if unclaimed.len() > 0 {
              let symbols = number_to_symbols(unclaimed.len(), "!", history_depth as usize);
              let message = format!(
                "{} There are still some left -> {} eras with delicious flakes to crunch {}",
                symbols,
                unclaimed.len(),
                symbols
              );
              let symbols = number_to_symbols(unclaimed.len(), "⚠️", history_depth as usize);
              let formatted_message = format!(
                "{} There are still some left --> {} eras with delicious flakes to crunch {}",
                symbols,
                unclaimed.len(),
                symbols
              );
              warn!("{} * {:?}", message, unclaimed);
              m.send_message(&message, &formatted_message).await?;
            } else {
              let message = format!("Well done! Stash account {} Just run out of flakes!", stash);
              let formatted_message = format!(
                "✌️ Well done! Stash account 👉 <code>{}</code> Just run out of flakes ✨💙",
                stash
              );
              info!("{}", message);
              m.send_message(&message, &formatted_message).await?;
            }
          } else {
            let message = format!("And nothing to crunch this time!");
            let formatted_message = format!("🙃 And nothing to crunch this time 🪴 📚");
            info!("{}", message);
            m.send_message(&message, &formatted_message).await?;
          }
        }
      } else {
        warn!(
          "{} * The Stash account specified does not have any Controller account!",
          stash
        );
      }
    }

    let message = format!(
      "Next crunch time will be in {} hours!",
      config.interval / 3600
    );
    let formatted_message = format!(
      "⏲️ Next crunch time will be in {} hours 💤",
      config.interval / 3600
    );
    info!("{}", message);
    m.send_message(&message, &formatted_message).await?;
    m.logout().await?;
    Ok(())
  }
  async fn inspect(&self) -> Result<(), CrunchError> {
    let client = self.client.clone();
    let config = CONFIG.clone();

    let properties = client.properties();
    // Display SS58 addresses based on the connected chain
    crypto::set_default_ss58_version(crypto::Ss58AddressFormat::Custom(
      properties.ss58_format.into(),
    ));

    let message = format!("Inspect stashes -> {}", config.stashes.join(","));
    info!("{}", message);

    let history_depth: u32 = client.history_depth(None).await?;
    let active_era = client.active_era(None).await?;
    for stash_str in config.stashes.iter() {
      let stash = AccountId32::from_str(stash_str)?;
      let message = format!("{} * Stash account", stash);
      info!("{}", message);

      let start_index = active_era.index - history_depth;
      let mut unclaimed: Vec<u32> = Vec::new();
      let mut claimed: Vec<u32> = Vec::new();

      if let Some(controller) = client.bonded(stash.clone(), None).await? {
        if let Some(ledger_response) = client.ledger(controller.clone(), None).await? {
          // Find unclaimed eras in previous 84 eras
          for era_index in start_index..active_era.index {
            // If reward was already claimed skip it
            if ledger_response.claimed_rewards.contains(&era_index) {
              claimed.push(era_index);
              continue;
            }
            // Verify if stash was active in set
            let exposure = client.eras_stakers(era_index, stash.clone(), None).await?;
            if exposure.total > 0 {
              unclaimed.push(era_index)
            }
          }
        }
      }
      let message = format!(
        "{} claimed eras in the last {} -> {:?}",
        claimed.len(),
        history_depth,
        claimed
      );
      info!("{}", message);
      let message = format!(
        "{} unclaimed eras in the last {} -> {:?}",
        unclaimed.len(),
        history_depth,
        unclaimed
      );
      info!("{}", message);
    }
    let message = format!("Job done!");
    info!("{}", message);
    Ok(())
  }
}

fn spawn_and_restart_crunch_flakes_on_error() {
  let crunch_task = task::spawn(async {
    let config = CONFIG.clone();
    loop {
      let c: Crunch = Crunch::new().await;
      if let Err(e) = c.run().await {
        error!("{}", e);
        thread::sleep(time::Duration::from_secs(5));
        continue;
      };
      thread::sleep(time::Duration::from_secs(config.interval));
    }
  });
  task::block_on(crunch_task);
}

fn spawn_crunch_view() {
  let crunch_task = task::spawn(async {
    let c: Crunch = Crunch::new().await;
    if let Err(e) = c.inspect().await {
      error!("{}", e);
    };
  });
  task::block_on(crunch_task);
}
