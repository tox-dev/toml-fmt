use super::default_settings;
use super::evaluate_full;
use _pyproject_fmt::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

fn long_settings() -> Settings {
    Settings {
        table_format: String::from("long"),
        ..default_settings()
    }
}

fn evaluate_long(start: &str) -> String {
    let result = format_toml(start, &long_settings()).unwrap();
    super::assert_valid_toml(&result);
    result
}

#[test]
fn test_autopep8_order() {
    let start = indoc::indoc! {r#"
    [tool.autopep8]
    recursive = true
    max_line_length = 100
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @"
    [tool.autopep8]
    max_line_length = 100
    recursive = true
    ");
}

#[test]
fn test_autopep8_no_table_noop() {
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
fn test_autopep8_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.autopep8]
    select = ["E501", "E302", "E401"]
    ignore = ["W503", "E203"]
    exclude = ["build", "dist", "tests"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.autopep8]
    ignore = [ "E203", "W503" ]
    select = [ "E302", "E401", "E501" ]
    exclude = [ "build", "dist", "tests" ]
    "#);
}

#[test]
fn test_autopep8_non_sortable_preserved() {
    let start = indoc::indoc! {r#"
    [tool.autopep8]
    verbose = 2
    aggressive = 1
    max_line_length = 100
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @"
    [tool.autopep8]
    max_line_length = 100
    aggressive = 1
    verbose = 2
    ");
}

#[test]
fn test_autopep8_long_format() {
    let start = indoc::indoc! {r#"
    [tool.autopep8]
    select = ["E501", "E302"]
    max_line_length = 100
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.autopep8]"));
    assert!(result.find("E302").unwrap() < result.find("E501").unwrap());
    assert!(result.find("max_line_length").unwrap() < result.find("select").unwrap());
}
