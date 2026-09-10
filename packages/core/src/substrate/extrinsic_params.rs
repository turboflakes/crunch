use subxt::config::{
    transaction_extensions, DefaultExtrinsicParamsBuilder, TransactionExtensions,
};

use crate::substrate::signed_extensions::RestrictOrigins;

pub type CrunchExtrinsicParams<T> = (
    transaction_extensions::VerifySignature<T>,
    transaction_extensions::CheckSpecVersion,
    transaction_extensions::CheckTxVersion,
    transaction_extensions::CheckNonce,
    transaction_extensions::CheckGenesis<T>,
    transaction_extensions::CheckMortality<T>,
    transaction_extensions::ChargeAssetTxPayment<T>,
    transaction_extensions::ChargeTransactionPayment,
    transaction_extensions::CheckMetadataHash,
    RestrictOrigins,
);

// Wraps subxt's [`DefaultExtrinsicParamsBuilder`], extended with the parameters.
#[derive(Default)]
pub struct CrunchExtrinsicParamsBuilder<T: subxt::Config>(
    DefaultExtrinsicParamsBuilder<T>,
);

impl<T: subxt::Config> CrunchExtrinsicParamsBuilder<T> {
    pub fn new() -> Self {
        Self(DefaultExtrinsicParamsBuilder::new())
    }

    pub fn nonce(self, nonce: u64) -> Self {
        Self(self.0.nonce(nonce))
    }

    pub fn mortal(self, for_n_blocks: u64) -> Self {
        Self(self.0.mortal(for_n_blocks))
    }

    pub fn tip(self, tip: u128) -> Self {
        Self(self.0.tip(tip))
    }

    pub fn build(self) -> <CrunchExtrinsicParams<T> as TransactionExtensions<T>>::Params {
        let default = self.0.build();
        (
            default.0,
            default.1,
            default.2,
            default.3,
            default.4,
            default.5,
            default.6,
            default.7,
            default.8,
            // Additional extensions take no parameters.
            (),
        )
    }
}
