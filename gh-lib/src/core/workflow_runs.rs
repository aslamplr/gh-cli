use super::repos::RepoRequest;
use crate::utils::http::get;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::form_urlencoded::Serializer;

#[cfg(not(test))]
const BASE_URL: &str = "https://api.github.com/repos";

#[async_trait]
pub trait Workflows {
    async fn get_workflow_runs(&self, workflow_id: i32) -> Result<WorkflowRunList>;
    async fn get_workflow_runs_with_params(
        &self,
        workflow_id: i32,
        params: WorkflowRunQueryParams<'_>,
    ) -> Result<WorkflowRunList>;
}

#[async_trait]
impl Workflows for RepoRequest<'_> {
    async fn get_workflow_runs(&self, workflow_id: i32) -> Result<WorkflowRunList> {
        get_workflow_runs(&self, workflow_id, None).await
    }

    async fn get_workflow_runs_with_params(
        &self,
        workflow_id: i32,
        params: WorkflowRunQueryParams<'_>,
    ) -> Result<WorkflowRunList> {
        get_workflow_runs(&self, workflow_id, Some(&params)).await
    }
}

pub struct WorkflowRunQueryParams<'a> {
    pub actor: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub event: Option<&'a str>,
    pub status: Option<&'a str>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkflowRunList {
    pub total_count: i32,
    pub workflow_runs: Vec<WorkflowRun>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkflowRun {
    pub id: i32,
    pub node_id: String,
    pub head_branch: String,
    pub head_sha: String,
    pub run_number: i32,
    pub event: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub workflow_id: i32,
    pub url: String,
    pub html_url: String,
    pub pull_requests: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub jobs_url: String,
    pub logs_url: String,
    pub check_suite_url: String,
    pub artifacts_url: String,
    pub cancel_url: String,
    pub rerun_url: String,
    pub workflow_url: String,
    pub head_commit: Commit,
    pub repository: Repository,
    pub head_repository: Repository,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Commit {
    pub id: String,
    pub tree_id: String,
    pub message: String,
    pub timestamp: String,
    pub author: CommitUser,
    pub committer: CommitUser,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CommitUser {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    pub login: String,
    pub id: i32,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
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
    pub user_type: String,
    pub site_admin: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Repository {
    pub id: i32,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub owner: User,
    pub private: bool,
    pub html_url: String,
    pub description: Option<String>,
    pub fork: bool,
    pub url: String,
    pub archive_url: String,
    pub assignees_url: String,
    pub blobs_url: String,
    pub branches_url: String,
    pub collaborators_url: String,
    pub comments_url: String,
    pub commits_url: String,
    pub compare_url: String,
    pub contents_url: String,
    pub contributors_url: String,
    pub deployments_url: String,
    pub downloads_url: String,
    pub events_url: String,
    pub forks_url: String,
    pub git_commits_url: String,
    pub git_refs_url: String,
    pub git_tags_url: String,
    pub git_url: Option<String>,
    pub issue_comment_url: String,
    pub issue_events_url: String,
    pub issues_url: String,
    pub keys_url: String,
    pub labels_url: String,
    pub languages_url: String,
    pub merges_url: String,
    pub milestones_url: String,
    pub notifications_url: String,
    pub pulls_url: String,
    pub releases_url: String,
    pub ssh_url: Option<String>,
    pub stargazers_url: String,
    pub statuses_url: String,
    pub subscribers_url: String,
    pub subscription_url: String,
    pub tags_url: String,
    pub teams_url: String,
    pub trees_url: String,
    pub hooks_url: Option<String>,
}

async fn get_workflow_runs(
    params: &RepoRequest<'_>,
    workflow_id: i32,
    filter: Option<&WorkflowRunQueryParams<'_>>,
) -> Result<WorkflowRunList> {
    let RepoRequest(repo, auth_token) = params;
    let url = with_base_url!("{}/actions/workflows/{}/runs", repo, workflow_id);
    let url = if let Some(filter) = filter {
        let mut serializer = Serializer::new(String::new());
        if let Some(actor) = filter.actor {
            serializer.append_pair("actor", actor);
        }
        if let Some(branch) = filter.branch {
            serializer.append_pair("branch", branch);
        }
        if let Some(status) = filter.status {
            serializer.append_pair("status", status);
        }
        if let Some(event) = filter.event {
            serializer.append_pair("event", event);
        }
        format!("{}?{}", url, serializer.finish())
    } else {
        url
    };
    let resp = get(&url, &auth_token).await?;
    let resp = resp.deserialize().await?;
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    fn create_basic_mock_http(path: &str, auth_token: &str) -> mockito::Mock {
        mock("GET", path).match_header(
            "Authorization",
            Matcher::Exact(format!("bearer {}", auth_token)),
        )
    }

    fn extend_with_resp_http(mock: mockito::Mock) -> mockito::Mock {
        mock.with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                  "total_count": 1,
                  "workflow_runs": [
                    {
                      "id": 30433642,
                      "node_id": "MDEyOldvcmtmbG93IFJ1bjI2OTI4OQ==",
                      "head_branch": "master",
                      "head_sha": "acb5820ced9479c074f688cc328bf03f341a511d",
                      "run_number": 562,
                      "event": "push",
                      "status": "queued",
                      "conclusion": null,
                      "workflow_id": 159038,
                      "url": "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642",
                      "html_url": "https://github.com/octo-org/octo-repo/actions/runs/30433642",
                      "pull_requests": [
                
                      ],
                      "created_at": "2020-01-22T19:33:08Z",
                      "updated_at": "2020-01-22T19:33:08Z",
                      "jobs_url": "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642/jobs",
                      "logs_url": "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642/logs",
                      "check_suite_url": "https://api.github.com/repos/octo-org/octo-repo/check-suites/414944374",
                      "artifacts_url": "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642/artifacts",
                      "cancel_url": "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642/cancel",
                      "rerun_url": "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642/rerun",
                      "workflow_url": "https://api.github.com/repos/octo-org/octo-repo/actions/workflows/159038",
                      "head_commit": {
                        "id": "acb5820ced9479c074f688cc328bf03f341a511d",
                        "tree_id": "d23f6eedb1e1b9610bbc754ddb5197bfe7271223",
                        "message": "Create linter.yml",
                        "timestamp": "2020-01-22T19:33:05Z",
                        "author": {
                          "name": "Octo Cat",
                          "email": "octocat@github.com"
                        },
                        "committer": {
                          "name": "GitHub",
                          "email": "noreply@github.com"
                        }
                      },
                      "repository": {
                        "id": 1296269,
                        "node_id": "MDEwOlJlcG9zaXRvcnkxMjk2MjY5",
                        "name": "Hello-World",
                        "full_name": "octocat/Hello-World",
                        "owner": {
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
                        },
                        "private": false,
                        "html_url": "https://github.com/octocat/Hello-World",
                        "description": "This your first repo!",
                        "fork": false,
                        "url": "https://api.github.com/repos/octocat/Hello-World",
                        "archive_url": "http://api.github.com/repos/octocat/Hello-World/{archive_format}{/ref}",
                        "assignees_url": "http://api.github.com/repos/octocat/Hello-World/assignees{/user}",
                        "blobs_url": "http://api.github.com/repos/octocat/Hello-World/git/blobs{/sha}",
                        "branches_url": "http://api.github.com/repos/octocat/Hello-World/branches{/branch}",
                        "collaborators_url": "http://api.github.com/repos/octocat/Hello-World/collaborators{/collaborator}",
                        "comments_url": "http://api.github.com/repos/octocat/Hello-World/comments{/number}",
                        "commits_url": "http://api.github.com/repos/octocat/Hello-World/commits{/sha}",
                        "compare_url": "http://api.github.com/repos/octocat/Hello-World/compare/{base}...{head}",
                        "contents_url": "http://api.github.com/repos/octocat/Hello-World/contents/{+path}",
                        "contributors_url": "http://api.github.com/repos/octocat/Hello-World/contributors",
                        "deployments_url": "http://api.github.com/repos/octocat/Hello-World/deployments",
                        "downloads_url": "http://api.github.com/repos/octocat/Hello-World/downloads",
                        "events_url": "http://api.github.com/repos/octocat/Hello-World/events",
                        "forks_url": "http://api.github.com/repos/octocat/Hello-World/forks",
                        "git_commits_url": "http://api.github.com/repos/octocat/Hello-World/git/commits{/sha}",
                        "git_refs_url": "http://api.github.com/repos/octocat/Hello-World/git/refs{/sha}",
                        "git_tags_url": "http://api.github.com/repos/octocat/Hello-World/git/tags{/sha}",
                        "git_url": "git:github.com/octocat/Hello-World.git",
                        "issue_comment_url": "http://api.github.com/repos/octocat/Hello-World/issues/comments{/number}",
                        "issue_events_url": "http://api.github.com/repos/octocat/Hello-World/issues/events{/number}",
                        "issues_url": "http://api.github.com/repos/octocat/Hello-World/issues{/number}",
                        "keys_url": "http://api.github.com/repos/octocat/Hello-World/keys{/key_id}",
                        "labels_url": "http://api.github.com/repos/octocat/Hello-World/labels{/name}",
                        "languages_url": "http://api.github.com/repos/octocat/Hello-World/languages",
                        "merges_url": "http://api.github.com/repos/octocat/Hello-World/merges",
                        "milestones_url": "http://api.github.com/repos/octocat/Hello-World/milestones{/number}",
                        "notifications_url": "http://api.github.com/repos/octocat/Hello-World/notifications{?since,all,participating}",
                        "pulls_url": "http://api.github.com/repos/octocat/Hello-World/pulls{/number}",
                        "releases_url": "http://api.github.com/repos/octocat/Hello-World/releases{/id}",
                        "ssh_url": "git@github.com:octocat/Hello-World.git",
                        "stargazers_url": "http://api.github.com/repos/octocat/Hello-World/stargazers",
                        "statuses_url": "http://api.github.com/repos/octocat/Hello-World/statuses/{sha}",
                        "subscribers_url": "http://api.github.com/repos/octocat/Hello-World/subscribers",
                        "subscription_url": "http://api.github.com/repos/octocat/Hello-World/subscription",
                        "tags_url": "http://api.github.com/repos/octocat/Hello-World/tags",
                        "teams_url": "http://api.github.com/repos/octocat/Hello-World/teams",
                        "trees_url": "http://api.github.com/repos/octocat/Hello-World/git/trees{/sha}"
                      },
                      "head_repository": {
                        "id": 217723378,
                        "node_id": "MDEwOlJlcG9zaXRvcnkyMTc3MjMzNzg=",
                        "name": "octo-repo",
                        "full_name": "octo-org/octo-repo",
                        "private": true,
                        "owner": {
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
                        },
                        "html_url": "https://github.com/octo-org/octo-repo",
                        "description": null,
                        "fork": false,
                        "url": "https://api.github.com/repos/octo-org/octo-repo",
                        "forks_url": "https://api.github.com/repos/octo-org/octo-repo/forks",
                        "keys_url": "https://api.github.com/repos/octo-org/octo-repo/keys{/key_id}",
                        "collaborators_url": "https://api.github.com/repos/octo-org/octo-repo/collaborators{/collaborator}",
                        "teams_url": "https://api.github.com/repos/octo-org/octo-repo/teams",
                        "hooks_url": "https://api.github.com/repos/octo-org/octo-repo/hooks",
                        "issue_events_url": "https://api.github.com/repos/octo-org/octo-repo/issues/events{/number}",
                        "events_url": "https://api.github.com/repos/octo-org/octo-repo/events",
                        "assignees_url": "https://api.github.com/repos/octo-org/octo-repo/assignees{/user}",
                        "branches_url": "https://api.github.com/repos/octo-org/octo-repo/branches{/branch}",
                        "tags_url": "https://api.github.com/repos/octo-org/octo-repo/tags",
                        "blobs_url": "https://api.github.com/repos/octo-org/octo-repo/git/blobs{/sha}",
                        "git_tags_url": "https://api.github.com/repos/octo-org/octo-repo/git/tags{/sha}",
                        "git_refs_url": "https://api.github.com/repos/octo-org/octo-repo/git/refs{/sha}",
                        "trees_url": "https://api.github.com/repos/octo-org/octo-repo/git/trees{/sha}",
                        "statuses_url": "https://api.github.com/repos/octo-org/octo-repo/statuses/{sha}",
                        "languages_url": "https://api.github.com/repos/octo-org/octo-repo/languages",
                        "stargazers_url": "https://api.github.com/repos/octo-org/octo-repo/stargazers",
                        "contributors_url": "https://api.github.com/repos/octo-org/octo-repo/contributors",
                        "subscribers_url": "https://api.github.com/repos/octo-org/octo-repo/subscribers",
                        "subscription_url": "https://api.github.com/repos/octo-org/octo-repo/subscription",
                        "commits_url": "https://api.github.com/repos/octo-org/octo-repo/commits{/sha}",
                        "git_commits_url": "https://api.github.com/repos/octo-org/octo-repo/git/commits{/sha}",
                        "comments_url": "https://api.github.com/repos/octo-org/octo-repo/comments{/number}",
                        "issue_comment_url": "https://api.github.com/repos/octo-org/octo-repo/issues/comments{/number}",
                        "contents_url": "https://api.github.com/repos/octo-org/octo-repo/contents/{+path}",
                        "compare_url": "https://api.github.com/repos/octo-org/octo-repo/compare/{base}...{head}",
                        "merges_url": "https://api.github.com/repos/octo-org/octo-repo/merges",
                        "archive_url": "https://api.github.com/repos/octo-org/octo-repo/{archive_format}{/ref}",
                        "downloads_url": "https://api.github.com/repos/octo-org/octo-repo/downloads",
                        "issues_url": "https://api.github.com/repos/octo-org/octo-repo/issues{/number}",
                        "pulls_url": "https://api.github.com/repos/octo-org/octo-repo/pulls{/number}",
                        "milestones_url": "https://api.github.com/repos/octo-org/octo-repo/milestones{/number}",
                        "notifications_url": "https://api.github.com/repos/octo-org/octo-repo/notifications{?since,all,participating}",
                        "labels_url": "https://api.github.com/repos/octo-org/octo-repo/labels{/name}",
                        "releases_url": "https://api.github.com/repos/octo-org/octo-repo/releases{/id}",
                        "deployments_url": "https://api.github.com/repos/octo-org/octo-repo/deployments"
                      }
                    }
                  ]
                }"#,
            )
            .expect(1)
            .create()
    }

    fn create_mock_http(path: &str, auth_token: &str) -> mockito::Mock {
        let mock = create_basic_mock_http(path, auth_token);
        let mock = extend_with_resp_http(mock);
        mock
    }

    fn create_mock_http_check_param(path: &str, auth_token: &str) -> mockito::Mock {
        let mock = create_basic_mock_http(path, auth_token);
        let mock = mock.match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("actor".into(), "aslamplr".into()),
            Matcher::UrlEncoded("branch".into(), "master".into()),
            Matcher::UrlEncoded("event".into(), "push".into()),
            Matcher::UrlEncoded("status".into(), "success".into()),
        ]));
        let mock = extend_with_resp_http(mock);
        mock
    }

    fn create_expected_run_list() -> Result<WorkflowRunList> {
        Ok(WorkflowRunList {
        total_count: 1,
        workflow_runs: vec![
          WorkflowRun {
            id: 30433642,
            node_id: "MDEyOldvcmtmbG93IFJ1bjI2OTI4OQ==".into(),
            head_branch: "master".into(),
            head_sha: "acb5820ced9479c074f688cc328bf03f341a511d".into(),
            run_number: 562,
            event: "push".into(),
            status: "queued".into(),
            conclusion: None,
            workflow_id: 159038,
            url: "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642".into(),
            html_url: "https://github.com/octo-org/octo-repo/actions/runs/30433642".into(),
            pull_requests: vec![],
            created_at: "2020-01-22T19:33:08Z".parse()?,
            updated_at: "2020-01-22T19:33:08Z".parse()?,
            jobs_url: "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642/jobs".into(),
            logs_url: "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642/logs".into(),
            check_suite_url: "https://api.github.com/repos/octo-org/octo-repo/check-suites/414944374".into(),
            artifacts_url: "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642/artifacts".into(),
            cancel_url: "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642/cancel".into(),
            rerun_url: "https://api.github.com/repos/octo-org/octo-repo/actions/runs/30433642/rerun".into(),
            workflow_url: "https://api.github.com/repos/octo-org/octo-repo/actions/workflows/159038".into(),
            head_commit: Commit {
              id: "acb5820ced9479c074f688cc328bf03f341a511d".into(),
              tree_id: "d23f6eedb1e1b9610bbc754ddb5197bfe7271223".into(),
              message: "Create linter.yml".into(),
              timestamp: "2020-01-22T19:33:05Z".parse()?,
              author: CommitUser {
                name: "Octo Cat".into(),
                email: "octocat@github.com".into()
              },
              committer: CommitUser {
                name: "GitHub".into(),
                email: "noreply@github.com".into()
              }
            },
            repository: Repository {
              id: 1296269,
              node_id: "MDEwOlJlcG9zaXRvcnkxMjk2MjY5".into(),
              name: "Hello-World".into(),
              full_name: "octocat/Hello-World".into(),
              owner: User {
                login: "octocat".into(),
                id: 1,
                node_id: "MDQ6VXNlcjE=".into(),
                avatar_url: "https://github.com/images/error/octocat_happy.gif".into(),
                gravatar_id: "".into(),
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
                user_type: "User".into(),
                site_admin: false
              },
              private: false,
              html_url: "https://github.com/octocat/Hello-World".into(),
              description: Some("This your first repo!".into()),
              fork: false,
              url: "https://api.github.com/repos/octocat/Hello-World".into(),
              archive_url: "http://api.github.com/repos/octocat/Hello-World/{archive_format}{/ref}".into(),
              assignees_url: "http://api.github.com/repos/octocat/Hello-World/assignees{/user}".into(),
              blobs_url: "http://api.github.com/repos/octocat/Hello-World/git/blobs{/sha}".into(),
              branches_url: "http://api.github.com/repos/octocat/Hello-World/branches{/branch}".into(),
              collaborators_url: "http://api.github.com/repos/octocat/Hello-World/collaborators{/collaborator}".into(),
              comments_url: "http://api.github.com/repos/octocat/Hello-World/comments{/number}".into(),
              commits_url: "http://api.github.com/repos/octocat/Hello-World/commits{/sha}".into(),
              compare_url: "http://api.github.com/repos/octocat/Hello-World/compare/{base}...{head}".into(),
              contents_url: "http://api.github.com/repos/octocat/Hello-World/contents/{+path}".into(),
              contributors_url: "http://api.github.com/repos/octocat/Hello-World/contributors".into(),
              deployments_url: "http://api.github.com/repos/octocat/Hello-World/deployments".into(),
              downloads_url: "http://api.github.com/repos/octocat/Hello-World/downloads".into(),
              events_url: "http://api.github.com/repos/octocat/Hello-World/events".into(),
              forks_url: "http://api.github.com/repos/octocat/Hello-World/forks".into(),
              git_commits_url: "http://api.github.com/repos/octocat/Hello-World/git/commits{/sha}".into(),
              git_refs_url: "http://api.github.com/repos/octocat/Hello-World/git/refs{/sha}".into(),
              git_tags_url: "http://api.github.com/repos/octocat/Hello-World/git/tags{/sha}".into(),
              git_url: Some("git:github.com/octocat/Hello-World.git".into()),
              issue_comment_url: "http://api.github.com/repos/octocat/Hello-World/issues/comments{/number}".into(),
              issue_events_url: "http://api.github.com/repos/octocat/Hello-World/issues/events{/number}".into(),
              issues_url: "http://api.github.com/repos/octocat/Hello-World/issues{/number}".into(),
              keys_url: "http://api.github.com/repos/octocat/Hello-World/keys{/key_id}".into(),
              labels_url: "http://api.github.com/repos/octocat/Hello-World/labels{/name}".into(),
              languages_url: "http://api.github.com/repos/octocat/Hello-World/languages".into(),
              merges_url: "http://api.github.com/repos/octocat/Hello-World/merges".into(),
              milestones_url: "http://api.github.com/repos/octocat/Hello-World/milestones{/number}".into(),
              notifications_url: "http://api.github.com/repos/octocat/Hello-World/notifications{?since,all,participating}".into(),
              pulls_url: "http://api.github.com/repos/octocat/Hello-World/pulls{/number}".into(),
              releases_url: "http://api.github.com/repos/octocat/Hello-World/releases{/id}".into(),
              ssh_url: Some("git@github.com:octocat/Hello-World.git".into()),
              stargazers_url: "http://api.github.com/repos/octocat/Hello-World/stargazers".into(),
              statuses_url: "http://api.github.com/repos/octocat/Hello-World/statuses/{sha}".into(),
              subscribers_url: "http://api.github.com/repos/octocat/Hello-World/subscribers".into(),
              subscription_url: "http://api.github.com/repos/octocat/Hello-World/subscription".into(),
              tags_url: "http://api.github.com/repos/octocat/Hello-World/tags".into(),
              teams_url: "http://api.github.com/repos/octocat/Hello-World/teams".into(),
              trees_url: "http://api.github.com/repos/octocat/Hello-World/git/trees{/sha}".into(),
              hooks_url: None
            },
            head_repository: Repository {
              id: 217723378,
              node_id: "MDEwOlJlcG9zaXRvcnkyMTc3MjMzNzg=".into(),
                name: "octo-repo".into(),
                full_name: "octo-org/octo-repo".into(),
                private: true,
                owner: User {
                  login: "octocat".into(),
                  id: 1,
                  node_id: "MDQ6VXNlcjE=".into(),
                  avatar_url: "https://github.com/images/error/octocat_happy.gif".into(),
                  gravatar_id: "".into(),
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
                  user_type: "User".into(),
                  site_admin: false
                },
                html_url: "https://github.com/octo-org/octo-repo".into(),
                description: None,
                fork: false,
                url: "https://api.github.com/repos/octo-org/octo-repo".into(),
                forks_url: "https://api.github.com/repos/octo-org/octo-repo/forks".into(),
                keys_url: "https://api.github.com/repos/octo-org/octo-repo/keys{/key_id}".into(),
                collaborators_url: "https://api.github.com/repos/octo-org/octo-repo/collaborators{/collaborator}".into(),
                teams_url: "https://api.github.com/repos/octo-org/octo-repo/teams".into(),
                hooks_url: Some("https://api.github.com/repos/octo-org/octo-repo/hooks".into()),
                issue_events_url: "https://api.github.com/repos/octo-org/octo-repo/issues/events{/number}".into(),
                events_url: "https://api.github.com/repos/octo-org/octo-repo/events".into(),
                assignees_url: "https://api.github.com/repos/octo-org/octo-repo/assignees{/user}".into(),
                branches_url: "https://api.github.com/repos/octo-org/octo-repo/branches{/branch}".into(),
                tags_url: "https://api.github.com/repos/octo-org/octo-repo/tags".into(),
                blobs_url: "https://api.github.com/repos/octo-org/octo-repo/git/blobs{/sha}".into(),
                git_tags_url: "https://api.github.com/repos/octo-org/octo-repo/git/tags{/sha}".into(),
                git_refs_url: "https://api.github.com/repos/octo-org/octo-repo/git/refs{/sha}".into(),
                trees_url: "https://api.github.com/repos/octo-org/octo-repo/git/trees{/sha}".into(),
                statuses_url: "https://api.github.com/repos/octo-org/octo-repo/statuses/{sha}".into(),
                languages_url: "https://api.github.com/repos/octo-org/octo-repo/languages".into(),
                stargazers_url: "https://api.github.com/repos/octo-org/octo-repo/stargazers".into(),
                contributors_url: "https://api.github.com/repos/octo-org/octo-repo/contributors".into(),
                subscribers_url: "https://api.github.com/repos/octo-org/octo-repo/subscribers".into(),
                subscription_url: "https://api.github.com/repos/octo-org/octo-repo/subscription".into(),
                commits_url: "https://api.github.com/repos/octo-org/octo-repo/commits{/sha}".into(),
                git_commits_url: "https://api.github.com/repos/octo-org/octo-repo/git/commits{/sha}".into(),
                comments_url: "https://api.github.com/repos/octo-org/octo-repo/comments{/number}".into(),
                issue_comment_url: "https://api.github.com/repos/octo-org/octo-repo/issues/comments{/number}".into(),
                contents_url: "https://api.github.com/repos/octo-org/octo-repo/contents/{+path}".into(),
                compare_url: "https://api.github.com/repos/octo-org/octo-repo/compare/{base}...{head}".into(),
                merges_url: "https://api.github.com/repos/octo-org/octo-repo/merges".into(),
                archive_url: "https://api.github.com/repos/octo-org/octo-repo/{archive_format}{/ref}".into(),
                downloads_url: "https://api.github.com/repos/octo-org/octo-repo/downloads".into(),
                issues_url: "https://api.github.com/repos/octo-org/octo-repo/issues{/number}".into(),
                pulls_url: "https://api.github.com/repos/octo-org/octo-repo/pulls{/number}".into(),
                milestones_url: "https://api.github.com/repos/octo-org/octo-repo/milestones{/number}".into(),
                notifications_url: "https://api.github.com/repos/octo-org/octo-repo/notifications{?since,all,participating}".into(),
                labels_url: "https://api.github.com/repos/octo-org/octo-repo/labels{/name}".into(),
                releases_url: "https://api.github.com/repos/octo-org/octo-repo/releases{/id}".into(),
                deployments_url: "https://api.github.com/repos/octo-org/octo-repo/deployments".into(),
                ssh_url: None,
                git_url: None
            }
          }
        ]
      })
    }

    #[tokio::test]
    async fn get_workflow_runs() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = create_mock_http(
            "/aslamplr/gh-cli/actions/workflows/30433642/runs",
            auth_token,
        );

        let expected_run_list = create_expected_run_list()?;

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let run_list = repo_req.get_workflow_runs(30433642).await?;

        m.assert();
        assert_eq!(run_list, expected_run_list);
        Ok(())
    }

    #[tokio::test]
    async fn get_workflow_runs_with_params() -> Result<()> {
        let repo_addr = "aslamplr/gh-cli";
        let auth_token = "auth_secret_token";

        let m = create_mock_http_check_param(
            "/aslamplr/gh-cli/actions/workflows/30433642/runs",
            auth_token,
        );

        let expected_run_list = create_expected_run_list()?;

        let repo_req = RepoRequest::try_from(repo_addr, auth_token)?;
        let run_list = repo_req
            .get_workflow_runs_with_params(
                30433642,
                WorkflowRunQueryParams {
                    actor: Some("aslamplr"),
                    branch: Some("master"),
                    event: Some("push"),
                    status: Some("success"),
                },
            )
            .await?;

        m.assert();
        assert_eq!(run_list, expected_run_list);
        Ok(())
    }
}
