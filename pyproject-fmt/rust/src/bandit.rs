pub const KEY_ORDER: &[&str] = &[
    "exclude_dirs",
    "targets",
    "tests",
    "skips",
    // plugin sub-tables collapse to dotted keys (assert_used.skips, ...)
    "assert_used",
    "hardcoded_tmp_directory",
    "hardcoded_bind_all_interfaces",
    "any_other_function_with_shell_equals_true",
    "ssl_with_bad_version",
    "ssl_with_bad_defaults",
    "weak_cryptographic_key",
];

// All array values are set semantics (rule IDs, paths, names), so they sort.
const SORT_ARRAYS_EXACT: &[&str] = &["exclude_dirs", "targets", "tests", "skips"];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS_EXACT.contains(&key) || is_inner_array(key)
}

fn is_inner_array(key: &str) -> bool {
    key.contains('.')
        && (key.ends_with(".skips")
            || key.ends_with(".tmp_dirs")
            || key.ends_with(".no_shell")
            || key.ends_with(".shell")
            || key.ends_with(".subprocess")
            || key.ends_with(".tests"))
}
