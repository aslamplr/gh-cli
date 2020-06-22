#![cfg(feature = "graphql-api")]
#![allow(clippy::redundant_static_lifetimes)]

use macros::include_graphql_queries;

include_graphql_queries!(
    "gh-lib/graphql/query",
    "https://developer.github.com/v4/public_schema/schema.public.graphql"
);
