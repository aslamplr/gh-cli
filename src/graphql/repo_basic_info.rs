/// Auto generated code using `graphql-client` CLI!
pub struct RepoBasicInfoQuery;
type URI = String;
type DateTime = String;
pub mod repo_basic_info_query {
    #![allow(dead_code)]
    pub const OPERATION_NAME: &'static str = "RepoBasicInfoQuery";
    pub const QUERY : & 'static str = "query RepoBasicInfoQuery($name: String!, $owner: String!) {\n  repository(name: $name, owner: $owner) {\n    nameWithOwner\n    description\n    createdAt\n    pushedAt\n    homepageUrl\n    isPrivate\n    isArchived\n    primaryLanguage {\n      name\n    }\n    licenseInfo {\n      name\n    }\n    stargazers {\n      totalCount\n    }\n  }\n}" ;
    use serde::{Deserialize, Serialize};
    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;
    #[doc = "An ISO-8601 encoded UTC date string.\n"]
    type DateTime = super::DateTime;
    #[doc = "An RFC 3986, RFC 3987, and RFC 6570 (level 4) compliant URI string.\n"]
    type URI = super::URI;
    #[derive(Deserialize)]
    #[doc = "Represents a given language found in repositories.\n"]
    pub struct RepoBasicInfoQueryRepositoryPrimaryLanguage {
        #[doc = "The name of the current language.\n"]
        pub name: String,
    }
    #[derive(Deserialize)]
    #[doc = "A repository's open source license\n"]
    pub struct RepoBasicInfoQueryRepositoryLicenseInfo {
        #[doc = "The license full name specified by <https://spdx.org/licenses>\n"]
        pub name: String,
    }
    #[derive(Deserialize)]
    #[doc = "The connection type for User.\n"]
    pub struct RepoBasicInfoQueryRepositoryStargazers {
        #[doc = "Identifies the total count of items in the connection.\n"]
        #[serde(rename = "totalCount")]
        pub total_count: Int,
    }
    #[derive(Deserialize)]
    #[doc = "A repository contains the content for a project.\n"]
    pub struct RepoBasicInfoQueryRepository {
        #[doc = "The repository's name with owner.\n"]
        #[serde(rename = "nameWithOwner")]
        pub name_with_owner: String,
        #[doc = "The description of the repository.\n"]
        pub description: Option<String>,
        #[doc = "Identifies the date and time when the object was created.\n"]
        #[serde(rename = "createdAt")]
        pub created_at: DateTime,
        #[doc = "Identifies when the repository was last pushed to.\n"]
        #[serde(rename = "pushedAt")]
        pub pushed_at: Option<DateTime>,
        #[doc = "The repository's URL.\n"]
        #[serde(rename = "homepageUrl")]
        pub homepage_url: Option<URI>,
        #[doc = "Identifies if the repository is private.\n"]
        #[serde(rename = "isPrivate")]
        pub is_private: Boolean,
        #[doc = "Indicates if the repository is unmaintained.\n"]
        #[serde(rename = "isArchived")]
        pub is_archived: Boolean,
        #[doc = "The primary language of the repository's code.\n"]
        #[serde(rename = "primaryLanguage")]
        pub primary_language: Option<RepoBasicInfoQueryRepositoryPrimaryLanguage>,
        #[doc = "The license associated with the repository\n"]
        #[serde(rename = "licenseInfo")]
        pub license_info: Option<RepoBasicInfoQueryRepositoryLicenseInfo>,
        #[doc = "A list of users who have starred this starrable.\n"]
        pub stargazers: RepoBasicInfoQueryRepositoryStargazers,
    }
    #[derive(Serialize)]
    pub struct Variables {
        pub name: String,
        pub owner: String,
    }
    impl Variables {}
    #[derive(Deserialize)]
    pub struct ResponseData {
        #[doc = "Lookup a given repository by the owner and repository name.\n"]
        pub repository: Option<RepoBasicInfoQueryRepository>,
    }
}
impl graphql_client::GraphQLQuery for RepoBasicInfoQuery {
    type Variables = repo_basic_info_query::Variables;
    type ResponseData = repo_basic_info_query::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: repo_basic_info_query::QUERY,
            operation_name: repo_basic_info_query::OPERATION_NAME,
        }
    }
}
