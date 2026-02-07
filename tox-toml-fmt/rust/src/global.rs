use tombi_syntax::SyntaxKind::{ARRAY, BASIC_STRING, INLINE_TABLE};
use tombi_syntax::SyntaxNode;

use common::string::load_text;
use common::table::{for_entries, Tables};

fn get_env_list_order(tables: &Tables) -> Vec<String> {
    let mut env_order = Vec::new();

    if let Some(root_tables) = tables.get("") {
        for table_ref in root_tables {
            let table = table_ref.borrow();
            for_entries(&table, &mut |key, entry| {
                if key == "env_list" && entry.kind() == ARRAY {
                    for array_child in entry.children_with_tokens() {
                        if array_child.kind() == BASIC_STRING {
                            let env_name = load_text(&array_child.to_string(), BASIC_STRING);
                            env_order.push(format!("env.{env_name}"));
                        }
                    }
                }
            });
        }
    }

    env_order
}

fn normalize_value(node: &SyntaxNode) {
    for child in node.children_with_tokens() {
        match child.kind() {
            ARRAY => {
                if let Some(array_node) = child.as_node() {
                    normalize_value(array_node);
                }
            }
            INLINE_TABLE => {
                if let Some(table_node) = child.as_node() {
                    normalize_value(table_node);
                }
            }
            _ => {}
        }
    }
}

pub fn normalize_strings(tables: &Tables) {
    for table_ref in &tables.table_set {
        let table = table_ref.borrow();
        for_entries(&table, &mut |_, value_node| {
            normalize_value(value_node);
        });
    }
}

pub fn reorder_tables(root_ast: &SyntaxNode, tables: &Tables) {
    // Build dynamic order based on env_list
    let env_list_order = get_env_list_order(tables);
    let has_env_list = !env_list_order.is_empty();

    // Build the full order: root, env_run_base, then env.* from env_list, then generic env
    let mut order: Vec<&str> = vec!["", "env_run_base"];

    // Convert env_list_order to &str for the order slice
    let env_refs: Vec<&str> = env_list_order.iter().map(|s| s.as_str()).collect();
    order.extend(env_refs);

    // Add generic "env" at the end for any env.* not explicitly listed
    order.push("env");

    // Only use multi_level_prefixes when we have specific env.* entries from env_list
    // Without env_list, all env.* tables should match the generic "env" in ordering
    let multi_level_prefixes: &[&str] = if has_env_list { &["env"] } else { &[] };
    tables.reorder(root_ast, &order, multi_level_prefixes);
}
