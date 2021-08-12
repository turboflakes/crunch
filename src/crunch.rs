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
use async_recursion::async_recursion;
use async_std::task;
use codec::Encode;
use log::{debug, error, info, warn};
use percent_encoding::percent_decode;
use rand::Rng;
use regex::Regex;
use std::{fs, result::Result, str::FromStr, thread, time};
use substrate_subxt::{
  identity::{IdentityOfStoreExt, SuperOfStoreExt},
  session::ValidatorsStoreExt,
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
  // Use regex to remove control characters
  let re = Regex::new(r"[\x00-\x1F]").unwrap();
  let clean_seed = re.replace_all(&seed.trim(), "");
  sr25519::Pair::from_string(&clean_seed, pass)
    .expect("constructed from known-good static value; qed")
}

fn fun() -> String {
  let words = vec![
    "delicious",
    "tasty",
    "mental",
    "psycho",
    "fruity",
    "crazy",
    "spicy",
    "yummy",
    "supernatural",
    "juicy",
    "super",
    "mellow",
    "sweet",
    "nutty",
    "insane",
    "fantastic",
    "unbelievable",
    "incredible",
  ];
  let mut rng = rand::thread_rng();
  words[rng.gen_range(0..words.len() - 1)].to_string()
}

fn context() -> String {
  let config = CONFIG.clone();
  if config.is_boring {
    return String::from("rewards");
  }
  format!("{} flakes", fun())
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
  matrix: Matrix,
}

impl Crunch {
  async fn new() -> Crunch {
    let client = create_or_await_substrate_node_client(CONFIG.clone()).await;
    let properties = client.properties();
    // Display SS58 addresses based on the connected chain
    crypto::set_default_ss58_version(crypto::Ss58AddressFormat::Custom(
      properties.ss58_format.into(),
    ));

    // Initialize matrix client
    let mut matrix: Matrix = Matrix::new();
    matrix
      .authenticate(properties.ss58_format.into())
      .await
      .unwrap_or_else(|e| {
        error!("{}", e);
        Default::default()
      });
    Crunch { client, matrix }
  }

  /// Returns the matrix configuration
  pub fn matrix(&self) -> &Matrix {
    &self.matrix
  }

  async fn send_message(&self, message: &str, formatted_message: &str) -> Result<(), CrunchError> {
    info!("{}", message);
    self
      .matrix()
      .send_message(message, formatted_message)
      .await?;
    Ok(())
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

    // Load seed account
    let seed =
      fs::read_to_string(config.seed_path).expect("Something went wrong reading the seed file");
    let seed_account: sr25519::Pair = get_from_seed(&seed, None);
    let seed_account_signer =
      PairSigner::<DefaultNodeRuntime, sr25519::Pair>::new(seed_account.clone());
    let seed_account_id: AccountId32 = seed_account.public().into();

    let mut message: Vec<String> = vec![];
    let mut formatted_message: Vec<String> = vec![];

    // message.push("Hey, it's crunch time!".to_owned());
    // formatted_message.push("⏰ Hey, it's crunch time 👀".to_owned());

    message.push(format!(
      "H{} v{}",
      env!("CARGO_PKG_NAME"),
      env!("CARGO_PKG_VERSION")
    ));
    formatted_message.push(format!(
      "🤖 <code>{} v{}</code>",
      env!("CARGO_PKG_NAME"),
      env!("CARGO_PKG_VERSION")
    ));
    // Get signer identity and log it
    let identity = self.get_identity(&seed_account_id, None).await?;
    message.push(format!("{} * Signer account", identity));
    formatted_message.push(format!("✍️ Signer account <code>{}</code>", identity));
    let history_depth: u32 = client.history_depth(None).await?;
    let active_era = client.active_era(None).await?;
    // get active validators
    let active_validators = client.validators(None).await?;
    for (_i, stash_str) in config.stashes.iter().enumerate() {
      let stash = AccountId32::from_str(stash_str)?;

      // Get stash identity
      let identity = self.get_identity(&stash, None).await?;

      message.push(format!("{} -> crunch time!", identity));
      formatted_message.push(format!(
        "<br>🧑‍🚀 <b>{}</b> -> <code>crunch</code> time 🏁",
        identity
      ));

      let start_index = active_era.index - history_depth;
      let mut unclaimed: Vec<u32> = Vec::new();
      let mut claimed: Vec<u32> = Vec::new();
      let mut maximum_payouts = Some(config.maximum_payouts);

      if let Some(controller) = client.bonded(stash.clone(), None).await? {
        message.push(format!("{} * Stash account", stash));
        formatted_message.push(format!("&middot; 💰 <code>{}</code>", stash));
        //
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
          // if claimed.len() > 0 {
          //   message.push(format!(
          //     "In the last {} eras -> {} have already been crunched * {:?}",
          //     history_depth,
          //     claimed.len(),
          //     claimed
          //   ));
          //   formatted_message.push(format!(
          //     "&middot; 📒 In the last {} eras -> {} have already been crunched ✨",
          //     history_depth,
          //     claimed.len()
          //   ));
          // } else {
          //   message.push(format!(
          //     "In the last {} eras -> There was nothing to crunch",
          //     history_depth
          //   ));
          //   formatted_message.push(format!(
          //     "&middot; 📒 In the last {} eras -> There was nothing to crunch 😞",
          //     history_depth
          //   ));
          // }
          debug!(
            "{} * Claimed rewards {:?}",
            stash, ledger_response.claimed_rewards
          );

          let inclusion_percentage =
            ((claimed.len() + unclaimed.len()) as f32 / history_depth as f32) * 100.0;

          message.push(format!(
            "Inclusion {}/{} ({:.2}%)",
            claimed.len() + unclaimed.len(),
            history_depth,
            inclusion_percentage
          ));
          formatted_message.push(format!(
            "&middot; 📒 Inclusion {}/{} ({:.2}%)",
            claimed.len() + unclaimed.len(),
            history_depth,
            inclusion_percentage
          ));

          let claimed_percentage =
            (claimed.len() as f32 / (claimed.len() + unclaimed.len()) as f32) * 100.0;

          if claimed.len() > 0 {
            message.push(format!(
              "Crunched {}/{} ({:.2}%)",
              claimed.len(),
              claimed.len() + unclaimed.len(),
              claimed_percentage
            ));
            formatted_message.push(format!(
              "&middot; 😋 Crunched {}/{} ({:.2}%)",
              claimed.len(),
              claimed.len() + unclaimed.len(),
              claimed_percentage
            ));
          }

          if unclaimed.len() > 0 {
            // Get how many eras will be claimed based on maximum_payouts
            // let quantity = if unclaimed.len() >= config.maximum_payouts.into() {
            //   config.maximum_payouts
            // } else {
            //   unclaimed.len()
            // };

            let symbols = number_to_symbols(unclaimed.len(), "!", history_depth as usize);
            message.push(format!(
              "{} And {} eras with {} to crunch {}",
              symbols,
              unclaimed.len(),
              context(),
              symbols,
            ));
            let symbols = number_to_symbols(unclaimed.len(), "⚡", history_depth as usize);
            formatted_message.push(format!(
              "&middot; {} And {} eras with {} to crunch {}",
              symbols,
              unclaimed.len(),
              context(),
              symbols,
            ));

            debug!("{} * Unclaimed rewards {:?}", stash, unclaimed);

            // Call extrinsic payout stakers as many and unclaimed eras or maximum_payouts reached
            while let Some(i) = maximum_payouts {
              if i == 0 {
                maximum_payouts = None;
              } else {
                if let Some(claim_era) = unclaimed.pop() {
                  message.push(format!("Crunching {} for era {}", context(), claim_era));
                  formatted_message.push(format!(
                    "&middot; 🥣 Crunching {} for era {} ⏳",
                    context(),
                    claim_era
                  ));

                  // Call extrinsic payout stakers and wait for event
                  let extrinsic = client
                    .payout_stakers_and_watch(&seed_account_signer, stash.clone(), claim_era)
                    .await?;

                  debug!("{} * Result {:?}", stash, extrinsic);

                  // Calculate validator and nominators reward amounts
                  let mut stash_amount_value: u128 = 0;
                  let mut others_amount_value: u128 = 0;
                  let mut others_quantity: u32 = 0;
                  for reward in extrinsic.find_events::<RewardEvent<_>>()? {
                    if reward.stash == stash {
                      stash_amount_value = reward.amount;
                    } else {
                      others_amount_value += reward.amount;
                      others_quantity += 1;
                    }
                  }

                  // Validator reward amount
                  let stash_amount = format!(
                    "{:.4} {}",
                    stash_amount_value as f64 / 10f64.powi(properties.token_decimals.into()),
                    properties.token_symbol
                  );
                  let stash_amount_percentage = (stash_amount_value as f64
                    / (stash_amount_value + others_amount_value) as f64)
                    * 100.0;
                  message.push(format!(
                    "{} -> crunched {} = {} ({:.2}%)",
                    identity,
                    context(),
                    stash_amount,
                    stash_amount_percentage,
                  ));
                  formatted_message.push(format!(
                    "&middot; &middot; 🧑‍🚀 <b>{}</b> -> crunched {} <b>{}</b> ({:.2}%)",
                    identity,
                    context(),
                    stash_amount,
                    stash_amount_percentage
                  ));

                  // Nominators reward amount
                  let others_amount = format!(
                    "{:.4} {}",
                    others_amount_value as f64 / 10f64.powi(properties.token_decimals.into()),
                    properties.token_symbol
                  );
                  let others_amount_percentage = (others_amount_value as f64
                    / (stash_amount_value + others_amount_value) as f64)
                    * 100.0;

                  message.push(format!(
                    "Nominators ({}) -> crunched {} = {} ({:.2}%)",
                    others_quantity,
                    context(),
                    others_amount,
                    others_amount_percentage,
                  ));
                  formatted_message.push(format!(
                    "&middot; &middot; 🦸 Nominators ({}) -> crunched {} {} ({:.2}%)",
                    others_quantity,
                    context(),
                    others_amount,
                    others_amount_percentage
                  ));

                  // Log block number
                  if let Some(header) = client.header(Some(extrinsic.block)).await? {
                    message.push(format!(
                      "Crunch finalized at block #{} ({}) https://polkadot.js.org/apps/?rpc={}#/explorer/query/{:?}",
                      header.number, extrinsic.block.to_string(), config.substrate_ws_url, extrinsic.block
                    ));
                    formatted_message.push( format!(
                      "&middot; &middot; 💯 Crunch finalized at block #{} (<a href=\"https://polkadot.js.org/apps/?rpc={}#/explorer/query/{:?}\">{}</a>) ✨",
                      header.number, config.substrate_ws_url, extrinsic.block, extrinsic.block.to_string()
                    ));
                  }
                }
                maximum_payouts = Some(i - 1);
              }
            }
            // Check if there are still eras left to claim
            if unclaimed.len() > 0 {
              let symbols = number_to_symbols(unclaimed.len(), "!", history_depth as usize);
              message.push(format!(
                "{} All good! But there are still {} eras left with {} to crunch {}",
                symbols,
                unclaimed.len(),
                context(),
                symbols,
              ));
              let symbols = number_to_symbols(unclaimed.len(), "⚡", history_depth as usize);
              formatted_message.push(format!(
                "&middot; {} All good! But there are still {} eras left with {} to crunch {}",
                symbols,
                unclaimed.len(),
                context(),
                symbols
              ));
            } else {
              message.push(format!(
                "Well done! {} Just run out of {}!",
                identity,
                context()
              ));
              formatted_message.push(format!(
                "&middot; ✌️ Well done! <b>{}</b> Just run out of {} ✨💙",
                identity,
                context()
              ));
            }
          } else {
            message.push(format!("And nothing to crunch this time!"));
            formatted_message.push(format!(
              "&middot; 🤔 And nothing to crunch this time 💭 🪴 📚 🧠 💡 👨‍💻"
            ));
          }
          // Active set
          if active_validators.contains(&stash) {
            message.push(format!("{} -> ACTIVE", identity));
            formatted_message.push(format!(
              "&middot; 🟢 <b>Active</b> -> 🌱 {} are on the way ✌️",
              context()
            ));
          } else {
            message.push(format!("{} INACTIVE", identity));
            formatted_message.push(format!(
              "&middot; 🔴 <b>Inactive</b> -> 🔎 looking out for Nominators 🦸🗳️"
            ));
          }
        }
      } else {
        message.push(format!(
          "{} * Stash account does not have a Controller account!",
          stash
        ));
        formatted_message.push(format!(
          "&middot; 💰 <code>{}</code> ⚠️ Stash account does not have a Controller account ⚠️",
          stash
        ));
      }
    }

    message.push(format!(
      "Job done -> next crunch time will be in {} hours!",
      config.interval / 3600
    ));
    formatted_message.push(format!(
      "<br>💨 Job done -> next crunch time will be in {} hours ⏱️ 💤",
      config.interval / 3600
    ));
    self
      .send_message(&message.join("\n"), &formatted_message.join("<br>"))
      .await?;
    Ok(())
  }

  //
  async fn inspect(&self) -> Result<(), CrunchError> {
    let client = self.client.clone();
    let config = CONFIG.clone();

    info!("Inspect stashes -> {}", config.stashes.join(","));
    let history_depth: u32 = client.history_depth(None).await?;
    let active_era = client.active_era(None).await?;
    for stash_str in config.stashes.iter() {
      let stash = AccountId32::from_str(stash_str)?;
      info!("{} * Stash account", stash);

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

  #[async_recursion]
  async fn get_identity(
    &self,
    stash: &AccountId32,
    sub_account_name: Option<String>,
  ) -> Result<String, CrunchError> {
    let client = self.client.clone();
    // Use regex to remove control characters
    let re = Regex::new(r"[\x00-\x1F]").unwrap();
    match client.identity_of(stash.clone(), None).await? {
      Some(registration) => {
        let encoded = registration.info.display.encode();
        let decoded = percent_decode(&encoded).decode_utf8()?;
        let parent = re.replace_all(&decoded.to_string(), "").trim().to_string();

        let name = match sub_account_name {
          Some(child) => format!("{}/{}", parent, child),
          None => parent,
        };
        Ok(name)
      }
      None => {
        if let Some((parent_account, data)) = client.super_of(stash.clone(), None).await? {
          let encoded = data.encode();
          let decoded = percent_decode(&encoded).decode_utf8()?;
          let sub_account_name = re.replace_all(&decoded.trim(), "");
          return self
            .get_identity(&parent_account, Some(sub_account_name.to_string()))
            .await;
        } else {
          let s = &stash.to_string();
          Ok(format!("{}...{}", &s[..6], &s[s.len() - 6..]))
        }
      }
    }
  }
}

fn spawn_and_restart_crunch_flakes_on_error() {
  let crunch_task = task::spawn(async {
    let config = CONFIG.clone();
    loop {
      let c: Crunch = Crunch::new().await;
      if let Err(e) = c.run().await {
        error!("{}", e);
        match e {
          CrunchError::MatrixError(_) => warn!("Matrix message skipped!"),
          _ => {
            let message = format!("On hold for 30 min!");
            let formatted_message = format!("🚨 An error was raised. Crunch 🤖 stays on hold for 30 min -> Rescue is on the way 🚁 🚒 🚑 🚓");
            c.send_message(&message, &formatted_message).await.unwrap();
          }
        }
        thread::sleep(time::Duration::from_secs(60 * 30));
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
