use ansi_term::{Color, Style};
use clap::Clap;
use gh_lib::core::{
    basic_info::{basic_info_response, BasicInfo as _},
    repos::RepoRequest,
    secrets::Secrets as _,
};

lazy_static::lazy_static! {
    static ref GIT_REPO_ADDR_FROM_REPO: anyhow::Result<String> = get_git_addr_from_repo();
    static ref REPO_ADDR: &'static str = GIT_REPO_ADDR_FROM_REPO.as_ref().map_or_else(|_| "", |addr| addr);
    static ref IS_ADDR_REQUIRED: bool = GIT_REPO_ADDR_FROM_REPO.is_err();
}

#[derive(Clap)]
#[clap(
    name = "GitHub CLI",
    version = "v0.3.1",
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
    Repo(Repo),
    Secrets(Secrets),
}

#[derive(Clap)]
#[clap(
    name = "GitHub Repo CLI",
    version = "v0.3.1",
    author = "Aslam Ahammed A. <aslamplr@gmail.com>",
    about = "GitHub Repo CLI"
)]
struct Repo {
    #[clap(
        long = "name",
        short = "n",
        value_name = "OWNER/NAME",
        about = "Repository address including the owner and name seperated by slash\nEg. aslamplr/gh-cli",
        display_order = 1,
        takes_value = true,
        required = *IS_ADDR_REQUIRED,
        default_value = &REPO_ADDR,
        hide_default_value = true,
    )]
    name: String,
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
}

#[derive(Clap)]
#[clap(
    name = "GitHub Actions Secrets CLI",
    version = "v0.3.1",
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
        required = *IS_ADDR_REQUIRED,
        default_value = &REPO_ADDR,
        hide_default_value = true,
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
        SubCommand::Repo(repo) => {
            let Repo { name, auth_token } = repo;
            let repo = RepoRequest::try_from(&name, &auth_token)?;
            let basic_info = repo.get_basic_info().await?;
            if let Some(repo) = &basic_info.repository {
                let basic_info_response::RepoBasicInfoQueryRepository {
                    name_with_owner,
                    description,
                    created_at,
                    pushed_at,
                    homepage_url,
                    is_private,
                    is_archived,
                    primary_language,
                    license_info,
                    stargazers,
                } = repo;
                let access_type = if *is_private { "Private" } else { "Public" };
                let license = if let Some(license_info) = &license_info {
                    &license_info.name
                } else {
                    "Unlicensed"
                };
                let stargazers = stargazers.total_count;
                let primary_language = if let Some(primary_language) = primary_language {
                    format!(" [{}]", &primary_language.name)
                } else {
                    "".to_owned()
                };
                println!("Repo Basic Information:\n");
                println!(
                    "{}",
                    Style::new().bold().paint(format!(
                        "{} [🚦 {}] [⚖️  {}] [⭐️ {}]{}",
                        name_with_owner, access_type, license, stargazers, primary_language
                    ))
                );
                if let Some(homepage_url) = homepage_url {
                    println!("{}", &homepage_url);
                }
                if *is_archived {
                    println!("This repo is archived");
                }
                if let Some(description) = description {
                    println!("{}", &description);
                }
                println!();
                println!("Created on \t{}", created_at);
                if let Some(pushed_at) = pushed_at {
                    println!("Last commit on \t{}", pushed_at);
                }
            }
        }
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

fn get_git_addr_from_repo() -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(&["config", "--get", "remote.origin.url"])
        .output()?
        .stdout;
    lazy_static::lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(r"github\.com[:/](\S+)/(\S+)\.git").unwrap();
    }
    let regex_cap_err = || anyhow::anyhow!("Unable to capture github git repo!");
    RE.captures(std::str::from_utf8(&output)?)
        .and_then(|caps| {
            match (
                caps.get(1).map(|c| c.as_str()),
                caps.get(2).map(|c| c.as_str()),
            ) {
                (Some(owner), Some(name)) => Some(format!("{}/{}", owner, name)),
                _ => None,
            }
        })
        .ok_or_else(regex_cap_err)
}
