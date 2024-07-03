use chrono::Local;
use std::{error::Error, fs, path::Path};
use tree_sitter::{Node, Parser};

use crate::{
    crate_paths::get_repo_path,
    results::{AnalyzisResults, CharLineCount},
    state::ScraperState,
    utils::pretty_print,
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
            let attribute = match attribute {
                Some(attribute) => attribute,
                None => continue,
            };
            let identifier = match attribute.child(0) {
                Some(identifier) => identifier,
                None => continue,
            };
            let value = &bytes[identifier.byte_range()];
            let value = String::from_utf8(value.to_vec()).unwrap();

            // TODO: tratar esse caso:
            // https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg-macro
            if value == "cfg" || value == "cfg_attr" {
                ignore_next = true;
                continue;
            }
        }

        let range = node.range();
        if range.end_point.row == range.start_point.row {
            let value = &bytes[node.byte_range()];
            let total_chars = value.len();
            let space_chars = value.iter().filter(|&&b| b == b' ').count();
            count += total_chars - space_chars;
        } else if node.child_count() > 0 {
            let c = count_chars(node, bytes);
            count += c;
        }
    }

    count
}

pub fn count_file_code(bytes: &[u8]) -> Result<CharLineCount, Box<dyn Error>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Error loading Rust grammar");
    let tree = parser.parse(bytes, None);

    match tree {
        Some(tree) => {
            let line_count = tree.root_node().range().end_point.row;
            let char_count = count_chars(tree.root_node(), bytes);

            Ok(CharLineCount(char_count, line_count))
        }
        None => Err("Failed to parse file".into()),
    }
}

fn count_dir_code(path: &Path) -> Result<CharLineCount, Box<dyn Error>> {
    let mut count = CharLineCount(0, 0);
    for entry in fs::read_dir(path)? {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            count += count_dir_code(&path)?;
            continue;
        }

        let file_name = path.file_name();
        if let Some(file_name) = file_name {
            if file_name != ".macro-expanded.rs"
                && path.extension() == Some(std::ffi::OsStr::new("rs"))
            {
                let string = fs::read_to_string(&path)?;
                let bytes = string.as_bytes();
                let c = count_file_code(bytes)?;
                count += c;
            }
        }
    }
    Ok(count)
}

pub fn count_crates_code(
    state: &mut ScraperState,
    results: &mut AnalyzisResults,
) -> Result<(), Box<dyn Error>> {
    if state.counted_code_at.is_some() {
        pretty_print(
            "Characters and lines already counted at",
            Some(&state.counted_code_at),
        );
        return Ok(());
    }
    println!("ue");
    for crate_path in results.crates.clone().keys() {
        match count_dir_code(&Path::new("./data/parsed_repos").join(crate_path)) {
            Ok(c) => {
                results.update_crate(crate_path, &mut |crate_analyzis| {
                    crate_analyzis.source_count = Some(c);
                });
                let repo_path = get_repo_path(crate_path);
                results.update_repo(&repo_path, &mut |repo_analyzis| {
                    if let Some(count) = repo_analyzis.source_count {
                        repo_analyzis.source_count = Some(count + c);
                    } else {
                        repo_analyzis.source_count = Some(c);
                    }
                })
            }
            Err(e) => {
                println!("Failed to count {}: {}", crate_path, e);
            }
        }
    }

    pretty_print("Characters and lines counted", None);
    state.counted_code_at = Some(Local::now());
    state.save()?;
    results.save()?;
    println!("{:?}", results);
    Ok(())
}

pub fn count_expanded_code(
    state: &mut ScraperState,
    results: &mut AnalyzisResults,
) -> Result<(), Box<dyn Error>> {
    if state.counted_expanded_chars_at.is_some() {
        pretty_print(
            "Expanded characters and lines already counted at",
            Some(&state.counted_expanded_chars_at),
        );
        return Ok(());
    }

    for crate_path in results.crates.clone().keys() {
        let repo_path = get_repo_path(crate_path);
        let expanded_path = Path::new("./data/parsed_repos")
            .join(crate_path)
            .join(".macro-expanded.rs");
        let count = match fs::read_to_string(&expanded_path) {
            Ok(string) => {
                let bytes = string.as_bytes();
                count_file_code(bytes)?
            }
            Err(_) => CharLineCount(0, 0),
        };
        results.update_crate(crate_path, &mut |crate_analyzis| {
            println!("crate_analyzis: {:?}", crate_analyzis);
            if crate_analyzis.expanded_count.is_none() {
                crate_analyzis.expanded_count = Some(Ok(count));
            }
        });
        results.update_repo(
            &repo_path,
            &mut |repo_analyzis| match repo_analyzis.expanded_count {
                None => {
                    repo_analyzis.expanded_count = Some(Ok(count));
                }
                Some(result) => {
                    if let Ok(c) = result {
                        repo_analyzis.expanded_count = Some(Ok(count + c));
                    }
                }
            },
        )
    }

    pretty_print("Expanded characters and lines counted", None);
    state.counted_expanded_chars_at = Some(Local::now());
    results.save()?;
    state.save()?;
    Ok(())
}
