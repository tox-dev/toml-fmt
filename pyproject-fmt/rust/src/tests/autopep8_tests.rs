use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::autopep8::fix;
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.autopep8"], 120);
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
fn test_autopep8_order() {
    let start = indoc::indoc! {r#"
    [tool.autopep8]
    recursive = true
    max_line_length = 100
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.autopep8]
    max_line_length = 100
    recursive = true
    "#);
}

#[test]
fn test_autopep8_no_table_noop() {
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
fn test_autopep8_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.autopep8]
    select = ["E501", "E302", "E401"]
    ignore = ["W503", "E203"]
    exclude = ["build", "dist", "tests"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.autopep8]
    ignore = [ "E203", "W503" ]
    select = [ "E302", "E401", "E501" ]
    exclude = [ "build", "dist", "tests" ]
    "#);
}

#[test]
fn test_autopep8_non_sortable_preserved() {
    let start = indoc::indoc! {r#"
    [tool.autopep8]
    verbose = 2
    aggressive = 1
    max_line_length = 100
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.autopep8]
    max_line_length = 100
    aggressive = 1
    verbose = 2
    "#);
}

#[test]
fn test_autopep8_long_format() {
    let start = indoc::indoc! {r#"
    [tool.autopep8]
    select = ["E501", "E302"]
    max_line_length = 100
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.autopep8]"));
    assert!(result.find("E302").unwrap() < result.find("E501").unwrap());
    assert!(result.find("max_line_length").unwrap() < result.find("select").unwrap());
}
