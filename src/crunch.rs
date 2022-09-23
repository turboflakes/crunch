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
use crate::runtimes::{
    aleph_zero, aleph_zero_testnet, kusama, lagoon, polkadot,
    support::{ChainPrefix, ChainTokenSymbol, SupportedRuntime},
    westend,
};
use async_std::task;
use log::{error, info, warn};
use rand::Rng;
use regex::Regex;
use std::{convert::TryInto, result::Result, thread, time};

use subxt::{
    rpc::JsonValue,
    sp_core::{crypto, sr25519, Pair as PairT},
    Client, ClientBuilder, DefaultConfig,
};

pub type ValidatorIndex = Option<usize>;
pub type ValidatorAmount = u128;
pub type NominatorsAmount = u128;

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
) -> Result<Client<DefaultConfig>, subxt::BasicError> {
    ClientBuilder::new()
        .set_url(config.substrate_ws_url)
        .build::<DefaultConfig>()
        .await
}

pub async fn create_or_await_substrate_node_client(
    config: Config,
) -> Client<DefaultConfig> {
    loop {
        match create_substrate_node_client(config.clone()).await {
            Ok(client) => {
                let chain = client
                    .rpc()
                    .system_chain()
                    .await
                    .unwrap_or_else(|_| "Chain undefined".to_string());
                let name = client
                    .rpc()
                    .system_name()
                    .await
                    .unwrap_or_else(|_| "Node name undefined".to_string());
                let version = client
                    .rpc()
                    .system_version()
                    .await
                    .unwrap_or_else(|_| "Node version undefined".to_string());

                info!(
                    "Connected to {} network using {} * Substrate node {} v{}",
                    chain, config.substrate_ws_url, name, version
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
pub fn get_from_seed(seed: &str, pass: Option<&str>) -> sr25519::Pair {
    // Use regex to remove control characters
    let re = Regex::new(r"[\x00-\x1F]").unwrap();
    let clean_seed = re.replace_all(&seed.trim(), "");
    sr25519::Pair::from_string(&clean_seed, pass)
        .expect("constructed from known-good static value; qed")
}

pub struct Crunch {
    runtime: SupportedRuntime,
    client: Client<DefaultConfig>,
    matrix: Matrix,
}

impl Crunch {
    async fn new() -> Crunch {
        let client = create_or_await_substrate_node_client(CONFIG.clone()).await;

        let properties = client.properties();
        // Display SS58 addresses based on the connected chain
        let chain_prefix: ChainPrefix =
            if let Some(ss58_format) = properties.get("ss58Format") {
                ss58_format.as_u64().unwrap_or_default().try_into().unwrap()
            } else {
                0
            };
        crypto::set_default_ss58_version(crypto::Ss58AddressFormat::custom(chain_prefix));

        let chain_token_symbol: ChainTokenSymbol =
            if let Some(token_symbol) = properties.get("tokenSymbol") {
                match token_symbol {
                    JsonValue::String(token_symbol) => token_symbol.to_string(),
                    _ => unreachable!("Token symbol with wrong type"),
                }
            } else {
                String::from("")
            };

        // Check for supported runtime by token symbol
        let runtime = SupportedRuntime::from(chain_token_symbol.clone());

        // Initialize matrix client
        let mut matrix: Matrix = Matrix::new();
        matrix
            .authenticate(chain_token_symbol.into())
            .await
            .unwrap_or_else(|e| {
                error!("{}", e);
                Default::default()
            });

        Crunch {
            runtime,
            client,
            matrix,
        }
    }

    pub fn client(&self) -> &Client<DefaultConfig> {
        &self.client
    }

    /// Returns the matrix configuration
    pub fn matrix(&self) -> &Matrix {
        &self.matrix
    }

    pub async fn send_message(
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

    async fn inspect(&self) -> Result<(), CrunchError> {
        match self.runtime {
            SupportedRuntime::Polkadot => polkadot::inspect(self).await,
            SupportedRuntime::Kusama => kusama::inspect(self).await,
            SupportedRuntime::Westend => westend::inspect(self).await,
            SupportedRuntime::AlephZero => aleph_zero::inspect(self).await,
            SupportedRuntime::AlephZeroTestnet => aleph_zero_testnet::inspect(self).await,
            SupportedRuntime::Lagoon => lagoon::inspect(self).await,
        }
    }

    async fn try_run_batch(&self) -> Result<(), CrunchError> {
        match self.runtime {
            SupportedRuntime::Polkadot => polkadot::try_run_batch(self, None).await,
            SupportedRuntime::Kusama => kusama::try_run_batch(self, None).await,
            SupportedRuntime::Westend => westend::try_run_batch(self, None).await,
            SupportedRuntime::AlephZero => aleph_zero::try_run_batch(self, None).await,
            SupportedRuntime::AlephZeroTestnet => {
                aleph_zero_testnet::try_run_batch(self, None).await
            }
            SupportedRuntime::Lagoon => lagoon::try_run_batch(self, None).await,
        }
    }

    async fn run_and_subscribe_era_payout_events(&self) -> Result<(), CrunchError> {
        match self.runtime {
            SupportedRuntime::Polkadot => {
                polkadot::run_and_subscribe_era_paid_events(self).await
            }
            SupportedRuntime::Kusama => {
                kusama::run_and_subscribe_era_paid_events(self).await
            }
            SupportedRuntime::Westend => {
                westend::run_and_subscribe_era_paid_events(self).await
            }
            SupportedRuntime::AlephZero => {
                aleph_zero::run_and_subscribe_era_paid_events(self).await
            }
            SupportedRuntime::AlephZeroTestnet => {
                aleph_zero_testnet::run_and_subscribe_era_paid_events(self).await
            }
            SupportedRuntime::Lagoon => {
                lagoon::run_and_subscribe_era_paid_events(self).await
            }
        }
    }
}

fn spawn_and_restart_subscription_on_error() {
    let t = task::spawn(async {
        let config = CONFIG.clone();
        loop {
            let c: Crunch = Crunch::new().await;
            if let Err(e) = c.run_and_subscribe_era_payout_events().await {
                match e {
                    CrunchError::SubscriptionFinished => warn!("{}", e),
                    CrunchError::MatrixError(_) => warn!("Matrix message skipped!"),
                    _ => {
                        error!("{}", e);
                        let message =
                            format!("On hold for {} min!", config.error_interval);
                        let formatted_message = format!("<br/>🚨 An error was raised -> <code>crunch</code> on hold for {} min while rescue is on the way 🚁 🚒 🚑 🚓<br/><br/>", config.error_interval);
                        c.send_message(&message, &formatted_message).await.unwrap();
                        thread::sleep(time::Duration::from_secs(
                            60 * config.error_interval,
                        ));
                        continue;
                    }
                }
                thread::sleep(time::Duration::from_secs(1));
            };
        }
    });
    task::block_on(t);
}

fn spawn_and_restart_crunch_flakes_on_error() {
    let t = task::spawn(async {
        let config = CONFIG.clone();
        loop {
            let c: Crunch = Crunch::new().await;
            if let Err(e) = c.try_run_batch().await {
                match e {
                    CrunchError::MatrixError(_) => warn!("Matrix message skipped!"),
                    _ => {
                        error!("{}", e);
                        let message =
                            format!("On hold for {} min!", config.error_interval);
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
    task::block_on(t);
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

pub fn random_wait(max: u64) -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..max)
}

pub async fn try_fetch_stashes_from_remote_url(
) -> Result<Option<Vec<String>>, CrunchError> {
    let config = CONFIG.clone();
    if config.stashes_url.len() == 0 {
        return Ok(None);
    }
    let response = reqwest::get(&config.stashes_url).await?.text().await?;
    let v: Vec<String> = response.split('\n').map(|s| s.to_string()).collect();
    if v.is_empty() {
        return Ok(None);
    }
    info!("{} stashes loaded from {}", v.len(), config.stashes_url);
    Ok(Some(v))
}
