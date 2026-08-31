use super::evaluate_full;

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

#[test]
fn test_commitizen_order() {
    let start = indoc::indoc! {r#"
    [tool.commitizen]
    update_changelog_on_bump = true
    version_files = ["src/pkg/__init__.py", "pyproject.toml"]
    tag_format = "v$version"
    version = "1.0.0"
    name = "cz_conventional_commits"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.commitizen]
    name = "cz_conventional_commits"
    version = "1.0.0"
    version_files = [ "pyproject.toml", "src/pkg/__init__.py" ]
    tag_format = "v$version"
    update_changelog_on_bump = true
    "#);
}

#[test]
fn test_commitizen_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.commitizen]
    name = "cz_conventional_commits"
    version = "1.0.0"
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}
