use crate_paths::find_crate_paths;
use expand::expand_crates;
use github::clone_repos;
use github::get_most_popular_repos;
use lines_count::count_crates_source_lines;
use state::ScraperState;
use std::path::Path;
use std::error::Error;
use tokio::task::JoinHandle;

#[macro_use]
pub mod utils;
pub mod crate_paths;
pub mod expand;
pub mod github;
pub mod lines_count;
pub mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut state = ScraperState::load().unwrap_or_default();
    let repos = get_most_popular_repos(&mut state).await?;
    let repos_path = clone_repos(&mut state, repos).await?;
    let crate_paths = find_crate_paths(&state, Path::new(&repos_path))?;
    count_crates_source_lines(&state, &crate_paths)?;
    expand_crates(&mut state, &crate_paths).await?;
    state.save()?;
    Ok(())
}
