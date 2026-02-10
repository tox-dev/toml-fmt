use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::coverage::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.coverage"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_coverage_run_before_report() {
    let start = indoc::indoc! {r#"
    [tool.coverage]
    report.omit = ["tests/*"]
    run.omit = ["tests/*"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    run.omit = [ "tests/*" ]
    report.omit = [ "tests/*" ]
    "#);
}

#[test]
fn test_coverage_paths_between_run_and_report() {
    let start = indoc::indoc! {r#"
    [tool.coverage]
    report.fail_under = 90
    paths.source = ["src/", "/build/src"]
    run.source = ["src"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    run.source = [ "src" ]
    paths.source = [ "src/", "/build/src" ]
    report.fail_under = 90
    "#);
}

#[test]
fn test_coverage_report_formats_after_report() {
    let start = indoc::indoc! {r#"
    [tool.coverage]
    xml.output = "coverage.xml"
    html.directory = "htmlcov"
    json.output = "coverage.json"
    lcov.output = "coverage.lcov"
    report.fail_under = 90
    run.branch = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    run.branch = true
    report.fail_under = 90
    html.directory = "htmlcov"
    json.output = "coverage.json"
    lcov.output = "coverage.lcov"
    xml.output = "coverage.xml"
    "#);
}

#[test]
fn test_coverage_grouped_options() {
    let start = indoc::indoc! {r#"
    [tool.coverage]
    run.branch = true
    run.omit = ["tests/*"]
    run.source = ["src"]
    run.include = ["**/*.py"]
    report.exclude_lines = ["pragma: no cover"]
    report.exclude_also = ["if TYPE_CHECKING:"]
    report.skip_empty = true
    report.skip_covered = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    run.branch = true
    run.include = [ "**/*.py" ]
    run.omit = [ "tests/*" ]
    run.source = [ "src" ]
    report.exclude_also = [ "if TYPE_CHECKING:" ]
    report.exclude_lines = [ "pragma: no cover" ]
    report.skip_covered = true
    report.skip_empty = true
    "#);
}

#[test]
fn test_coverage_comments_preserved() {
    let start = indoc::indoc! {r#"
    [tool.coverage]
    # Run configuration
    run.branch = true
    run.omit = [
        "tests/*",  # Don't measure tests
    ]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    # Run configuration
    run.branch = true
    run.omit = [
      "tests/*",  # Don't measure tests
    ]
    "#);
}

#[test]
fn test_coverage_run_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.coverage]
    run.omit = ["z_tests/*", "a_fixtures/*", "m_mocks/*"]
    run.source = ["zulu", "alpha", "bravo"]
    run.concurrency = ["multiprocessing", "gevent", "thread"]
    run.plugins = ["coverage_plugin", "another_plugin"]
    run.debug = ["trace", "config", "sys"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    run.concurrency = [ "gevent", "multiprocessing", "thread" ]
    run.debug = [ "config", "sys", "trace" ]
    run.omit = [ "a_fixtures/*", "m_mocks/*", "z_tests/*" ]
    run.plugins = [ "another_plugin", "coverage_plugin" ]
    run.source = [ "alpha", "bravo", "zulu" ]
    "#);
}

#[test]
fn test_coverage_report_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.coverage]
    report.omit = ["tests/*", "fixtures/*", "conftest.py"]
    report.exclude_lines = ["pragma: no cover", "if TYPE_CHECKING:", "raise NotImplementedError"]
    report.partial_branches = ["pragma: no branch", "if DEBUG:"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    report.exclude_lines = [ "if TYPE_CHECKING:", "pragma: no cover", "raise NotImplementedError" ]
    report.omit = [ "conftest.py", "fixtures/*", "tests/*" ]
    report.partial_branches = [ "if DEBUG:", "pragma: no branch" ]
    "#);
}

#[test]
fn test_coverage_trailing_comment_on_single_line_array() {
    let start = indoc::indoc! {r#"
    [tool.coverage.run]
    omit = [
      "**/__main__.py",
      "**/cli.py",
    ]
    core = "sysmon" # default for 3.14+, available for 3.12+
    disable_warnings = [ "no-sysmon" ] # 3.11 and earlier

    [tool.coverage.report]
    # Regexes for lines to exclude from consideration
    exclude_also = [
      # Don't complain if non-runnable code isn't run:
      "if __name__ == .__main__.:",
    ]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    run.core = "sysmon"  # default for 3.14+, available for 3.12+
    run.disable_warnings = [ "no-sysmon" ]  # 3.11 and earlier
    run.omit = [
      "**/__main__.py",
      "**/cli.py",
    ]
    # Regexes for lines to exclude from consideration
    report.exclude_also = [
      # Don't complain if non-runnable code isn't run:
      "if __name__ == .__main__.:",
    ]
    "#);
}

#[test]
fn test_coverage_paths_not_sorted() {
    let start = indoc::indoc! {r#"
    [tool.coverage]
    paths.source = ["src/mypackage", "/home/user/project/src/mypackage", "/build/src/mypackage"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    paths.source = [ "src/mypackage", "/home/user/project/src/mypackage", "/build/src/mypackage" ]
    "#);
}
