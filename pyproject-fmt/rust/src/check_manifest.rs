use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

// Arrays are file-glob lists with set semantics, so they sort.
const KEY_ORDER: &[&str] = &["", "ignore", "ignore-bad-ideas", "ignore-default-rules"];
const SORT_ARRAYS: &[&str] = &["ignore", "ignore-bad-ideas"];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.check-manifest") else {
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
