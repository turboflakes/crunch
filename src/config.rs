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

/// provides default value for seed_file if CRUNCH_SEED_FILE env var is not set
fn default_seed_file() -> String {
  ".private.seed".into()
}

/// provides default value for maximum_payouts if CRUNCH_MAXIMUM_PAYOUTS env var is not set
fn default_maximum_payouts() -> usize {
  4
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
  #[serde(default = "default_interval")]
  pub interval: u64,
  pub substrate_ws_url: String,
  #[serde(default = "default_seed_file")]
  pub seed_file: String,
  pub stashes: Vec<String>,
  #[serde(default = "default_maximum_payouts")]
  pub maximum_payouts: usize,
  // matrix configuration
  pub matrix_username: String,
  pub matrix_password: String,
  pub matrix_server: String,
}

/// Inject dotenv and env vars into the Config struct
fn get_config() -> Config {
  // Define CLI flags with clap
  let matches = App::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .arg(
      Arg::with_name("NETWORK")
          .index(1)
          .possible_values(&["westend", "kusama", "polkadot"])
          .required(true)
          .help(
            "Choose the network to look for available flakes (staking rewards)",
          )
    )
    .subcommand(SubCommand::with_name("flakes")
      .about("Crunch delicious flakes daily or in turbo mode (4x faster) (Submit payout for unclaimed staking rewards daily or four times a day) (default)")
      .arg(
        Arg::with_name("MODE")
            .index(1)
            .possible_values(&["daily", "turbo"])
            .required(true)
            .help(
              "Choose how often flakes (staking rewards) should be crunched - once a day or four times a day",
            )
      )
      .arg(
        Arg::with_name("seed_file")
          .short("f")
          .long("seed_file")
          .takes_value(true)
          .help(
            "File path containing the private seed phrase to Sign the extrinsic payout call. (default is \".private.seed\")",
          ),
      )
      .arg(
        Arg::with_name("maximum_payouts")
          .short("m")
          .long("maximum_payouts")
          .takes_value(true)
          .help("Maximum number of unclaimed eras to which an extrinsic payout will be submitted. (default is 4)"))
      .arg(
        Arg::with_name("debug")
          .short("d")
          .long("debug")
          .help("Print debug information verbosely."))
    )
    .arg(
      Arg::with_name("stashes")
        .short("s")
        .long("stashes")
        .takes_value(true)
        .help(
          "Crunch flakes for the stash addresses - specify more than one like, e.g. stash_1,stash_2,stash_3.",
        ),
    )
    .arg(
      Arg::with_name("substrate_ws_url")
        .short("w")
        .long("substrate_ws_url")
        .takes_value(true)
        .help(
          "Substrate websocket endpoint to connect to, e.g. wss://kusama-rpc.polkadot.io",
        ),
    )
    .arg(
      Arg::with_name("config_file")
        .short("c")
        .long("config_file")
        .takes_value(true)
        .help(
          "File path containing Crunch environment configuration variables. (default is \".env\")",
        ),
    )
    .get_matches();

  // Try to load configuration from file first
  let config_file = matches.value_of("config_file").unwrap_or(".env");
  if let Some(_) = dotenv::from_filename(&config_file).ok() {
    info!("Loading configuration from {} file", &config_file);
  }

  match matches.value_of("NETWORK") {
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

  if let Some(seed_file) = matches.value_of("seed_file") {
    env::set_var("CRUNCH_SEED_FILE", seed_file);
  }

  if let Some(stashes) = matches.value_of("stashes") {
    env::set_var("CRUNCH_STASHES", stashes);
  }

  if let Some(substrate_ws_url) = matches.value_of("substrate_ws_url") {
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

      if let Some(maximum_payouts) = flakes_matches.value_of("maximum_payouts") {
        env::set_var("CRUNCH_MAXIMUM_PAYOUTS", maximum_payouts);
      }
    }
    _ => {
      warn!("Do not forget to specify subcommand \"flakes\" when running Crunch CLI.");
    },
  }

  if matches.is_present("debug") {
    env::set_var("RUST_LOG", "crunch=debug,substrate_subxt=debug");
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
