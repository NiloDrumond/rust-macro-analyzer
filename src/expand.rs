use crate::{
    crate_paths::get_repo_path, results::AnalyzisResults, state::ScraperState, utils::pretty_print,
};
use chrono::Local;
use std::{error::Error, path::Path, sync::{atomic::{AtomicUsize, Ordering}, Arc}};
use tokio::{io::AsyncWriteExt, sync::Semaphore};

const WORKER_POOL_SIZE: usize = 20;

async fn expand_crate(path: String) -> Result<(), String> {
    let cargo_path = Path::new(&path).join("Cargo.toml");
    let output_path = Path::new(&path).join(".macro-expanded.rs");

    let output = tokio::process::Command::new("cargo")
        .arg("+nightly")
        .arg("expand")
        // .arg("--no-default-features")
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

    let semaphore = Arc::new(Semaphore::new(WORKER_POOL_SIZE));

    let counter = Arc::new(AtomicUsize::new(0));
    let crates = results.crates.keys().cloned();
    let tasks: Vec<_> = crates
        .map(|path| {
            let semaphore_clone = semaphore.clone();
            let counter_clone = counter.clone();

            async move {
                let _permit = semaphore_clone
                    .acquire()
                    .await
                    .unwrap_or_else(|_| panic!("Failed to acquire permit"));
                let count = counter_clone.fetch_add(1, Ordering::Relaxed);
                pretty_print("Expanded crates", Some(&count));
                pretty_print("Expanding crate", Some(&path.to_string()));
                expand_crate_task(path.to_string()).await
            }
        })
        .collect();

    for task in tasks {
        let result = task.await;
        if let Err(err) = result.1 {
            results.update_crate(&result.0, &mut |crate_analyzis| {
                crate_analyzis.expanded_count = Some(Err(err.to_string()));
            });

            let repo_path = get_repo_path(&result.0);
            results.update_repo(&repo_path, &mut |repo_analyzis| {
                if let Some(Err(prev)) = repo_analyzis.expanded_count {
                    repo_analyzis.expanded_count = Some(Err(prev + 1));
                } else {
                    repo_analyzis.expanded_count = Some(Err(1));
                }
            })
        }
    }

    state.expanded_macros_at = Some(Local::now());
    state.save()?;
    pretty_print("Macros expanded", None);
    Ok(())
}
