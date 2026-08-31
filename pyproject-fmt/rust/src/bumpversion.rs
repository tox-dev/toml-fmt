pub const KEY_ORDER: &[&str] = &[
    "current_version",
    "parse",
    "serialize",
    "search",
    "replace",
    "regex",
    "ignore_missing_version",
    "ignore_missing_files",
    "tag",
    "sign_tags",
    "tag_name",
    "tag_message",
    "allow_dirty",
    "commit",
    "commit_args",
    "message",
    "moveable_tags",
    "pre_n_label",
    "pre_l_label",
    "files",
    "parts",
];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(_key: &str) -> bool {
    false
}
