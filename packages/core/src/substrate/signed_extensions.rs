use codec::Encode;
use scale_info::PortableRegistry;
use subxt::{
    config::{ClientState, TransactionExtension},
    error::TransactionExtensionError,
    ext::frame_decode::extrinsics::{
        TransactionExtension as TransactionExtensionExt,
        TransactionExtensionError as TransactionExtensionErrorExt,
    },
};

#[derive(Debug, Clone, Default)]
pub struct RestrictOrigins(bool);

impl<T: subxt::Config> TransactionExtension<T> for RestrictOrigins {
    type Decoded = bool;
    type Params = ();

    fn new(
        _client: &ClientState<T>,
        _params: Self::Params,
    ) -> Result<Self, TransactionExtensionError> {
        Ok(Self(true))
    }
}

impl TransactionExtensionExt<PortableRegistry> for RestrictOrigins {
    const NAME: &str = "RestrictOrigins";

    fn encode_value_to(
        &self,
        _type_id: u32,
        _type_resolver: &PortableRegistry,
        out: &mut Vec<u8>,
    ) -> Result<(), TransactionExtensionErrorExt> {
        self.0.encode_to(out);
        Ok(())
    }

    fn encode_implicit_to(
        &self,
        _type_id: u32,
        _type_resolver: &PortableRegistry,
        _out: &mut Vec<u8>,
    ) -> Result<(), TransactionExtensionErrorExt> {
        Ok(())
    }
}
