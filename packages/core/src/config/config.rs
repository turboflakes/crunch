use crate::config::extrinsic_params::CrunchExtrinsicParams;
use subxt::{
    config::substrate::{DynamicHasher256, SubstrateHeader},
    utils::{AccountId32, MultiAddress, MultiSignature},
};

// Default set of commonly used types by Polkadot nodes.
// Note: The trait implementations exist just to make life easier,
// but shouldn't strictly be necessary since users can't instantiate this type.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum CrunchConfig {}

impl subxt::Config for CrunchConfig {
    type AccountId = AccountId32;
    type Address = MultiAddress<Self::AccountId, u32>;
    type Signature = MultiSignature;
    type Hasher = DynamicHasher256;
    type Header = SubstrateHeader<u32, DynamicHasher256>;
    type AssetId = u32;

    // Override only ExtrinsicParams to add the new extensions
    type ExtrinsicParams = CrunchExtrinsicParams<Self>;
}
