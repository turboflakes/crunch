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
use crate::runtime::{
    // polkadot::{
    //     self,
    //     // DefaultConfig,
    //     // polkadot,
    //     // polkadot::polkadot
    //     // node_runtime,
    //     // node_runtime::{runtime_types::pallet_identity::types::Data, staking},
    // },
    polkadot,
    kusama,
    westend
};
use crate::stats;
use async_recursion::async_recursion;
use async_std::task;
use codec::Decode;
use log::{debug, error, info, warn};
use regex::Regex;
use std::{cmp, fs, result::Result, str::FromStr, thread, time};

use subxt::{
    extrinsic::PairSigner,
    sp_core::{crypto, sr25519},
    // sp_core::{crypto, sr25519, Pair as PairT, H256},
    sp_runtime::AccountId32,
    Client,
    ClientBuilder,
    Config as NodeConfig,
    EventSubscription,
    RawEvent,
};

type EraIndex = u32;

#[derive(Debug)]
pub struct Points {
    pub validator: u32,
    pub era_avg: f64,
    pub ci99_9_interval: (f64, f64),
    pub outlier_limits: (f64, f64),
}

#[derive(Debug)]
pub struct Payout {
    pub block: u32,
    // pub extrinsic: H256, TODO
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
    // pub active_era: EraIndex,
    pub active_era: u32,
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

// pub async fn create_substrate_node_client(
//     config: Config,
// ) -> Result<Client<DefaultNodeRuntime>, subxt::Error> {
//     ClientBuilder::new()
//         .set_url(config.substrate_ws_url)
//         .build()
//         .await
// }

// pub async fn create_or_await_substrate_node_client(config: Config) -> Client<DefaultNodeRuntime> {
//     loop {
//         match create_substrate_node_client(config.clone()).await {
//             Ok(client) => {
//                 info!(
//                     "Connected to {} network using {} * Substrate node {} v{}",
//                     client.chain_name(),
//                     config.substrate_ws_url,
//                     client.node_name(),
//                     client.node_version()
//                 );
//                 break client;
//             }
//             Err(e) => {
//                 error!("{}", e);
//                 info!("Awaiting for connection using {}", config.substrate_ws_url);
//                 thread::sleep(time::Duration::from_secs(6));
//             }
//         }
//     }
// }

/// Helper function to generate a crypto pair from seed
fn get_from_seed(seed: &str, pass: Option<&str>) -> sr25519::Pair {
    // Use regex to remove control characters
    let re = Regex::new(r"[\x00-\x1F]").unwrap();
    let clean_seed = re.replace_all(&seed.trim(), "");
    sr25519::Pair::from_string(&clean_seed, pass)
        .expect("constructed from known-good static value; qed")
}

// trait Context {
//     // fn to_runtime_api<R: From<Self>>(self) -> R;
//     // Instance method signature
//     // fn api(&self) -> &'static str;
//     fn api(&self) -> &'static str;
// }

// pub struct PolkadotContext {
//     pub api: polkadot::RuntimeApi<DefaultConfig>,
// }

// pub struct KusamaContext {
//     pub api: polkadot::RuntimeApi<DefaultConfig>,
// }

// pub struct WestendContext {
//     pub api: polkadot::RuntimeApi<DefaultConfig>,
// }

// struct Cow {}

// impl Context for Cow {}

// async fn test_context() -> Result<Box<dyn Context>, String> {
//     let config = CONFIG.clone();
//     if config.is_kusama() {
//         let client = ClientBuilder::new()
//             .set_url(config.substrate_ws_url)
//             .build::<DefaultConfig>()
//             .await?;
//         Ok(Box::new(
//             client.to_runtime_api::<polkadot::RuntimeApi<polkadot::DefaultConfig>>(),
//         ))
//     } else {
//         Ok(Box::new(Cow {}))
//     }
// }

fn set_default_ss58_version() {
    let config = CONFIG.clone();
    let ss58_version = if config.is_westend() {
        crypto::Ss58AddressFormatRegistry::SubstrateAccount
    } else if config.is_kusama() {
        crypto::Ss58AddressFormatRegistry::KusamaAccount
    } else {
        crypto::Ss58AddressFormatRegistry::PolkadotAccount
    }
    .into();

    crypto::set_default_ss58_version(ss58_version);
}

async fn create_substrate_node_client<C: subxt::Config>(config: Config) -> Client<C> {
    loop {
        let result = ClientBuilder::new()
            .set_url(config.substrate_ws_url.clone())
            .build::<C>()
            .await;
        match result {
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
            Err(err) => {
                error!("{}", err);
                thread::sleep(time::Duration::from_secs(5));
            }
        }
    }
}

// pub async fn polkadot_api(client: &Client<polkadot::DefaultConfig>) -> polkadot::RuntimeApi<polkadot::DefaultConfig> {
//     client.clone().to_runtime_api()
// }

pub async fn westend_api() -> westend::RuntimeApi<westend::DefaultConfig> {
    let client = create_substrate_node_client::<westend::DefaultConfig>(CONFIG.clone()).await;
    client.clone().to_runtime_api()
}

pub async fn kusama_api() -> kusama::RuntimeApi<kusama::DefaultConfig> {
    let client = create_substrate_node_client::<kusama::DefaultConfig>(CONFIG.clone()).await;
    client.clone().to_runtime_api()
}

pub async fn polkadot_api() -> polkadot::RuntimeApi<polkadot::DefaultConfig> {
    let client = create_substrate_node_client::<polkadot::DefaultConfig>(CONFIG.clone()).await;
    client.clone().to_runtime_api()
}

pub struct Crunch<R: subxt::Config> {
    // api: polkadot::RuntimeApi<DefaultConfig>,
    // polkadot_client: Client<DefaultConfig>,
    // kusama_client: Client<DefaultConfig>,
    // westend_client: Client<DefaultConfig>,
    client: Client<R>,
    matrix: Matrix,
}

impl<R> Crunch<R>
where
    R: subxt::Config,
{
    async fn new() -> Crunch<R> {
        // let client = create_or_await_substrate_node_client(CONFIG.clone()).await;

        // pub api: node_runtime::RuntimeApi<DefaultConfig>,

        // let api: node_runtime::RuntimeApi<DefaultNodeRuntime> = client
        //     .clone()
        //     .to_runtime_api::<polkadot::RuntimeApi<polkadot::DefaultConfig>>();

        // let properties = client.properties();
        // Display SS58 addresses based on the connected chain
        // crypto::set_default_ss58_version(crypto::Ss58AddressFormat::Custom(
        //     properties.ss58_format.into(),
        // ));

        // Display SS58 addresses based on the connected chain
        set_default_ss58_version();

        // Initialize matrix client
        let mut matrix: Matrix = Matrix::new();
        // matrix
        //     .authenticate(properties.ss58_format.into())
        //     .await
        //     .unwrap_or_else(|e| {
        //         error!("{}", e);
        //         Default::default()
        //     });

        let config = CONFIG.clone();
        let a = if config.is_kusama() {
            kusama_api().await
        } else {
            westend_api().await
        };

        // let polkadot_node = TestNodeProcess::<DefaultConfig>::connect().await.unwrap();
        // .with_authority(key)
        // .scan_for_open_ports()
        // .spawn::<DefaultConfig>()
        // .await;

        let client = create_substrate_node_client::<R>(CONFIG.clone()).await;
        // pub async fn polkadot_api(&self) -> polkadot::RuntimeApi<polkadot::DefaultConfig> {
        //     let node = Node::<polkadot::DefaultConfig>::new().await;
        //     node.client().clone().to_runtime_api()
        // }
        // let c = Crunch::<polkadot::DefaultConfig>::new().await;
        // let api: polkadot::RuntimeApi<polkadot::DefaultConfig> = c.client().clone().to_runtime_api();

        Self { client, matrix }
    }

    

    pub fn client(&self) -> &Client<R> {
        &self.client
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
    // pub fn flakes() {
    //     spawn_and_restart_crunch_flakes_on_error();
    // }

    /// Spawn and restart subscription on error
    // pub fn subscribe() {
    //     spawn_and_restart_subscription_on_error();
    // }

    /// Spawn crunch view task
    // pub fn view() {
    //     spawn_crunch_view();
    // }

    async fn run_in_batch(&self) -> Result<(), CrunchError> {
        // let client = self.client();
        let config = CONFIG.clone();
        // let api = polkadot_api(self.client()).await;
        // let api = if config.is_westend() {
        //     self.client().to_runtime_api::<polkadot::RuntimeApi<polkadot::DefaultConfig>>()
        //     // Crunch::<polkadot::DefaultConfig>::new().await
        // } else if config.is_kusama() {
        //     self.client().to_runtime_api::<polkadot::RuntimeApi<polkadot::DefaultConfig>>()
        // } else {
        //     self.client().to_runtime_api::<polkadot::RuntimeApi<polkadot::DefaultConfig>>()
        // };
        
        let properties = client.properties();

        let active_era_index = match api.storage().staking().active_era(None).await? {
            Some(active_era_info) => active_era_info.index,
            None => return Err(CrunchError::Other("Active era not available".into())),
        };

        // Set network info
        let network = Network {
            active_era: active_era_index,
            name: client.chain_name().clone(),
            token_symbol: properties.token_symbol.clone(),
            token_decimals: properties.token_decimals,
        };
        debug!("network {:?}", network);

        // Load seed account
        let seed = fs::read_to_string(config.seed_path)
            .expect("Something went wrong reading the seed file");
        let seed_account: sr25519::Pair = get_from_seed(&seed, None);
        let seed_account_signer =
            PairSigner::<polkadot::DefaultConfig, sr25519::Pair>::new(seed_account.clone());
        let seed_account_id: AccountId32 = seed_account.public().into();

        // Get signer account identity
        let signer_name = self.get_display_name(&seed_account_id, None).await?;
        let mut signer = Signer {
            account: seed_account_id.clone(),
            name: signer_name,
            warnings: Vec::new(),
        };
        debug!("signer {:?}", signer);

        // Warn if signer account is running low on funds (if lower than 2x Existential Deposit)
        let ed = self.get_existential_deposit()?;
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

        let mut calls_to_be: Vec<(AccountId32, EraIndex)> = vec![];
        let mut validators = self.collect_validators_data(active_era_index).await?;
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
                            calls_to_be.push((v.stash.clone(), claim_era));
                        }
                        maximum_payouts = Some(i - 1);
                    }
                }
            }
        }

        if calls_to_be.len() > 0 {
            // TODO check batch call weight or maximum 8
            let mut validator_index: ValidatorIndex = None;
            // let mut era_index: EraIndex = 0;
            let mut validator_amount_value: ValidatorAmount = 0;
            let mut nominators_amount_value: NominatorsAmount = 0;
            let mut nominators_quantity = 0;

            info!("{} extrinsics ready to be dispatched", calls_to_be.len());

            // TODO activate batch calls by optional flag when subxt library is ready for utility().batch()
            // currently is failing with Substrate_subxt error: Rpc error: The background task been terminated because: Custom error: Unparsable response; restart required
            //
            // let batch_response = api
            //     .tx()
            //     .utility()
            //     .batch(calls)
            //     .sign_and_submit_then_watch(&seed_account_signer)
            //     .await?;
            // debug!("batch_response {:?}", batch_response);

            for (stash, era_index) in calls_to_be {
                let response = api
                    .tx()
                    .staking()
                    .payout_stakers(stash.clone(), era_index)
                    .sign_and_submit_then_watch(&seed_account_signer)
                    .await?;
                debug!("response {:?}", response);

                // Get block number
                let block = if let Some(header) = client.rpc().header(Some(response.block)).await? {
                    header.number
                } else {
                    0
                };

                // Iterate over events to calculate respective reward amounts
                for event in response.events {
                    debug!("{:?}", event);
                    match event {
                        RawEvent {
                            ref pallet,
                            ref variant,
                            data,
                            ..
                        } if pallet == "Staking" && variant == "PayoutStarted" => {
                            let event_decoded =
                                staking::events::PayoutStarted::decode(&mut &data[..])?;
                            debug!("{:?}", event_decoded);
                            let validator_index_ref =
                                &mut validators.iter().position(|v| v.stash == event_decoded.1);
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
                            let event_decoded = staking::events::Rewarded::decode(&mut &data[..])?;
                            debug!("{:?}", event_decoded);
                            if event_decoded.0 == stash {
                                validator_amount_value = event_decoded.1;
                            } else {
                                nominators_amount_value += event_decoded.1;
                                nominators_quantity += 1;
                            }
                        }
                        // RawEvent {
                        //     ref module,
                        //     ref variant,
                        //     data: _,
                        // } if module == "Utility" && variant == "ItemCompleted" => {
                        //     if let Some(i) = validator_index {
                        //         let validator = &mut validators[i];
                        //         // Add era to claimed vec
                        //         validator.claimed.push(era_index);
                        //         // Fetch stash points
                        //         let points = self
                        //             .get_validator_points_info(era_index, validator.stash.clone())
                        //             .await?;

                        //         let p = Payout {
                        //             block,
                        //             extrinsic: batch_response.extrinsic,
                        //             era_index,
                        //             validator_amount_value,
                        //             nominators_amount_value,
                        //             nominators_quantity,
                        //             points,
                        //         };
                        //         validator.payouts.push(p);
                        //     }
                        // }
                        _ => (),
                    };
                }
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
                        // extrinsic: response.extrinsic, //TODO
                        era_index,
                        validator_amount_value,
                        nominators_amount_value,
                        nominators_quantity,
                        points,
                    };
                    validator.payouts.push(p);
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
        self.send_message(&report.message(), &report.formatted_message())
            .await?;

        Ok(())
    }

    //
    async fn inspect(&self) -> Result<(), CrunchError> {
        let api = self.api();
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

    #[async_recursion]
    async fn get_display_name(
        &self,
        stash: &AccountId32,
        sub_account_name: Option<String>,
    ) -> Result<String, CrunchError> {
        let api = self.api();

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

    fn get_existential_deposit(&self) -> Result<u128, CrunchError> {
        let client = self.client();
        let balances_metadata = client.metadata().pallet("Balances")?;
        let constant_metadata = balances_metadata.constant("ExistentialDeposit")?;
        let ed = u128::decode(&mut &constant_metadata.value[..])?;
        Ok(ed)
    }

    async fn collect_validators_data(
        &self,
        era_index: EraIndex,
    ) -> Result<Validators, CrunchError> {
        let api = self.api();
        let config = CONFIG.clone();

        // Get unclaimed eras for the stash addresses
        let active_validators = api.storage().session().validators(None).await?;
        debug!("active_validators {:?}", active_validators);
        let mut validators: Validators = Vec::new();

        for (_i, stash_str) in config.stashes.iter().enumerate() {
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
            v.name = self.get_display_name(&stash, None).await?;

            // Check if validator is in active set
            v.is_active = active_validators.contains(&stash);

            // Look for unclaimed eras, starting on current_era - maximum_eras
            let start_index = self.get_era_index_start(era_index).await?;

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

    async fn get_era_index_start(&self, era_index: EraIndex) -> Result<EraIndex, CrunchError> {
        let api = self.api();
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

    async fn get_validator_points_info(
        &self,
        era_index: EraIndex,
        stash: AccountId32,
    ) -> Result<Points, CrunchError> {
        let api = self.api();
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

    async fn run_and_subscribe_era_payout_events(&self) -> Result<(), CrunchError> {
        // Run once before start subscription
        self.run_in_batch().await?;
        info!("Subscribe 'EraPaid' on-chain finalized event");
        let client = self.client();
        let sub = client.rpc().subscribe_finalized_events().await?;
        let decoder = client.events_decoder();
        let mut sub = EventSubscription::<DefaultNodeRuntime>::new(sub, &decoder);
        sub.filter_event::<staking::events::EraPaid>();
        while let Some(result) = sub.next().await {
            if let Ok(raw) = result {
                match staking::events::PayoutStarted::decode(&mut &raw.data[..]) {
                    Ok(event) => {
                        info!("Successfully decoded event {:?}", event);
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

pub fn spawn_and_restart_crunch_flakes_on_error<C: subxt::Config>() {
    let crunch_task = task::spawn(async {
        let config = CONFIG.clone();
        loop {
            let c = Crunch::<C>::new().await;
            if let Err(e) = c.run_in_batch().await {
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

fn parse_identity_data(data: Data) -> String {
    match data {
        Data::Raw0(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw1(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw2(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw3(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw4(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw5(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw6(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw7(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw8(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw9(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw10(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw11(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw12(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw13(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw14(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw15(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw16(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw17(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw18(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw19(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw20(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw21(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw22(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw23(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw24(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw25(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw26(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw27(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw28(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw29(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw30(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw31(bytes) => parse_display_name(bytes.to_vec()),
        Data::Raw32(bytes) => parse_display_name(bytes.to_vec()),
        _ => format!("???"),
    }
}

fn parse_display_name(bytes: Vec<u8>) -> String {
    format!("{}", String::from_utf8(bytes).expect("Identity not utf-8"))
}
