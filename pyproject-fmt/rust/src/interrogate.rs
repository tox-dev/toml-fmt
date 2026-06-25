use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    "fail-under",
    "fail_under",
    "ignore-init-method",
    "ignore-init-module",
    "ignore-magic",
    "ignore-semiprivate",
    "ignore-private",
    "ignore-property-decorators",
    "ignore-module",
    "ignore-nested-functions",
    "ignore-nested-classes",
    "ignore-setters",
    "ignore-overloaded-functions",
    "ignore-regex",
    "exclude",
    "extend-exclude",
    "color",
    "verbose",
    "quiet",
    "omit-covered-files",
    "generate-badge",
    "badge-format",
    "badge-style",
];

const SORT_ARRAYS: &[&str] = &["exclude", "extend-exclude", "ignore-regex"];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.interrogate") else {
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
