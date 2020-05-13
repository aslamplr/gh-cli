use ansi_term::{Color, Style};
use clap::Clap;
use gh_cli::core::{repos::RepoRequest, secrets::Secrets as _};

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
        env = "GH_ACCESS_TOKEN",
        hide_env_values = true,
        about = "Generate token - https://github.com/settings/tokens",
        display_order = 2,
        takes_value = true,
        required = true
    )]
    auth_token: String,
    #[clap(
        long = "name",
        short = "n",
        value_name = "OWNER/NAME",
        about = "Repository address including the owner and name seperated by slash\nEg. aslamplr/gh-cli",
        display_order = 1,
        takes_value = true,
        required = true
    )]
    name: String,
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
async fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Secrets(secrets) => {
            let Secrets {
                name,
                auth_token,
                action,
                secret_key,
                secret_value,
            } = secrets;

            let repo = RepoRequest::try_from(&name, &auth_token)?;

            match (action.as_ref(), secret_key, secret_value) {
                ("list", _, _) => {
                    let secret_list = repo.get_all_secrets().await?;
                    println!(
                        "All Secrets:\n\n{}",
                        Style::new().bold().paint(secret_list.to_string())
                    );
                }
                ("get", Some(secret_key), _) => {
                    let secret = repo.get_a_secret(&secret_key).await?;
                    println!(
                        "Secret:\n\n{}",
                        Style::new().bold().paint(secret.to_string())
                    );
                }
                (action, Some(secret_key), Some(secret_value))
                    if ["add", "update"].contains(&action) =>
                {
                    repo.save_secret(&secret_key, &secret_value).await?;
                    println!(
                        "{}",
                        Color::Green
                            .bold()
                            .paint(format!("Secret {} successful!", action))
                    );
                }
                ("delete", Some(secret_key), _) => {
                    repo.delete_a_secret(&secret_key).await?;
                    println!("{}", Color::Green.bold().paint("Secret delete successful!"));
                }
                _ => {}
            }
        }
    }

    Ok(())
}
