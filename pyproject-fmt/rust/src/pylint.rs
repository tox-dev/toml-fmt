use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

// Sub-table order follows the pylint docs (main → messages_control → category checks); keys within each sub-table
// alphabetize, since a hand-curated full order would rot.
const KEY_ORDER: &[&str] = &[
    "",
    "main",
    "master", // legacy alias of `main`
    "messages_control",
    "messages control", // historic ini-style key name
    "reports",
    "basic",
    "format",
    "design",
    "classes",
    "exceptions",
    "imports",
    "logging",
    "method_args",
    "refactoring",
    "similarities",
    "spelling",
    "string",
    "typecheck",
    "variables",
    "miscellaneous",
];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.pylint") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        if is_sortable_array(key.as_str()) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}

fn is_sortable_array(key: &str) -> bool {
    // Match the leaf key: identifier/rule-code/module-path lists are all set semantics.
    let leaf = key.rsplit('.').next().unwrap_or(key);
    matches!(
        leaf,
        "enable"
            | "disable"
            | "load-plugins"
            | "extension-pkg-allow-list"
            | "extension-pkg-whitelist"
            | "ignore"
            | "ignore-patterns"
            | "ignore-paths"
            | "ignored-modules"
            | "ignored-classes"
            | "ignored-argument-names"
            | "good-names"
            | "bad-names"
            | "init-import"
            | "logging-modules"
            | "valid-classmethod-first-arg"
            | "valid-metaclass-classmethod-first-arg"
            | "callbacks"
            | "additional-builtins"
            | "allowed-redefined-builtins"
            | "dummy-variables-rgx"
            | "exclude-too-few-public-methods"
            | "preferred-modules"
            | "deprecated-modules"
            | "known-third-party"
            | "known-standard-library"
            | "allowed-modules"
            | "expected-line-ending-format"
            | "overgeneral-exceptions"
            | "defining-attr-methods"
            | "exclude-protected"
            | "valid-class-attribute-rgx"
    )
}
