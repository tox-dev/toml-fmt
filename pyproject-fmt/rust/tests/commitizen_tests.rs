use _pyproject_fmt::Settings;

use super::{default_settings, evaluate_full as evaluate, evaluate_settings};

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

#[test]
fn test_commitizen_questions_gain_a_trailing_comma_when_wrapped() {
    let settings = Settings {
        column_width: 100,
        ..default_settings()
    };
    let inline = indoc::indoc! {r#"
    [tool.commitizen]
    customize.questions = [ { type = "list", name = "change_type" }, { type = "input", name = "message" } ]
    "#};
    let tables = indoc::indoc! {r#"
    [[tool.commitizen.customize.questions]]
    type = "list"
    name = "change_type"

    [[tool.commitizen.customize.questions]]
    type = "input"
    name = "message"
    "#};
    let expected = indoc::indoc! {r#"
    [tool.commitizen]
    customize.questions = [
      { type = "list", name = "change_type" },
      { type = "input", name = "message" },
    ]
    "#};

    for source in [inline, tables, expected] {
        assert_eq!(evaluate_settings(source, &settings), expected);
    }
}
