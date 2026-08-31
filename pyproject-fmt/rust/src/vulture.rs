pub const KEY_ORDER: &[&str] = &[
    "paths",
    "exclude",
    "ignore_names",
    "ignore_decorators",
    "make_whitelist",
    "min_confidence",
    "sort_by_size",
    "verbose",
];

const SORT_ARRAYS: &[&str] = &["paths", "exclude", "ignore_names", "ignore_decorators"];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
