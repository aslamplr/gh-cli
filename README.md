# gh-actions-secrets

```
Github Actions Secret - CLI (Unofficial) 0.1.0
Aslam Ahammed A. <aslamplr@gmail.com>
Deal with Github actions secrets

USAGE:
    gh-actions-secrets [OPTIONS] --action <ACTION> --auth_token <PERSONAL_ACCESS_TOKEN> --repo_name <REPO_NAME> --repo_owner <REPO_OWNER>

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

Example - Add new secret to Github actions secrets

```
gh-actions-secrets --auth_token=qwertyuipasdfghjklzxcvbnmlkgsdfg --repo_owner aslamplr --repo_name gh-actions-secrets --action add --secret_key SECRET_KEY --secret_value SECRET_VALUE_XYZ_BLAH_BLAH
```

Example - List all secrets

```
gh-actions-secrets --auth_token=qwertyuipasdfghjklzxcvbnmlkgsdfg --repo_owner aslamplr --repo_name gh-actions-secrets --action list
```

### Requirements for development

- Rust (rustc 1.42.0)

### Run 

```
cargo run -- --help
```

### Build (release)

```
cargo build --release
```

