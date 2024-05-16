use std::{collections::HashMap, error::Error, io::BufRead, path::Path};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncWriteExt,
    task::{self, JoinHandle},
};

use crate::{crate_paths::CratePaths, state::ScraperState, utils::pretty_print};

const MACRO_EXPAND_PATH: &str = "./data/macro_expand.ron";

#[derive(Serialize, Deserialize)]
struct MacroExpandResults(HashMap<String, Result<usize, String>>);
impl_save_load!(MacroExpandResults, MACRO_EXPAND_PATH);

async fn expand_crate(path: String) -> Result<usize, String> {
    let cargo_path = Path::new(&path).join("Cargo.toml");
    let output_path = Path::new(&path).join(".macro-expanded.rs");
    let output = tokio::process::Command::new("cargo")
        .arg("expand")
        .arg("--manifest-path")
        .arg(&cargo_path)
        .output()
        .await.map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }

    let mut file = tokio::fs::File::create(output_path).await.map_err(|e| e.to_string())?;
    file.write_all(&output.stdout).await.map_err(|e| e.to_string())?;

    Ok(output.stdout.lines().count())
}

async fn expand_crate_and_count(path: String) -> (String, Result<usize, String>) {
    let result = expand_crate(path.clone()).await;

    (path, result)
}

pub async fn expand_crates(
    state: &mut ScraperState,
    crate_paths: &CratePaths,
) -> Result<(), Box<dyn Error>> {
    if state.expanded_macros_at.is_some() && MacroExpandResults::load().is_some() {
        pretty_print(
            "Macros already expanded at",
            Some(&state.expanded_macros_at),
        );
        return Ok(());
    }

    let mut handles: Vec<JoinHandle<(String, Result<usize, String>)>> = Vec::new();
    let mut results: MacroExpandResults = MacroExpandResults(HashMap::new());
    for path in crate_paths {
        let handle = task::spawn(expand_crate_and_count(path.to_string()));
        handles.push(handle);
    }
    for handle in handles {
        match handle.await {
            Ok(result) => {
                results.0.insert(result.0, result.1);
            },
            Err(e) => println!("Task failed to start: {:?}", e),
        }
    }

    state.expanded_macros_at = Some(Utc::now());
    state.save()?;
    results.save()?;
    pretty_print("Macros expanded", None);
    Ok(())
}
