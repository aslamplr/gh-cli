#[derive(Debug)]
pub struct Repo<'a> {
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
}

#[derive(Debug)]
pub struct RepoRequest<'a>(pub Repo<'a>, pub &'a str);

impl<'a> RepoRequest<'a> {
    pub fn new(repo_owner: &'a str, repo_name: &'a str, auth_token: &'a str) -> Self {
        let repo = Repo {
            repo_owner: &repo_owner,
            repo_name: &repo_name,
        };
        RepoRequest(repo, &auth_token)
    }
}
