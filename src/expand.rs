use crate::{
    cargo::CargoToml, crate_paths::get_repo_path, results::AnalyzisResults, state::ScraperState,
    utils::pretty_print,
};
use chrono::Local;
use futures::future::join_all;
use std::{
    error::Error,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::{fs, io::AsyncWriteExt, sync::Semaphore, task};

const WORKER_POOL_SIZE: usize = 10;

pub async fn expand_crate(path: String) -> Result<(), String> {
    let crate_path = Path::new("./data/repos").join(path);
    let cargo_path = crate_path.join("Cargo.toml");

    let cargo_toml = fs::read_to_string(cargo_path.clone())
        .await
        .map_err(|e| e.to_string())?;
    let cargo_toml: CargoToml =
        toml::from_str(&cargo_toml).map_err(|e| format!("Failed to load Cargo.toml: {}", e))?;

    let output_path = crate_path.join(".macro-expanded.rs");

    let mut command = tokio::process::Command::new("cargo");
    command
        .env("RUSTUP_TOOLCHAIN", "nightly")
        .arg("+nightly")
        .arg("expand")
        .arg("--no-default-features")
        .arg("--manifest-path")
        .arg(&cargo_path);

    if cargo_toml.lib.is_some() {
        command.arg("--lib");
    } else if let Some(bin_entries) = cargo_toml.bin {
        if let Some(first_bin) = bin_entries.first() {
            command.arg("--bin");
            command.arg(first_bin.name.clone());
        }
    }

    let output = command.output().await.map_err(|e| e.to_string())?;

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
    analyzis_results: &mut AnalyzisResults,
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
    let crates = analyzis_results.crates.keys().cloned().collect::<Vec<_>>();

    let tasks: Vec<_> = crates
        .into_iter()
        .map(|path| {
            let semaphore_clone = semaphore.clone();
            let counter_clone = counter.clone();
            let path_string = path.to_string();

            task::spawn(async move {
                let _permit = semaphore_clone
                    .acquire()
                    .await
                    .unwrap_or_else(|_| panic!("Failed to acquire permit"));
                let count = counter_clone.fetch_add(1, Ordering::Relaxed);
                pretty_print("Expanded crates", Some(&count));
                pretty_print("Expanding crate", Some(&path_string));
                expand_crate_task(path_string).await
            })
        })
        .collect();

    let results = join_all(tasks).await;

    for result in results {
        match result {
            Ok((crate_path, expand_result)) => {
                if let Err(err) = expand_result {
                    analyzis_results.update_crate(&crate_path, &mut |crate_analyzis| {
                        crate_analyzis.expanded_count = Some(Err(err.to_string()));
                    });

                    let repo_path = get_repo_path(&crate_path);
                    analyzis_results.update_repo(&repo_path, &mut |repo_analyzis| {
                        if let Some(Err(prev)) = repo_analyzis.expanded_count {
                            repo_analyzis.expanded_count = Some(Err(prev + 1));
                        } else {
                            repo_analyzis.expanded_count = Some(Err(1));
                        }
                    });
                }
            }
            Err(e) => {
                // Handle the join error (e.g., task was cancelled or panicked)
                eprintln!("Task join error: {:?}", e);
            }
        }
    }

    state.expanded_macros_at = Some(Local::now());
    state.save()?;
    analyzis_results.save()?;
    pretty_print("Macros expanded", None);
    Ok(())
}
