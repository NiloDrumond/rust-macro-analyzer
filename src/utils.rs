use std::fmt::Debug;

use tree_sitter::{Parser, Tree};

pub fn parse_file(bytes: &[u8]) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Error loading Rust grammar");
    parser.parse(bytes, None).expect("Failed to parse file")
}

pub fn remove_data_prefix(s: &str) -> String {
    let prefixes = ["./data/repos/", "data/repos/"];
    
    for prefix in &prefixes {
        match s.starts_with(prefix) {
            true => {
                return s[prefix.len()..].to_owned();
            }
            false => (),
        }
    }
    
    s.to_owned()
}

macro_rules! impl_save_load {
    ($struct_name:ident, $path:expr) => {
        impl $struct_name {
            pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
                let mut file = std::fs::File::create($path)?;
                let ron_string =
                    ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::default())?;
                std::io::Write::write_all(&mut file, ron_string.as_bytes())?;
                Ok(())
            }

            pub fn load() -> Option<Self> {
                let mut state_file = match std::fs::File::open($path) {
                    Ok(file) => file,
                    Err(_) => return None,
                };
                let mut buf: String = String::new();
                if let Err(e) = std::io::Read::read_to_string(&mut state_file, &mut buf) {
                    println!("Error reading file: {:?}", e);
                    return None;
                }
                match ron::from_str::<Self>(&buf) {
                    Ok(state) => Some(state),
                    Err(e) => {
                        println!("Error deserializing RON: {:?}", e);
                        None
                    }
                }
            }
        }
    };
}

macro_rules! update_field_if_condition {
    ($self:expr, $value:expr, $field_name:ident, $( $condition:expr ),*) => {
        {
            if [$($condition),*].iter().any(|&v| v == $value) {
                $self.$field_name += 1;
                return Some(());
            }
        };
    };
}

pub fn pretty_print(title: &str, description: Option<&dyn Debug>) {
    let mut output = format!("[{}]", title);
    if let Some(description) = description {
        output = format!("{}: {:?}", output, description);
    }
    println!("{}", output);
}

pub const FOLDERS_TO_IGNORE: [&str; 2] = ["target", "malformed"];
pub const BUILTIN_ATTRIBUTES: [&str; 48] = [
    "cfg",
    "cfg_attr",
    "test",
    "ignore",
    "should_panic",
    "derive",
    "automatically_derived",
    "macro_export",
    "macro_use",
    "proc_macro",
    "proc_macro_derive",
    "proc_macro_attribute",
    "allow",
    "deny",
    "warn",
    "forbid",
    "deprecated",
    "must_use",
    "link",
    "link_name",
    "link_ordinal",
    "no_link",
    "repr",
    "crate_type",
    "no_main",
    "export_name",
    "link_section",
    "no_mangle",
    "used",
    "crate_name",
    "inline",
    "cold",
    "no_builtins",
    "target_feature",
    "track_caller",
    "instruction_set",
    "doc",
    "no_std",
    "no_implicit_prelude",
    "path",
    "recursion_limit",
    "type_length_limit",
    "panic_handler",
    "global_allocator",
    "windows_subsystem",
    "feature",
    "non_exhaustive",
    "debugger_visualizer",
];
