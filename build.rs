use curl::easy::Easy;
use graphql_client_codegen::{
    generate_module_token_stream, CodegenMode, GraphQLClientCodegenOptions,
};
use std::fs::File;
use std::io::{BufWriter, Write as _};
use std::path::PathBuf;
use syn::Token;

const GH_PUBLIC_SCHEMA_URL: &'static str =
    "https://developer.github.com/v4/public_schema/schema.public.graphql";
const SCHEMA_DOWNLOAD_PATH: &'static str = "graphql/schema.public.graphql";

const OUTPUT_DIRECTORY_PATH: &'static str = "src/graphql/";

// Queries
const REPO_BASIC_INFO: &'static str = "graphql/query/repo_basic_info.graphql";

fn main() {
    println!("cargo:rerun-if-changed={}", REPO_BASIC_INFO);
    download_schema();
    generate_code(&REPO_BASIC_INFO);
}

fn generate_code(query_path: &str) {
    let query_path = PathBuf::from(query_path);
    let mut options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);
    options.set_module_visibility(
        syn::VisPublic {
            pub_token: <Token![pub]>::default(),
        }
        .into(),
    );
    let schema_path = PathBuf::from(SCHEMA_DOWNLOAD_PATH);
    let gen = generate_module_token_stream(query_path.clone(), &schema_path, options)
        .expect("Module token stream generation failed!");

    let gen = quote::quote! {
      type DateTime = String;
      type URI = String;
      #gen
    };

    let generated_code = gen.to_string();

    let query_file_name: ::std::ffi::OsString = query_path
        .file_name()
        .map(ToOwned::to_owned)
        .expect("Failed to find a file name in the provided query path.");

    let dest_file_path: PathBuf = PathBuf::from(OUTPUT_DIRECTORY_PATH)
        .join(query_file_name)
        .with_extension("rs");

    let mut file = File::create(dest_file_path).unwrap();
    let file_write_err_msg = "Unable to write the generated code to file!";
    writeln!(
        file,
        r###"// This is auto-generated using `build.rs` file! Do not modify!

#![allow(clippy::redundant_static_lifetimes)]
#![allow(unknown_lints)]
"###
    )
    .expect(file_write_err_msg);
    write!(file, "{}", generated_code).expect(file_write_err_msg);
}

fn download_schema() {
    // Check if schema file exists in path
    let schema_file_path = PathBuf::from(SCHEMA_DOWNLOAD_PATH);
    if !schema_file_path.exists() {
        // Download the file to the path!
        let f = File::create(&schema_file_path).unwrap();
        let mut writer = BufWriter::new(f);
        let mut easy = Easy::new();
        easy.url(&GH_PUBLIC_SCHEMA_URL).unwrap();
        easy.write_function(move |data| Ok(writer.write(data).unwrap()))
            .unwrap();
        easy.perform().unwrap();
        let response_code = easy.response_code().unwrap();
        if response_code != 200 {
            panic!(
                "Unexpected response code {} for {}",
                response_code, GH_PUBLIC_SCHEMA_URL
            );
        }
    } else {
        // Do nothing!
    }
}
