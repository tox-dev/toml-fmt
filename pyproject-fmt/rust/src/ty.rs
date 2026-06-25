use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

// Pre-1.0 schema: keep the canonical set small, let unknown keys alphabetize.
const KEY_ORDER: &[&str] = &[
    "",
    "src",
    "respect-ignore-files",
    "environment",
    "rules",
    "terminal",
    "overrides",
];

const SORT_ARRAYS: &[&str] = &["src", "src.include", "src.exclude"];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.ty") else {
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
