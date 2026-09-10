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
//
use async_std::task;
use crunch_config::{RunMode, CONFIG};
use crunch_core::Crunch;
use crunch_error::CrunchError;
use crunch_support::SupportedRuntime;
use log::{error, info, warn};
use std::{env, result::Result, thread, time};

fn main() {
    let config = CONFIG.clone();
    if config.is_debug {
        env::set_var("RUST_LOG", "crunch=debug,subxt=debug");
    } else {
        env::set_var("RUST_LOG", "crunch=info");
    }
    env_logger::try_init().unwrap_or_default();

    info!(
        "{} v{} * {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION")
    );

    if config.only_view {
        return spawn_crunch_view();
    }

    match config.run_mode {
        RunMode::Once => spawn_crunch_once(),
        RunMode::Daily | RunMode::Turbo => spawn_and_restart_crunch_flakes_on_error(),
        RunMode::Era => spawn_and_restart_subscription_on_error(),
    }
}

fn spawn_crunch_view() {
    let crunch_task = task::spawn(async {
        let crunch: Crunch = Crunch::new().await;
        if let Err(e) = inspect(&crunch).await {
            error!("{}", e);
        };
    });
    task::block_on(crunch_task);
}

fn spawn_crunch_once() {
    let crunch_task = task::spawn(async {
        let crunch: Crunch = Crunch::new().await;
        if let Err(e) = try_run_batch(&crunch).await {
            error!("{}", e);
        };
    });
    task::block_on(crunch_task);
}

fn spawn_and_restart_crunch_flakes_on_error() {
    let t = task::spawn(async {
        let config = CONFIG.clone();
        let mut n = 1_u32;
        loop {
            let crunch: Crunch = Crunch::new().await;
            if let Err(e) = try_run_batch(&crunch).await {
                let sleep_min = u32::pow(config.error_interval, n);
                match e {
                    CrunchError::MatrixError(_) => warn!("Matrix message skipped!"),
                    CrunchError::DryRunError(err) => {
                        let sleep_min = u32::pow(config.error_interval, n);
                        warn!(
                            "DryRunError: {}, on hold for {} secs",
                            err,
                            60 * sleep_min
                        );
                        thread::sleep(time::Duration::from_secs((60 * sleep_min).into()));
                    }
                    _ => {
                        error!("{}", e);
                        let message = format!("On hold for {} min!", sleep_min);
                        let formatted_message = format!("<br/>🚨 An error was raised -> <code>crunch</code> on hold for {} min while rescue is on the way 🚁 🚒 🚑 🚓<br/><br/>", sleep_min);
                        if let Err(matrix_err) =
                            crunch.send_message(&message, &formatted_message).await
                        {
                            warn!(
                                "Failed to send error notification message: {}",
                                matrix_err
                            );
                        }
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

fn spawn_and_restart_subscription_on_error() {
    let t = task::spawn(async {
        let config = CONFIG.clone();
        let mut n = 1_u32;
        loop {
            let crunch: Crunch = Crunch::new().await;
            if let Err(e) = run_and_subscribe_era_paid_events(&crunch).await {
                match e {
                    CrunchError::SubscriptionFinished
                    | CrunchError::MatrixError(_)
                    | CrunchError::RpcError(_)
                    | CrunchError::RuntimeUpgradeDetected(_, _) => {
                        warn!("{} - On hold for 30 secs!", e);
                        thread::sleep(time::Duration::from_secs(30));
                    }
                    CrunchError::DryRunError(_) => {
                        let sleep_min = u32::pow(config.error_interval, n);
                        warn!("{} - On hold for {} secs!", e, 60 * sleep_min);
                        thread::sleep(time::Duration::from_secs((60 * sleep_min).into()));
                    }
                    CrunchError::SubxtError(ref subxt_err)
                        if subxt_err.to_string().contains("connection was lost") =>
                    {
                        warn!("{} - On hold for 30 secs!", subxt_err);
                        thread::sleep(time::Duration::from_secs(30));
                    }
                    _ => {
                        error!("{}", e);
                        let mut sleep_min = u32::pow(config.error_interval, n);
                        if sleep_min > config.maximum_error_interval {
                            sleep_min = config.maximum_error_interval;
                        }
                        let message = format!("On hold for {} min!", sleep_min);
                        let formatted_message = format!("<br/>🚨 An error was raised -> <code>crunch</code> on hold for {} min while rescue is on the way 🚁 🚒 🚑 🚓<br/><br/>", sleep_min);
                        if let Err(matrix_err) =
                            crunch.send_message(&message, &formatted_message).await
                        {
                            warn!(
                                "Failed to send error notification message: {}",
                                matrix_err
                            );
                        }
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

async fn inspect(crunch: &Crunch) -> Result<(), CrunchError> {
    crunch.validate_genesis().await?;
    match crunch.runtime() {
        SupportedRuntime::Polkadot => crunch_polkadot::inspect(crunch).await,
        SupportedRuntime::Kusama => crunch_kusama::inspect(crunch).await,
        SupportedRuntime::Paseo => crunch_paseo::inspect(crunch).await,
        SupportedRuntime::Westend => crunch_westend::inspect(crunch).await,
        // _ => panic!("Unsupported runtime"),
    }
}

async fn try_run_batch(crunch: &Crunch) -> Result<(), CrunchError> {
    crunch.validate_genesis().await?;
    match crunch.runtime() {
        SupportedRuntime::Polkadot => crunch_polkadot::try_crunch(crunch).await,
        SupportedRuntime::Kusama => crunch_kusama::try_crunch(crunch).await,
        SupportedRuntime::Paseo => crunch_paseo::try_crunch(crunch).await,
        SupportedRuntime::Westend => crunch_westend::try_crunch(crunch).await,
        // _ => panic!("Unsupported runtime"),
    }
}

async fn run_and_subscribe_era_paid_events(crunch: &Crunch) -> Result<(), CrunchError> {
    crunch.validate_genesis().await?;
    match crunch.runtime() {
        SupportedRuntime::Polkadot => {
            crunch_polkadot::run_and_subscribe_era_paid_events(crunch).await
        }
        SupportedRuntime::Kusama => {
            crunch_kusama::run_and_subscribe_era_paid_events(crunch).await
        }
        SupportedRuntime::Paseo => {
            crunch_paseo::run_and_subscribe_era_paid_events(crunch).await
        }
        SupportedRuntime::Westend => {
            crunch_westend::run_and_subscribe_era_paid_events(crunch).await
        } // _ => panic!("Unsupported runtime"),
    }
}
