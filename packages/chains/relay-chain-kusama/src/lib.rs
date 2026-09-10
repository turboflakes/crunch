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
use subxt::utils::AccountId32;

#[subxt::subxt(
    runtime_metadata_path = "metadata/kusama_metadata_small.scale",
    derive_for_all_types = "Clone, PartialEq"
)]
mod rc_metadata {}

/// Fetch the set of authorities (validators) at the latest block hash
pub async fn fetch_authorities(crunch: &Crunch) -> Result<Vec<AccountId32>, CrunchError> {
    let api = crunch.client().clone();
    let addr = rc_metadata::storage().session().validators();

    let at = api.at_current_block().await?;
    let value = at.storage().entry(addr)?.fetch(()).await?.decode()?;

    Ok(value)
}
