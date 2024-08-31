use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use crate::{
    analyzis::MacroAnalyzis,
    crate_paths::{get_repo_path, CratePaths},
    github::Repository,
};

const RESULTS_PATH: &str = "./data/analyzis.ron";

type RepoPath = String;
type CratePath = String;

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone, Copy)]
// (Characters, Lines)
pub struct CharLineCount(pub usize, pub usize);
impl std::ops::AddAssign for CharLineCount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl std::ops::Add for CharLineCount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
pub struct CrateAnalyzis {
    pub source_count: Option<CharLineCount>,
    pub expanded_count: Option<Result<CharLineCount, String>>,
    pub macro_usage: Option<MacroAnalyzis>,
}

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
pub struct RepoAnalyzis {
    pub path: RepoPath,
    pub crates_count: usize,
    pub source_count: Option<CharLineCount>,
    pub expanded_count: Option<Result<CharLineCount, usize>>,
    pub macro_usage: Option<MacroAnalyzis>,
    pub star_count: i64,
}

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
pub struct OverallAnalyzis {
    pub source_count: Option<CharLineCount>,
    pub macro_usage: Option<MacroAnalyzis>,
}

#[derive(TS, Serialize, Deserialize, Default, Debug)]
#[ts(export)]
pub struct AnalyzisResults {
    pub crates: HashMap<CratePath, CrateAnalyzis>,
    pub repos: HashMap<RepoPath, RepoAnalyzis>,
    pub overall: OverallAnalyzis,
}

impl From<(&CratePaths, &Vec<Repository>)> for AnalyzisResults {
    fn from((paths, repos_query): (&CratePaths, &Vec<Repository>)) -> Self {
        let mut crates = HashMap::new();
        let mut repos: HashMap<String, RepoAnalyzis> = HashMap::new();
        for path in paths {
            crates.insert(
                path.to_string(),
                CrateAnalyzis {
                    ..Default::default()
                },
            );
            let repo_path = get_repo_path(path);
            let mut repo_analyzis = match repos.get(&repo_path) {
                Some(repo_analyzis) => repo_analyzis.clone(),
                None => RepoAnalyzis {
                    star_count: repos_query
                        .iter()
                        .find(|repository| {
                            let folder_name =
                                format!("{}.{}", repository.owner.login, repository.name);
                            folder_name == repo_path
                        })
                        .unwrap()
                        .stargazers
                        .total_count,
                    ..Default::default()
                },
            };
            repo_analyzis.path = repo_path.to_string();
            repo_analyzis.crates_count += 1;
            repos.insert(repo_path, repo_analyzis);
        }
        Self {
            crates,
            repos,
            overall: OverallAnalyzis::default(),
        }
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
