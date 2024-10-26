use common::array::{sort, transform};
use common::pep508::{format_requirement, get_canonic_requirement_name};
use common::table::{collapse_sub_tables, for_entries, reorder_table_keys, Tables};

pub fn fix(tables: &mut Tables, keep_full_version: bool) {
    collapse_sub_tables(tables, "dependency-groups");
    let table_element = tables.get("dependency-groups");
    if table_element.is_none() {
        return;
    }

    let table = &mut table_element.unwrap().first().unwrap().borrow_mut();
    for_entries(table, &mut |_key, entry| {
        transform(entry, &|s| format_requirement(s, keep_full_version));
        sort(entry, |e| {
            get_canonic_requirement_name(e).to_lowercase() + " " + &format_requirement(e, keep_full_version)
        });
    });
    reorder_table_keys(
        table,
        &[
            "",
            "dev",
            "test",
            "docs",
        ],
    );
}
