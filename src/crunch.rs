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
    kusama,
    // westend,
    paseo,
    polkadot,
    support::{ChainPrefix, ChainTokenSymbol, SupportedRuntime},
};
use async_std::task;
use log::{debug, error, info, warn};
use rand::Rng;
use regex::Regex;
use serde::Deserialize;
use std::{convert::TryInto, fs, result::Result, str::FromStr, thread, time};

use subxt::{
    backend::{
        legacy::{rpc_methods::StorageKey, LegacyRpcMethods},
        rpc::RpcClient,
    },
    ext::sp_core::crypto,
    utils::{validate_url_is_secure, AccountId32},
    OnlineClient, SubstrateConfig,
};

use subxt_signer::{sr25519::Keypair, SecretUri};

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

pub async fn create_substrate_rpc_client_from_config(
    config: Config,
) -> Result<RpcClient, subxt::Error> {
    if let Err(_) = validate_url_is_secure(config.substrate_ws_url.as_ref()) {
        warn!("Insecure URL provided: {}", config.substrate_ws_url);
    };
    RpcClient::from_insecure_url(config.substrate_ws_url).await
}

pub async fn create_substrate_client_from_rpc_client(
    rpc_client: RpcClient,
) -> Result<OnlineClient<SubstrateConfig>, subxt::Error> {
    OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client).await
}

pub async fn create_or_await_substrate_node_client(
    config: Config,
) -> (
    OnlineClient<SubstrateConfig>,
    LegacyRpcMethods<SubstrateConfig>,
    SupportedRuntime,
) {
    loop {
        match create_substrate_rpc_client_from_config(config.clone()).await {
            Ok(rpc_client) => {
                let rpc = LegacyRpcMethods::<SubstrateConfig>::new(rpc_client.clone());
                let chain = rpc.system_chain().await.unwrap_or_default();
                let name = rpc.system_name().await.unwrap_or_default();
                let version = rpc.system_version().await.unwrap_or_default();
                let properties = rpc.system_properties().await.unwrap_or_default();

                // Display SS58 addresses based on the connected chain
                let chain_prefix: ChainPrefix =
                    if let Some(ss58_format) = properties.get("ss58Format") {
                        ss58_format.as_u64().unwrap_or_default().try_into().unwrap()
                    } else {
                        0
                    };

                crypto::set_default_ss58_version(crypto::Ss58AddressFormat::custom(
                    chain_prefix,
                ));

                let chain_token_symbol: ChainTokenSymbol =
                    if let Some(token_symbol) = properties.get("tokenSymbol") {
                        use serde_json::Value::String;
                        match token_symbol {
                            String(token_symbol) => token_symbol.to_string(),
                            _ => unreachable!("Token symbol with wrong type"),
                        }
                    } else {
                        String::from("")
                    };

                info!(
                    "Connected to {} network using {} * Substrate node {} v{}",
                    chain, config.substrate_ws_url, name, version
                );

                match create_substrate_client_from_rpc_client(rpc_client.clone()).await {
                    Ok(client) => {
                        break (client, rpc, SupportedRuntime::from(chain_token_symbol));
                    }
                    Err(e) => {
                        error!("{}", e);
                        info!(
                            "Awaiting for connection using {}",
                            config.substrate_ws_url
                        );
                        thread::sleep(time::Duration::from_secs(6));
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
                info!("Awaiting for connection using {}", config.substrate_ws_url);
                thread::sleep(time::Duration::from_secs(6));
            }
        }
    }
}

// /// Helper function to generate a crypto pair from seed
// pub fn get_from_seed_DEPRECATED(seed: &str, pass: Option<&str>) -> sr25519::Pair {
//     // Use regex to remove control characters
//     let re = Regex::new(r"[\x00-\x1F]").unwrap();
//     let clean_seed = re.replace_all(&seed.trim(), "");
//     sr25519::Pair::from_string(&clean_seed, pass)
//         .expect("constructed from known-good static value; qed")
// }

/// Helper function to generate a keypair from the content of the seed file
pub fn get_keypair_from_seed_file() -> Result<Keypair, CrunchError> {
    let config = CONFIG.clone();

    // load data from seed file
    let data = fs::read_to_string(config.seed_path)?;

    // clear control characters from data
    let re = Regex::new(r"[\x00-\x1F]").unwrap();
    let data = re.replace_all(&data.trim(), "");

    // parse data into a secret
    let uri = SecretUri::from_str(&data)?;
    Ok(Keypair::from_uri(&uri)?)
}

pub struct Crunch {
    runtime: SupportedRuntime,
    client: OnlineClient<SubstrateConfig>,
    rpc: LegacyRpcMethods<SubstrateConfig>,
    matrix: Matrix,
}

impl Crunch {
    async fn new() -> Crunch {
        let (client, rpc, runtime) =
            create_or_await_substrate_node_client(CONFIG.clone()).await;

        // Initialize matrix client
        let mut matrix: Matrix = Matrix::new();
        matrix.authenticate(runtime).await.unwrap_or_else(|e| {
            error!("{}", e);
            Default::default()
        });

        Crunch {
            runtime,
            client,
            rpc,
            matrix,
        }
    }

    pub fn client(&self) -> &OnlineClient<SubstrateConfig> {
        &self.client
    }

    pub fn rpc(&self) -> &LegacyRpcMethods<SubstrateConfig> {
        &self.rpc
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
            SupportedRuntime::Paseo => paseo::inspect(self).await,
            // SupportedRuntime::Westend => westend::inspect(self).await,
            _ => unreachable!(),
        }
    }

    async fn try_run_batch(&self) -> Result<(), CrunchError> {
        match self.runtime {
            SupportedRuntime::Polkadot => polkadot::try_crunch(self).await,
            SupportedRuntime::Kusama => kusama::try_crunch(self).await,
            SupportedRuntime::Paseo => paseo::try_crunch(self).await,
            // SupportedRuntime::Westend => westend::try_crunch(self).await,
            _ => unreachable!(),
        }
    }

    async fn run_and_subscribe_era_paid_events(&self) -> Result<(), CrunchError> {
        match self.runtime {
            SupportedRuntime::Polkadot => {
                polkadot::run_and_subscribe_era_paid_events(self).await
            }
            SupportedRuntime::Kusama => {
                kusama::run_and_subscribe_era_paid_events(self).await
            }
            SupportedRuntime::Paseo => {
                paseo::run_and_subscribe_era_paid_events(self).await
            }
            // SupportedRuntime::Westend => {
            //     westend::run_and_subscribe_era_paid_events(self).await
            // }
            _ => unreachable!(),
        }
    }
}

fn spawn_and_restart_subscription_on_error() {
    let t = task::spawn(async {
        let config = CONFIG.clone();
        let mut n = 1_u32;
        loop {
            let c: Crunch = Crunch::new().await;
            if let Err(e) = c.run_and_subscribe_era_paid_events().await {
                match e {
                    CrunchError::SubscriptionFinished => warn!("{}", e),
                    CrunchError::MatrixError(_) => warn!("Matrix message skipped!"),
                    _ => {
                        error!("{}", e);
                        let sleep_min = u32::pow(config.error_interval, n);
                        let message = format!("On hold for {} min!", sleep_min);
                        let formatted_message = format!("<br/>🚨 An error was raised -> <code>crunch</code> on hold for {} min while rescue is on the way 🚁 🚒 🚑 🚓<br/><br/>", sleep_min);
                        c.send_message(&message, &formatted_message).await.unwrap();
                        thread::sleep(time::Duration::from_secs((60 * sleep_min).into()));
                        n += 1;
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
        let mut n = 1_u32;
        loop {
            let c: Crunch = Crunch::new().await;
            if let Err(e) = c.try_run_batch().await {
                let sleep_min = u32::pow(config.error_interval, n);
                match e {
                    CrunchError::MatrixError(_) => warn!("Matrix message skipped!"),
                    _ => {
                        error!("{}", e);
                        let message = format!("On hold for {} min!", sleep_min);
                        let formatted_message = format!("<br/>🚨 An error was raised -> <code>crunch</code> on hold for {} min while rescue is on the way 🚁 🚒 🚑 🚓<br/><br/>", sleep_min);
                        c.send_message(&message, &formatted_message).await.unwrap();
                    }
                }
                thread::sleep(time::Duration::from_secs((60 * sleep_min).into()));
                n += 1;
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

    let response = if config.github_pat.len() == 0 {
        // Fetch public remote file
        reqwest::get(&config.stashes_url).await?.text().await?
    } else {
        // Fetch github private remote file
        let client = reqwest::Client::new();
        client
            .get(&config.stashes_url)
            .header("Authorization", format!("token {}", config.github_pat))
            .header("Accept", "application/vnd.github.v4+raw")
            .send()
            .await?
            .text()
            .await?
    };

    let v: Vec<String> = response.trim().split('\n').map(|s| s.to_string()).collect();
    if v.is_empty() {
        return Ok(None);
    }
    info!("{} stashes loaded from {}", v.len(), config.stashes_url);
    Ok(Some(v))
}

#[derive(Deserialize, Clone, Debug)]
pub struct OnetData {
    pub address: String,
    pub grade: String,
    pub authority_inclusion: f64,
    pub para_authority_inclusion: f64,
    pub sessions: Vec<u32>,
}

pub async fn try_fetch_onet_data(
    chain_name: String,
    stash: AccountId32,
) -> Result<Option<OnetData>, CrunchError> {
    let config = CONFIG.clone();
    if !config.onet_api_enabled {
        return Ok(None);
    }

    let endpoint = if config.onet_api_url != "" {
        config.onet_api_url
    } else {
        format!("https://{}-onet-api.turboflakes.io", chain_name)
    };

    let url = format!(
        "{}/api/v1/validators/{}/grade?number_last_sessions={}",
        endpoint, stash, config.onet_number_last_sessions
    );

    debug!("Crunch <> ONE-T grade loaded from {}", url);
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("X-API-KEY", config.onet_api_key)
        .send()
        .await
    {
        Ok(response) => {
            match response.status() {
                reqwest::StatusCode::OK => {
                    match response.json::<OnetData>().await {
                        Ok(parsed) => return Ok(Some(parsed)),
                        Err(e) => error!(
                            "Unable to parse ONE-T response for stash {} error: {:?}",
                            stash, e
                        ),
                    };
                }
                other => {
                    warn!("Unexpected code {:?} from ONE-T url {}", other, url);
                }
            };
        }
        Err(e) => error!("{:?}", e),
    };
    Ok(None)
}

pub fn get_account_id_from_storage_key(key: StorageKey) -> AccountId32 {
    let s = &key[key.len() - 32..];
    let v: [u8; 32] = s.try_into().expect("slice with incorrect length");
    v.into()
}
