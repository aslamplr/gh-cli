#![cfg(feature = "graphql-api")]
#![allow(clippy::redundant_static_lifetimes)]
pub mod repo_basic_info {
    // Include generated code, see `build.rs`!
    include!(concat!(env!("OUT_DIR"), "/graphql/repo_basic_info.rs"));
}
