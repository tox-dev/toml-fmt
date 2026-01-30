use common::string::load_text;
use common::table::{for_entries, Tables};
use common::taplo::rowan::SyntaxNode;
use common::taplo::syntax::Lang;
use common::taplo::syntax::SyntaxKind::{ARRAY, STRING, VALUE};

/// Extract environment names from env_list array in the root table
fn get_env_list_order(tables: &Tables) -> Vec<String> {
    let mut env_order = Vec::new();

    if let Some(root_tables) = tables.get("") {
        for table_ref in root_tables {
            let table = table_ref.borrow();
            for_entries(&table, &mut |key, entry| {
                if key == "env_list" {
                    // Iterate over the array to extract environment names
                    for child in entry.children_with_tokens() {
                        if child.kind() == ARRAY {
                            for array_child in child.as_node().unwrap().children_with_tokens() {
                                if array_child.kind() == VALUE {
                                    for value_child in array_child.as_node().unwrap().children_with_tokens() {
                                        if value_child.kind() == STRING {
                                            let env_name = load_text(value_child.as_token().unwrap().text(), STRING);
                                            env_order.push(format!("env.{env_name}"));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    env_order
}

pub fn reorder_tables(root_ast: &SyntaxNode<Lang>, tables: &Tables) {
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
