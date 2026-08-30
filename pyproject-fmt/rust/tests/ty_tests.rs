use super::evaluate_full;

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

#[test]
fn test_ty_order() {
    let start = indoc::indoc! {r#"
    [tool.ty]
    rules.unresolved-import = "ignore"
    src.include = ["src", "lib"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.ty]
    src.include = [ "lib", "src" ]
    rules.unresolved-import = "ignore"
    "#);
}

/// ty reads these the way a gitignore is read, where a `!pattern` takes back what a broader one
/// excluded, so a negation written after it stays after it.
#[test]
fn test_ty_exclude_keeps_its_order() {
    let start = indoc::indoc! {r#"
    [tool.ty]
    src.exclude = ["generated/**", "!generated/keep.py"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.ty]
    src.exclude = [ "generated/**", "!generated/keep.py" ]
    "#);
}

/// The same keys written under their own header say what the dotted ones say, and are put in the
/// same order, whether the header folds into `[tool.ty]` or stays written out.
#[test]
fn test_ty_src_written_out_as_a_table() {
    let start = indoc::indoc! {r#"
    [tool.ty.src]
    exclude = ["generated/**"]
    include = ["src", "lib"]
    respect-ignore-files = true
    "#};

    insta::assert_snapshot!(evaluate(start), @r#"
    [tool.ty]
    src.respect-ignore-files = true
    src.include = [ "lib", "src" ]
    src.exclude = [ "generated/**" ]
    "#);
    insta::assert_snapshot!(super::evaluate_long(start), @r#"
    [tool.ty.src]
    respect-ignore-files = true
    include = [ "lib", "src" ]
    exclude = [ "generated/**" ]
    "#);
}
