use super::evaluate_full as evaluate;

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

/// Pyright searches these roots in the order they are given, so the one written first is the one it
/// resolves an import from.
#[test]
fn test_pyright_extra_paths_keep_their_order() {
    let start = indoc::indoc! {r#"
    [tool.pyright]
    extraPaths = ["stubs", "src"]
    include = ["src", "docs"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.pyright]
    include = [ "docs", "src" ]
    extraPaths = [ "stubs", "src" ]
    "#);
}
