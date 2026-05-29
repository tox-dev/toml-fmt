use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::pytest::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.pytest"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_pytest_top_level_order() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    log_cli_level = "INFO"
    markers = ["slow", "fast"]
    addopts = ["--strict-markers", "-ra"]
    testpaths = ["tests"]
    minversion = "8"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    ini_options.minversion = "8"
    ini_options.testpaths = [ "tests" ]
    ini_options.addopts = [ "--strict-markers", "-ra" ]
    ini_options.markers = [ "fast", "slow" ]
    ini_options.log_cli_level = "INFO"
    "#);
}

#[test]
fn test_pytest_markers_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    markers = [
      "slow: marks tests as slow",
      "integration: integration tests",
      "fast: marks tests as fast",
    ]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    ini_options.markers = [
      "fast: marks tests as fast",
      "integration: integration tests",
      "slow: marks tests as slow",
    ]
    "#);
}

#[test]
fn test_pytest_filterwarnings_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    filterwarnings = [
      "error",
      "ignore::PendingDeprecationWarning",
      "ignore::DeprecationWarning",
    ]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    ini_options.filterwarnings = [
      "error",
      "ignore::DeprecationWarning",
      "ignore::PendingDeprecationWarning",
    ]
    "#);
}

#[test]
fn test_pytest_addopts_preserve_order() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    addopts = ["-ra", "--strict-markers", "--strict-config"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    ini_options.addopts = [ "-ra", "--strict-markers", "--strict-config" ]
    "#);
}

#[test]
fn test_pytest_pythonpath_preserve_order() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    pythonpath = ["src", "vendor"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    ini_options.pythonpath = [ "src", "vendor" ]
    "#);
}

#[test]
fn test_pytest_python_files_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    python_files = ["test_*.py", "*_test.py"]
    python_classes = ["Test*"]
    python_functions = ["test_*"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    ini_options.python_files = [ "*_test.py", "test_*.py" ]
    ini_options.python_classes = [ "Test*" ]
    ini_options.python_functions = [ "test_*" ]
    "#);
}

#[test]
fn test_pytest_logging_keys_grouped() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    log_file = "pytest.log"
    log_cli = true
    log_format = "%(asctime)s %(levelname)s %(message)s"
    log_cli_level = "INFO"
    log_level = "DEBUG"
    log_file_level = "DEBUG"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    ini_options.log_format = "%(asctime)s %(levelname)s %(message)s"
    ini_options.log_level = "DEBUG"
    ini_options.log_cli = true
    ini_options.log_cli_level = "INFO"
    ini_options.log_file = "pytest.log"
    ini_options.log_file_level = "DEBUG"
    "#);
}

#[test]
fn test_pytest_junit_keys_grouped() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    junit_logging = "all"
    junit_family = "xunit2"
    junit_suite_name = "tests"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    ini_options.junit_suite_name = "tests"
    ini_options.junit_family = "xunit2"
    ini_options.junit_logging = "all"
    "#);
}

#[test]
fn test_pytest_required_plugins_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    required_plugins = ["pytest-cov", "pytest-asyncio", "pytest-mock"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    ini_options.required_plugins = [ "pytest-asyncio", "pytest-cov", "pytest-mock" ]
    "#);
}

#[test]
fn test_pytest_unknown_keys_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    zzz_unknown = true
    aaa_unknown = false
    minversion = "8"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    ini_options.minversion = "8"
    ini_options.aaa_unknown = false
    ini_options.zzz_unknown = true
    "#);
}

#[test]
fn test_pytest_comments_preserved() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    # Strict marker validation
    addopts = ["--strict-markers"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pytest]
    # Strict marker validation
    ini_options.addopts = [ "--strict-markers" ]
    "#);
}

#[test]
fn test_pytest_no_table_is_noop() {
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

#[test]
fn test_pytest_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.pytest.ini_options]
    minversion = "8"
    testpaths = [ "tests" ]
    addopts = [ "-ra", "--strict-markers" ]
    markers = [ "fast", "slow" ]
    filterwarnings = [ "error", "ignore::DeprecationWarning" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}
