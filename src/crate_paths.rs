use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

use crate::{state::ScraperState, utils::pretty_print};
const CRATE_PATHS_PATH: &str = "./data/crates.ron";

#[derive(Deserialize, Default)]
struct CargoTomlWorkspace {
    members: Vec<String>,
}

#[derive(Deserialize, Default)]
struct CargoToml {
    workspace: Option<CargoTomlWorkspace>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct CratePaths(pub Vec<String>);

impl std::fmt::Debug for CratePaths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, path) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", path)?;
        }
        Ok(())
    }
}
impl<'a> IntoIterator for &'a CratePaths {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl_save_load!(CratePaths, CRATE_PATHS_PATH);

pub fn get_repo_path(crate_path: &str) -> String {
    let path_parts: Vec<&str> = crate_path.split('/').collect();
    let repo_path = &path_parts[0..4].join("/");
    repo_path.to_string()
}

pub fn find_project_crates(root_dir: &std::path::Path) -> CratePaths {
    let mut crate_paths = Vec::new();

    // Read the Cargo.toml file at the root directory
    let cargo_toml_path = root_dir.join("Cargo.toml");
    let cargo_toml = match fs::read_to_string(cargo_toml_path) {
        Ok(content) => content,
        Err(_no_file) => {
            return CratePaths(vec![]);
        }
    };
    let cargo_toml: CargoToml = toml::from_str(&cargo_toml).unwrap_or_default();

    // Check if the root directory is a crate or a workspace
    if cargo_toml.workspace.is_none() {
        // If it's a crate, add its path to the CratePaths vector
        crate_paths.push(root_dir.to_string_lossy().to_string());
    } else {
        // If it's a workspace, read the members field to get the paths to the crates inside that workspace
        if let Some(workspace) = &cargo_toml.workspace {
            for member in &workspace.members {
                if member == "." {
                    for entry in fs::read_dir(root_dir).unwrap() {
                        let entry = entry.unwrap();
                        let path = entry.path();

                        if path.is_dir() {
                            crate_paths.extend(find_project_crates(&path).0);
                        }
                    }
                } else if member.find('*').is_some() {
                    if let Ok(glob) = glob::glob(&format!("{}/{}", root_dir.display(), member)) {
                        for entry in glob {
                            let entry = entry.unwrap();
                            if entry.is_dir() {
                                crate_paths.extend(find_project_crates(entry.as_path()).0);
                            }
                        }
                    }
                } else {
                    crate_paths.extend(find_project_crates(&root_dir.join(member)).0);
                }
            }
        }
    }

    CratePaths(crate_paths)
}

pub fn find_crate_paths(
    state: &mut ScraperState,
    root_dir: &std::path::Path,
) -> Result<CratePaths, Box<dyn Error>> {
    if state.crates_parsed_at.is_some() {
        if let Some(data) = CratePaths::load() {
            pretty_print("Crates already parsed at", Some(&data.0.len()));
            return Ok(data);
        }
    }
    let mut all_crate_paths = CratePaths(Vec::new());

    // Iterate over each subdirectory in the root directory
    for entry in fs::read_dir(root_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // Check if the entry is a directory
        if path.is_dir() {
            // Call find_crate_paths on the subdirectory
            let crate_paths = find_project_crates(&path);
            all_crate_paths.0.extend(crate_paths.0);
        }
    }

    all_crate_paths.save()?;
    state.crates_parsed_at = Some(Local::now());
    pretty_print("Crates found", Some(&all_crate_paths.0.len()));
    Ok(all_crate_paths)
}
