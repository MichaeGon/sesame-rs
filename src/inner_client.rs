use std::fmt;
use std::error::Error;
use std::str::from_utf8;
use futures::{Stream};
use hyper::{Client, Request, StatusCode, Response};
use hyper::client::{HttpConnector};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

/// Sesame Client body
pub struct InnerClient {
    pub auth_token: Option<String>,
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub core: Core,
}

impl InnerClient {
    pub fn parse_result(&mut self, res: Response) -> (StatusCode, String) {
        let status = res.status();
        let bstr = self.core.run(res.body().concat2()).unwrap();
        let data = from_utf8(&bstr).unwrap();

        (status, data.to_string())
    }

    pub fn get_token_with_check(&self) -> Result<String, String> {
        if let Some(token) = self.auth_token.clone() {
            Ok(token)
        }
        else {
            Err("Not logged in".to_string())
        }
    }

    pub fn request(&mut self, request: Request) -> Result<Response, String> {
        let resp = self.client.request(request);

        self.core.run(resp).or_else(|err| {
            Err(err.description().to_string())
        })
    }
}

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
