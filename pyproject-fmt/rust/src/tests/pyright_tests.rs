use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::pyright::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.pyright", "tool.basedpyright"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_pyright_top_level_key_order() {
    let start = indoc::indoc! {r#"
    [tool.pyright]
    strict = ["strict_file.py"]
    typeCheckingMode = "strict"
    pythonVersion = "3.12"
    exclude = ["build"]
    include = ["src"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyright]
    pythonVersion = "3.12"
    typeCheckingMode = "strict"
    strict = [ "strict_file.py" ]
    include = [ "src" ]
    exclude = [ "build" ]
    "#);
}

#[test]
fn test_pyright_path_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pyright]
    include = ["zebra", "alpha"]
    exclude = ["build", "dist", "**/.venv"]
    ignore = ["zeta/*.py", "alpha/*.py"]
    extraPaths = ["src", "stubs"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyright]
    include = [ "alpha", "zebra" ]
    exclude = [ "**/.venv", "build", "dist" ]
    ignore = [ "alpha/*.py", "zeta/*.py" ]
    extraPaths = [ "src", "stubs" ]
    "#);
}

#[test]
fn test_pyright_report_rules_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.pyright]
    reportUnusedVariable = "warning"
    reportMissingImports = "error"
    reportGeneralTypeIssues = "error"
    pythonVersion = "3.11"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyright]
    pythonVersion = "3.11"
    reportGeneralTypeIssues = "error"
    reportMissingImports = "error"
    reportUnusedVariable = "warning"
    "#);
}

#[test]
fn test_pyright_strict_toggles_after_paths() {
    let start = indoc::indoc! {r#"
    [tool.pyright]
    strictListInference = true
    enableExperimentalFeatures = true
    include = ["src"]
    pythonVersion = "3.12"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyright]
    pythonVersion = "3.12"
    include = [ "src" ]
    strictListInference = true
    enableExperimentalFeatures = true
    "#);
}

#[test]
fn test_pyright_execution_environments_last() {
    let start = indoc::indoc! {r#"
    [tool.pyright]
    executionEnvironments = [{ root = "src" }]
    pythonVersion = "3.12"
    reportMissingImports = "error"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyright]
    pythonVersion = "3.12"
    reportMissingImports = "error"
    executionEnvironments = [ { root = "src" } ]
    "#);
}

#[test]
fn test_basedpyright_uses_same_schema() {
    let start = indoc::indoc! {r#"
    [tool.basedpyright]
    reportMissingImports = "error"
    failOnWarnings = true
    typeCheckingMode = "all"
    pythonVersion = "3.13"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.basedpyright]
    pythonVersion = "3.13"
    typeCheckingMode = "all"
    failOnWarnings = true
    reportMissingImports = "error"
    "#);
}

#[test]
fn test_pyright_unknown_keys_alphabetized_after_known() {
    let start = indoc::indoc! {r#"
    [tool.pyright]
    zzz_unknown = true
    aaa_unknown = false
    pythonVersion = "3.12"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyright]
    pythonVersion = "3.12"
    aaa_unknown = false
    zzz_unknown = true
    "#);
}

#[test]
fn test_pyright_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.pyright]
    pythonVersion = "3.12"
    typeCheckingMode = "strict"
    include = [ "src" ]
    exclude = [ "build" ]
    reportMissingImports = "error"
    reportUnusedVariable = "warning"
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_pyright_no_table_is_noop() {
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
