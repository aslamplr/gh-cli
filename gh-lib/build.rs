#[cfg(feature = "graphql-api")]
use curl::easy::Easy;
#[cfg(feature = "graphql-api")]
use graphql_client_codegen::{
    generate_module_token_stream, CodegenMode, GraphQLClientCodegenOptions,
};
#[cfg(feature = "graphql-api")]
use std::fs::File;
#[cfg(feature = "graphql-api")]
use std::io::{BufWriter, Write as _};
#[cfg(feature = "graphql-api")]
use std::path::{Path, PathBuf};
#[cfg(feature = "graphql-api")]
use syn::Token;

#[cfg(feature = "graphql-api")]
const GH_PUBLIC_SCHEMA_URL: &str =
    "https://developer.github.com/v4/public_schema/schema.public.graphql";
#[cfg(feature = "graphql-api")]
const SCHEMA_DOWNLOAD_PATH: &str = "graphql/schema.public.graphql";

// Queries
#[cfg(feature = "graphql-api")]
const REPO_BASIC_INFO: &str = "graphql/query/repo_basic_info.graphql";

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "graphql-api")]
    {
        println!("cargo:rerun-if-changed={}", REPO_BASIC_INFO);
        download_schema()?;
        generate_code(&REPO_BASIC_INFO)?;
        println!("cargo:rerun-if-changed=build.rs");
    }
    Ok(())
}

#[cfg(feature = "graphql-api")]
fn generate_code(query_path: &str) -> anyhow::Result<()> {
    let query_path = PathBuf::from(query_path);
    let mut options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);
    options.set_module_visibility(
        syn::VisPublic {
            pub_token: <Token![pub]>::default(),
        }
        .into(),
    );
    options.set_response_derives(String::from("Serialize,PartialEq,Debug"));
    let schema_path = PathBuf::from(SCHEMA_DOWNLOAD_PATH);
    let gen = generate_module_token_stream(query_path.clone(), &schema_path, options)
        .expect("[build.rs] Module token stream generation failed!");

    let gen = quote::quote! {
      #[cfg(feature = "chrono")]
      type DateTime = chrono::DateTime<chrono::Utc>;
      #[cfg(not(feature = "chrono"))]
      type DateTime = String;
      type URI = String;
      #gen
    };

    let generated_code = gen.to_string();

    let query_file_name = query_path
        .file_name()
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            anyhow::anyhow!("[build.rs] Failed to find a file name in the provided query path.")
        })?;

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_dir_path: PathBuf = Path::new(&out_dir).join("graphql");
    if !dest_dir_path.exists() {
        std::fs::create_dir_all(&dest_dir_path)?;
    }
    let dest_file_path = dest_dir_path.join(query_file_name).with_extension("rs");

    let mut file = File::create(dest_file_path)?;
    write!(file, "{}", generated_code)?;
    Ok(())
}

#[cfg(feature = "graphql-api")]
fn download_schema() -> anyhow::Result<()> {
    // Check if schema file exists in path
    let schema_file_path = PathBuf::from(SCHEMA_DOWNLOAD_PATH);
    if !schema_file_path.exists() {
        // Download the file to the path!
        let f = File::create(&schema_file_path)?;
        let mut writer = BufWriter::new(f);
        let mut easy = Easy::new();
        easy.url(&GH_PUBLIC_SCHEMA_URL)?;
        easy.write_function(move |data| {
            Ok(writer
                .write(data)
                .expect("[build.rs][curl] unable to write"))
        })?;
        easy.perform()?;
        let response_code = easy.response_code()?;
        if response_code != 200 {
            panic!(
                "[build.rs][curl] Unexpected response code {} for {}",
                response_code, GH_PUBLIC_SCHEMA_URL
            );
        }
    } else {
        println!(
            "[build.rs][curl] Found {}, Not downloading from internet!",
            SCHEMA_DOWNLOAD_PATH
        );
    }
    Ok(())
}
