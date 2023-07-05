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
use hex::ToHex;
use std::str::FromStr;
use subxt::utils::AccountId32;

/// The type of account being created.
#[allow(dead_code)]
pub enum AccountType {
    Bonded,
    Reward,
}

impl AccountType {
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Bonded => vec![0u8],
            Self::Reward => vec![1u8],
        }
    }
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bonded => write!(f, "bonded"),
            Self::Reward => write!(f, "reward"),
        }
    }
}

pub fn nomination_pool_account(account_type: AccountType, pool_id: u32) -> AccountId32 {
    // NOTE: nomination pools pallet id could be retrieved
    // from metadata constants nomination_pools.pallet_id()
    let pallet_module = b"modlpy/nopls";
    // concatenate all information
    let mut buffer = Vec::<u8>::new();
    buffer.extend(pallet_module);
    buffer.extend(account_type.as_bytes());
    buffer.extend(pool_id.to_le_bytes());
    buffer.extend(vec![0u8; 15]);
    // convert to hex
    let buffer_hex = buffer.encode_hex::<String>();
    // return account
    return AccountId32::from_str(&buffer_hex).unwrap();
}

#[test]
fn test_pools() {
    assert_eq!(
        nomination_pool_account(AccountType::Reward, 2),
        AccountId32::from_str("13UVJyLnbVp8c4FQeiGUcWddfDNNLSajaPyfpYzx9QbrvLfR")
            .unwrap()
    );
    assert_eq!(
        nomination_pool_account(AccountType::Bonded, 2),
        AccountId32::from_str("13UVJyLnbVp8c4FQeiGCsV63YihAstUrqj3AGcK7gaj8eubS")
            .unwrap()
    );
}
