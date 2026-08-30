pub const KEY_ORDER: &[&str] = &[
    "fail-under",
    "fail_under",
    "ignore-init-method",
    "ignore-init-module",
    "ignore-magic",
    "ignore-semiprivate",
    "ignore-private",
    "ignore-property-decorators",
    "ignore-module",
    "ignore-nested-functions",
    "ignore-nested-classes",
    "ignore-setters",
    "ignore-overloaded-functions",
    "ignore-regex",
    "exclude",
    "extend-exclude",
    "color",
    "verbose",
    "quiet",
    "omit-covered-files",
    "generate-badge",
    "badge-format",
    "badge-style",
];

const SORT_ARRAYS: &[&str] = &["exclude", "extend-exclude", "ignore-regex"];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
