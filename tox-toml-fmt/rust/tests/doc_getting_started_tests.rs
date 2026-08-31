//! The examples `docs/tutorial/getting-started.rst` shows, formatted the way the docs print them.

use indoc::indoc;
use insta::assert_snapshot;

use super::format_doc_example;

#[test]
fn test_doc_getting_started() {
    let start = indoc! {r#"
        env_list = ["3.13", "3.12", "lint"]

        [env_run_base]
        description = "run the test suite with pytest"
        deps = [
            "pytest>=8",
        ]
        commands = [["pytest", { replace = "posargs", default = ["tests"], extend = true }]]

        [env.lint]
        description = "run linters"
        skip_install = true
        deps = ["ruff"]
        commands = [["ruff", "check", { replace = "posargs", default = ["."], extend = true }]]
        "#};
    let got = format_doc_example(start);
    assert_snapshot!(got, @r#"
    env_list = [ "3.13", "3.12", "lint" ]

    [env_run_base]
    description = "run the test suite with pytest"
    deps = [
      "pytest>=8",
    ]
    commands = [ [ "pytest", { replace = "posargs", default = [ "tests" ], extend = true } ] ]

    [env.lint]
    description = "run linters"
    skip_install = true
    deps = [ "ruff" ]
    commands = [ [ "ruff", "check", { replace = "posargs", default = [ "." ], extend = true } ] ]
    "#);
}

#[test]
fn test_doc_full_tox_toml_structure() {
    let start = indoc! {r#"
        # tox.toml - values at root level are core settings
        requires = ["tox>=4.20"]
        env_list = ["3.13", "3.12", "lint"]

        # base settings for run environments
        [env_run_base]
        deps = ["pytest>=8"]
        commands = [["pytest", "tests"]]

        # environment-specific overrides
        [env.lint]
        skip_install = true
        deps = ["ruff"]
        commands = [["ruff", "check", "."]]
        "#};
    let got = format_doc_example(start);
    assert_snapshot!(got, @r#"
    # tox.toml - values at root level are core settings
    requires = [ "tox>=4.20" ]
    env_list = [ "3.13", "3.12", "lint" ]

    # base settings for run environments
    [env_run_base]
    deps = [ "pytest>=8" ]
    commands = [ [ "pytest", "tests" ] ]

    # environment-specific overrides
    [env.lint]
    skip_install = true
    deps = [ "ruff" ]
    commands = [ [ "ruff", "check", "." ] ]
    "#);
}

#[test]
fn test_doc_env_base_tutorial() {
    let start = indoc! {r#"
        [env_base.test]
        factors = [["3.13", "3.14"]]
        deps = ["pytest>=8"]
        commands = [["pytest"]]
        "#};
    let got = format_doc_example(start);
    assert_snapshot!(got, @r#"
    [env_base.test]
    factors = [ [ "3.13", "3.14" ] ]
    deps = [ "pytest>=8" ]
    commands = [ [ "pytest" ] ]
    "#);
}
