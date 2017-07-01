/// root module
/// sesame client and sesame devices
///

mod serialized;
mod inner_client;

#[macro_use]
extern crate hyper;
extern crate futures;
extern crate hyper_tls;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::sync::{Arc, RwLock};
use std::error::Error;
use std::str::from_utf8;
use futures::{Stream};
use hyper::{Client, Request, Method, StatusCode, Response};
use hyper::header::{ContentType};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

use serialized::*;
use inner_client::*;

const URI: &'static str = "https://api.candyhouse.co/v1";
const LOGIN_ENDPOINT: &'static str  = "/accounts/login";
const SESAME_ENDPOINT: &'static str = "/sesames";
const CONTROL_ENDPOINT: &'static str = "/control";

header! { (XAuth, "X-Authorization") => [String] }


/// Sesame client wrapper
pub struct SesameClient {
    body: Arc<RwLock<InnerClient>>,
}

/// Sesame device
pub struct Sesame {
    client: Arc<RwLock<InnerClient>>,
    device_id: String,
    nickname: String,
    is_unlocked: bool,
    api_enabled: bool,
    battery: u64,
}

impl SesameClient {
    pub fn new() -> Self {
        let core = Core::new().unwrap();
        let handle = core.handle();

        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);

        SesameClient {
            body: Arc::new(RwLock::new(
                InnerClient {
                    auth_token: None,
                    client: client,
                    core: core,
            }))
        }
    }

    /// Log in to a candy house account
    pub fn login(&self, email: String, password: String) -> Result<(), String> {
        let json = format!(r#"{{"email":"{}", "password":"{}"}}"#, email, password);

        let uri = format!("{}{}", URI, LOGIN_ENDPOINT).parse().unwrap();

        let mut request = Request::new(Method::Post, uri);
        request.headers_mut().set(ContentType::json());
        request.set_body(json);

        self.request(request).and_then(|res| {
            let (status, data) = self.parse_result(res);

            if status == StatusCode::Ok {
                let mut client = self.body.write().unwrap();

                let token: AuthToken = serde_json::from_str(&data).unwrap();
                client.auth_token = Some(token.authorization);

                Ok(())
            }
            else {
                let message: Result<Message, _> = serde_json::from_str(&data);
                match message {
                    Ok(msg) => Err(msg.message),
                    Err(err) => Err(err.description().to_string())
                }
            }
        })
    }

    /// get a sesame device by device_id
    pub fn get_sesame(&self, device_id: String) -> Result<Sesame, String> {

        self.get_token_with_check().and_then(|token| {
            let uri = format!("{}{}/{}", URI, SESAME_ENDPOINT, device_id).parse().unwrap();

            let mut request = Request::new(Method::Get, uri);
            request.headers_mut().set(XAuth(token.to_owned()));

            self.request(request).and_then(|res| {
                let (status, data) = self.parse_result(res);

                if status == StatusCode::Ok {
                    let body: SesameBody = serde_json::from_str(&data).unwrap();

                    Ok(Sesame {
                        client: self.body.clone(),
                        device_id: device_id,
                        nickname: body.nickname,
                        is_unlocked: body.is_unlocked,
                        api_enabled: body.api_enabled,
                        battery: body.battery,
                    })
                }
                else {
                    let msg: Message = serde_json::from_str(&data).unwrap();
                    Err(msg.message)
                }
            })
        })
    }

    /// get sesame devices
    pub fn list_sesames(&self) -> Result<Vec<Sesame>, String> {
        self.get_token_with_check().and_then(|token| {
            let uri = format!("{}{}", URI, SESAME_ENDPOINT).parse().unwrap();

            let mut request = Request::new(Method::Get, uri);
            request.headers_mut().set(XAuth(token.to_owned()));

            self.request(request).and_then(|res| {
                let (status, data) = self.parse_result(res);

                if status == StatusCode::Ok {
                    let sesames: SesameList = serde_json::from_str(&data).unwrap();

                    Ok(sesames.sesames.into_iter().map(|elem| {
                        Sesame {
                            client: self.body.clone(),
                            device_id: elem.device_id,
                            nickname: elem.nickname,
                            is_unlocked: elem.is_unlocked,
                            api_enabled: elem.api_enabled,
                            battery: elem.battery,
                        }
                    }).collect())
                }
                else {
                    let msg: Message = serde_json::from_str(&data).unwrap();
                    Err(msg.message)
                }
            })
        })
    }



    fn parse_result(&self, res: Response) -> (StatusCode, String) {
        let mut client = self.body.write().unwrap();

        client.parse_result(res)
    }

    fn get_token_with_check(&self) -> Result<String, String> {
        let client = self.body.read().unwrap();

        client.get_token_with_check()
    }

    fn request(&self, request: Request) -> Result<Response, String> {
        let mut client = self.body.write().unwrap();

        client.request(request)
    }
}

impl Sesame {
    /// getters
    pub fn get_device_id(&self) -> String {
        self.device_id.clone()
    }

    pub fn get_nickname(&self) -> String {
        self.nickname.clone()
    }

    pub fn is_unlocked(&self) -> bool {
        self.is_unlocked
    }

    pub fn api_enabled(&self) -> bool {
        self.api_enabled
    }

    pub fn get_battery(&self) -> u64 {
        self.battery
    }


    /// lock this sesame
    pub fn lock(&mut self) -> Result<(), String> {
        self.control("lock".to_string())
    }

    /// unlock this sesame
    pub fn unlock(&mut self) -> Result<(), String> {
        self.control("unlock".to_string())
    }

    fn control(&mut self, ctype: String) ->Result<(), String> {
        let atoken = {
            let client = self.client.read().unwrap();
            client.auth_token.clone()
        };

        let flag = ctype == "unlock";

        if self.is_unlocked == flag {
            if let Some(token) = atoken {
                let mut client = self.client.write().unwrap();

                let json = format!(r#"{{"type":"{}"}}"#, ctype);

                let uri = format!("{}{}/{}{}", URI, SESAME_ENDPOINT, self.device_id, CONTROL_ENDPOINT).parse().unwrap();

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


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
