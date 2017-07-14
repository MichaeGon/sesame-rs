extern crate futures;
#[macro_use]
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;

mod client;
mod serialized;
mod sesame;
mod utility;

use std::error::Error;
use std::sync::{Arc, RwLock};
use hyper::{Client, Method, Request, StatusCode};
use hyper::header::ContentType;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

use client::*;
use serialized::*;
use sesame::*;
use utility::*;

/// Sesame Client
pub struct SesameClient {
    body: Arc<RwLock<ClientBody>>,
}

/// Sesame device
pub struct Sesame {
    client: Arc<RwLock<ClientBody>>,
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
                ClientBody {
                    auth_token: None,
                    client: client,
                    core: core,
                }
            ))
        }
    }

    /// Log in to a candy house account
    pub fn login(&self, email: String, password: String) -> Result<(), String> {
        let json = format!(r#"{{"email":"{}", "password":"{}"}}"#, email, password);

        let uri = format!("{}{}", PREFIX, LOGIN_ENDPOINT).parse().unwrap();

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
            let uri = format!("{}{}/{}", PREFIX, SESAME_ENDPOINT, device_id).parse().unwrap();

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
            let uri = format!("{}{}", PREFIX, SESAME_ENDPOINT).parse().unwrap();

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
}
