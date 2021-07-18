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
use log::info;
use serde::Deserialize;
use std::env;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
  pub interval: u64,
  pub substrate_ws_url: String,
  pub seed_phrase_filename: String,
  pub stashes: Vec<String>,
  pub max_unclaimed: u32,
}

// Set Config struct into a CONFIG lazy_static to avoid multiple processing
lazy_static! {
  pub static ref CONFIG: Config = get_config();
}

/// Inject dotenv and env vars into the Config struct
fn get_config() -> Config {
  let matches = App::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .arg(
      Arg::with_name("FLAVOUR")
          .index(1)
          .possible_values(&["westend", "kusama", "polkadot"])
          .help(
            "Choose your favorite flakes to crunch from",
          )
    )
    .subcommand(SubCommand::with_name("flakes")
      .about("Crunch delicious flakes daily or in turbo mode (4x faster) (Submit payout for unclaimed staking rewards daily or four times a day)")
      .arg(
        Arg::with_name("MODE")
            .index(1)
            .possible_values(&["daily", "turbo"])
            .required(true)
            .help(
              "Choose how often to crunch flakes from",
            )
      )
      .arg(
        Arg::with_name("seed_phrase_file")
          .short("p")
          .long("seed_phrase_file")
          .takes_value(true)
          .help(
            "Path to .private.seed file containing private seed phrase",
          ),
      )
    )
    .arg(
      Arg::with_name("stashes")
        .short("s")
        .long("stashes")
        .takes_value(true)
        .required(true)
        .help(
          "Source flakes from these stash addresses - specify more than one like, e.g. stash_1,stash_2,stash_3",
        ),
    )
    .arg(
      Arg::with_name("substrate_ws_url")
        .short("w")
        .long("substrate_ws_url")
        .takes_value(true)
        .help(
          "Substrate node websocket URL to connect to, e.g. wss://kusama-rpc.polkadot.io",
        ),
    )
    .arg(
      Arg::with_name("max")
        .short("m")
        .long("max")
        .help("Maximum number of unclaimed eras to submit staking rewards. [default = 1]"))
    .arg(
      Arg::with_name("debug")
        .short("d")
        .help("Print debug information verbosely"))
    .get_matches();

  match matches.value_of("FLAVOUR") {
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

  let seed_phrase_filename = matches
    .value_of("seed_phrase_filename")
    .unwrap_or(".private.seed");
  env::set_var("CRUNCH_SEED_PHRASE_FILENAME", seed_phrase_filename);

  if let Some(stashes) = matches.value_of("stashes") {
    env::set_var("CRUNCH_STASHES", stashes);
  }

  if let Some(substrate_ws_url) = matches.value_of("substrate_ws_url") {
    env::set_var("CRUNCH_SUBSTRATE_WS_URL", substrate_ws_url);
  }

  let max = matches.value_of("max").unwrap_or("1");
  env::set_var("CRUNCH_MAX_UNCLAIMED", max);

  if matches.is_present("debug") {
    env::set_var("RUST_LOG", "crunch=debug,substrate_subxt=debug");
  } else {
    env::set_var("RUST_LOG", "crunch=info");
  }

  match matches.subcommand() {
    ("flakes", Some(flakes_matches)) => match flakes_matches.value_of("MODE").unwrap() {
      "daily" => {
        env::set_var("CRUNCH_INTERVAL", "86400");
      }
      "turbo" => {
        env::set_var("CRUNCH_INTERVAL", "21600");
      }
      _ => unreachable!(),
    },
    _ => {
      // Default
      env::set_var("CRUNCH_INTERVAL", "21600");
    }
  }

  // let config_filename = env::var("CRUNCH_CONFIG_FILENAME").unwrap_or(".env".to_string());
  // dotenv::from_filename(&config_filename).ok();

  env_logger::try_init().unwrap_or_default();
  // info!("Loading configuration from {} file", &config_filename);

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
