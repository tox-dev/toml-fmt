use indoc::indoc;
use insta::assert_snapshot;

use super::assert_valid_toml;
use crate::{format_toml, Settings};

fn format_toml_helper(start: &str, indent: usize) -> String {
    let settings = Settings {
        column_width: 80,
        indent,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_valid_toml(&got);
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
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    # comment
    requires = [ "tox>=4.22" ]
    env_list = [
        "3.13",
        "3.12",
        "3.11",
        "3.10",
        "3.9",
        "docs",
        "fix",
        "pkg_meta",
        "type",
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
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    requires = [ "tox>=4.22" ]
    env_list = [ "test" ]

    [env_run_base]
    description = "run tests"
    "#);
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
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    [env_run_base]
    description = 'run "tests"'
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
    env_list = [ "lint", "test" ]

    [env.lint]
    commands = [ [ "ruff", "check" ] ]

    [env.test]
    commands = [ [ "pytest" ] ]
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
    set_env = { PYTHONPATH = "." }
    commands = [ [ "python", "-c", "print('hello')" ] ]
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
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    env_list = [
      "py311",
      "py310",
      "py39",
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
      "lint", # Run linter
      "test", # Run tests
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
    deps = [ "pytest" ]
    commands = [ [ "pytest" ] ]
    "#);
}

#[test]
fn test_format_with_boolean_values() {
    let start = indoc! {r#"
        skip_missing_interpreters = true
        parallel_show_output = false
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @"
    skip_missing_interpreters = true
    parallel_show_output = false
    ");
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
    pass_env = [ "HOME", "PATH" ]
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
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
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
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"env_list = [ "test" ]"#);
}

#[test]
fn test_format_with_narrow_column_width() {
    let start = indoc! {r#"
        description = "A very long description that exceeds the narrow column width"
        "#};
    let settings = Settings {
        column_width: 30,
        indent: 2,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    description = """\
      A very long description \
      that exceeds the narrow \
      column width\
      """
    "#);
}

#[test]
fn test_settings_new() {
    let settings = Settings::new(120, 4, String::from("short"), vec![], vec![], vec![], vec![]);
    assert_eq!(settings.column_width, 120);
    assert_eq!(settings.indent, 4);
}

#[test]
fn test_settings_default_values() {
    let settings = Settings::new(80, 2, String::from("short"), vec![], vec![], vec![], vec![]);
    assert_eq!(settings.column_width, 80);
    assert_eq!(settings.indent, 2);
}

#[test]
fn test_settings_field_access() {
    let settings = Settings {
        column_width: 100,
        indent: 3,
        table_format: String::from("long"),
        expand_tables: vec![String::from("env.test")],
        collapse_tables: vec![String::from("env.lint")],
        skip_wrap_for_keys: vec![String::from("*.commands")],
        pin_envs: vec![String::from("fix")],
    };
    assert_eq!(settings.column_width, 100);
    assert_eq!(settings.indent, 3);
    assert_eq!(settings.table_format, "long");
    assert_eq!(settings.expand_tables, vec!["env.test"]);
    assert_eq!(settings.collapse_tables, vec!["env.lint"]);
    assert_eq!(settings.skip_wrap_for_keys, vec!["*.commands"]);
    assert_eq!(settings.pin_envs, vec!["fix"]);
}

#[test]
fn test_format_toml_with_direct_settings() {
    let content = "env_list = ['a', 'b']";
    let settings = Settings::new(80, 2, String::from("short"), vec![], vec![], vec![], vec![]);
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

#[test]
fn test_array_multiline_expansion() {
    let start = indoc! {r#"
        [env_run_base]
        deps = ["pytest", "pytest-cov", "pytest-mock", "coverage", "hypothesis"]
        "#};
    let settings = Settings {
        column_width: 40,
        indent: 2,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got, "formatting should be idempotent");
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [
      "coverage",
      "hypothesis",
      "pytest",
      "pytest-cov",
      "pytest-mock",
    ]
    "#);
}

#[test]
fn test_long_string_wrapping() {
    let start = indoc! {r#"
        [env_run_base]
        description = "This is a very long description string that should be wrapped because it exceeds the column width limit"
        "#};
    let settings = Settings {
        column_width: 40,
        indent: 2,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got, "formatting should be idempotent");
    assert_snapshot!(got, @r#"
    [env_run_base]
    description = """\
      This is a very long description \
      string that should be wrapped \
      because it exceeds the column width \
      limit\
      """
    "#);
}

#[test]
fn test_table_collapse_short_format() {
    let start = indoc! {r#"
        [env.test]
        description = "run tests"
        [env.test.sub]
        value = 1
        "#};
    let settings = Settings {
        column_width: 80,
        indent: 2,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got, "formatting should be idempotent");
    assert_snapshot!(got, @r#"
    [env.test]
    description = "run tests"
    sub.value = 1
    "#);
}

#[test]
fn test_table_expand_long_format() {
    let start = indoc! {r#"
        [env.test]
        description = "run tests"
        sub.value = 1
        "#};
    let settings = Settings {
        column_width: 80,
        indent: 2,
        table_format: String::from("long"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got, "formatting should be idempotent");
    assert_snapshot!(got, @r#"
    [env.test]
    description = "run tests"

    [env.test.sub]
    value = 1
    "#);
}

#[test]
fn test_skip_wrap_for_keys() {
    let start = indoc! {r#"
        [env_run_base]
        description = "This is a very long description string that should be wrapped because it exceeds the column width"
        skip_me = "This is a very long string value that should NOT be wrapped because of skip config for this key"
        "#};
    let settings = Settings {
        column_width: 40,
        indent: 2,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![String::from("*.skip_me")],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got, "formatting should be idempotent");
    assert_snapshot!(got, @r#"
    [env_run_base]
    description = """\
      This is a very long description \
      string that should be wrapped \
      because it exceeds the column width\
      """
    skip_me = "This is a very long string value that should NOT be wrapped because of skip config for this key"
    "#);
}

#[test]
fn test_alias_normalization_root() {
    let start = indoc! {r#"
        envlist = ["test"]
        minversion = "4.0"
        skipsdist = true
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    min_version = "4.0"
    env_list = [ "test" ]
    no_package = true
    "#);
}

#[test]
fn test_alias_normalization_env() {
    let start = indoc! {r#"
        [env_run_base]
        basepython = "python3"
        setenv = { FOO = "bar" }
        passenv = ["HOME"]
        changedir = "src"
        usedevelop = true
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    base_python = "python3"
    package = "editable"
    pass_env = [ "HOME" ]
    set_env = { FOO = "bar" }
    change_dir = "src"
    "#);
}

#[test]
fn test_root_key_reorder() {
    let start = indoc! {r#"
        min_version = "4.0"
        env_list = ["test"]
        requires = ["tox>=4"]
        skip_missing_interpreters = true
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    min_version = "4.0"
    requires = [ "tox>=4" ]
    env_list = [ "test" ]
    skip_missing_interpreters = true
    "#);
}

#[test]
fn test_env_key_reorder() {
    let start = indoc! {r#"
        [env_run_base]
        commands = [["pytest"]]
        deps = ["pytest"]
        description = "run tests"
        pass_env = ["HOME"]
        set_env = { FOO = "bar" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    description = "run tests"
    deps = [ "pytest" ]
    pass_env = [ "HOME" ]
    set_env = { FOO = "bar" }
    commands = [ [ "pytest" ] ]
    "#);
}

#[test]
fn test_sort_deps() {
    let start = indoc! {r#"
        [env_run_base]
        deps = ["pytest-cov", "hypothesis", "pytest", "coverage"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [ "coverage", "hypothesis", "pytest", "pytest-cov" ]
    "#);
}

#[test]
fn test_sort_deps_pep508_normalization() {
    let start = indoc! {r#"
        [env_run_base]
        deps = ["Pytest-Cov>=3", "pytest>=7"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [ "pytest>=7", "pytest-cov>=3" ]
    "#);
}

#[test]
fn test_sort_pass_env() {
    let start = indoc! {r#"
        [env_run_base]
        pass_env = ["PATH", "HOME", "CI"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    pass_env = [ "CI", "HOME", "PATH" ]
    "#);
}

#[test]
fn test_sort_pass_env_with_replacement_objects() {
    let start = indoc! {r#"
        [env_run_base]
        pass_env = ["PATH", {replace = "default", name = "FOO"}, "HOME"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    pass_env = [ { replace = "default", name = "FOO" }, "HOME", "PATH" ]
    "#);
}

#[test]
fn test_sort_allowlist_externals() {
    let start = indoc! {r#"
        [env_run_base]
        allowlist_externals = ["make", "git", "bash"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    allowlist_externals = [ "bash", "git", "make" ]
    "#);
}

#[test]
fn test_sort_extras() {
    let start = indoc! {r#"
        [env_run_base]
        extras = ["testing", "docs", "all"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    extras = [ "all", "docs", "testing" ]
    "#);
}

#[test]
fn test_sort_depends() {
    let start = indoc! {r#"
        [env.coverage]
        depends = ["py312", "py311", "py310"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.coverage]
    depends = [ "py310", "py311", "py312" ]
    "#);
}

#[test]
fn test_commands_not_sorted() {
    let start = indoc! {r#"
        [env_run_base]
        commands = [["step2"], ["step1"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    commands = [ [ "step2" ], [ "step1" ] ]
    "#);
}

#[test]
fn test_sort_env_list_python_versions_descending() {
    let start = indoc! {r#"
        env_list = ["3.10", "3.12", "3.11", "3.13"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"env_list = [ "3.13", "3.12", "3.11", "3.10" ]"#);
}

#[test]
fn test_sort_env_list_mixed() {
    let start = indoc! {r#"
        env_list = ["lint", "3.12", "type", "3.13", "docs"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"env_list = [ "3.13", "3.12", "docs", "lint", "type" ]"#);
}

#[test]
fn test_sort_env_list_with_pin() {
    let start = indoc! {r#"
        env_list = ["lint", "3.12", "type", "3.13", "fix"]
        "#};
    let settings = Settings {
        column_width: 80,
        indent: 2,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![String::from("fix"), String::from("type")],
    };
    let got = format_toml(start, &settings);
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got, "formatting should be idempotent");
    assert_snapshot!(got, @r#"env_list = [ "fix", "type", "3.13", "3.12", "lint" ]"#);
}

#[test]
fn test_sort_env_list_py_prefix() {
    let start = indoc! {r#"
        env_list = ["py310", "py312", "py311"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"env_list = [ "py312", "py311", "py310" ]"#);
}

#[test]
fn test_normalize_requires() {
    let start = indoc! {r#"
        requires = ["Tox>=4.22", "virtualenv>=20"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"requires = [ "tox>=4.22", "virtualenv>=20" ]"#);
}

#[test]
fn test_sort_requires() {
    let start = indoc! {r#"
        requires = ["virtualenv>=20", "tox>=4"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"requires = [ "tox>=4", "virtualenv>=20" ]"#);
}

#[test]
fn test_env_pkg_base_ordering() {
    let start = indoc! {r#"
        requires = ["tox>=4"]

        [env.test]
        description = "test"

        [env_pkg_base]
        description = "pkg base"

        [env_run_base]
        description = "run base"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    requires = [ "tox>=4" ]

    [env_run_base]
    description = "run base"

    [env_pkg_base]
    description = "pkg base"

    [env.test]
    description = "test"
    "#);
}

#[test]
fn test_full_formatting_pipeline() {
    let start = indoc! {r#"
        envlist = ["lint", "3.12", "type", "3.13"]
        requires = ["Tox>=4.22"]
        minversion = "4.0"

        [env.type]
        commands = [["mypy", "src"]]
        description = "type check"

        [env_run_base]
        passenv = ["PATH", "HOME"]
        deps = ["pytest-cov", "pytest"]
        commands = [["pytest"]]
        description = "run tests"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    min_version = "4.0"
    requires = [ "tox>=4.22" ]
    env_list = [ "3.13", "3.12", "lint", "type" ]

    [env_run_base]
    description = "run tests"
    deps = [ "pytest", "pytest-cov" ]
    pass_env = [ "HOME", "PATH" ]
    commands = [ [ "pytest" ] ]

    [env.type]
    description = "type check"
    commands = [ [ "mypy", "src" ] ]
    "#);
}

#[test]
fn test_sort_env_list_compound_envs() {
    let start = indoc! {r#"
        env_list = ["py39-django", "py312-django", "py311-django", "lint"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"env_list = [ "py312-django", "py311-django", "py39-django", "lint" ]"#);
}

#[test]
fn test_sort_env_list_compound_pin() {
    let start = indoc! {r#"
        env_list = ["py312-django", "lint", "py311-django", "fix"]
        "#};
    let settings = Settings {
        column_width: 80,
        indent: 2,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![String::from("fix")],
    };
    let got = format_toml(start, &settings);
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got, "formatting should be idempotent");
    assert_snapshot!(got, @r#"env_list = [ "fix", "py312-django", "py311-django", "lint" ]"#);
}

#[test]
fn test_sort_env_list_pypy() {
    let start = indoc! {r#"
        env_list = ["pypy39", "py312", "pypy310", "py311"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"env_list = [ "py312", "py311", "pypy310", "pypy39" ]"#);
}

#[test]
fn test_sort_env_list_major_only() {
    let start = indoc! {r#"
        env_list = ["py3", "pypy3", "lint", "py2"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"env_list = [ "py3", "py2", "pypy3", "lint" ]"#);
}

#[test]
fn test_sort_env_list_dotted_versions() {
    let start = indoc! {r#"
        env_list = ["py3.11", "py3.13", "py3.12", "pypy3.10", "pypy3.9"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"env_list = [ "py3.13", "py3.12", "py3.11", "pypy3.10", "pypy3.9" ]"#);
}

#[test]
fn test_use_develop_true_to_package_editable() {
    let start = indoc! {r#"
        [env_run_base]
        description = "test"
        use_develop = true
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    description = "test"
    package = "editable"
    "#);
}

#[test]
fn test_use_develop_false_kept() {
    let start = indoc! {r#"
        [env_run_base]
        description = "test"
        use_develop = false
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    description = "test"
    use_develop = false
    "#);
}

#[test]
fn test_use_develop_true_with_existing_package() {
    let start = indoc! {r#"
        [env_run_base]
        use_develop = true
        package = "wheel"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    package = "wheel"
    "#);
}

#[test]
fn test_sort_dependency_groups() {
    let start = indoc! {r#"
        [env_run_base]
        dependency_groups = ["test", "dev", "docs"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    dependency_groups = [ "dev", "docs", "test" ]
    "#);
}

#[test]
fn test_sort_constraints() {
    let start = indoc! {r#"
        [env_run_base]
        constraints = ["urllib3<2", "certifi>=2023"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    constraints = [ "certifi>=2023", "urllib3<2" ]
    "#);
}

#[test]
fn test_sort_labels() {
    let start = indoc! {r#"
        [env.test]
        labels = ["ci", "test", "all"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    labels = [ "all", "ci", "test" ]
    "#);
}

#[test]
fn test_env_dotted_keys_expand_to_tables() {
    let start = indoc! {r#"
        [env]
        fix.description = "fix"
        fix.skip_install = true
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.fix]
    description = "fix"
    skip_install = true
    "#);
}

#[test]
fn test_env_tables_not_collapsed_in_short_format() {
    let start = indoc! {r#"
        [env.fix]
        description = "fix"
        skip_install = true

        [env.test]
        description = "test"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.fix]
    description = "fix"
    skip_install = true

    [env.test]
    description = "test"
    "#);
}

#[test]
fn test_env_sub_tables_still_collapse_in_short_format() {
    let start = indoc! {r#"
        [env.test]
        description = "run tests"

        [env.test.sub]
        value = 1
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    description = "run tests"
    sub.value = 1
    "#);
}

#[test]
fn test_env_quoted_key_with_dot_not_collapsed() {
    let start = indoc! {r#"
        [env."3.13t"]
        base_python = "3.13t"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env."3.13t"]
    base_python = "3.13t"
    "#);
}

#[test]
fn test_env_quoted_key_dotted_expand() {
    let start = indoc! {r#"
        [env]
        "3.13t".base_python = "3.13t"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env."3.13t"]
    base_python = "3.13t"
    "#);
}

#[test]
fn test_deps_r_c_flags_not_normalized() {
    let start = indoc! {r#"
        [env.test]
        deps = ["-r requirements-test.txt", "-c constraints.txt", "pytest"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    deps = [ "-c constraints.txt", "-r requirements-test.txt", "pytest" ]
    "#);
}

#[test]
fn test_constraints_normalize_and_sort() {
    let start = indoc! {r#"
        [env.test]
        constraints = ["urllib3<2", "Certifi>=2023"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    constraints = [ "certifi>=2023", "urllib3<2" ]
    "#);
}

#[test]
fn test_constraints_r_c_flags_not_normalized() {
    let start = indoc! {r#"
        [env.test]
        constraints = ["-c base-constraints.txt", "-r requirements.txt", "urllib3<2"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    constraints = [ "-c base-constraints.txt", "-r requirements.txt", "urllib3<2" ]
    "#);
}

#[test]
fn test_env_base_key_ordering() {
    let start = indoc! {r#"
        [env_base.test]
        commands = [["pytest"]]
        deps = ["pytest"]
        description = "run tests"
        factors = [["py312", "py313"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_base.test]
    factors = [ [ "py312", "py313" ] ]
    description = "run tests"
    deps = [ "pytest" ]
    commands = [ [ "pytest" ] ]
    "#);
}

#[test]
fn test_env_base_table_ordering() {
    let start = indoc! {r#"
        requires = ["tox>=4"]

        [env.lint]
        description = "lint"

        [env_base.test]
        factors = [["py312", "py313"]]
        description = "test"

        [env_run_base]
        description = "base"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    requires = [ "tox>=4" ]

    [env_run_base]
    description = "base"

    [env.lint]
    description = "lint"

    [env_base.test]
    factors = [ [ "py312", "py313" ] ]
    description = "test"
    "#);
}

#[test]
fn test_env_base_alias_normalization() {
    let start = indoc! {r#"
        [env_base.test]
        factors = [["py312"]]
        basepython = "python3"
        passenv = ["HOME"]
        setenv = { FOO = "bar" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_base.test]
    factors = [ [ "py312" ] ]
    base_python = "python3"
    pass_env = [ "HOME" ]
    set_env = { FOO = "bar" }
    "#);
}

#[test]
fn test_env_base_deps_normalization() {
    let start = indoc! {r#"
        [env_base.test]
        factors = [["py312"]]
        deps = ["Pytest-Cov>=3", "-r requirements.txt", "pytest>=7"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_base.test]
    factors = [ [ "py312" ] ]
    deps = [ "-r requirements.txt", "pytest>=7", "pytest-cov>=3" ]
    "#);
}

#[test]
fn test_env_list_with_product_expansion() {
    let start = indoc! {r#"
        env_list = [
            "lint",
            { product = [["py312", "py313"], ["django42"]] },
            "docs",
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      { product = [ [ "py312", "py313" ], [ "django42" ] ] },
      "docs",
      "lint",
    ]
    "#);
}

#[test]
fn test_new_env_key_ordering() {
    let start = indoc! {r#"
        [env.test]
        commands = [["pytest"]]
        commands_retry = 2
        fail_fast = true
        recreate_commands = [["rm", "-rf", ".cache"]]
        recreate = true
        pylock = "pylock.toml"
        deps = ["pytest"]
        virtualenv_spec = "virtualenv<20.22.0"
        default_base_python = ["python3.12"]
        extra_setup_commands = [["echo", "setup"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    default_base_python = [ "python3.12" ]
    virtualenv_spec = "virtualenv<20.22.0"
    deps = [ "pytest" ]
    pylock = "pylock.toml"
    recreate = true
    recreate_commands = [ [ "rm", "-rf", ".cache" ] ]
    fail_fast = true
    commands_retry = 2
    extra_setup_commands = [ [ "echo", "setup" ] ]
    commands = [ [ "pytest" ] ]
    "#);
}

#[test]
fn test_env_multiple_quoted_keys_not_collapsed() {
    let start = indoc! {r#"
        [env."3.13t"]
        base_python = "3.13t"

        [env."3.14t"]
        base_python = "3.14t"

        [env.fix]
        description = "fix"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.fix]
    description = "fix"

    [env."3.13t"]
    base_python = "3.13t"

    [env."3.14t"]
    base_python = "3.14t"
    "#);
}

#[test]
fn test_inline_table_reorder_substitution() {
    let start = indoc! {r#"
        [env_run_base]
        set_env.UV_INDEX_URL = { default = "https://pypi.org/simple", name = "UV_INDEX_URL", replace = "env" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env.UV_INDEX_URL = { replace = "env", name = "UV_INDEX_URL", default = "https://pypi.org/simple" }
    "#);
}

#[test]
fn test_inline_table_reorder_substitution_ref() {
    let start = indoc! {r#"
        [env_pkg_base]
        set_env = { of = ["env_run_base", "set_env"], replace = "ref" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_pkg_base]
    set_env = { replace = "ref", of = [ "env_run_base", "set_env" ] }
    "#);
}

#[test]
fn test_inline_table_reorder_posargs() {
    let start = indoc! {r#"
        [env.test]
        commands = [["pytest", { default = [], extend = true, replace = "posargs" }]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    commands = [
      [ "pytest", { replace = "posargs", default = [], extend = true } ]
    ]
    "#);
}

#[test]
fn test_inline_table_reorder_range() {
    let start = indoc! {r#"
        env_list = [{ product = [{ stop = 14, prefix = "py3", start = 12 }] }]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [ { product = [ { prefix = "py3", start = 12, stop = 14 } ] } ]
    "#);
}

#[test]
fn test_inline_table_reorder_product() {
    let start = indoc! {r#"
        env_list = [{ exclude = ["py312-django50"], product = [["py312", "py313"], ["django42", "django50"]] }]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      { product = [ [ "py312", "py313" ], [ "django42", "django50" ] ], exclude = [
        "py312-django50"
      ] },
    ]
    "#);
}

#[test]
fn test_inline_table_already_ordered() {
    let start = indoc! {r#"
        [env_run_base]
        set_env.FOO = { replace = "env", name = "FOO", default = "bar" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env.FOO = { replace = "env", name = "FOO", default = "bar" }
    "#);
}

#[test]
fn test_inline_table_unknown_schema_not_reordered() {
    let start = indoc! {r#"
        [env.test]
        set_env = { ZZZ = "last", AAA = "first" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    set_env = { ZZZ = "last", AAA = "first" }
    "#);
}

#[test]
fn test_inline_table_reorder_if_conditional() {
    let start = indoc! {r#"
        [env.test]
        deps = [{ "else" = "no", extend = true, then = ["Django>=5.0"], condition = "factor.django50", replace = "if" }]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    deps = [
      { replace = "if", condition = "factor.django50", then = [ "Django>=5.0" ], else = "no", extend = true },
    ]
    "#);
}

#[test]
fn test_inline_table_reorder_ref() {
    let start = indoc! {r#"
        [env.test]
        extras = [{ extend = true, key = "extras", env = "src", replace = "ref" }]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    extras = [ { replace = "ref", env = "src", key = "extras", extend = true } ]
    "#);
}

#[test]
fn test_inline_table_reorder_glob() {
    let start = indoc! {r#"
        [env.test]
        commands = [["twine", "upload", { extend = true, pattern = "dist/*.whl", replace = "glob" }]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    commands = [
      [
        "twine",
        "upload",
        { replace = "glob", pattern = "dist/*.whl", extend = true }
      ],
    ]
    "#);
}

#[test]
fn test_inline_table_reorder_value_marker() {
    let start = indoc! {r#"
        [env_run_base]
        set_env.LINUX_VAR = { marker = "sys_platform == 'linux'", value = "1" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env.LINUX_VAR = { value = "1", marker = "sys_platform == 'linux'" }
    "#);
}

#[test]
fn test_inline_table_reorder_env_with_marker() {
    let start = indoc! {r#"
        [env_run_base]
        set_env.X = { marker = "sys_platform == 'linux'", default = "fallback", name = "MY_VAR", replace = "env" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env.X = { replace = "env", name = "MY_VAR", default = "fallback", marker = "sys_platform == 'linux'" }
    "#);
}
