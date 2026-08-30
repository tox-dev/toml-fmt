pub const KEY_ORDER: &[&str] = &[
    "exclude",
    "extend_exclude",
    "ignore",
    "ignore_notebooks",
    "ignore_unused",
    "ignore_obsolete",
    "ignore_missing",
    "ignore_transitive",
    "ignore_misplaced_dev",
    "ignore_definition",
    "ignore_external",
    "per_rule_ignores",
    "known_first_party",
    "requirements_files",
    "requirements_files_dev",
    "package_module_name_map",
    "pep621_dev_dependency_groups",
];

const SORT_ARRAYS: &[&str] = &[
    "exclude",
    "extend_exclude",
    "ignore",
    "ignore_unused",
    "ignore_obsolete",
    "ignore_missing",
    "ignore_transitive",
    "ignore_misplaced_dev",
    "ignore_definition",
    "known_first_party",
    "requirements_files",
    "requirements_files_dev",
    "pep621_dev_dependency_groups",
];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
