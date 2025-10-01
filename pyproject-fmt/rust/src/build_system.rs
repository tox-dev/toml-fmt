use common::array::{sort_strings, transform};
use common::pep508::Requirement;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::{lexical_cmp, natural_lexical_cmp};

pub fn fix(tables: &Tables, keep_full_version: bool) {
    let table_element = tables.get("build-system");
    if table_element.is_none() {
        return;
    }
    let table = &mut table_element.unwrap().first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| match key.as_str() {
        "requires" => {
            transform(entry, &|s| {
                Requirement::new(s).unwrap().normalize(keep_full_version).to_string()
            });
            sort_strings::<String, _, _>(
                entry,
                |s| Requirement::new(s.as_str()).unwrap().canonical_name(),
                &|lhs, rhs| natural_lexical_cmp(lhs, rhs),
            );
        }
        "backend-path" => {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| lexical_cmp(lhs, rhs));
        }
        _ => {}
    });
    reorder_table_keys(table, &["", "build-backend", "requires", "backend-path"]);
}
