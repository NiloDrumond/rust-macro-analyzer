use crate::{
    crate_paths::get_repo_path,
    results::AnalyzisResults,
    state::ScraperState,
    utils::pretty_print,
};
use chrono::Local;
use std::{error::Error, path::Path};
use tokio::{
    io::AsyncWriteExt,
    task::{self, JoinHandle},
};

async fn expand_crate(path: String) -> Result<(), String> {
    let cargo_path = Path::new(&path).join("Cargo.toml");
    let output_path = Path::new(&path).join(".macro-expanded.rs");

    let output = tokio::process::Command::new("cargo")
        .arg("+nightly")
        .arg("expand")
        .arg("--manifest-path")
        .arg(&cargo_path)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }

    let mut file = tokio::fs::File::create(output_path)
        .await
        .map_err(|e| e.to_string())?;
    file.write_all(&output.stdout)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

async fn expand_crate_task(path: String) -> (String, Result<(), String>) {
    let result = expand_crate(path.clone()).await;

    (path, result)
}

pub async fn expand_crates(
    state: &mut ScraperState,
    results: &mut AnalyzisResults,
) -> Result<(), Box<dyn Error>> {
    if state.expanded_macros_at.is_some() {
        pretty_print(
            "Macros already expanded at",
            Some(&state.expanded_macros_at),
        );
        return Ok(());
    }

    let mut handles: Vec<JoinHandle<(String, Result<(), String>)>> = Vec::new();
    for path in results.crates.clone().keys() {
        let handle = task::spawn(expand_crate_task(path.to_string()));
        handles.push(handle);
    }
    for handle in handles {
        match handle.await {
            Ok(result) => {
                if let Err(err) = result.1 {
                    results.update_crate(&result.0, &mut |crate_analyzis| {
                        crate_analyzis.expanded_char_count = Some(Err(err.to_string()));
                    });

                    let repo_path = get_repo_path(&result.0);
                    results.update_repo(&repo_path, &mut |repo_analyzis| {
                        if let Some(Err(prev)) = repo_analyzis.expanded_char_count {
                            repo_analyzis.expanded_char_count = Some(Err(prev + 1));
                        } else {
                            repo_analyzis.expanded_char_count = Some(Err(1));
                        }
                    })
                }
            }
            Err(e) => println!("Task failed to start: {:?}", e),
        }
    }

    state.expanded_macros_at = Some(Local::now());
    state.save()?;
    pretty_print("Macros expanded", None);
    Ok(())
}
