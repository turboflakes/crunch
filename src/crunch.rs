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
use crate::report::Report;
use crate::stats;
use async_recursion::async_recursion;
use async_std::task;
use codec::Decode;
use log::{debug, error, info, warn};
use rand::Rng;
use regex::Regex;
use std::{fs, result::Result, str::FromStr, thread, time};
use substrate_subxt::{
    balances::Balances,
    identity::{Data, IdentityOfStoreExt, SuperOfStoreExt},
    session::ValidatorsStoreExt,
    sp_core::{crypto, hash::H256, sr25519, Pair as PairT},
    sp_runtime::AccountId32,
    staking::{
        ActiveEraStoreExt, BondedStoreExt, EraIndex, EraPaidEvent, ErasRewardPointsStoreExt,
        ErasStakersStoreExt, HistoryDepthStoreExt, LedgerStoreExt, PayoutStakersCall,
        PayoutStakersCallExt, PayoutStartedEvent, RewardedEvent,
    },
    system::AccountStoreExt,
    utility::{BatchCallExt, BatchInterruptedEvent},
    Client, ClientBuilder, DefaultNodeRuntime, EventSubscription, PairSigner, RawEvent,
};

type Balance = <DefaultNodeRuntime as Balances>::Balance;

#[derive(Debug)]
pub struct Points {
    pub validator: u32,
    pub era_avg: f64,
    pub ci99: (f64, f64),
}

#[derive(Debug)]
pub struct Payout {
    pub block: u32,
    pub extrinsic: H256,
    pub era_index: u32,
    pub validator_amount_value: u128,
    pub nominators_amount_value: u128,
    pub nominators_quantity: u32,
    pub points: Points,
}

#[derive(Debug)]
pub struct Validator {
    pub stash: AccountId32,
    pub controller: Option<AccountId32>,
    pub name: String,
    pub is_active: bool,
    pub claimed: Vec<EraIndex>,
    pub unclaimed: Vec<EraIndex>,
    pub payouts: Vec<Payout>,
    pub warnings: Vec<String>,
}

impl Validator {
    fn new(stash: AccountId32) -> Validator {
        Validator {
            stash,
            controller: None,
            name: "".to_string(),
            is_active: false,
            claimed: Vec::new(),
            unclaimed: Vec::new(),
            payouts: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

type Validators = Vec<Validator>;

type ValidatorIndex = Option<usize>;
type ValidatorAmount = u128;
type NominatorsAmount = u128;

#[derive(Debug)]
pub struct Signer {
    pub account: AccountId32,
    pub name: String,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct Network {
    pub active_era: EraIndex,
    pub name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
}

#[derive(Debug)]
pub struct RawData {
    pub network: Network,
    pub signer: Signer,
    pub validators: Validators,
}

type Message = Vec<String>;

trait MessageTrait {
    fn log(&self);
    fn show_or_hide(&mut self, value: String, hidden: bool);
    fn show_or_hide_and_log(&mut self, value: String, hidden: bool);
}

impl MessageTrait for Message {
    fn log(&self) {
        info!("{}", &self[self.len() - 1]);
    }

    fn show_or_hide(&mut self, value: String, hidden: bool) {
        if !hidden {
            self.push(value);
        }
    }

    fn show_or_hide_and_log(&mut self, value: String, hidden: bool) {
        if !hidden {
            self.push(value);
            self.log();
        }
    }
}

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
                    "Connected to {} network using {} * Substrate node {} v{}",
                    client.chain_name(),
                    config.substrate_ws_url,
                    client.node_name(),
                    client.node_version()
                );
                break client;
            }
            Err(e) => {
                error!("{}", e);
                info!("Awaiting for connection using {}", config.substrate_ws_url);
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

fn trend(a: f64, b: f64) -> String {
    if a > b {
        String::from("↑")
    } else {
        String::from("↓")
    }
}

fn good_performance(a: f64, b: f64, out: String) -> String {
    if a >= b {
        return out;
    }
    String::from("")
}

fn poor_performance(a: f64, b: f64, out: String) -> String {
    if a <= b {
        return out;
    }
    String::from("")
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

    async fn send_message(
        &self,
        message: &str,
        formatted_message: &str,
    ) -> Result<(), CrunchError> {
        self.matrix()
            .send_message(message, formatted_message)
            .await?;
        Ok(())
    }

    /// Spawn and restart crunch flakes task on error
    pub fn flakes() {
        spawn_and_restart_crunch_flakes_on_error();
    }

    /// Spawn and restart subscription on error
    pub fn subscribe() {
        spawn_and_restart_subscription_on_error();
    }

    /// Spawn crunch view task
    pub fn view() {
        spawn_crunch_view();
    }

    fn get_existential_deposit(&self) -> Result<Balance, CrunchError> {
        let client = self.client.clone();
        let balances_metadata = client.metadata().module("Balances")?;
        let ed_metadata = balances_metadata.constant("ExistentialDeposit")?;
        let ed: u128 = ed_metadata.value()?;
        Ok(ed)
    }

    async fn collect_validators_data(
        &self,
        era_index: EraIndex,
    ) -> Result<Validators, CrunchError> {
        let client = self.client.clone();
        let config = CONFIG.clone();

        // Get unclaimed eras for the stash addresses
        let history_depth: u32 = client.history_depth(None).await?;
        let active_validators = client.validators(None).await?;
        let mut validators: Validators = Vec::new();

        for (_i, stash_str) in config.stashes.iter().enumerate() {
            let stash = AccountId32::from_str(stash_str)?;

            // Check if stash has bonded controller
            let controller = match client.bonded(stash.clone(), None).await? {
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
            // Instantiates a new validator struct
            let mut v = Validator::new(stash.clone());

            // Set controller
            v.controller = Some(controller.clone());

            // Get validator name
            v.name = self.get_display_name(&stash, None).await?;

            // Check if validator is in active set
            v.is_active = active_validators.contains(&stash);

            // Get unclaimed eras
            let start_index = era_index - history_depth;

            // Get staking info from ledger
            if let Some(staking_ledger) = client.ledger(controller.clone(), None).await? {
                debug!(
                    "{} * claimed_rewards: {:?}",
                    stash, staking_ledger.claimed_rewards
                );

                // Find unclaimed eras in previous 84 eras (reverse order)
                for era_index in (start_index..era_index).rev() {
                    // If reward was already claimed skip it
                    if staking_ledger.claimed_rewards.contains(&era_index) {
                        v.claimed.push(era_index);
                        continue;
                    }
                    // Verify if stash was active in set
                    let exposure = client.eras_stakers(era_index, stash.clone(), None).await?;
                    if exposure.total > 0 {
                        v.unclaimed.push(era_index)
                    }
                }
            }
            validators.push(v);
        }
        debug!("validators {:?}", validators);
        Ok(validators)
    }

    async fn get_validator_points_info(
        &self,
        era_index: EraIndex,
        stash: AccountId32,
    ) -> Result<Points, CrunchError> {
        let client = self.client.clone();
        // Get era reward points
        let era_reward_points = client.eras_reward_points(era_index, None).await?;
        let stash_points = match era_reward_points
            .individual
            .iter()
            .find(|(s, _)| *s == &stash)
        {
            Some((_, p)) => *p,
            None => 0,
        };

        // Calculate average points
        let points: Vec<f64> = era_reward_points
            .individual
            .into_iter()
            .map(|(_, points)| points as f64)
            .collect();

        let points = Points {
            validator: stash_points,
            era_avg: stats::mean(&points),
            ci99: stats::confidence_interval_99(&points),
        };

        Ok(points)
    }

    async fn run_in_batch(&self) -> Result<(), CrunchError> {
        let client = self.client.clone();
        let config = CONFIG.clone();
        let properties = client.properties();

        // Get network info
        let active_era = client.active_era(None).await?;
        let network = Network {
            active_era: active_era.index,
            name: client.chain_name().clone(),
            token_symbol: properties.token_symbol.clone(),
            token_decimals: properties.token_decimals,
        };

        // Load seed account
        let seed = fs::read_to_string(config.seed_path)
            .expect("Something went wrong reading the seed file");
        let seed_account: sr25519::Pair = get_from_seed(&seed, None);
        let seed_account_signer =
            PairSigner::<DefaultNodeRuntime, sr25519::Pair>::new(seed_account.clone());
        let seed_account_id: AccountId32 = seed_account.public().into();

        // Get signer account identity
        let signer_name = self.get_display_name(&seed_account_id, None).await?;
        let mut signer = Signer {
            account: seed_account_id.clone(),
            name: signer_name,
            warnings: Vec::new(),
        };

        // Warn if signer account is running low on funds (if lower than 2x Existential Deposit)
        let ed = self.get_existential_deposit()?;
        let seed_account_info = client.account(&seed_account_id, None).await?;
        let one = (std::u32::MAX as u128 + 1) * 10u128.pow(properties.token_decimals.into());
        let free: f64 = seed_account_info.data.free as f64 / one as f64;
        if free * 10f64.powi(properties.token_decimals.into()) <= (ed as f64 * 2_f64) {
            signer
                .warnings
                .push("⚡ Signer account is running low on funds ⚡".to_string());
        }
        // Add unclaimed eras into payout staker calls
        let mut calls = vec![];
        let mut validators = self.collect_validators_data(active_era.index).await?;
        for v in &mut validators {
            //
            if v.unclaimed.len() > 0 {
                let mut maximum_payouts = Some(config.maximum_payouts);
                // encode extrinsic payout stakers calls as many as unclaimed eras or maximum_payouts reached
                while let Some(i) = maximum_payouts {
                    if i == 0 {
                        maximum_payouts = None;
                    } else {
                        if let Some(claim_era) = v.unclaimed.pop() {
                            let call = client
                                .encode(PayoutStakersCall {
                                    validator_stash: v.stash.clone(),
                                    era: claim_era,
                                })
                                .unwrap();
                            calls.push(call);
                        }
                        maximum_payouts = Some(i - 1);
                    }
                }
            }
        }

        if calls.len() > 0 {
            // Batch calls
            // TODO check batch call weight or maximum 8
            let mut validator_index: ValidatorIndex = None;
            let mut era_index: EraIndex = 0;
            let mut validator_amount_value: ValidatorAmount = 0;
            let mut nominators_amount_value: NominatorsAmount = 0;
            let mut nominators_quantity = 0;

            info!(
                "Batch call with {} extrinsics ready to be dispatched",
                calls.len()
            );
            // Call extrinsic batch with payout stakers extrinsics as many as unclaimed eras or maximum_payouts reached
            // Wait for ExtrinsicSuccess event to calculate rewards and fetch points
            let batch_response = client.batch_and_watch(&seed_account_signer, calls).await?;
            debug!("batch_response: {:?}", batch_response);
            // Get block number
            let block = if let Some(header) = client.header(Some(batch_response.block)).await? {
                header.number
            } else {
                0
            };
            // Iterate over events to calculate respective reward amounts
            for event in batch_response.events {
                match event {
                    RawEvent {
                        ref module,
                        ref variant,
                        data,
                    } if module == "Staking" && variant == "PayoutStarted" => {
                        let event =
                            PayoutStartedEvent::<DefaultNodeRuntime>::decode(&mut &data[..])?;
                        let validator_index_ref = &mut validators
                            .iter()
                            .position(|v| v.stash == event.validator_stash);
                        validator_index = *validator_index_ref;
                        era_index = event.era_index;
                        validator_amount_value = 0;
                        nominators_amount_value = 0;
                        nominators_quantity = 0;
                    }
                    RawEvent {
                        ref module,
                        ref variant,
                        data,
                    } if module == "Staking" && variant == "Rewarded" => {
                        if let Some(i) = validator_index {
                            let event =
                                RewardedEvent::<DefaultNodeRuntime>::decode(&mut &data[..])?;
                            let validator = &validators[i];
                            if event.stash == validator.stash {
                                validator_amount_value = event.amount;
                            } else {
                                nominators_amount_value += event.amount;
                                nominators_quantity += 1;
                            }
                        }
                    }
                    RawEvent {
                        ref module,
                        ref variant,
                        data: _,
                    } if module == "Utility" && variant == "ItemCompleted" => {
                        if let Some(i) = validator_index {
                            let validator = &mut validators[i];
                            // Add era to claimed vec
                            validator.claimed.push(era_index);
                            // Fetch stash points
                            let points = self
                                .get_validator_points_info(era_index, validator.stash.clone())
                                .await?;

                            let p = Payout {
                                block,
                                extrinsic: batch_response.extrinsic,
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
                        ref module,
                        ref variant,
                        data: _,
                    } if module == "Utility" && variant == "BatchCompleted" => {
                        // TODO
                        info!("Batch completed");
                    }
                    RawEvent {
                        ref module,
                        ref variant,
                        data,
                    } if module == "Utility" && variant == "BatchInterrupted" => {
                        // TODO
                        let event =
                            BatchInterruptedEvent::<DefaultNodeRuntime>::decode(&mut &data[..])?;
                        error!("BatchInterrupted {:?}", event);
                    }
                    _ => (),
                };
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
        self.send_message(&report.message(), &report.formatted_message())
            .await?;

        Ok(())
    }

    //
    async fn run(&self) -> Result<(), CrunchError> {
        let client = self.client.clone();
        let config = CONFIG.clone();
        let properties = client.properties();

        // Load seed account
        let seed = fs::read_to_string(config.seed_path)
            .expect("Something went wrong reading the seed file");
        let seed_account: sr25519::Pair = get_from_seed(&seed, None);
        let seed_account_signer =
            PairSigner::<DefaultNodeRuntime, sr25519::Pair>::new(seed_account.clone());
        let seed_account_id: AccountId32 = seed_account.public().into();

        let mut message: Message = vec![];
        let mut formatted_message: Message = vec![];

        message.show_or_hide_and_log("Hey, it's crunch time!".to_owned(), false);
        formatted_message
            .show_or_hide("⏰ Hey, it's <code>crunch</code> time 👀".to_owned(), false);

        message.show_or_hide_and_log(
            format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            config.is_short,
        );
        formatted_message.show_or_hide(
            format!(
                "🤖 <code>{} v{}</code>",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ),
            config.is_short,
        );

        // Get signer identity and log it
        let identity = self.get_display_name(&seed_account_id, None).await?;
        message.show_or_hide_and_log(format!("{} * Signer account", identity), config.is_short);
        formatted_message.show_or_hide(
            format!("✍️ Signer account &middot; <code>{}</code>", identity),
            config.is_short,
        );
        // Warn if signer account is running low on funds (if lower than 2x Existential Deposit)
        let ed = self.get_existential_deposit()?;
        let seed_account_info = client.account(&seed_account_id, None).await?;
        let one = (std::u32::MAX as u128 + 1) * 10u128.pow(properties.token_decimals.into());
        let free: f64 = seed_account_info.data.free as f64 / one as f64;
        if free * 10f64.powi(properties.token_decimals.into()) <= (ed as f64 * 2_f64) {
            message
                .show_or_hide_and_log("Signer account is running low on funds".to_owned(), false);
            formatted_message.show_or_hide(
                "⚡ Signer account is running low on funds ⚡".to_owned(),
                false,
            );
        }

        let history_depth: u32 = client.history_depth(None).await?;
        let active_era = client.active_era(None).await?;

        message.show_or_hide_and_log(
            format!("{} active era {}", client.chain_name(), active_era.index),
            false,
        );
        formatted_message.show_or_hide(
            format!(
                "<br/>💙 <b>{}</b> -> playing era {} 🎶 🎹",
                client.chain_name(),
                active_era.index
            ),
            false,
        );

        // Get active validators
        let active_validators = client.validators(None).await?;
        for (_i, stash_str) in config.stashes.iter().enumerate() {
            let stash = AccountId32::from_str(stash_str)?;

            // Get stash identity
            let identity = self.get_display_name(&stash, None).await?;

            // Check if in active set
            let is_active = if active_validators.contains(&stash) {
                true
            } else {
                false
            };
            if is_active {
                message.show_or_hide_and_log(
                    format!(
                        "{} -> ACTIVE (https://{}.subscan.io/validator/{})",
                        identity,
                        client.chain_name().to_lowercase(),
                        stash
                    ),
                    false,
                );
                formatted_message.show_or_hide(
                    format!(
            "<br/>🧑‍🚀 <b><a href=\"https://{}.subscan.io/validator/{}\">{}</a></b> -> 🟢 Active",
            client.chain_name().to_lowercase(),
            stash,
            identity
          ),
                    false,
                );
            } else {
                message.show_or_hide_and_log(
                    format!(
                        "{} INACTIVE (https://{}.subscan.io/validator/{})",
                        identity,
                        client.chain_name().to_lowercase(),
                        stash
                    ),
                    false,
                );
                formatted_message.show_or_hide(
                    format!(
            "<br/>🧑‍🚀 <b><a href=\"https://{}.subscan.io/validator/{}\">{}</a></b> -> 🔴 Inactive",
            client.chain_name().to_lowercase(),
            stash,
            identity
          ),
                    false,
                );
            }

            // message.show_or_hide_and_log(format!("{} -> crunch {}", identity, subcommand()), config.is_short);
            // formatted_message.show_or_hide(format!(
            //   "<br/>🧑‍🚀 <b>{}</b> -> <code>crunch {}</code>",
            //   identity,
            //   subcommand()
            // ), config.is_short);
            //

            let start_index = active_era.index - history_depth;
            let mut unclaimed: Vec<u32> = Vec::new();
            let mut claimed: Vec<u32> = Vec::new();
            let mut maximum_payouts = Some(config.maximum_payouts);

            if let Some(controller) = client.bonded(stash.clone(), None).await? {
                message.show_or_hide_and_log(format!("{} * Stash account", stash), config.is_short);
                formatted_message.show_or_hide(
                    format!("💰 Stash account &middot; <code>{}</code>", stash),
                    config.is_short,
                );
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
                    //   message.show_or_hide_and_log(format!(
                    //     "In the last {} eras -> {} have already been crunched * {:?}",
                    //     history_depth,
                    //     claimed.len(),
                    //     claimed
                    //   ));
                    //   formatted_message.show_or_hide(format!(
                    //     "📒 In the last {} eras -> {} have already been crunched ✨",
                    //     history_depth,
                    //     claimed.len()
                    //   ));
                    // } else {
                    //   message.show_or_hide_and_log(format!(
                    //     "In the last {} eras -> There was nothing to crunch",
                    //     history_depth
                    //   ));
                    //   formatted_message.show_or_hide(format!(
                    //     "📒 In the last {} eras -> There was nothing to crunch 😞",
                    //     history_depth
                    //   ));
                    // }
                    debug!(
                        "{} * Claimed rewards {:?}",
                        stash, ledger_response.claimed_rewards
                    );

                    if unclaimed.len() > 0 {
                        // Get how many eras will be claimed based on maximum_payouts
                        // let quantity = if unclaimed.len() >= config.maximum_payouts.into() {
                        //   config.maximum_payouts
                        // } else {
                        //   unclaimed.len()
                        // };

                        // let symbols = number_to_symbols(unclaimed.len(), "!", history_depth as usize);
                        // message.show_or_hide_and_log(format!(
                        //   "{} And {} eras with {} to crunch {}",
                        //   symbols,
                        //   unclaimed.len(),
                        //   context(),
                        //   symbols,
                        // ));
                        // let symbols = number_to_symbols(unclaimed.len(), "⚡", history_depth as usize);
                        // formatted_message.show_or_hide(format!(
                        //   "{} And {} eras with {} to crunch {}",
                        //   symbols,
                        //   unclaimed.len(),
                        //   context(),
                        //   symbols,
                        // ));
                        //

                        debug!("{} * Unclaimed rewards {:?}", stash, unclaimed);

                        // Call extrinsic payout stakers as many and unclaimed eras or maximum_payouts reached
                        while let Some(i) = maximum_payouts {
                            if i == 0 {
                                maximum_payouts = None;
                            } else {
                                if let Some(claim_era) = unclaimed.pop() {
                                    // message.show_or_hide_and_log(format!("Crunch {} for era {}", context(), claim_era));
                                    // formatted_message.show_or_hide(format!(
                                    //   "🥣 <code>crunch</code> {} for era {} ⏳",
                                    //   context(),
                                    //   claim_era
                                    // ));
                                    //

                                    // Call extrinsic payout stakers and wait for event
                                    let event = client
                                        .payout_stakers_and_watch(
                                            &seed_account_signer,
                                            stash.clone(),
                                            claim_era,
                                        )
                                        .await?;

                                    debug!("{} * Result {:?}", stash, event);

                                    // Log Points
                                    let era_reward_points =
                                        client.eras_reward_points(claim_era, None).await?;
                                    let stash_points = match era_reward_points
                                        .individual
                                        .iter()
                                        .find(|(s, _)| *s == &stash)
                                    {
                                        Some((_, p)) => *p,
                                        None => 0,
                                    };

                                    // Calculate average points
                                    let points: Vec<f64> = era_reward_points
                                        .individual
                                        .into_iter()
                                        .map(|(_, points)| points as f64)
                                        .collect();
                                    let avg = stats::mean(&points);

                                    // Calculate validator and nominators reward amounts
                                    let mut stash_amount_value: u128 = 0;
                                    let mut others_amount_value: u128 = 0;
                                    let mut others_quantity: u32 = 0;
                                    for reward in event.find_events::<RewardedEvent<_>>()? {
                                        if reward.stash == stash {
                                            stash_amount_value = reward.amount;
                                        } else {
                                            others_amount_value += reward.amount;
                                            others_quantity += 1;
                                        }
                                    }

                                    // Total reward amount
                                    let ci99 = stats::confidence_interval_99(&points);
                                    let reward_amount = format!(
                                        "{:.4} {} {}",
                                        (stash_amount_value + others_amount_value) as f64
                                            / 10f64.powi(properties.token_decimals.into()),
                                        properties.token_symbol,
                                        good_performance(
                                            stash_points.into(),
                                            ci99.1,
                                            "🤯 🚀".into()
                                        )
                                    );
                                    message.show_or_hide_and_log(
                                        format!(
                                            "Points {} {}{}{} ({:.0}) -> {}",
                                            stash_points,
                                            trend(stash_points.into(), avg),
                                            good_performance(
                                                stash_points.into(),
                                                ci99.1,
                                                "↑".into()
                                            ),
                                            poor_performance(
                                                stash_points.into(),
                                                ci99.0,
                                                "↓".into()
                                            ),
                                            avg,
                                            reward_amount
                                        ),
                                        false,
                                    );
                                    formatted_message.show_or_hide(
                                        format!(
                                            "🎲 Points {} {}{}{} ({:.0}) -> 💸 {}",
                                            stash_points,
                                            trend(stash_points.into(), avg),
                                            good_performance(
                                                stash_points.into(),
                                                ci99.1,
                                                "↑".into()
                                            ),
                                            poor_performance(
                                                stash_points.into(),
                                                ci99.0,
                                                "↓".into()
                                            ),
                                            avg,
                                            reward_amount
                                        ),
                                        false,
                                    );

                                    // Validator reward amount
                                    let stash_amount = format!(
                                        "{:.4} {}",
                                        stash_amount_value as f64
                                            / 10f64.powi(properties.token_decimals.into()),
                                        properties.token_symbol
                                    );
                                    let stash_amount_percentage = (stash_amount_value as f64
                                        / (stash_amount_value + others_amount_value) as f64)
                                        * 100.0;
                                    message.show_or_hide_and_log(
                                        format!(
                                            "{} -> crunched {} ({:.2}%)",
                                            identity,
                                            // context(),
                                            stash_amount,
                                            stash_amount_percentage,
                                        ),
                                        config.is_short,
                                    );
                                    formatted_message.show_or_hide(
                                        format!(
                                            "🧑‍🚀 {} -> 💸 <b>{}</b> ({:.2}%)",
                                            identity,
                                            // context(),
                                            stash_amount,
                                            stash_amount_percentage
                                        ),
                                        config.is_short,
                                    );

                                    // Nominators reward amount
                                    let others_amount = format!(
                                        "{:.4} {}",
                                        others_amount_value as f64
                                            / 10f64.powi(properties.token_decimals.into()),
                                        properties.token_symbol
                                    );
                                    let others_amount_percentage = (others_amount_value as f64
                                        / (stash_amount_value + others_amount_value) as f64)
                                        * 100.0;

                                    message.show_or_hide_and_log(
                                        format!(
                                            "Nominators ({}) -> crunched {} ({:.2}%)",
                                            others_quantity,
                                            // context(),
                                            others_amount,
                                            others_amount_percentage,
                                        ),
                                        config.is_short,
                                    );
                                    formatted_message.show_or_hide(
                                        format!(
                                            "🦸 Nominators ({}) -> 💸 {} ({:.2}%)",
                                            others_quantity,
                                            // context(),
                                            others_amount,
                                            others_amount_percentage
                                        ),
                                        config.is_short,
                                    );

                                    // Log block number
                                    if let Some(header) = client.header(Some(event.block)).await? {
                                        message.show_or_hide_and_log(format!(
                      "Crunch era {} finalized at block #{} ({}) (https://{}.subscan.io/block/{:?})",
                      claim_era, header.number, event.block.to_string(), client.chain_name().to_lowercase(), event.block
                    ), false);
                                        formatted_message.show_or_hide( format!(
                      "💯 <code>crunch</code> era <del>{}</del> finalized at block #{} (<a href=\"https://{}.subscan.io/block/{:?}\">{}</a>) ✨",
                      claim_era, header.number, client.chain_name().to_lowercase(), event.block, event.block.to_string()
                    ), false);
                                        // push era to claimed vec
                                        claimed.push(claim_era);
                                    }
                                }
                                maximum_payouts = Some(i - 1);
                            }
                        }
                        // Check if there are still eras left to claim
                        if unclaimed.len() > 0 {
                            let symbols =
                                number_to_symbols(unclaimed.len(), "!", history_depth as usize);
                            message.show_or_hide_and_log(
                                format!(
                                    "{} And there are still {} eras left with {} to crunch {}",
                                    symbols,
                                    unclaimed.len(),
                                    context(),
                                    symbols,
                                ),
                                config.is_short,
                            );
                            let symbols =
                                number_to_symbols(unclaimed.len(), "⚡", history_depth as usize);
                            formatted_message.show_or_hide(
                                format!(
                                    "{} And there are still {} eras left with {} to crunch {}",
                                    symbols,
                                    unclaimed.len(),
                                    context(),
                                    symbols
                                ),
                                config.is_short,
                            );
                        } else {
                            message.show_or_hide_and_log(
                                format!("Well done! {} Just run out of {}!", identity, context()),
                                config.is_short,
                            );
                            formatted_message.show_or_hide(
                                format!(
                                    "✌️ <b>{}</b> just run out of {} 💫 💙",
                                    identity,
                                    context()
                                ),
                                config.is_short,
                            );
                        }
                    } else {
                        let m = if is_active {
                            "😊 🌱 ☀️ 🏄"
                        } else {
                            "🤔 💭 📚 💡 🦸 🗳️"
                        };
                        message.show_or_hide_and_log(
                            format!("Nothing to crunch this time!"),
                            config.is_short,
                        );
                        formatted_message.show_or_hide(
                            format!("🥣 Nothing to <code>crunch</code> this time -> {}", m),
                            config.is_short,
                        );
                    }
                    // General stats
                    // Inclusion
                    let inclusion_percentage =
                        ((claimed.len() + unclaimed.len()) as f32 / history_depth as f32) * 100.0;

                    message.show_or_hide_and_log(
                        format!(
                            "Inclusion {}/{} ({:.2}%)",
                            claimed.len() + unclaimed.len(),
                            history_depth,
                            inclusion_percentage
                        ),
                        config.is_short,
                    );
                    formatted_message.show_or_hide(
                        format!(
                            "📒 Inclusion {}/{} ({:.2}%)",
                            claimed.len() + unclaimed.len(),
                            history_depth,
                            inclusion_percentage
                        ),
                        config.is_short,
                    );

                    // Claimed
                    let claimed_percentage =
                        (claimed.len() as f32 / (claimed.len() + unclaimed.len()) as f32) * 100.0;

                    if claimed.len() > 0 {
                        message.show_or_hide_and_log(
                            format!(
                                "Crunched {}/{} ({:.2}%)",
                                claimed.len(),
                                claimed.len() + unclaimed.len(),
                                claimed_percentage
                            ),
                            config.is_short,
                        );
                        formatted_message.show_or_hide(
                            format!(
                                "😋 Crunched {}/{} ({:.2}%)",
                                claimed.len(),
                                claimed.len() + unclaimed.len(),
                                claimed_percentage
                            ),
                            config.is_short,
                        );
                    }
                }
            } else {
                message.show_or_hide_and_log(
                    format!(
                        "{} * Stash account does not have a Controller account!",
                        stash
                    ),
                    false,
                );
                formatted_message.show_or_hide(
                    format!(
                        "💰 <code>{}</code> ⚠️ Stash account does not have a Controller account ⚠️",
                        stash
                    ),
                    false,
                );
            }
        }

        if config.is_mode_era {
            message.show_or_hide_and_log(format!("Next crunch time will be at era end"), false);
            formatted_message.show_or_hide(
                format!("<br/>💨 Next <code>crunch</code> time will be at era end 💤<br/>___<br/>"),
                false,
            );
        } else {
            message.show_or_hide_and_log(
                format!(
                    "Next crunch time will be in {} hours!",
                    config.interval / 3600
                ),
                false,
            );
            formatted_message.show_or_hide(
                format!(
                    "<br/>💨 Next <code>crunch</code> time will be in {} hours 💤<br/>___<br/>",
                    config.interval / 3600
                ),
                false,
            );
        }
        self.send_message(&message.join("\n"), &formatted_message.join("<br/>"))
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
    async fn get_display_name(
        &self,
        stash: &AccountId32,
        sub_account_name: Option<String>,
    ) -> Result<String, CrunchError> {
        let client = self.client.clone();
        match client.identity_of(stash.clone(), None).await? {
            Some(identity) => {
                let parent = match identity.info.display {
                    Data::Raw(bytes) => format!(
                        "{}",
                        String::from_utf8(bytes.to_vec()).expect("Identity not utf-8")
                    ),
                    _ => format!("???"),
                };
                let name = match sub_account_name {
                    Some(child) => format!("{}/{}", parent, child),
                    None => parent,
                };
                Ok(name)
            }
            None => {
                if let Some((parent_account, data)) = client.super_of(stash.clone(), None).await? {
                    let sub_account_name = match data {
                        Data::Raw(bytes) => format!(
                            "{}",
                            String::from_utf8(bytes.to_vec()).expect("Identity not utf-8")
                        ),
                        _ => format!("???"),
                    };
                    return self
                        .get_display_name(&parent_account, Some(sub_account_name.to_string()))
                        .await;
                } else {
                    let s = &stash.to_string();
                    Ok(format!("{}...{}", &s[..6], &s[s.len() - 6..]))
                }
            }
        }
    }

    async fn run_and_subscribe_era_payout_events(&self) -> Result<(), CrunchError> {
        // Run once before start subscription
        self.run_in_batch().await?;
        info!("Subscribe 'EraPaid' on-chain finalized event");
        let client = self.client.clone();
        let sub = client.subscribe_finalized_events().await?;
        let decoder = client.events_decoder();
        let mut sub = EventSubscription::<DefaultNodeRuntime>::new(sub, decoder);
        sub.filter_event::<EraPaidEvent<DefaultNodeRuntime>>();
        while let Some(result) = sub.next().await {
            if let Ok(raw_event) = result {
                match EraPaidEvent::<DefaultNodeRuntime>::decode(&mut &raw_event.data[..]) {
                    Ok(event) => {
                        info!("Successfully decoded event {:?}", event);
                        // self.run().await?;
                        self.run_in_batch().await?;
                    }
                    Err(e) => return Err(CrunchError::CodecError(e)),
                }
            }
        }
        // If subscription has closed for some reason await and subscribe again
        Err(CrunchError::SubscriptionFinished)
    }
}

fn spawn_and_restart_subscription_on_error() {
    let crunch_task = task::spawn(async {
        let config = CONFIG.clone();
        loop {
            let c: Crunch = Crunch::new().await;
            if let Err(e) = c.run_and_subscribe_era_payout_events().await {
                match e {
                    CrunchError::SubscriptionFinished => warn!("{}", e),
                    CrunchError::MatrixError(_) => warn!("Matrix message skipped!"),
                    _ => {
                        error!("{}", e);
                        let message = format!("On hold for {} min!", config.error_interval);
                        let formatted_message = format!("<br/>🚨 An error was raised -> <code>crunch</code> on hold for {} min while rescue is on the way 🚁 🚒 🚑 🚓<br/><br/>", config.error_interval);
                        c.send_message(&message, &formatted_message).await.unwrap();
                        thread::sleep(time::Duration::from_secs(60 * config.error_interval));
                        continue;
                    }
                }
                thread::sleep(time::Duration::from_secs(1));
            };
        }
    });
    task::block_on(crunch_task);
}

fn spawn_and_restart_crunch_flakes_on_error() {
    let crunch_task = task::spawn(async {
        let config = CONFIG.clone();
        loop {
            let c: Crunch = Crunch::new().await;
            if let Err(e) = c.run().await {
                match e {
                    CrunchError::MatrixError(_) => warn!("Matrix message skipped!"),
                    _ => {
                        error!("{}", e);
                        let message = format!("On hold for {} min!", config.error_interval);
                        let formatted_message = format!("<br/>🚨 An error was raised -> <code>crunch</code> on hold for {} min while rescue is on the way 🚁 🚒 🚑 🚓<br/><br/>", config.error_interval);
                        c.send_message(&message, &formatted_message).await.unwrap();
                    }
                }
                thread::sleep(time::Duration::from_secs(60 * config.error_interval));
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
