use common::arrays::sort_names_in;
use common::sections;
use lexical_sort::natural_lexical_cmp;
use toml_doc::Document;

// Shared schema for [tool.pyright] and [tool.basedpyright].
pub const KEY_ORDER_PRE_REPORTS: &[&str] = &[
    "pythonVersion",
    "pythonPlatform",
    "pythonPath",
    "venv",
    "venvPath",
    "typeshedPath",
    "stubPath",
    "typeCheckingMode",
    "strict",
    "failOnWarnings",
    "useLibraryCodeForTypes",
    "include",
    "exclude",
    "ignore",
    "extraPaths",
    "strictListInference",
    "strictDictionaryInference",
    "strictSetInference",
    "strictParameterNoneValue",
    "enableExperimentalFeatures",
    "enableTypeIgnoreComments",
    "analyzeUnannotatedFunctions",
    "disableBytesTypePromotions",
    "deprecateTypingAliases",
    "defineConstant",
];

// report* rules are inserted between PRE and POST; this block trails them.
const KEY_ORDER_POST_REPORTS: &[&str] = &["executionEnvironments"];

// `extraPaths` is the order the roots are searched in, so what it says depends on where each one
// sits; the rest only select files.
const SORT_ARRAYS: &[&str] = &["include", "exclude", "ignore", "strict"];

pub fn fix(document: &mut Document<'_>) {
    for table_name in ["tool.pyright", "tool.basedpyright"] {
        fix_one(document, table_name);
    }
}

fn fix_one(document: &mut Document<'_>, table_name: &str) {
    let path = sections::parse_name(table_name);
    let mut names: Vec<String> = Vec::new();
    sections::for_names_under(document, &path, |tail, _| names.push(sections::dotted_name(tail)));
    let order = key_order_of(&names);
    sections::for_keys_under(document, &path, |key, value| {
        if SORT_ARRAYS.contains(&key) {
            sort_names_in(value);
        }
    });
    let refs: Vec<&str> = order.iter().map(String::as_str).collect();
    sections::reorder_under(document, &path, &refs);
}

/// report* rules are collected from the input and inserted alphabetized between the static pre/post blocks, so
/// pyright's 70+ diagnostic rules and any basedpyright extensions need no hardcoded list (it evolves between releases).
pub fn build_key_order(entries: &[toml_doc::Entry<'_>]) -> Vec<String> {
    let names: Vec<String> = entries
        .iter()
        .map(|entry| common::sections::dispatch_name(&entry.key_value.key))
        .collect();
    key_order_of(&names)
}

/// The same, read from the names the table holds however the file wrote them.
fn key_order_of(names: &[String]) -> Vec<String> {
    let mut order: Vec<String> = KEY_ORDER_PRE_REPORTS.iter().map(|name| (*name).to_string()).collect();

    let mut report_keys: Vec<String> = Vec::new();
    let key_names = names
        .iter()
        .map(|path| path.split('.').next().unwrap_or(path).to_owned());
    for name in key_names {
        if name.starts_with("report") && !report_keys.contains(&name) {
            report_keys.push(name);
        }
    }
    report_keys.sort_by(|a, b| natural_lexical_cmp(&a.to_lowercase(), &b.to_lowercase()));
    order.extend(report_keys);

    order.extend(KEY_ORDER_POST_REPORTS.iter().map(|name| (*name).to_string()));
    order
}
