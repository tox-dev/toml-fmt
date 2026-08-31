use super::evaluate_full;

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

#[test]
fn test_yapf_based_on_style_first() {
    let start = indoc::indoc! {r#"
    [tool.yapf]
    column_limit = 120
    indent_width = 4
    based_on_style = "google"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.yapf]
    based_on_style = "google"
    column_limit = 120
    indent_width = 4
    "#);
}
