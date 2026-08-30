pub const KEY_ORDER: &[&str] = &[
    "minimum-version",
    "build-dir",
    "fail",
    "experimental",
    "strict-config",
    "build",
    "cmake",
    "ninja",
    "sdist",
    "wheel",
    "install",
    "editable",
    "logging",
    "messages",
    "metadata",
    "search",
    "generate",
    "overrides",
];

/// `include` and `exclude` are read the way a gitignore is, cmake runs the targets and installs the
/// components in the order they are listed, and `packages` is read into a mapping where the later
/// path wins, so what is left here selects by name alone.
fn is_sortable(key: &str) -> bool {
    let leaf = key.rsplit('.').next().unwrap_or(key);
    matches!(leaf, "files" | "exclude-fields")
}

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    is_sortable(key)
}
