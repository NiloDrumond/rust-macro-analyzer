use std::{collections::HashMap, fs, path::Path};

use crate::{
    crate_paths::get_repo_path,
    error::{Error, ErrorMessage},
    results::AnalyzisResults,
    state::ScraperState,
    utils::{BUILTIN_ATTRIBUTES, FOLDERS_TO_IGNORE},
};
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser, Tree};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct DeriveMacroUsage {
    count: usize,
    avg: f64,
}

impl DeriveMacroUsage {
    pub fn add_point(&mut self, value: usize) {
        let total_sum = self.avg * self.count as f64 + value as f64;
        self.count += 1;
        self.avg = total_sum / self.count as f64;
    }
}

impl std::ops::Add for DeriveMacroUsage {
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

// https://doc.rust-lang.org/reference/attributes.html#built-in-attributes-index
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct AttributeMacroUsage(HashMap<String, usize>);

impl AttributeMacroUsage {
    fn add_builtin(&mut self, value: &str) -> Option<()> {
        if value.starts_with("rustfmt::") || value.starts_with("clippy::") {
            let prev = self.0.get("rustfmt::").unwrap_or(&0);
            self.0.insert("rustfmt::".to_string(), prev + 1);
            return Some(());
        }
        if BUILTIN_ATTRIBUTES.iter().any(|v| *v == value) {
            let prev = self.0.get(value).unwrap_or(&0);
            self.0.insert(value.to_string(), prev + 1);
            return Some(());
        }
        None
    }
}

impl std::ops::Add for AttributeMacroUsage {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.0.clone(); // Start with a clone of map1

        for (key, value) in rhs.0 {
            // Update the value for the key in the result map, or insert the key-value pair if it doesn't exist
            *result.entry(key).or_insert(0) += value;
        }

        Self(result)
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MacroAnalyzis {
    attribute_macro_definition_count: usize,
    declarative_macro_definition_count: usize,
    procedural_macro_definition_count: usize,
    derive_macro_definition_count: usize,
    derive_macro_usage: DeriveMacroUsage,
    attribute_macro_invocation_count: usize,
    attribute_macro_usage: AttributeMacroUsage,
    macro_invocation_count: usize,
}

impl std::ops::Add for MacroAnalyzis {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            attribute_macro_usage: self.attribute_macro_usage + rhs.attribute_macro_usage,
            declarative_macro_definition_count: self.declarative_macro_definition_count
                + rhs.declarative_macro_definition_count,
            derive_macro_definition_count: self.derive_macro_definition_count
                + rhs.derive_macro_definition_count,
            procedural_macro_definition_count: self.procedural_macro_definition_count
                + rhs.procedural_macro_definition_count,
            attribute_macro_invocation_count: self.attribute_macro_invocation_count
                + rhs.attribute_macro_invocation_count,
            attribute_macro_definition_count: self.attribute_macro_definition_count
                + rhs.attribute_macro_definition_count,
            derive_macro_usage: self.derive_macro_usage + rhs.derive_macro_usage,
            macro_invocation_count: self.macro_invocation_count + rhs.macro_invocation_count,
        }
    }
}

fn count_macro_usage(root: Node, bytes: &[u8]) -> Result<MacroAnalyzis, Error> {
    let mut analyzis = MacroAnalyzis::default();
    let mut ignore_next = false;
    for node in root.children(&mut root.walk()) {
        if ignore_next {
            ignore_next = false;
            continue;
        }

        if node.kind() == "macro_definition" {
            analyzis.declarative_macro_definition_count += 1;
        }

        // Invocation of function-like macros and declarative macros
        if node.kind() == "macro_invocation" {
            analyzis.macro_invocation_count += 1;
        }

        // Handling attributes
        if node.kind() == "attribute_item" {
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

            let is_builtin = analyzis.attribute_macro_usage.add_builtin(&value);
            if is_builtin.is_none() {
                analyzis.attribute_macro_invocation_count += 1;
            }

            // Checking for Attribute Macro definition
            if value == "proc_macro_attribute" {
                analyzis.attribute_macro_definition_count += 1;
            }
            // Checking for Function-like Macro definition
            if value == "proc_macro" {
                analyzis.procedural_macro_definition_count += 1;
            }
            // Checking for Derive Macro definition
            if value == "proc_macro_derive" {
                analyzis.derive_macro_definition_count += 1;
            }
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
                analyzis.derive_macro_usage.add_point(derive_count);
            }
            if value == "cfg" || value == "cfg_attr" {
                // TODO: tratar caso de target_os
                // https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
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

fn count_dir_macro_usage(path: &Path) -> Result<MacroAnalyzis, Error> {
    let mut analyzis = MacroAnalyzis::default();
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
    // let result = count_dir_macro_usage(Path::new("./data/repos/rust-lang.rust/src/tools/rls"));
    // println!("result: {:?}", result);
    // return Ok(());

    // let string = fs::read_to_string(Path::new("./test.rs")).unwrap();
    // let bytes = string.as_bytes();
    // let tree = parse_file(bytes);
    // let result = count_macro_usage(tree.root_node(), bytes);
    // println!("result: {:?}", result);
    // return Ok(());

    for crate_path in results.crates.clone().keys() {
        let repo_path = get_repo_path(crate_path);
        let analyzis = count_dir_macro_usage(Path::new(&crate_path))?;

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
