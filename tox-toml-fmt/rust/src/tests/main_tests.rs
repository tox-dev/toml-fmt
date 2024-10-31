use indoc::indoc;
use rstest::rstest;

use crate::{format_toml, Settings};

#[rstest]
#[case::simple(
        indoc ! {r#"
        requires = ["tox>=4.22"]
        env_list = ["3.13", "3.12"]
        skip_missing_interpreters = true

        [env_run_base]
        description = "run the tests with pytest under {env_name}"
        commands = [ ["pytest"] ]

        [env.type]
        description = "run type check on code base"
        commands = [["mypy", "src{/}tox_toml_fmt"], ["mypy", "tests"]]
    "#},
        indoc ! {r#"
        requires = [ "tox>=4.22" ]
        env_list = [ "3.13", "3.12" ]
        skip_missing_interpreters = true

        [env_run_base]
        description = "run the tests with pytest under {env_name}"
        commands = [ [ "pytest" ] ]

        [env.type]
        description = "run type check on code base"
        commands = [ [ "mypy", "src{/}tox_toml_fmt" ], [ "mypy", "tests" ] ]
    "#},
        2,
)]
#[case::empty(
        indoc ! {r""},
        "\n",
        2,
)]
fn test_format_toml(#[case] start: &str, #[case] expected: &str, #[case] indent: usize) {
    let settings = Settings {
        column_width: 80,
        indent,
    };
    let got = format_toml(start, &settings);
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test that the column width is respected,
/// and that arrays are neither exploded nor collapsed without reason
#[test]
fn test_column_width() {
    let start = indoc! {r#"
        # comment
        requires = ["tox>=4.22"]
        env_list = ["fix", "3.13", "3.12", "3.11", "3.10", "3.9", "type", "docs", "pkg_meta"]
        "#};
    let settings = Settings {
        column_width: 50,
        indent: 4,
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        # comment
        requires = [ "tox>=4.22" ]
        env_list = [
            "fix",
            "3.13",
            "3.12",
            "3.11",
            "3.10",
            "3.9",
            "type",
            "docs",
            "pkg_meta",
        ]
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}
