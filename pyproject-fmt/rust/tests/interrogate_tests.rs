use super::{evaluate_full as evaluate, evaluate_long};

#[test]
fn test_interrogate_order() {
    let start = indoc::indoc! {r#"
    [tool.interrogate]
    verbose = 2
    fail-under = 80
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @"
    [tool.interrogate]
    fail-under = 80
    verbose = 2
    ");
}

#[test]
fn test_interrogate_no_table_noop() {
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
fn test_interrogate_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.interrogate]
    exclude = ["tests", "build", "docs"]
    extend-exclude = ["zeta", "alpha"]
    ignore-regex = ["^test_.*$", "^_.*$"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.interrogate]
    ignore-regex = [ "^_.*$", "^test_.*$" ]
    exclude = [ "build", "docs", "tests" ]
    extend-exclude = [ "alpha", "zeta" ]
    "#);
}

#[test]
fn test_interrogate_non_sortable_preserved() {
    let start = indoc::indoc! {r#"
    [tool.interrogate]
    color = true
    badge-format = "svg"
    verbose = 2
    fail-under = 80
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.interrogate]
    fail-under = 80
    color = true
    verbose = 2
    badge-format = "svg"
    "#);
}

#[test]
fn test_interrogate_long_format() {
    let start = indoc::indoc! {r#"
    [tool.interrogate]
    exclude = ["zeta", "alpha"]
    fail-under = 80
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.interrogate]"));
    assert!(result.find("alpha").unwrap() < result.find("zeta").unwrap());
    assert!(result.find("fail-under").unwrap() < result.find("exclude").unwrap());
}
