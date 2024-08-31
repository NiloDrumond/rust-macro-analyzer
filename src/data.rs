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
#[ts(export)]
pub struct Data {
    // macro_definitions_by_type: MacroDefinitionsByType,
    // macro_invocations_by_type: MacroInvocationsByType,
    macro_definitions_per_repo: Vec<(String, u32)>,
    macro_invocations_per_repo: Vec<(String, u32)>,
    macro_definitions_per_crate: Vec<(String, u32)>,
    macro_invocations_per_crate: Vec<(String, u32)>,
    total_macro_usage: MacroAnalyzis,
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
            let macro_usage = c.macro_usage.clone().expect("Expected crate to have macro_usage by then");
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
            // macro_definitions_by_type,
            // macro_invocations_by_type,
        }
    }
}
