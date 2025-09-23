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

// Load environment variables into a Config struct
//
// Envy is a library for deserializing environment variables into
// typesafe structs
//
// Dotenv loads environment variables from a .env file, if available,
// and mashes those with the actual environment variables provided by
// the operative system.
//
// Set Config struct into a CONFIG lazy_static to avoid multiple processing.
//
use clap::{App, Arg, SubCommand};
use dotenv;
use lazy_static::lazy_static;
use log::{info, warn};
use serde::Deserialize;
use std::env;

// Set Config struct into a CONFIG lazy_static to avoid multiple processing
lazy_static! {
    pub static ref CONFIG: Config = get_config();
}

/// provides default value for interval if CRUNCH_INTERVAL env var is not set
fn default_interval() -> u64 {
    21600
}

/// provides default value for error interval if CRUNCH_ERROR_INTERVAL env var is not set
fn default_error_interval() -> u32 {
    2
}

/// provides default value for seed_path if CRUNCH_SEED_PATH env var is not set
fn default_seed_path() -> String {
    ".private.seed".into()
}

/// provides default value for maximum_payouts if CRUNCH_MAXIMUM_PAYOUTS env var is not set
fn default_maximum_payouts() -> u32 {
    4
}

/// provides default value for maximum_history_eras if CRUNCH_MAXIMUM_HISTORY_ERAS env var is not set
fn default_maximum_history_eras() -> u32 {
    4
}

/// provides default value for maximum_calls if CRUNCH_MAXIMUM_CALLS env var is not set
fn default_maximum_calls() -> u32 {
    2
}

/// provides default value for existential_deposit_factor_warning if CRUNCH_EXISTENTIAL_DEPOSIT_FACTOR_WARNING env var is not set
/// polkadot 2x
/// kusama 1000x
fn default_existential_deposit_factor_warning() -> u32 {
    2
}

/// provides the default tip in PLANCKS for the block author
fn default_tx_tip() -> u64 {
    0
}

/// provides the default number of blocks the transaction is mortal for
fn default_tx_mortal_period() -> u64 {
    0
}

/// provides default value for pool_compound_threshold if CRUNCH_POOL_COMPOUND_THRESHOLD env var is not set
fn default_pool_compound_threshold() -> u64 {
    100000000000
}

/// provides default value for maximum_pool_calls if CRUNCH_MAXIMUM_POOL_CALLS env var is not set
fn default_maximum_pool_calls() -> u32 {
    128
}

/// provides default value for onet_api_key if CRUNCH_ONET_API_KEY env var is not set
fn default_onet_api_key() -> String {
    "crunch-101".into()
}

/// provides default value for onet_number_last_sessions if CRUNCH_ONET_NUMBER_LAST_SESSIONS env var is not set
fn default_onet_number_last_sessions() -> u32 {
    6
}

/// provides default value for run_mode
fn default_run_mode() -> RunMode {
    RunMode::Era
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_interval")]
    pub interval: u64,
    #[serde(default = "default_error_interval")]
    pub error_interval: u32,
    #[serde(default)]
    pub substrate_ws_url: String,
    #[serde(default)]
    pub substrate_people_ws_url: String,
    #[serde(default)]
    pub substrate_asset_hub_ws_url: String,
    #[serde(default)]
    pub stashes_url: String,
    #[serde(default)]
    pub github_pat: String,
    #[serde(default)]
    pub pool_ids: Vec<u32>,
    #[serde(default)]
    pub pool_active_nominees_payout_enabled: bool,
    #[serde(default)]
    pub pool_all_nominees_payout_enabled: bool,
    #[serde(default)]
    pub pool_claim_commission_enabled: bool,
    #[serde(default)]
    pub pool_members_compound_enabled: bool,
    #[serde(default)]
    pub pool_only_operator_compound_enabled: bool,
    #[serde(default = "default_pool_compound_threshold")]
    pub pool_compound_threshold: u64,
    #[serde(default = "default_maximum_pool_calls")]
    pub maximum_pool_calls: u32,
    #[serde(default)]
    pub unique_stashes_enabled: bool,
    #[serde(default)]
    pub group_identity_enabled: bool,
    #[serde(default = "default_seed_path")]
    pub seed_path: String,
    pub stashes: Vec<String>,
    #[serde(default = "default_maximum_payouts")]
    pub maximum_payouts: u32,
    #[serde(default = "default_maximum_history_eras")]
    pub maximum_history_eras: u32,
    #[serde(default = "default_maximum_calls")]
    pub maximum_calls: u32,
    #[serde(default = "default_existential_deposit_factor_warning")]
    pub existential_deposit_factor_warning: u32,
    #[serde(default = "default_tx_tip")]
    pub tx_tip: u64,
    #[serde(default = "default_tx_mortal_period")]
    pub tx_mortal_period: u64,
    #[serde(default)]
    pub only_view: bool,
    #[serde(default)]
    pub is_debug: bool,
    #[serde(default)]
    pub is_boring: bool,
    #[serde(default)]
    pub is_short: bool,
    #[serde(default)]
    pub is_medium: bool,
    #[serde(default = "default_run_mode")]
    pub run_mode: RunMode,
    // ONE-T integration
    #[serde(default)]
    pub onet_api_enabled: bool,
    #[serde(default)]
    pub onet_api_url: String,
    #[serde(default = "default_onet_api_key")]
    pub onet_api_key: String,
    #[serde(default = "default_onet_number_last_sessions")]
    pub onet_number_last_sessions: u32,
    // matrix configuration
    #[serde(default)]
    pub matrix_user: String,
    #[serde(default)]
    pub matrix_bot_user: String,
    #[serde(default)]
    pub matrix_bot_password: String,
    #[serde(default)]
    pub matrix_disabled: bool,
    #[serde(default)]
    pub matrix_public_room_disabled: bool,
    #[serde(default)]
    pub matrix_bot_display_name_disabled: bool,
    // light client configuration
    #[serde(default)]
    pub light_client_enabled: bool,
    #[serde(default)]
    pub chain_name: String,
}

#[derive(Default, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RunMode {
    #[default]
    Era,
    Daily,
    Turbo,
    Once,
}

/// Inject dotenv and env vars into the Config struct
fn get_config() -> Config {
    // Define CLI flags with clap
    let matches = App::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .arg(
      Arg::with_name("CHAIN")
          .index(1)
          .possible_values(&["kusama", "polkadot", "paseo", "westend"])
          .takes_value(true)
          .help(
            "Sets the substrate-based chain for which 'crunch' will try to connect",
          )
    )
    .subcommand(SubCommand::with_name("flakes")
      .about("Crunch awesome flakes (rewards) every era, daily or in turbo mode -> 4x faster")
      .arg(
        Arg::with_name("MODE")
            .index(1)
            .possible_values(&["era", "daily", "turbo", "once"])
            .default_value("era")
            .help(
              "Sets how often flakes (staking rewards) should be crunched (claimed) from unclaimed eras. (e.g. the option 'era' sets 'crunch' task to run as soon as the EraPaid on-chain event is triggered; the option 'daily' sets 'crunch' task to be repeated every 24 hours; option 'turbo' sets 'crunch' task to be repeated every 6 hours; option 'once' tries to run the payout once and exit;)",
            ))
      .arg(
        Arg::with_name("seed-path")
          .short("f")
          .long("seed-path")
          .takes_value(true)
          .value_name("FILE")
          .help(
            "Sets a custom seed file path. The seed file contains the private seed phrase to Sign the extrinsic payout call.",
          ))
      .arg(
        Arg::with_name("maximum-payouts")
          .short("m")
          .long("maximum-payouts")
          .takes_value(true)
          .help("Maximum number of unclaimed eras for which an extrinsic payout will be submitted. (e.g. a value of 4 means that if there are unclaimed eras in the last 84 the maximum unclaimed payout calls for each stash address will be 4)."))
      .arg(
        Arg::with_name("maximum-history-eras")
              .long("maximum-history-eras")
              .takes_value(true)
              .help("Maximum number of history eras for which crunch will look for unclaimed rewards. The maximum value supported is the one defined by the constant history_depth - usually 84 - (e.g. a value of 4 means that crunch will only check in latest 4 eras if there are any unclaimed rewards for each stash address). [default: 4]"))
      .arg(
        Arg::with_name("maximum-calls")
              .long("maximum-calls")
              .takes_value(true)
              .help("Maximum number of calls in a single batch. [default: 3]"))
      .arg(
        Arg::with_name("debug")
          .long("debug")
          .help("Prints debug information verbosely."))
      .arg(
        Arg::with_name("matrix-user")
          .long("matrix-user")
          .takes_value(true)
          .help("Your regular matrix user. e.g. '@your-regular-matrix-account:matrix.org' this user account will receive notifications from your other 'Crunch Bot' matrix account."))
      .arg(
            Arg::with_name("matrix-bot-user")
              .long("matrix-bot-user")
              .takes_value(true)
              .help("Your new 'Crunch Bot' matrix user. e.g. '@your-own-crunch-bot-account:matrix.org' this user account will be your 'Crunch Bot' which will be responsible to send messages/notifications to your private or public 'Crunch Bot' rooms."))
      .arg(
        Arg::with_name("matrix-bot-password")
          .long("matrix-bot-password")
          .takes_value(true)
          .help("Password for the 'Crunch Bot' matrix user sign in."))
      .arg(
        Arg::with_name("disable-matrix")
          .long("disable-matrix")
          .help(
            "Disable matrix bot for 'crunch flakes'. (e.g. with this flag active 'crunch flakes' will not send messages/notifications about claimed or unclaimed staking rewards to your private or public 'Crunch Bot' rooms) (https://matrix.org/)",
          ))
      .arg(
        Arg::with_name("disable-public-matrix-room")
          .long("disable-public-matrix-room")
          .help(
            "Disable notifications to matrix public rooms for 'crunch flakes'. (e.g. with this flag active 'crunch flakes' will not send messages/notifications about claimed or unclaimed staking rewards to any public 'Crunch Bot' room)",
          ))
      .arg(
        Arg::with_name("disable-matrix-bot-display-name")
          .long("disable-matrix-bot-display-name")
          .help(
            "Disable matrix bot display name update for 'crunch flakes'. (e.g. with this flag active 'crunch flakes' will not change the matrix bot user display name)",
          ))
      .arg(
        Arg::with_name("short")
          .long("short")
          .help("Display only minimum information (e.g. with this flag active 'crunch rewards' will send minimum verbose messages/notifications about claimed rewards)"))
      .arg(
        Arg::with_name("medium")
          .long("medium")
          .help("Display essential information (e.g. with this flag active 'crunch rewards' will send essential verbose messages/notifications about claimed rewards like points and validator rewards)"))
      .arg(
        Arg::with_name("error-interval")
          .long("error-interval")
          .takes_value(true)
          .help("Interval value (in minutes) from which 'crunch' will restart again in case of a critical error."))
      .arg(
        Arg::with_name("pool-ids")
          .long("pool-ids")
          .takes_value(true)
          .help(
            "Nomination pool ids for which 'crunch' will try to fetch the validator stash addresses (e.g. poll_id_1, pool_id_2).",
          ))
      .arg(
        Arg::with_name("tx-tip")
          .long("tx-tip")
          .takes_value(true)
          .help(
            "Define a tip in PLANCKS for the block author.",
          ))
      .arg(
        Arg::with_name("tx-mortal-period")
          .long("tx-mortal-period")
          .takes_value(true)
          .help(
            "Define the number of blocks the transaction is mortal for.",
          ))
      .arg(
        Arg::with_name("enable-pool-compound-threshold")
          .long("enable-pool-compound-threshold")
          .takes_value(true)
          .help(
            "Define minimum pending rewards threshold in PLANCKS. (e.g. Only pending rewards above the threshold are include in the auto-compound batch)",
          ))
      .arg(
        Arg::with_name("enable-pool-members-compound")
          .long("enable-pool-members-compound")
          .help(
            "Allow 'crunch' to compound rewards for every member that belongs to the pools previously selected by '--pool-ids' option. Note that members have to have their permissions set as PermissionlessCompound or PermissionlessAll.",
          ))
      .arg(
        Arg::with_name("enable-pool-claim-commission")
          .long("enable-pool-claim-commission")
          .help(
            "Allow 'crunch' to claim the pool commission. Note that the nomination pool root account has to explicitly set this feature via extrinsic `set_commission_claim_permission`.",
          ))
      .arg(
        Arg::with_name("enable-pool-only-operator-compound")
          .long("enable-pool-only-operator-compound")
          .help(
            "Allow 'crunch' to compound rewards for the pool operator member that belongs to the pools previously selected by '--pool-ids' option. Note that the operator member account have to have their permissions set as PermissionlessCompound or PermissionlessAll.",
          ))
      .arg(
        Arg::with_name("enable-pool-active-nominees-payout")
          .long("enable-pool-active-nominees-payout")
          .help(
            "Enable payouts only for ACTIVE nominees assigned to the Nomination Pools defined in 'pool-ids'. (e.g. with this flag active 'crunch' will try to trigger payouts only for the ACTIVE nominees and not all).",
          ))
      .arg(
        Arg::with_name("enable-pool-all-nominees-payout")
          .long("enable-pool-all-nominees-payout")
          .help(
            "Enable payouts for ALL the nominees assigned to the Nomination Pools defined in 'pool-ids'. (e.g. with this flag active 'crunch' will try to trigger payouts for ALL nominees and not only the active ones - the ones the stake of the Nomination Pool was allocated).",
          ))
      .arg(
        Arg::with_name("enable-onet-api")
          .long("enable-onet-api")
          .help(
            "Allow 'crunch' to fetch grades for every stash from ONE-T API.",
          ))
    )
    .subcommand(SubCommand::with_name("rewards")
      .about("Claim staking rewards for unclaimed eras once a day or four times a day [default subcommand]")
      .arg(
        Arg::with_name("MODE")
            .index(1)
            .possible_values(&["era", "daily", "turbo", "once"])
            .default_value("era")
            .help(
              "Sets how often staking rewards should be claimed from unclaimed eras. (e.g. the option 'era' sets 'crunch' task to run as soon as the EraPaid on-chain event is triggered; the option 'daily' sets 'crunch' task to be repeated every 24 hours; option 'turbo' sets 'crunch' task to be repeated every 6 hours;option 'once' tries to run the payout once and exit;)",
            ))
      .arg(
        Arg::with_name("seed-path")
          .short("f")
          .long("seed-path")
          .takes_value(true)
          .value_name("FILE")
          .help(
            "Sets a custom seed file path. The seed file contains the private seed phrase to Sign the extrinsic payout call.",
          ))
      .arg(
        Arg::with_name("maximum-payouts")
          .short("m")
          .long("maximum-payouts")
          .takes_value(true)
          .help("Maximum number of unclaimed eras for which an extrinsic payout will be submitted. (e.g. a value of 4 means that if there are unclaimed eras in the last 84 the maximum unclaimed payout calls for each stash address will be 4)."))
      .arg(
        Arg::with_name("maximum-history-eras")
              .long("maximum-history-eras")
              .takes_value(true)
              .help("Maximum number of history eras for which crunch will look for unclaimed rewards. The maximum value supported is the one defined by the constant history_depth - usually 84 - (e.g. a value of 4 means that crunch will only check in latest 4 eras if there are any unclaimed rewards for each stash address). [default: 4]"))
      .arg(
        Arg::with_name("maximum-calls")
              .long("maximum-calls")
              .takes_value(true)
              .help("Maximum number of calls in a single batch. [default: 8]"))
      .arg(
        Arg::with_name("debug")
          .long("debug")
          .help("Prints debug information verbosely."))
      .arg(
        Arg::with_name("matrix-user")
          .long("matrix-user")
          .takes_value(true)
          .help("Your regular matrix user. e.g. '@your-regular-matrix-account:matrix.org' this user account will receive notifications from your other 'Crunch Bot' matrix account."))
      .arg(
            Arg::with_name("matrix-bot-user")
              .long("matrix-bot-user")
              .takes_value(true)
              .help("Your new 'Crunch Bot' matrix user. e.g. '@your-own-crunch-bot-account:matrix.org' this user account will be your 'Crunch Bot' which will be responsible to send messages/notifications to your private or public 'Crunch Bot' rooms."))
      .arg(
        Arg::with_name("matrix-bot-password")
          .long("matrix-bot-password")
          .takes_value(true)
          .help("Password for the 'Crunch Bot' matrix user sign in."))
      .arg(
        Arg::with_name("disable-matrix")
          .long("disable-matrix")
          .help(
            "Disable matrix bot for 'crunch rewards'. (e.g. with this flag active 'crunch rewards' will not send messages/notifications about claimed or unclaimed staking rewards to your private or public 'Crunch Bot' rooms) (https://matrix.org/)",
          ))
      .arg(
        Arg::with_name("disable-public-matrix-room")
          .long("disable-public-matrix-room")
          .help(
            "Disable notifications to matrix public rooms for 'crunch rewards'. (e.g. with this flag active 'crunch rewards' will not send messages/notifications about claimed or unclaimed staking rewards to any public 'Crunch Bot' room)",
          ))
      .arg(
        Arg::with_name("disable-matrix-bot-display-name")
          .long("disable-matrix-bot-display-name")
          .help(
            "Disable matrix bot display name update for 'crunch rewards'. (e.g. with this flag active 'crunch rewards' will not change the matrix bot user display name)",
          ))
      .arg(
        Arg::with_name("short")
          .long("short")
          .help("Display only minimum information (e.g. with this flag active 'crunch rewards' will send minimum verbose messages/notifications about claimed rewards)"))
      .arg(
        Arg::with_name("medium")
          .long("medium")
          .help("Display essential information (e.g. with this flag active 'crunch rewards' will send essential verbose messages/notifications about claimed rewards like points and validator rewards)"))
      .arg(
        Arg::with_name("error-interval")
          .long("error-interval")
          .takes_value(true)
          .help("Interval value (in minutes) from which 'crunch' will restart again in case of a critical error."))
      .arg(
        Arg::with_name("pool-ids")
          .long("pool-ids")
          .takes_value(true)
          .help(
            "Nomination pool ids for which 'crunch' will try to fetch the validator stash addresses (e.g. poll_id_1, pool_id_2).",
          ))
      .arg(
        Arg::with_name("tx-tip")
          .long("tx-tip")
          .takes_value(true)
          .help(
            "Define a tip in PLANCKS for the block author.",
          ))
      .arg(
        Arg::with_name("tx-mortal-period")
          .long("tx-mortal-period")
          .takes_value(true)
          .help(
            "Define the number of blocks the transaction is mortal for (default is 64 blocks)",
          ))
      .arg(
        Arg::with_name("enable-pool-compound-threshold")
          .long("enable-pool-compound-threshold")
          .takes_value(true)
          .help(
            "Define minimum pending rewards threshold in PLANCKS. (e.g. Only pending rewards above the threshold are include in the auto-compound batch)",
          ))
      .arg(
        Arg::with_name("enable-pool-members-compound")
          .long("enable-pool-members-compound")
          .help(
            "Allow 'crunch' to compound rewards for every member that belongs to the pools previously selected by '--pool-ids' option. Note that members have to have their permissions set as PermissionlessCompound or PermissionlessAll.",
          ))
      .arg(
        Arg::with_name("enable-pool-claim-commission")
          .long("enable-pool-claim-commission")
          .help(
            "Allow 'crunch' to claim the pool commission. Note that the nomination pool root account has to explicitly set this feature via extrinsic `set_commission_claim_permission`.",
          ))
      .arg(
        Arg::with_name("enable-pool-only-operator-compound")
          .long("enable-pool-only-operator-compound")
          .help(
            "Allow 'crunch' to compound rewards for the pool operator member that belongs to the pools previously selected by '--pool-ids' option. Note that the operator member account have to have their permissions set as PermissionlessCompound or PermissionlessAll.",
          ))
      .arg(
        Arg::with_name("enable-pool-active-nominees-payout")
          .long("enable-pool-active-nominees-payout")
          .help(
            "Enable payouts only for ACTIVE nominees assigned to the Nomination Pools defined in 'pool-ids'. (e.g. with this flag active 'crunch' will try to trigger payouts only for the ACTIVE nominees and not all).",
          ))
      .arg(
        Arg::with_name("enable-pool-all-nominees-payout")
          .long("enable-pool-all-nominees-payout")
          .help(
            "Enable payouts for ALL the nominees assigned to the Nomination Pools defined in 'pool-ids'. (e.g. with this flag active 'crunch' will try to trigger payouts for ALL nominees and not only the active ones - the ones the stake of the Nomination Pool was allocated).",
          ))
      .arg(
        Arg::with_name("enable-onet-api")
          .long("enable-onet-api")
          .help(
            "Allow 'crunch' to fetch grades for every stash from ONE-T API.",
          ))
    )
    .subcommand(SubCommand::with_name("view")
      .about("Inspect staking rewards for the given stashes and display claimed and unclaimed eras.")
    )
    .arg(
      Arg::with_name("stashes")
        .short("s")
        .long("stashes")
        .takes_value(true)
        .help(
          "Validator stash addresses for which 'crunch view', 'crunch flakes' or 'crunch rewards' will be applied. If needed specify more than one (e.g. stash_1,stash_2,stash_3).",
        ))
    .arg(
      Arg::with_name("stashes-url")
        .long("stashes-url")
        .takes_value(true)
        .help(
          "Remote stashes endpoint for which 'crunch' will try to fetch the validator stash addresses (e.g. https://raw.githubusercontent.com/turboflakes/crunch/main/.remote.stashes.example).",
        ))
    .arg(
      Arg::with_name("github-pat")
        .long("github-pat")
        .takes_value(true)
        .help(
          "Github Personal Access Token with read access to the private repo defined at 'stashes-url'.",
      ))
    .arg(
      Arg::with_name("enable-unique-stashes")
        .long("enable-unique-stashes")
        .help(
          "From all given stashes crunch will Sort by stash adddress and Remove duplicates.",
        ))
    .arg(
      Arg::with_name("enable-light-client")
        .long("enable-light-client")
        .help(
          "Enable lightweight client to connect to substrate-based chains. With this option enabled there is no need to specify specific RPCs endpoints for 'substrate-ws-url' or 'substrate-people-ws-url'",
        ))
    .arg(
      Arg::with_name("enable-group-identity")
        .long("enable-group-identity")
        .help(
          "Enables payouts and messages to be grouped and processed by main identity.",
        ))
    .arg(
      Arg::with_name("substrate-ws-url")
        .short("w")
        .long("substrate-ws-url")
        .takes_value(true)
        .help(
          "Substrate websocket endpoint for which 'crunch' will try to connect. (e.g. wss://polkadot.rpc.turboflakes.io:443) (NOTE: substrate_ws_url takes precedence than <CHAIN> argument)",
        ))
    .arg(
      Arg::with_name("substrate-people-ws-url")
        .long("substrate-people-ws-url")
        .takes_value(true)
        .help(
          "Substrate websocket endpoint for which 'crunch' will try to connect and retrieve identities from. (e.g. wss://people-polkadot.rpc.turboflakes.io:443)",
        ),
    )
    .arg(
      Arg::with_name("substrate-asset-hub-ws-url")
        .long("substrate-asset-hub-ws-url")
        .takes_value(true)
        .help(
          "NOTE: Only available for Paseo or Westend chains. Substrate websocket endpoint for which 'crunch' will try to connect and crunch rewards from. (e.g. wss://asset-hub-paseo.rpc.turboflakes.io:443)",
        ),
    )
    .arg(
      Arg::with_name("config-path")
        .short("c")
        .long("config-path")
        .takes_value(true)
        .value_name("FILE")
        .default_value(".env")
        .help(
          "Sets a custom config file path. The config file contains 'crunch' configuration variables.",
        ))
    .get_matches();

    // Try to load configuration from file first
    let config_path = matches.value_of("config-path").unwrap_or(".env");
    match dotenv::from_filename(&config_path).ok() {
        Some(_) => info!("Loading configuration from {} file", &config_path),
        None => {
            let config_path =
                env::var("CRUNCH_CONFIG_FILENAME").unwrap_or(".env".to_string());
            if let Some(_) = dotenv::from_filename(&config_path).ok() {
                info!("Loading configuration from {} file", &config_path);
            }
        }
    }

    match matches.value_of("CHAIN") {
        Some("westend") => {
            env::set_var(
                "CRUNCH_SUBSTRATE_WS_URL",
                "wss://westend.rpc.turboflakes.io:443",
            );
            if env::var("CRUNCH_SUBSTRATE_PEOPLE_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_PEOPLE_WS_URL",
                    "wss://people-westend.rpc.turboflakes.io:443",
                );
            }
            if env::var("CRUNCH_SUBSTRATE_ASSET_HUB_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_ASSET_HUB_WS_URL",
                    "wss://asset-hub-westend.rpc.turboflakes.io:443",
                );
            }
            env::set_var("CRUNCH_CHAIN_NAME", "westend");
        }
        Some("kusama") => {
            if env::var("CRUNCH_SUBSTRATE_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_WS_URL",
                    "wss://kusama.rpc.turboflakes.io:443",
                );
            }
            if env::var("CRUNCH_SUBSTRATE_PEOPLE_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_PEOPLE_WS_URL",
                    "wss://people-kusama.rpc.turboflakes.io:443",
                );
            }
            if env::var("CRUNCH_SUBSTRATE_ASSET_HUB_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_ASSET_HUB_WS_URL",
                    "wss://asset-hub-kusama.rpc.turboflakes.io:443",
                );
            }
            env::set_var("CRUNCH_CHAIN_NAME", "kusama");
        }
        Some("polkadot") => {
            if env::var("CRUNCH_SUBSTRATE_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_WS_URL",
                    "wss://polkadot.rpc.turboflakes.io:443",
                );
            }
            if env::var("CRUNCH_SUBSTRATE_PEOPLE_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_PEOPLE_WS_URL",
                    "wss://people-polkadot.rpc.turboflakes.io:443",
                );
            }
            if env::var("CRUNCH_SUBSTRATE_ASSET_HUB_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_ASSET_HUB_WS_URL",
                    "wss://asset-hub-polkadot.rpc.turboflakes.io:443",
                );
            }
            env::set_var("CRUNCH_CHAIN_NAME", "polkadot");
        }
        Some("paseo") => {
            if env::var("CRUNCH_SUBSTRATE_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_WS_URL",
                    "wss://paseo.rpc.turboflakes.io:443",
                );
            }
            if env::var("CRUNCH_SUBSTRATE_PEOPLE_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_PEOPLE_WS_URL",
                    "wss://people-paseo.rpc.turboflakes.io:443",
                );
            }
            if env::var("CRUNCH_SUBSTRATE_ASSET_HUB_WS_URL").is_err() {
                env::set_var(
                    "CRUNCH_SUBSTRATE_ASSET_HUB_WS_URL",
                    "wss://asset-hub-paseo.rpc.turboflakes.io:443",
                );
            }
            env::set_var("CRUNCH_CHAIN_NAME", "paseo");
        }
        _ => {
            if env::var("CRUNCH_SUBSTRATE_WS_URL").is_err() {
                env::set_var("CRUNCH_SUBSTRATE_WS_URL", "ws://127.0.0.1:9944");
            };
        }
    }

    if let Some(substrate_ws_url) = matches.value_of("substrate-ws-url") {
        env::set_var("CRUNCH_SUBSTRATE_WS_URL", substrate_ws_url);
    }

    if let Some(substrate_people_ws_url) = matches.value_of("substrate-people-ws-url") {
        env::set_var("CRUNCH_SUBSTRATE_PEOPLE_WS_URL", substrate_people_ws_url);
    }

    if let Some(substrate_asset_hub_ws_url) =
        matches.value_of("substrate-asset-hub-ws-url")
    {
        env::set_var(
            "CRUNCH_SUBSTRATE_ASSET_HUB_WS_URL",
            substrate_asset_hub_ws_url,
        );
    }

    if let Some(seed_path) = matches.value_of("seed-path") {
        env::set_var("CRUNCH_SEED_PATH", seed_path);
    }

    if let Some(stashes_url) = matches.value_of("stashes-url") {
        env::set_var("CRUNCH_STASHES_URL", stashes_url);
    }

    if let Some(github_pat) = matches.value_of("github-pat") {
        env::set_var("CRUNCH_GITHUB_PAT", github_pat);
    }

    if let Some(stashes) = matches.value_of("stashes") {
        env::set_var("CRUNCH_STASHES", stashes);
    }

    if matches.is_present("enable-unique-stashes") {
        env::set_var("CRUNCH_UNIQUE_STASHES_ENABLED", "true");
    }

    if matches.is_present("enable-light-client") {
        env::set_var("CRUNCH_LIGHT_CLIENT_ENABLED", "true");
    }

    if matches.is_present("enable-group-identity") {
        env::set_var("CRUNCH_GROUP_IDENTITY_ENABLED", "true");
    }

    match matches.subcommand() {
        ("flakes", Some(flakes_matches)) | ("rewards", Some(flakes_matches)) => {
            let mode = flakes_matches.value_of("MODE").unwrap_or_default();
            env::set_var("CRUNCH_RUN_MODE", mode);
            match mode {
                "daily" => {
                    env::set_var("CRUNCH_INTERVAL", "86400");
                }
                "turbo" => {
                    env::set_var("CRUNCH_INTERVAL", "21600");
                }
                _ => {}
            }

            if let Some(seed_path) = flakes_matches.value_of("seed-path") {
                env::set_var("CRUNCH_SEED_PATH", seed_path);
            }

            if let Some(maximum_payouts) = flakes_matches.value_of("maximum-payouts") {
                env::set_var("CRUNCH_MAXIMUM_PAYOUTS", maximum_payouts);
            }

            if let Some(maximum_history_eras) =
                flakes_matches.value_of("maximum-history-eras")
            {
                env::set_var("CRUNCH_MAXIMUM_HISTORY_ERAS", maximum_history_eras);
            }

            if let Some(maximum_calls) = flakes_matches.value_of("maximum-calls") {
                env::set_var("CRUNCH_MAXIMUM_CALLS", maximum_calls);
            }

            if flakes_matches.is_present("debug") {
                env::set_var("CRUNCH_IS_DEBUG", "true");
            }

            if flakes_matches.is_present("short") {
                env::set_var("CRUNCH_IS_SHORT", "true");
            }

            if flakes_matches.is_present("medium") {
                env::set_var("CRUNCH_IS_MEDIUM", "true");
            }

            if flakes_matches.is_present("subscribe") {
                env::set_var("CRUNCH_IS_SUBSCRIPTION", "true");
            }

            if flakes_matches.is_present("disable-matrix") {
                env::set_var("CRUNCH_MATRIX_DISABLED", "true");
            }

            if flakes_matches.is_present("disable-public-matrix-room") {
                env::set_var("CRUNCH_MATRIX_PUBLIC_ROOM_DISABLED", "true");
            }

            if let Some(matrix_user) = flakes_matches.value_of("matrix-user") {
                env::set_var("CRUNCH_MATRIX_ACCOUNT", matrix_user);
            }

            if let Some(matrix_bot_user) = flakes_matches.value_of("matrix-bot-user") {
                env::set_var("CRUNCH_MATRIX_BOT_USER", matrix_bot_user);
            }

            if let Some(matrix_bot_password) =
                flakes_matches.value_of("matrix-bot-password")
            {
                env::set_var("CRUNCH_MATRIX_BOT_PASSWORD", matrix_bot_password);
            }

            if let Some(error_interval) = flakes_matches.value_of("error-interval") {
                env::set_var("CRUNCH_ERROR_INTERVAL", error_interval);
            }

            if let Some(pool_ids) = flakes_matches.value_of("pool-ids") {
                env::set_var("CRUNCH_POOL_IDS", pool_ids);
            }

            if let Some(tx_tip) = flakes_matches.value_of("tx-tip") {
                env::set_var("CRUNCH_TX_TIP", tx_tip);
            }

            if let Some(tx_mortal_period) = flakes_matches.value_of("tx-mortal-period") {
                env::set_var("CRUNCH_TX_MORTAL_PERIOD", tx_mortal_period);
            }

            if let Some(threshold) =
                flakes_matches.value_of("enable-pool-compound-threshold")
            {
                env::set_var("CRUNCH_POOL_COMPOUND_THRESHOLD", threshold);
            }

            if flakes_matches.is_present("enable-pool-only-operator-compound") {
                env::set_var("CRUNCH_POOL_ONLY_OPERATOR_COMPOUND_ENABLED", "true");
            }

            if flakes_matches.is_present("enable-pool-members-compound") {
                env::set_var("CRUNCH_POOL_MEMBERS_COMPOUND_ENABLED", "true");
            }

            if flakes_matches.is_present("enable-pool-claim-commission") {
                env::set_var("CRUNCH_POOL_CLAIM_COMMISSION_ENABLED", "true");
            }

            if flakes_matches.is_present("enable-pool-active-nominees-payout") {
                env::set_var("CRUNCH_POOL_ACTIVE_NOMINEES_PAYOUT_ENABLED", "true");
            }

            if flakes_matches.is_present("enable-pool-all-nominees-payout") {
                env::set_var("CRUNCH_POOL_ALL_NOMINEES_PAYOUT_ENABLED", "true");
            }

            if flakes_matches.is_present("enable-onet-api") {
                env::set_var("CRUNCH_ONET_API_ENABLED", "true");
            }
        }
        ("view", Some(_)) => {
            env::set_var("CRUNCH_ONLY_VIEW", "true");
        }
        _ => {
            warn!("Besides subcommand 'flakes' being the default subcommand, would be cool to have it visible, so that CLI becomes more expressive (e.g. 'crunch flakes daily')");
        }
    }

    if matches.is_present("rewards") {
        env::set_var("CRUNCH_IS_BORING", "true");
    }

    match envy::prefixed("CRUNCH_").from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("Configuration error: {:#?}", error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gets_a_config() {
        let config = get_config();
        assert_ne!(config.substrate_ws_url, "".to_string());
    }

    #[test]
    fn it_gets_a_config_from_the_lazy_static() {
        let config = &CONFIG;
        assert_ne!(config.substrate_ws_url, "".to_string());
    }
}
