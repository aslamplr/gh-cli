#![cfg(feature = "collaborators")]
use super::repos::RepoRequest;
use crate::utils::http::{HttpBody, StatusCode};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(not(test))]
const BASE_URL: &str = "https://api.github.com/repos";

#[async_trait]
pub trait Collaborators {
    async fn get_collaborators(&self) -> Result<Vec<Collaborator>>;
    async fn is_collaborator(&self, username: &str) -> Result<bool>;
    async fn add_collaborator<T: Into<CollaboratorPermission> + Send>(
        &self,
        username: &str,
        permission: T,
    ) -> Result<AddCollaboratorResponse>;
    async fn remove_collaborator(&self, username: &str) -> Result<()>;
    async fn get_permission(&self, username: &str) -> Result<CollaboratorPermissionResponse>;
}

#[async_trait]
impl Collaborators for RepoRequest<'_> {
    async fn get_collaborators(&self) -> Result<Vec<Collaborator>> {
        get_collaborators(&self).await
    }

    async fn is_collaborator(&self, username: &str) -> Result<bool> {
        is_collaborator(&self, username).await
    }

    async fn add_collaborator<T: Into<CollaboratorPermission> + Send>(
        &self,
        username: &str,
        permission: T,
    ) -> Result<AddCollaboratorResponse> {
        add_collaborator(&self, username, permission.into()).await
    }

    async fn remove_collaborator(&self, username: &str) -> Result<()> {
        remove_collaborator(&self, username).await
    }

    async fn get_permission(&self, username: &str) -> Result<CollaboratorPermissionResponse> {
        get_permission(&self, username).await
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Collaborator {
    pub login: String,
    pub id: u32,
    pub node_id: String,
    pub avatar_url: Option<String>,
    pub gravatar_id: Option<String>,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub site_admin: bool,
    pub permissions: Permission,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Permission {
    pub pull: bool,
    pub push: bool,
    pub admin: bool,
}

#[derive(Debug, PartialEq)]
pub enum CollaboratorPermission {
    Pull,
    Push,
    Admin,
    Maintain,
    Triage,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct CollaboratorPermissionResponse {
    pub permission: String,
}

impl<T: AsRef<str>> From<T> for CollaboratorPermission {
    fn from(permission: T) -> Self {
        match permission.as_ref() {
            "pull" => Self::Pull,
            "push" => Self::Push,
            "admin" => Self::Admin,
            "maintain" => Self::Maintain,
            "triage" => Self::Triage,
            _ => Self::Push,
        }
    }
}

impl std::fmt::Display for CollaboratorPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let permission = match self {
            Self::Pull => "pull",
            Self::Push => "push",
            Self::Admin => "admin",
            Self::Maintain => "maintain",
            Self::Triage => "triage",
        };
        write!(f, "{}", permission)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum AddCollaboratorResponse {
    InvitationCreated,
    AlreadyCollaborator,
}

async fn get_collaborators(params: &RepoRequest<'_>) -> Result<Vec<Collaborator>> {
    let RepoRequest { repo, http_client } = params;
    let url = with_base_url!("{}/collaborators", repo);
    http_client.get(&url).await?.deserialize().await
}

async fn is_collaborator(params: &RepoRequest<'_>, username: &str) -> Result<bool> {
    let RepoRequest { repo, http_client } = params;
    let url = with_base_url!("{}/collaborators/{}", repo, username);
    Ok(http_client.get(&url).await.is_ok())
}

async fn add_collaborator(
    params: &RepoRequest<'_>,
    username: &str,
    permission: CollaboratorPermission,
) -> Result<AddCollaboratorResponse> {
    let RepoRequest { repo, http_client } = params;
    let url = with_base_url!("{}/collaborators/{}", repo, username);
    match http_client
        .put(
            &url,
            HttpBody::from(format!(r#"{{"permission": "{}"}}"#, permission)),
        )
        .await?
        .status()
    {
        StatusCode::CREATED => Ok(AddCollaboratorResponse::InvitationCreated),
        StatusCode::NO_CONTENT => Ok(AddCollaboratorResponse::AlreadyCollaborator),
        status => Err(anyhow::anyhow!("Unknown: {}", status)),
    }
}

async fn remove_collaborator(params: &RepoRequest<'_>, username: &str) -> Result<()> {
    let RepoRequest { repo, http_client } = params;
    let url = with_base_url!("{}/collaborators/{}", repo, username);
    http_client.delete(&url).await?;
    Ok(())
}

async fn get_permission(
    params: &RepoRequest<'_>,
    username: &str,
) -> Result<CollaboratorPermissionResponse> {
    let RepoRequest { repo, http_client } = params;
    let url = with_base_url!("{}/collaborators/{}/permission", repo, username);
    Ok(http_client.get(&url).await?.deserialize().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn get_collaborators() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/collaborators")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
  {
    "login": "octocat",
    "id": 1,
    "node_id": "MDQ6VXNlcjE=",
    "avatar_url": "https://github.com/images/error/octocat_happy.gif",
    "gravatar_id": "",
    "url": "https://api.github.com/users/octocat",
    "html_url": "https://github.com/octocat",
    "followers_url": "https://api.github.com/users/octocat/followers",
    "following_url": "https://api.github.com/users/octocat/following{/other_user}",
    "gists_url": "https://api.github.com/users/octocat/gists{/gist_id}",
    "starred_url": "https://api.github.com/users/octocat/starred{/owner}{/repo}",
    "subscriptions_url": "https://api.github.com/users/octocat/subscriptions",
    "organizations_url": "https://api.github.com/users/octocat/orgs",
    "repos_url": "https://api.github.com/users/octocat/repos",
    "events_url": "https://api.github.com/users/octocat/events{/privacy}",
    "received_events_url": "https://api.github.com/users/octocat/received_events",
    "type": "User",
    "site_admin": false,
    "permissions": {
      "pull": true,
      "push": true,
      "admin": false
    }
  }
]"#,
            )
            .expect(1)
            .create();

        let expected_collaborators = vec![Collaborator {
            login: "octocat".into(),
            id: 1,
            node_id: "MDQ6VXNlcjE=".into(),
            avatar_url: Some("https://github.com/images/error/octocat_happy.gif".into()),
            gravatar_id: Some("".into()),
            url: "https://api.github.com/users/octocat".into(),
            html_url: "https://github.com/octocat".into(),
            followers_url: "https://api.github.com/users/octocat/followers".into(),
            following_url: "https://api.github.com/users/octocat/following{/other_user}".into(),
            gists_url: "https://api.github.com/users/octocat/gists{/gist_id}".into(),
            starred_url: "https://api.github.com/users/octocat/starred{/owner}{/repo}".into(),
            subscriptions_url: "https://api.github.com/users/octocat/subscriptions".into(),
            organizations_url: "https://api.github.com/users/octocat/orgs".into(),
            repos_url: "https://api.github.com/users/octocat/repos".into(),
            events_url: "https://api.github.com/users/octocat/events{/privacy}".into(),
            received_events_url: "https://api.github.com/users/octocat/received_events".into(),
            data_type: "User".into(),
            site_admin: false,
            permissions: Permission {
                pull: true,
                push: true,
                admin: false,
            },
        }];

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let collaborators = repo_req.get_collaborators().await?;

        m.assert();
        assert_eq!(collaborators, expected_collaborators);
        Ok(())
    }

    #[tokio::test]
    async fn is_collaborator_truthy() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/collaborators/aslamplr")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(201)
            .expect(1)
            .create();

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let is_collaborator = repo_req.is_collaborator("aslamplr").await?;

        m.assert();
        assert!(is_collaborator);
        Ok(())
    }

    #[tokio::test]
    async fn is_collaborator_falsy() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/collaborators/aslamplr")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(404)
            .expect(1)
            .create();

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let is_collaborator = repo_req.is_collaborator("aslamplr").await?;

        m.assert();
        assert!(!is_collaborator);
        Ok(())
    }

    #[tokio::test]
    async fn add_contributor() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("PUT", "/aslamplr/gh-cli/collaborators/aslamplr")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(201)
            .expect(1)
            .create();

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let resp = repo_req
            .add_collaborator("aslamplr", CollaboratorPermission::Push)
            .await?;

        m.assert();
        assert_eq!(AddCollaboratorResponse::InvitationCreated, resp);
        Ok(())
    }

    #[tokio::test]
    async fn remove_collaborator() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("DELETE", "/aslamplr/gh-cli/collaborators/aslamplr")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(204)
            .expect(1)
            .create();

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        repo_req.remove_collaborator("aslamplr").await?;

        m.assert();
        Ok(())
    }

    #[tokio::test]
    async fn get_permission() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/collaborators/aslamplr/permission")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"
{
  "permission": "admin",
  "user": {
    "login": "octocat",
    "id": 1,
    "node_id": "MDQ6VXNlcjE=",
    "avatar_url": "https://github.com/images/error/octocat_happy.gif",
    "gravatar_id": "",
    "url": "https://api.github.com/users/octocat",
    "html_url": "https://github.com/octocat",
    "followers_url": "https://api.github.com/users/octocat/followers",
    "following_url": "https://api.github.com/users/octocat/following{/other_user}",
    "gists_url": "https://api.github.com/users/octocat/gists{/gist_id}",
    "starred_url": "https://api.github.com/users/octocat/starred{/owner}{/repo}",
    "subscriptions_url": "https://api.github.com/users/octocat/subscriptions",
    "organizations_url": "https://api.github.com/users/octocat/orgs",
    "repos_url": "https://api.github.com/users/octocat/repos",
    "events_url": "https://api.github.com/users/octocat/events{/privacy}",
    "received_events_url": "https://api.github.com/users/octocat/received_events",
    "type": "User",
    "site_admin": false
  }
}
"#,
            )
            .expect(1)
            .create();

        let expected_permission = CollaboratorPermissionResponse {
            permission: "admin".into(),
        };
        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let permission = repo_req.get_permission("aslamplr").await?;

        m.assert();
        assert_eq!(permission, expected_permission);
        Ok(())
    }
}
