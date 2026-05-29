use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::deptry::fix;
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.deptry"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

fn default_settings() -> Settings {
    Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        sub_table_spacing: String::new(),
        separate_root_table: String::from("\n"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
    }
}

fn long_settings() -> Settings {
    Settings {
        table_format: String::from("long"),
        ..default_settings()
    }
}

fn evaluate_long(start: &str) -> String {
    let result = format_toml(start, &long_settings());
    assert_valid_toml(&result);
    result
}

#[test]
fn test_deptry_order() {
    let start = indoc::indoc! {r#"
    [tool.deptry]
    ignore_unused = ["pytest"]
    exclude = ["tests", "docs"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.deptry]
    exclude = [ "docs", "tests" ]
    ignore_unused = [ "pytest" ]
    "#);
}

#[test]
fn test_deptry_no_table_noop() {
    let start = indoc::indoc! {r#"
    [project]
    name = "x"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "x"
    "#);
}

#[test]
fn test_deptry_non_sortable_entry_preserved() {
    let start = indoc::indoc! {r#"
    [tool.deptry]
    ignore_notebooks = true
    per_rule_ignores = { DEP001 = ["foo", "bar"] }
    exclude = ["zeta", "alpha"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.deptry]
    exclude = [ "alpha", "zeta" ]
    ignore_notebooks = true
    per_rule_ignores = { DEP001 = [ "foo", "bar" ] }
    "#);
}

#[test]
fn test_deptry_long_format() {
    let start = indoc::indoc! {r#"
    [tool.deptry]
    exclude = ["zeta", "alpha"]
    ignore = ["DEP002", "DEP001"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.deptry]"));
    assert!(result.find("alpha").unwrap() < result.find("zeta").unwrap());
    assert!(result.find("DEP001").unwrap() < result.find("DEP002").unwrap());
}
