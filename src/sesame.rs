use std::fmt;
use std::str::from_utf8;
use futures::Stream;
use hyper::{Method, Request, StatusCode};
use hyper::header::ContentType;
use serde_json;

use super::Sesame;
use serialized::*;
use utility::*;

/// Control Type
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ControlType {
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

impl Sesame {
    pub fn control(&mut self, ctype: ControlType) -> Result<(), String> {
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
