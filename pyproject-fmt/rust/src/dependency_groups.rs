use std::cmp::Ordering;

use lexical_sort::natural_lexical_cmp;
use tombi_syntax::SyntaxKind::{BASIC_STRING, INLINE_TABLE};

use common::array::{sort, transform};
use common::pep508::Requirement;
use common::string::{get_string_token, load_text};
use common::table::{collapse_sub_tables, find_key, for_entries, reorder_table_keys, Tables};

pub fn fix(tables: &mut Tables, keep_full_version: bool) {
    collapse_sub_tables(tables, "dependency-groups");
    let table_element = tables.get("dependency-groups");
    if table_element.is_none() {
        return;
    }

    let table = &mut table_element.unwrap().first().unwrap().borrow_mut();
    for_entries(table, &mut |_key, entry| {
        transform(entry, &|s| {
            Requirement::new(s).unwrap().normalize(keep_full_version).to_string()
        });

        sort::<(u8, String, String), _, _>(
            entry,
            |node| match node.kind() {
                BASIC_STRING => get_string_token(node).map(|token| {
                    let val = load_text(token.text(), BASIC_STRING);
                    let package_name = Requirement::new(val.as_str()).unwrap().canonical_name();
                    (0, package_name, val)
                }),
                INLINE_TABLE => find_key(node, "include-group").and_then(|n| {
                    get_string_token(&n).map(|token| (1, load_text(token.text(), BASIC_STRING), String::new()))
                }),
                _ => None,
            },
            &|lhs, rhs| {
                let mut res = lhs.0.cmp(&rhs.0);
                if res == Ordering::Equal {
                    res = natural_lexical_cmp(lhs.1.as_str(), rhs.1.as_str());
                    if res == Ordering::Equal {
                        res = natural_lexical_cmp(lhs.2.as_str(), rhs.2.as_str());
                    }
                }
                res
            },
        );
    });

    reorder_table_keys(table, &["", "dev", "test", "type", "docs"]);
}
