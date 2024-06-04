use analyzis::analyze_crates;
use count_chars::count_crates_characters;
use count_chars::count_expanded_characters;
use crate_paths::find_crate_paths;
use expand::expand_crates;
use github::clone_repos;
use github::get_most_popular_repos;
use results::AnalyzisResults;
use state::ScraperState;
use std::error::Error;
use std::path::Path;
use tokio::task::JoinHandle;

#[macro_use]
mod utils;
mod count_chars;
mod crate_paths;
mod expand;
mod github;
mod lines_count;
mod state;
mod results;
mod analyzis;
mod error;

const DATA_PATH: &str = "./data";

fn create_data_folder() {
    std::fs::create_dir_all(DATA_PATH).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    create_data_folder();
    let mut state = ScraperState::load().unwrap_or_default();
    let repos = get_most_popular_repos(&mut state).await?;
    let repos_path = clone_repos(&mut state, repos).await?;
    let crate_paths = find_crate_paths(&state, Path::new(&repos_path))?;
    let mut results = AnalyzisResults::load().unwrap_or(AnalyzisResults::from(&crate_paths));
    count_crates_characters(&mut state, &mut results)?;
    expand_crates(&mut state, &mut results).await?;
    count_expanded_characters(&mut state, &mut results)?;
    analyze_crates(&mut state, &mut results)?;
    results.save()?;
    state.save()?;
    Ok(())
}
