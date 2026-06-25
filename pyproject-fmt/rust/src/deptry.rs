use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    "exclude",
    "extend_exclude",
    "ignore",
    "ignore_notebooks",
    "ignore_unused",
    "ignore_obsolete",
    "ignore_missing",
    "ignore_transitive",
    "ignore_misplaced_dev",
    "ignore_definition",
    "ignore_external",
    "per_rule_ignores",
    "known_first_party",
    "requirements_files",
    "requirements_files_dev",
    "package_module_name_map",
    "pep621_dev_dependency_groups",
];

const SORT_ARRAYS: &[&str] = &[
    "exclude",
    "extend_exclude",
    "ignore",
    "ignore_unused",
    "ignore_obsolete",
    "ignore_missing",
    "ignore_transitive",
    "ignore_misplaced_dev",
    "ignore_definition",
    "known_first_party",
    "requirements_files",
    "requirements_files_dev",
    "pep621_dev_dependency_groups",
];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.deptry") else {
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
