use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
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

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.djlint") else {
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
