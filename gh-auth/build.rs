use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let success_html = fs::read_to_string("success.html")?;
    let (client_id, client_secret) = {
        let client_id = std::env::var("GH_OAUTH_CLIENT_ID");
        let client_secret = std::env::var("GH_OAUTH_CLIENT_SECRET");
        match std::env::var("PROFILE").unwrap().as_ref() {
            "release" => (
                client_id.expect("Required to set env $GH_OAUTH_CLIENT_ID"),
                client_secret.expect("Required to set env $GH_OAUTH_CLIENT_SECRET"),
            ),
            _ => (
                client_id.unwrap_or_default(),
                client_secret.unwrap_or_default(),
            ),
        }
    };

    let generated_code = quote::quote! {
      pub const OAUTH_HOST: &str = "github.com";
      const SUCCESS_HTML: &str = #success_html;
      pub const OAUTH_CLIENT_ID: &str = #client_id;
      const OAUTH_CLIENT_SECRET: &str = #client_secret;
    };
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_file_path: PathBuf = Path::new(&out_dir).join("constants").with_extension("rs");
    let mut file = fs::File::create(dest_file_path)?;
    write!(file, "{}", generated_code.to_string())?;
    println!("cargo:rerun-if-env-changed=GH_OAUTH_CLIENT_ID");
    println!("cargo:rerun-if-env-changed=GH_OAUTH_CLIENT_SECRET");
    println!("cargo:rerun-if-changed=success.html");
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
