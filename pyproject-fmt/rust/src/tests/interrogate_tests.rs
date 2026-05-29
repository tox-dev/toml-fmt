use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::interrogate::fix;
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.interrogate"], 120);
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
fn test_interrogate_order() {
    let start = indoc::indoc! {r#"
    [tool.interrogate]
    verbose = 2
    fail-under = 80
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.interrogate]
    fail-under = 80
    verbose = 2
    "#);
}

#[test]
fn test_interrogate_no_table_noop() {
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
fn test_interrogate_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.interrogate]
    exclude = ["tests", "build", "docs"]
    extend-exclude = ["zeta", "alpha"]
    ignore-regex = ["^test_.*$", "^_.*$"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.interrogate]
    ignore-regex = [ "^_.*$", "^test_.*$" ]
    exclude = [ "build", "docs", "tests" ]
    extend-exclude = [ "alpha", "zeta" ]
    "#);
}

#[test]
fn test_interrogate_non_sortable_preserved() {
    let start = indoc::indoc! {r#"
    [tool.interrogate]
    color = true
    badge-format = "svg"
    verbose = 2
    fail-under = 80
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.interrogate]
    fail-under = 80
    color = true
    verbose = 2
    badge-format = "svg"
    "#);
}

#[test]
fn test_interrogate_long_format() {
    let start = indoc::indoc! {r#"
    [tool.interrogate]
    exclude = ["zeta", "alpha"]
    fail-under = 80
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.interrogate]"));
    assert!(result.find("alpha").unwrap() < result.find("zeta").unwrap());
    assert!(result.find("fail-under").unwrap() < result.find("exclude").unwrap());
}
