use gh_actions_secrets::{core::repos::ReposRequestParams, Result};

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
        | (Some("update"), Some(secret_key), Some(secret_value)) => {
            repo.save_secret(secret_key, secret_value).await?;
        }
        (Some("delete"), Some(secret_key), _) => {
            repo.delete_a_secret(secret_key).await?;
        }
        _ => {}
    }
    Ok(())
}
