use crate::ScraperState;
use crate::{utils::pretty_print, JoinHandle};
use chrono::Utc;
use graphql_client::{GraphQLQuery, Response};
use reqwest::{header, Client};
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    path::Path,
};
use tokio::task;

const GRAPHQL_URL: &str = "https://api.github.com/graphql";
const USER_AGENT: &str = "NiloDrumond (https://github.com/NiloDrumond)";
const REPOS_PATH: &str = "./data/repos.ron";
const CLONED_REPOS_PATH: &str = "./data/repos";
const REPOS_TO_FETCH: i64 = 10;

#[allow(clippy::upper_case_acronyms)]
type URI = String;

type Repository = repos_query::ReposQuerySearchNodesOnRepository;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/github_schema.json",
    query_path = "graphql/repos_query.graphql",
    response_derives = "Debug,Serialize,Deserialize"
)]
pub struct ReposQuery;

impl From<repos_query::ResponseData> for Vec<Repository> {
    fn from(val: repos_query::ResponseData) -> Self {
        val.search
            .nodes
            .unwrap()
            .into_iter()
            .filter_map(|repo| {
                if let Some(repo) = repo {
                    return match repo {
                        repos_query::ReposQuerySearchNodes::Repository(repository) => {
                            Some(repository)
                        }
                        _ => None,
                    };
                }
                None
            })
            .collect()
    }
}

fn load_popular_repos() -> Option<repos_query::ResponseData> {
    let mut state_file = match File::open(REPOS_PATH) {
        Ok(file) => file,
        Err(_) => return None,
    };
    let mut buf: String = String::new();
    if let Err(e) = state_file.read_to_string(&mut buf) {
        println!("Error reading file: {:?}", e);
        return None;
    }
    match ron::from_str::<repos_query::ResponseData>(&buf) {
        Ok(state) => Some(state),
        Err(e) => {
            println!("Error deserializing RON: {:?}", e);
            None
        }
    }
}

async fn fetch_most_popular_repos(
    state: &mut ScraperState,
) -> Result<Vec<Repository>, Box<dyn Error>> {
    let github_token = std::env::var("GITHUB_TOKEN")?;
    let request_body = ReposQuery::build_query(repos_query::Variables {
        qstr: "language:Rust stars:>=1 sort:stars-desc".to_string(),
        first: REPOS_TO_FETCH,
        after: None,
    });
    let client = Client::new()
        .post(GRAPHQL_URL)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("token {}", github_token),
        )
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header(header::USER_AGENT, USER_AGENT)
        .json(&request_body);

    let res = client.send().await?;
    let response_body: Response<repos_query::ResponseData> = res.json().await?;
    let data = response_body.data.unwrap();
    let ron_string = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default())?;
    let mut file = File::create(REPOS_PATH)?;
    state.repos_query_at = Some(Utc::now());
    state.save()?;
    file.write_all(ron_string.as_bytes())?;
    Ok(data.into())
}

pub async fn get_most_popular_repos(
    state: &mut ScraperState,
) -> Result<Vec<Repository>, Box<dyn Error>> {
    if state.repos_query_at.is_some() {
        if let Some(data) = load_popular_repos() {
            let repos: Vec<Repository> = data.into();
            pretty_print("Loaded popular repositories", Some(&repos.len()));
            return Ok(repos);
        }
    }

    let repos = fetch_most_popular_repos(state).await?;
    pretty_print("Fetched popular repositories", Some(&repos.len()));
    Ok(repos)
}

async fn clone_repo(repository: Repository) -> Result<(), Box<dyn Error + Send + Sync>> {
    let output_folder_name = format!("{}.{}", repository.owner.login, repository.name);
    let path = Path::new(CLONED_REPOS_PATH).join(output_folder_name);
    let output = tokio::process::Command::new("git")
        .arg("clone")
        .arg(repository.url)
        .arg(path)
        .stdout(std::io::stdout())
        .output()
        .await?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }
    Ok(())
}

pub async fn clone_repos(
    state: &mut ScraperState,
    repositories: Vec<Repository>,
) -> Result<String, Box<dyn Error>> {
    let mut handles: Vec<JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>> = Vec::new();
    pretty_print("Starting to clone repositores", None);
    for repository in repositories {
        let handle = task::spawn(clone_repo(repository));
        handles.push(handle);
    }
    for handle in handles {
        if let Err(err) = handle.await {
            println!("Failed to clone: {}", err)
        }
    }
    state.cloned_repos_at = Some(Utc::now());
    state.save()?;
    pretty_print("Repositories cloned", None);
    Ok(CLONED_REPOS_PATH.to_string())
}
