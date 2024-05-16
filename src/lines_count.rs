use crate::{crate_paths::CratePaths, state::ScraperState, utils::pretty_print};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs::{self},
    io::BufRead,
    path::{Path, PathBuf},
};

const LINES_COUNT_PATH: &str = "./data/lines_count.ron";

#[derive(Deserialize, Serialize, Default, Debug)]
struct LinesCount(HashMap<String, usize>);

impl LinesCount {
    pub fn total(&self) -> usize {
        self.0.values().sum()
    }
}

impl_save_load!(LinesCount, LINES_COUNT_PATH);

fn count_source_lines(path: &PathBuf) -> Result<usize, Box<dyn Error>> {
    let mut lines = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            lines += count_source_lines(&path)?;
        }

        if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("rs")) {
            let file = fs::File::open(&path).unwrap();
            let reader = std::io::BufReader::new(file);
            for _ in reader.lines() {
                lines += 1;
            }
        }
    }

    Ok(lines)
}

pub fn count_crates_source_lines(
    state: &ScraperState,
    crate_paths: &CratePaths,
) -> Result<(), Box<dyn Error>> {
    if state.repos_query_at.is_some() {
        if let Some(data) = LinesCount::load() {
            pretty_print("Lines of code before macro expansion", Some(&data.total()));
            return Ok(());
        }
    }
    let mut count = HashMap::new();
    for crate_path in crate_paths.0.iter() {
        let path = Path::new(&crate_path);
        match count_source_lines(&path.into()) {
            Ok(c) => {
                count.insert(crate_path.to_string(), c);
            }
            Err(_) => {
                pretty_print("Failed to count source lines", Some(&path));
            }
        }
    }

    let count = LinesCount(count);
    count.save()?;
    pretty_print("Lines of code before macro expansion", Some(&count.total()));
    Ok(())
}
