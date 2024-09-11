use analyzis::analyze_crates;
use analyzis::calculate_overall;
use clear_cfg::parse_code;
use count_code::{count_crates_code, count_expanded_code};
use crate_paths::find_crate_paths;
use data::Data;
use expand::expand_crates;
use github::clone_repos;
use github::get_most_popular_repos;
use http::start_server;
use results::AnalyzisResults;
use state::ScraperState;
use std::error::Error;
use std::path::Path;
use utils::create_data_folder;

#[macro_use]
mod utils;
mod analyzis;
mod cargo;
mod clear_cfg;
mod count_code;
mod crate_paths;
mod data;
mod error;
mod expand;
mod github;
mod http;
mod results;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    create_data_folder();
    let mut state = ScraperState::load().unwrap_or_default();
    let repos = get_most_popular_repos(&mut state).await?;
    let repos_path = clone_repos(&mut state, &repos).await?;
    let crate_paths = find_crate_paths(&mut state, Path::new(&repos_path))?;
    let mut results =
        AnalyzisResults::load().unwrap_or(AnalyzisResults::from((&crate_paths, &repos)));
    results.save()?;
    // Without expanded:
    analyze_crates(&mut state, &mut results)?;
    // remove whitespace
    // parse_code(&mut state, &crate_paths)?;
    count_crates_code(&mut state, &mut results)?;

    // With expanded:
    // analyze_crates(&mut state, &mut results)?;
    // clear_conditional_compilation(&mut state, &crate_paths)?;
    // count_crates_code(&mut state, &mut results)?;
    // expand_crates(&mut state, &mut results).await?;
    // count_expanded_code(&mut state, &mut results)?;

    let mut data: Data = results.clone().into();
    data.date = state
        .cloned_repos_at
        .expect("Repositories should have been cloned by now");
    let serialized = serde_json::to_string(&data)?;
    let mut file = std::fs::File::create("data/data.json")?;
    std::io::Write::write_all(&mut file, serialized.as_bytes())?;
    results.save()?;
    state.save()?;

    start_server(data).await?;
    Ok(())
}
