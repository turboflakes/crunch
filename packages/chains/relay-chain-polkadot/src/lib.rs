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

use crunch_core::Crunch;
use crunch_error::CrunchError;
use std::result::Result;

#[subxt::subxt(
    runtime_metadata_path = "metadata/polkadot_metadata_small.scale",
    derive_for_all_types = "Clone, PartialEq"
)]
mod rc_metadata {}

use rc_metadata::session::storage::types::validators::Validators;

/// Fetch the set of authorities (validators) at the latest block hash
pub async fn fetch_authorities(crunch: &Crunch) -> Result<Validators, CrunchError> {
    let api = crunch.client().clone();
    let addr = rc_metadata::storage().session().validators();

    api.storage()
        .at_latest()
        .await?
        .fetch(&addr)
        .await?
        .ok_or_else(|| {
            CrunchError::from(format!(
                "Current validators not defined at latest block hash"
            ))
        })
}
