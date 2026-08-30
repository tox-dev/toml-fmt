pub const KEY_ORDER: &[&str] = &[
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

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
