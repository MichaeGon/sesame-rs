use std::error::Error;
use std::str::from_utf8;
use futures::Stream;
use hyper::{Client, Request, Response, StatusCode};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

use super::SesameClient;

/// Sesame client body
pub struct ClientBody {
    pub auth_token: Option<String>,
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub core: Core,
}

impl ClientBody {
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

impl SesameClient {
    pub fn parse_result(&self, res: Response) -> (StatusCode, String) {
        let mut client = self.body.write().unwrap();

        client.parse_result(res)
    }

    pub fn get_token_with_check(&self) -> Result<String, String> {
        let client = self.body.read().unwrap();

        client.get_token_with_check()
    }

    pub fn request(&self, request: Request) -> Result<Response, String> {
        let mut client = self.body.write().unwrap();

        client.request(request)
    }
}
