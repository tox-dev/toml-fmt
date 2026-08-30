use super::evaluate_full;

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

#[test]
fn test_check_manifest_sort() {
    let start = indoc::indoc! {r#"
    [tool.check-manifest]
    ignore-default-rules = true
    ignore = ["zebra.txt", "alpha.txt"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.check-manifest]
    ignore = [ "alpha.txt", "zebra.txt" ]
    ignore-default-rules = true
    "#);
}
