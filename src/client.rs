use std::error::Error;
use std::str::from_utf8;
use std::sync::{Arc, RwLock};
use futures::Stream;
use hyper::{Client, Method, Request, Response, StatusCode};
use hyper::client::HttpConnector;
use hyper::header::ContentType;
use hyper_tls::HttpsConnector;
use serde_json;
use tokio_core::reactor::Core;


use sesame::*;
use serialized::*;
use utility::*;

/// Sesame Client
pub struct SesameClient {
    body: Arc<RwLock<ClientBody>>,
}

/// Sesame client body
pub struct ClientBody {
    pub auth_token: Option<String>,
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub core: Core,
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

                    Ok(make_sesame(
                        self.body.clone(),
                        device_id,
                        body.nickname,
                        body.is_unlocked,
                        body.api_enabled,
                        body.battery
                    ))
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
                        make_sesame(
                            self.body.clone(),
                            elem.device_id,
                            elem.nickname,
                            elem.is_unlocked,
                            elem.api_enabled,
                            elem.battery,
                        )
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


impl ClientBody {
    pub fn request(&mut self, request: Request) -> Result<Response, String> {
        let resp = self.client.request(request);

        self.core.run(resp).or_else(|err| {
            Err(err.description().to_string())
        })
    }

    fn parse_result(&mut self, res: Response) -> (StatusCode, String) {
        let status = res.status();
        let bstr = self.core.run(res.body().concat2()).unwrap();
        let data = from_utf8(&bstr).unwrap();

        (status, data.to_string())
    }

    fn get_token_with_check(&self) -> Result<String, String> {
        if let Some(token) = self.auth_token.clone() {
            Ok(token)
        }
        else {
            Err("Not logged in".to_string())
        }
    }
}
