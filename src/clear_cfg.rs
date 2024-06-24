use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{self, BufReader, Read, Write},
    path::Path,
};
use tree_sitter::{Node, Parser};

use crate::{
    results::AnalyzisResults,
    state::ScraperState,
    utils::{pretty_print, remove_data_prefix},
};

const RANGES_TO_REMOVE_PATH: &str = "./data/cfg_ranges.ron";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Range(usize, usize);

impl From<tree_sitter::Range> for Range {
    fn from(value: tree_sitter::Range) -> Self {
        Self(value.start_byte, value.end_byte)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RangesToRemove(pub HashMap<String, Vec<Range>>);

impl_save_load!(RangesToRemove, RANGES_TO_REMOVE_PATH);

fn get_node_cfg_ranges(root: Node, bytes: &[u8]) -> Vec<Range> {
    let mut ranges_to_remove: Vec<Range> = vec![];
    let mut ignore_next = false;
    for node in root.children(&mut root.walk()) {
        if ignore_next {
            ranges_to_remove.push(node.range().into());
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
                ranges_to_remove.push(node.range().into());
                continue;
            }
        }

        if node.child_count() > 0 {
            let ranges = get_node_cfg_ranges(node, bytes);
            ranges_to_remove.extend(ranges)
        }
    }

    ranges_to_remove
}

pub fn get_file_cfg_ranges(bytes: &[u8]) -> Result<Vec<Range>, Box<dyn Error>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Error loading Rust grammar");
    let tree = parser.parse(bytes, None);

    match tree {
        Some(tree) => {
            let ranges_to_remove = get_node_cfg_ranges(tree.root_node(), bytes);

            Ok(ranges_to_remove)
        }
        None => Err("Failed to parse file".into()),
    }
}

fn get_cfg_ranges(
    path: &Path,
    ranges_to_remove: &mut RangesToRemove,
) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(path)? {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            get_cfg_ranges(&path, ranges_to_remove)?;
            continue;
        }

        let file_name = path.file_name();
        if let Some(file_name) = file_name {
            if file_name != ".macro-expanded.rs"
                && path.extension() == Some(std::ffi::OsStr::new("rs"))
            {
                let string = fs::read_to_string(&path)?;
                let bytes = string.as_bytes();
                let ranges = get_file_cfg_ranges(bytes)?;
                let parsed_path = remove_data_prefix(path.to_str().unwrap());
                ranges_to_remove.0.insert(parsed_path, ranges);
            }
        }
    }
    Ok(())
}

fn get_crates_cfg_ranges(results: &AnalyzisResults) -> Result<RangesToRemove, Box<dyn Error>> {
    let mut ranges_to_remove = RangesToRemove::load().unwrap_or_default();
    for crate_path in results.crates.clone().keys() {
        match get_cfg_ranges(Path::new(&crate_path), &mut ranges_to_remove) {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to count {}: {}", crate_path, e);
            }
        }
    }

    ranges_to_remove.save()?;
    pretty_print("Characters and lines counted", None);
    Ok(ranges_to_remove)
}

fn remove_file_ranges(file_path: &Path, ranges: &[Range]) -> io::Result<()> {
    let file = fs::File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    // Remove byte ranges
    for range in ranges.iter().rev() {
        content = format!("{}{}", &content[..range.0], &content[range.1..]);
    }

    // Clean up empty lines
    let mut lines: Vec<&str> = content.split('\n').collect();
    lines.retain(|line| !line.trim().is_empty());
    content = lines.join("\n");

    let mut writer = fs::File::create(file_path)?;
    writer.write_all(content.as_bytes())?;

    Ok(())
}

fn process_directory(
    source: &Path,
    dest: &Path,
    ranges_to_remove: &RangesToRemove,
) -> io::Result<()> {
    // Create the destination directory if it doesn't exist
    fs::create_dir_all(dest)?;

    // Iterate over items in the source directory
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(src_path.file_name().unwrap());

        // Check if the item is a file or a directory
        if src_path.is_file() {
            fs::copy(&src_path, &dest_path)?;
            // Copy the file to the destination directory
            if src_path.extension() == Some(std::ffi::OsStr::new("rs")) {
                let parsed_path = remove_data_prefix(src_path.to_str().unwrap());
                if let Some(ranges) = ranges_to_remove.0.get(&parsed_path) {
                    remove_file_ranges(&dest_path, ranges)?;
                }
            }
        } else if src_path.is_dir() {
            // Recursively copy directories
            process_directory(&src_path, &dest_path, ranges_to_remove)?;
        }
    }

    Ok(())
}

fn parse_repositories(ranges_to_remove: &RangesToRemove) -> Result<(), Box<dyn Error>> {
    let source_dir = Path::new("data/repos");
    let dest_dir = Path::new("data/parsed_repos");

    process_directory(source_dir, dest_dir, &ranges_to_remove)?;

    Ok(())
}

pub fn clear_conditional_compilation(
    state: &mut ScraperState,
    results: &AnalyzisResults,
) -> Result<(), Box<dyn Error>> {
    if state.cleared_cfg_at.is_some() {
        pretty_print(
            "Repositories already parsed at",
            Some(&state.expanded_macros_at),
        );
        return Ok(());
    }
    let ranges = get_crates_cfg_ranges(results)?;
    parse_repositories(&ranges);

    pretty_print("Copied repos while clearing conditional compilation", None);
    state.cleared_cfg_at = Some(Local::now());
    state.save()?;
    Ok(())
}