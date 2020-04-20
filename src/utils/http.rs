use crate::Result;
use bytes::buf::BufExt;
pub use hyper::Method as HttpMethod;
use hyper::{
    body::aggregate,
    http::{request::Builder, response::Response},
    Body, Client, Request, Uri,
};
use hyper_tls::HttpsConnector;

type HttpsClient =
    hyper::client::Client<HttpsConnector<hyper::client::connect::HttpConnector>, hyper::Body>;

fn create_https_client() -> HttpsClient {
    let https = HttpsConnector::new();
    Client::builder().build(https)
}

fn create_request(auth_token: &str) -> hyper::http::request::Builder {
    Request::builder()
        .header("Authorization", format!("bearer {}", auth_token))
        .header("User-Agent", "gh-actions-secrets-cli")
}

pub struct HttpBody {
    inner: Body,
}

impl HttpBody {
    pub fn empty() -> Self {
        HttpBody {
            inner: Body::empty(),
        }
    }

    pub fn try_from_serialize<T>(body: &T) -> Result<Self>
    where
        T: serde::Serialize,
    {
        Ok(HttpBody {
            inner: Body::from(serde_json::to_string(&body)?),
        })
    }
}

pub struct HttpResponse {
    inner: Response<Body>,
}

impl HttpResponse {
    pub fn from(response_body: Response<Body>) -> HttpResponse {
        HttpResponse {
            inner: response_body,
        }
    }

    pub async fn deserialize<T>(self) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let body = aggregate(self.inner).await?;
        let body = serde_json::from_reader(body.reader())?;
        Ok(body)
    }

    pub fn status(self) -> hyper::StatusCode {
        self.inner.status()
    }
}

pub struct HttpRequest {
    client: HttpsClient,
    request: Request<hyper::Body>,
}

impl HttpRequest {
    pub fn from(client: HttpsClient, request: Request<hyper::Body>) -> Self {
        HttpRequest { client, request }
    }

    pub async fn call(self) -> Result<HttpResponse> {
        let res = self.client.request(self.request).await?;
        Ok(HttpResponse::from(res))
    }
}

pub struct HttpRequestBuilder {
    client: HttpsClient,
    builder: Builder,
}

impl HttpRequestBuilder {
    pub fn from(client: HttpsClient, builder: Builder) -> Self {
        Self { client, builder }
    }

    pub fn _header(self, key: &str, value: &str) -> Self {
        let builder = self.builder.header(key, value);
        Self { builder, ..self }
    }

    pub fn body(self, body: HttpBody) -> Result<HttpRequest> {
        let request = self.builder.body(body.inner)?;
        Ok(HttpRequest::from(self.client, request))
    }

    pub async fn call(self) -> Result<HttpResponse> {
        self.body(HttpBody::empty())?.call().await
    }
}

pub fn request(url: &str, method: HttpMethod, auth_token: &str) -> HttpRequestBuilder {
    let uri = url.parse::<Uri>().unwrap();
    let client = create_https_client();
    let builder = create_request(auth_token).method(method).uri(uri);
    HttpRequestBuilder::from(client, builder)
}

pub async fn get(url: &str, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::GET, &auth_token).call().await
}

pub async fn _post(url: &str, body: HttpBody, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::POST, &auth_token)
        .body(body)?
        .call()
        .await
}

pub async fn put(url: &str, body: HttpBody, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::PUT, &auth_token)
        .body(body)?
        .call()
        .await
}

pub async fn _patch(url: &str, body: HttpBody, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::PATCH, &auth_token)
        .body(body)?
        .call()
        .await
}

pub async fn delete(url: &str, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::DELETE, &auth_token).call().await
}

pub async fn _options(url: &str, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::OPTIONS, &auth_token).call().await
}
