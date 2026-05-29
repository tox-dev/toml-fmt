use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    // top-level
    "minimum-version",
    "build-dir",
    "fail",
    "experimental",
    "strict-config",
    // build
    "build",
    // cmake
    "cmake",
    "ninja",
    // distribution
    "sdist",
    "wheel",
    "install",
    "editable",
    // logging / messages
    "logging",
    "messages",
    // metadata
    "metadata",
    "search",
    // generate (AoT)
    "generate",
    // overrides (AoT)
    "overrides",
];

fn is_sortable(key: &str) -> bool {
    let leaf = key.rsplit('.').next().unwrap_or(key);
    matches!(
        leaf,
        "include" | "exclude" | "packages" | "files" | "args" | "define" | "targets" | "components" | "exclude-fields"
    )
}

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.scikit-build") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        // args/define are CLI argv and cache entries: reordering changes behavior.
        let leaf = key.as_str().rsplit('.').next().unwrap_or(key.as_str());
        if matches!(leaf, "args" | "define") {
            return;
        }
        if is_sortable(key.as_str()) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}
