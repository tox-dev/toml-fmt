pub const KEY_ORDER: &[&str] = &[
    "tag_format",
    "major_on_zero",
    "allow_zero_version",
    "version_variables",
    "version_toml",
    "version_pattern",
    "version_translator",
    "build_command",
    "build_command_env",
    "no_git_verify",
    "assets",
    "repo_dir",
    "commit_message",
    "commit_author",
    "logging_use_named_masks",
    "exclude_commit_patterns",
    "commit_parser",
    "commit_parser_options",
    "branches",
    "publish",
    "changelog",
    "remote",
];

// each declaration writes in turn and the later one decides what the file ends up holding, and the
// assets are published in order, so only the patterns matched with `any` are a set
const SORT_ARRAYS: &[&str] = &["exclude_commit_patterns"];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
