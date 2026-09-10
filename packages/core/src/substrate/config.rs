use crate::substrate::extrinsic_params::CrunchExtrinsicParams;

use subxt::{
    config::{
        substrate::{DynamicHasher256, SubstrateHeader},
        Hasher,
    },
    utils::{AccountId32, MultiAddress, MultiSignature},
};

// Copy of the default [`subxt::config::CustomConfig`] customized with new transaction extensions.
#[derive(Debug, Clone, Default)]
pub struct CrunchConfig {}

impl subxt::Config for CrunchConfig {
    type AccountId = AccountId32;
    type Address = MultiAddress<Self::AccountId, u32>;
    type Signature = MultiSignature;
    type Hasher = DynamicHasher256;
    type Header = SubstrateHeader<<Self::Hasher as Hasher>::Hash>;
    type AssetId = u32;

    // Override only TransactionExtensions to add the new extensions
    type TransactionExtensions = CrunchExtrinsicParams<Self>;
}
