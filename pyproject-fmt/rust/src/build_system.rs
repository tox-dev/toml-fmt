use common::array::{sort, transform};
use common::pep508::{format_requirement, get_canonic_requirement_name};
use common::table::{for_entries, reorder_table_keys, Tables};

pub fn fix(tables: &Tables, keep_full_version: bool) {
    let table_element = tables.get("build-system");
    if table_element.is_none() {
        return;
    }
    let table = &mut table_element.unwrap().first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| match key.as_str() {
        "requires" => {
            transform(entry, &|s| format_requirement(s, keep_full_version));
            sort(entry, |e| get_canonic_requirement_name(e).to_lowercase());
        }
        "backend-path" => {
            sort(entry, str::to_lowercase);
        }
        _ => {}
    });
    reorder_table_keys(table, &["", "build-backend", "requires", "backend-path"]);
}
