use clap::Clap;
use crossterm::style::{Colorize, Styler};
#[cfg(feature = "basic-info")]
use gh_lib::core::basic_info::{basic_info_response, BasicInfo as _};
#[cfg(feature = "collaborators")]
use gh_lib::core::collaborators::Collaborators as _;
use gh_lib::core::repos::RepoRequest;
#[cfg(feature = "secrets")]
use gh_lib::core::secrets::{Secret, SecretListResponse, Secrets as _};
#[cfg(feature = "workflows")]
use gh_lib::core::{
    workflow_jobs::WorkflowJobs as _,
    workflow_runs::WorkflowRuns as _,
    workflows::{WorkflowList, WorkflowUsage, WorkflowUsageTiming, Workflows as _},
};

#[cfg(feature = "config")]
mod config;

macro_rules! printmd {
    ($($arg:tt)*) => ({
        printmd(&format!($($arg)*));
    })
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

fn get_git_addr_from_repo() -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(&["config", "--get", "remote.origin.url"])
        .output()?
        .stdout;
    let re = regex::Regex::new(r"github\.com[:/](\S+)/(\S+)\.git").unwrap();
    let regex_cap_err = || anyhow::anyhow!("Unable to capture github git repo!");
    re.captures(std::str::from_utf8(&output)?)
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

lazy_static::lazy_static! {
    static ref GIT_REPO_ADDR_FROM_REPO: anyhow::Result<String> = get_git_addr_from_repo();
    static ref REPO_ADDR: &'static str = GIT_REPO_ADDR_FROM_REPO.as_ref().map_or_else(|_| "", |addr| addr);
    static ref IS_ADDR_REQUIRED: bool = GIT_REPO_ADDR_FROM_REPO.is_err();
}

#[derive(Clap)]
#[clap(
    name = "GitHub CLI",
    version = concat!("v", clap::crate_version!()),
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
    #[cfg(feature = "login")]
    #[clap(about = "Login using GitHub OAuth (requires web browser)")]
    Login,
    #[cfg(any(feature = "basic-info", feature = "collaborators"))]
    #[clap(about = "Repository operations")]
    Repo(Repo),
    #[cfg(feature = "secrets")]
    #[clap(about = "Actions secrets")]
    Secrets(Secrets),
    #[cfg(any(feature = "secrets", feature = "workflows"))]
    #[clap(about = "GitHub Actions operations")]
    Actions(Actions),
}

#[cfg(any(feature = "basic-info", feature = "collaborators"))]
#[derive(Clap)]
#[clap(
    name = "GitHub Repo CLI",
    version = concat!("v", clap::crate_version!()),
    author = "Aslam Ahammed A. <aslamplr@gmail.com>",
    about = "GitHub Repo CLI"
)]
struct Repo {
    #[clap(subcommand)]
    subcmd: RepoSubCommand,
}

#[cfg(any(feature = "basic-info", feature = "collaborators"))]
#[derive(Clap)]
enum RepoSubCommand {
    #[cfg(feature = "basic-info")]
    #[clap(about = "View repository basic information")]
    View(RepoView),
    #[cfg(feature = "collaborators")]
    #[clap(about = "List and manage repository collaborators")]
    Collaborators(Collaborators),
}

#[cfg(feature = "basic-info")]
#[derive(Clap)]
struct RepoView {
    #[clap(
        long = "name",
        short = 'n',
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
        short = 't',
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

#[cfg(feature = "collaborators")]
#[derive(Clap)]
struct Collaborators {
    #[clap(
        long = "name",
        short = 'n',
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
        short = 't',
        value_name = "PERSONAL_ACCESS_TOKEN",
        env = "GH_ACCESS_TOKEN",
        hide_env_values = true,
        about = "Generate token - https://github.com/settings/tokens",
        display_order = 2,
        takes_value = true,
        required = true
    )]
    auth_token: String,
    #[clap(subcommand)]
    subcmd: CollaboratorsSubCommand,
}

#[cfg(feature = "collaborators")]
#[derive(Clap)]
enum CollaboratorsSubCommand {
    #[clap(about = "List all repository contributors")]
    List,
    #[clap(about = "Check if the user with username is a collaborator to the the repository")]
    Check(CollaboratorUser),
    #[clap(about = "Add the user with username as a collaborator to the repository")]
    Add(CollaboratorUserWithPermission),
    #[clap(about = "Remove the user with username as a collaborator from the repository")]
    Remove(CollaboratorUser),
    #[clap(about = "Get collaborators permission for the repository")]
    Permission(CollaboratorUser),
}

#[cfg(feature = "collaborators")]
#[derive(Clap)]
struct CollaboratorUser {
    #[clap(name = "USER_NAME", about = "GitHub username", index = 1)]
    username: String,
}

#[cfg(feature = "collaborators")]
#[derive(Clap)]
struct CollaboratorUserWithPermission {
    #[clap(name = "USER_NAME", about = "GitHub username", index = 1)]
    username: String,
    #[clap( name = "PERMISSION", about = "Contributor permission", index = 2, possible_values = &["pull", "push", "admin", "maintain", "triage"], default_value = "push")]
    permission: String,
}

#[cfg(any(feature = "secrets", feature = "workflows"))]
#[derive(Clap)]
#[clap(
    name = "GitHub Actions CLI",
    version = concat!("v", clap::crate_version!()),
    author = "Aslam Ahammed A. <aslamplr@gmail.com>",
    about = "GitHub Actions CLI have commands to view and control action workflows, runs and jobs"
)]
struct Actions {
    #[clap(subcommand)]
    subcmd: ActionsSubCommand,
}

#[cfg(any(feature = "secrets", feature = "workflows"))]
#[derive(Clap)]
enum ActionsSubCommand {
    #[cfg(feature = "workflows")]
    #[clap(about = "Actions Workflows")]
    Workflows(Workflows),
    #[cfg(feature = "workflows")]
    #[clap(about = "Actions Workflow Runs")]
    Runs(WorkflowRuns),
    #[cfg(feature = "workflows")]
    #[clap(about = "Actions Workflow Jobs")]
    Jobs(WorkflowJobs),
    #[cfg(feature = "secrets")]
    #[clap(about = "Actions Secrets")]
    Secrets(Secrets),
}

#[cfg(feature = "workflows")]
#[derive(Clap)]
struct Workflows {
    #[clap(
        long = "auth_token",
        short = 't',
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
        short = 'n',
        value_name = "OWNER/NAME",
        about = "Repository address including the owner and name seperated by slash\nEg. aslamplr/gh-cli",
        display_order = 1,
        takes_value = true,
        required = *IS_ADDR_REQUIRED,
        default_value = &REPO_ADDR,
        hide_default_value = true,
    )]
    name: String,
    #[clap(subcommand)]
    subcmd: WorkflowsSubCommand,
}

#[cfg(feature = "workflows")]
#[derive(Clap)]
enum WorkflowsSubCommand {
    List,
    Get(WorkflowId),
    Usage(WorkflowId),
}

#[cfg(feature = "workflows")]
#[derive(Clap)]
struct WorkflowId {
    workflow_id: u32,
}

#[cfg(feature = "workflows")]
#[derive(Clap)]
struct WorkflowRuns {
    #[clap(
        long = "auth_token",
        short = 't',
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
        short = 'n',
        value_name = "OWNER/NAME",
        about = "Repository address including the owner and name seperated by slash\nEg. aslamplr/gh-cli",
        display_order = 1,
        takes_value = true,
        required = *IS_ADDR_REQUIRED,
        default_value = &REPO_ADDR,
        hide_default_value = true,
    )]
    name: String,
    #[clap(subcommand)]
    subcmd: WorkflowRunsSubCommand,
}

#[cfg(feature = "workflows")]
#[derive(Clap)]
enum WorkflowRunsSubCommand {
    #[clap(about = "List All Repo Workflow Runs")]
    List,
    #[clap(about = "List All Workflow Runs for <workflow_id>")]
    ListWorkflow(WorkflowId),
    #[clap(about = "Get a Workflow Run for <run_id>")]
    Get(WorkflowRunId),
    #[clap(about = "Re-Run a Workflow Run for <run_id>")]
    ReRun(WorkflowRunId),
    #[clap(about = "Cancel a Workflow Run for <run_id>")]
    Cancel(WorkflowRunId),
    #[clap(about = "Download logs url for a Workflow Run for <run_id>")]
    DownloadLogs(WorkflowRunId),
    #[clap(about = "Delete logs for a Workflow Run for <run_id>")]
    DeleteLogs(WorkflowRunId),
    #[clap(about = "Get usage of a Workflow Run for <run_id>")]
    Usage(WorkflowRunId),
}

#[cfg(feature = "workflows")]
#[derive(Clap)]
struct WorkflowRunId {
    run_id: u32,
}

#[cfg(feature = "workflows")]
#[derive(Clap)]
struct WorkflowJobs {
    #[clap(
        long = "auth_token",
        short = 't',
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
        short = 'n',
        value_name = "OWNER/NAME",
        about = "Repository address including the owner and name seperated by slash\nEg. aslamplr/gh-cli",
        display_order = 1,
        takes_value = true,
        required = *IS_ADDR_REQUIRED,
        default_value = &REPO_ADDR,
        hide_default_value = true,
    )]
    name: String,
    #[clap(subcommand)]
    subcmd: WorkflowJobsSubCommand,
}

#[cfg(feature = "workflows")]
#[derive(Clap)]
enum WorkflowJobsSubCommand {
    #[clap(about = "List jobs for a Workflow Run for <run_id>")]
    List(WorkflowRunId),
    #[clap(about = "Get a job for <job_id>")]
    Get(WorkflowJobId),
    #[clap(about = "Get logs url for a job for <job_id>")]
    DownloadLogs(WorkflowJobId),
}

#[cfg(feature = "workflows")]
#[derive(Clap)]
struct WorkflowJobId {
    job_id: u32,
}

#[cfg(feature = "secrets")]
#[derive(Clap)]
struct Secrets {
    #[clap(
        long = "auth_token",
        short = 't',
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
        short = 'n',
        value_name = "OWNER/NAME",
        about = "Repository address including the owner and name seperated by slash\nEg. aslamplr/gh-cli",
        display_order = 1,
        takes_value = true,
        required = *IS_ADDR_REQUIRED,
        default_value = &REPO_ADDR,
        hide_default_value = true,
    )]
    name: String,
    #[clap(subcommand)]
    subcmd: SecretsSubCommand,
}

#[cfg(feature = "secrets")]
#[derive(Clap)]
enum SecretsSubCommand {
    #[clap(about = "List all secrets")]
    List,
    #[clap(about = "Print a secret")]
    Get(SecretsName),
    #[cfg(feature = "secrets-save")]
    #[clap(about = "Add a new secret")]
    Add(SecretsNameValue),
    #[cfg(feature = "secrets-save")]
    #[clap(about = "Update a secret")]
    Update(SecretsNameValue),
    #[cfg(feature = "secrets-save")]
    #[clap(about = "Update a secret (an alias for update)")]
    Edit(SecretsNameValue),
    #[clap(about = "Delete a secret")]
    Delete(SecretsName),
}

#[cfg(feature = "secrets")]
impl std::fmt::Display for SecretsSubCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let act = match self {
            SecretsSubCommand::List => "list",
            SecretsSubCommand::Get(_) => "get",
            #[cfg(feature = "secrets-save")]
            SecretsSubCommand::Add(_)
            | SecretsSubCommand::Update(_)
            | SecretsSubCommand::Edit(_) => "save",
            SecretsSubCommand::Delete(_) => "delete",
        };
        write!(f, "{}", act)?;
        Ok(())
    }
}

#[cfg(feature = "secrets")]
#[derive(Clap)]
struct SecretsName {
    #[clap(name = "SECRET_NAME", index = 1)]
    name: String,
}

#[cfg(feature = "secrets-save")]
#[derive(Clap)]
struct SecretsNameValue {
    #[clap(name = "SECRET_NAME", index = 1)]
    name: String,
    #[clap(name = "SECRET_VALUE", index = 2)]
    value: String,
}

#[cfg(feature = "login")]
async fn handle_login() -> anyhow::Result<()> {
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
        #[cfg(feature = "config")]
        {
            let config = config::Config::new("_", &access_token);
            if let Ok(config_path) = config::save_config(config).await {
                eprintln!("# Access token saved to config file: {:?}", config_path);
            } else {
                eprintln!("# Unable to establish a config file!");
                eprintln!("# Run the following to use the access token in subesquent requests!\n");
                println!("export GH_ACCESS_TOKEN={}", access_token);
            }
        }
        #[cfg(not(feature = "config"))]
        {
            eprintln!("# Run the following to use the access token in subesquent requests!\n");
            println!("export GH_ACCESS_TOKEN={}", access_token);
        }
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
    Ok(())
}

#[cfg(feature = "basic-info")]
async fn handle_repo_view(repo: &RepoView) -> anyhow::Result<()> {
    let RepoView {
        name,
        auth_token,
        readme,
    } = repo;
    let repo = RepoRequest::try_from(&name, &auth_token)?;
    let (basic_info, readme) = {
        if *readme {
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
        println!("Created at \t{}", created_at);
        if let Some(pushed_at) = pushed_at {
            println!("Last commit at \t{}", pushed_at);
        }
        if let Some(readme) = readme {
            println!();
            printmd("---");
            printmd(&readme);
            printmd("---");
        }
    }
    Ok(())
}

#[cfg(feature = "basic-info")]
async fn handle_repo_collaborators(collab: &Collaborators) -> anyhow::Result<()> {
    let Collaborators {
        name,
        auth_token,
        subcmd,
    } = collab;

    let repo = RepoRequest::try_from(&name, &auth_token)?;

    match &subcmd {
        CollaboratorsSubCommand::List => {
            let collaborators = repo.get_collaborators().await?;
            println!("Collaborators: \n{:#?}", collaborators);
        }
        CollaboratorsSubCommand::Check(user) => {
            let is_collaborator = repo.is_collaborator(&user.username).await?;
            println!("Is collaborator: {}", is_collaborator);
        }
        CollaboratorsSubCommand::Add(user) => {
            let add_response = repo
                .add_collaborator(&user.username, &user.permission)
                .await?;
            println!("Add collaborator response: {:?}", add_response);
        }
        CollaboratorsSubCommand::Remove(user) => {
            repo.remove_collaborator(&user.username).await?;
            println!("Removed collaborator");
        }
        CollaboratorsSubCommand::Permission(user) => {
            let resp = repo.get_permission(&user.username).await?;
            println!("User permission: {}", resp.permission);
        }
    }

    Ok(())
}

#[cfg(feature = "workflows")]
async fn handle_actions_workflows(workflows: &Workflows) -> anyhow::Result<()> {
    let Workflows {
        name,
        auth_token,
        subcmd,
    } = workflows;

    let repo = RepoRequest::try_from(&name, &auth_token)?;

    match &subcmd {
        WorkflowsSubCommand::List => {
            let WorkflowList {
                total_count,
                workflows,
            } = repo.get_all_workflows().await?;
            let workflows = workflows
                .iter()
                .map(|w| {
                    [
                        format!("|name|{}", w.name),
                        format!("|id|{}", w.id),
                        format!("|node_id|{}", w.node_id),
                        format!("|path|{}", w.path),
                        format!("|state|{}", w.state),
                        format!("|created_at|{}", w.created_at),
                        format!("|updated_at|{}", w.updated_at),
                        format!("|url|{}", w.url),
                        format!("|html_url|{}", w.html_url),
                        format!("|badge_url|{}", w.badge_url),
                    ]
                    .join("\n")
                })
                .collect::<Vec<_>>()
                .join("\n|-:|:-|\n");
            printmd!(
                r#"|-:|:-|
|**Field**|**Value**|
|-:|:-|
{}
|-

Total workflows: {}"#,
                workflows,
                total_count
            );
        }
        WorkflowsSubCommand::Get(WorkflowId { workflow_id }) => {
            let workflow = repo.get_a_workflow(*workflow_id).await?;
            printmd!(
                r#"|-:|:-|
|**Field**|**Value**|
|-:|:-|
{}
|-"#,
                [
                    format!("|name|{}", workflow.name),
                    format!("|id|{}", workflow.id),
                    format!("|node_id|{}", workflow.node_id),
                    format!("|path|{}", workflow.path),
                    format!("|state|{}", workflow.state),
                    format!("|created_at|{}", workflow.created_at),
                    format!("|updated_at|{}", workflow.updated_at),
                    format!("|url|{}", workflow.url),
                    format!("|html_url|{}", workflow.html_url),
                    format!("|badge_url|{}", workflow.badge_url),
                ]
                .join("\n")
            );
        }
        WorkflowsSubCommand::Usage(WorkflowId { workflow_id }) => {
            let WorkflowUsage { billable } = repo.get_workflow_usage(*workflow_id).await?;
            let get_usage_pf = |pf: &str, timing: Option<WorkflowUsageTiming>| {
                format!(
                    "|{}|{}|",
                    pf,
                    timing
                        .map(|t| format!("{}ms", t.total_ms))
                        .unwrap_or_else(|| "unknown".into())
                )
            };
            printmd!(
                r#"|-:|-:|
|**Platform**|**Usage**|
|-:|-:|
{}
|-:|-:|
{}
|-:|-:|
{}
|-"#,
                get_usage_pf("MACOS", billable.MACOS),
                get_usage_pf("UBUNTU", billable.UBUNTU),
                get_usage_pf("WINDOWS", billable.WINDOWS)
            );
        }
    }

    Ok(())
}

#[cfg(feature = "workflows")]
async fn handle_actions_workflow_runs(workflow_runs: &WorkflowRuns) -> anyhow::Result<()> {
    let WorkflowRuns {
        name,
        auth_token,
        subcmd,
    } = workflow_runs;

    let repo = RepoRequest::try_from(&name, &auth_token)?;

    match &subcmd {
        WorkflowRunsSubCommand::List => {
            let all_repo_runs = repo.get_all_workflow_runs().await?;
            println!("All Repo Workflow Runs: {:#?}", all_repo_runs);
        }
        WorkflowRunsSubCommand::ListWorkflow(WorkflowId { workflow_id }) => {
            let workflow_runs = repo.get_workflow_runs(*workflow_id).await?;
            println!("Workflow Runs: {:#?}", workflow_runs);
        }
        WorkflowRunsSubCommand::Get(WorkflowRunId { run_id }) => {
            let workflow_run = repo.get_a_workflow_run(*run_id).await?;
            println!("Workflow Run: {:#?}", workflow_run);
        }
        WorkflowRunsSubCommand::ReRun(WorkflowRunId { run_id }) => {
            repo.rerun_a_workflow(*run_id).await?;
            println!("Workflow Re-Run Initiated!");
        }
        WorkflowRunsSubCommand::Cancel(WorkflowRunId { run_id }) => {
            repo.cancel_a_workflow_run(*run_id).await?;
            println!("Workflow Re-Run Initiated!");
        }
        WorkflowRunsSubCommand::DownloadLogs(WorkflowRunId { run_id }) => {
            let url = repo.get_run_logs_url(*run_id).await?;
            println!("Logs Download Url: {}", url);
        }
        WorkflowRunsSubCommand::DeleteLogs(WorkflowRunId { run_id }) => {
            repo.delete_run_logs(*run_id).await?;
            println!("Logs Deleted!");
        }
        WorkflowRunsSubCommand::Usage(WorkflowRunId { run_id }) => {
            let usage = repo.get_workflow_run_usage(*run_id).await?;
            println!("Workflow Run Usage: {:#?}", usage);
        }
    }

    Ok(())
}

#[cfg(feature = "workflows")]
async fn handle_actions_workflow_jobs(workflow_jobs: &WorkflowJobs) -> anyhow::Result<()> {
    let WorkflowJobs {
        name,
        auth_token,
        subcmd,
    } = workflow_jobs;

    let repo = RepoRequest::try_from(&name, &auth_token)?;

    match &subcmd {
        WorkflowJobsSubCommand::List(WorkflowRunId { run_id }) => {
            let jobs = repo.get_workflow_run_jobs(*run_id).await?;
            println!("Workflow Run Jobs: {:#?}", jobs);
        }
        WorkflowJobsSubCommand::Get(WorkflowJobId { job_id }) => {
            let job = repo.get_a_workflow_run_job(*job_id).await?;
            println!("Workflow Rub Job: {:#?}", job);
        }
        WorkflowJobsSubCommand::DownloadLogs(WorkflowJobId { job_id }) => {
            let url = repo.get_job_logs_url(*job_id).await?;
            println!("Logs Download Url: {}", url);
        }
    }

    Ok(())
}

#[cfg(feature = "secrets")]
async fn handle_actions_secrets(secrets: &Secrets) -> anyhow::Result<()> {
    let Secrets {
        name,
        auth_token,
        subcmd,
    } = secrets;

    let repo = RepoRequest::try_from(&name, &auth_token)?;

    match &subcmd {
        SecretsSubCommand::List => {
            let SecretListResponse {
                total_count,
                secrets,
            } = repo.get_all_secrets().await?;
            let secrets = secrets
                .iter()
                .map(|s| format!("|{}|{}|{}", s.name, s.created_at, s.updated_at))
                .collect::<Vec<_>>()
                .join("\n");
            printmd("## Secrets");
            printmd!("**Total: {}", total_count);
            printmd!(
                r#"|:-:|:-:|:-:
|**Name**|**Created At**|**Updated At**|
|-:|:-:|:-
{}
|-"#,
                secrets
            );
        }
        SecretsSubCommand::Get(SecretsName { name }) => {
            let Secret {
                name,
                created_at,
                updated_at,
            } = repo.get_a_secret(&name).await?;
            printmd!("## Secret");
            printmd!("**Name**:\t{}", name);
            printmd!("**Created At**:\t{}", created_at);
            printmd!("**Updated At**:\t{}", updated_at);
        }
        #[cfg(feature = "secrets-save")]
        SecretsSubCommand::Add(name_value)
        | SecretsSubCommand::Update(name_value)
        | SecretsSubCommand::Edit(name_value) => {
            let SecretsNameValue { name, value } = name_value;
            repo.save_secret(&name, &value).await?;
            println!(
                "{}",
                format!("Secret {} successful!", &subcmd).bold().green()
            );
        }
        SecretsSubCommand::Delete(SecretsName { name }) => {
            repo.delete_a_secret(&name).await?;
            println!("{}", "Secret delete successful!".bold().green());
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "config")]
    {
        if let Some(config::Config { access_token, .. }) = config::get_config().await {
            const GH_ACCESS_TOKEN: &str = "GH_ACCESS_TOKEN";
            if std::env::var(GH_ACCESS_TOKEN).is_err() {
                std::env::set_var(GH_ACCESS_TOKEN, access_token);
            } else {
                eprint!(
                    "{} {}\n{}\n",
                    "warning: ".bold().yellow(),
                    "using $GH_ACCESS_TOKEN from env, ignored token in config file!".yellow(),
                    "run `unset GH_ACCESS_TOKEN` if this is not intentional.".dark_yellow()
                );
            }
        }
    }
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        #[cfg(feature = "login")]
        SubCommand::Login => handle_login().await?,
        #[cfg(feature = "basic-info")]
        SubCommand::Repo(repo) => match repo.subcmd {
            RepoSubCommand::View(repo_view) => handle_repo_view(&repo_view).await?,
            RepoSubCommand::Collaborators(collab) => handle_repo_collaborators(&collab).await?,
        },
        #[cfg(feature = "secrets")]
        SubCommand::Secrets(secrets) => {
            eprint!(
                "{}{}{}{}{}",
                "warning: ".bold().yellow(),
                "[Deprecation] ".bold().dark_yellow(),
                "This command is deprecated \nuse ".dark_yellow(),
                "gh-cli actions secrets <..> ".bold(),
                "instead! \n".dark_yellow()
            );
            handle_actions_secrets(&secrets).await?
        }
        #[cfg(any(feature = "secrets", feature = "workflows"))]
        SubCommand::Actions(actions) => match actions.subcmd {
            #[cfg(feature = "workflows")]
            ActionsSubCommand::Workflows(workflows) => handle_actions_workflows(&workflows).await?,
            #[cfg(feature = "workflows")]
            ActionsSubCommand::Runs(workflow_runs) => {
                handle_actions_workflow_runs(&workflow_runs).await?
            }
            #[cfg(feature = "workflows")]
            ActionsSubCommand::Jobs(workflow_jobs) => {
                handle_actions_workflow_jobs(&workflow_jobs).await?
            }
            #[cfg(feature = "secrets")]
            ActionsSubCommand::Secrets(secrets) => handle_actions_secrets(&secrets).await?,
        },
    }

    Ok(())
}
