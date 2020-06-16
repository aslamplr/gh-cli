use curl::easy::Easy;
use graphql_client_codegen::{
    generate_module_token_stream, CodegenMode, GraphQLClientCodegenOptions,
};
use proc_macro::TokenStream;
use quote::quote;
use std::fs::File;
use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};
use syn::parse::{self, Parse, ParseStream};
use syn::{parse_macro_input, LitStr, Token};

const SCHEMA_DOWNLOAD_PATH: &str = "gh-lib/graphql/schema.public.graphql";

#[derive(Debug)]
struct MacroArgs {
    query_dir: String,
    schema_url: String,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let query_dir = input.parse::<LitStr>()?.value();
        input.parse::<Token![,]>()?;
        let schema_url = input.parse::<LitStr>()?.value();
        Ok(Self {
            query_dir,
            schema_url,
        })
    }
}

#[proc_macro]
pub fn include_graphql_queries(input: TokenStream) -> TokenStream {
    let MacroArgs {
        query_dir,
        schema_url,
    } = parse_macro_input!(input as MacroArgs);
    download_schema(&schema_url).expect("Failed to download GraphQL schema!");
    let generated_modules = Path::new(&query_dir)
        .read_dir()
        .unwrap()
        .filter(|p| p.is_ok())
        .map(|p| p.as_ref().unwrap().path())
        .filter(|p| p.extension().unwrap_or_default() == "graphql")
        .map(|p| {
            let file_stem = p.file_stem().unwrap();
            let module_name = syn::Ident::new(
                file_stem.to_str().unwrap_or_default(),
                proc_macro2::Span::call_site(),
            );

            let query_module = generate_query_module(&p);

            quote! {
                pub mod #module_name {
                    #query_module
                }
            }
        });

    (quote! {
        #(#generated_modules)*
    })
    .into()
}

fn generate_query_module(query_path: &PathBuf) -> proc_macro2::TokenStream {
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
        .expect("[generate_query] Module token stream generation failed!");

    quote! {
      #[cfg(feature = "chrono")]
      type DateTime = chrono::DateTime<chrono::Utc>;
      #[cfg(not(feature = "chrono"))]
      type DateTime = String;
      type URI = String;
      #gen
    }
}

fn download_schema(schema_url: &str) -> anyhow::Result<()> {
    // Check if schema file exists in path
    let schema_file_path = PathBuf::from(SCHEMA_DOWNLOAD_PATH);
    if !schema_file_path.exists() {
        // Download the file to the path!
        let f = File::create(&schema_file_path)?;
        let mut writer = BufWriter::new(f);
        let mut easy = Easy::new();
        easy.url(schema_url)?;
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
                response_code, schema_url
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
