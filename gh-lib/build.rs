use curl::easy::Easy;
use std::fs::File;
use std::io::{BufWriter, Write as _};
use std::path::Path;

const SCHEMA_DOWNLOAD_PATH: &str = "graphql/schema.public.graphql";
const GH_PUBLIC_SCHEMA_URL: &str =
    "https://developer.github.com/v4/public_schema/schema.public.graphql";
const QRAPHQL_QUERY_PATH: &str = "graphql/query";

fn main() -> anyhow::Result<()> {
    copy_graphql_queries()?;
    download_schema()?;
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn copy_graphql_queries() -> anyhow::Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let query_dest_path = Path::new(&out_dir).join(QRAPHQL_QUERY_PATH);
    if !query_dest_path.exists() {
        std::fs::create_dir_all(&query_dest_path)?;
    }
    Path::new(QRAPHQL_QUERY_PATH)
        .read_dir()
        .unwrap()
        .filter(|p| p.is_ok())
        .map(|p| p.as_ref().unwrap().path())
        .filter(|p| p.extension().unwrap_or_default() == "graphql")
        .for_each(|src| {
            let file_name = src.file_name().unwrap();
            std::fs::copy(&src, query_dest_path.join(file_name)).unwrap();
        });
    Ok(())
}

fn download_schema() -> anyhow::Result<()> {
    // Check if schema file exists in path
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let schema_file_path = Path::new(&out_dir).join(SCHEMA_DOWNLOAD_PATH);
    if !schema_file_path.exists() {
        let parent_dir = schema_file_path.parent().unwrap();
        if !parent_dir.exists() {
            std::fs::create_dir_all(&parent_dir)?;
        }
        // Download the file to the path!
        let f = File::create(&schema_file_path)?;
        let mut writer = BufWriter::new(f);
        let mut easy = Easy::new();
        easy.url(GH_PUBLIC_SCHEMA_URL)?;
        easy.write_function(move |data| {
            Ok(writer
                .write(data)
                .expect("[download_schema][curl] unable to write"))
        })?;
        easy.perform()?;
        let response_code = easy.response_code()?;
        if response_code != 200 {
            panic!(
                "[download_schema][curl] Unexpected response code {} for {}",
                response_code, GH_PUBLIC_SCHEMA_URL
            );
        }
    } else {
        println!(
            "[download_schema][curl] Found {}, Not downloading from internet!",
            SCHEMA_DOWNLOAD_PATH
        );
    }
    Ok(())
}
