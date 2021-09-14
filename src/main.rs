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

mod config;
mod crunch;
mod errors;
mod matrix;
mod stats;

use crate::config::CONFIG;
use crate::crunch::Crunch;
use log::info;
use std::env;

fn main() {
    let config = CONFIG.clone();
    if config.is_debug {
        env::set_var("RUST_LOG", "crunch=debug,substrate_subxt=debug");
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
        return Crunch::view();
    }
    if config.is_mode_era {
        return Crunch::subscribe();
    }
    Crunch::flakes()
}
