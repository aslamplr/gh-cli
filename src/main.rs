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

  let secret_key = matches.value_of("secret_key");
  let secret_value = matches.value_of("secret_value");

  match (matches.value_of("action"), secret_key, secret_value) {
    (Some("list"), _, _) => {
      let secret_list = get_secret_list(repo_owner, repo_name, auth_token).await?;
      println!("Secret List:\n\n{:#?}", secret_list);
    }
    (Some("get"), Some(secret_key), _) => {
      let secret = get_a_secret(repo_owner, repo_name, auth_token, secret_key).await?;
      println!("Secret:\n\n{:#?}", secret);
    }
    (Some("add"), Some(secret_key), Some(secret_value))
    | (Some("edit"), Some(secret_key), Some(secret_value)) => {
      let public_key = get_public_key(repo_owner, repo_name, auth_token).await?;
      SecretSaveRequest::from(secret_key, secret_value, public_key)?
        .make_api_call(repo_owner, repo_name, auth_token)
        .await?;
    }
    (Some("delete"), Some(secret_key), _) => {
      delete_a_secret(repo_owner, repo_name, auth_token, secret_key).await?;
    }
    _ => {}
  }
  Ok(())
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

  async fn make_api_call(&self, repo_owner: &str, repo_name: &str, auth_token: &str) -> Result<()> {
    put_gh_secret(repo_owner, repo_name, auth_token, &self.key, &self).await
  }
}

async fn put_gh_secret(
  repo_owner: &str,
  repo_name: &str,
  auth_token: &str,
  secret_key: &str,
  secret_save_req: &SecretSaveRequest,
) -> Result<()> {
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

async fn get_from_gh<T>(
  path: &str,
  repo_owner: &str,
  repo_name: &str,
  auth_token: &str,
) -> Result<T>
where
  T: serde::de::DeserializeOwned,
{
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

  let secret_list: T = serde_json::from_reader(body.reader())?;

  Ok(secret_list)
}

#[derive(Deserialize, Serialize, Debug)]
struct Secret {
  name: String,
  created_at: String,
  updated_at: String,
}

async fn get_a_secret(
  repo_owner: &str,
  repo_name: &str,
  auth_token: &str,
  secret_key: &str,
) -> Result<Secret> {
  get_from_gh(
    &format!("actions/secrets/{}", secret_key),
    repo_owner,
    repo_name,
    auth_token,
  )
  .await
}

#[derive(Deserialize, Serialize, Debug)]
struct SecretListResponse {
  total_count: u32,
  secrets: Vec<Secret>,
}

async fn get_secret_list(
  repo_owner: &str,
  repo_name: &str,
  auth_token: &str,
) -> Result<SecretListResponse> {
  get_from_gh("actions/secrets", repo_owner, repo_name, auth_token).await
}

#[derive(Deserialize, Serialize, Debug)]
struct PublicKeyResponse {
  key_id: String,
  key: String,
}

async fn get_public_key(
  repo_owner: &str,
  repo_name: &str,
  auth_token: &str,
) -> Result<PublicKeyResponse> {
  get_from_gh(
    "actions/secrets/public-key",
    repo_owner,
    repo_name,
    auth_token,
  )
  .await
}

async fn delete_a_secret(
  repo_owner: &str,
  repo_name: &str,
  auth_token: &str,
  secret_key: &str,
) -> Result<()> {
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
    .header(
      "Authorization",
      format!("bearer {}", auth_token),
    )
    .header("User-Agent", "gh-actions-secrets-cli")
}

fn seal(message: &str, public_key_base64: &str) -> Result<String> {
  let mut rng = rand::thread_rng();
  let secret_key = SecretKey::generate(&mut rng);

  let public_key = base64::decode(public_key_base64)?;

  let mut public_key_buffer = [0u8; 32];
  for (i, n) in public_key[..32].iter().enumerate() {
    public_key_buffer[i] = *n;
  }

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
