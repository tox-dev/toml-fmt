use common::table::{reorder_table_keys, Tables};

// Group: behavior → format width → wrap → other.
const KEY_ORDER: &[&str] = &[
    "",
    "in-place",
    "recursive",
    "check",
    "diff",
    "black",
    "pep257",
    "non-strict",
    "line-length",
    "wrap-summaries",
    "wrap-descriptions",
    "tab-width",
    "make-summary-multi-line",
    "close-quotes-on-newline",
    "pre-summary-newline",
    "pre-summary-multi-line",
    "pre-summary-space",
    "post-description-blank",
    "force-wrap",
    "line-range",
    "docstring-length",
    "non-cap",
    "exclude",
    "config",
];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.docformatter") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    reorder_table_keys(table, KEY_ORDER);
}
