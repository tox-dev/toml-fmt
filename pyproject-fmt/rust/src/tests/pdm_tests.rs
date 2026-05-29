use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::pdm::fix;
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.pdm"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_pdm_top_level_order() {
    let start = indoc::indoc! {r#"
    [tool.pdm.build]
    includes = ["src/**"]

    [tool.pdm.version]
    source = "scm"

    [tool.pdm]
    distribution = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pdm]
    distribution = true
    version.source = "scm"
    build.includes = [ "src/**" ]
    "#);
}

#[test]
fn test_pdm_build_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pdm.build]
    includes = ["zebra/**", "alpha/**"]
    excludes = ["tests/*"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pdm]
    build.includes = [ "alpha/**", "zebra/**" ]
    build.excludes = [ "tests/*" ]
    "#);
}

#[test]
fn test_pdm_dev_dependencies_inner_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pdm.dev-dependencies]
    test = ["pytest", "coverage"]
    lint = ["ruff", "black"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pdm]
    dev-dependencies.lint = [ "black", "ruff" ]
    dev-dependencies.test = [ "coverage", "pytest" ]
    "#);
}

#[test]
fn test_pdm_source_aot_key_order() {
    let start = indoc::indoc! {r#"
    [[tool.pdm.source]]
    verify_ssl = false
    type = "find_links"
    url = "https://example.com/links"
    name = "internal"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pdm]
    source = [ { verify_ssl = false, type = "find_links", url = "https://example.com/links", name = "internal" } ]
    "#);
}

#[test]
fn test_pdm_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.pdm]
    distribution = true
    version.source = "scm"
    build.includes = [ "src/**" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_pdm_no_table_is_noop() {
    let start = indoc::indoc! {r#"
    [project]
    name = "demo"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "demo"
    "#);
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
fn test_pdm_long_format_scripts() {
    let start = indoc::indoc! {r#"
    [tool.pdm.scripts]
    test = "pytest"
    lint = "ruff check"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.pdm.scripts]"));
    assert!(result.contains("test = \"pytest\""));
    assert!(result.contains("lint = \"ruff check\""));
}

#[test]
fn test_pdm_long_format_dev_dependencies() {
    let start = indoc::indoc! {r#"
    [tool.pdm.dev-dependencies]
    test = ["pytest", "coverage"]
    lint = ["ruff", "black"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.pdm.dev-dependencies]"));
    let block = result.split("[tool.pdm.dev-dependencies]").nth(1).unwrap();
    assert!(block.find("\"black\"").unwrap() < block.find("\"ruff\"").unwrap());
    assert!(block.find("\"coverage\"").unwrap() < block.find("\"pytest\"").unwrap());
}

#[test]
fn test_pdm_long_format_source_aot() {
    let start = indoc::indoc! {r#"
    [[tool.pdm.source]]
    verify_ssl = false
    exclude_packages = ["zeta", "alpha"]
    type = "find_links"
    include_packages = ["zinc", "amber"]
    url = "https://example.com/links"
    name = "internal"
    extra = "value"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.pdm.source]]"));
    let block = result.split("[[tool.pdm.source]]").nth(1).unwrap();
    assert!(block.find("name").unwrap() < block.find("url").unwrap());
    assert!(block.find("url").unwrap() < block.find("type").unwrap());
    assert!(block.find("type").unwrap() < block.find("verify_ssl").unwrap());
    assert!(block.find("verify_ssl").unwrap() < block.find("include_packages").unwrap());
    assert!(block.find("\"amber\"").unwrap() < block.find("\"zinc\"").unwrap());
    assert!(block.find("\"alpha\"").unwrap() < block.find("\"zeta\"").unwrap());
}
