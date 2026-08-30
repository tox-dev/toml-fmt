// Sub-table order follows the pylint docs (main → messages_control → category checks); keys within each sub-table
// alphabetize, since a hand-curated full order would rot.
pub const KEY_ORDER: &[&str] = &[
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

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    is_sortable_array(key)
}

fn is_sortable_array(key: &str) -> bool {
    // Match the leaf key: identifier/rule-code/module-path lists are all set semantics.
    let leaf = key.rsplit('.').next().unwrap_or(key);
    matches!(
        leaf,
        "enable"
            | "disable"
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
