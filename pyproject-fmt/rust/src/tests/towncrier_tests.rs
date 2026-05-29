use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::towncrier::fix;
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.towncrier"], 120);
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
fn test_towncrier_order() {
    let start = indoc::indoc! {r#"
    [tool.towncrier]
    wrap = true
    directory = "changes"
    filename = "CHANGELOG.md"
    package = "my_pkg"
    name = "My Project"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.towncrier]
    name = "My Project"
    package = "my_pkg"
    directory = "changes"
    filename = "CHANGELOG.md"
    wrap = true
    "#);
}

#[test]
fn test_towncrier_type_aot_inner_order() {
    let start = indoc::indoc! {r#"
    [[tool.towncrier.type]]
    showcontent = true
    name = "Added"
    directory = "added"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.towncrier]
    type = [ { showcontent = true, name = "Added", directory = "added" } ]
    "#);
}

#[test]
fn test_towncrier_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.towncrier]
    name = "Demo"
    package = "demo"
    directory = "changes"
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_towncrier_no_table_noop() {
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
fn test_towncrier_ignore_sorted() {
    let start = indoc::indoc! {r#"
    [tool.towncrier]
    package = "p"
    ignore = ["zeta.rst", "alpha.rst", "beta.rst"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.towncrier]
    package = "p"
    ignore = [ "alpha.rst", "beta.rst", "zeta.rst" ]
    "#);
}

#[test]
fn test_towncrier_long_format_type_aot() {
    let start = indoc::indoc! {r#"
    [tool.towncrier]
    package = "p"

    [[tool.towncrier.type]]
    showcontent = true
    name = "Added"
    directory = "added"

    [[tool.towncrier.type]]
    showcontent = false
    name = "Removed"
    directory = "removed"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.towncrier.type]]"));
    let first_block = result.split("[[tool.towncrier.type]]").nth(1).unwrap();
    assert!(first_block.find("directory").unwrap() < first_block.find("name").unwrap());
    assert!(first_block.find("name").unwrap() < first_block.find("showcontent").unwrap());
}

#[test]
fn test_towncrier_long_format_section_aot() {
    let start = indoc::indoc! {r#"
    [tool.towncrier]
    package = "p"

    [[tool.towncrier.section]]
    showcontent = true
    name = "Core"
    path = "src/core"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.towncrier.section]]"));
    let block = result.split("[[tool.towncrier.section]]").nth(1).unwrap();
    assert!(block.find("path").unwrap() < block.find("name").unwrap());
    assert!(block.find("name").unwrap() < block.find("showcontent").unwrap());
}
