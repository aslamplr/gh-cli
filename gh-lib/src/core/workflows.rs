#![cfg(feature = "workflows")]
use super::repos::RepoRequest;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(not(test))]
const BASE_URL: &str = "https://api.github.com/repos";

#[async_trait]
pub trait Workflows {
    async fn get_all_workflows(&self) -> Result<WorkflowList>;
    async fn get_a_workflow(&self, workflow_id: u32) -> Result<Workflow>;
    async fn get_workflow_usage(&self, workflow_id: u32) -> Result<WorkflowUsage>;
}

#[async_trait]
impl Workflows for RepoRequest<'_> {
    async fn get_all_workflows(&self) -> Result<WorkflowList> {
        get_all_workflows(&self).await
    }

    async fn get_a_workflow(&self, workflow_id: u32) -> Result<Workflow> {
        get_a_workflow(&self, workflow_id).await
    }

    async fn get_workflow_usage(&self, workflow_id: u32) -> Result<WorkflowUsage> {
        get_workflow_usage(&self, workflow_id).await
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkflowList {
    pub total_count: u32,
    pub workflows: Vec<Workflow>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Workflow {
    pub id: u32,
    pub node_id: String,
    pub name: String,
    pub path: String,
    pub state: String,
    #[cfg(feature = "chrono")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[cfg(not(feature = "chrono"))]
    pub created_at: String,
    #[cfg(feature = "chrono")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[cfg(not(feature = "chrono"))]
    pub updated_at: String,
    pub url: String,
    pub html_url: String,
    pub badge_url: String,
}

macro_rules! platform_usage {
    (
        $(
            $(#[$docs:meta])*
            $field:ident,
        )+
    ) => {
        #[allow(non_snake_case)]
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        pub struct WorkflowUsagePlatform {
            $(
                $(#[$docs])*
                pub $field: Option<WorkflowUsageTiming>,
            )+
        }
    };
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkflowUsage {
    pub billable: WorkflowUsagePlatform,
}

platform_usage!(UBUNTU, MACOS, WINDOWS,);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkflowUsageTiming {
    pub total_ms: u32,
}

async fn get_all_workflows(params: &RepoRequest<'_>) -> Result<WorkflowList> {
    let RepoRequest {
        repo,
        auth_token,
        http_client,
    } = params;
    let url = with_base_url!("{}/actions/workflows", repo);
    let resp = http_client.get(&url, &auth_token).await?;
    let resp = resp.deserialize().await?;
    Ok(resp)
}

async fn get_a_workflow(params: &RepoRequest<'_>, workflow_id: u32) -> Result<Workflow> {
    let RepoRequest {
        repo,
        auth_token,
        http_client,
    } = params;
    let url = with_base_url!("{}/actions/workflows/{}", repo, workflow_id);
    let resp = http_client.get(&url, &auth_token).await?;
    let resp = resp.deserialize().await?;
    Ok(resp)
}

async fn get_workflow_usage(params: &RepoRequest<'_>, workflow_id: u32) -> Result<WorkflowUsage> {
    let RepoRequest {
        repo,
        auth_token,
        http_client,
    } = params;
    let url = with_base_url!("{}/actions/workflows/{}/timing", repo, workflow_id);
    let resp = http_client.get(&url, &auth_token).await?;
    // eprintln!("body: {:?}", resp.body().await);
    // Err(anyhow::anyhow!("Error!"))
    let resp = resp.deserialize().await?;
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn get_all_workflows() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/actions/workflows")
            .match_header("Authorization", Matcher::Exact(format!("Bearer {}", auth_token)))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "total_count": 2,
                "workflows": [
                  {
                    "id": 161335,
                    "node_id": "MDg6V29ya2Zsb3cxNjEzMzU=",
                    "name": "CI",
                    "path": ".github/workflows/blank.yml",
                    "state": "active",
                    "created_at": "2020-01-08T23:48:37.000-08:00",
                    "updated_at": "2020-01-08T23:50:21.000-08:00",
                    "url": "https://api.github.com/repos/octo-org/octo-repo/actions/workflows/161335",
                    "html_url": "https://github.com/octo-org/octo-repo/blob/master/.github/workflows/161335",
                    "badge_url": "https://github.com/octo-org/octo-repo/workflows/CI/badge.svg"
                  },
                  {
                    "id": 269289,
                    "node_id": "MDE4OldvcmtmbG93IFNlY29uZGFyeTI2OTI4OQ==",
                    "name": "Linter",
                    "path": ".github/workflows/linter.yml",
                    "state": "active",
                    "created_at": "2020-01-08T23:48:37.000-08:00",
                    "updated_at": "2020-01-08T23:50:21.000-08:00",
                    "url": "https://api.github.com/repos/octo-org/octo-repo/actions/workflows/269289",
                    "html_url": "https://github.com/octo-org/octo-repo/blob/master/.github/workflows/269289",
                    "badge_url": "https://github.com/octo-org/octo-repo/workflows/Linter/badge.svg"
                  }
                ]
              }"#)
            .expect(1)
            .create();

        let expected_workflows = WorkflowList {
            total_count: 2,
            workflows: vec![
                Workflow {
                    id: 161335,
                    node_id: "MDg6V29ya2Zsb3cxNjEzMzU=".into(),
                    name: "CI".into(),
                    path: ".github/workflows/blank.yml".into(),
                    state: "active".into(),
                    created_at: "2020-01-08T23:48:37.000-08:00".parse()?,
                    updated_at: "2020-01-08T23:50:21.000-08:00".parse()?,
                    url: "https://api.github.com/repos/octo-org/octo-repo/actions/workflows/161335"
                        .into(),
                    html_url:
                        "https://github.com/octo-org/octo-repo/blob/master/.github/workflows/161335"
                            .into(),
                    badge_url: "https://github.com/octo-org/octo-repo/workflows/CI/badge.svg"
                        .into(),
                },
                Workflow {
                    id: 269289,
                    node_id: "MDE4OldvcmtmbG93IFNlY29uZGFyeTI2OTI4OQ==".into(),
                    name: "Linter".into(),
                    path: ".github/workflows/linter.yml".into(),
                    state: "active".into(),
                    created_at: "2020-01-08T23:48:37.000-08:00".parse()?,
                    updated_at: "2020-01-08T23:50:21.000-08:00".parse()?,
                    url: "https://api.github.com/repos/octo-org/octo-repo/actions/workflows/269289"
                        .into(),
                    html_url:
                        "https://github.com/octo-org/octo-repo/blob/master/.github/workflows/269289"
                            .into(),
                    badge_url: "https://github.com/octo-org/octo-repo/workflows/Linter/badge.svg"
                        .into(),
                },
            ],
        };

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let workflows = repo_req.get_all_workflows().await?;

        m.assert();
        assert_eq!(workflows, expected_workflows);
        Ok(())
    }

    #[tokio::test]
    async fn get_a_workflow() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";
        let workflow_id = 161335;

        let m = mock("GET", "/aslamplr/gh-cli/actions/workflows/161335")
            .match_header("Authorization", Matcher::Exact(format!("Bearer {}", auth_token)))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                    "id": 161335,
                    "node_id": "MDg6V29ya2Zsb3cxNjEzMzU=",
                    "name": "CI",
                    "path": ".github/workflows/blank.yml",
                    "state": "active",
                    "created_at": "2020-01-08T23:48:37.000-08:00",
                    "updated_at": "2020-01-08T23:50:21.000-08:00",
                    "url": "https://api.github.com/repos/octo-org/octo-repo/actions/workflows/161335",
                    "html_url": "https://github.com/octo-org/octo-repo/blob/master/.github/workflows/161335",
                    "badge_url": "https://github.com/octo-org/octo-repo/workflows/CI/badge.svg"
                  }"#)
            .expect(1)
            .create();

        let expected_workflow = Workflow {
            id: 161335,
            node_id: "MDg6V29ya2Zsb3cxNjEzMzU=".into(),
            name: "CI".into(),
            path: ".github/workflows/blank.yml".into(),
            state: "active".into(),
            created_at: "2020-01-08T23:48:37.000-08:00".parse()?,
            updated_at: "2020-01-08T23:50:21.000-08:00".parse()?,
            url: "https://api.github.com/repos/octo-org/octo-repo/actions/workflows/161335".into(),
            html_url: "https://github.com/octo-org/octo-repo/blob/master/.github/workflows/161335"
                .into(),
            badge_url: "https://github.com/octo-org/octo-repo/workflows/CI/badge.svg".into(),
        };

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let workflows = repo_req.get_a_workflow(workflow_id).await?;

        m.assert();
        assert_eq!(workflows, expected_workflow);
        Ok(())
    }

    #[tokio::test]
    async fn get_workflow_usage() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";
        let workflow_id = 161335;

        let m = mock("GET", "/aslamplr/gh-cli/actions/workflows/161335/timing")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("Bearer {}", auth_token)),
            )
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "billable": {
                  "UBUNTU": {
                    "total_ms": 180000
                  },
                  "MACOS": {
                    "total_ms": 240000
                  },
                  "WINDOWS": {
                    "total_ms": 300000
                  }
                }
              }"#,
            )
            .expect(1)
            .create();

        let expected_usage = WorkflowUsage {
            billable: WorkflowUsagePlatform {
                UBUNTU: Some(WorkflowUsageTiming { total_ms: 180000 }),
                MACOS: Some(WorkflowUsageTiming { total_ms: 240000 }),
                WINDOWS: Some(WorkflowUsageTiming { total_ms: 300000 }),
            },
        };

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let usage = repo_req.get_workflow_usage(workflow_id).await?;

        m.assert();
        assert_eq!(usage, expected_usage);
        Ok(())
    }
}
