use common::array::{sort, sort_strings, transform};
use common::create::{make_array, make_array_entry, make_comma, make_entry_of_string, make_newline};
use common::pep508::Requirement;
use common::string::{load_text, update_content};
use common::table::{collapse_sub_tables, for_entries, reorder_table_keys, Tables};
use common::taplo::syntax::SyntaxKind::{
    ARRAY, BRACKET_END, BRACKET_START, COMMA, ENTRY, IDENT, INLINE_TABLE, KEY, NEWLINE, STRING, VALUE,
};
use common::taplo::syntax::{SyntaxElement, SyntaxNode};
use common::taplo::util::StrExt;
use common::taplo::HashSet;
use lexical_sort::natural_lexical_cmp;
use regex::Regex;
use std::cell::RefMut;
use std::cmp::Ordering;

pub fn fix(
    tables: &mut Tables,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    min_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
) {
    collapse_sub_tables(tables, "project");
    let table_element = tables.get("project");
    if table_element.is_none() {
        return;
    }
    let table = &mut table_element.unwrap().first().unwrap().borrow_mut();
    let re = Regex::new(r" \.(\W)").unwrap();
    expand_entry_points_inline_tables(table);
    for_entries(table, &mut |key, entry| match key.split('.').next().unwrap() {
        "name" => {
            update_content(entry, |s| Requirement::new(s).unwrap().canonical_name());
        }
        "version" | "readme" | "license-files" | "scripts" | "entry-points" | "gui-scripts" => {
            update_content(entry, |s| String::from(s));
        }
        "description" => {
            update_content(entry, |s| {
                re.replace_all(
                    &s.trim()
                        .lines()
                        .map(|part| {
                            part.split_whitespace()
                                .filter(|part| !part.trim().is_empty())
                                .collect::<Vec<&str>>()
                                .join(" ")
                        })
                        .collect::<Vec<String>>()
                        .join(" "),
                    ".$1",
                )
                .to_string()
            });
        }
        "requires-python" => {
            update_content(entry, |s| s.split_whitespace().collect());
        }
        "dependencies" | "optional-dependencies" => {
            transform(entry, &|s| {
                Requirement::new(s).unwrap().normalize(keep_full_version).to_string()
            });
            sort::<(String, String), _, _>(
                entry,
                |node| {
                    for child in node.children_with_tokens() {
                        if let STRING = child.kind() {
                            let val = load_text(child.as_token().unwrap().text(), STRING);
                            let package_name = Requirement::new(val.as_str()).unwrap().canonical_name();
                            return Some((package_name, val));
                        }
                    }
                    None
                },
                &|lhs, rhs| {
                    let mut res = natural_lexical_cmp(lhs.0.as_str(), rhs.0.as_str());
                    if res == Ordering::Equal {
                        res = natural_lexical_cmp(lhs.1.as_str(), rhs.1.as_str());
                    }
                    res
                },
            );
        }
        "dynamic" | "keywords" => {
            transform(entry, &|s| String::from(s));
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        "classifiers" => {
            transform(entry, &|s| String::from(s));
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        _ => {}
    });

    generate_classifiers(
        table,
        max_supported_python,
        min_supported_python,
        generate_python_version_classifiers,
    );

    for_entries(table, &mut |key, entry| {
        if key.as_str() == "classifiers" {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(
        table,
        &[
            "",
            "name",
            "version",
            "description",
            "readme",
            "keywords",
            "license",
            "license-files",
            "maintainers",
            "authors",
            "requires-python",
            "classifiers",
            "dynamic",
            "dependencies",
            // these go at the end as they may be inline or exploded
            "optional-dependencies",
            "urls",
            "scripts",
            "gui-scripts",
            "entry-points",
        ],
    );
}

fn expand_entry_points_inline_tables(table: &mut RefMut<Vec<SyntaxElement>>) {
    let (mut to_insert, mut count, mut key) = (Vec::<SyntaxElement>::new(), 0, String::new());
    for s_table_entry in table.iter() {
        count += 1;
        if s_table_entry.kind() == ENTRY {
            let mut has_inline_table = false;
            for s_in_table in s_table_entry.as_node().unwrap().children_with_tokens() {
                if s_in_table.kind() == KEY {
                    key = s_in_table.as_node().unwrap().text().to_string().trim().to_string();
                } else if key.starts_with("entry-points.") && s_in_table.kind() == VALUE {
                    for s_in_value in s_in_table.as_node().unwrap().children_with_tokens() {
                        if s_in_value.kind() == INLINE_TABLE {
                            has_inline_table = true;
                            for s_in_inline_table in s_in_value.as_node().unwrap().children_with_tokens() {
                                if s_in_inline_table.kind() == ENTRY {
                                    let mut with_key = String::new();
                                    for s_in_entry in s_in_inline_table.as_node().unwrap().children_with_tokens() {
                                        if s_in_entry.kind() == KEY {
                                            for s_in_key in s_in_entry.as_node().unwrap().children_with_tokens() {
                                                if s_in_key.kind() == IDENT {
                                                    with_key = load_text(s_in_key.as_token().unwrap().text(), IDENT);
                                                    with_key = String::from(with_key.strip_quotes());
                                                    break;
                                                }
                                            }
                                        } else if s_in_entry.kind() == VALUE {
                                            for s_in_b_value in s_in_entry.as_node().unwrap().children_with_tokens() {
                                                if s_in_b_value.kind() == STRING {
                                                    let value =
                                                        load_text(s_in_b_value.as_token().unwrap().text(), STRING);
                                                    if to_insert.last().unwrap().kind() != NEWLINE {
                                                        to_insert.push(make_newline());
                                                    }
                                                    let new_key = format!("{key}.{with_key}");
                                                    let got = make_entry_of_string(&new_key, &value);
                                                    to_insert.push(got);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !has_inline_table {
                to_insert.push(s_table_entry.clone());
            }
        } else {
            to_insert.push(s_table_entry.clone());
        }
    }
    table.splice(0..count, to_insert);
}

fn generate_classifiers(
    table: &mut RefMut<Vec<SyntaxElement>>,
    max_supported_python: (u8, u8),
    min_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
) {
    if generate_python_version_classifiers {
        let (min, max, omit, classifiers) =
            get_python_requires_with_classifier(table, max_supported_python, min_supported_python);
        match classifiers {
            None => {
                let entry = make_array("classifiers");
                generate_classifiers_to_entry(entry.as_node().unwrap(), min, max, &omit, &HashSet::new());
                table.push(entry);
            }
            Some(c) => {
                let mut key_value = String::new();
                for table_row in table.iter() {
                    if table_row.kind() == ENTRY {
                        for entry in table_row.as_node().unwrap().children_with_tokens() {
                            if entry.kind() == KEY {
                                key_value = entry.as_node().unwrap().text().to_string().trim().to_string();
                            } else if entry.kind() == VALUE && key_value == "classifiers" {
                                generate_classifiers_to_entry(table_row.as_node().unwrap(), min, max, &omit, &c);
                            }
                        }
                    }
                }
            }
        };
    } else {
        for_entries(table, &mut |key, entry| {
            if key.as_str() == "classifiers" {
                for child in entry.children_with_tokens() {
                    if child.kind() == ARRAY {
                        let orig = child.as_node().unwrap().children_with_tokens().collect::<Vec<_>>();
                        let mut new_children: Vec<SyntaxElement> = Vec::new();
                        let mut at = 0;
                        while at < orig.len() {
                            let node = &orig[at];
                            if node.kind() == VALUE {
                                // determine if this VALUE is a Python classifier
                                let mut is_python = false;
                                for inner in node.as_node().unwrap().children_with_tokens() {
                                    if inner.kind() == STRING {
                                        let txt = load_text(inner.as_token().unwrap().text(), STRING);
                                        if txt.starts_with("Programming Language :: Python :: 3")
                                            || txt.starts_with("Programming Language :: Python :: Implementation")
                                        {
                                            is_python = true;
                                            break;
                                        }
                                    }
                                }
                                if is_python {
                                    // skip this VALUE and also skip following comma/newline if present
                                    at += 1;
                                    if at < orig.len() && orig[at].kind() == COMMA {
                                        at += 1;
                                    }
                                    if at < orig.len() && orig[at].kind() == NEWLINE {
                                        at += 1;
                                    }
                                    continue;
                                } else {
                                    new_children.push(node.clone());
                                    at += 1;
                                }
                            } else {
                                new_children.push(node.clone());
                                at += 1;
                            }
                        }
                        child
                            .as_node()
                            .unwrap()
                            .splice_children(0..child.as_node().unwrap().children_with_tokens().count(), new_children);
                    }
                }
            }
        });
    }
}

fn generate_classifiers_to_entry(
    node: &SyntaxNode,
    min: (u8, u8),
    max: (u8, u8),
    omit: &[u8],
    existing: &HashSet<String>,
) {
    for array in node.children_with_tokens() {
        if array.kind() == VALUE {
            for root_value in array.as_node().unwrap().children_with_tokens() {
                if root_value.kind() == ARRAY {
                    let mut must_have: HashSet<String> = HashSet::new();
                    must_have.insert(String::from("Programming Language :: Python :: 3 :: Only"));
                    must_have.extend(
                        (min.1..=max.1)
                            .filter(|i| !omit.contains(i))
                            .map(|i| format!("Programming Language :: Python :: 3.{i}")),
                    );

                    let mut count = 0;
                    let delete = existing
                        .iter()
                        .filter(|e| e.starts_with("Programming Language :: Python :: 3") && !must_have.contains(*e))
                        .collect::<HashSet<&String>>();
                    let mut to_insert = Vec::<SyntaxElement>::new();
                    let mut delete_mode = false;
                    for array_entry in root_value.as_node().unwrap().children_with_tokens() {
                        count += 1;
                        let kind = array_entry.kind();
                        if delete_mode & [NEWLINE, BRACKET_END].contains(&kind) {
                            delete_mode = false;
                            if kind == NEWLINE {
                                continue;
                            }
                        } else if kind == VALUE {
                            for array_entry_value in array_entry.as_node().unwrap().children_with_tokens() {
                                if array_entry_value.kind() == STRING {
                                    let txt = load_text(array_entry_value.as_token().unwrap().text(), STRING);
                                    delete_mode = delete.contains(&txt);
                                    if delete_mode {
                                        // delete from previous comma/start until next newline
                                        let mut remove_count = to_insert.len();
                                        for (at, v) in to_insert.iter().rev().enumerate() {
                                            if [COMMA, BRACKET_START].contains(&v.kind()) {
                                                remove_count = at;
                                                for (i, e) in to_insert.iter().enumerate().skip(to_insert.len() - at) {
                                                    if e.kind() == NEWLINE {
                                                        remove_count = i + 1;
                                                        break;
                                                    }
                                                }
                                                break;
                                            }
                                        }
                                        to_insert.truncate(remove_count);
                                    }
                                    break;
                                }
                            }
                        }
                        if !delete_mode {
                            to_insert.push(array_entry);
                        }
                    }
                    let to_add: HashSet<_> = must_have.difference(existing).collect();
                    if !to_add.is_empty() {
                        // make sure we have a comma
                        let mut trail_at = 0;
                        for (at, v) in to_insert.iter().rev().enumerate() {
                            trail_at = to_insert.len() - at;
                            if v.kind() == COMMA {
                                for (i, e) in to_insert.iter().enumerate().skip(trail_at) {
                                    if e.kind() == NEWLINE || e.kind() == BRACKET_END {
                                        trail_at = i;
                                        break;
                                    }
                                }
                                break;
                            } else if v.kind() == BRACKET_START {
                                break;
                            } else if v.kind() == VALUE {
                                to_insert.insert(trail_at, make_comma());
                                trail_at += 1;
                                break;
                            }
                        }
                        let trail = to_insert.split_off(trail_at);
                        for add in to_add {
                            to_insert.push(make_array_entry(add));
                            to_insert.push(make_comma());
                        }
                        to_insert.extend(trail);
                    }
                    root_value.as_node().unwrap().splice_children(0..count, to_insert);
                }
            }
        }
    }
}

type MaxMinPythonWithClassifier = ((u8, u8), (u8, u8), Vec<u8>, Option<HashSet<String>>);

fn get_python_requires_with_classifier(
    table: &[SyntaxElement],
    max_supported_python: (u8, u8),
    min_supported_python: (u8, u8),
) -> MaxMinPythonWithClassifier {
    let mut classifiers: Option<HashSet<String>> = None;
    let mut mins: Vec<u8> = vec![];
    let mut maxs: Vec<u8> = vec![];
    let mut omit: Vec<u8> = vec![];
    assert_eq!(max_supported_python.0, 3, "for now only Python 3 supported");
    assert_eq!(min_supported_python.0, 3, "for now only Python 3 supported");

    for_entries(table, &mut |key, entry| {
        if key == "requires-python" {
            let re = Regex::new(r"^(?<op><|<=|==|!=|>=|>)3[.](?<minor>\d+)").unwrap();
            for child in entry.children_with_tokens() {
                if child.kind() == STRING {
                    let found_str_value = load_text(child.as_token().unwrap().text(), STRING);
                    for part in found_str_value.split(',') {
                        if let Some(caps) = re.captures(part) {
                            let minor = caps["minor"].parse::<u8>().unwrap();
                            match &caps["op"] {
                                "==" => {
                                    mins.push(minor);
                                    maxs.push(minor);
                                }
                                ">=" => {
                                    mins.push(minor);
                                }
                                ">" => {
                                    mins.push(minor + 1);
                                }
                                "<=" => {
                                    maxs.push(minor);
                                }
                                "<" => {
                                    maxs.push(minor - 1);
                                }
                                "!=" => {
                                    omit.push(minor);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        } else if key == "classifiers" {
            for child in entry.children_with_tokens() {
                if child.kind() == ARRAY {
                    let mut found_elements = HashSet::<String>::new();
                    for array in child.as_node().unwrap().children_with_tokens() {
                        if array.kind() == VALUE {
                            for value in array.as_node().unwrap().children_with_tokens() {
                                if value.kind() == STRING {
                                    let found = value.as_token().unwrap().text();
                                    let found_str_value: String = String::from(&found[1..found.len() - 1]);
                                    found_elements.insert(found_str_value);
                                }
                            }
                        }
                    }
                    classifiers = Some(found_elements);
                }
            }
        }
    });
    let min_py = (3, *mins.iter().max().unwrap_or(&min_supported_python.1));
    let max_py = (3, *maxs.iter().min().unwrap_or(&max_supported_python.1));
    (min_py, max_py, omit, classifiers)
}
