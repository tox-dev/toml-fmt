use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::cibuildwheel::fix;
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.cibuildwheel"], 120);
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
fn test_cibw_selection_first() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    test-command = "pytest {project}/tests"
    archs = ["x86_64", "arm64"]
    skip = ["pp*"]
    build = "cp3{10,11,12}-*"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    build = "cp3{10,11,12}-*"
    skip = [ "pp*" ]
    archs = [ "x86_64", "arm64" ]
    test-command = "pytest {project}/tests"
    "#);
}

#[test]
fn test_cibw_enable_sorted() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    enable = ["pypy", "cpython-prerelease", "cpython-freethreading"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    enable = [ "cpython-freethreading", "cpython-prerelease", "pypy" ]
    "#);
}

#[test]
fn test_cibw_addopts_preserve_order() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    before-all = "bash setup.sh"
    test-requires = ["pytest", "pytest-cov"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    before-all = "bash setup.sh"
    test-requires = [ "pytest", "pytest-cov" ]
    "#);
}

#[test]
fn test_cibw_per_platform_collapsed() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel.linux]
    before-all = "yum install -y openssl"
    archs = ["x86_64"]

    [tool.cibuildwheel.macos]
    archs = ["x86_64", "arm64"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    linux.archs = [ "x86_64" ]
    linux.before-all = "yum install -y openssl"
    macos.archs = [ "x86_64", "arm64" ]
    "#);
}

#[test]
fn test_cibw_overrides_aot_select_first() {
    let start = indoc::indoc! {r#"
    [[tool.cibuildwheel.overrides]]
    test-command = "pytest {project}/tests/cpython"
    select = "cp3{10,11}-*"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    overrides = [ { test-command = "pytest {project}/tests/cpython", select = "cp3{10,11}-*" } ]
    "#);
}

#[test]
fn test_cibw_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    build = "cp3{10,11,12}-*"
    skip = [ "pp*" ]
    archs = [ "x86_64" ]
    enable = [ "cpython-freethreading", "pypy" ]
    test-command = "pytest"
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_cibw_no_table_noop() {
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
fn test_cibw_long_format_per_platform_tables() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel.linux]
    before-all = "yum install -y openssl"
    enable = ["pypy", "cpython-freethreading"]
    archs = ["x86_64"]

    [tool.cibuildwheel.macos]
    enable = ["pypy", "cpython-freethreading"]
    archs = ["x86_64", "arm64"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.cibuildwheel.linux]"));
    assert!(result.contains("[tool.cibuildwheel.macos]"));
    assert!(result.find("cpython-freethreading").unwrap() < result.find("pypy").unwrap());
}

#[test]
fn test_cibw_long_format_overrides_aot() {
    let start = indoc::indoc! {r#"
    [[tool.cibuildwheel.overrides]]
    test-command = "pytest {project}/tests/cpython"
    select = "cp3{10,11}-*"
    enable = ["pypy", "cpython-freethreading"]
    build = "cp3*-*"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.cibuildwheel.overrides]]"));
    let block = result.split("[[tool.cibuildwheel.overrides]]").nth(1).unwrap();
    assert!(block.find("select").unwrap() < block.find("build").unwrap());
    assert!(block.find("build").unwrap() < block.find("enable").unwrap());
    assert!(block.find("cpython-freethreading").unwrap() < block.find("pypy").unwrap());
}
