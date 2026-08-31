pub const KEY_ORDER: &[&str] = &[
    "max_line_length",
    "indent_size",
    "in-place",
    "recursive",
    "diff",
    "list-fixes",
    "ignore",
    "select",
    "exclude",
    "hang-closing",
    "aggressive",
    "experimental",
    "pep8_passes",
    "max_doc_length",
    "global-config",
    "ignore-local-config",
    "verbose",
];

const SORT_ARRAYS: &[&str] = &["ignore", "select", "exclude"];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
