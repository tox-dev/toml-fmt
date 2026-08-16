use common::array::{dedupe_strings, sort_strings};
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

pub const KEY_ORDER: &[&str] = &[
    "",
    "column_width",
    "indent",
    "keep_full_version",
    "generate_python_version_classifiers",
    "max_supported_python",
    "table_format",
    "sub_table_spacing",
    "separate_root_table",
    "expand_tables",
    "collapse_tables",
    "skip_wrap_for_keys",
];

// Consumed as sets: expand/collapse via HashSet::contains, skip_wrap via matches_pattern under .any().
// Element order never reaches the logic, so sorting is display-only and dropping a byte-identical
// duplicate is inert. Dedup stays case-sensitive because those lookups are case-sensitive.
const SORT_ARRAYS: &[&str] = &["expand_tables", "collapse_tables", "skip_wrap_for_keys"];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.pyproject-fmt") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        if SORT_ARRAYS.contains(&key.as_str()) {
            dedupe_strings(entry, str::to_string);
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}
