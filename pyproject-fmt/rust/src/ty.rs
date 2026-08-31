// Pre-1.0 schema: keep the canonical set small, let unknown keys alphabetize.
pub const KEY_ORDER: &[&str] = &[
    "src.respect-ignore-files",
    "src.include",
    "src.exclude",
    "src.exclude-scripts",
    "src",
    "environment",
    "rules",
    "terminal",
    "overrides",
];

pub const SRC_KEY_ORDER: &[&str] = &["respect-ignore-files", "include", "exclude", "exclude-scripts"];

// `src.exclude` is read the way a gitignore is, where a later `!pattern` takes back what an earlier
// one excluded, so its order is what it says.
const SORT_ARRAYS: &[&str] = &["src.include"];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}

/// Whether the name under `src` holds a list of names.
pub fn sorts_in_src(key: &str) -> bool {
    key == "include"
}
