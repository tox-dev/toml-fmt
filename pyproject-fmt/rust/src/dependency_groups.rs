use common::array::{sort, transform};
use common::pep508::Requirement;
use common::string::{load_text, update_content};
use common::table::{collapse_sub_tables, find_key, for_entries, reorder_table_keys, Tables};
use common::taplo::syntax::SyntaxKind::{ARRAY, ENTRY, INLINE_TABLE, STRING, VALUE};
use common::util::iter;
use lexical_sort::natural_lexical_cmp;
use std::cmp::Ordering;

pub fn fix(tables: &mut Tables, keep_full_version: bool) {
    collapse_sub_tables(tables, "dependency-groups");
    let table_element = tables.get("dependency-groups");
    if table_element.is_none() {
        return;
    }

    let table = &mut table_element.unwrap().first().unwrap().borrow_mut();
    for_entries(table, &mut |_key, entry| {
        // format dependency specifications
        transform(entry, &|s| {
            Requirement::new(s).unwrap().normalize(keep_full_version).to_string()
        });

        // update inline table values to double-quoted string, e.g. include-group
        iter(entry, [ARRAY, VALUE, INLINE_TABLE, ENTRY, VALUE].as_ref(), &|node| {
            update_content(node, |s| String::from(s));
        });

        // sort array elements
        sort::<(u8, String, String), _, _>(
            entry,
            |node| {
                for child in node.children_with_tokens() {
                    match child.kind() {
                        STRING => {
                            let val = load_text(child.as_token().unwrap().text(), STRING);
                            let package_name = Requirement::new(val.as_str()).unwrap().canonical_name();
                            return Some((0, package_name, val));
                        }
                        INLINE_TABLE => {
                            match find_key(child.as_node().unwrap(), "include-group") {
                                None => {}
                                Some(n) => {
                                    return Some((
                                        1,
                                        load_text(n.first_token().unwrap().text(), STRING),
                                        String::from(""),
                                    ));
                                }
                            };
                        }
                        _ => {}
                    }
                }
                None
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
