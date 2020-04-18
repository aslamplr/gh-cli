use hyper::{Client, Request};
use hyper_tls::HttpsConnector;

pub fn create_https_client(
) -> hyper::client::Client<HttpsConnector<hyper::client::connect::HttpConnector>, hyper::Body> {
    let https = HttpsConnector::new();
    Client::builder().build::<_, hyper::Body>(https)
}

pub fn create_request(auth_token: &str) -> hyper::http::request::Builder {
    Request::builder()
        .header("Authorization", format!("bearer {}", auth_token))
        .header("User-Agent", "gh-actions-secrets-cli")
}
