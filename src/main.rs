use base64;
use bytes::buf::BufExt;
use crypto_box::{aead::Aead, Box, PublicKey, SecretKey};
use hyper::{Client, Request};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, std::boxed::Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
  let matches = clap::App::new("Github Actions Secret - CLI (Unofficial)")
    .version("0.1.0")
    .author("Aslam Ahammed A. <aslamplr@gmail.com>")
    .about("Deal with Github actions secrets")
    .arg(
      clap::Arg::with_name("repo_owner")
        .long("repo_owner")
        .short("o")
        .value_name("REPO_OWNER")
        .help("Repository owner")
        .display_order(1)
        .takes_value(true)
        .required(true),
    )
    .arg(
      clap::Arg::with_name("repo_name")
        .long("repo_name")
        .short("n")
        .value_name("REPO_NAME")
        .help("Repository name")
        .display_order(2)
        .takes_value(true)
        .required(true),
    )
    .arg(
      clap::Arg::with_name("auth_token")
        .long("auth_token")
        .short("t")
        .value_name("PERSONAL_ACCESS_TOKEN")
        .help("Generate token - https://github.com/settings/tokens")
        .display_order(3)
        .takes_value(true)
        .required(true),
    )
    .arg(
      clap::Arg::with_name("action")
        .long("action")
        .short("a")
        .value_name("ACTION")
        .display_order(4)
        .possible_values(&["list", "get", "add", "update", "delete"])
        .takes_value(true)
        .required(true),
    )
    .arg(
      clap::Arg::with_name("secret_key")
        .long("secret_key")
        .value_name("SECRET_KEY")
        .takes_value(true)
        .required_ifs(&[
          ("action", "add"),
          ("action", "update"),
          ("action", "get"),
          ("action", "delete"),
        ]),
    )
    .arg(
      clap::Arg::with_name("secret_value")
        .long("secret_value")
        .value_name("SECRET_VALUE")
        .takes_value(true)
        .required_ifs(&[("action", "add"), ("action", "update")]),
    )
    .get_matches();

  let repo_owner = matches.value_of("repo_owner").unwrap();
  let repo_name = matches.value_of("repo_name").unwrap();
  let auth_token = matches.value_of("auth_token").unwrap();

  let action = matches.value_of("action");
  let secret_key = matches.value_of("secret_key");
  let secret_value = matches.value_of("secret_value");

  let repo = ReposRequestParams::new(repo_owner, repo_name, auth_token);

  match (action, secret_key, secret_value) {
    (Some("list"), _, _) => {
      let secret_list = repo.get_secret_list().await?;
      println!("All Secrets:\n\n{}", secret_list);
    }
    (Some("get"), Some(secret_key), _) => {
      let secret = repo.get_a_secret(&secret_key).await?;
      println!("Secret:\n\n{}", secret);
    }
    (Some("add"), Some(secret_key), Some(secret_value))
    | (Some("edit"), Some(secret_key), Some(secret_value)) => {
      repo.save_secret(secret_key, secret_value).await?;
    }
    (Some("delete"), Some(secret_key), _) => {
      repo.delete_a_secret(secret_key).await?;
    }
    _ => {}
  }
  Ok(())
}

struct Repo<'a> {
  repo_owner: &'a str,
  repo_name: &'a str,
}

struct ReposRequestParams<'a>(Repo<'a>, &'a str);

impl<'a> ReposRequestParams<'a> {
  fn new(repo_owner: &'a str, repo_name: &'a str, auth_token: &'a str) -> Self {
    let repo = Repo {
      repo_owner: &repo_owner,
      repo_name: &repo_name,
    };
    ReposRequestParams(repo, &auth_token)
  }

  async fn get_public_key(&self) -> Result<PublicKeyResponse> {
    get_public_key(&self).await
  }

  async fn get_secret_list(&self) -> Result<SecretListResponse> {
    get_secret_list(&self).await
  }

  async fn get_a_secret(&self, secret_key: &str) -> Result<Secret> {
    get_a_secret(&self, &secret_key).await
  }

  async fn save_secret(&self, secret_key: &str, secret_value: &str) -> Result<()> {
    let public_key = self.get_public_key().await?;
    let secret_save_req = SecretSaveRequest::from(secret_key, secret_value, public_key)?;
    secret_save_req.make_api_call(&self).await
  }

  async fn delete_a_secret(&self, secret_key: &str) -> Result<()> {
    delete_a_secret(&self, secret_key).await
  }
}

#[derive(Deserialize, Serialize, Debug)]
struct SecretSaveRequest {
  #[serde(skip_serializing)]
  key: String,
  encrypted_value: String,
  key_id: String,
  #[serde(skip_serializing)]
  public_key: PublicKeyResponse,
}

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

  async fn make_api_call(&self, params: &ReposRequestParams<'_>) -> Result<()> {
    put_gh_secret(&params, &self.key, &self).await
  }
}

async fn put_gh_secret(
  params: &ReposRequestParams<'_>,
  secret_key: &str,
  secret_save_req: &SecretSaveRequest,
) -> Result<()> {
  let ReposRequestParams(repo, auth_token) = params;
  let Repo {
    repo_owner,
    repo_name,
  } = repo;
  let uri = format!(
    "https://api.github.com/repos/{}/{}/actions/secrets/{}",
    repo_owner, repo_name, secret_key
  )
  .parse::<hyper::Uri>()
  .unwrap();
  let body = serde_json::to_string(&secret_save_req)?;
  let client = create_https_client();
  let req = create_request(auth_token)
    .method(hyper::Method::PUT)
    .uri(uri)
    .body(hyper::Body::from(body))?;

  let res = client.request(req).await?;

  println!("Response: {}", res.status());

  Ok(())
}

async fn get_from_gh<T>(path: &str, params: &ReposRequestParams<'_>) -> Result<T>
where
  T: serde::de::DeserializeOwned,
{
  let ReposRequestParams(repo, auth_token) = params;
  let Repo {
    repo_owner,
    repo_name,
  } = repo;
  let uri = format!(
    "https://api.github.com/repos/{}/{}/{}",
    repo_owner, repo_name, path
  )
  .parse::<hyper::Uri>()
  .unwrap();
  let client = create_https_client();
  let req = create_request(auth_token)
    .method(hyper::Method::GET)
    .uri(uri)
    .body(hyper::Body::empty())?;

  let res = client.request(req).await?;

  let body = hyper::body::aggregate(res).await?;

  let secret_list = serde_json::from_reader(body.reader())?;

  Ok(secret_list)
}

#[derive(Deserialize, Serialize, Debug)]
struct Secret {
  name: String,
  created_at: String,
  updated_at: String,
}

impl std::fmt::Display for Secret {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "{}",
      serde_json::to_string_pretty(&self).unwrap_or_default()
    )
  }
}

async fn get_a_secret(params: &ReposRequestParams<'_>, secret_key: &str) -> Result<Secret> {
  get_from_gh(&format!("actions/secrets/{}", secret_key), &params).await
}

#[derive(Deserialize, Serialize, Debug)]
struct SecretListResponse {
  total_count: u32,
  secrets: Vec<Secret>,
}

impl std::fmt::Display for SecretListResponse {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "{}",
      serde_json::to_string_pretty(&self).unwrap_or_default()
    )
  }
}

async fn get_secret_list(params: &ReposRequestParams<'_>) -> Result<SecretListResponse> {
  get_from_gh("actions/secrets", &params).await
}

#[derive(Deserialize, Serialize, Debug)]
struct PublicKeyResponse {
  key_id: String,
  key: String,
}

async fn get_public_key(params: &ReposRequestParams<'_>) -> Result<PublicKeyResponse> {
  get_from_gh("actions/secrets/public-key", &params).await
}

async fn delete_a_secret(params: &ReposRequestParams<'_>, secret_key: &str) -> Result<()> {
  let ReposRequestParams(repo, auth_token) = params;
  let Repo {
    repo_owner,
    repo_name,
  } = repo;
  let uri = format!(
    "https://api.github.com/repos/{}/{}/actions/secrets/{}",
    repo_owner, repo_name, secret_key
  )
  .parse::<hyper::Uri>()
  .unwrap();
  let client = create_https_client();
  let req = create_request(auth_token)
    .method(hyper::Method::DELETE)
    .uri(uri)
    .body(hyper::Body::empty())?;

  let res = client.request(req).await?;

  println!("Response: {}", res.status());

  Ok(())
}

fn create_https_client(
) -> hyper::client::Client<HttpsConnector<hyper::client::connect::HttpConnector>, hyper::Body> {
  let https = HttpsConnector::new();
  Client::builder().build::<_, hyper::Body>(https)
}

fn create_request(auth_token: &str) -> hyper::http::request::Builder {
  Request::builder()
    .header("Authorization", format!("bearer {}", auth_token))
    .header("User-Agent", "gh-actions-secrets-cli")
}

fn seal(message: &str, public_key_base64: &str) -> Result<String> {
  let mut rng = rand::thread_rng();
  let secret_key = SecretKey::generate(&mut rng);

  let public_key = base64::decode(public_key_base64)?;

  let public_key_buffer =
    public_key[..32]
      .iter()
      .enumerate()
      .fold([0u8; 32], |mut buffer, (i, n)| {
        buffer[i] = *n;
        buffer
      });

  let public_key = PublicKey::from(public_key_buffer);

  let boxed_secret = Box::new(&public_key, &secret_key);
  let nonce = crypto_box::generate_nonce(&mut rng);

  match boxed_secret.encrypt(&nonce, message.as_bytes()) {
    Ok(cipher_text) => {
      let t = base64::encode(&cipher_text);
      Ok(t)
    }
    Err(e) => Err(format!("[Error] unable to encypt: {:?}", e).into()),
  }
}
