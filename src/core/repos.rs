#[derive(Debug)]
pub struct Repo<'a> {
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
}

#[derive(Debug)]
pub struct ReposRequestParams<'a>(pub Repo<'a>, pub &'a str);

impl<'a> ReposRequestParams<'a> {
    pub fn new(repo_owner: &'a str, repo_name: &'a str, auth_token: &'a str) -> Self {
        let repo = Repo {
            repo_owner: &repo_owner,
            repo_name: &repo_name,
        };
        ReposRequestParams(repo, &auth_token)
    }
}
