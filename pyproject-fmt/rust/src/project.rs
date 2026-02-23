use std::cell::RefMut;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::LazyLock;

use lexical_sort::natural_lexical_cmp;
use regex::Regex;
use tombi_syntax::SyntaxKind::{
    ARRAY, BASIC_STRING, BRACKET_END, BRACKET_START, COMMA, COMMENT, EQUAL, INLINE_TABLE, KEYS, KEY_VALUE, LINE_BREAK,
    WHITESPACE,
};
use tombi_syntax::{SyntaxElement, SyntaxNode};

use common::array::{dedupe_strings, ensure_trailing_comma, sort, sort_strings, transform};
use common::create::{
    make_array, make_array_entry, make_comma, make_entry_of_string, make_key, make_newline,
    make_table_array_with_entries, make_whitespace_n,
};
use common::pep508::Requirement;
use common::string::{get_string_token, load_text, update_content};
use common::table::{for_entries, reorder_table_keys, Tables};

use crate::TableFormatConfig;

fn normalize_and_sort_requirements(entry: &SyntaxNode, keep_full_version: bool) {
    transform(entry, &|s| {
        Requirement::new(s).unwrap().normalize(keep_full_version).to_string()
    });
    sort::<(String, String), _, _>(
        entry,
        |node| {
            get_string_token(node).map(|token| {
                let val = load_text(token.text(), node.kind());
                let package_name = Requirement::new(val.as_str()).unwrap().canonical_name();
                (package_name, val)
            })
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

pub fn fix(
    tables: &mut Tables,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    min_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
    table_config: &TableFormatConfig,
) {
    let key_order = &["name", "email"];

    if !table_config.should_collapse("project.authors") {
        expand_array_of_tables(tables, "project.authors", key_order);
    }
    if !table_config.should_collapse("project.maintainers") {
        expand_array_of_tables(tables, "project.maintainers", key_order);
    }

    let table_element = tables.get("project");
    if table_element.is_none() {
        return;
    }
    let table = &mut table_element.unwrap().first().unwrap().borrow_mut();

    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" \.(\W)").unwrap());
    expand_entry_points_inline_tables(table);

    for_entries(table, &mut |key, entry| match key.split('.').next().unwrap() {
        "name" => {
            update_content(entry, |s| Requirement::new(s).unwrap().canonical_name());
        }
        "license" => {
            static LICENSE_RE: LazyLock<Regex> =
                LazyLock::new(|| Regex::new(r"(?i)([^-])\b(and|or|with)\b([^-])").unwrap());
            update_content(entry, |s| {
                LICENSE_RE
                    .replace_all(s, |caps: &regex::Captures| {
                        format!("{}{}{}", &caps[1], caps[2].to_uppercase(), &caps[3])
                    })
                    .to_string()
            });
        }
        "description" => {
            update_content(entry, |s| {
                RE.replace_all(
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
            normalize_and_sort_requirements(entry, keep_full_version);
        }
        "dynamic" => {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        "keywords" => {
            dedupe_strings(entry, |s| s.to_lowercase());
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        "import-names" | "import-namespaces" => {
            transform(entry, &|s| {
                static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*;\s*").unwrap());
                RE.replace_all(s, "; ").trim_end().to_string()
            });
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        "classifiers" => {
            dedupe_strings(entry, |s| s.to_lowercase());
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        "authors" | "maintainers" => {}
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
            dedupe_strings(entry, |s| s.to_lowercase());
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });

    normalize_extra_names(table);

    reorder_table_keys(
        table,
        &[
            "",
            "name",
            "version",
            "import-names",
            "import-namespaces",
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

    if let Some(opt_deps_tables) = tables.get("project.optional-dependencies") {
        for table_ref in opt_deps_tables {
            let opt_deps_table = &mut table_ref.borrow_mut();
            for_entries(opt_deps_table, &mut |_key, entry| {
                normalize_and_sort_requirements(entry, keep_full_version);
            });
        }
    }
}

fn expand_entry_points_inline_tables(table: &mut RefMut<Vec<SyntaxElement>>) {
    let (mut to_insert, mut count, mut key) = (Vec::<SyntaxElement>::new(), 0, String::new());
    for s_table_entry in table.iter() {
        count += 1;
        if s_table_entry.kind() == KEY_VALUE {
            let mut has_inline_table = false;
            for s_in_table in s_table_entry.as_node().unwrap().children_with_tokens() {
                if s_in_table.kind() == KEYS {
                    key = s_in_table.as_node().unwrap().text().to_string().trim().to_string();
                } else if key.starts_with("entry-points.") && s_in_table.kind() == INLINE_TABLE {
                    has_inline_table = true;
                    for s_in_inline_table in s_in_table.as_node().unwrap().children_with_tokens() {
                        if s_in_inline_table.kind() == KEY_VALUE {
                            let mut with_key = String::new();
                            for s_in_entry in s_in_inline_table.as_node().unwrap().children_with_tokens() {
                                if s_in_entry.kind() == KEYS {
                                    with_key = s_in_entry.as_node().unwrap().text().to_string().trim().to_string();
                                } else if s_in_entry.kind() == BASIC_STRING {
                                    if let Some(string_node) = s_in_entry.as_node() {
                                        if let Some(token) = get_string_token(string_node) {
                                            let value = load_text(token.text(), BASIC_STRING);
                                            if !to_insert.is_empty() && to_insert.last().unwrap().kind() != LINE_BREAK {
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
                table.push(make_newline());
            }
            Some(c) => {
                let mut key_value = String::new();
                for table_row in table.iter() {
                    if table_row.kind() == KEY_VALUE {
                        for entry in table_row.as_node().unwrap().children_with_tokens() {
                            if entry.kind() == KEYS {
                                key_value = entry.as_node().unwrap().text().to_string().trim().to_string();
                            } else if !matches!(entry.kind(), KEYS | EQUAL | WHITESPACE | LINE_BREAK | COMMENT)
                                && key_value == "classifiers"
                            {
                                generate_classifiers_to_entry(table_row.as_node().unwrap(), min, max, &omit, &c);
                            }
                        }
                    }
                }
            }
        };
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
        if array.kind() == ARRAY {
            let array_node = array.as_node().unwrap();
            ensure_trailing_comma(array_node);
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
            for array_entry in array_node.children_with_tokens() {
                count += 1;
                let kind = array_entry.kind();
                if delete_mode & [LINE_BREAK, BRACKET_END].contains(&kind) {
                    delete_mode = false;
                    if kind == LINE_BREAK {
                        continue;
                    }
                } else if kind == BASIC_STRING {
                    if let Some(string_node) = array_entry.as_node() {
                        if let Some(token) = get_string_token(string_node) {
                            let txt = load_text(token.text(), BASIC_STRING);
                            delete_mode = delete.contains(&txt);
                            if delete_mode {
                                let mut truncate_at = to_insert.len();
                                for (rev_idx, v) in to_insert.iter().rev().enumerate() {
                                    let fwd_idx = to_insert.len() - 1 - rev_idx;
                                    if v.kind() == BRACKET_START {
                                        truncate_at = fwd_idx + 1;
                                        break;
                                    }
                                    if v.kind() == COMMA {
                                        // Keep the comma, remove whitespace/newline after it
                                        truncate_at = fwd_idx + 1;
                                        break;
                                    }
                                }
                                to_insert.truncate(truncate_at);
                            }
                        }
                    }
                }
                if !delete_mode {
                    to_insert.push(array_entry);
                }
            }
            let mut to_add: Vec<_> = must_have.difference(existing).map(|s| s.as_str()).collect();
            to_add.sort();
            if !to_add.is_empty() {
                let mut trail_at = 0;
                for (at, v) in to_insert.iter().rev().enumerate() {
                    trail_at = to_insert.len() - at;
                    if v.kind() == COMMA {
                        for (i, e) in to_insert.iter().enumerate().skip(trail_at) {
                            if e.kind() == LINE_BREAK || e.kind() == BRACKET_END {
                                trail_at = i;
                                break;
                            }
                        }
                        break;
                    } else if v.kind() == BRACKET_START {
                        break;
                    }
                }
                let trail = to_insert.split_off(trail_at);
                for add in to_add {
                    to_insert.push(make_newline());
                    to_insert.push(make_whitespace_n(2));
                    to_insert.push(make_array_entry(add));
                    to_insert.push(make_comma());
                }
                to_insert.extend(trail);
            }
            array_node.splice_children(0..count, to_insert);
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
            static RE: LazyLock<Regex> =
                LazyLock::new(|| Regex::new(r"^(?<op><|<=|==|!=|>=|>|~=)3[.](?<minor>\d+)").unwrap());
            for child in entry.children_with_tokens() {
                if child.kind() == BASIC_STRING {
                    if let Some(token) = child.as_token() {
                        let found_str_value = load_text(token.text(), BASIC_STRING);
                        for part in found_str_value.split(',') {
                            if let Some(caps) = RE.captures(part) {
                                let minor = caps["minor"].parse::<u8>().unwrap();
                                match &caps["op"] {
                                    "==" | "~=" => {
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
            }
        } else if key == "classifiers" && entry.kind() == ARRAY {
            let mut found_elements = HashSet::<String>::new();
            for array_child in entry.children_with_tokens() {
                if array_child.kind() == BASIC_STRING {
                    if let Some(string_node) = array_child.as_node() {
                        if let Some(token) = get_string_token(string_node) {
                            let found = token.text();
                            let found_str_value: String = String::from(&found[1..found.len() - 1]);
                            found_elements.insert(found_str_value);
                        }
                    }
                }
            }
            classifiers = Some(found_elements);
        }
    });
    let min_py = (3, *mins.iter().max().unwrap_or(&min_supported_python.1));
    let max_py = (3, *maxs.iter().min().unwrap_or(&max_supported_python.1));
    (min_py, max_py, omit, classifiers)
}

fn normalize_extra_names(table: &mut RefMut<Vec<SyntaxElement>>) {
    static EXTRA_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[-_.]+").unwrap());
    for element in table.iter() {
        if element.kind() != KEY_VALUE {
            continue;
        }
        let entry_node = element.as_node().unwrap();
        for child in entry_node.children_with_tokens() {
            if child.kind() != KEYS {
                continue;
            }
            let key_node = child.as_node().unwrap();
            let key_text = key_node.text().to_string().trim().to_string();
            if !key_text.starts_with("optional-dependencies.") {
                continue;
            }
            let extra_name = key_text.strip_prefix("optional-dependencies.").unwrap();
            let normalized = EXTRA_RE.replace_all(&extra_name.to_lowercase(), "-").to_string();
            if extra_name != normalized {
                let new_key = make_key(&format!("optional-dependencies.{normalized}"));
                let count = key_node.children_with_tokens().count();
                key_node.splice_children(0..count, new_key.as_node().unwrap().children_with_tokens().collect());
            }
        }
    }
}

fn expand_array_of_tables(tables: &mut Tables, full_name: &str, key_order: &[&str]) {
    let (parent_name, field_name) = full_name.split_once('.').expect("full_name must contain '.'");

    if let Some(positions) = tables.header_to_pos.get(full_name) {
        if !positions.is_empty() {
            return;
        }
    }

    let parent_positions = match tables.header_to_pos.get(parent_name) {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return,
    };

    let mut inline_table_entries: Vec<Vec<(String, String)>> = Vec::new();
    let mut entry_to_remove_index: Option<usize> = None;

    {
        let parent = tables.table_set[parent_positions[0]].borrow();
        let mut entry_index = 0;

        for element in parent.iter() {
            if element.kind() != KEY_VALUE {
                continue;
            }
            let entry_node = element.as_node().unwrap();
            let mut current_key = String::new();

            for child in entry_node.children_with_tokens() {
                if child.kind() == KEYS {
                    current_key = child.as_node().unwrap().text().to_string().trim().to_string();
                } else if current_key == field_name && child.kind() == ARRAY {
                    for array_element in child.as_node().unwrap().children_with_tokens() {
                        if array_element.kind() == INLINE_TABLE {
                            let mut fields: Vec<(String, String)> = Vec::new();
                            for inline_entry in array_element.as_node().unwrap().children_with_tokens() {
                                if inline_entry.kind() == KEY_VALUE {
                                    let mut key_name = String::new();
                                    let mut value_str = String::new();
                                    for e in inline_entry.as_node().unwrap().children_with_tokens() {
                                        match e.kind() {
                                            KEYS => {
                                                key_name = e.as_node().unwrap().text().to_string().trim().to_string();
                                            }
                                            BASIC_STRING => {
                                                if let Some(string_node) = e.as_node() {
                                                    value_str = string_node.text().to_string();
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    if !key_name.is_empty() && !value_str.is_empty() {
                                        fields.push((key_name, value_str));
                                    }
                                }
                            }
                            if !fields.is_empty() {
                                fields.sort_by(|a, b| {
                                    let order =
                                        |s: &str| key_order.iter().position(|&k| k == s).unwrap_or(key_order.len());
                                    order(&a.0).cmp(&order(&b.0)).then_with(|| a.0.cmp(&b.0))
                                });
                                inline_table_entries.push(fields);
                            }
                        }
                    }
                    entry_to_remove_index = Some(entry_index);
                }
            }
            entry_index += 1;
        }
    }

    if inline_table_entries.is_empty() {
        return;
    }

    if let Some(remove_idx) = entry_to_remove_index {
        let mut parent = tables.table_set[parent_positions[0]].borrow_mut();
        let mut new_elements = Vec::new();
        let mut entry_index = 0;
        let mut skip_next_line_break = false;

        for element in parent.iter() {
            if element.kind() == KEY_VALUE {
                if entry_index == remove_idx {
                    skip_next_line_break = true;
                } else {
                    new_elements.push(element.clone());
                }
                entry_index += 1;
            } else if skip_next_line_break && element.kind() == LINE_BREAK {
                skip_next_line_break = false;
            } else {
                new_elements.push(element.clone());
            }
        }

        let parent_len = parent.len();
        parent.splice(0..parent_len, new_elements);
    }

    for fields in inline_table_entries {
        let new_table = make_table_array_with_entries(full_name, &fields);
        let pos = tables.table_set.len();
        tables.table_set.push(std::cell::RefCell::new(new_table));
        tables
            .header_to_pos
            .entry(String::from(full_name))
            .or_default()
            .push(pos);
    }
}
