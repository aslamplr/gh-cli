use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=gh-web=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "gh-web=info");
    }
    pretty_env_logger::init();

    let api = filters::repo();

    let routes = api
        .with(warp::log("gh-web"))
        .recover(problem::unpack_problem);

    let port = env::var_os("PORT")
        .map(|p| p.to_str().unwrap().trim().parse::<u16>().unwrap())
        .unwrap_or(3030u16);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

mod filters {
    use super::handlers;
    use warp::{self, Filter, Rejection, Reply};

    pub fn repo() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        get_repo_basic_info()
            .or(get_repo_secrets())
            .or(save_repo_secrets())
    }

    // GET /repo/:repo_owner/:repo_name
    pub fn get_repo_basic_info() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("repo" / String / String)
            .and(warp::get())
            .and(access_token_header())
            .and_then(handlers::get_repo_basic_info)
    }

    // GET /repo/:repo_owner/:repo_name/secrets
    pub fn get_repo_secrets() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("repo" / String / String / "secrets")
            .and(warp::get())
            .and(access_token_header())
            .and_then(handlers::get_repo_secrets)
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct SecretInput {
        key: String,
        value: String,
    }

    // POST /repo/:repo_owner/:repo_name/secrets
    pub fn save_repo_secrets() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("repo" / String / String / "secrets")
            .and(warp::post())
            .and(access_token_header())
            .and(warp::body::json())
            .map(|owner, name, auth_token, secret: SecretInput| {
                (owner, name, auth_token, secret.key, secret.value)
            })
            .untuple_one()
            .and_then(handlers::save_repo_secrets)
    }

    pub fn access_token_header() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
        warp::header::<String>("Access-Token")
    }
}

mod handlers {
    use warp::{self, Rejection, Reply};

    pub async fn get_repo_basic_info(
        repo_owner: String,
        repo_name: String,
        auth_token: String,
    ) -> Result<impl Reply, Rejection> {
        super::services::get_repo_basic_info(&format!("{}/{}", repo_owner, repo_name), &auth_token)
            .await
            .map(|r| warp::reply::json(&r))
            .map_err(super::problem::from_anyhow)
            .map_err(warp::reject::custom)
    }

    pub async fn get_repo_secrets(
        repo_owner: String,
        repo_name: String,
        auth_token: String,
    ) -> Result<impl Reply, Rejection> {
        super::services::get_repo_secrets(&format!("{}/{}", repo_owner, repo_name), &auth_token)
            .await
            .map(|r| warp::reply::json(&r))
            .map_err(super::problem::from_anyhow)
            .map_err(warp::reject::custom)
    }

    pub async fn save_repo_secrets(
        repo_owner: String,
        repo_name: String,
        auth_token: String,
        secret_key: String,
        secret_value: String,
    ) -> Result<impl Reply, Rejection> {
        super::services::save_repo_secrets(
            &format!("{}/{}", repo_owner, repo_name),
            &auth_token,
            &secret_key,
            &secret_value,
        )
        .await
        .map(|_| warp::reply())
        .map_err(super::problem::from_anyhow)
        .map_err(warp::reject::custom)
    }
}

mod services {
    use anyhow::{anyhow, Result};
    use gh_lib::core::{
        basic_info::{basic_info_response::RepoBasicInfoQueryRepository, BasicInfo as _},
        repos::RepoRequest,
        secrets::{SecretListResponse, Secrets as _},
    };

    pub async fn get_repo_basic_info(
        repo_addr: &str,
        auth_token: &str,
    ) -> Result<RepoBasicInfoQueryRepository> {
        let repo_request = RepoRequest::try_from(&repo_addr, &auth_token)?;
        let basic_info = repo_request.get_basic_info().await?;
        basic_info
            .repository
            .ok_or_else(|| anyhow!("Repository not found!"))
    }

    pub async fn get_repo_secrets(repo_addr: &str, auth_token: &str) -> Result<SecretListResponse> {
        let repo_request = RepoRequest::try_from(&repo_addr, &auth_token)?;
        repo_request.get_all_secrets().await
    }

    pub async fn save_repo_secrets(
        repo_addr: &str,
        auth_token: &str,
        secret_key: &str,
        secret_value: &str,
    ) -> Result<()> {
        let repo_request = RepoRequest::try_from(&repo_addr, &auth_token)?;
        repo_request.save_secret(secret_key, secret_value).await
    }
}

mod problem {
    use http_api_problem::HttpApiProblem;
    use warp::{
        self,
        http::{self, StatusCode},
        Rejection, Reply,
    };

    pub fn from_anyhow(e: anyhow::Error) -> HttpApiProblem {
        let e = match e.downcast::<HttpApiProblem>() {
            Ok(problem) => return problem,
            Err(e) => e,
        };
        HttpApiProblem::new(format!("Internal Server Error\n{:?}", e))
            .set_status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn unpack_problem(rejection: Rejection) -> Result<impl Reply, Rejection> {
        if let Some(problem) = rejection.find::<HttpApiProblem>() {
            let code = problem.status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            let reply = warp::reply::json(problem);
            let reply = warp::reply::with_status(reply, code);
            let reply = warp::reply::with_header(
                reply,
                http::header::CONTENT_TYPE,
                http_api_problem::PROBLEM_JSON_MEDIA_TYPE,
            );

            return Ok(reply);
        }

        Err(rejection)
    }
}
