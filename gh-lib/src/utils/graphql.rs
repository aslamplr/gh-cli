#![cfg(feature = "graphql-api")]
use super::http::{HttpBody, HttpClient};
use anyhow::Result;
use graphql_client::{QueryBody, Response};

#[cfg(not(test))]
const BASE_URL: &str = crate::BASE_URL;

pub async fn query_graphql<T, U>(
    http_client: &HttpClient,
    query: QueryBody<T>,
    auth_token: &str,
) -> Result<Response<U>>
where
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    let body = HttpBody::try_from_serialize(&query)?;
    let url = with_base_url!("graphql");
    http_client
        .post(&url, body, auth_token)
        .await?
        .deserialize()
        .await
}
