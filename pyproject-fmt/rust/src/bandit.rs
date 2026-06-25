use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    "exclude_dirs",
    "targets",
    "tests",
    "skips",
    // plugin sub-tables collapse to dotted keys (assert_used.skips, ...)
    "assert_used",
    "hardcoded_tmp_directory",
    "hardcoded_bind_all_interfaces",
    "any_other_function_with_shell_equals_true",
    "ssl_with_bad_version",
    "ssl_with_bad_defaults",
    "weak_cryptographic_key",
];

// All array values are set semantics (rule IDs, paths, names), so they sort.
const SORT_ARRAYS_EXACT: &[&str] = &["exclude_dirs", "targets", "tests", "skips"];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.bandit") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        let k = key.as_str();
        if SORT_ARRAYS_EXACT.contains(&k) || is_inner_array(k) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}

fn is_inner_array(key: &str) -> bool {
    key.contains('.')
        && (key.ends_with(".skips")
            || key.ends_with(".tmp_dirs")
            || key.ends_with(".no_shell")
            || key.ends_with(".shell")
            || key.ends_with(".subprocess")
            || key.ends_with(".tests"))
}
