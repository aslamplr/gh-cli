use super::repos::RepoRequest;
use crate::utils::http::get;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(not(test))]
const BASE_URL: &str = "https://api.github.com/repos";

#[async_trait]
pub trait Workflows {
    async fn get_workflow_run_jobs(&self, run_id: i32) -> Result<WorkflowRunJobList>;
    async fn get_a_workflow_run_job(&self, job_id: i32) -> Result<WorkflowRunJob>;
    async fn get_job_logs_url(&self, job_id: i32) -> Result<String>;
}

#[async_trait]
impl Workflows for RepoRequest<'_> {
    async fn get_workflow_run_jobs(&self, run_id: i32) -> Result<WorkflowRunJobList> {
        get_workflow_run_jobs(&self, run_id).await
    }

    async fn get_a_workflow_run_job(&self, job_id: i32) -> Result<WorkflowRunJob> {
        get_a_workflow_run_job(&self, job_id).await
    }

    async fn get_job_logs_url(&self, job_id: i32) -> Result<String> {
        get_job_logs_url(&self, job_id).await
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkflowRunJobList {
    pub total_count: i32,
    pub jobs: Vec<WorkflowRunJob>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkflowRunJob {
    pub id: i32,
    pub run_id: i32,
    pub run_url: String,
    pub node_id: String,
    pub head_sha: String,
    pub url: String,
    pub html_url: String,
    pub status: String,
    pub conclusion: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub name: String,
    pub steps: Vec<WorkflowRunJobStep>,
    pub check_run_url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkflowRunJobStep {
    pub name: String,
    pub status: String,
    pub conclusion: String,
    pub number: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

async fn get_workflow_run_jobs(
    params: &RepoRequest<'_>,
    run_id: i32,
) -> Result<WorkflowRunJobList> {
    let RepoRequest(repo, auth_token) = params;
    let url = with_base_url!("{}/actions/runs/{}/jobs", repo, run_id);
    let resp = get(&url, &auth_token).await?;
    let resp = resp.deserialize().await?;
    Ok(resp)
}

async fn get_a_workflow_run_job(params: &RepoRequest<'_>, job_id: i32) -> Result<WorkflowRunJob> {
    let RepoRequest(repo, auth_token) = params;
    let url = with_base_url!("{}/actions/jobs/{}", repo, job_id);
    let resp = get(&url, &auth_token).await?;
    let resp = resp.deserialize().await?;
    Ok(resp)
}

async fn get_job_logs_url(params: &RepoRequest<'_>, job_id: i32) -> Result<String> {
    let RepoRequest(repo, auth_token) = params;
    let url = with_base_url!("{}/actions/jobs/{}/logs", repo, job_id);
    let resp = get(&url, &auth_token).await?;
    let resp = resp.get_header("Location");
    resp.ok_or_else(|| anyhow::anyhow!("Location header with log url not found in response!"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn get_workflow_run_jobs() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/actions/runs/29679449/jobs")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("bearer {}", auth_token)),
            )
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "total_count": 1,
                "jobs": [
                  {
                    "id": 399444496,
                    "run_id": 29679449,
                    "run_url": "https://api.github.com/repos/octo-org/octo-repo/actions/runs/29679449",
                    "node_id": "MDEyOldvcmtmbG93IEpvYjM5OTQ0NDQ5Ng==",
                    "head_sha": "f83a356604ae3c5d03e1b46ef4d1ca77d64a90b0",
                    "url": "https://api.github.com/repos/octo-org/octo-repo/actions/jobs/399444496",
                    "html_url": "https://github.com/octo-org/octo-repo/runs/399444496",
                    "status": "completed",
                    "conclusion": "success",
                    "started_at": "2020-01-20T17:42:40Z",
                    "completed_at": "2020-01-20T17:44:39Z",
                    "name": "build",
                    "steps": [
                      {
                        "name": "Set up job",
                        "status": "completed",
                        "conclusion": "success",
                        "number": 1,
                        "started_at": "2020-01-20T09:42:40.000-08:00",
                        "completed_at": "2020-01-20T09:42:41.000-08:00"
                      },
                      {
                        "name": "Run actions/checkout@v2",
                        "status": "completed",
                        "conclusion": "success",
                        "number": 2,
                        "started_at": "2020-01-20T09:42:41.000-08:00",
                        "completed_at": "2020-01-20T09:42:45.000-08:00"
                      }
                    ],
                    "check_run_url": "https://api.github.com/repos/octo-org/octo-repo/check-runs/399444496"
                  }
                ]
              }"#)
            .expect(1)
            .create();

        let expected_job_list = WorkflowRunJobList {
            total_count: 1,
            jobs: vec![WorkflowRunJob {
                id: 399444496,
                run_id: 29679449,
                run_url: "https://api.github.com/repos/octo-org/octo-repo/actions/runs/29679449"
                    .into(),
                node_id: "MDEyOldvcmtmbG93IEpvYjM5OTQ0NDQ5Ng==".into(),
                head_sha: "f83a356604ae3c5d03e1b46ef4d1ca77d64a90b0".into(),
                url: "https://api.github.com/repos/octo-org/octo-repo/actions/jobs/399444496"
                    .into(),
                html_url: "https://github.com/octo-org/octo-repo/runs/399444496".into(),
                status: "completed".into(),
                conclusion: "success".into(),
                started_at: "2020-01-20T17:42:40Z".parse()?,
                completed_at: "2020-01-20T17:44:39Z".parse()?,
                name: "build".into(),
                steps: vec![
                    WorkflowRunJobStep {
                        name: "Set up job".into(),
                        status: "completed".into(),
                        conclusion: "success".into(),
                        number: 1,
                        started_at: "2020-01-20T09:42:40.000-08:00".parse()?,
                        completed_at: "2020-01-20T09:42:41.000-08:00".parse()?,
                    },
                    WorkflowRunJobStep {
                        name: "Run actions/checkout@v2".into(),
                        status: "completed".into(),
                        conclusion: "success".into(),
                        number: 2,
                        started_at: "2020-01-20T09:42:41.000-08:00".parse()?,
                        completed_at: "2020-01-20T09:42:45.000-08:00".parse()?,
                    },
                ],
                check_run_url:
                    "https://api.github.com/repos/octo-org/octo-repo/check-runs/399444496".into(),
            }],
        };

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let job_list = repo_req.get_workflow_run_jobs(29679449).await?;

        m.assert();
        assert_eq!(job_list, expected_job_list);
        Ok(())
    }

    #[tokio::test]
    async fn get_a_workflow_run_job() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/actions/jobs/399444496")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("bearer {}", auth_token)),
            )
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                    "id": 399444496,
                    "run_id": 29679449,
                    "run_url": "https://api.github.com/repos/octo-org/octo-repo/actions/runs/29679449",
                    "node_id": "MDEyOldvcmtmbG93IEpvYjM5OTQ0NDQ5Ng==",
                    "head_sha": "f83a356604ae3c5d03e1b46ef4d1ca77d64a90b0",
                    "url": "https://api.github.com/repos/octo-org/octo-repo/actions/jobs/399444496",
                    "html_url": "https://github.com/octo-org/octo-repo/runs/399444496",
                    "status": "completed",
                    "conclusion": "success",
                    "started_at": "2020-01-20T17:42:40Z",
                    "completed_at": "2020-01-20T17:44:39Z",
                    "name": "build",
                    "steps": [
                      {
                        "name": "Set up job",
                        "status": "completed",
                        "conclusion": "success",
                        "number": 1,
                        "started_at": "2020-01-20T09:42:40.000-08:00",
                        "completed_at": "2020-01-20T09:42:41.000-08:00"
                      },
                      {
                        "name": "Run actions/checkout@v2",
                        "status": "completed",
                        "conclusion": "success",
                        "number": 2,
                        "started_at": "2020-01-20T09:42:41.000-08:00",
                        "completed_at": "2020-01-20T09:42:45.000-08:00"
                      }
                    ],
                    "check_run_url": "https://api.github.com/repos/octo-org/octo-repo/check-runs/399444496"
                  }"#)
            .expect(1)
            .create();

        let expected_job = WorkflowRunJob {
            id: 399444496,
            run_id: 29679449,
            run_url: "https://api.github.com/repos/octo-org/octo-repo/actions/runs/29679449".into(),
            node_id: "MDEyOldvcmtmbG93IEpvYjM5OTQ0NDQ5Ng==".into(),
            head_sha: "f83a356604ae3c5d03e1b46ef4d1ca77d64a90b0".into(),
            url: "https://api.github.com/repos/octo-org/octo-repo/actions/jobs/399444496".into(),
            html_url: "https://github.com/octo-org/octo-repo/runs/399444496".into(),
            status: "completed".into(),
            conclusion: "success".into(),
            started_at: "2020-01-20T17:42:40Z".parse()?,
            completed_at: "2020-01-20T17:44:39Z".parse()?,
            name: "build".into(),
            steps: vec![
                WorkflowRunJobStep {
                    name: "Set up job".into(),
                    status: "completed".into(),
                    conclusion: "success".into(),
                    number: 1,
                    started_at: "2020-01-20T09:42:40.000-08:00".parse()?,
                    completed_at: "2020-01-20T09:42:41.000-08:00".parse()?,
                },
                WorkflowRunJobStep {
                    name: "Run actions/checkout@v2".into(),
                    status: "completed".into(),
                    conclusion: "success".into(),
                    number: 2,
                    started_at: "2020-01-20T09:42:41.000-08:00".parse()?,
                    completed_at: "2020-01-20T09:42:45.000-08:00".parse()?,
                },
            ],
            check_run_url: "https://api.github.com/repos/octo-org/octo-repo/check-runs/399444496"
                .into(),
        };

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let job = repo_req.get_a_workflow_run_job(399444496).await?;

        m.assert();
        assert_eq!(job, expected_job);
        Ok(())
    }

    #[tokio::test]
    async fn get_job_logs_url() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = mock("GET", "/aslamplr/gh-cli/actions/jobs/399444496/logs")
            .match_header(
                "Authorization",
                Matcher::Exact(format!("bearer {}", auth_token)),
            )
            .with_status(302)
            .with_header("Location", "https://pipelines.actions.githubusercontent.com/ab1f3cCFPB34Nd6imvFxpGZH5hNlDp2wijMwl2gDoO0bcrrlJj/_apis/pipelines/1/jobs/19/signedlogcontent?urlExpires=2020-01-22T22%3A44%3A54.1389777Z&urlSigningMethod=HMACV1&urlSignature=2TUDfIg4fm36OJmfPy6km5QD5DLCOkBVzvhWZM8B%2BUY%3D")
            .expect(1)
            .create();

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let logs_url = repo_req.get_job_logs_url(399444496).await?;

        m.assert();
        assert_eq!(logs_url, "https://pipelines.actions.githubusercontent.com/ab1f3cCFPB34Nd6imvFxpGZH5hNlDp2wijMwl2gDoO0bcrrlJj/_apis/pipelines/1/jobs/19/signedlogcontent?urlExpires=2020-01-22T22%3A44%3A54.1389777Z&urlSigningMethod=HMACV1&urlSignature=2TUDfIg4fm36OJmfPy6km5QD5DLCOkBVzvhWZM8B%2BUY%3D".to_string());
        Ok(())
    }
}
