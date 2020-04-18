use crate::Result;
use bytes::buf::BufExt;
pub use hyper::Method as HttpMethod;
use hyper::{body::aggregate, http::response::Response, Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;

fn create_https_client(
) -> hyper::client::Client<HttpsConnector<hyper::client::connect::HttpConnector>, hyper::Body> {
    let https = HttpsConnector::new();
    Client::builder().build::<_, hyper::Body>(https)
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

pub async fn request(
    url: &str,
    method: HttpMethod,
    body: HttpBody,
    auth_token: &str,
) -> Result<HttpResponse> {
    let uri = url.parse::<Uri>().unwrap();
    let client = create_https_client();
    let req = create_request(auth_token)
        .method(method)
        .uri(uri)
        .body(body.inner)?;
    let res = client.request(req).await?;
    let res = HttpResponse::from(res);
    Ok(res)
}
