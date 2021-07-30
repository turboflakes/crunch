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

use crate::config::CONFIG;
use crate::errors::MatrixError;
use async_recursion::async_recursion;
use base64::encode;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::result::Result;
use std::{thread, time};
use url::form_urlencoded::byte_serialize;

const MATRIX_URL: &str = "https://matrix.org/_matrix/client/r0";

type AccessToken = String;
type RoomID = String;
type EventID = String;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Chain {
  Polkadot,
  Kusama,
  Westend,
  Other,
}

impl Chain {
  fn public_room_alias(&self) -> String {
    format!(
      "#{}-crunch-bot-test1:matrix.org",
      self.to_string().to_lowercase()
    )
  }
}

impl std::fmt::Display for Chain {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Polkadot => write!(f, "Polkadot"),
      Self::Kusama => write!(f, "Kusama"),
      Self::Westend => write!(f, "Westend"),
      Self::Other => write!(f, "Other"),
    }
  }
}

impl From<u8> for Chain {
  fn from(v: u8) -> Self {
    match v {
      0 => Chain::Polkadot,
      2 => Chain::Kusama,
      42 => Chain::Westend,
      _ => Chain::Other,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
  r#type: String,
  user: String,
  password: String,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
  user_id: String,
  access_token: AccessToken,
  home_server: String,
  device_id: String,
  // "well_known": {
  //   "m.homeserver": {
  //       "base_url": "https://matrix-client.matrix.org/"
  //   }
  // }
}

#[derive(Deserialize, Debug)]
struct GetRoomIdByRoomAliasResponse {
  room_id: RoomID,
  servers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateRoomRequest {
  name: String,
  room_alias_name: String,
  topic: String,
  preset: String,
  is_direct: bool,
}

#[derive(Deserialize, Debug)]
struct CreateRoomResponse {
  room_id: RoomID,
  room_alias: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendRoomMessageRequest {
  msgtype: String,
  body: String,
  format: String,
  formatted_body: String,
}

#[derive(Deserialize, Debug)]
struct SendRoomMessageResponse {
  event_id: EventID,
}

#[derive(Deserialize, Debug)]
struct ErrorResponse {
  errcode: String,
  error: String,
}

fn get_user_private_room_alias_name(chain_name: &str) -> String {
  let config = CONFIG.clone();
  let room = format!(
    "{}/{}/{}",
    env!("CARGO_PKG_NAME"),
    chain_name,
    config.matrix_username
  );
  encode(room.as_bytes())
}

pub struct Matrix {
  pub client: reqwest::Client,
  access_token: Option<String>,
  disabled: bool,
  chain: Chain,
}

impl Default for Matrix {
  fn default() -> Matrix {
    Matrix {
      client: reqwest::Client::new(),
      access_token: None,
      chain: Chain::Westend,
      disabled: false,
    }
  }
}

impl Matrix {
  pub async fn new(chain: Chain) -> Matrix {
    let config = CONFIG.clone();
    Matrix {
      chain: chain,
      disabled: config.matrix_disabled,
      ..Default::default()
    }
  }

  pub async fn login(&mut self) -> Result<(), MatrixError> {
    if self.disabled {
      return Ok(());
    }
    let client = self.client.clone();
    let config = CONFIG.clone();

    let req = LoginRequest {
      r#type: "m.login.password".into(),
      user: format!("@{}:{}", config.matrix_username, config.matrix_server),
      password: config.matrix_password.into(),
    };

    let res = client
      .post(format!("{}/login", MATRIX_URL))
      .json(&req)
      .send()
      .await?;

    debug!("response {:?}", res);
    match res.status() {
      reqwest::StatusCode::OK => {
        let response = res.json::<LoginResponse>().await?;
        self.access_token = Some(response.access_token);
        Ok(())
      }
      _ => {
        let response = res.json::<ErrorResponse>().await?;
        Err(MatrixError::Other(response.error))
      }
    }
  }
  pub async fn logout(&mut self) -> Result<(), MatrixError> {
    if self.disabled {
      return Ok(());
    }
    match &self.access_token {
      Some(access_token) => {
        let client = self.client.clone();
        let res = client
          .post(format!(
            "{}/logout?access_token={}",
            MATRIX_URL, access_token
          ))
          .send()
          .await?;
        debug!("response {:?}", res);
        match res.status() {
          reqwest::StatusCode::OK => {
            self.access_token = None;
            Ok(())
          }
          _ => {
            let response = res.json::<ErrorResponse>().await?;
            Err(MatrixError::Other(response.error))
          }
        }
      }
      None => Err(MatrixError::Other("access_token not defined".into())),
    }
  }

  async fn get_room_id_by_room_alias_name(
    &self,
    room_alias: &str,
  ) -> Result<Option<RoomID>, MatrixError> {
    let client = self.client.clone();
    let room_alias_encoded: String = byte_serialize(room_alias.as_bytes()).collect();
    let res = client
      .get(format!(
        "{}/directory/room/{}",
        MATRIX_URL, room_alias_encoded
      ))
      .send()
      .await?;
    debug!("response {:?}", res);
    match res.status() {
      reqwest::StatusCode::OK => {
        let response = res.json::<GetRoomIdByRoomAliasResponse>().await?;
        info!("{} * Matrix room alias", room_alias);
        Ok(Some(response.room_id))
      }
      reqwest::StatusCode::NOT_FOUND => Ok(None),
      _ => {
        let response = res.json::<ErrorResponse>().await?;
        Err(MatrixError::Other(response.error))
      }
    }
  }

  pub async fn create_private_room(
    &self,
    room_alias_name: &str,
  ) -> Result<Option<RoomID>, MatrixError> {
    match &self.access_token {
      Some(access_token) => {
        let client = self.client.clone();
        let req = CreateRoomRequest {
          name: format!("{} Crunch 🤖 (Private)", self.chain),
          room_alias_name: room_alias_name.into(),
          topic: "Crunch Bot <> Automate staking rewards (flakes) every X hours".into(),
          preset: "trusted_private_chat".into(),
          is_direct: true,
        };
        let res = client
          .post(format!(
            "{}/createRoom?access_token={}",
            MATRIX_URL, access_token
          ))
          .json(&req)
          .send()
          .await?;

        debug!("response {:?}", res);
        match res.status() {
          reqwest::StatusCode::OK => {
            let response = res.json::<CreateRoomResponse>().await?;
            info!(
              "{} * Matrix private room alias created",
              response.room_alias
            );
            Ok(Some(response.room_id))
          }
          _ => {
            let response = res.json::<ErrorResponse>().await?;
            Err(MatrixError::Other(response.error))
          }
        }
      }
      None => Err(MatrixError::Other("access_token not defined".into())),
    }
  }

  async fn get_user_private_room_id(&self) -> Result<Option<RoomID>, MatrixError> {
    let config = CONFIG.clone();
    match &self.access_token {
      Some(_) => {
        //  First verify if room_id already exists based on user_private_room_alias_name
        let room_alias_name = get_user_private_room_alias_name(&self.chain.to_string());
        let room_alias = format!("#{}:{}", room_alias_name, config.matrix_server);
        match self.get_room_id_by_room_alias_name(&room_alias).await? {
          Some(room_id) => Ok(Some(room_id)),
          None => Ok(self.create_private_room(&room_alias_name).await?),
        }
      }
      None => Err(MatrixError::Other("access_token not defined".into())),
    }
  }

  pub async fn send_message(
    &self,
    message: &str,
    formatted_message: &str,
  ) -> Result<(), MatrixError> {
    if self.disabled {
      return Ok(());
    }
    // Send message to user private room
    if let Some(private_room_id) = self.get_user_private_room_id().await? {
      self
        .dispatch_message(&private_room_id, &message, &formatted_message)
        .await?;
    }
    // Send message to public room
    let config = CONFIG.clone();
    if !config.matrix_public_room_disabled {
      if let Some(public_room_id) = self
        .get_room_id_by_room_alias_name(&self.chain.public_room_alias())
        .await?
      {
        self
          .dispatch_message(&public_room_id, &message, &formatted_message)
          .await?;
      }
    }

    Ok(())
  }

  #[async_recursion]
  async fn dispatch_message(
    &self,
    room_id: &str,
    message: &str,
    formatted_message: &str,
  ) -> Result<Option<EventID>, MatrixError> {
    if self.disabled {
      return Ok(None);
    }
    match &self.access_token {
      Some(access_token) => {
        let client = self.client.clone();
        let req = SendRoomMessageRequest {
          msgtype: "m.text".into(),
          body: message.to_string(),
          format: "org.matrix.custom.html".into(),
          formatted_body: formatted_message.to_string(),
        };

        let res = client
          .post(format!(
            "{}/rooms/{}/send/m.room.message?access_token={}",
            MATRIX_URL, room_id, access_token
          ))
          .json(&req)
          .send()
          .await?;

        debug!("response {:?}", res);
        match res.status() {
          reqwest::StatusCode::OK => {
            let response = res.json::<SendRoomMessageResponse>().await?;
            debug!("{:?} * Matrix messsage dispatched", response);
            Ok(Some(response.event_id))
          }
          reqwest::StatusCode::TOO_MANY_REQUESTS => {
            let response = res.json::<ErrorResponse>().await?;
            warn!("Matrix {} -> Wait 5 seconds and try again", response.error);
            thread::sleep(time::Duration::from_secs(5));
            return self
              .dispatch_message(room_id, message, formatted_message)
              .await;
          }
          _ => {
            let response = res.json::<ErrorResponse>().await?;
            Err(MatrixError::Other(response.error))
          }
        }
      }
      None => Err(MatrixError::Other("access_token not defined".into())),
    }
  }
}
