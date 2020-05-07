# gh-cli 
named `gh-actions-secrets` earlier, renamed to `gh-cli`.

[🗃 » Download the latest release «](https://github.com/aslamplr/gh-cli/releases)


```
$ gh-cli help
GitHub CLI v0.3.0
Aslam Ahammed A. <aslamplr@gmail.com>
Yet another unofficial GitHub CLI!
Minimalistic, opinionated, and unofficial by default.
Work is in progress to add more subcommands.
Absolute No Warranty!

USAGE:
    gh-cli <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    secrets    GitHub Actions Secrets CLI wrapper for GitHub Actions Secrets API
```

## Sub Commands

### Secrets

```
$ gh-cli secrets
gh-cli-secrets v0.3.0
Aslam Ahammed A. <aslamplr@gmail.com>
GitHub Actions Secrets CLI wrapper for GitHub Actions Secrets API

USAGE:
    gh-cli secrets [OPTIONS] --auth_token <PERSONAL_ACCESS_TOKEN> --repo_owner <REPO_OWNER> --repo_name <REPO_NAME> --action <ACTION>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --repo_owner <REPO_OWNER>               Repository owner
    -n, --repo_name <REPO_NAME>                 Repository name
    -t, --auth_token <PERSONAL_ACCESS_TOKEN>    Generate token - https://github.com/settings/tokens
    -a, --action <ACTION>                        [possible values: list, get, add, update, delete]
        --secret_key <SECRET_KEY>               
        --secret_value <SECRET_VALUE>   
```

#### Example

**Add new secret to Github actions secrets**

```
gh-cli secrets --auth_token=qwertyuipasdfghjklzxcvbnmlkgsdfg --repo_owner aslamplr --repo_name gh-actions-secrets --action add --secret_key SECRET_KEY --secret_value SECRET_VALUE_XYZ_BLAH_BLAH
```

**List all secrets**

```
gh-cli secrets --auth_token=qwertyuipasdfghjklzxcvbnmlkgsdfg --repo_owner aslamplr --repo_name gh-actions-secrets --action list
```

## Development
### Requirements

- Rust (rustc 1.43.0)

### Run 

```
cargo run -- --help
```

### Build (release)

```
cargo build --release
```

## Roadmap
- Blazing fast Unofficial Github CLI implemented in Rust 
- Rust client library for Github API
