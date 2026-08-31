// based_on_style sets defaults and column_limit is most-used, so they lead; the rest alphabetizes via the fallback.
pub const KEY_ORDER: &[&str] = &[
    "based_on_style",
    "column_limit",
    "indent_width",
    "continuation_indent_width",
];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(_key: &str) -> bool {
    false
}
