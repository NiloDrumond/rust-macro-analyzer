use analyzis::analyze_crates;
use clear_cfg::clear_conditional_compilation;
use count_code::{count_crates_code, count_expanded_code};
use crate_paths::find_crate_paths;
use github::clone_repos;
use github::get_most_popular_repos;
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
mod error;
mod expand;
mod github;
mod results;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    create_data_folder();
    let mut state = ScraperState::load().unwrap_or_default();
    let repos = get_most_popular_repos(&mut state).await?;
    let repos_path = clone_repos(&mut state, repos).await?;
    let crate_paths = find_crate_paths(&mut state, Path::new(&repos_path))?;
    let mut results = AnalyzisResults::load().unwrap_or(AnalyzisResults::from(&crate_paths));
    results.save()?;
    analyze_crates(&mut state, &mut results)?;
    // clear_conditional_compilation(&mut state, &crate_paths)?;
    // count_crates_code(&mut state, &mut results)?;
    // expand_crates(&mut state, &mut results).await?;
    // count_expanded_code(&mut state, &mut results)?;
    results.save()?;
    state.save()?;
    Ok(())
}
