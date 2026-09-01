use super::evaluate_full as evaluate;

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
