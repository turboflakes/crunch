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

use async_recursion::async_recursion;
use crunch_core::Crunch;
use crunch_error::CrunchError;
use crunch_report::{replace_emoji_lowercase, Validators};
use log::debug;
use std::result::Result;
use subxt::utils::AccountId32;

#[subxt::subxt(
    runtime_metadata_path = "metadata/people_paseo_metadata_small.scale",
    derive_for_all_types = "Clone, PartialEq"
)]
mod people_metadata {}

// Recursive function that looks up the identity of a validator given its stash,
// outputs a tuple with [primary identity/ sub-identity], primary identity and whether
// an identity is present.
#[async_recursion]
pub async fn get_display_name(
    crunch: &Crunch,
    stash: &AccountId32,
    sub_account_name: Option<String>,
) -> Result<(String, String, bool), CrunchError> {
    if let Some(api) = crunch.people_client().clone() {
        let identity_of_addr = people_metadata::storage()
            .identity()
            .identity_of(stash.clone());
        match api
            .storage()
            .at_latest()
            .await?
            .fetch(&identity_of_addr)
            .await?
        {
            Some(identity) => {
                debug!("identity {:?}", identity);
                let parent = parse_identity_data(identity.info.display);
                let name = match sub_account_name {
                    Some(child) => format!("{}/{}", &parent, child),
                    None => parent.clone(),
                };
                Ok((name, parent.clone(), true))
            }
            None => {
                let super_of_addr = people_metadata::storage()
                    .identity()
                    .super_of(stash.clone());
                if let Some((parent_account, data)) = api
                    .storage()
                    .at_latest()
                    .await?
                    .fetch(&super_of_addr)
                    .await?
                {
                    let sub_account_name = parse_identity_data(data);
                    return get_display_name(
                        &crunch,
                        &parent_account,
                        Some(sub_account_name.to_string()),
                    )
                    .await;
                } else {
                    let s = &stash.to_string();
                    let stash_address = format!("{}...{}", &s[..6], &s[s.len() - 6..]);
                    Ok((stash_address, "".to_string(), false))
                }
            }
        }
    } else {
        let s = &stash.to_string();
        let stash_address = format!("{}...{}", &s[..6], &s[s.len() - 6..]);
        Ok((stash_address, "".to_string(), false))
    }
}

//
fn parse_identity_data(
    data: people_metadata::runtime_types::pallet_identity::types::Data,
) -> String {
    match data {
        people_metadata::runtime_types::pallet_identity::types::Data::Raw0(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw1(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw2(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw3(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw4(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw5(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw6(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw7(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw8(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw9(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw10(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw11(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw12(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw13(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw14(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw15(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw16(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw17(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw18(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw19(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw20(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw21(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw22(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw23(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw24(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw25(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw26(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw27(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw28(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw29(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw30(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw31(bytes) => {
            str(bytes.to_vec())
        }
        people_metadata::runtime_types::pallet_identity::types::Data::Raw32(bytes) => {
            str(bytes.to_vec())
        }
        _ => format!("???"),
    }
}

fn str(bytes: Vec<u8>) -> String {
    format!("{}", String::from_utf8(bytes).expect("Identity not utf-8"))
}

// Provides a distinct and sorted vector of parent identities by string
// where there are entries without identities, these are placed to the end of the vector
pub fn get_distinct_parent_identites(validators: Validators) -> Vec<String> {
    // Obtains a sorted distinct list of valid identities
    let mut parent_identities: Vec<String> = validators
        .clone()
        .iter()
        .filter(|val| val.has_identity)
        .map(|val| replace_emoji_lowercase(&val.parent_identity))
        .collect();
    parent_identities.sort();
    parent_identities.dedup();

    // Note: If there are stashes/validators without identity they should be grouped in the bottom of the list
    let none_counter = validators
        .clone()
        .iter()
        .filter(|val| !val.has_identity)
        .count();

    if none_counter > 0 {
        parent_identities.push("".to_string());
    }

    parent_identities
}
