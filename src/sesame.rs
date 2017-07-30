use std::fmt;
use std::str::from_utf8;
use std::sync::{Arc, RwLock};
use futures::Stream;
use hyper::{Method, Request, StatusCode};
use hyper::header::ContentType;
use serde_json;

use client::*;
use serialized::*;
use utility::*;

/// Sesame device
pub struct Sesame {
    client: Arc<RwLock<ClientBody>>,
    device_id: String,
    nickname: String,
    is_unlocked: bool,
    api_enabled: bool,
    battery: u64,
}

/// Control Type
#[derive(Copy, Clone, Eq, PartialEq)]
enum ControlType {
    Lock,
    Unlock,
}

impl fmt::Display for ControlType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ControlType::Lock => write!(f, "lock"),
            &ControlType::Unlock => write!(f, "unlock")
        }
    }
}

pub fn make_sesame(client: Arc<RwLock<ClientBody>>, device_id: String, nickname: String, is_unlocked: bool, api_enabled: bool, battery: u64) -> Sesame {
    Sesame {
        client: client,
        device_id: device_id,
        nickname: nickname,
        is_unlocked: is_unlocked,
        api_enabled: api_enabled,
        battery: battery,
    }
}

impl Sesame {
    /// get device_id
    pub fn device_id(&self) -> String {
        self.device_id.clone()
    }

    /// get nickname
    pub fn nickname(&self) -> String {
        self.nickname.clone()
    }


    pub fn is_unlocked(&self) -> bool {
        self.is_unlocked
    }

    pub fn api_enabled(&self) -> bool {
        self.api_enabled
    }

    pub fn battery(&self) -> u64 {
        self.battery
    }


    /// lock this sesame
    pub fn lock(&mut self) -> Result<(), String> {
        self.control(ControlType::Lock)
    }

    /// unlock this sesame
    pub fn unlock(&mut self) -> Result<(), String> {
        self.control(ControlType::Unlock)
    }

    fn control(&mut self, ctype: ControlType) -> Result<(), String> {
        let atoken = {
            let client = self.client.read().unwrap();
            client.auth_token.clone()
        };

        let flag = ctype != ControlType::Unlock;

        if self.is_unlocked == flag {
            if let Some(token) = atoken {
                let mut client = self.client.write().unwrap();

                let json = format!(r#"{{"type":"{}"}}"#, ctype);

                let uri = format!("{}{}/{}{}", PREFIX, SESAME_ENDPOINT, self.device_id, CONTROL_ENDPOINT).parse().unwrap();

                let mut request = Request::new(Method::Post, uri);
                request.headers_mut().set(XAuth(token));
                request.headers_mut().set(ContentType::json());
                request.set_body(json);

                let res = client.request(request).and_then(|res| {
                    let status = res.status();

                    if status == StatusCode::NoContent {
                        Ok(())
                    }
                    else {
                        let bstr = client.core.run(res.body().concat2()).unwrap();
                        let data = from_utf8(&bstr).unwrap();

                        let msg: Message = serde_json::from_str(data).unwrap();
                        Err(msg.message)
                    }
                });

                if res.is_ok() {
                    self.is_unlocked = !flag;
                }

                res
            }
            else {
                Err("Not logged in".to_string())
            }

        }
        else {
            Err(format!("Sesame{{id: {}, nickname: {}}}: already {}ed", self.device_id, self.nickname, ctype))
        }
    }
}
