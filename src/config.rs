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
fn default_error_interval() -> u64 {
    5
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
    4
}

/// provides default value for existential_deposit_factor_warning if CRUNCH_EXISTENTIAL_DEPOSIT_FACTOR_WARNING env var is not set
/// polkadot 2x
/// kusama 1000x
fn default_existential_deposit_factor_warning() -> u32 {
    2
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_interval")]
    pub interval: u64,
    #[serde(default = "default_error_interval")]
    pub error_interval: u64,
    pub substrate_ws_url: String,
    #[serde(default)]
    pub stashes_url: String,
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
    #[serde(default)]
    pub only_view: bool,
    #[serde(default)]
    pub is_debug: bool,
    #[serde(default)]
    pub is_boring: bool,
    #[serde(default)]
    pub is_short: bool,
    #[serde(default)]
    pub is_mode_era: bool,
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
          .possible_values(&["westend", "kusama", "polkadot", "azero", "tzero"])
          .help(
            "Sets the substrate-based chain for which 'crunch' will try to connect",
          )
    )
    .subcommand(SubCommand::with_name("flakes")
      .about("Crunch awesome flakes (rewards) every era, daily or in turbo mode -> 4x faster")
      .arg(
        Arg::with_name("MODE")
            .index(1)
            .possible_values(&["era", "daily", "turbo"])
            .default_value("era")
            .help(
              "Sets how often flakes (staking rewards) should be crunched (claimed) from unclaimed eras. (e.g. the option 'era' sets 'crunch' task to run as soon as the EraPaid on-chain event is triggered; the option 'daily' sets 'crunch' task to be repeated every 24 hours; option 'turbo' sets 'crunch' task to be repeated every 6 hours)",
            )
      )
      .arg(
        Arg::with_name("seed-path")
          .short("f")
          .long("seed-path")
          .takes_value(true)
          .value_name("FILE")
          .help(
            "Sets a custom seed file path. The seed file contains the private seed phrase to Sign the extrinsic payout call.",
          ),
      )
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
            "Disable matrix bot for 'crunch flakes'. (e.g. with this flag active 'crunch flakes' will not send messages/notifications about claimed or unclaimed staking rewards to your private or public 'Crunch Bot' rooms) (https://matrix.org/)",
          ),
      )
      .arg(
        Arg::with_name("disable-public-matrix-room")
          .long("disable-public-matrix-room")
          .help(
            "Disable notifications to matrix public rooms for 'crunch flakes'. (e.g. with this flag active 'crunch flakes' will not send messages/notifications about claimed or unclaimed staking rewards to any public 'Crunch Bot' room)",
          ),
      )
      .arg(
        Arg::with_name("disable-matrix-bot-display-name")
          .long("disable-matrix-bot-display-name")
          .help(
            "Disable matrix bot display name update for 'crunch flakes'. (e.g. with this flag active 'crunch flakes' will not change the matrix bot user display name)",
          ),
        )
      .arg(
        Arg::with_name("short")
          .long("short")
          .help("Display only essential information (e.g. with this flag active 'crunch flakes' will only send essential messages/notifications about claimed rewards)"))
      .arg(
        Arg::with_name("error-interval")
          .long("error-interval")
          .takes_value(true)
          .help("Interval value (in minutes) from which 'crunch' will restart again in case of a critical error."))
    )
    .subcommand(SubCommand::with_name("rewards")
      .about("Claim staking rewards for unclaimed eras once a day or four times a day [default subcommand]")
      .arg(
        Arg::with_name("MODE")
            .index(1)
            .possible_values(&["era", "daily", "turbo"])
            .default_value("era")
            .help(
              "Sets how often staking rewards should be claimed from unclaimed eras. (e.g. the option 'era' sets 'crunch' task to run as soon as the EraPaid on-chain event is triggered; the option 'daily' sets 'crunch' task to be repeated every 24 hours; option 'turbo' sets 'crunch' task to be repeated every 6 hours)",
            )
      )
      .arg(
        Arg::with_name("seed-path")
          .short("f")
          .long("seed-path")
          .takes_value(true)
          .value_name("FILE")
          .help(
            "Sets a custom seed file path. The seed file contains the private seed phrase to Sign the extrinsic payout call.",
          ),
      )
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
          ),
      )
      .arg(
        Arg::with_name("disable-public-matrix-room")
          .long("disable-public-matrix-room")
          .help(
            "Disable notifications to matrix public rooms for 'crunch rewards'. (e.g. with this flag active 'crunch rewards' will not send messages/notifications about claimed or unclaimed staking rewards to any public 'Crunch Bot' room)",
          ),
      )
      .arg(
        Arg::with_name("disable-matrix-bot-display-name")
          .long("disable-matrix-bot-display-name")
          .help(
            "Disable matrix bot display name update for 'crunch rewards'. (e.g. with this flag active 'crunch rewards' will not change the matrix bot user display name)",
          ),
        )
      .arg(
        Arg::with_name("short")
          .long("short")
          .help("Display only essential information (e.g. with this flag active 'crunch rewards' will only send essential messages/notifications about claimed rewards)"))
      .arg(
        Arg::with_name("error-interval")
          .long("error-interval")
          .takes_value(true)
          .help("Interval value (in minutes) from which 'crunch' will restart again in case of a critical error."))
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
        ),
    )
    .arg(
      Arg::with_name("stashes-url")
        .long("stashes-url")
        .takes_value(true)
        .help(
          "Remote stashes endpoint for which 'crunch' will try to fetch the validator stash addresses (e.g. https://raw.githubusercontent.com/turboflakes/crunch/main/.remote.stashes.example).",
        ),
    )
    .arg(
      Arg::with_name("substrate-ws-url")
        .short("w")
        .long("substrate-ws-url")
        .takes_value(true)
        .help(
          "Substrate websocket endpoint for which 'crunch' will try to connect. (e.g. wss://kusama-rpc.polkadot.io) (NOTE: substrate_ws_url takes precedence than <CHAIN> argument)",
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
        ),
    )
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
                "wss://westend-rpc.polkadot.io:443",
            );
        }
        Some("kusama") => {
            env::set_var(
                "CRUNCH_SUBSTRATE_WS_URL",
                "wss://kusama-rpc.polkadot.io:443",
            );
        }
        Some("polkadot") => {
            env::set_var("CRUNCH_SUBSTRATE_WS_URL", "wss://rpc.polkadot.io:443");
        }
        Some("azero") => {
            env::set_var("CRUNCH_SUBSTRATE_WS_URL", "wss://ws.azero.dev:443");
        }
        Some("tzero") => {
            env::set_var("CRUNCH_SUBSTRATE_WS_URL", "wss://ws.test.azero.dev:443");
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

    if let Some(seed_path) = matches.value_of("seed-path") {
        env::set_var("CRUNCH_SEED_PATH", seed_path);
    }

    if let Some(stashes_url) = matches.value_of("stashes-url") {
        env::set_var("CRUNCH_STASHES_URL", stashes_url);
    }

    if let Some(stashes) = matches.value_of("stashes") {
        env::set_var("CRUNCH_STASHES", stashes);
    }

    match matches.subcommand() {
        ("flakes", Some(flakes_matches)) | ("rewards", Some(flakes_matches)) => {
            match flakes_matches.value_of("MODE").unwrap() {
                "era" => {
                    env::set_var("CRUNCH_IS_MODE_ERA", "true");
                }
                "daily" => {
                    env::set_var("CRUNCH_INTERVAL", "86400");
                }
                "turbo" => {
                    env::set_var("CRUNCH_INTERVAL", "21600");
                }
                _ => unreachable!(),
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
