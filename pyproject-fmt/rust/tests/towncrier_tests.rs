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
fn test_towncrier_order() {
    let start = indoc::indoc! {r#"
    [tool.towncrier]
    wrap = true
    directory = "changes"
    filename = "CHANGELOG.md"
    package = "my_pkg"
    name = "My Project"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.towncrier]
    name = "My Project"
    package = "my_pkg"
    directory = "changes"
    filename = "CHANGELOG.md"
    wrap = true
    "#);
}

#[test]
fn test_towncrier_type_aot_inner_order() {
    let start = indoc::indoc! {r#"
    [[tool.towncrier.type]]
    showcontent = true
    name = "Added"
    directory = "added"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.towncrier]
    type = [ { directory = "added", name = "Added", showcontent = true } ]
    "#);
}

#[test]
fn test_towncrier_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.towncrier]
    name = "Demo"
    package = "demo"
    directory = "changes"
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_towncrier_no_table_noop() {
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
fn test_towncrier_ignore_sorted() {
    let start = indoc::indoc! {r#"
    [tool.towncrier]
    package = "p"
    ignore = ["zeta.rst", "alpha.rst", "beta.rst"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.towncrier]
    package = "p"
    ignore = [ "alpha.rst", "beta.rst", "zeta.rst" ]
    "#);
}

#[test]
fn test_towncrier_long_format_type_aot() {
    let start = indoc::indoc! {r#"
    [tool.towncrier]
    package = "p"

    [[tool.towncrier.type]]
    showcontent = true
    name = "Added"
    directory = "added"

    [[tool.towncrier.type]]
    showcontent = false
    name = "Removed"
    directory = "removed"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.towncrier.type]]"));
    let first_block = result.split("[[tool.towncrier.type]]").nth(1).unwrap();
    assert!(first_block.find("directory").unwrap() < first_block.find("name").unwrap());
    assert!(first_block.find("name").unwrap() < first_block.find("showcontent").unwrap());
}

#[test]
fn test_towncrier_long_format_section_aot() {
    let start = indoc::indoc! {r#"
    [tool.towncrier]
    package = "p"

    [[tool.towncrier.section]]
    showcontent = true
    name = "Core"
    path = "src/core"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.towncrier.section]]"));
    let block = result.split("[[tool.towncrier.section]]").nth(1).unwrap();
    assert!(block.find("path").unwrap() < block.find("name").unwrap());
    assert!(block.find("name").unwrap() < block.find("showcontent").unwrap());
}

/// A section reads the same folded into its table as it does written out.
#[test]
fn test_towncrier_section_aot_inner_order_in_both_forms() {
    let start = indoc::indoc! {r#"
    [[tool.towncrier.section]]
    showcontent = true
    name = "Core"
    path = "core"
    "#};
    insta::assert_snapshot!(evaluate(start), @r#"
    [tool.towncrier]
    section = [ { path = "core", name = "Core", showcontent = true } ]
    "#);
    insta::assert_snapshot!(evaluate_long(start), @r#"
    [[tool.towncrier.section]]
    path = "core"
    name = "Core"
    showcontent = true
    "#);
}
