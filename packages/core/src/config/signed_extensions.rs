use codec::Encode;
use scale_info::PortableRegistry;
use subxt::{
    config::{
        ExtrinsicParams, ExtrinsicParamsEncoder, ExtrinsicParamsError,
        TransactionExtension,
    },
    utils::MultiSignature,
};

#[derive(Debug, Clone, Default)]
pub struct AuthorizeValueTransfer(pub Option<[u8; 64]>);

impl ExtrinsicParamsEncoder for AuthorizeValueTransfer {
    fn encode_value_to(&self, v: &mut Vec<u8>) {
        self.0.encode_to(v);
    }
}

impl<T: subxt::Config> ExtrinsicParams<T> for AuthorizeValueTransfer {
    type Params = ();

    fn new(
        _client: &subxt::client::ClientState<T>,
        _params: (),
    ) -> Result<Self, ExtrinsicParamsError> {
        Ok(Self(None))
    }
}

impl<T: subxt::Config> TransactionExtension<T> for AuthorizeValueTransfer {
    type Decoded = ();
    fn matches(identifier: &str, _type_id: u32, _types: &PortableRegistry) -> bool {
        identifier == "AuthorizeValueTransfer"
    }
}

#[derive(Debug, Clone, Encode)]
pub enum PgasCollection {
    People,
    LitePeople,
}

#[derive(Debug, Clone, Encode)]
pub enum AsPgasInfo {
    Claim {
        proof: Vec<u8>,
        ring_index: u32,
        revision: u32,
        collection: PgasCollection,
        day: u32,
    },
}

#[derive(Debug, Clone, Default)]
pub struct AsPgas(pub Option<AsPgasInfo>);

impl ExtrinsicParamsEncoder for AsPgas {
    fn encode_value_to(&self, v: &mut Vec<u8>) {
        self.0.encode_to(v); // None → 0x00
    }
}

impl<T: subxt::Config> ExtrinsicParams<T> for AsPgas {
    type Params = ();

    fn new(
        _client: &subxt::client::ClientState<T>,
        _params: (),
    ) -> Result<Self, ExtrinsicParamsError> {
        Ok(Self(None))
    }
}

impl<T: subxt::Config> TransactionExtension<T> for AsPgas {
    type Decoded = ();
    fn matches(identifier: &str, _type_id: u32, _types: &PortableRegistry) -> bool {
        identifier == "AsPgas"
    }
}

#[derive(Debug, Clone, Encode)]
pub enum AsRingAliasInfo {
    WithAccount(u32),
}

#[derive(Debug, Clone, Default)]
pub struct AsRingAlias(pub Option<AsRingAliasInfo>);

impl ExtrinsicParamsEncoder for AsRingAlias {
    fn encode_value_to(&self, v: &mut Vec<u8>) {
        self.0.encode_to(v); // None → 0x00
    }
}

impl<T: subxt::Config> ExtrinsicParams<T> for AsRingAlias {
    type Params = ();

    fn new(
        _client: &subxt::client::ClientState<T>,
        _params: (),
    ) -> Result<Self, ExtrinsicParamsError> {
        Ok(Self(None))
    }
}

impl<T: subxt::Config> TransactionExtension<T> for AsRingAlias {
    type Decoded = ();
    fn matches(identifier: &str, _type_id: u32, _types: &PortableRegistry) -> bool {
        identifier == "AsRingAlias"
    }
}

#[derive(Debug, Clone, Encode)]
pub enum AsDotnsGatewayInfo {
    RegisterFullName {
        proof: Vec<u8>,
        ring_index: u32,
        signature: MultiSignature,
    },
}

#[derive(Debug, Clone, Default)]
pub struct AsDotnsGateway(pub Option<AsDotnsGatewayInfo>);

impl ExtrinsicParamsEncoder for AsDotnsGateway {
    fn encode_value_to(&self, v: &mut Vec<u8>) {
        self.0.encode_to(v); // None → 0x00
    }
}

impl<T: subxt::Config> ExtrinsicParams<T> for AsDotnsGateway {
    type Params = ();

    fn new(
        _client: &subxt::client::ClientState<T>,
        _params: (),
    ) -> Result<Self, ExtrinsicParamsError> {
        Ok(Self(None))
    }
}

impl<T: subxt::Config> TransactionExtension<T> for AsDotnsGateway {
    type Decoded = ();
    fn matches(identifier: &str, _type_id: u32, _types: &PortableRegistry) -> bool {
        identifier == "AsDotnsGateway"
    }
}

#[derive(Debug, Clone, Default)]
pub struct RestrictOrigins(bool);

impl<T: subxt::Config> ExtrinsicParams<T> for RestrictOrigins {
    type Params = ();

    fn new(
        _client: &subxt::client::ClientState<T>,
        _params: Self::Params,
    ) -> Result<Self, ExtrinsicParamsError> {
        Ok(Self(true))
    }
}

impl ExtrinsicParamsEncoder for RestrictOrigins {
    fn encode_value_to(&self, v: &mut Vec<u8>) {
        self.0.encode_to(v);
    }
}

impl<T: subxt::Config> TransactionExtension<T> for RestrictOrigins {
    type Decoded = bool;
    fn matches(identifier: &str, _type_id: u32, _types: &PortableRegistry) -> bool {
        identifier == "RestrictOrigins"
    }
}

#[derive(Debug, Clone, Default)]
pub struct AuthorizeCall;

impl<T: subxt::Config> ExtrinsicParams<T> for AuthorizeCall {
    type Params = ();

    fn new(
        _client: &subxt::client::ClientState<T>,
        _params: (),
    ) -> Result<Self, ExtrinsicParamsError> {
        Ok(Self)
    }
}

impl ExtrinsicParamsEncoder for AuthorizeCall {}

impl<T: subxt::Config> TransactionExtension<T> for AuthorizeCall {
    type Decoded = ();
    fn matches(identifier: &str, _type_id: u32, _types: &PortableRegistry) -> bool {
        identifier == "AuthorizeCall"
    }
}
