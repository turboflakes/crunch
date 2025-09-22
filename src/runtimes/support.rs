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

use crate::{
    config::CONFIG,
    runtimes::{kusama, paseo, polkadot, westend},
};
pub type ChainPrefix = u16;
pub type ChainTokenSymbol = String;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SupportedRuntime {
    Polkadot,
    Kusama,
    Westend,
    Paseo,
}

impl SupportedRuntime {
    pub fn asset_hub_runtime(&self) -> Option<SupportedParasRuntime> {
        match &self {
            Self::Polkadot => Some(SupportedParasRuntime::AssetHubPolkadot),
            Self::Kusama => Some(SupportedParasRuntime::AssetHubKusama),
            Self::Westend => Some(SupportedParasRuntime::AssetHubWestend),
            Self::Paseo => Some(SupportedParasRuntime::AssetHubPaseo),
        }
    }

    pub fn people_runtime(&self) -> Option<SupportedParasRuntime> {
        match &self {
            Self::Polkadot => Some(SupportedParasRuntime::PeoplePolkadot),
            Self::Kusama => Some(SupportedParasRuntime::PeopleKusama),
            Self::Westend => Some(SupportedParasRuntime::PeopleWestend),
            Self::Paseo => Some(SupportedParasRuntime::PeoplePaseo),
        }
    }

    pub fn chain_specs(&self) -> &str {
        match &self {
            Self::Polkadot => polkadot::POLKADOT_SPEC,
            Self::Kusama => kusama::KUSAMA_SPEC,
            Self::Westend => westend::WESTEND_SPEC,
            Self::Paseo => paseo::PASEO_SPEC,
        }
    }

    // NOTE: Hardcoded here to support staking on asset hub after asset hub migration finished
    pub fn is_staking_on_asset_hub(&self) -> bool {
        match &self {
            Self::Polkadot => false,
            Self::Kusama => false,
            Self::Westend => true,
            Self::Paseo => true,
        }
    }

    pub fn subdomain(&self) -> String {
        if self.is_staking_on_asset_hub() {
            return match &self {
                Self::Polkadot => "assethub-polkadot".to_string(),
                Self::Kusama => "assethub-kusama".to_string(),
                Self::Westend => "assethub-westend".to_string(),
                Self::Paseo => "assethub-paseo".to_string(),
            };
        }
        match &self {
            Self::Polkadot => "polkadot".to_string(),
            Self::Kusama => "kusama".to_string(),
            Self::Westend => "westend".to_string(),
            Self::Paseo => "paseo".to_string(),
        }
    }
}

impl From<ChainPrefix> for SupportedRuntime {
    fn from(v: ChainPrefix) -> Self {
        match v {
            0 => Self::Polkadot,
            2 => Self::Kusama,
            42 => Self::Westend,
            // TODO: Add Paseo for completeness
            _ => unimplemented!("Chain prefix not supported"),
        }
    }
}

impl From<&str> for SupportedRuntime {
    fn from(s: &str) -> Self {
        match s {
            "DOT" => Self::Polkadot,
            "polkadot" => Self::Polkadot,
            "KSM" => Self::Kusama,
            "kusama" => Self::Kusama,
            "WND" => Self::Westend,
            "westend" => Self::Westend,
            "PAS" => Self::Paseo,
            "paseo" => Self::Paseo,
            _ => unimplemented!("Chain not supported"),
        }
    }
}

impl From<String> for SupportedRuntime {
    fn from(v: String) -> Self {
        match v.as_str() {
            "DOT" => Self::Polkadot,
            "polkadot" => Self::Polkadot,
            "KSM" => Self::Kusama,
            "kusama" => Self::Kusama,
            "WND" => Self::Westend,
            "westend" => Self::Westend,
            "PAS" => Self::Paseo,
            "paseo" => Self::Paseo,
            _ => unimplemented!("Chain not supported"),
        }
    }
}

impl std::fmt::Display for SupportedRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Polkadot => write!(f, "Polkadot"),
            Self::Kusama => write!(f, "Kusama"),
            Self::Westend => write!(f, "Westend"),
            Self::Paseo => write!(f, "Paseo"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SupportedParasRuntime {
    PeoplePolkadot,
    PeopleKusama,
    PeopleWestend,
    PeoplePaseo,
    AssetHubPolkadot,
    AssetHubKusama,
    AssetHubPaseo,
    AssetHubWestend,
}

impl SupportedParasRuntime {
    pub fn default_rpc_url(&self) -> String {
        let config = CONFIG.clone();
        match &self {
            _ => config.substrate_people_ws_url,
        }
    }

    pub fn chain_specs(&self) -> &str {
        match &self {
            Self::PeoplePolkadot => polkadot::PEOPLE_POLKADOT_SPEC,
            Self::PeopleKusama => kusama::PEOPLE_KUSAMA_SPEC,
            Self::PeopleWestend => westend::PEOPLE_WESTEND_SPEC,
            Self::PeoplePaseo => paseo::PEOPLE_PASEO_SPEC,
            Self::AssetHubPolkadot => polkadot::ASSET_HUB_POLKADOT_SPEC,
            Self::AssetHubKusama => kusama::ASSET_HUB_KUSAMA_SPEC,
            Self::AssetHubWestend => westend::ASSET_HUB_WESTEND_SPEC,
            Self::AssetHubPaseo => paseo::ASSET_HUB_PASEO_SPEC,
        }
    }
}

impl std::fmt::Display for SupportedParasRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PeoplePolkadot => write!(f, "People Polkadot"),
            Self::PeopleKusama => write!(f, "People Kusama"),
            Self::PeopleWestend => write!(f, "People Westend"),
            Self::PeoplePaseo => write!(f, "People Paseo"),
            Self::AssetHubPolkadot => write!(f, "Asset Hub Polkadot"),
            Self::AssetHubKusama => write!(f, "Asset Hub Kusama"),
            Self::AssetHubWestend => write!(f, "Asset Hub Westend"),
            Self::AssetHubPaseo => write!(f, "Asset Hub Paseo"),
        }
    }
}
