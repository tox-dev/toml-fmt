use super::evaluate_full as evaluate;

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
    run.source = [ "src" ]
    run.include = [ "**/*.py" ]
    run.omit = [ "tests/*" ]
    run.branch = true
    report.exclude_lines = [ "pragma: no cover" ]
    report.exclude_also = [ "if TYPE_CHECKING:" ]
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
    run.omit = [
      "tests/*", # Don't measure tests
    ]
    # Run configuration
    run.branch = true
    "#);
}

#[test]
/// coverage.py imports each plug-in and calls its hook in the order they are listed, so that list
/// keeps its order while the set-valued ones sort.
fn test_coverage_run_arrays_sorted_but_the_plugins() {
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
    run.source = [ "alpha", "bravo", "zulu" ]
    run.omit = [ "a_fixtures/*", "m_mocks/*", "z_tests/*" ]
    run.concurrency = [ "gevent", "multiprocessing", "thread" ]
    run.debug = [ "config", "sys", "trace" ]
    run.plugins = [ "coverage_plugin", "another_plugin" ]
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
    report.omit = [ "conftest.py", "fixtures/*", "tests/*" ]
    report.exclude_lines = [ "if TYPE_CHECKING:", "pragma: no cover", "raise NotImplementedError" ]
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
    run.omit = [
      "**/__main__.py",
      "**/cli.py",
    ]
    run.disable_warnings = [ "no-sysmon" ]  # 3.11 and earlier
    run.core = "sysmon"  # default for 3.14+, available for 3.12+
    # Regexes for lines to exclude from consideration
    report.exclude_also = [
      # Don't complain if non-runnable code isn't run:
      "if __name__ == .__main__.:",
    ]
    "#);
}

#[test]
fn test_coverage_string_include_not_array() {
    let start = indoc::indoc! {r#"
    [tool.coverage.run]
    branch = true
    include = "src"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    run.include = "src"
    run.branch = true
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

/// coverage maps a file with the first `[paths]` group that matches, so the groups keep the order
/// the file gave them.
#[test]
fn test_coverage_path_groups_keep_their_order() {
    let start = indoc::indoc! {r#"
    [tool.coverage.paths]
    z_specific = ["src", "/build/project/src"]
    a_fallback = ["other", "/build/*/src"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    paths.z_specific = [ "src", "/build/project/src" ]
    paths.a_fallback = [ "other", "/build/*/src" ]
    "#);
}
