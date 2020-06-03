use clap::Clap;
use crossterm::style::{Colorize, Styler};
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
    #[clap(about = "Login using GitHub OAuth (requires web browser)")]
    Login,
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
    #[clap(long, short, about = "Print README")]
    readme: bool,
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
        SubCommand::Login => {
            let user_input = {
                use std::io::{Read, Write};
                print!("Press Enter to open github.com in your browser for auth...");
                std::io::stdout().flush()?;
                let mut buf = [0u8; 1];
                std::io::stdin().read_exact(&mut buf)?;
                buf[0] as char
            };
            if user_input == '\n' {
                let access_token = gh_auth::start_auth_flow().await?;
                eprintln!("# Run the following to use the access token in subesquent requests!\n");
                println!("export GH_ACCESS_TOKEN={}", access_token);
                eprintln!("");
                let oauth_host = gh_auth::OAUTH_HOST;
                let client_id = gh_auth::OAUTH_CLIENT_ID;
                eprintln!(
                    "# Review or revoke access visit - https://{}/settings/connections/applications/{}",
                    oauth_host, client_id
                );
            } else {
                return Err(anyhow::anyhow!("Unexpected input!"));
            }
        }
        SubCommand::Repo(repo) => {
            let Repo {
                name,
                auth_token,
                readme,
            } = repo;
            let repo = RepoRequest::try_from(&name, &auth_token)?;
            let (basic_info, readme) = {
                if readme {
                    tokio::join!(repo.get_basic_info(), async {
                        repo.get_raw_readme().await.ok()
                    })
                } else {
                    (repo.get_basic_info().await, None)
                }
            };
            let basic_info = basic_info?;
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
                    format!(
                        "{} [🚦 {}] [⚖️  {}] [⭐️ {}]{}",
                        name_with_owner, access_type, license, stargazers, primary_language
                    )
                    .bold()
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
                if let Some(readme) = readme {
                    println!();
                    println!("README:");
                    printmd("---");
                    printmd(&readme);
                    printmd("---");
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
                    println!("All Secrets:\n\n{}", secret_list.to_string().bold());
                }
                ("get", Some(secret_key), _) => {
                    let secret = repo.get_a_secret(&secret_key).await?;
                    println!("Secret:\n\n{}", secret.to_string().bold());
                }
                (action, Some(secret_key), Some(secret_value))
                    if ["add", "update"].contains(&action) =>
                {
                    repo.save_secret(&secret_key, &secret_value).await?;
                    println!(
                        "{}",
                        format!("Secret {} successful!", action).bold().green()
                    );
                }
                ("delete", Some(secret_key), _) => {
                    repo.delete_a_secret(&secret_key).await?;
                    println!("{}", "Secret delete successful!".bold().green());
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

fn printmd(md: &str) {
    use crossterm::style::{Attribute, Color};
    use termimad::{rgb, MadSkin, StyledChar};
    lazy_static::lazy_static! {
        static ref TERM_SKIN: MadSkin = {
            let mut skin = MadSkin::default();
            skin.set_headers_fg(Color::DarkCyan);
            skin.headers.iter_mut().for_each(|h| h.add_attr(Attribute::Bold));
            skin.headers[0].set_fg(Color::Yellow);
            skin.headers[0].set_bg(Color::DarkCyan);
            skin.bold.set_fg(Color::DarkYellow);
            skin.italic.set_fgbg(Color::Magenta, rgb(30, 30, 40));
            skin.bullet = StyledChar::from_fg_char(Color::Yellow, '⟡');
            skin.quote_mark.set_fg(Color::Yellow);
            skin
        };
    }
    TERM_SKIN.print_text(md);
}
