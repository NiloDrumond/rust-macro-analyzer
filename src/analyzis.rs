use std::{collections::HashMap, fs, path::Path};

use crate::{
    crate_paths::get_repo_path,
    error::{Error, ErrorMessage},
    results::AnalyzisResults,
    state::ScraperState,
    utils::{parse_file, pretty_print, BUILTIN_ATTRIBUTES, FILES_TO_IGNORE, FOLDERS_TO_IGNORE},
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
pub struct DeriveMacroUsage {
    count: usize,
    avg: f64,
    derives: MacroUsage,
}

impl DeriveMacroUsage {
    pub fn add_point(&mut self, derives: Vec<String>) {
        let total_sum = self.avg * self.count as f64 + derives.len() as f64;
        self.count += 1;
        self.avg = total_sum / self.count as f64;
        self.derives.add_multiple(derives);
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
        let new_derives = self.derives + rhs.derives;

        Self {
            count: new_count,
            avg: new_avg,
            derives: new_derives,
        }
    }
}

// https://doc.rust-lang.org/reference/attributes.html#built-in-attributes-index
#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
pub struct MacroUsage(pub HashMap<String, usize>);

impl MacroUsage {
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

    fn add_multiple(&mut self, values: Vec<String>) {
        for value in values.iter() {
            let prev = self.0.get(value).unwrap_or(&0);
            self.0.insert(value.to_string(), prev + 1);
        }
    }
}

impl std::ops::AddAssign<&str> for MacroUsage {
    fn add_assign(&mut self, value: &str) {
        let prev = self.0.get(value).unwrap_or(&0);
        self.0.insert(value.to_string(), prev + 1);
    }
}

impl std::ops::Add for MacroUsage {
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

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
pub struct MacroAnalyzis {
    pub attribute_macro_definitions: MacroUsage,
    pub declarative_macro_definitions: MacroUsage,
    pub procedural_macro_definitions: MacroUsage,
    pub derive_macro_definitions: MacroUsage,
    pub derive_macro_usage: DeriveMacroUsage,
    pub attribute_macro_invocations: MacroUsage,
    pub builtin_attribute_macro_invocations: MacroUsage,
    pub macro_invocations: MacroUsage,
}

impl std::ops::Add for MacroAnalyzis {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            builtin_attribute_macro_invocations: self.builtin_attribute_macro_invocations
                + rhs.builtin_attribute_macro_invocations,
            declarative_macro_definitions: self.declarative_macro_definitions
                + rhs.declarative_macro_definitions,
            derive_macro_definitions: self.derive_macro_definitions + rhs.derive_macro_definitions,
            procedural_macro_definitions: self.procedural_macro_definitions
                + rhs.procedural_macro_definitions,
            attribute_macro_invocations: self.attribute_macro_invocations
                + rhs.attribute_macro_invocations,
            attribute_macro_definitions: self.attribute_macro_definitions
                + rhs.attribute_macro_definitions,
            derive_macro_usage: self.derive_macro_usage + rhs.derive_macro_usage,
            macro_invocations: self.macro_invocations + rhs.macro_invocations,
        }
    }
}

fn find_next_non_macro(node: Node) -> Option<Node> {
    let next = match node.next_sibling() {
        Some(next) => next,
        None => return None,
    };
    if next.kind() == "attribute_item" {
        return find_next_non_macro(next);
    }
    Some(next)
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
            let identifier = match node.child(1) {
                Some(identifier) => identifier,
                None => {
                    return Err(Error {
                        message: ErrorMessage::FailedToFindMacroIdentifier(
                            node.range().start_point,
                        ),
                        path: None,
                    });
                }
            };
            let value = &bytes[identifier.byte_range()];
            let value = String::from_utf8(value.to_vec()).unwrap();
            analyzis.declarative_macro_definitions += &value;
        }

        // Invocation of function-like macros and declarative macros
        if node.kind() == "macro_invocation" {
            let identifier = match node.child(0) {
                Some(identifier) => identifier,
                None => {
                    return Err(Error {
                        message: ErrorMessage::FailedToFindMacroIdentifier(
                            node.range().start_point,
                        ),
                        path: None,
                    });
                }
            };
            let value = &bytes[identifier.byte_range()];
            let value = String::from_utf8(value.to_vec()).unwrap();
            analyzis.macro_invocations += &value;
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

            let is_builtin = analyzis
                .builtin_attribute_macro_invocations
                .add_builtin(&value);
            if is_builtin.is_none() {
                analyzis.attribute_macro_invocations += &value;
            }

            // Checking for Attribute Macro definition
            if value == "proc_macro_attribute" {
                let next = match find_next_non_macro(node) {
                    Some(next) => next,
                    None => continue,
                };
                let identifier = match next
                    .children(&mut next.walk())
                    .find(|n| n.kind() == "identifier")
                {
                    Some(identifier) => identifier,
                    None => {
                        return Err(Error {
                            message: ErrorMessage::FailedToFindMacroIdentifier(
                                node.range().start_point,
                            ),
                            path: None,
                        });
                    }
                };
                let value = &bytes[identifier.byte_range()];
                let value = String::from_utf8(value.to_vec()).unwrap();
                analyzis.attribute_macro_definitions += &value;
            }
            // Checking for Function-like Macro definition
            if value == "proc_macro" {
                let next = match find_next_non_macro(node) {
                    Some(next) => next,
                    None => continue,
                };
                let identifier = match next
                    .children(&mut next.walk())
                    .find(|n| n.kind() == "identifier")
                {
                    Some(identifier) => identifier,
                    None => {
                        return Err(Error {
                            message: ErrorMessage::FailedToFindMacroIdentifier(
                                node.range().start_point,
                            ),
                            path: None,
                        });
                    }
                };
                let value = &bytes[identifier.byte_range()];
                let value = String::from_utf8(value.to_vec()).unwrap();
                analyzis.procedural_macro_definitions += &value;
            }
            // Checking for Derive Macro definition
            if value == "proc_macro_derive" {
                let next = match find_next_non_macro(node) {
                    Some(next) => next,
                    None => continue,
                };
                let identifier = match next
                    .children(&mut next.walk())
                    .find(|n| n.kind() == "identifier")
                {
                    Some(identifier) => identifier,
                    None => {
                        return Err(Error {
                            message: ErrorMessage::FailedToFindMacroIdentifier(
                                node.range().start_point,
                            ),
                            path: None,
                        });
                    }
                };
                let value = &bytes[identifier.byte_range()];
                let value = String::from_utf8(value.to_vec()).unwrap();
                analyzis.derive_macro_definitions += &value;
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
                let derives: Vec<String> = token_tree
                    .children(&mut token_tree.walk())
                    .filter(|n| n.kind() == "identifier")
                    .map(|identifier| {
                        let value = &bytes[identifier.byte_range()];
                        String::from_utf8(value.to_vec()).unwrap()
                    })
                    .collect();
                analyzis.derive_macro_usage.add_point(derives);
            }
        }
        if node.child_count() > 0 {
            let res = count_macro_usage(node, bytes)?;
            analyzis = analyzis + res;
        }
    }

    Ok(analyzis)
}

fn count_dir_macro_usage(path: &Path) -> Result<MacroAnalyzis, Error> {
    let mut analyzis = MacroAnalyzis::default();
    let folders_to_ignore = FOLDERS_TO_IGNORE.map(std::ffi::OsStr::new);
    let files_to_ignore = FILES_TO_IGNORE.map(std::ffi::OsStr::new);
    let entries = fs::read_dir(path).map_err(|_| Error {
        path: Some(path.display().to_string()),
        message: ErrorMessage::FailedToReadDirectory,
    })?;
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = entry.file_name();

        if path.is_dir() {
            if folders_to_ignore.iter().any(|v| *v == name) {
                continue;
            }
            let output = count_dir_macro_usage(&path)?;
            analyzis = analyzis + output;
        }

        let file_name = path.file_name();
        if let Some(file_name) = file_name {
            if files_to_ignore.iter().any(|v| *v == file_name) {
                continue;
            }
            if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("rs")) {
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

pub fn calculate_overall(results: &mut AnalyzisResults) {
    for repo in results.repos.values() {
        results.overall.macro_usage = Some(
            repo.macro_usage
                .clone()
                .expect("Expected repo to have macro_usage by here")
                + results.overall.macro_usage.clone().unwrap_or_default(),
        );
        results.overall.source_count = Some(
            results.overall.source_count.unwrap_or_default()
                + repo
                    .source_count
                    .expect("Expected repo to have source count by here"),
        );
    }
}

pub fn analyze_crates(
    state: &mut ScraperState,
    results: &mut AnalyzisResults,
) -> Result<(), Box<dyn std::error::Error>> {
    if state.analyzed_macros_at.is_some() {
        pretty_print(
            "Macros already analyzed at",
            Some(&state.analyzed_macros_at),
        );
        return Ok(());
    }

    for crate_path in results.crates.clone().keys() {
        let repo_path = get_repo_path(crate_path);
        let analyzis = count_dir_macro_usage(&Path::new("./data/repos").join(crate_path))?;

        results.update_crate(crate_path, &mut |crate_analyzis| {
            crate_analyzis.macro_usage = Some(analyzis.clone());
        });
        results.update_repo(&repo_path, &mut |repo_analyzis| {
            let prev = repo_analyzis.macro_usage.clone().unwrap_or_default();
            repo_analyzis.macro_usage = Some(prev + analyzis.clone());
        })
    }

    state.analyzed_macros_at = Some(Local::now());
    state.save()?;
    pretty_print("Macros analyzed", None);
    Ok(())
}
