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
fn test_deptry_order() {
    let start = indoc::indoc! {r#"
    [tool.deptry]
    ignore_unused = ["pytest"]
    exclude = ["tests", "docs"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.deptry]
    exclude = [ "docs", "tests" ]
    ignore_unused = [ "pytest" ]
    "#);
}

#[test]
fn test_deptry_no_table_noop() {
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
fn test_deptry_non_sortable_entry_preserved() {
    let start = indoc::indoc! {r#"
    [tool.deptry]
    ignore_notebooks = true
    per_rule_ignores = { DEP001 = ["foo", "bar"] }
    exclude = ["zeta", "alpha"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.deptry]
    exclude = [ "alpha", "zeta" ]
    ignore_notebooks = true
    per_rule_ignores = { DEP001 = [ "foo", "bar" ] }
    "#);
}

#[test]
fn test_deptry_long_format() {
    let start = indoc::indoc! {r#"
    [tool.deptry]
    exclude = ["zeta", "alpha"]
    ignore = ["DEP002", "DEP001"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.deptry]"));
    assert!(result.find("alpha").unwrap() < result.find("zeta").unwrap());
    assert!(result.find("DEP001").unwrap() < result.find("DEP002").unwrap());
}
