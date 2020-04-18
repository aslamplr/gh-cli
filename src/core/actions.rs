use super::repos::{Repo, ReposRequestParams};
use crate::utils::http::request;
use crate::{
    utils::{
        http::{HttpBody, HttpMethod},
        sealed_box::seal,
    },
    Result,
};
use serde::{Deserialize, Serialize};

impl ReposRequestParams<'_> {
    pub async fn get_public_key(&self) -> Result<PublicKeyResponse> {
        get_public_key(&self).await
    }

    pub async fn get_secret_list(&self) -> Result<SecretListResponse> {
        get_secret_list(&self).await
    }

    pub async fn get_a_secret(&self, secret_key: &str) -> Result<Secret> {
        get_a_secret(&self, &secret_key).await
    }

    pub async fn save_secret(&self, secret_key: &str, secret_value: &str) -> Result<()> {
        let public_key = self.get_public_key().await?;
        let secret_save_req = SecretSaveRequest::from(secret_key, secret_value, public_key)?;
        secret_save_req.make_api_call(&self).await
    }

    pub async fn delete_a_secret(&self, secret_key: &str) -> Result<()> {
        delete_a_secret(&self, secret_key).await
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Secret {
    name: String,
    created_at: String,
    updated_at: String,
}

impl std::fmt::Display for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(&self).unwrap_or_default()
        )
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SecretListResponse {
    total_count: u32,
    secrets: Vec<Secret>,
}

impl std::fmt::Display for SecretListResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(&self).unwrap_or_default()
        )
    }
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

    async fn make_api_call(&self, params: &ReposRequestParams<'_>) -> Result<()> {
        put_gh_secret(&params, &self.key, &self).await
    }
}

async fn get_from_gh<T>(path: &str, params: &ReposRequestParams<'_>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let ReposRequestParams(repo, auth_token) = params;
    let Repo {
        repo_owner,
        repo_name,
    } = repo;
    let url = format!(
        "https://api.github.com/repos/{}/{}/{}",
        repo_owner, repo_name, path
    );
    let resp = request(&url, HttpMethod::GET, HttpBody::empty(), &auth_token).await?;
    let resp = resp.deserialize().await?;
    Ok(resp)
}

async fn get_public_key(params: &ReposRequestParams<'_>) -> Result<PublicKeyResponse> {
    get_from_gh("actions/secrets/public-key", &params).await
}

async fn get_secret_list(params: &ReposRequestParams<'_>) -> Result<SecretListResponse> {
    get_from_gh("actions/secrets", &params).await
}

async fn get_a_secret(params: &ReposRequestParams<'_>, secret_key: &str) -> Result<Secret> {
    get_from_gh(&format!("actions/secrets/{}", secret_key), &params).await
}

async fn put_gh_secret(
    params: &ReposRequestParams<'_>,
    secret_key: &str,
    secret_save_req: &SecretSaveRequest,
) -> Result<()> {
    let ReposRequestParams(repo, auth_token) = params;
    let Repo {
        repo_owner,
        repo_name,
    } = repo;
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/secrets/{}",
        repo_owner, repo_name, secret_key
    );
    let res = request(
        &url,
        HttpMethod::PUT,
        HttpBody::try_from_serialize(&secret_save_req)?,
        auth_token,
    )
    .await?;
    println!("Response: {}", res.status());
    Ok(())
}

async fn delete_a_secret(params: &ReposRequestParams<'_>, secret_key: &str) -> Result<()> {
    let ReposRequestParams(repo, auth_token) = params;
    let Repo {
        repo_owner,
        repo_name,
    } = repo;
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/secrets/{}",
        repo_owner, repo_name, secret_key
    );
    let res = request(&url, HttpMethod::DELETE, HttpBody::empty(), &auth_token).await?;
    println!("Response: {}", res.status());
    Ok(())
}
