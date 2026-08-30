pub const KEY_ORDER: &[&str] = &[
    "required-version",
    "target-version",
    "line-length",
    "include",
    "extend-exclude",
    "force-exclude",
    "exclude",
    "skip-string-normalization",
    "skip-magic-trailing-comma",
    "preview",
    "unstable",
    "enable-unstable-feature",
    "fast",
    "workers",
    "color",
    "verbose",
    "quiet",
];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    matches!(key, "target-version" | "enable-unstable-feature")
}
