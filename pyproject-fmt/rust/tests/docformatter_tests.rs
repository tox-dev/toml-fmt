use super::evaluate_full as evaluate;

#[test]
fn test_docformatter_order() {
    let start = indoc::indoc! {r#"
    [tool.docformatter]
    line-length = 100
    in-place = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @"
    [tool.docformatter]
    in-place = true
    line-length = 100
    ");
}
