use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::scikit_build::fix;
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.scikit-build"], 120);
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
fn test_scikit_build_order() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    wheel.packages = ["src/foo"]
    cmake.version = ">=3.20"
    minimum-version = "0.9"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    minimum-version = "0.9"
    cmake.version = ">=3.20"
    wheel.packages = [ "src/foo" ]
    "#);
}

#[test]
fn test_scikit_build_no_table_noop() {
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
fn test_scikit_build_args_preserved() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    cmake.args = ["-DZ=1", "-DA=2"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    cmake.args = [ "-DZ=1", "-DA=2" ]
    "#);
}

#[test]
fn test_scikit_build_define_preserved() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build.cmake]
    define = { Z_FLAG = "z", A_FLAG = "a" }
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    cmake.define = { Z_FLAG = "z", A_FLAG = "a" }
    "#);
}

#[test]
fn test_scikit_build_packages_sorted() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    wheel.packages = ["src/zeta", "src/alpha", "src/beta"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    wheel.packages = [ "src/alpha", "src/beta", "src/zeta" ]
    "#);
}

#[test]
fn test_scikit_build_exclude_sorted() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    sdist.exclude = ["zeta/**", "alpha/**"]
    sdist.include = ["zinc/*", "amber/*"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    sdist.exclude = [ "alpha/**", "zeta/**" ]
    sdist.include = [ "amber/*", "zinc/*" ]
    "#);
}

#[test]
fn test_scikit_build_files_targets_components_sorted() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    install.components = ["zlib", "alib"]
    wheel.exclude-fields = ["metadata.version", "metadata.author"]
    install.targets = ["zt", "at"]
    sdist.files = ["zf", "af"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    sdist.files = [ "af", "zf" ]
    wheel.exclude-fields = [ "metadata.author", "metadata.version" ]
    install.components = [ "alib", "zlib" ]
    install.targets = [ "at", "zt" ]
    "#);
}

#[test]
fn test_scikit_build_long_format() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    wheel.packages = ["src/zeta", "src/alpha"]
    cmake.args = ["-DZ=1", "-DA=2"]
    minimum-version = "0.9"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.scikit-build]"));
    assert!(result.contains("minimum-version = \"0.9\""));
}
