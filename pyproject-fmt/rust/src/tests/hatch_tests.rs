use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::hatch::fix;
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.hatch"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

fn evaluate_full(start: &str) -> String {
    let s = Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 13),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        sub_table_spacing: String::new(),
        separate_root_table: String::from("\n"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
    };
    let r = format_toml(start, &s);
    assert_valid_toml(&r);
    r
}

#[test]
fn test_hatch_version_first_then_build() {
    let start = indoc::indoc! {r#"
    [tool.hatch.build]
    include = ["src/**/*.py"]

    [tool.hatch.version]
    path = "src/my_pkg/__init__.py"
    source = "regex"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    version.source = "regex"
    version.path = "src/my_pkg/__init__.py"
    build.include = [ "src/**/*.py" ]
    "#);
}

#[test]
fn test_hatch_build_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.hatch.build]
    include = ["zebra/**", "alpha/**", "beta/**"]
    exclude = ["zeta_tests/*", "alpha_tests/*"]
    packages = ["src/zebra_pkg", "src/alpha_pkg"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    build.packages = [ "src/alpha_pkg", "src/zebra_pkg" ]
    build.include = [ "alpha/**", "beta/**", "zebra/**" ]
    build.exclude = [ "alpha_tests/*", "zeta_tests/*" ]
    "#);
}

#[test]
fn test_hatch_env_inner_key_order() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.test]
    scripts.run = "pytest"
    dependencies = ["pytest", "coverage"]
    python = "3.11"
    detached = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    envs.test.detached = true
    envs.test.python = "3.11"
    envs.test.dependencies = [ "coverage", "pytest" ]
    envs.test.scripts.run = "pytest"
    "#);
}

#[test]
fn test_hatch_env_dependencies_sorted() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.default]
    dependencies = ["pytest", "black", "mypy", "ruff"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    envs.default.dependencies = [ "black", "mypy", "pytest", "ruff" ]
    "#);
}

#[test]
fn test_hatch_metadata_keys_grouped() {
    let start = indoc::indoc! {r#"
    [tool.hatch.metadata]
    allow-ambiguous-features = true
    allow-direct-references = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    metadata.allow-direct-references = true
    metadata.allow-ambiguous-features = true
    "#);
}

#[test]
fn test_hatch_targets_wheel_after_build_top_level() {
    let start = indoc::indoc! {r#"
    [tool.hatch.build.targets.wheel]
    packages = ["src/my_pkg"]

    [tool.hatch.build]
    include = ["src/**"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    build.include = [ "src/**" ]

    build.targets.wheel.packages = [ "src/my_pkg" ]
    "#);
}

#[test]
fn test_hatch_multiple_envs_handled() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.lint]
    dependencies = ["ruff"]

    [tool.hatch.envs.test]
    dependencies = ["pytest"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    envs.lint.dependencies = [ "ruff" ]
    envs.test.dependencies = [ "pytest" ]
    "#);
}

#[test]
fn test_hatch_comments_preserved() {
    let start = indoc::indoc! {r#"
    [tool.hatch.version]
    # Read version from __init__.py
    path = "src/pkg/__init__.py"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    # Read version from __init__.py
    version.path = "src/pkg/__init__.py"
    "#);
}

#[test]
fn test_hatch_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.hatch.version]
    path = "src/pkg/__init__.py"
    source = "regex"

    [tool.hatch.build]
    include = ["src/**"]
    packages = ["src/pkg"]

    [tool.hatch.envs.default]
    dependencies = ["pytest"]
    "#};
    let once = evaluate_full(start);
    let twice = evaluate_full(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_hatch_no_table_is_noop() {
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

fn evaluate_long(start: &str) -> String {
    let s = Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 13),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        sub_table_spacing: String::new(),
        separate_root_table: String::from("\n"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
    };
    let r = format_toml(start, &s);
    assert_valid_toml(&r);
    r
}

#[test]
fn test_hatch_long_format_env_table_keys_reordered() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.default]
    scripts = { test = "pytest" }
    dependencies = ["zeta", "alpha"]
    python = "3.12"
    type = "virtual"
    extra-dependencies = ["zinc", "amber"]
    features = ["dev", "test"]
    platforms = ["linux", "macos"]
    pre-install-commands = ["echo pre"]
    post-install-commands = ["echo post"]
    env-include = ["FOO_*"]
    env-exclude = ["BAR_*"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.hatch.envs.default]"));
    let block = result.split("[tool.hatch.envs.default]").nth(1).unwrap();
    assert!(block.find("type").unwrap() < block.find("python").unwrap());
    assert!(block.find("python").unwrap() < block.find("dependencies").unwrap());
    assert!(block.find("\"alpha\"").unwrap() < block.find("\"zeta\"").unwrap());
    assert!(block.find("\"amber\"").unwrap() < block.find("\"zinc\"").unwrap());
}

#[test]
fn test_hatch_long_format_env_scripts_subtable() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.default.scripts]
    test = "pytest"
    lint = "ruff check"

    [tool.hatch.envs.default.env-vars]
    FOO = "1"
    BAR = "2"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.hatch.envs.default.scripts]"));
    assert!(result.contains("[tool.hatch.envs.default.env-vars]"));
    assert!(result.contains("test = \"pytest\""));
    assert!(result.contains("FOO = \"1\""));
}

#[test]
fn test_hatch_envs_key_without_inner_segment() {
    let start = indoc::indoc! {r#"
    [tool.hatch]
    envs.bare = "value"
    "#};
    let result = evaluate(start);
    assert!(result.contains("envs.bare"));
}

#[test]
fn test_hatch_long_format_matrix_aot() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.default]
    python = "3.12"

    [[tool.hatch.envs.default.matrix]]
    python = ["3.12", "3.11"]

    [[tool.hatch.envs.default.matrix]]
    python = ["3.10"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.hatch.envs.default.matrix]]"));
}
