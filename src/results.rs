use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    analyzis::MacroUsageAnalyzis,
    crate_paths::{get_repo_path, CratePaths},
};

const RESULTS_PATH: &str = "./data/analyzis.ron";

type RepoPath = String;
type CratePath = String;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct CrateAnalyzis {
    pub path: CratePath,
    pub char_count: Option<usize>,
    pub expanded_char_count: Option<Result<usize, String>>,
    pub macro_usage: Option<MacroUsageAnalyzis>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct RepoAnalyzis {
    pub path: RepoPath,
    pub crates_count: usize,
    pub char_count: Option<usize>,
    pub expanded_char_count: Option<Result<usize, usize>>,
    pub macro_usage: Option<MacroUsageAnalyzis>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AnalyzisResults {
    pub crates: HashMap<CratePath, CrateAnalyzis>,
    pub repos: HashMap<RepoPath, RepoAnalyzis>,
}

impl From<&CratePaths> for AnalyzisResults {
    fn from(paths: &CratePaths) -> Self {
        let mut crates = HashMap::new();
        let mut repos: HashMap<String, RepoAnalyzis> = HashMap::new();
        for path in paths {
            crates.insert(
                path.to_string(),
                CrateAnalyzis {
                    path: path.to_string(),
                    ..Default::default()
                },
            );
            let repo_path = get_repo_path(path);
            let mut repo_analyzis = match repos.get(&repo_path) {
                Some(repo_analyzis) => repo_analyzis.clone(),
                None => RepoAnalyzis::default(),
            };
            repo_analyzis.path = repo_path.to_string();
            repo_analyzis.crates_count += 1;
            repos.insert(repo_path, repo_analyzis);
        }
        Self { crates, repos }
    }
}

impl AnalyzisResults {
    pub fn update_repo(&mut self, repo_path: &str, update: &mut dyn FnMut(&mut RepoAnalyzis)) {
        let mut repo_analyzis = self
            .repos
            .get(repo_path)
            .unwrap_or_else(|| panic!("Failed to get repo analyzis from repo path {}", repo_path))
            .clone();

        update(&mut repo_analyzis);

        self.repos.insert(repo_path.to_string(), repo_analyzis);
    }

    pub fn update_crate(&mut self, crate_path: &str, update: &mut dyn FnMut(&mut CrateAnalyzis)) {
        let mut crate_analyzis = self
            .crates
            .get(crate_path)
            .unwrap_or_else(|| {
                panic!(
                    "Failed to get crate analyzis from crate path {}",
                    crate_path
                )
            })
            .clone();

        update(&mut crate_analyzis);

        self.crates.insert(crate_path.to_string(), crate_analyzis);
    }
}

impl_save_load!(AnalyzisResults, RESULTS_PATH);
