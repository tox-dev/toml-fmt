use indoc::indoc;
use insta::assert_snapshot;

use crate::{format_toml, Settings};

fn format_toml_helper(start: &str, indent: usize) -> String {
    let settings = Settings {
        column_width: 80,
        indent,
    };
    let got = format_toml(start, &settings);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got, "formatting should be idempotent");
    got
}

#[test]
fn test_format_toml_simple() {
    let start = indoc! {r#"
        requires = ["tox>=4.22"]
        env_list = ["3.13", "3.12"]
        skip_missing_interpreters = true

        [env_run_base]
        description = "run the tests with pytest under {env_name}"
        commands = [ ["pytest"] ]

        [env.type]
        description = "run type check on code base"
        commands = [["mypy", "src{/}tox_toml_fmt"], ["mypy", "tests"]]
    "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    requires = [ "tox>=4.22" ]
    env_list = [ "3.13", "3.12" ]
    skip_missing_interpreters = true

    [env_run_base]
    description = "run the tests with pytest under {env_name}"
    commands = [ [ "pytest" ] ]

    [env.type]
    description = "run type check on code base"
    commands = [ [ "mypy", "src{/}tox_toml_fmt" ], [ "mypy", "tests" ] ]
    "#);
}

#[test]
fn test_format_toml_empty() {
    let start = indoc! {r""};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @"");
}

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
    assert_snapshot!(got, @r#"
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
            "pkg_meta"
        ]
        "#);
}

#[test]
fn test_string_quote_normalization() {
    let start = indoc! {r#"
        requires = ['tox>=4.22']
        env_list = ['test']

        [env_run_base]
        description = 'run tests'
        "#};
    let settings = Settings {
        column_width: 80,
        indent: 2,
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r###"
    requires = [ "tox>=4.22" ]
    env_list = [ "test" ]

    [env_run_base]
    description = "run tests"
    "###);
}

#[test]
fn test_string_with_double_quote_preserved() {
    let start = indoc! {r#"
        [env_run_base]
        description = "run \"tests\""
        "#};
    let settings = Settings {
        column_width: 80,
        indent: 2,
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    [env_run_base]
    description = "run \"tests\""
    "#);
}

#[test]
fn test_format_with_multiple_env_sections() {
    let start = indoc! {r#"
        requires = ["tox>=4"]
        env_list = ["test", "lint"]

        [env.test]
        commands = [["pytest"]]

        [env.lint]
        commands = [["ruff", "check"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    requires = [ "tox>=4" ]
    env_list = [ "test", "lint" ]

    [env.test]
    commands = [ [ "pytest" ] ]

    [env.lint]
    commands = [ [ "ruff", "check" ] ]
    "#);
}

#[test]
fn test_format_with_nested_arrays() {
    let start = indoc! {r#"
        [env_run_base]
        commands = [["python", "-c", "print('hello')"]]
        set_env = {PYTHONPATH = "."}
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    commands = [ [ "python", "-c", "print('hello')" ] ]
    set_env = { PYTHONPATH = "." }
    "#);
}

#[test]
fn test_format_with_comments() {
    let start = indoc! {r#"
        # Main config comment
        requires = ["tox>=4"]

        # Environment settings
        [env_run_base]
        description = "base env"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    # Main config comment
    requires = [ "tox>=4" ]

    # Environment settings
    [env_run_base]
    description = "base env"
    "#);
}

#[test]
fn test_format_with_multiline_arrays() {
    let start = indoc! {r#"
        env_list = [
          "py39",
          "py310",
          "py311",
        ]
        "#};
    let settings = Settings {
        column_width: 40,
        indent: 2,
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    env_list = [
      "py39",
      "py310",
      "py311",
    ]
    "#);
}

#[test]
fn test_format_with_inline_comments() {
    let start = indoc! {r#"
        env_list = [
          "test",  # Run tests
          "lint",  # Run linter
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      "test", # Run tests
      "lint", # Run linter
    ]
    "#);
}

#[test]
fn test_format_preserves_key_order_in_section() {
    let start = indoc! {r#"
        [env.test]
        description = "run tests"
        commands = [["pytest"]]
        deps = ["pytest"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    description = "run tests"
    commands = [ [ "pytest" ] ]
    deps = [ "pytest" ]
    "#);
}

#[test]
fn test_format_with_boolean_values() {
    let start = indoc! {r#"
        skip_missing_interpreters = true
        parallel_show_output = false
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    skip_missing_interpreters = true
    parallel_show_output = false
    "#);
}

#[test]
fn test_format_with_special_characters_in_strings() {
    let start = indoc! {r#"
        [env_run_base]
        description = "run with {env_name} - uses Python's stdlib"
        pass_env = ["PATH", "HOME"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    description = "run with {env_name} - uses Python's stdlib"
    pass_env = [ "PATH", "HOME" ]
    "#);
}

#[test]
fn test_idempotent_formatting() {
    let start = indoc! {r#"
        requires = ["tox>=4.22"]
        env_list = ["3.13", "3.12"]

        [env_run_base]
        description = "test environment"
        commands = [["pytest", "-v"]]
        "#};
    let settings = Settings {
        column_width: 80,
        indent: 2,
    };
    let first = format_toml(start, &settings);
    let second = format_toml(&first, &settings);
    let third = format_toml(&second, &settings);
    assert_eq!(second, first, "Second pass should match first");
    assert_eq!(third, second, "Third pass should match second");
}

#[test]
fn test_format_with_large_indent() {
    let start = indoc! {r#"
        env_list = ["test"]
        "#};
    let settings = Settings {
        column_width: 80,
        indent: 4,
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    env_list = [ "test" ]
    "#);
}

#[test]
fn test_format_with_narrow_column_width() {
    let start = indoc! {r#"
        description = "A very long description that exceeds the narrow column width"
        "#};
    let settings = Settings {
        column_width: 30,
        indent: 2,
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"description = "A very long description that exceeds the narrow column width""#);
}

#[test]
fn test_settings_new() {
    let settings = Settings::new(120, 4);
    assert_eq!(settings.column_width, 120);
    assert_eq!(settings.indent, 4);
}

#[test]
fn test_settings_default_values() {
    let settings = Settings::new(80, 2);
    assert_eq!(settings.column_width, 80);
    assert_eq!(settings.indent, 2);
}

#[test]
fn test_settings_field_access() {
    let settings = Settings {
        column_width: 100,
        indent: 3,
    };
    assert_eq!(settings.column_width, 100);
    assert_eq!(settings.indent, 3);
}

#[test]
fn test_format_toml_with_direct_settings() {
    let content = "env_list = ['a', 'b']";
    let settings = Settings::new(80, 2);
    let result = format_toml(content, &settings);
    assert!(result.contains("env_list"));
    assert!(result.contains("\"a\""));
    assert!(result.contains("\"b\""));
}

#[test]
fn test_lib_module_registration() {
    use pyo3::types::PyAnyMethods;

    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let module = pyo3::types::PyModule::new(py, "_lib").unwrap();
        crate::_lib(&module.as_borrowed()).unwrap();

        assert!(module.hasattr("format_toml").unwrap());
        assert!(module.hasattr("Settings").unwrap());
    });
}

#[test]
fn test_format_with_nested_inline_tables() {
    let start = indoc! {r#"
        [env_run_base]
        set_env = {OUTER = {INNER = "value"}}
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env = { OUTER = { INNER = "value" } }
    "#);
}

#[test]
fn test_format_with_array_of_inline_tables() {
    let start = indoc! {r#"
        [env_run_base]
        configs = [{name = "a"}, {name = "b"}]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    configs = [ { name = "a" }, { name = "b" } ]
    "#);
}
