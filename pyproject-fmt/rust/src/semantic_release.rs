use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
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

const SORT_ARRAYS: &[&str] = &["version_variables", "version_toml", "assets", "exclude_commit_patterns"];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.semantic_release") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        if SORT_ARRAYS.contains(&key.as_str()) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}
