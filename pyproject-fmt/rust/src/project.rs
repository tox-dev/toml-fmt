use common::array::{dedupe_strings, sort, sort_strings, transform};
use common::create::{
    make_array, make_array_entry, make_comma, make_entry_of_string, make_entry_with_array_of_inline_tables, make_key,
    make_newline, make_table_array_with_entries,
};
use common::pep508::Requirement;
use common::string::{load_text, update_content};
use common::table::{collapse_sub_tables, expand_sub_tables, for_entries, reorder_table_keys, Tables};
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
use std::sync::LazyLock;

use crate::TableFormatConfig;

pub fn fix(
    tables: &mut Tables,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    min_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
    table_config: &TableFormatConfig,
) {
    let key_order = &["name", "email"];

    // Handle array of tables (authors/maintainers)
    if table_config.should_collapse("project.authors") {
        collapse_array_of_tables(tables, "project.authors", key_order);
    } else {
        expand_array_of_tables(tables, "project.authors", key_order);
    }
    if table_config.should_collapse("project.maintainers") {
        collapse_array_of_tables(tables, "project.maintainers", key_order);
    } else {
        expand_array_of_tables(tables, "project.maintainers", key_order);
    }

    // Handle sub-tables (urls, scripts, gui-scripts, optional-dependencies, entry-points)
    if table_config.should_collapse("project") {
        collapse_sub_tables(tables, "project");
    } else {
        expand_sub_tables(tables, "project");
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
        "version" | "readme" | "license-files" | "scripts" | "entry-points" | "gui-scripts" => {
            update_content(entry, |s| String::from(s));
        }
        "license" => {
            static LICENSE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(and|or|with)\b").unwrap());
            update_content(entry, |s| {
                LICENSE_RE
                    .replace_all(s, |caps: &regex::Captures| caps[1].to_uppercase())
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
        "dynamic" => {
            transform(entry, &|s| String::from(s));
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        "keywords" => {
            transform(entry, &|s| String::from(s));
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
            transform(entry, &|s| String::from(s));
            dedupe_strings(entry, |s| s.to_lowercase());
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        "authors" | "maintainers" => {
            sort::<(String, String), _, _>(
                entry,
                |node| {
                    let mut name = String::new();
                    let mut email = String::new();
                    for child in node.children_with_tokens() {
                        if child.kind() == INLINE_TABLE {
                            for item in child.as_node().unwrap().children_with_tokens() {
                                if item.kind() == ENTRY {
                                    let mut current_key = String::new();
                                    for e in item.as_node().unwrap().children_with_tokens() {
                                        match e.kind() {
                                            KEY => {
                                                for k in e.as_node().unwrap().children_with_tokens() {
                                                    if k.kind() == IDENT {
                                                        current_key = k.as_token().unwrap().text().to_string();
                                                    }
                                                }
                                            }
                                            VALUE => {
                                                for v in e.as_node().unwrap().children_with_tokens() {
                                                    if v.kind() == STRING {
                                                        let val = load_text(v.as_token().unwrap().text(), STRING);
                                                        match current_key.as_str() {
                                                            "name" => name = val.to_lowercase(),
                                                            "email" => email = val.to_lowercase(),
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some((name, email))
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
            static RE: LazyLock<Regex> =
                LazyLock::new(|| Regex::new(r"^(?<op><|<=|==|!=|>=|>|~=)3[.](?<minor>\d+)").unwrap());
            for child in entry.children_with_tokens() {
                if child.kind() == STRING {
                    let found_str_value = load_text(child.as_token().unwrap().text(), STRING);
                    for part in found_str_value.split(',') {
                        if let Some(caps) = RE.captures(part) {
                            let minor = caps["minor"].parse::<u8>().unwrap();
                            match &caps["op"] {
                                "==" | "~=" => {
                                    // ~= is compatible release: ~=3.12.7 means >=3.12.7,<3.13
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

fn normalize_extra_names(table: &mut RefMut<Vec<SyntaxElement>>) {
    static EXTRA_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[-_.]+").unwrap());
    for element in table.iter() {
        if element.kind() != ENTRY {
            continue;
        }
        let entry_node = element.as_node().unwrap();
        for child in entry_node.children_with_tokens() {
            if child.kind() != KEY {
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

fn collapse_array_of_tables(tables: &mut Tables, full_name: &str, key_order: &[&str]) {
    let positions = match tables.header_to_pos.get(full_name) {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return,
    };

    let parts: Vec<&str> = full_name.splitn(2, '.').collect();
    if parts.len() != 2 {
        return;
    }
    let parent_name = parts[0];
    let field_name = parts[1];

    let mut inline_tables = Vec::new();

    for pos in &positions {
        let table = tables.table_set[*pos].borrow();
        let mut fields = Vec::new();

        for element in table.iter() {
            if element.kind() != ENTRY {
                continue;
            }
            let entry_node = element.as_node().unwrap();
            let mut key_name = String::new();
            let mut value_str = String::new();

            for child in entry_node.children_with_tokens() {
                match child.kind() {
                    KEY => {
                        for k in child.as_node().unwrap().children_with_tokens() {
                            if k.kind() == IDENT {
                                key_name = k.as_token().unwrap().text().to_string();
                            }
                        }
                    }
                    VALUE => {
                        for v in child.as_node().unwrap().children_with_tokens() {
                            if v.kind() == STRING {
                                value_str = v.as_token().unwrap().text().to_string();
                            }
                        }
                    }
                    _ => {}
                }
            }

            if !key_name.is_empty() && !value_str.is_empty() {
                fields.push(format!("{key_name} = {value_str}"));
            }
        }

        if !fields.is_empty() {
            fields.sort_by(|a, b| {
                let order = |s: &str| {
                    for (i, key) in key_order.iter().enumerate() {
                        if s.starts_with(&format!("{key} ")) {
                            return i;
                        }
                    }
                    key_order.len()
                };
                order(a).cmp(&order(b)).then_with(|| a.cmp(b))
            });
            inline_tables.push(format!("{{ {} }}", fields.join(", ")));
        }
    }

    for pos in &positions {
        tables.table_set[*pos].borrow_mut().clear();
    }

    if inline_tables.is_empty() {
        return;
    }

    let parent_positions = match tables.header_to_pos.get(parent_name) {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return,
    };

    let entry = make_entry_with_array_of_inline_tables(field_name, &inline_tables);
    let mut parent = tables.table_set[parent_positions[0]].borrow_mut();
    if parent.last().is_some_and(|e| e.kind() != NEWLINE) {
        parent.push(make_newline());
    }
    parent.push(entry);
}

/// Expand inline table arrays to array of tables format.
/// This is the reverse of `collapse_array_of_tables`.
/// For example, `authors = [{ name = "John", email = "john@example.com" }]`
/// becomes `[[project.authors]]` with `name = "John"` and `email = "john@example.com"`.
fn expand_array_of_tables(tables: &mut Tables, full_name: &str, key_order: &[&str]) {
    let parts: Vec<&str> = full_name.splitn(2, '.').collect();
    if parts.len() != 2 {
        return;
    }
    let parent_name = parts[0];
    let field_name = parts[1];

    // Check if we already have array of tables entries
    if let Some(positions) = tables.header_to_pos.get(full_name) {
        if !positions.is_empty() {
            // Already expanded, nothing to do
            return;
        }
    }

    // Find the inline table array in the parent
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
            if element.kind() != ENTRY {
                continue;
            }
            let entry_node = element.as_node().unwrap();
            let mut current_key = String::new();

            for child in entry_node.children_with_tokens() {
                if child.kind() == KEY {
                    current_key = child.as_node().unwrap().text().to_string().trim().to_string();
                } else if child.kind() == VALUE && current_key == field_name {
                    // Found the array entry
                    for value_child in child.as_node().unwrap().children_with_tokens() {
                        if value_child.kind() == ARRAY {
                            // Parse inline tables from the array
                            for array_element in value_child.as_node().unwrap().children_with_tokens() {
                                if array_element.kind() == VALUE {
                                    for inner in array_element.as_node().unwrap().children_with_tokens() {
                                        if inner.kind() == INLINE_TABLE {
                                            let mut fields: Vec<(String, String)> = Vec::new();
                                            for inline_entry in inner.as_node().unwrap().children_with_tokens() {
                                                if inline_entry.kind() == ENTRY {
                                                    let mut key_name = String::new();
                                                    let mut value_str = String::new();
                                                    for e in inline_entry.as_node().unwrap().children_with_tokens() {
                                                        match e.kind() {
                                                            KEY => {
                                                                for k in e.as_node().unwrap().children_with_tokens() {
                                                                    if k.kind() == IDENT {
                                                                        key_name =
                                                                            k.as_token().unwrap().text().to_string();
                                                                    }
                                                                }
                                                            }
                                                            VALUE => {
                                                                for v in e.as_node().unwrap().children_with_tokens() {
                                                                    if v.kind() == STRING {
                                                                        value_str =
                                                                            v.as_token().unwrap().text().to_string();
                                                                    }
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
                                                // Sort fields according to key_order
                                                fields.sort_by(|a, b| {
                                                    let order = |s: &str| {
                                                        key_order
                                                            .iter()
                                                            .position(|&k| k == s)
                                                            .unwrap_or(key_order.len())
                                                    };
                                                    order(&a.0).cmp(&order(&b.0)).then_with(|| a.0.cmp(&b.0))
                                                });
                                                inline_table_entries.push(fields);
                                            }
                                        }
                                    }
                                }
                            }
                            entry_to_remove_index = Some(entry_index);
                        }
                    }
                }
            }
            entry_index += 1;
        }
    }

    if inline_table_entries.is_empty() {
        return;
    }

    // Remove the inline table array entry from the parent
    if let Some(remove_idx) = entry_to_remove_index {
        let mut parent = tables.table_set[parent_positions[0]].borrow_mut();
        let mut new_elements = Vec::new();
        let mut entry_index = 0;

        for element in parent.iter() {
            if element.kind() == ENTRY {
                if entry_index != remove_idx {
                    new_elements.push(element.clone());
                }
                entry_index += 1;
            } else {
                new_elements.push(element.clone());
            }
        }

        let parent_len = parent.len();
        parent.splice(0..parent_len, new_elements);
    }

    // Create new array of tables entries
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
