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

use crate::config::CONFIG;
use crate::errors::CrunchError;
// use crate::validator::{Validator, Validators};
use codec::Decode;
use log::{debug, info};
use std::{process::Command, result::Result, str::FromStr};
use subxt::{sp_runtime::AccountId32, Client, DefaultConfig, DefaultExtra, EventSubscription};

#[subxt::subxt(
    runtime_metadata_path = "metadata/kusama_metadata.scale",
    generated_type_derives = "Clone, Debug"
)]
mod kusama {}

pub type KusamaApi = kusama::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>;

pub async fn run_and_subscribe_new_session_events(
    client: &Client<DefaultConfig>,
) -> Result<(), CrunchError> {
    info!("Inspect and `crunch` unclaimed payout rewards");
    // Run once before start subscription
    // check_validator_status(client).await?;
    info!("Subscribe 'EraPaid' on-chain finalized event");
    let client = client.clone();
    let sub = client.rpc().subscribe_finalized_events().await?;
    let decoder = client.events_decoder();
    let mut sub = EventSubscription::<DefaultConfig>::new(sub, &decoder);
    sub.filter_event::<kusama::staking::events::EraPaid>();
    while let Some(result) = sub.next().await {
        if let Ok(raw) = result {
            match kusama::staking::events::EraPaid::decode(&mut &raw.data[..]) {
                Ok(event) => {
                    info!("Successfully decoded event {:?}", event);
                    // self.run_in_batch().await?;
                }
                Err(e) => return Err(CrunchError::CodecError(e)),
            }
        }
    }
    // If subscription has closed for some reason await and subscribe again
    Err(CrunchError::SubscriptionFinished)
}
