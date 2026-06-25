use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    "python_version",
    "python_platform",
    "python_interpreter",
    "project_includes",
    "project_excludes",
    "search_path",
    "site_package_path",
    "use_untyped_imports",
    "replace_imports_with_any",
    "ignore_errors_in_generated_code",
    "errors",
];

const SORT_ARRAYS: &[&str] = &[
    "project_includes",
    "project_excludes",
    "search_path",
    "site_package_path",
    "replace_imports_with_any",
];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.pyrefly") else {
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
