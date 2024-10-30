use common::array::{sort, transform};
use common::pep508::{format_requirement, get_canonic_requirement_name};
use common::string::update_content;
use common::table::{collapse_sub_tables, for_entries, reorder_table_keys, Tables};
use common::taplo::syntax::SyntaxKind::{ARRAY, ENTRY, INLINE_TABLE, VALUE};
use common::util::iter;

pub fn fix(tables: &mut Tables, keep_full_version: bool) {
    collapse_sub_tables(tables, "dependency-groups");
    let table_element = tables.get("dependency-groups");
    if table_element.is_none() {
        return;
    }

    let table = &mut table_element.unwrap().first().unwrap().borrow_mut();
    for_entries(table, &mut |_key, entry| {
        // format dependency specifications
        transform(entry, &|s| format_requirement(s, keep_full_version));
        // update include-group key to double-quoted string
        iter(entry, [ARRAY, VALUE, INLINE_TABLE, ENTRY, VALUE].as_ref(), &|node| {
            update_content(node, |s| String::from(s));
        });
        // sort array elements
        sort(entry, |e| {
            get_canonic_requirement_name(e).to_lowercase() + " " + &format_requirement(e, keep_full_version)
        });
    });
    reorder_table_keys(table, &["", "dev", "test", "type", "docs"]);
}
