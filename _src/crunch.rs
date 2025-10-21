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
use crate::{
    config::CONFIG,
    errors::CrunchError,
    matrix::Matrix,
    runtimes::{
        // kusama,
        // paseo,
        // polkadot,
        support::{ChainPrefix, ChainTokenSymbol, SupportedRuntime},
        westend,
    },
};
use async_std::task;
use log::{debug, error, info, warn};
use rand::Rng;
use regex::Regex;
use serde::Deserialize;
use std::{convert::TryInto, fs, result::Result, str::FromStr, thread, time};

use sp_core::crypto;
use subxt::{
    backend::{
        legacy::{rpc_methods::StorageKey, LegacyRpcMethods},
        rpc::{
            reconnecting_rpc_client::{
                ExponentialBackoff, RpcClient as ReconnectingRpcClient,
            },
            RpcClient,
        },
    },
    ext::subxt_rpcs::utils::validate_url_is_secure,
    lightclient::{LightClient, LightClientError, LightClientRpc},
    utils::AccountId32,
    OnlineClient, SubstrateConfig,
};
use subxt_signer::{sr25519::Keypair, SecretUri};

pub type ValidatorIndex = Option<usize>;
pub type ValidatorAmount = u128;
pub type NominatorsAmount = u128;

#[allow(dead_code)]
type Message = Vec<String>;

#[allow(dead_code)]
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

// pub async fn create_substrate_rpc_client_from_config(
//     url: &str,
// ) -> Result<RpcClient, subxt::Error> {
//     if let Err(_) = validate_url_is_secure(url) {
//         warn!("Insecure URL provided: {}", url);
//     };
//     RpcClient::from_insecure_url(url).await
// }

pub async fn create_substrate_rpc_client_from_url(
    url: &str,
) -> Result<ReconnectingRpcClient, CrunchError> {
    if let Err(_) = validate_url_is_secure(url) {
        warn!("Insecure URL provided: {}", url);
    };
    info!("Using RPC endpoint {}", url);
    ReconnectingRpcClient::builder()
        .retry_policy(
            ExponentialBackoff::from_millis(100).max_delay(time::Duration::from_secs(10)),
        )
        .build(url.to_string())
        .await
        .map_err(|err| CrunchError::RpcError(err.into()))
}

pub async fn create_substrate_client_from_rpc_client(
    rpc_client: RpcClient,
) -> Result<OnlineClient<SubstrateConfig>, CrunchError> {
    OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client)
        .await
        .map_err(|err| CrunchError::SubxtError(err.into()))
}

pub async fn create_light_client_from_relay_chain_specs(
    chain: &str,
) -> Result<(LightClient, LightClientRpc), LightClientError> {
    let (lc, rpc) =
        LightClient::relay_chain(SupportedRuntime::from(chain).chain_specs())?;

    Ok((lc, rpc))
}

pub async fn create_light_client_from_people_chain_specs(
    chain: &str,
) -> Result<LightClientRpc, LightClientError> {
    let (lc, _) = create_light_client_from_relay_chain_specs(&chain).await?;
    let runtime = SupportedRuntime::from(chain);
    let people_runtime = runtime.people_runtime().unwrap();
    lc.parachain(people_runtime.chain_specs())
}

pub async fn create_light_client_from_asset_hub_chain_specs(
    chain: &str,
) -> Result<LightClientRpc, LightClientError> {
    let (lc, _) = create_light_client_from_relay_chain_specs(&chain).await?;
    let runtime = SupportedRuntime::from(chain);
    let asset_hub_runtime = runtime.asset_hub_runtime().unwrap();
    lc.parachain(asset_hub_runtime.chain_specs())
}

pub async fn create_substrate_rpc_client_from_config() -> Result<RpcClient, CrunchError> {
    let config = CONFIG.clone();

    if config.light_client_enabled {
        let (_, rpc) =
            create_light_client_from_relay_chain_specs(&config.chain_name).await?;
        return Ok(rpc.into());
    } else {
        let rpc = create_substrate_rpc_client_from_url(&config.substrate_ws_url).await?;
        return Ok(rpc.into());
    }
}

pub async fn create_or_await_substrate_node_client() -> (
    OnlineClient<SubstrateConfig>,
    LegacyRpcMethods<SubstrateConfig>,
    SupportedRuntime,
) {
    loop {
        match create_substrate_rpc_client_from_config().await {
            Ok(rpc_client) => {
                let legacy_rpc =
                    LegacyRpcMethods::<SubstrateConfig>::new(rpc_client.clone().into());
                let chain = legacy_rpc.system_chain().await.unwrap_or_default();
                let name = legacy_rpc.system_name().await.unwrap_or_default();
                let version = legacy_rpc.system_version().await.unwrap_or_default();
                let properties = legacy_rpc.system_properties().await.unwrap_or_default();

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
                    "Connected to {} network * client {} v{}",
                    chain, name, version
                );

                match create_substrate_client_from_rpc_client(rpc_client.clone()).await {
                    Ok(relay_client) => {
                        // Identify supported runtime based on token symbol
                        let runtime = SupportedRuntime::from(chain_token_symbol);
                        break (relay_client, legacy_rpc, runtime);
                    }
                    Err(e) => {
                        error!("{}", e);
                        thread::sleep(time::Duration::from_secs(6));
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
                thread::sleep(time::Duration::from_secs(6));
            }
        }
    }
}

pub async fn create_people_rpc_client_from_config() -> Result<RpcClient, CrunchError> {
    let config = CONFIG.clone();

    if config.light_client_enabled {
        let runtime = SupportedRuntime::from(config.chain_name.clone());
        if runtime.people_runtime().is_none() {
            return Err(CrunchError::Other(format!(
                "People chain not supported for the relay {}",
                runtime.to_string()
            )));
        }
        let rpc = create_light_client_from_people_chain_specs(&config.chain_name).await?;
        return Ok(rpc.into());
    } else {
        let rpc =
            create_substrate_rpc_client_from_url(&config.substrate_people_ws_url).await?;
        return Ok(rpc.into());
    }
}

pub async fn create_asset_hub_rpc_client_from_config() -> Result<RpcClient, CrunchError> {
    let config = CONFIG.clone();

    if config.light_client_enabled {
        let runtime = SupportedRuntime::from(config.chain_name.clone());
        if runtime.asset_hub_runtime().is_none() {
            return Err(CrunchError::Other(format!(
                "Asset Hub chain not supported for the relay {}",
                runtime.to_string()
            )));
        }
        let rpc =
            create_light_client_from_asset_hub_chain_specs(&config.chain_name).await?;
        return Ok(rpc.into());
    } else {
        let rpc =
            create_substrate_rpc_client_from_url(&config.substrate_asset_hub_ws_url)
                .await?;
        return Ok(rpc.into());
    }
}

pub async fn create_or_await_people_client() -> OnlineClient<SubstrateConfig> {
    loop {
        match create_people_rpc_client_from_config().await {
            Ok(rpc_client) => {
                let legacy_rpc =
                    LegacyRpcMethods::<SubstrateConfig>::new(rpc_client.clone().into());
                let chain = legacy_rpc.system_chain().await.unwrap_or_default();
                let name = legacy_rpc.system_name().await.unwrap_or_default();
                let version = legacy_rpc.system_version().await.unwrap_or_default();

                info!(
                    "Connected to {} network * Substrate node {} v{}",
                    chain, name, version
                );

                match create_substrate_client_from_rpc_client(rpc_client.clone()).await {
                    Ok(client) => {
                        break client;
                    }
                    Err(e) => {
                        error!("{}", e);
                        thread::sleep(time::Duration::from_secs(6));
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
                thread::sleep(time::Duration::from_secs(6));
            }
        }
    }
}

pub async fn create_or_await_asset_hub_client() -> (
    OnlineClient<SubstrateConfig>,
    LegacyRpcMethods<SubstrateConfig>,
) {
    loop {
        match create_asset_hub_rpc_client_from_config().await {
            Ok(rpc_client) => {
                let legacy_rpc =
                    LegacyRpcMethods::<SubstrateConfig>::new(rpc_client.clone().into());
                let chain = legacy_rpc.system_chain().await.unwrap_or_default();
                let name = legacy_rpc.system_name().await.unwrap_or_default();
                let version = legacy_rpc.system_version().await.unwrap_or_default();

                info!(
                    "Connected to {} network * Substrate node {} v{}",
                    chain, name, version
                );

                match create_substrate_client_from_rpc_client(rpc_client.clone()).await {
                    Ok(client) => {
                        break (client, legacy_rpc);
                    }
                    Err(e) => {
                        error!("{}", e);
                        thread::sleep(time::Duration::from_secs(6));
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
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
    // Note: Consider setting RC client API to become optional in chains where staking is already LIVE on Asset Hub;
    // Example, if substrate_ws_url is not provided then the Active/Inactive status for each stash is not displayed;
    client: OnlineClient<SubstrateConfig>,
    rpc: LegacyRpcMethods<SubstrateConfig>,
    // Note: AssetHub client API could stop being optional after all staking operations are mgrated to AH on all supported crunch chains.
    asset_hub_client_option: Option<OnlineClient<SubstrateConfig>>,
    asset_hub_rpc_option: Option<LegacyRpcMethods<SubstrateConfig>>,
    // Note: People client API is optional, if substrate_people_ws_url is not defined
    // identities are just not displayed and the full stash is displayed instead.
    people_client_option: Option<OnlineClient<SubstrateConfig>>,
    matrix: Matrix,
}

impl Crunch {
    async fn new() -> Crunch {
        let config = CONFIG.clone();

        let (client, rpc, runtime) = create_or_await_substrate_node_client().await;

        // Initialize people node client if supported and people url is defined
        let people_client_option = if let Some(people_runtime) = runtime.people_runtime()
        {
            if config.light_client_enabled {
                let people_client = create_or_await_people_client().await;
                Some(people_client)
            } else if !people_runtime.rpc_url().is_empty() {
                let people_client = create_or_await_people_client().await;
                Some(people_client)
            } else {
                None
            }
        } else {
            None
        };

        // Initialize AH node client if supported and AH url is defined
        let (asset_hub_client_option, asset_hub_rpc_option) = if let Some(ah_runtime) =
            runtime.asset_hub_runtime()
        {
            if config.light_client_enabled {
                let (ah_client, ah_rpc_client) = create_or_await_asset_hub_client().await;
                (Some(ah_client), Some(ah_rpc_client))
            } else if !ah_runtime.rpc_url().is_empty() {
                let (ah_client, ah_rpc_client) = create_or_await_asset_hub_client().await;
                (Some(ah_client), Some(ah_rpc_client))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

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
            asset_hub_client_option,
            asset_hub_rpc_option,
            people_client_option,
            matrix,
        }
    }

    pub fn client(&self) -> &OnlineClient<SubstrateConfig> {
        &self.client
    }

    pub fn asset_hub_client(&self) -> &Option<OnlineClient<SubstrateConfig>> {
        &self.asset_hub_client_option
    }

    pub fn asset_hub_rpc(&self) -> &Option<LegacyRpcMethods<SubstrateConfig>> {
        &self.asset_hub_rpc_option
    }

    pub fn people_client(&self) -> &Option<OnlineClient<SubstrateConfig>> {
        &self.people_client_option
    }

    pub fn rpc(&self) -> &LegacyRpcMethods<SubstrateConfig> {
        &self.rpc
    }

    /// Returns the matrix configuration
    pub fn matrix(&self) -> &Matrix {
        &self.matrix
    }

    pub fn subdomain(&self, is_staking_on_asset_hub: bool) -> String {
        self.runtime.subdomain(is_staking_on_asset_hub)
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

    /// Spawn crunch once task
    pub fn once() {
        spawn_crunch_once();
    }

    async fn validate_relay_genesis(&self) -> Result<(), CrunchError> {
        let api = self.client();
        let state_root = self.runtime.chain_state_root_hash();

        if let Some(header) = self
            .rpc()
            .chain_get_header(Some(api.genesis_hash()))
            .await?
        {
            if header.state_root != state_root {
                let config = CONFIG.clone();
                return Err(CrunchError::GenesisError(format!(
                    "verify {} endpoint {} as state root {}",
                    self.runtime, config.substrate_ws_url, header.state_root
                )));
            }
        }

        Ok(())
    }

    async fn validate_asset_hub_genesis(&self) -> Result<(), CrunchError> {
        let config = CONFIG.clone();

        if config.light_client_enabled {
            return Ok(());
        }

        if let Some(asset_hub_runtime) = self.runtime.asset_hub_runtime() {
            let state_root = asset_hub_runtime.chain_state_root_hash();

            let rpc_client =
                if let Err(_) = validate_url_is_secure(&asset_hub_runtime.rpc_url()) {
                    RpcClient::from_insecure_url(&asset_hub_runtime.rpc_url()).await?
                } else {
                    RpcClient::from_url(&asset_hub_runtime.rpc_url()).await?
                };

            let api = self
                .asset_hub_client()
                .as_ref()
                .expect("AH API to be available");

            let rpc = LegacyRpcMethods::<SubstrateConfig>::new(rpc_client.clone().into());
            if let Some(header) = rpc.chain_get_header(Some(api.genesis_hash())).await? {
                if header.state_root != state_root {
                    return Err(CrunchError::GenesisError(format!(
                        "verify {} endpoint {} as state root {}",
                        asset_hub_runtime.to_string(),
                        asset_hub_runtime.rpc_url(),
                        header.state_root
                    )));
                }
            }
        }

        Ok(())
    }

    async fn validate_people_genesis(&self) -> Result<(), CrunchError> {
        let config = CONFIG.clone();

        if config.light_client_enabled {
            return Ok(());
        }

        if let Some(people_runtime) = self.runtime.people_runtime() {
            let state_root = people_runtime.chain_state_root_hash();

            let rpc_client =
                if let Err(_) = validate_url_is_secure(&people_runtime.rpc_url()) {
                    RpcClient::from_insecure_url(&people_runtime.rpc_url()).await?
                } else {
                    RpcClient::from_url(&people_runtime.rpc_url()).await?
                };

            let api = self
                .people_client()
                .as_ref()
                .expect("People API to be available");

            let rpc = LegacyRpcMethods::<SubstrateConfig>::new(rpc_client.clone().into());
            if let Some(header) = rpc.chain_get_header(Some(api.genesis_hash())).await? {
                if header.state_root != state_root {
                    return Err(CrunchError::GenesisError(format!(
                        "verify {} endpoint {} as state root {}",
                        people_runtime.to_string(),
                        people_runtime.rpc_url(),
                        header.state_root
                    )));
                }
            }
        }

        Ok(())
    }

    pub async fn validate_genesis(&self) -> Result<(), CrunchError> {
        self.validate_relay_genesis().await?;
        self.validate_asset_hub_genesis().await?;
        self.validate_people_genesis().await?;

        Ok(())
    }

    async fn inspect(&self) -> Result<(), CrunchError> {
        self.validate_genesis().await?;
        match self.runtime {
            // SupportedRuntime::Polkadot => polkadot::inspect(self).await,
            // SupportedRuntime::Kusama => kusama::inspect(self).await,
            // SupportedRuntime::Paseo => paseo::inspect(self).await,
            SupportedRuntime::Westend => westend::inspect(self).await,
            _ => panic!("Unsupported runtime"),
        }
    }

    async fn try_run_batch(&self) -> Result<(), CrunchError> {
        self.validate_genesis().await?;
        match self.runtime {
            // SupportedRuntime::Polkadot => polkadot::try_crunch(self).await,
            // SupportedRuntime::Kusama => kusama::try_crunch(self).await,
            // SupportedRuntime::Paseo => paseo::try_crunch(self).await,
            SupportedRuntime::Westend => westend::try_crunch(self).await,
            _ => panic!("Unsupported runtime"),
        }
    }

    async fn run_and_subscribe_era_paid_events(&self) -> Result<(), CrunchError> {
        self.validate_genesis().await?;
        match self.runtime {
            // SupportedRuntime::Polkadot => {
            //     polkadot::run_and_subscribe_era_paid_events(self).await
            // }
            // SupportedRuntime::Kusama => {
            //     kusama::run_and_subscribe_era_paid_events(self).await
            // }
            // SupportedRuntime::Paseo => {
            //     paseo::run_and_subscribe_era_paid_events(self).await
            // }
            SupportedRuntime::Westend => {
                westend::run_and_subscribe_era_paid_events(self).await
            }
            _ => panic!("Unsupported runtime"),
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
                        let mut sleep_min = u32::pow(config.error_interval, n);
                        if sleep_min > config.maximum_error_interval {
                            sleep_min = config.maximum_error_interval;
                        }
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

fn spawn_crunch_once() {
    let crunch_task = task::spawn(async {
        let c: Crunch = Crunch::new().await;
        if let Err(e) = c.try_run_batch().await {
            error!("{}", e);
        };
    });
    task::block_on(crunch_task);
}

pub fn random_wait(max: u64) -> u64 {
    let mut rng = rand::rng();
    rng.random_range(0..max)
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
#[allow(dead_code)]
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

    let endpoint = if !config.onet_api_url.is_empty() {
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
                        Err(e) => {
                            error!(
                                "Unable to parse ONE-T response for stash {} error: {:?}",
                                stash, e
                            )
                        }
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

pub fn to_hex(bytes: impl AsRef<[u8]>) -> String {
    format!("0x{}", hex::encode(bytes.as_ref()))
}
