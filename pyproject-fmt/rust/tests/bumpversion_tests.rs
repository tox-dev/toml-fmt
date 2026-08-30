use super::evaluate_full;

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

#[test]
fn test_bumpversion_order() {
    let start = indoc::indoc! {r#"
    [tool.bumpversion]
    commit = true
    tag = true
    current_version = "1.0.0"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bumpversion]
    current_version = "1.0.0"
    tag = true
    commit = true
    "#);
}
