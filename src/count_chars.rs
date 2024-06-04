use chrono::{Local, Utc};
use std::{collections::HashMap, error::Error, fs, path::Path};
use tree_sitter::{Node, Parser};

use crate::{
    crate_paths::get_repo_path, results::AnalyzisResults, state::ScraperState, utils::pretty_print,
};

fn count_chars(root: Node, bytes: &[u8]) -> usize {
    let mut count = 0;
    let mut ignore_next = false;
    for node in root.children(&mut root.walk()) {
        if ignore_next {
            ignore_next = false;
            continue;
        }

        if let "attribute_item" = node.kind() {
            let attribute = node.child(2);
            if let Some(attribute) = attribute {
                let token_tree = attribute.child(1);
                if let Some(token_tree) = token_tree {
                    if token_tree.child_count() > 1 {
                        let identifier = token_tree.child(1).unwrap();
                        let value = &bytes[identifier.byte_range()];
                        let value = String::from_utf8(value.to_vec()).unwrap();
                        if value == "test" || value == "windows" {
                            ignore_next = true;
                            continue;
                        }
                    }
                }
            }
        }

        let range = node.range();
        if range.end_point.row == range.start_point.row {
            let value = &bytes[node.byte_range()];
            let total_chars = value.len();
            let space_chars = value.iter().filter(|&&b| b == b' ').count();
            count += total_chars - space_chars;
        } else if node.child_count() > 0 {
            count += count_chars(node, bytes);
        }
    }

    count
}

pub fn count_file_chars(bytes: &[u8]) -> Result<usize, Box<dyn Error>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Error loading Rust grammar");
    let tree = parser.parse(bytes, None);

    match tree {
        Some(tree) => {
            let count = count_chars(tree.root_node(), bytes);

            Ok(count)
        }
        None => Err("Failed to parse file".into()),
    }
}

pub fn count_dir_chars(path: &Path) -> Result<usize, Box<dyn Error>> {
    let mut chars = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            chars += count_dir_chars(&path)?;
        }

        let file_name = path.file_name();
        if let Some(file_name) = file_name {
            if file_name != ".macro-expanded.rs"
                && path.extension() == Some(std::ffi::OsStr::new("rs"))
            {
                let string = fs::read_to_string(&path)?;
                let bytes = string.as_bytes();
                chars += count_file_chars(bytes)?;
            }
        }
    }
    Ok(chars)
}

pub fn count_crates_characters(
    state: &mut ScraperState,
    results: &mut AnalyzisResults,
) -> Result<(), Box<dyn Error>> {
    if state.counted_chars_at.is_some() {
        pretty_print(
            "Characters already counted at",
            Some(&state.counted_chars_at),
        );
        return Ok(());
    }
    for crate_path in results.crates.clone().keys() {
        match count_dir_chars(Path::new(&crate_path)) {
            Ok(c) => {
                results.update_crate(crate_path, &mut |crate_analyzis| {
                    crate_analyzis.char_count = Some(c);
                });
                let repo_path = get_repo_path(crate_path);
                results.update_repo(&repo_path, &mut |repo_analyzis| {
                    if let Some(count) = repo_analyzis.char_count {
                        repo_analyzis.char_count = Some(count + c);
                    } else {
                        repo_analyzis.char_count = Some(c);
                    }
                })
            }
            Err(e) => {
                println!("Failed to count {} characters: {}", crate_path, e);
            }
        }
    }

    state.counted_chars_at = Some(Local::now());
    Ok(())
}

pub fn count_expanded_characters(
    state: &mut ScraperState,
    results: &mut AnalyzisResults,
) -> Result<(), Box<dyn Error>> {
    if state.counted_expanded_chars_at.is_some() {
        pretty_print(
            "Expanded characters already counted at",
            Some(&state.counted_expanded_chars_at),
        );
        return Ok(());
    }

    for crate_path in results.crates.clone().keys() {
        let repo_path = get_repo_path(crate_path);
        let expanded_path = Path::new(&crate_path).join(".macro-expanded.rs");
        let count = match fs::read_to_string(&expanded_path) {
            Ok(string) => {
                let bytes = string.as_bytes();
                count_file_chars(bytes)?
            }
            Err(_) => 0,
        };
        results.update_crate(crate_path, &mut |crate_analyzis| {
            if crate_analyzis.expanded_char_count.is_none() {
                crate_analyzis.expanded_char_count = Some(Ok(count));
            }
        });
        results.update_repo(
            &repo_path,
            &mut |repo_analyzis| match repo_analyzis.expanded_char_count {
                None => {
                    repo_analyzis.expanded_char_count = Some(Ok(count));
                }
                Some(result) => {
                    if let Ok(c) = result {
                        repo_analyzis.expanded_char_count = Some(Ok(count + c));
                    }
                }
            },
        )
    }

    state.counted_expanded_chars_at = Some(Local::now());
    Ok(())
}
