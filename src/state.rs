use chrono::prelude::*;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

const STATE_PATH: &str = "./data/state.ron";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ScraperState {
    pub repos_query_at: Option<DateTime<Local>>,
    pub crates_parsed_at: Option<DateTime<Local>>,
    pub cloned_repos_at: Option<DateTime<Local>>,
    pub counted_code_at: Option<DateTime<Local>>,
    pub expanded_macros_at: Option<DateTime<Local>>,
    pub counted_expanded_chars_at: Option<DateTime<Local>>,
    pub cleared_cfg_at: Option<DateTime<Local>>
}

impl_save_load!(ScraperState, STATE_PATH);
