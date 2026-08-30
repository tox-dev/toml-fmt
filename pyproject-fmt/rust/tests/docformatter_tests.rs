use super::evaluate_full;

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

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
