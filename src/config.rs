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

/// provides default value for seed_path if CRUNCH_SEED_PATH env var is not set
fn default_seed_path() -> String {
  ".private.seed".into()
}

/// provides default value for maximum_payouts if CRUNCH_MAXIMUM_PAYOUTS env var is not set
fn default_maximum_payouts() -> usize {
  4
}

/// provides default value for only_view if CRUNCH_ONLY_VIEW env var is not set
fn default_false() -> bool {
  false
}

fn default_empty() -> String {
  String::from("")
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
  #[serde(default = "default_interval")]
  pub interval: u64,
  pub substrate_ws_url: String,
  #[serde(default = "default_seed_path")]
  pub seed_path: String,
  pub stashes: Vec<String>,
  #[serde(default = "default_maximum_payouts")]
  pub maximum_payouts: usize,
  #[serde(default = "default_false")]
  pub only_view: bool,
  // matrix configuration
  #[serde(default = "default_empty")]
  pub matrix_username: String,
  #[serde(default = "default_empty")]
  pub matrix_password: String,
  #[serde(default = "default_empty")]
  pub matrix_server: String,
  #[serde(default = "default_false")]
  pub matrix_disabled: bool,
  #[serde(default = "default_false")]
  pub matrix_public_room_disabled: bool,
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
          .possible_values(&["westend", "kusama", "polkadot"])
          .default_value("westend")
          .help(
            "Choose the substrate-based chain for which 'crunch' will try to connect",
          )
    )
    .subcommand(SubCommand::with_name("flakes")
      .about("Crunch delicious flakes daily or in turbo mode -> 4x faster (Claim staking rewards for unclaimed eras once a day or four times a day) [default subcommand]")
      .arg(
        Arg::with_name("MODE")
            .index(1)
            .possible_values(&["daily", "turbo"])
            .default_value("turbo")
            .help(
              "Choose how often flakes (staking rewards) should be crunched (claimed) from unclaimed eras. (e.g. with the option 'daily' means 'crunch' task will be repeated every 24 hours; option 'turbo' means 'crunch' task will be repeated every 6 hours)",
            )
      )
      .arg(
        Arg::with_name("seed-path")
          .short("f")
          .long("seed-path")
          .takes_value(true)
          .default_value(".private.seed")
          .help(
            "File path containing the private seed phrase to Sign the extrinsic payout call.",
          ),
      )
      .arg(
        Arg::with_name("maximum-payouts")
          .short("m")
          .long("maximum-payouts")
          .takes_value(true)
          .default_value("4")
          .help("Maximum number of unclaimed eras for which an extrinsic payout will be submitted. (e.g. a value of 4 means that if there are unclaimed eras in the last 84 the maximum unclaimed payout calls for each stash address will be 4)."))
      .arg(
        Arg::with_name("debug")
          .long("debug")
          .help("Prints debug information verbosely."))
      .arg(
        Arg::with_name("matrix-username")
          .long("matrix-username")
          .takes_value(true)
          .help("Username for matrix sign in."))
      .arg(
        Arg::with_name("matrix-password")
          .long("matrix-password")
          .takes_value(true)
          .help("Password for matrix sign in."))
      .arg(
        Arg::with_name("matrix-server")
          .long("matrix-server")
          .takes_value(true)
          .default_value("matrix.org")
          .help("Matrix server. (https://matrix.org/)"))
      .arg(
        Arg::with_name("disable-matrix")
          .long("disable-matrix")
          .help(
            "Disable matrix bot within 'crunch flakes'. (e.g. with this flag active 'crunch flakes' will not send messages/notifications about claimed or unclaimed staking rewards to your private or public 'Crunch Bot' rooms) (https://matrix.org/)",
          ),
      )
      .arg(
        Arg::with_name("disable-public-matrix-room")
          .long("disable-public-matrix-room")
          .help(
            "Disable notifications to matrix public rooms within 'crunch flakes'. (e.g. with this flag active 'crunch flakes' will not send messages/notifications about claimed or unclaimed staking rewards to any public 'Crunch Bot' room)",
          ),
      )
    )
    .subcommand(SubCommand::with_name("view")
      .about("Inspect for delicious flakes (staking rewards) to crunch (claim) and display claimed and unclaimed eras.")
    )
    .arg(
      Arg::with_name("stashes")
        .short("s")
        .long("stashes")
        .takes_value(true)
        .help(
          "Validator stash addresses for which 'crunch view' and 'crunch flakes' will be applied. If needed specify more than one (e.g. stash_1,stash_2,stash_3).",
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
        .default_value(".env")
        .help(
          "File path containing 'crunch' configuration variables.",
        ),
    )
    .get_matches();

  // Try to load configuration from file first
  let config_path = matches.value_of("config-path").unwrap_or(".env");
  match dotenv::from_filename(&config_path).ok() {
    Some(_) => info!("Loading configuration from {} file", &config_path),
    None => {
      let config_path = env::var("CRUNCH_CONFIG_FILENAME").unwrap_or(".env".to_string());
      if let Some(_) = dotenv::from_filename(&config_path).ok() {
        info!("Loading configuration from {} file", &config_path);
      }
    }
  }

  match matches.value_of("CHAIN") {
    Some("westend") => {
      env::set_var("CRUNCH_SUBSTRATE_WS_URL", "wss://westend-rpc.polkadot.io");
    }
    Some("kusama") => {
      env::set_var("CRUNCH_SUBSTRATE_WS_URL", "wss://kusama-rpc.polkadot.io");
    }
    Some("polkadot") => {
      env::set_var("CRUNCH_SUBSTRATE_WS_URL", "wss://rpc.polkadot.io");
    }
    _ => {
      env::set_var("CRUNCH_SUBSTRATE_WS_URL", "wss://westend-rpc.polkadot.io");
    }
  }

  if let Some(seed_path) = matches.value_of("seed-path") {
    env::set_var("CRUNCH_SEED_PATH", seed_path);
  }

  if let Some(stashes) = matches.value_of("stashes") {
    env::set_var("CRUNCH_STASHES", stashes);
  }

  if let Some(substrate_ws_url) = matches.value_of("substrate-ws-url") {
    env::set_var("CRUNCH_SUBSTRATE_WS_URL", substrate_ws_url);
  }

  match matches.subcommand() {
    ("flakes", Some(flakes_matches)) => {
      match flakes_matches.value_of("MODE").unwrap() {
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

      if flakes_matches.is_present("debug") {
        env::set_var("RUST_LOG", "crunch=debug,substrate_subxt=debug");
      }

      if matches.is_present("disable-matrix") {
        env::set_var("CRUNCH_MATRIX_DISABLED", "true");
      }

      if matches.is_present("disable-public-matrix-room") {
        env::set_var("CRUNCH_MATRIX_PUBLIC_ROOM_DISABLED", "true");
      }

      if let Some(matrix_username) = flakes_matches.value_of("matrix-username") {
        env::set_var("CRUNCH_MATRIX_USERNAME", matrix_username);
      }

      if let Some(matrix_password) = flakes_matches.value_of("matrix-password") {
        env::set_var("CRUNCH_MATRIX_PASSWORD", matrix_password);
      }

      if let Some(matrix_server) = flakes_matches.value_of("matrix-server") {
        env::set_var("CRUNCH_MATRIX_SERVER", matrix_server);
      }

    }
    ("view", Some(_)) => {
      env::set_var("CRUNCH_ONLY_VIEW", "true");
    }
    _ => {
      warn!("Besides subcommand 'flakes' being the default subcommand, would be cool to have it visible, so that CLI becomes more expressive (e.g. 'crunch flakes daily')");
    }
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
