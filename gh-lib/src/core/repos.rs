use crate::utils::http::HttpClient;
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Repo<'a> {
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
}

#[derive(Debug)]
pub struct RepoRequest<'a> {
    pub repo: Repo<'a>,
    pub auth_token: &'a str,
    pub http_client: HttpClient,
}

impl<'a> RepoRequest<'a> {
    pub fn try_from(repo_addr: &'a str, auth_token: &'a str) -> Result<Self> {
        let slash_idx = repo_addr
            .find('/')
            .ok_or_else(|| anyhow!("Unable to parse repo_name from: {}", repo_addr))?;
        let (repo_owner, repo_name) = repo_addr.split_at(slash_idx);
        let repo = Repo {
            repo_owner: &repo_owner,
            repo_name: &repo_name[1..],
        };
        let http_client = HttpClient::new()?;
        Ok(RepoRequest {
            repo,
            auth_token,
            http_client,
        })
    }
}

impl std::fmt::Display for Repo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.repo_owner, self.repo_name)
    }
}
