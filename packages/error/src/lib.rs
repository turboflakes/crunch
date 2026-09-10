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

use std::{str::Utf8Error, string::String};
use subxt::{error::DispatchError, lightclient::LightClientError};

use thiserror::Error;

/// Crunch specific error messages
#[derive(Error, Debug)]
pub enum CrunchError {
    #[error("Subxt error: {0}")]
    SubxtError(Box<subxt::Error>),
    #[error("LightClient error: {0}")]
    LightClientError(#[from] LightClientError),
    #[error("Codec error: {0}")]
    CodecError(#[from] codec::Error),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Dispatch error: {0}")]
    DispatchError(#[from] DispatchError),
    #[error("RPC error: {0}")]
    RpcError(#[from] subxt::rpcs::Error),
    #[error("Matrix error: {0}")]
    MatrixError(#[from] crunch_matrix::error::MatrixError),
    #[error("Subscription finished")]
    SubscriptionFinished,
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("ParseError error: {0}")]
    ParseError(#[from] url::ParseError),
    #[error("SubxtSignerError error: {0}")]
    SubxtSignerError(#[from] subxt_signer::sr25519::Error),
    #[error("SecretError error: {0}")]
    SecretError(#[from] subxt_signer::SecretUriError),
    #[error("IOError error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Genesis mismatch: {0}")]
    GenesisError(String),
    #[error("Weight exceeded: {0}")]
    MaxWeightExceeded(String),
    #[error("Weight exceeded for one extrinsic")]
    MaxWeightExceededForOneExtrinsic,
    #[error("DryRunError: {0}")]
    DryRunError(String),
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),
    #[error("RuntimeUpgradeDetected: {0} -> {1}")]
    RuntimeUpgradeDetected(u32, u32),
    #[error("Other error: {0}")]
    Other(String),
}

/// Convert subxt::Error to CrunchError, boxing it to keep `CrunchError` small
impl From<subxt::Error> for CrunchError {
    fn from(error: subxt::Error) -> Self {
        CrunchError::SubxtError(Box::new(error))
    }
}

/// subxt 0.50 splits what used to be one `subxt::Error` into many granular,
/// per-operation error enums. Each of them still converts into `subxt::Error`
/// via `#[from]`, so route them all through the same `SubxtError` variant
/// instead of growing `CrunchError` by one variant per subxt error type.
macro_rules! impl_from_subxt_error {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for CrunchError {
                fn from(error: $ty) -> Self {
                    CrunchError::from(subxt::Error::from(error))
                }
            }
        )*
    };
}

impl_from_subxt_error!(
    subxt::error::StorageError,
    subxt::error::StorageValueError,
    subxt::error::StorageKeyError,
    subxt::error::OnlineClientAtBlockError,
    subxt::error::OnlineClientError,
    subxt::error::ConstantError,
    subxt::error::RuntimeApiError,
    subxt::error::ExtrinsicError,
    subxt::error::TransactionProgressError,
    subxt::error::TransactionEventsError,
    subxt::error::EventsError,
    subxt::error::AccountNonceError,
    subxt::error::BlocksError,
    subxt::error::BlockError,
    subxt::error::DispatchErrorDecodeError,
);

/// Convert &str to CrunchError
impl From<&str> for CrunchError {
    fn from(error: &str) -> Self {
        CrunchError::Other(error.into())
    }
}

/// Convert String to CrunchError
impl From<String> for CrunchError {
    fn from(error: String) -> Self {
        CrunchError::Other(error)
    }
}
