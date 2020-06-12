#![cfg(feature = "graphql-api")]
use super::http::{post, HttpBody};
use anyhow::Result;
use graphql_client::{QueryBody, Response};

#[cfg(not(test))]
const BASE_URL: &str = crate::BASE_URL;

pub async fn query_graphql<T, U>(query: QueryBody<T>, auth_token: &str) -> Result<Response<U>>
where
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    let body = HttpBody::try_from_serialize(&query)?;
    let url = with_base_url!("graphql");
    post(&url, body, auth_token).await?.deserialize().await
}
