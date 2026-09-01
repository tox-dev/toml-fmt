use super::evaluate_full as evaluate;

#[test]
fn test_djlint_order_and_sort() {
    let start = indoc::indoc! {r#"
    [tool.djlint]
    max_line_length = 120
    indent = 2
    ignore = ["H006", "H013", "H005"]
    profile = "django"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.djlint]
    profile = "django"
    indent = 2
    max_line_length = 120
    ignore = [ "H005", "H006", "H013" ]
    "#);
}

#[test]
fn test_djlint_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.djlint]
    profile = "django"
    indent = 2
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}
