use common::table::{reorder_table_keys, Tables};

// based_on_style sets defaults and column_limit is most-used, so they lead; the rest
// alphabetizes via the fallback.
const KEY_ORDER: &[&str] = &[
    "",
    "based_on_style",
    "column_limit",
    "indent_width",
    "continuation_indent_width",
];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.yapf") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    reorder_table_keys(table, KEY_ORDER);
}
