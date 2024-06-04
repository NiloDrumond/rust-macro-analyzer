use std::{fs, path::Path};

use tree_sitter::{Node, Parser};

// const SOURCE_PATH: &str = "./data/repos/FuelLabs.fuel-core/crates/trace/src/lib.rs";
const SOURCE_PATH: &str = "/home/nilo/dev/tcc/rust-macros-analyzer/test.rs";
// const SOURCE_PATH: &str = "/home/nilo/dev/tcc/rust-macros-analyzer/data/repos/rustdesk.rustdesk/libs/virtual_display/dylib/src/lib.rs";
const EXPANDED_PATH: &str = "./data/repos/FuelLabs.fuel-core/crates/trace/.macro-expanded.rs";

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

fn count(path: Path) -> usize {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Error loading Rust grammar");
    let string = fs::read_to_string(path).unwrap();
    let bytes = string.as_bytes();
    let tree = parser.parse(bytes, None).unwrap();

    let count = count_chars(tree.root_node(), bytes);

    count
}

fn _count_lines(path: String) {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Error loading Rust grammar");
    let string = fs::read_to_string(path).unwrap();
    let bytes = string.as_bytes();
    let tree = parser.parse(bytes, None).unwrap();

    let mut line_count = 0;
    let mut cursor = tree.root_node().walk();
    let mut ignore_next = false;
    for node in tree.root_node().children(&mut cursor) {
        println!("{:?} {}", node, line_count);
        if !ignore_next {
            line_count += node.range().end_point.row - node.range().start_point.row + 1;
            ignore_next = false;
        }
        println!("{:?} {}", node, line_count);
        if let "attribute_item" = node.kind() {
            let attribute = node.child(2).unwrap();
            for child in attribute.children(&mut attribute.walk()) {
                if let "token_tree" = child.kind() {
                    let identifier = child.child(1).unwrap();
                    let value = &bytes[identifier.byte_range()];
                    let value = String::from_utf8(value.to_vec()).unwrap();
                    if value == "test" || value == "windows" {
                        ignore_next = true;
                        line_count -= 1;
                    }
                }
            }
        }
    }

    println!("line count: {}", line_count)
}

// pub fn test_parser() {
//     let count = count(SOURCE_PATH.to_string());
//     println!("{:?}", count);
// }
