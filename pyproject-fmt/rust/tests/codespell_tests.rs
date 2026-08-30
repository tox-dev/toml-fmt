use super::evaluate_full;

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

#[test]
fn test_codespell_order_and_sorted_arrays() {
    let start = indoc::indoc! {r#"
    [tool.codespell]
    write-changes = true
    skip = ["./vendor", "./build"]
    ignore-words-list = ["fo", "ba"]
    builtin = "clear,rare"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.codespell]
    builtin = "clear,rare"
    ignore-words-list = [ "ba", "fo" ]
    skip = [ "./build", "./vendor" ]
    write-changes = true
    "#);
}

#[test]
fn test_codespell_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.codespell]
    skip = [ "build", "dist" ]
    ignore-words-list = [ "fo", "ba" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

/// codespell loads the dictionaries in the order they are listed and the later file's correction
/// wins, so the order is the override chain.
#[test]
fn test_codespell_dictionary_keeps_its_order() {
    let start = indoc::indoc! {r#"
    [tool.codespell]
    dictionary = ["z-shared.txt", "a-project.txt"]
    builtin = ["clear", "rare"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.codespell]
    builtin = [ "clear", "rare" ]
    dictionary = [ "z-shared.txt", "a-project.txt" ]
    "#);
}
