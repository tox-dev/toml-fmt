use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    "builtin",
    "dictionary",
    "ignore-words",
    "ignore-words-list",
    "ignore-regex",
    "ignore-multiline-regex",
    "exclude-file",
    "skip",
    "uri-ignore-words-list",
    "check-filenames",
    "check-hidden",
    "hidden",
    "regex",
    "user-input",
    "write-changes",
    "interactive",
    "enable-colors",
    "disable-colors",
    "count",
    "quiet-level",
    "summary",
];

const SORT_ARRAYS: &[&str] = &[
    "builtin",
    "dictionary",
    "skip",
    "ignore-words-list",
    "uri-ignore-words-list",
];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.codespell") else {
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
