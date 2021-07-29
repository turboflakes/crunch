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
use std::string::String;
use thiserror::Error;

/// Crunch specific error messages
#[derive(Error, Debug)]
pub enum CrunchError {
    #[error("Substrate_subxt error: {0}")]
    SubxtError(#[from] substrate_subxt::Error),
    #[error("Codec error: {0}")]
    CodecError(#[from] codec::Error),
    #[error("Matrix error: {0}")]
    MatrixError(String),
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

/// Convert MatrixError to Sttring
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

