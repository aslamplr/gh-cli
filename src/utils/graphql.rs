use super::http::{post, HttpBody};
use anyhow::Result;
use graphql_client::{QueryBody, Response};

const GH_GRAPHQL_URL: &str = "https://api.github.com/graphql";

pub async fn query_graphql<T, U>(query: QueryBody<T>, auth_token: &str) -> Result<Response<U>>
where
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    let body = HttpBody::try_from_serialize(&query)?;
    post(GH_GRAPHQL_URL, body, auth_token)
        .await?
        .deserialize()
        .await
}
