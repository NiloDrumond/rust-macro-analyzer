use std::{fs, path::Path};

use crate::{
    crate_paths::get_repo_path,
    error::{Error, ErrorMessage},
    results::AnalyzisResults,
    state::ScraperState,
};
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser, Tree};

const FOLDERS_TO_IGNORE: [&str; 2] = ["target", "malformed"];

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct DeriveMacroData {
    count: usize,
    avg: f64,
}

impl DeriveMacroData {
    pub fn add_point(&mut self, value: usize) {
        let total_sum = self.avg * self.count as f64 + value as f64;
        self.count += 1;
        self.avg = total_sum / self.count as f64;
    }
}

impl std::ops::Add for DeriveMacroData {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.count == 0 {
            return rhs;
        }
        if rhs.count == 0 {
            return self;
        }
        let new_count = self.count + rhs.count;

        let new_avg =
            ((self.avg * self.count as f64) + (rhs.avg * rhs.count as f64)) / new_count as f64;

        Self {
            count: new_count,
            avg: new_avg,
        }
    }
}

pub struct MacroDefinitionsAnalyzis {}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MacroUsageAnalyzis {
    derive_macro: DeriveMacroData,
    macro_invocation_count: usize,
}

impl std::ops::Add for MacroUsageAnalyzis {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            derive_macro: self.derive_macro + rhs.derive_macro,
            macro_invocation_count: self.macro_invocation_count + rhs.macro_invocation_count,
        }
    }
}

fn count_macro_definitions() {}

fn count_macro_usage(root: Node, bytes: &[u8]) -> Result<MacroUsageAnalyzis, Error> {
    let mut analyzis = MacroUsageAnalyzis::default();
    let mut ignore_next = false;
    for node in root.children(&mut root.walk()) {
        if ignore_next {
            ignore_next = false;
            continue;
        }

        // Invocation of function-like macros and declarative macros
        if node.kind() == "macro_invocation" {
            analyzis.macro_invocation_count += 1;
        }

        if let "attribute_item" = node.kind() {
            let attribute = node.child(2);
            if let Some(attribute) = attribute {
                if let Some(identifier) = attribute.child(0) {
                    let value = &bytes[identifier.byte_range()];
                    let value = String::from_utf8(value.to_vec()).unwrap();
                    // Check if its derive macro
                    if value == "derive" {
                        let token_tree = match attribute.child(1) {
                            Some(token_tree) => token_tree,
                            None => {
                                return Err(Error {
                                    message: ErrorMessage::DeriveMacroExpectedTokenTree,
                                    path: None,
                                });
                            }
                        };
                        let derive_count = token_tree
                            .children(&mut token_tree.walk())
                            .filter(|n| n.kind() == "identifier")
                            .count();
                        analyzis.derive_macro.add_point(derive_count);
                    }
                }
                // Check if its cfg macro
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
        if node.child_count() > 0 {
            let res = count_macro_usage(node, bytes)?;
            analyzis = analyzis + res;
        }
    }

    Ok(analyzis)
}

fn parse_file(bytes: &[u8]) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Error loading Rust grammar");
    parser.parse(bytes, None).expect("Failed to parse file")
}

fn count_dir_macro_usage(path: &Path) -> Result<MacroUsageAnalyzis, Error> {
    let mut analyzis = MacroUsageAnalyzis::default();
    let ignore = FOLDERS_TO_IGNORE.map(std::ffi::OsStr::new);
    for entry in
        fs::read_dir(path).unwrap_or_else(|_| panic!("Failed to read directory {:?}", path))
    {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = entry.file_name();

        if path.is_dir() {
            if ignore.iter().any(|v| *v == name) {
                continue;
            }
            let output = count_dir_macro_usage(&path)?;
            analyzis = analyzis + output;
        }

        let file_name = path.file_name();
        if let Some(file_name) = file_name {
            if file_name != ".macro-expanded.rs"
                && path.extension() == Some(std::ffi::OsStr::new("rs"))
            {
                match fs::read_to_string(&path) {
                    Ok(string) => {
                        let bytes = string.as_bytes();
                        let tree = parse_file(bytes);
                        let result = count_macro_usage(tree.root_node(), bytes)
                            .map_err(|err| err.add_path(path.to_str().unwrap()))?;
                        analyzis = analyzis + result;
                    }
                    Err(e) => {
                        println!("Failed to read file of path {:?}. Error: {}", path, e);
                    }
                }
            }
        }
    }
    Ok(analyzis)
}

pub fn analyze_crates(
    state: &mut ScraperState,
    results: &mut AnalyzisResults,
) -> Result<(), Error> {
    let result = count_dir_macro_usage(Path::new("./data/repos/sharkdp.bat"));
    println!("result: {:?}", result);
    return Ok(());

    let string = fs::read_to_string(Path::new("./test.rs")).unwrap();
    let bytes = string.as_bytes();
    let tree = parse_file(bytes);
    let result = count_macro_usage(tree.root_node(), bytes);
    println!("result: {:?}", result);
    return Ok(());

    for crate_path in results.crates.clone().keys() {
        let repo_path = get_repo_path(crate_path);
        let analyzis = count_dir_macro_usage(Path::new(&repo_path))?;

        results.update_crate(crate_path, &mut |crate_analyzis| {
            crate_analyzis.macro_usage = Some(analyzis.clone());
        });
        results.update_repo(&repo_path, &mut |repo_analyzis| {
            let prev = repo_analyzis.macro_usage.clone().unwrap_or_default();
            repo_analyzis.macro_usage = Some(prev + analyzis.clone());
        })
    }

    // state.counted_expanded_chars_at = Some(Local::now());
    Ok(())
}
