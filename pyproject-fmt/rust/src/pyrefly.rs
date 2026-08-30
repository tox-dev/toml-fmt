// pyrefly spells its options with hyphens; the underscore forms are what older files hold, and both
// are ordered here so a file keeps its shape whichever spelling it uses.
pub const KEY_ORDER: &[&str] = &[
    "python-version",
    "python_version",
    "python-platform",
    "python_platform",
    "python-interpreter-path",
    "python_interpreter",
    "project-includes",
    "project_includes",
    "project-excludes",
    "project_excludes",
    "search-path",
    "search_path",
    "site-package-path",
    "site_package_path",
    "use-untyped-imports",
    "use_untyped_imports",
    "replace-imports-with-any",
    "replace_imports_with_any",
    "ignore-errors-in-generated-code",
    "ignore_errors_in_generated_code",
    "errors",
];

// the paths are searched in the order they are listed, so `search-path` and `site-package-path` say
// what they say by where each entry sits, and pyrefly takes the first replacement rule that matches,
// so a `!` rule exempts what a broader rule below it would otherwise replace
const SORT_ARRAYS: &[&str] = &[
    "project-includes",
    "project_includes",
    "project-excludes",
    "project_excludes",
];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
