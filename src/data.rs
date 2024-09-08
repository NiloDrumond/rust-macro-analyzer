use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    analyzis::{DeriveMacroUsage, MacroAnalyzis, MacroUsage},
    results::AnalyzisResults,
};

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
struct MacroDefinitionsByType {
    derive_macros: u32,
    attribute_macros: u32,
    declarative_macros: u32,
    function_macros: u32,
}

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
struct MacroInvocationsByType {
    derive_macros: u32,
    attribute_macros: u32,
    builtin_attribute_macros: u32,
    function_declarative_macros: u32,
}

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
struct DeriveUsage {
    avg: f32,
    median: f32,
    mode: Vec<usize>,
    max: usize,
    sorted_data: Vec<usize>,
}

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
#[ts(export)]
pub struct Data {
    // macro_definitions_by_type: MacroDefinitionsByType,
    // macro_invocations_by_type: MacroInvocationsByType,
    macro_definitions_per_repo: Vec<(String, u32)>,
    macro_invocations_per_repo: Vec<(String, u32)>,
    macro_definitions_per_crate: Vec<(String, u32)>,
    macro_invocations_per_crate: Vec<(String, u32)>,
    derive_usage: DeriveUsage,
    total_macro_usage: MacroAnalyzis,
}

fn calculate_statistics(sorted_data: Vec<usize>) -> DeriveUsage {
    if sorted_data.is_empty() {
        panic!(); // Handle empty input
    }

    // 1. Calculate average (as u32)
    let sum: usize = sorted_data.iter().sum();
    let avg: f32  = (sum as f64 / sorted_data.len() as f64) as f32;

    let median: f32 = if sorted_data.len() % 2 == 0 {
        // Average of the two middle values for even length
        let mid1 = sorted_data[sorted_data.len() / 2 - 1];
        let mid2 = sorted_data[sorted_data.len() / 2];
        (mid1 as f32 + mid2 as f32) / 2.0
    } else {
        // Middle value for odd length
        sorted_data[sorted_data.len() / 2] as f32
    };

    // 3. Calculate mode
    let mut frequency_map: HashMap<usize, usize> = HashMap::new();
    for &num in &sorted_data {
        *frequency_map.entry(num).or_insert(0) += 1;
    }

    let max_frequency = frequency_map.values().cloned().max().unwrap_or(0);
    let mode: Vec<usize> = frequency_map
        .into_iter()
        .filter(|&(_, count)| count == max_frequency)
        .map(|(num, _)| num)
        .collect();

    let max = sorted_data.last().unwrap_or(&0);
    DeriveUsage {
        avg,
        median,
        mode,
        max: *max,
        sorted_data,
    }
}

impl From<AnalyzisResults> for Data {
    fn from(value: AnalyzisResults) -> Self {
        let mut attribute_macro_definitions = MacroUsage(HashMap::new());
        let mut declarative_macro_definitions = MacroUsage(HashMap::new());
        let mut derive_macro_definitions = MacroUsage(HashMap::new());
        let mut procedural_macro_definitions = MacroUsage(HashMap::new());
        let mut attribute_macro_invocations = MacroUsage(HashMap::new());
        let mut builtin_attribute_macro_invocations = MacroUsage(HashMap::new());
        let mut macro_invocations = MacroUsage(HashMap::new());
        let mut derive_macro_usage = DeriveMacroUsage::default();

        let mut macro_invocations_by_type = MacroInvocationsByType::default();
        let mut macro_definitions_by_type = MacroDefinitionsByType::default();

        let mut macro_invocations_per_repo = vec![];
        let mut macro_definitions_per_repo = vec![];

        for (path, repo) in value.repos.iter() {
            let macro_usage = repo
                .macro_usage
                .clone()
                .expect("Expected Repo to have macro_usage by then");
            attribute_macro_definitions =
                attribute_macro_definitions + macro_usage.attribute_macro_definitions.clone();
            declarative_macro_definitions =
                declarative_macro_definitions + macro_usage.declarative_macro_definitions.clone();
            derive_macro_definitions =
                derive_macro_definitions + macro_usage.derive_macro_definitions.clone();
            procedural_macro_definitions =
                procedural_macro_definitions + macro_usage.procedural_macro_definitions.clone();
            attribute_macro_invocations =
                attribute_macro_invocations + macro_usage.attribute_macro_invocations.clone();
            macro_invocations = macro_invocations + macro_usage.macro_invocations.clone();
            builtin_attribute_macro_invocations = builtin_attribute_macro_invocations
                + macro_usage.builtin_attribute_macro_invocations.clone();
            derive_macro_usage = derive_macro_usage + macro_usage.derive_macro_usage.clone();

            let macro_invocations: u32 = (macro_usage.macro_invocations
                + macro_usage.attribute_macro_invocations
                + macro_usage.builtin_attribute_macro_invocations)
                .into();
            macro_invocations_per_repo.push((path.to_string(), macro_invocations));
            let macro_definitions: u32 = (macro_usage.procedural_macro_definitions.clone()
                + macro_usage.attribute_macro_definitions
                + macro_usage.procedural_macro_definitions
                + macro_usage.declarative_macro_definitions)
                .into();
            macro_definitions_per_repo.push((path.to_string(), macro_definitions));
        }

        let mut macro_invocations_per_crate = vec![];
        let mut macro_definitions_per_crate = vec![];
        for (path, c) in value.crates.iter() {
            let macro_usage = c
                .macro_usage
                .clone()
                .expect("Expected crate to have macro_usage by then");
            let macro_invocations: u32 = (macro_usage.macro_invocations
                + macro_usage.attribute_macro_invocations
                + macro_usage.builtin_attribute_macro_invocations)
                .into();
            macro_invocations_per_crate.push((path.to_string(), macro_invocations));
            let macro_definitions: u32 = (macro_usage.procedural_macro_definitions.clone()
                + macro_usage.attribute_macro_definitions
                + macro_usage.procedural_macro_definitions
                + macro_usage.declarative_macro_definitions)
                .into();
            macro_definitions_per_crate.push((path.to_string(), macro_definitions));
        }
        let mut derives_per_invocation: Vec<usize> = vec![];
        for repo in value.repos.values() {
            derives_per_invocation = [
                derives_per_invocation,
                repo.macro_usage
                    .clone()
                    .expect("Expected repo to have macro_usage by then")
                    .derive_macro_usage
                    .derives_per_invocation,
            ]
            .concat();
        }

        derives_per_invocation.sort();
        let derive_usage = calculate_statistics(derives_per_invocation);

        let total_macro_usage = MacroAnalyzis {
            attribute_macro_definitions,
            declarative_macro_definitions,
            procedural_macro_definitions,
            attribute_macro_invocations,
            builtin_attribute_macro_invocations,
            macro_invocations,
            derive_macro_usage,
            derive_macro_definitions,
        };
        Self {
            total_macro_usage,
            macro_invocations_per_repo,
            macro_definitions_per_repo,
            macro_definitions_per_crate,
            macro_invocations_per_crate,
            derive_usage,
            // macro_definitions_by_type,
            // macro_invocations_by_type,
        }
    }
}
