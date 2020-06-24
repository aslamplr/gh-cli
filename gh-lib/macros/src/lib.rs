use graphql_client_codegen::{
    generate_module_token_stream, CodegenMode, GraphQLClientCodegenOptions,
};
use proc_macro::TokenStream;
use quote::quote;
use std::path::{Path, PathBuf};
use syn::Token;

const SCHEMA_DOWNLOAD_PATH: &str = "graphql/schema.public.graphql";
const QRAPHQL_QUERY_PATH: &str = "graphql/query";

#[proc_macro]
pub fn include_graphql_queries(_input: TokenStream) -> TokenStream {
    let query_dir = get_query_dir();
    let generated_modules = query_dir
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
    let schema_path = get_schema_download_path();
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

fn get_schema_download_path() -> PathBuf {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    Path::new(&out_dir).join(SCHEMA_DOWNLOAD_PATH)
}

fn get_query_dir() -> PathBuf {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    Path::new(&out_dir).join(QRAPHQL_QUERY_PATH)
}
