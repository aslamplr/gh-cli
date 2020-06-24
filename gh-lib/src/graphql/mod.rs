#![cfg(feature = "graphql-api")]
#![allow(clippy::redundant_static_lifetimes)]

use macros::include_graphql_queries;

include_graphql_queries!();
