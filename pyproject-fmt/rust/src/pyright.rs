use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;
use tombi_syntax::SyntaxElement;

// Shared schema for [tool.pyright] and [tool.basedpyright].
const KEY_ORDER_PRE_REPORTS: &[&str] = &[
    "",
    // Platform / interpreter
    "pythonVersion",
    "pythonPlatform",
    "pythonPath",
    "venv",
    "venvPath",
    "typeshedPath",
    "stubPath",
    // Mode flags
    "typeCheckingMode",
    "strict",
    "failOnWarnings",
    "useLibraryCodeForTypes",
    // Paths
    "include",
    "exclude",
    "ignore",
    "extraPaths",
    // Strict-flavor toggles
    "strictListInference",
    "strictDictionaryInference",
    "strictSetInference",
    "strictParameterNoneValue",
    "enableExperimentalFeatures",
    "enableTypeIgnoreComments",
    "analyzeUnannotatedFunctions",
    "disableBytesTypePromotions",
    "deprecateTypingAliases",
    // Constants
    "defineConstant",
];

// report* rules are inserted between PRE and POST; this block trails them.
const KEY_ORDER_POST_REPORTS: &[&str] = &["executionEnvironments"];

const SORT_ARRAYS: &[&str] = &["include", "exclude", "ignore", "extraPaths", "strict"];

pub fn fix(tables: &mut Tables) {
    for table_name in ["tool.pyright", "tool.basedpyright"] {
        fix_one(tables, table_name);
    }
}

fn fix_one(tables: &mut Tables, table_name: &str) {
    let Some(elements) = tables.get(table_name) else {
        return;
    };
    let order = {
        let table_elements: &Vec<SyntaxElement> = &elements.first().unwrap().borrow();
        build_key_order(table_elements)
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        if SORT_ARRAYS.contains(&key.as_str()) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    let refs: Vec<&str> = order.iter().map(String::as_str).collect();
    reorder_table_keys(table, &refs);
}

/// Build a per-input KEY_ORDER. Pre-report keys are static; report* rules are collected
/// from the input table and inserted alphabetized between the static blocks. This handles
/// pyright's 70+ diagnostic rules plus any basedpyright extensions without hardcoding
/// the full list (which evolves between releases).
fn build_key_order(table: &[SyntaxElement]) -> Vec<String> {
    use tombi_syntax::SyntaxKind::{KEYS, KEY_VALUE};
    let mut order: Vec<String> = KEY_ORDER_PRE_REPORTS.iter().map(|s| (*s).to_string()).collect();

    let mut report_keys: Vec<String> = Vec::new();
    let key_names = table
        .iter()
        .filter(|e| e.kind() == KEY_VALUE)
        .filter_map(|element| element.as_node())
        .filter_map(|kv| kv.children().find(|c| c.kind() == KEYS))
        .map(|keys| {
            let raw = keys.text().to_string().trim().to_string();
            raw.split('.').next().unwrap_or(&raw).trim_matches('"').to_string()
        });
    for name in key_names {
        if name.starts_with("report") && !report_keys.contains(&name) {
            report_keys.push(name);
        }
    }
    report_keys.sort_by(|a, b| natural_lexical_cmp(&a.to_lowercase(), &b.to_lowercase()));
    order.extend(report_keys);

    order.extend(KEY_ORDER_POST_REPORTS.iter().map(|s| (*s).to_string()));
    order
}
