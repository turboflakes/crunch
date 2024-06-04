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

use codec;
use reqwest;
use std::{str::Utf8Error, string::String};
use subxt::{
    error::{DispatchError, MetadataError, RpcError},
    lightclient::LightClientError,
};

use thiserror::Error;

/// Crunch specific error messages
#[derive(Error, Debug)]
pub enum CrunchError {
    #[error("Subxt error: {0}")]
    SubxtError(#[from] subxt::Error),
    #[error("SubxtCore error: {0}")]
    SubxtCoreError(#[from] subxt::ext::subxt_core::Error),
    #[error("LightClient error: {0}")]
    LightClientError(#[from] LightClientError),
    #[error("Codec error: {0}")]
    CodecError(#[from] codec::Error),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Metadata error: {0}")]
    MetadataError(#[from] MetadataError),
    #[error("Dispatch error: {0}")]
    DispatchError(#[from] DispatchError),
    #[error("{0}")]
    RpcError(#[from] RpcError),
    #[error("Matrix error: {0}")]
    MatrixError(String),
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
    #[error("Other error: {0}")]
    Other(String),
}

/// Convert &str to CrunchError
impl From<&str> for CrunchError {
    fn from(error: &str) -> Self {
        CrunchError::Other(error.into())
    }
}

/// Crunch specific error messages
#[derive(Error, Debug)]
pub enum MatrixError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("ParseError error: {0}")]
    ParseError(#[from] url::ParseError),
    #[error("{0}")]
    Other(String),
}

/// Convert MatrixError to String
impl From<MatrixError> for String {
    fn from(error: MatrixError) -> Self {
        format!("{}", error).to_string()
    }
}

/// Convert MatrixError to CrunchError
impl From<MatrixError> for CrunchError {
    fn from(error: MatrixError) -> Self {
        CrunchError::MatrixError(error.into())
    }
}
