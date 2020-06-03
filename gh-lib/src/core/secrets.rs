use super::repos::RepoRequest;
use crate::utils::{
    http::{delete, get, put, HttpBody},
    sealed_box::seal,
};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.github.com/repos";

#[async_trait]
pub trait Secrets {
    async fn get_public_key(&self) -> Result<PublicKeyResponse>;
    async fn get_all_secrets(&self) -> Result<SecretListResponse>;
    async fn get_a_secret(&self, secret_key: &str) -> Result<Secret>;
    async fn save_secret(&self, secret_key: &str, secret_value: &str) -> Result<()>;
    async fn delete_a_secret(&self, secret_key: &str) -> Result<()>;
}

#[async_trait]
impl Secrets for RepoRequest<'_> {
    async fn get_public_key(&self) -> Result<PublicKeyResponse> {
        get_public_key(&self).await
    }

    async fn get_all_secrets(&self) -> Result<SecretListResponse> {
        get_all_secrets(&self).await
    }

    async fn get_a_secret(&self, secret_key: &str) -> Result<Secret> {
        get_a_secret(&self, &secret_key).await
    }

    async fn save_secret(&self, secret_key: &str, secret_value: &str) -> Result<()> {
        let public_key = self.get_public_key().await?;
        let secret_save_req = SecretSaveRequest::from(secret_key, secret_value, public_key)?;
        secret_save_req.make_api_call(&self).await
    }

    async fn delete_a_secret(&self, secret_key: &str) -> Result<()> {
        delete_a_secret(&self, secret_key).await
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Secret {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SecretListResponse {
    pub total_count: u32,
    pub secrets: Vec<Secret>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PublicKeyResponse {
    key_id: String,
    key: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct SecretSaveRequest {
    #[serde(skip_serializing)]
    key: String,
    encrypted_value: String,
    key_id: String,
    #[serde(skip_serializing)]
    public_key: PublicKeyResponse,
}

impl SecretSaveRequest {
    fn from(key: &str, value: &str, public_key: PublicKeyResponse) -> Result<Self> {
        let encrypted_value = seal(value, &public_key.key)?;
        let key = key.into();
        let key_id = public_key.key_id.to_owned();
        Ok(SecretSaveRequest {
            key,
            encrypted_value,
            key_id,
            public_key,
        })
    }

    async fn make_api_call(&self, params: &RepoRequest<'_>) -> Result<()> {
        put_gh_secret(&params, &self.key, &self).await
    }
}

async fn get_from_gh<T>(path: &str, params: &RepoRequest<'_>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let RepoRequest(repo, auth_token) = params;
    let url = format!("{}/{}/{}", BASE_URL, repo, path);
    let resp = get(&url, &auth_token).await?;
    let resp = resp.deserialize().await?;
    Ok(resp)
}

async fn get_public_key(params: &RepoRequest<'_>) -> Result<PublicKeyResponse> {
    get_from_gh("actions/secrets/public-key", &params).await
}

async fn get_all_secrets(params: &RepoRequest<'_>) -> Result<SecretListResponse> {
    get_from_gh("actions/secrets", &params).await
}

async fn get_a_secret(params: &RepoRequest<'_>, secret_key: &str) -> Result<Secret> {
    get_from_gh(&format!("actions/secrets/{}", secret_key), &params).await
}

async fn put_gh_secret(
    params: &RepoRequest<'_>,
    secret_key: &str,
    secret_save_req: &SecretSaveRequest,
) -> Result<()> {
    let RepoRequest(repo, auth_token) = params;
    let url = format!("{}/{}/actions/secrets/{}", BASE_URL, repo, secret_key);
    put(
        &url,
        HttpBody::try_from_serialize(&secret_save_req)?,
        auth_token,
    )
    .await?;
    Ok(())
}

async fn delete_a_secret(params: &RepoRequest<'_>, secret_key: &str) -> Result<()> {
    let RepoRequest(repo, auth_token) = params;
    let url = format!("{}/{}/actions/secrets/{}", BASE_URL, repo, secret_key);
    delete(&url, &auth_token).await?;
    Ok(())
}
