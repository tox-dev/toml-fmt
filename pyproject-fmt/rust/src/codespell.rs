pub const KEY_ORDER: &[&str] = &[
    "builtin",
    "dictionary",
    "ignore-words",
    "ignore-words-list",
    "ignore-regex",
    "ignore-multiline-regex",
    "exclude-file",
    "skip",
    "uri-ignore-words-list",
    "check-filenames",
    "check-hidden",
    "hidden",
    "regex",
    "user-input",
    "write-changes",
    "interactive",
    "enable-colors",
    "disable-colors",
    "count",
    "quiet-level",
    "summary",
];

const SORT_ARRAYS: &[&str] = &["builtin", "skip", "ignore-words-list", "uri-ignore-words-list"];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
