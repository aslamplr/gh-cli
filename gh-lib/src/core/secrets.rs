#![cfg(feature = "secrets")]
use super::repos::RepoRequest;
#[cfg(feature = "secrets-save")]
use crate::utils::http::HttpBody;
#[cfg(feature = "secrets-save")]
use crate::utils::sealed_box::seal;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(not(test))]
const BASE_URL: &str = "https://api.github.com/repos";

#[async_trait]
pub trait Secrets {
    async fn get_public_key(&self) -> Result<PublicKeyResponse>;
    async fn get_all_secrets(&self) -> Result<SecretListResponse>;
    async fn get_a_secret(&self, name: &str) -> Result<Secret>;
    #[cfg(feature = "secrets-save")]
    async fn save_secret(&self, name: &str, value: &str) -> Result<()>;
    async fn delete_a_secret(&self, name: &str) -> Result<()>;
}

#[async_trait]
impl Secrets for RepoRequest<'_> {
    async fn get_public_key(&self) -> Result<PublicKeyResponse> {
        get_public_key(&self).await
    }

    async fn get_all_secrets(&self) -> Result<SecretListResponse> {
        get_all_secrets(&self).await
    }

    async fn get_a_secret(&self, name: &str) -> Result<Secret> {
        get_a_secret(&self, &name).await
    }

    #[cfg(feature = "secrets-save")]
    async fn save_secret(&self, name: &str, value: &str) -> Result<()> {
        let public_key = self.get_public_key().await?;
        let secret_save_req = SecretSaveRequest::from(name, value, public_key)?;
        secret_save_req.make_api_call(&self).await
    }

    async fn delete_a_secret(&self, name: &str) -> Result<()> {
        delete_a_secret(&self, name).await
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Secret {
    pub name: String,
    #[cfg(feature = "chrono")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[cfg(not(feature = "chrono"))]
    pub created_at: String,
    #[cfg(feature = "chrono")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[cfg(not(feature = "chrono"))]
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct SecretListResponse {
    pub total_count: u32,
    pub secrets: Vec<Secret>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct PublicKeyResponse {
    key_id: String,
    key: String,
}

#[cfg(feature = "secrets-save")]
#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct SecretSaveRequest {
    #[serde(skip_serializing)]
    key: String,
    encrypted_value: String,
    key_id: String,
    #[serde(skip_serializing)]
    public_key: PublicKeyResponse,
}

#[cfg(feature = "secrets-save")]
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
    let RepoRequest { repo, http_client } = params;
    let url = with_base_url!("{}/{}", repo, path);
    let resp = http_client.get(&url).await?;
    let resp = resp.deserialize().await?;
    Ok(resp)
}

async fn get_public_key(params: &RepoRequest<'_>) -> Result<PublicKeyResponse> {
    get_from_gh("actions/secrets/public-key", &params).await
}

async fn get_all_secrets(params: &RepoRequest<'_>) -> Result<SecretListResponse> {
    get_from_gh("actions/secrets", &params).await
}

async fn get_a_secret(params: &RepoRequest<'_>, name: &str) -> Result<Secret> {
    get_from_gh(&format!("actions/secrets/{}", name), &params).await
}

#[cfg(feature = "secrets-save")]
async fn put_gh_secret(
    params: &RepoRequest<'_>,
    name: &str,
    secret_save_req: &SecretSaveRequest,
) -> Result<()> {
    let RepoRequest { repo, http_client } = params;
    let url = with_base_url!("{}/actions/secrets/{}", repo, name);
    http_client
        .put(&url, HttpBody::try_from_serialize(&secret_save_req)?)
        .await?;
    Ok(())
}

async fn delete_a_secret(params: &RepoRequest<'_>, name: &str) -> Result<()> {
    let RepoRequest { repo, http_client } = params;
    let url = with_base_url!("{}/actions/secrets/{}", repo, name);
    http_client.delete(&url).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn get_all_secrets() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/actions/secrets")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "total_count": 2,
                "secrets": [
                  {
                    "name": "GH_TOKEN",
                    "created_at": "2019-08-10T14:59:22Z",
                    "updated_at": "2020-01-10T14:59:22Z"
                  },
                  {
                    "name": "GIST_ID",
                    "created_at": "2020-01-10T10:59:22Z",
                    "updated_at": "2020-01-11T11:59:22Z"
                  }
                ]
              }"#,
            )
            .expect(1)
            .create();

        let expected_secrets = SecretListResponse {
            total_count: 2,
            secrets: vec![
                Secret {
                    name: "GH_TOKEN".into(),
                    created_at: "2019-08-10T14:59:22Z".parse()?,
                    updated_at: "2020-01-10T14:59:22Z".parse()?,
                },
                Secret {
                    name: "GIST_ID".into(),
                    created_at: "2020-01-10T10:59:22Z".parse()?,
                    updated_at: "2020-01-11T11:59:22Z".parse()?,
                },
            ],
        };

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let secrets = repo_req.get_all_secrets().await?;

        m.assert();
        assert_eq!(secrets, expected_secrets);
        Ok(())
    }

    #[tokio::test]
    async fn get_a_secret() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/actions/secrets/GH_TOKEN")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "name": "GH_TOKEN",
                    "created_at": "2019-08-10T14:59:22Z",
                    "updated_at": "2020-01-10T14:59:22Z"
                  }"#,
            )
            .expect(1)
            .create();

        let expected_secret = Secret {
            name: "GH_TOKEN".into(),
            created_at: "2019-08-10T14:59:22Z".parse()?,
            updated_at: "2020-01-10T14:59:22Z".parse()?,
        };

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let secret = repo_req.get_a_secret("GH_TOKEN").await?;

        m.assert();
        assert_eq!(secret, expected_secret);
        Ok(())
    }

    #[tokio::test]
    async fn get_public_key() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/actions/secrets/public-key")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "key_id": "012345678912345678",
                    "key": "2Sg8iYjAxxmI2LvUXpJjkYrMxURPc8r+dB7TJyvv1234"
                  }"#,
            )
            .expect(1)
            .create();

        let expected_public_key = PublicKeyResponse {
            key_id: "012345678912345678".into(),
            key: "2Sg8iYjAxxmI2LvUXpJjkYrMxURPc8r+dB7TJyvv1234".into(),
        };

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let public_key = repo_req.get_public_key().await?;

        m.assert();
        assert_eq!(public_key, expected_public_key);
        Ok(())
    }

    #[cfg(feature = "secrets-save")]
    #[tokio::test]
    async fn save_secret() -> Result<()> {
        let public_key_base64 = {
            use sodiumoxide::crypto::box_::{curve25519xsalsa20poly1305::PublicKey, gen_keypair};

            let (pk, _) = gen_keypair();
            let PublicKey(pk_bytes) = pk;
            base64::encode(pk_bytes)
        };

        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m1 = mock("GET", "/aslamplr/gh-cli/actions/secrets/public-key")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(&format!(
                r#"{{
                "key_id": "012345678912345678",
                "key": "{}"
              }}"#,
                public_key_base64
            ))
            .expect(1)
            .create();

        let m2 = mock("PUT", "/aslamplr/gh-cli/actions/secrets/GH_TOKEN")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .match_body(Matcher::Regex("encrypted_value".to_string()))
            .match_body(Matcher::Regex("key_id".to_string()))
            .match_body(Matcher::Regex("012345678912345678".to_string()))
            .with_status(201)
            .expect(1)
            .create();

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        repo_req.save_secret("GH_TOKEN", "SECRET").await?;

        m1.assert();
        m2.assert();
        Ok(())
    }

    #[tokio::test]
    async fn delete_a_secret() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("DELETE", "/aslamplr/gh-cli/actions/secrets/GH_TOKEN")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(204)
            .with_header("content-type", "application/json")
            .expect(1)
            .create();

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        repo_req.delete_a_secret("GH_TOKEN").await?;

        m.assert();
        Ok(())
    }
}
