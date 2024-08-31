use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    analyzis::{DeriveMacroUsage, MacroAnalyzis, MacroUsage},
    results::AnalyzisResults,
};

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
struct MacroDefinitionsByType {
    derive_macros: u64,
    attribute_macros: u64,
    declarative_macros: u64,
    function_macros: u64,
}

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
struct MacroInvocationsByType {
    derive_macros: u64,
    attribute_macros: u64,
    builtin_attribute_macros: u64,
    function_declaritive_macros: u64,
}

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone)]
#[ts(export)]
pub struct Data {
    macro_definitions_by_type: MacroDefinitionsByType,
    macro_invocations_by_type: MacroInvocationsByType,
    macro_definitions_per_repo: Vec<(String, u64)>,
    macro_invocations_per_repo: Vec<(String, u64)>,
    macro_definitions_per_crate: Vec<(String, u64)>,
    macro_invocations_per_crate: Vec<(String, u64)>,
    total_macro_usage: MacroAnalyzis,
}

impl From<AnalyzisResults> for Data {
    fn from(value: AnalyzisResults) -> Self {
        let attribute_macro_definitions = MacroUsage(HashMap::new());
        let declarative_macro_definitions = MacroUsage(HashMap::new());
        let derive_macro_definitions = MacroUsage(HashMap::new());
        let procedural_macro_definitions = MacroUsage(HashMap::new());
        let attribute_macro_invocations = MacroUsage(HashMap::new());
        let builtin_attribute_macro_invocations = MacroUsage(HashMap::new());
        let macro_invocations = MacroUsage(HashMap::new());
        let derive_macro_usage = DeriveMacroUsage::default();
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
        todo!()
    }
}
