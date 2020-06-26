#![cfg(any(feature = "graphql-api", feature = "http-api"))]
use anyhow::{anyhow, Result};
pub use reqwest::Method as HttpMethod;
use reqwest::{Body, Client, Request, RequestBuilder, Response};

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

fn create_https_client() -> Result<Client> {
    reqwest::ClientBuilder::new()
        .user_agent(APP_USER_AGENT)
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

pub fn request(url: &str, method: HttpMethod, auth_token: &str) -> Result<HttpRequestBuilder> {
    let client = create_https_client()?;
    let builder = client.request(method, url).bearer_auth(auth_token);
    Ok(HttpRequestBuilder::from(client, builder))
}

pub async fn get(url: &str, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::GET, &auth_token)?.call().await
}

pub async fn post(url: &str, body: HttpBody, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::POST, &auth_token)?
        .body(body)?
        .call()
        .await
}

pub async fn put(url: &str, body: HttpBody, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::PUT, &auth_token)?
        .body(body)?
        .call()
        .await
}

pub async fn _patch(url: &str, body: HttpBody, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::PATCH, &auth_token)?
        .body(body)?
        .call()
        .await
}

pub async fn delete(url: &str, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::DELETE, &auth_token)?.call().await
}

pub async fn _options(url: &str, auth_token: &str) -> Result<HttpResponse> {
    request(&url, HttpMethod::OPTIONS, &auth_token)?
        .call()
        .await
}
