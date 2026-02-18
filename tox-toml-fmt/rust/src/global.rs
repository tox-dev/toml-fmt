use std::cell::RefCell;

use lexical_sort::natural_lexical_cmp;
use regex::Regex;
use tombi_syntax::SyntaxKind::{ARRAY, BASIC_STRING, INLINE_TABLE};
use tombi_syntax::{SyntaxElement, SyntaxNode};

use common::array::{sort, sort_strings, transform};
use common::create::make_entry_of_string;
use common::pep508::Requirement;
use common::string::load_text;
use common::table::{for_entries, rename_keys, reorder_table_keys, Tables};

fn is_env_table(key: &str) -> bool {
    key == "env_run_base" || key == "env_pkg_base" || (key.starts_with("env.") && !key["env.".len()..].contains('.'))
}

fn env_tables(tables: &Tables) -> Vec<(&String, Vec<&RefCell<Vec<SyntaxElement>>>)> {
    tables
        .header_to_pos
        .keys()
        .filter(|k| is_env_table(k))
        .map(|k| (k, tables.get(k).unwrap()))
        .collect()
}

const ROOT_ALIASES: &[(&str, &str)] = &[
    ("envlist", "env_list"),
    ("toxinidir", "tox_root"),
    ("toxworkdir", "work_dir"),
    ("skipsdist", "no_package"),
    ("isolated_build_env", "package_env"),
    ("setupdir", "package_root"),
    ("minversion", "min_version"),
    ("ignore_basepython_conflict", "ignore_base_python_conflict"),
];

const ENV_ALIASES: &[(&str, &str)] = &[
    ("setenv", "set_env"),
    ("passenv", "pass_env"),
    ("envdir", "env_dir"),
    ("envtmpdir", "env_tmp_dir"),
    ("envlogdir", "env_log_dir"),
    ("changedir", "change_dir"),
    ("basepython", "base_python"),
    ("usedevelop", "use_develop"),
    ("sitepackages", "system_site_packages"),
    ("alwayscopy", "always_copy"),
];

const ROOT_KEY_ORDER: &[&str] = &[
    "",
    "min_version",
    "requires",
    "provision_tox_env",
    "env_list",
    "labels",
    "base",
    "package_env",
    "package_root",
    "no_package",
    "skip_missing_interpreters",
    "ignore_base_python_conflict",
    "work_dir",
    "temp_dir",
    "tox_root",
];

const ENV_KEY_ORDER: &[&str] = &[
    "",
    "runner",
    "description",
    "base_python",
    "system_site_packages",
    "always_copy",
    "download",
    "package",
    "package_env",
    "wheel_build_env",
    "package_tox_env_type",
    "package_root",
    "skip_install",
    "use_develop",
    "meta_dir",
    "pkg_dir",
    "pip_pre",
    "install_command",
    "list_dependencies_command",
    "deps",
    "dependency_groups",
    "constraints",
    "constrain_package_deps",
    "use_frozen_constraints",
    "extras",
    "recreate",
    "parallel_show_output",
    "skip_missing_interpreters",
    "pass_env",
    "disallow_pass_env",
    "set_env",
    "change_dir",
    "platform",
    "args_are_paths",
    "ignore_errors",
    "ignore_outcome",
    "commands_pre",
    "commands",
    "commands_post",
    "allowlist_externals",
    "labels",
    "suicide_timeout",
    "interrupt_timeout",
    "terminate_timeout",
    "depends",
    "env_dir",
    "env_tmp_dir",
    "env_log_dir",
];

pub fn normalize_aliases(tables: &Tables) {
    if let Some(root_tables) = tables.get("") {
        for table_ref in root_tables {
            rename_keys(&mut table_ref.borrow_mut(), ROOT_ALIASES);
        }
    }
    for (_key, table_refs) in env_tables(tables) {
        for table_ref in table_refs {
            rename_keys(&mut table_ref.borrow_mut(), ENV_ALIASES);
        }
    }
}

pub fn fix_root(tables: &Tables) {
    let Some(root_tables) = tables.get("") else {
        return;
    };
    for table_ref in root_tables {
        let table = &mut table_ref.borrow_mut();
        for_entries(table, &mut |key, entry| {
            if key == "requires" {
                transform(entry, &|s| Requirement::new(s).unwrap().normalize(false).to_string());
                sort_strings::<String, _, _>(
                    entry,
                    |s| Requirement::new(s.as_str()).unwrap().canonical_name(),
                    &|lhs, rhs| natural_lexical_cmp(lhs, rhs),
                );
            }
        });
        reorder_table_keys(table, ROOT_KEY_ORDER);
    }
}

pub fn fix_envs(tables: &Tables) {
    for (_key, table_refs) in env_tables(tables) {
        for table_ref in table_refs {
            let table = &mut table_ref.borrow_mut();
            upgrade_use_develop(table);
            for_entries(table, &mut |key, entry| {
                fix_env_entry(&key, entry);
            });
            reorder_table_keys(table, ENV_KEY_ORDER);
        }
    }
}

fn get_key_name(entry: &SyntaxElement) -> Option<String> {
    use tombi_syntax::SyntaxKind::KEYS;
    let node = entry.as_node()?;
    let keys = node
        .children_with_tokens()
        .find(|c| c.kind() == KEYS)
        .expect("KEY_VALUE must have KEYS child");
    Some(
        keys.as_node()
            .expect("KEYS must be a node")
            .text()
            .to_string()
            .trim()
            .to_string(),
    )
}

fn upgrade_use_develop(table: &mut Vec<SyntaxElement>) {
    use tombi_syntax::SyntaxKind::{KEY_VALUE, WHITESPACE};
    let mut use_develop_idx = None;
    let mut is_true = false;
    let mut has_package = false;
    for (i, entry) in table.iter().enumerate() {
        if entry.kind() != KEY_VALUE {
            continue;
        }
        let key_text = get_key_name(entry).expect("KEY_VALUE entry must have key name");
        if key_text == "use_develop" {
            use_develop_idx = Some(i);
            is_true = entry.as_node().unwrap().text().to_string().contains("true");
        } else if key_text == "package" {
            has_package = true;
        }
    }
    let Some(idx) = use_develop_idx else {
        return;
    };
    if !is_true {
        return;
    }
    // Remove use_develop = true and any trailing whitespace/newline
    table.remove(idx);
    while idx < table.len() && matches!(table[idx].kind(), WHITESPACE | tombi_syntax::SyntaxKind::LINE_BREAK) {
        table.remove(idx);
    }
    if !has_package {
        let entry = make_entry_of_string(&String::from("package"), &String::from("editable"));
        table.insert(idx, entry);
    }
}

fn fix_env_entry(key: &str, entry: &SyntaxNode) {
    match key {
        "deps" => {
            transform(entry, &|s| Requirement::new(s).unwrap().normalize(false).to_string());
            sort_strings::<String, _, _>(
                entry,
                |s| Requirement::new(s.as_str()).unwrap().canonical_name(),
                &|lhs, rhs| natural_lexical_cmp(lhs, rhs),
            );
        }
        "dependency_groups" | "allowlist_externals" | "extras" | "labels" | "depends" | "constraints" => {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        "pass_env" => sort_pass_env(entry),
        _ => {}
    }
}

fn sort_pass_env(entry: &SyntaxNode) {
    sort::<(u8, String), _, _>(
        entry,
        |node| {
            let kind = node.kind();
            if kind == INLINE_TABLE {
                Some((0, String::new()))
            } else {
                let text = node.text().to_string();
                let val = load_text(&text, kind);
                Some((1, val.to_lowercase()))
            }
        },
        &|lhs, rhs| lhs.0.cmp(&rhs.0).then_with(|| natural_lexical_cmp(&lhs.1, &rhs.1)),
    );
}

fn classify_env_part(part: &str) -> Option<(i32, i32, i32)> {
    let py_re = Regex::new(r"^(?:py)?(\d+)\.?(\d+)?$").unwrap();
    let pypy_re = Regex::new(r"^pypy(\d+)\.?(\d+)?$").unwrap();
    if let Some(caps) = pypy_re.captures(part) {
        let major = caps.get(1).map_or(0, |m| m.as_str().parse::<i32>().unwrap_or(0));
        let minor = caps.get(2).map_or(0, |m| m.as_str().parse::<i32>().unwrap_or(0));
        return Some((2, -major, -minor));
    }
    if let Some(caps) = py_re.captures(part) {
        let major = caps.get(1).map_or(0, |m| m.as_str().parse::<i32>().unwrap_or(0));
        let minor = caps.get(2).map_or(0, |m| m.as_str().parse::<i32>().unwrap_or(0));
        return Some((1, -major, -minor));
    }
    None
}

pub fn sort_env_list(tables: &Tables, pin_envs: &[String]) {
    let Some(root_tables) = tables.get("") else {
        return;
    };
    for table_ref in root_tables {
        let table = table_ref.borrow();
        for_entries(&table, &mut |key, entry| {
            if key != "env_list" {
                return;
            }
            sort::<(i32, i32, i32, String), _, _>(
                entry,
                |node| {
                    let kind = node.kind();
                    let text = node.text().to_string();
                    let val = load_text(&text, kind);
                    let lower = val.to_lowercase();
                    for part in lower.split('-') {
                        if let Some(idx) = pin_envs.iter().position(|p| p.to_lowercase() == part) {
                            return Some((0, idx as i32, 0, lower));
                        }
                        if let Some((cat, major, minor)) = classify_env_part(part) {
                            return Some((cat, major, minor, lower));
                        }
                    }
                    Some((3, 0, 0, lower))
                },
                &|lhs, rhs| {
                    lhs.0
                        .cmp(&rhs.0)
                        .then_with(|| lhs.1.cmp(&rhs.1))
                        .then_with(|| lhs.2.cmp(&rhs.2))
                        .then_with(|| natural_lexical_cmp(&lhs.3, &rhs.3))
                },
            );
        });
    }
}

fn normalize_value(node: &SyntaxNode) {
    for child in node.children_with_tokens() {
        match child.kind() {
            ARRAY => {
                if let Some(array_node) = child.as_node() {
                    normalize_value(array_node);
                }
            }
            INLINE_TABLE => {
                if let Some(table_node) = child.as_node() {
                    normalize_value(table_node);
                }
            }
            _ => {}
        }
    }
}

pub fn normalize_strings(tables: &Tables) {
    for table_ref in &tables.table_set {
        let table = table_ref.borrow();
        for_entries(&table, &mut |_, value_node| {
            normalize_value(value_node);
        });
    }
}

fn get_env_list_order(tables: &Tables) -> Vec<String> {
    let mut env_order = Vec::new();

    if let Some(root_tables) = tables.get("") {
        for table_ref in root_tables {
            let table = table_ref.borrow();
            for_entries(&table, &mut |key, entry| {
                if key == "env_list" && entry.kind() == ARRAY {
                    for array_child in entry.children_with_tokens() {
                        if array_child.kind() == BASIC_STRING {
                            let env_name = load_text(&array_child.to_string(), BASIC_STRING);
                            env_order.push(format!("env.{env_name}"));
                        }
                    }
                }
            });
        }
    }

    env_order
}

pub fn reorder_tables(root_ast: &SyntaxNode, tables: &Tables) {
    let env_list_order = get_env_list_order(tables);
    let has_env_list = !env_list_order.is_empty();

    let mut order: Vec<&str> = vec!["", "env_run_base", "env_pkg_base"];

    let env_refs: Vec<&str> = env_list_order.iter().map(|s| s.as_str()).collect();
    order.extend(env_refs);

    order.push("env");

    let multi_level_prefixes: &[&str] = if has_env_list { &["env"] } else { &[] };
    tables.reorder(root_ast, &order, multi_level_prefixes);
}
