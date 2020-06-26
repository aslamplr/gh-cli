#![cfg(feature = "basic-info")]
use super::repos::{Repo, RepoRequest};
pub use crate::graphql::repo_basic_info::repo_basic_info_query as basic_info_response;
use crate::utils::http::HttpMethod;
use crate::{graphql::repo_basic_info::RepoBasicInfoQuery, utils::graphql::query_graphql};
use anyhow::Result;
use async_trait::async_trait;
use graphql_client::GraphQLQuery as _;

#[cfg(not(test))]
const BASE_URL: &str = "https://api.github.com/repos";

pub type BasicInfoResponse = basic_info_response::ResponseData;

#[async_trait]
pub trait BasicInfo {
    async fn get_basic_info(&self) -> Result<BasicInfoResponse>;
    async fn get_raw_readme(&self) -> Result<String>;
}

impl From<&Repo<'_>> for basic_info_response::Variables {
    fn from(repo: &Repo<'_>) -> Self {
        basic_info_response::Variables {
            name: repo.repo_name.to_owned(),
            owner: repo.repo_owner.to_owned(),
        }
    }
}

#[async_trait]
impl BasicInfo for RepoRequest<'_> {
    async fn get_basic_info(&self) -> Result<BasicInfoResponse> {
        let RepoRequest {
            repo,
            auth_token,
            http_client,
        } = self;
        let graphql_query = RepoBasicInfoQuery::build_query(repo.into());
        let resp = query_graphql(http_client, graphql_query, &auth_token).await?;
        resp.data
            .ok_or_else(|| anyhow::anyhow!("Couldn't find repository basic information!"))
    }

    async fn get_raw_readme(&self) -> Result<String> {
        let RepoRequest {
            repo,
            auth_token,
            http_client,
        } = self;
        let resp = http_client
            .request(
                &with_base_url!("{}/readme", repo),
                HttpMethod::GET,
                auth_token,
            )
            .header("Accept", "application/vnd.github.VERSION.raw")
            .call()
            .await?;
        resp.body().await
    }
}

#[cfg(test)]
mod tests {
    use super::basic_info_response::*;
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn get_basic_info() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("POST", "/graphql")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "data": {
                  "repository": {
                    "nameWithOwner": "aslamplr/gh-cli",
                    "description": "🖥 Yet another unofficial GitHub CLI! Minimalistic, opinionated, and unofficial by default.",
                    "createdAt": "2020-04-15T23:59:51Z",
                    "pushedAt": "2020-06-04T06:35:57Z",
                    "homepageUrl": "https://github.com/aslamplr/gh-cli#gh-cli",
                    "isPrivate": false,
                    "isArchived": false,
                    "primaryLanguage": {
                      "name": "Rust"
                    },
                    "licenseInfo": {
                      "name": "MIT License"
                    },
                    "stargazers": {
                      "totalCount": 1
                    }
                  }
                }
              }"#)
            .expect(1)
            .create();

        let expected_basic_info = BasicInfoResponse {
            repository: Some(RepoBasicInfoQueryRepository {
                name_with_owner: "aslamplr/gh-cli".into(),
                description: Some("🖥 Yet another unofficial GitHub CLI! Minimalistic, opinionated, and unofficial by default.".into()),
                created_at: "2020-04-15T23:59:51Z".parse()?,
                pushed_at: Some("2020-06-04T06:35:57Z".parse()?),
                homepage_url: Some("https://github.com/aslamplr/gh-cli#gh-cli".into()),
                is_private: false,
                is_archived: false,
                primary_language: Some(RepoBasicInfoQueryRepositoryPrimaryLanguage {
                    name: "Rust".into()
                }),
                license_info: Some(RepoBasicInfoQueryRepositoryLicenseInfo {
                    name: "MIT License".into()
                }),
                stargazers: RepoBasicInfoQueryRepositoryStargazers {
                    total_count: 1
                }
            })

        };

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let basic_info = repo_req.get_basic_info().await?;

        m.assert();
        assert_eq!(basic_info, expected_basic_info);
        Ok(())
    }

    #[tokio::test]
    async fn get_raw_readme() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/readme")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .match_header("Accept", "application/vnd.github.VERSION.raw")
            .with_status(201)
            .with_body(r#"# Readme "#)
            .expect(1)
            .create();

        let expected_output = "# Readme ".to_string();
        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let output = repo_req.get_raw_readme().await?;

        m.assert();
        assert_eq!(output, expected_output);
        Ok(())
    }
}
