#![allow(clippy::all, warnings)]
pub struct ReposQuery;
pub mod repos_query {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "ReposQuery";
    pub const QUERY : & str = "query ReposQuery($qstr: String!, $first: Int!, $after: String) {\n  search(query: $qstr, type: REPOSITORY, first: $first, after: $after) {\n    repositoryCount\n    nodes {\n      __typename\n      ... on Repository {\n        nameWithOwner\n        owner {\n          __typename\n          login\n        }\n        url\n        stargazers {\n          totalCount\n        }\n      }\n    }\n  }\n}\n" ;
    use super::*;
    use serde::{Deserialize, Serialize};
    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;
    type URI = super::URI;
    #[derive(Serialize)]
    pub struct Variables {
        pub qstr: String,
        pub first: Int,
        pub after: Option<String>,
    }
    impl Variables {}
    #[derive(Deserialize)]
    pub struct ResponseData {
        pub search: ReposQuerySearch,
    }
    #[derive(Deserialize)]
    pub struct ReposQuerySearch {
        #[serde(rename = "repositoryCount")]
        pub repository_count: Int,
        pub nodes: Option<Vec<Option<ReposQuerySearchNodes>>>,
    }
    #[derive(Deserialize)]
    #[serde(tag = "__typename")]
    pub enum ReposQuerySearchNodes {
        App,
        Discussion,
        Issue,
        MarketplaceListing,
        Organization,
        PullRequest,
        Repository(ReposQuerySearchNodesOnRepository),
        User,
    }
    #[derive(Deserialize)]
    pub struct ReposQuerySearchNodesOnRepository {
        #[serde(rename = "nameWithOwner")]
        pub name_with_owner: String,
        pub owner: ReposQuerySearchNodesOnRepositoryOwner,
        pub url: URI,
        pub stargazers: ReposQuerySearchNodesOnRepositoryStargazers,
    }
    #[derive(Deserialize)]
    pub struct ReposQuerySearchNodesOnRepositoryOwner {
        pub login: String,
        #[serde(flatten)]
        pub on: ReposQuerySearchNodesOnRepositoryOwnerOn,
    }
    #[derive(Deserialize)]
    #[serde(tag = "__typename")]
    pub enum ReposQuerySearchNodesOnRepositoryOwnerOn {
        Organization,
        User,
    }
    #[derive(Deserialize)]
    pub struct ReposQuerySearchNodesOnRepositoryStargazers {
        #[serde(rename = "totalCount")]
        pub total_count: Int,
    }
}
impl graphql_client::GraphQLQuery for ReposQuery {
    type Variables = repos_query::Variables;
    type ResponseData = repos_query::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: repos_query::QUERY,
            operation_name: repos_query::OPERATION_NAME,
        }
    }
}
