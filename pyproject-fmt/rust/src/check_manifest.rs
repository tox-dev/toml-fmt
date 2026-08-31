// Arrays are file-glob lists with set semantics, so they sort.
pub const KEY_ORDER: &[&str] = &["ignore", "ignore-bad-ideas", "ignore-default-rules"];
const SORT_ARRAYS: &[&str] = &["ignore", "ignore-bad-ideas"];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
