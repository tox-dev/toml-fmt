use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    "max_line_length",
    "indent_size",
    "in-place",
    "recursive",
    "diff",
    "list-fixes",
    "ignore",
    "select",
    "exclude",
    "hang-closing",
    "aggressive",
    "experimental",
    "pep8_passes",
    "max_doc_length",
    "global-config",
    "ignore-local-config",
    "verbose",
];

const SORT_ARRAYS: &[&str] = &["ignore", "select", "exclude"];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.autopep8") else {
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
