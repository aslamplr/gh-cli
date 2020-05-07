use clap::Clap;
use gh_cli::{core::repos::ReposRequestParams, Result};

#[derive(Clap)]
#[clap(
    name = "GitHub CLI",
    version = "v0.3.0",
    author = "Aslam Ahammed A. <aslamplr@gmail.com>",
    about = r#"Yet another unofficial GitHub CLI!
Minimalistic, opinionated, and unofficial by default.
Work is in progress to add more subcommands.
Absolute No Warranty!"#
)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Secrets(Secrets),
}

#[derive(Clap)]
#[clap(
    name = "GitHub Actions Secrets CLI",
    version = "v0.3.0",
    author = "Aslam Ahammed A. <aslamplr@gmail.com>",
    about = "GitHub Actions Secrets CLI wrapper for GitHub Actions Secrets API"
)]
struct Secrets {
    #[clap(
        long = "auth_token",
        short = "t",
        value_name = "PERSONAL_ACCESS_TOKEN",
        about = "Generate token - https://github.com/settings/tokens",
        display_order = 3,
        takes_value = true,
        required = true
    )]
    auth_token: String,
    #[clap(
        long = "repo_owner",
        short = "o",
        value_name = "REPO_OWNER",
        about = "Repository owner",
        display_order = 1,
        takes_value = true,
        required = true
    )]
    repo_owner: String,
    #[clap(
        long = "repo_name",
        short = "n",
        value_name = "REPO_NAME",
        about = "Repository name",
        display_order = 2,
        takes_value = true,
        required = true
    )]
    repo_name: String,
    #[clap(long = "action", short = "a", value_name = "ACTION", possible_values = &["list", "get", "add", "update", "delete"], display_order = 4, takes_value = true, required = true)]
    action: String,
    #[clap(long = "secret_key", value_name = "SECRET_KEY", takes_value = true, required_ifs = &[
        ("action", "add"),
        ("action", "update"),
        ("action", "get"),
        ("action", "delete"),
    ])]
    secret_key: Option<String>,
    #[clap(long = "secret_value", value_name = "SECRET_VALUE", takes_value = true, required_ifs = &[("action", "add"), ("action", "update")])]
    secret_value: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Secrets(secrets) => {
            let Secrets {
                repo_name,
                repo_owner,
                auth_token,
                action,
                secret_key,
                secret_value,
            } = secrets;

            let repo = ReposRequestParams::new(&repo_owner, &repo_name, &auth_token);

            match (action.as_ref(), secret_key, secret_value) {
                ("list", _, _) => {
                    let secret_list = repo.get_secret_list().await?;
                    println!("All Secrets:\n\n{}", secret_list);
                }
                ("get", Some(secret_key), _) => {
                    let secret = repo.get_a_secret(&secret_key).await?;
                    println!("Secret:\n\n{}", secret);
                }
                ("add", Some(secret_key), Some(secret_value))
                | ("update", Some(secret_key), Some(secret_value)) => {
                    repo.save_secret(&secret_key, &secret_value).await?;
                }
                ("delete", Some(secret_key), _) => {
                    repo.delete_a_secret(&secret_key).await?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
