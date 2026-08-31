pub const KEY_ORDER: &[&str] = &[
    "profile",
    "extension",
    "include",
    "exclude",
    "extend_exclude",
    "use_gitignore",
    "indent",
    "indent_css",
    "indent_js",
    "max_attribute_length",
    "max_blank_lines",
    "max_line_length",
    "preserve_blank_lines",
    "preserve_leading_space",
    "blank_line_after_tag",
    "blank_line_before_tag",
    "line_break_after_multiline_tag",
    "close_void_tags",
    "no_function_formatting",
    "no_set_formatting",
    "no_line_after_yaml",
    "format_attribute_template_tags",
    "format_css",
    "format_js",
    "custom_blocks",
    "custom_html",
    "lint",
    "reformat",
    "statistics",
    "require_pragma",
    "ignore_case",
    "ignore_blocks",
    "ignore",
    "per_file_ignores",
    "quiet",
];

const SORT_ARRAYS: &[&str] = &[
    "exclude",
    "extend_exclude",
    "custom_blocks",
    "custom_html",
    "ignore",
    "ignore_blocks",
];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
