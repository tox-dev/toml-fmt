use super::evaluate_full;

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

#[test]
fn test_vulture_order_and_sort() {
    let start = indoc::indoc! {r#"
    [tool.vulture]
    min_confidence = 80
    ignore_names = ["zlib", "alpha"]
    paths = ["src", "tests"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.vulture]
    paths = [ "src", "tests" ]
    ignore_names = [ "alpha", "zlib" ]
    min_confidence = 80
    "#);
}
