#![cfg(any(feature = "graphql-api", feature = "http-api"))]
use anyhow::{anyhow, Result};
pub use reqwest::Method as HttpMethod;
use reqwest::{header, Body, Client, Request, RequestBuilder, Response};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

fn create_https_client(auth_token: &str) -> Result<Client> {
    let headers = {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", auth_token))?,
        );
        headers
    };
    reqwest::ClientBuilder::new()
        .user_agent(APP_USER_AGENT)
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| anyhow::anyhow!("An error occured while creating the client, {:?}", e))
}

pub struct HttpBody {
    inner: Option<Body>,
}

impl HttpBody {
    pub fn empty() -> Self {
        HttpBody { inner: None }
    }

    pub fn try_from_serialize<T>(body: &T) -> Result<Self>
    where
        T: serde::Serialize,
    {
        Ok(HttpBody::from(serde_json::to_string(&body)?))
    }
}

impl<T: Into<Body>> From<T> for HttpBody {
    fn from(body: T) -> Self {
        HttpBody {
            inner: Some(body.into()),
        }
    }
}

pub struct HttpResponse {
    inner: Response,
}

impl HttpResponse {
    pub fn from(response: Response) -> HttpResponse {
        HttpResponse { inner: response }
    }

    pub async fn deserialize<T>(self) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.inner
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("An error occured while deserializing, {:?}", e))
    }

    pub fn get_header(&self, key: &str) -> Option<String> {
        self.inner
            .headers()
            .get(key)
            .and_then(|x| x.to_str().ok())
            .map(String::from)
    }

    pub async fn body(self) -> Result<String> {
        self.inner
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("An error occured while reading response body, {:?}", e))
    }
}

pub struct HttpRequest {
    client: Client,
    request: Request,
}

impl HttpRequest {
    pub fn from(client: Client, request: Request) -> Self {
        HttpRequest { client, request }
    }

    pub async fn call(self) -> Result<HttpResponse> {
        let res = self.client.execute(self.request).await?;
        let status = res.status();
        if status.is_redirection() || status.is_success() {
            Ok(HttpResponse::from(res))
        } else {
            let body = res.text().await?;
            Err(anyhow!("[{}] {}", status, body))
        }
    }
}

pub struct HttpRequestBuilder {
    client: Client,
    builder: RequestBuilder,
}

impl HttpRequestBuilder {
    pub fn from(client: Client, builder: RequestBuilder) -> Self {
        Self { client, builder }
    }

    pub fn header(self, key: &str, value: &str) -> Self {
        let builder = self.builder.header(key, value);
        Self { builder, ..self }
    }

    pub fn body(self, body: HttpBody) -> Result<HttpRequest> {
        let req_builder = match body.inner {
            Some(body) => self.builder.body(body),
            None => self.builder,
        };
        let request = req_builder.build()?;
        Ok(HttpRequest::from(self.client, request))
    }

    pub async fn call(self) -> Result<HttpResponse> {
        self.body(HttpBody::empty())?.call().await
    }
}

#[derive(Debug)]
pub struct HttpClient {
    inner: Client,
}

impl HttpClient {
    pub fn new(auth_token: &str) -> Result<Self> {
        Ok(Self {
            inner: create_https_client(auth_token)?,
        })
    }

    pub fn request(&self, url: &str, method: HttpMethod) -> HttpRequestBuilder {
        let client = self.inner.clone();
        let builder = client.request(method, url);
        HttpRequestBuilder::from(client, builder)
    }

    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.request(&url, HttpMethod::GET).call().await
    }

    pub async fn post(&self, url: &str, body: HttpBody) -> Result<HttpResponse> {
        self.request(&url, HttpMethod::POST)
            .body(body)?
            .call()
            .await
    }

    pub async fn put(&self, url: &str, body: HttpBody) -> Result<HttpResponse> {
        self.request(&url, HttpMethod::PUT).body(body)?.call().await
    }

    pub async fn _patch(&self, url: &str, body: HttpBody) -> Result<HttpResponse> {
        self.request(&url, HttpMethod::PATCH)
            .body(body)?
            .call()
            .await
    }

    pub async fn delete(&self, url: &str) -> Result<HttpResponse> {
        self.request(&url, HttpMethod::DELETE).call().await
    }

    pub async fn _options(&self, url: &str) -> Result<HttpResponse> {
        self.request(&url, HttpMethod::OPTIONS).call().await
    }
}
