use common::table::{reorder_table_keys, Tables};

// Group: identity → format → tag → commit → behavior → [[files]]/[[parts]] (AoT, last).
const KEY_ORDER: &[&str] = &[
    "",
    "current_version",
    "parse",
    "serialize",
    "search",
    "replace",
    "regex",
    "ignore_missing_version",
    "ignore_missing_files",
    "tag",
    "sign_tags",
    "tag_name",
    "tag_message",
    "allow_dirty",
    "commit",
    "commit_args",
    "message",
    "moveable_tags",
    "pre_n_label",
    "pre_l_label",
    "files",
    "parts",
];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.bumpversion") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    reorder_table_keys(table, KEY_ORDER);
}
