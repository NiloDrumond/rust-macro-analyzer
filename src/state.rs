use chrono::prelude::*;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Debug;

const STATE_PATH: &str = "./data/state.ron";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ScraperState {
    pub repos_query_at: Option<DateTime<Utc>>,
    pub cloned_repos_at: Option<DateTime<Utc>>,
    pub expanded_macros_at: Option<DateTime<Utc>>,
}

impl_save_load!(ScraperState, STATE_PATH);
