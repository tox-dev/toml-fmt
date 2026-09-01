use indoc::indoc;
use insta::assert_snapshot;
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyDict, PyDictMethods};
use pyo3::{Bound, PyResult, Python};

use super::{assert_valid_toml, default_settings, evaluate_settings};
use _tox_toml_fmt::{format_toml, Settings};

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
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
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
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
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
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
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
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
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
        ..default_settings()
    };
    let first = format_toml(start, &settings).expect("the formatter reads its own output");
    let second = format_toml(&first, &settings).expect("the formatter reads its own output");
    let third = format_toml(&second, &settings).expect("the formatter reads its own output");
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
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
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
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
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
    let settings = new_settings(Settings {
        column_width: 120,
        indent: 4,
        expand_tables: vec![String::from("env.test")],
        ..default_settings()
    })
    .expect("the settings name tables");
    assert_eq!(settings.column_width, 120);
    assert_eq!(settings.indent, 4);
}

#[test]
fn test_settings_new_rejects_an_unexpected_keyword() -> PyResult<()> {
    Python::attach(|python| {
        let kwargs = PyDict::new(python);
        kwargs.set_item("unexpected", true)?;

        let error = Settings::new(Some(&kwargs))
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default();
        assert_eq!(error, "TypeError: unexpected keyword argument: 'unexpected'");
        Ok(())
    })
}

#[test]
fn test_settings_new_requires_keyword_arguments() {
    let error = Settings::new(None)
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default();

    assert_eq!(error, "TypeError: missing keyword argument: 'column_width'");
}

#[test]
fn test_settings_new_requires_every_keyword() -> PyResult<()> {
    Python::attach(|python| {
        for name in [
            "column_width",
            "indent",
            "table_format",
            "sub_table_spacing",
            "separate_root_table",
            "expand_tables",
            "collapse_tables",
            "skip_wrap_for_keys",
            "pin_envs",
        ] {
            let kwargs = settings_kwargs(python, default_settings())?;
            kwargs.del_item(name)?;

            let error = Settings::new(Some(&kwargs))
                .err()
                .map(|error| error.to_string())
                .unwrap_or_default();
            assert_eq!(error, format!("TypeError: missing keyword argument: '{name}'"));
        }
        Ok(())
    })
}

#[test]
fn test_settings_new_rejects_every_wrong_value_type() -> PyResult<()> {
    Python::attach(|python| {
        for name in [
            "column_width",
            "indent",
            "table_format",
            "sub_table_spacing",
            "separate_root_table",
            "expand_tables",
            "collapse_tables",
            "skip_wrap_for_keys",
            "pin_envs",
        ] {
            let kwargs = settings_kwargs(python, default_settings())?;
            kwargs.set_item(name, python.None())?;

            let error = Settings::new(Some(&kwargs)).err().expect("None is not a setting value");
            assert!(error.is_instance_of::<PyTypeError>(python));
        }
        Ok(())
    })
}

#[test]
fn test_settings_new_rejects_a_non_string_keyword() -> PyResult<()> {
    Python::attach(|python| {
        let kwargs = settings_kwargs(python, default_settings())?;
        kwargs.set_item(0, true)?;

        let error = Settings::new(Some(&kwargs)).err().expect("keyword names are strings");
        assert!(error.is_instance_of::<PyTypeError>(python));
        Ok(())
    })
}

/// A pin names an environment and a pattern names a key, so a list holding neither is told.
#[test]
fn test_settings_reject_a_name_written_as_nothing() {
    for (at, setting) in [(7, "skip_wrap_for_keys"), (8, "pin_envs")] {
        let mut lists = vec![Vec::new(); 4];
        lists[at - 5] = vec![String::from(" ")];
        let built = new_settings(Settings {
            expand_tables: lists[0].clone(),
            collapse_tables: lists[1].clone(),
            skip_wrap_for_keys: lists[2].clone(),
            pin_envs: lists[3].clone(),
            ..default_settings()
        });

        let why = built.err().map(|error| error.to_string()).unwrap_or_default();
        assert!(why.contains(setting), "{why}");
    }
}

/// A selector names a table the way TOML names one, so a setting asking for a name no key spells is
/// told rather than read as some other table.
#[test]
fn test_settings_reject_a_selector_that_names_no_table() {
    let built = new_settings(Settings {
        expand_tables: vec![String::from("env.\"test")],
        ..default_settings()
    });

    let why = built.err().map(|error| error.to_string()).unwrap_or_default();
    assert!(why.contains("expand_tables: env.\"test is not a table name"), "{why}");
}

#[test]
fn test_settings_default_values() {
    let settings = new_settings(default_settings()).expect("the settings name tables");
    assert_eq!(settings.column_width, 80);
    assert_eq!(settings.indent, 2);
}

#[test]
fn test_settings_field_access() {
    let settings = Settings {
        column_width: 100,
        indent: 3,
        table_format: String::from("long"),
        sub_table_spacing: String::from("\n"),
        separate_root_table: String::from("\n\n"),
        expand_tables: vec![String::from("env.test")],
        collapse_tables: vec![String::from("env.lint")],
        skip_wrap_for_keys: vec![String::from("*.commands")],
        pin_envs: vec![String::from("fix")],
    };
    assert_eq!(settings.column_width, 100);
    assert_eq!(settings.indent, 3);
    assert_eq!(settings.table_format, "long");
    assert_eq!(settings.sub_table_spacing, "\n");
    assert_eq!(settings.separate_root_table, "\n\n");
    assert_eq!(settings.expand_tables, vec!["env.test"]);
    assert_eq!(settings.collapse_tables, vec!["env.lint"]);
    assert_eq!(settings.skip_wrap_for_keys, vec!["*.commands"]);
    assert_eq!(settings.pin_envs, vec!["fix"]);
}

#[test]
fn test_format_toml_with_direct_settings() {
    let content = "env_list = ['a', 'b']";
    let settings = new_settings(default_settings()).expect("the settings name tables");
    let result = format_toml(content, &settings).expect("the formatter reads its own output");
    assert!(result.contains("env_list"));
    assert!(result.contains("\"a\""));
    assert!(result.contains("\"b\""));
}

#[cfg(feature = "extension-module")]
#[test]
fn test_lib_module_registration() {
    use pyo3::types::PyAnyMethods;

    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let module = pyo3::types::PyModule::new(py, "_lib").unwrap();
        _tox_toml_fmt::_lib(&module.as_borrowed()).unwrap();

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
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings).expect("the formatter reads its own output");
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
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings).expect("the formatter reads its own output");
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
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings).expect("the formatter reads its own output");
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
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings).expect("the formatter reads its own output");
    assert_eq!(second, got, "formatting should be idempotent");
    assert_snapshot!(got, @r#"
    [env.test]
    description = "run tests"
    [env.test.sub]
    value = 1
    "#);
}

#[test]
fn test_expanded_sub_tables_follow_env_key_order() {
    let start = indoc! {r#"
        [env.py313]
        description = "run tests"
        custom.a = 1
        set_env.A = "1"
        [labels.sub]
        b = 2
        "#};
    let settings = Settings {
        table_format: String::from("long"),
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings).expect("the formatter reads its own output");
    assert_eq!(second, got, "formatting should be idempotent");
    assert_snapshot!(got, @r#"
    [env.py313]
    description = "run tests"
    [env.py313.set_env]
    A = "1"
    [env.py313.custom]
    a = 1

    [labels.sub]
    b = 2
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
        skip_wrap_for_keys: vec![String::from("*.skip_me")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings).expect("the formatter reads its own output");
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
        base_python_file = [".python-version"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    description = "run tests"
    base_python_file = [ ".python-version" ]
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

/// `env_list` is written newest interpreter first, CPython before PyPy, and everything the two do
/// not name after them by name. A compound name is placed by the first part of it that names one.
#[test]
fn test_env_list_is_written_newest_interpreter_first() {
    for (written, expected) in [
        (
            r#"[ "3.10", "3.12", "3.11", "3.13" ]"#,
            r#"[ "3.13", "3.12", "3.11", "3.10" ]"#,
        ),
        (
            r#"[ "lint", "3.12", "type", "3.13", "docs" ]"#,
            r#"[ "3.13", "3.12", "docs", "lint", "type" ]"#,
        ),
        (r#"[ "py310", "py312", "py311" ]"#, r#"[ "py312", "py311", "py310" ]"#),
        (
            r#"[ "py39-django", "py312-django", "py311-django", "lint" ]"#,
            r#"[ "py312-django", "py311-django", "py39-django", "lint" ]"#,
        ),
        (
            r#"[ "pypy39", "py312", "pypy310", "py311" ]"#,
            r#"[ "py312", "py311", "pypy310", "pypy39" ]"#,
        ),
        (
            r#"[ "py3", "pypy3", "lint", "py2" ]"#,
            r#"[ "py3", "py2", "pypy3", "lint" ]"#,
        ),
        (
            r#"[ "py3.11", "py3.13", "py3.12", "pypy3.10", "pypy3.9" ]"#,
            r#"[ "py3.13", "py3.12", "py3.11", "pypy3.10", "pypy3.9" ]"#,
        ),
        // a name the version grammar does not read is a name like any other
        (
            r#"[ "3.15", "3.15t", "3.14", "docs" ]"#,
            r#"[ "3.15", "3.14", "3.15t", "docs" ]"#,
        ),
    ] {
        assert_eq!(
            format_toml_helper(&format!("env_list = {written}\n"), 2),
            format!("env_list = {expected}\n"),
            "{written}"
        );
    }
}

/// An entry that generates environments names none of them, and what it generates is read where it
/// sits, so it holds the place the file gave it while the names around it move.
#[test]
fn test_a_generated_env_list_entry_holds_its_place() {
    assert_eq!(
        format_toml_helper("env_list = [ \"b\", { product = [ [ \"x\", \"y\" ] ] }, \"a\" ]\n", 2),
        "env_list = [ \"a\", { product = [ [ \"x\", \"y\" ] ] }, \"b\" ]\n"
    );
}

/// A pin puts the environments it names at the head of the list, in the order the pin gives them.
#[test]
fn test_a_pin_writes_the_environments_it_names_first() {
    let settings = Settings {
        pin_envs: vec![String::from("type"), String::from("fix")],
        ..default_settings()
    };

    assert_eq!(
        evaluate_settings("env_list = [ \"lint\", \"fix\", \"3.13\", \"type\" ]\n", &settings),
        "env_list = [ \"type\", \"fix\", \"3.13\", \"lint\" ]\n"
    );
}

/// A pin puts the environments it names at the head of `env_list` and writes their tables first,
/// so the run order and the file read the same way round.
#[test]
fn test_a_pin_leads_both_the_list_and_the_tables() {
    let start = indoc! {r#"
        env_list = ["lint", "fix"]

        [env.lint]
        description = "lint"

        [env.fix]
        description = "fix"
        "#};
    let settings = Settings {
        pin_envs: vec![String::from("fix")],
        ..default_settings()
    };

    assert_snapshot!(evaluate_settings(start, &settings), @r#"
    env_list = [ "fix", "lint" ]

    [env.fix]
    description = "fix"

    [env.lint]
    description = "lint"
    "#);
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

/// tox reads `use_develop` before `package` and installs an editable package whatever `package`
/// says, so the key that stays holds the mode the environment ran with.
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
    package = "editable"
    "#);
    assert_eq!(format_toml_helper(&got, 2), got);
    assert_snapshot!(
        format_toml_helper("[env_run_base]\nuse_develop = true\npackage = \"sdist\"\n", 2),
        @r#"
    [env_run_base]
    package = "editable"
    "#
    );
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

/// Each constraint names a file, so the list is left as written.
#[test]
fn test_constraints_are_left_as_written() {
    let start = indoc! {r#"
        [env_run_base]
        constraints = ["urllib3<2", "certifi>=2023"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    constraints = [ "urllib3<2", "certifi>=2023" ]
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
    deps = [ "-r requirements-test.txt", "-c constraints.txt", "pytest" ]
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
    constraints = [ "urllib3<2", "Certifi>=2023" ]
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
    deps = [ "pytest-cov>=3", "-r requirements.txt", "pytest>=7" ]
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
      "docs",
      { product = [ [ "py312", "py313" ], [ "django42" ] ] },
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
    [env."3.13t"]
    base_python = "3.13t"

    [env."3.14t"]
    base_python = "3.14t"

    [env.fix]
    description = "fix"
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
    assert_snapshot!(got, @r#"env_list = [ { product = [ { prefix = "py3", start = 12, stop = 14 } ] } ]"#);
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

#[test]
fn test_deps_editable_local_path_not_normalized() {
    let start = indoc! {r#"
        [env.test]
        deps = ["-e ./opentelemetry-python-lineage-api[dtp]", "-e ./opentelemetry-python-lineage-sdk[dtp,fastapi]"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    deps = [
      "-e ./opentelemetry-python-lineage-api[dtp]",
      "-e ./opentelemetry-python-lineage-sdk[dtp,fastapi]",
    ]
    "#);
}

#[test]
fn test_deps_tox_substitution_not_normalized() {
    let start = indoc! {r#"
        [env.test]
        deps = ["{tox_root}/subproject[extras]", "pytest"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    deps = [ "{tox_root}/subproject[extras]", "pytest" ]
    "#);
}

#[test]
fn test_deps_editable_with_tox_substitution_not_normalized() {
    let start = indoc! {r#"
        [env.test]
        deps = ["-e {tox_root}/subproject[extras]", "pytest"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    deps = [ "-e {tox_root}/subproject[extras]", "pytest" ]
    "#);
}

/// A path names a distribution to install from disk rather than a requirement to resolve, so the
/// requirement rules leave it as the file wrote it.
#[test]
fn test_a_path_dependency_is_left_as_written() {
    for (name, written) in [
        ("relative", "./my.package[test]"),
        ("parent", "../my.package[test]"),
        ("absolute", "/opt/my.package[test]"),
    ] {
        let start = format!("[env.test]\ndeps = [\"{written}\", \"pytest\"]\n");

        assert_eq!(
            format_toml_helper(&start, 2),
            format!("[env.test]\ndeps = [ \"{written}\", \"pytest\" ]\n"),
            "{name}"
        );
    }
}

#[test]
fn test_constraints_editable_and_paths_not_normalized() {
    let start = indoc! {r#"
        [env.test]
        constraints = ["-e ./local_pkg[dev]", "{tox_root}/constraints.txt"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.test]
    constraints = [ "-e ./local_pkg[dev]", "{tox_root}/constraints.txt" ]
    "#);
}

#[test]
fn test_sub_table_spacing_blank_line() {
    let start = indoc! {r#"
        env_list = ["test", "lint"]

        [env_run_base]
        description = "base"

        [env.test]
        description = "test"

        [env.lint]
        description = "lint"
        "#};
    let settings = Settings {
        sub_table_spacing: String::from("\n"),
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    env_list = [ "lint", "test" ]

    [env_run_base]
    description = "base"

    [env.lint]
    description = "lint"

    [env.test]
    description = "test"
    "#);
}

#[test]
fn test_env_list_that_is_not_an_array() {
    let start = indoc! {r#"
        env_list = "docs"

        [env.docs]
        description = "docs"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = "docs"

    [env.docs]
    description = "docs"
    "#);
}

#[test]
fn test_collapse_tables_beats_the_long_format() {
    let start = indoc! {r#"
        [env.docs]
        set_env.A = "1"
        "#};
    let settings = Settings {
        table_format: String::from("long"),
        collapse_tables: vec![String::from("env.docs")],
        ..default_settings()
    };
    assert_snapshot!(format_toml(start, &settings).expect("the formatter reads its own output"), @r#"
    [env.docs]
    set_env.A = "1"
    "#);
}

#[test]
fn test_expand_tables_beats_the_short_format() {
    let start = indoc! {r#"
        [env.docs]
        set_env.A = "1"
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("env.docs")],
        ..default_settings()
    };
    assert_snapshot!(format_toml(start, &settings).expect("the formatter reads its own output"), @r#"
    [env.docs.set_env]
    A = "1"
    "#);
}

#[test]
fn test_source_that_is_not_toml_is_handed_back_as_written() {
    assert!(format_toml("env_list = [\n", &default_settings()).is_err());
}

#[test]
fn test_deps_keep_a_requirement_that_cannot_be_read() {
    let start = indoc! {r#"
        [env.test]
        deps = ["good >= 1.0.0", "!! not a requirement !!"]
        "#};
    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.test]
    deps = [ "good>=1", "!! not a requirement !!" ]
    "#);
}

/// `use_develop = true` is the boolean, not text or a container written with those letters in it.
#[test]
fn test_use_develop_upgrades_only_the_boolean() {
    let upgraded = |value: &str| format_toml_helper(&format!("[env.test]\nuse_develop = {value}\n"), 2);

    assert_snapshot!(upgraded("true"), @r#"
    [env.test]
    package = "editable"
    "#);
    assert_snapshot!(upgraded("false"), @r#"
    [env.test]
    use_develop = false
    "#);
    assert_snapshot!(upgraded("\"true\""), @r#"
    [env.test]
    use_develop = "true"
    "#);
    assert_snapshot!(upgraded("[ true ]"), @r#"
    [env.test]
    use_develop = [ true ]
    "#);
    assert_snapshot!(upgraded("{ a = true }"), @r#"
    [env.test]
    use_develop = { a = true }
    "#);
}

#[test]
fn test_use_develop_carries_the_comment_beside_it() {
    assert_snapshot!(
        format_toml_helper("[env.test]\nuse_develop = true  # develop install\n", 2),
        @r#"
    [env.test]
    package = "editable"  # develop install
    "#
    );
    assert_snapshot!(
        format_toml_helper("[env.test]\nuse_develop = true  # develop install\npackage = \"wheel\"\n", 2),
        @r#"
    [env.test]
    package = "editable"  # develop install
    "#
    );
}

/// An environment name the file quoted because it holds a dot is one segment, and folds like any
/// other name.
#[test]
fn test_an_environment_name_holding_a_dot_folds() {
    assert_snapshot!(format_toml_helper("[env.\"a.b\".deps]\npytest = [\"pytest\"]\n", 2), @r#"
    [env."a.b"]
    deps.pytest = [ "pytest" ]
    "#);

    let settings = Settings {
        table_format: String::from("long"),
        ..default_settings()
    };
    assert_snapshot!(format_toml("[env.\"a.b\"]\ndeps.pytest = [\"pytest\"]\n", &settings).expect("the formatter reads its own output"), @r#"
    [env."a.b".deps]
    pytest = [ "pytest" ]
    "#);
}

/// A setting names a table the way TOML does, so `env."a.b"` and `env.a.b` name different tables
/// and select them apart.
#[test]
fn test_a_setting_tells_a_quoted_name_from_a_path() {
    let source = concat!(
        "[env.\"a.b\".deps]\npytest = [\"pytest\"]\n",
        "[env.a.b.deps]\ncoverage = [\"coverage\"]\n",
    );

    let quoted = Settings {
        expand_tables: vec![String::from("env.\"a.b\"")],
        ..default_settings()
    };
    assert_snapshot!(format_toml(source, &quoted).expect("the formatter reads its own output"), @r#"
    [env.a]
    b.deps.coverage = [ "coverage" ]

    [env."a.b".deps]
    pytest = [ "pytest" ]
    "#);

    let plain = Settings {
        expand_tables: vec![String::from("env.a.b")],
        ..default_settings()
    };
    assert_snapshot!(format_toml(source, &plain).expect("the formatter reads its own output"), @r#"
    [env."a.b"]
    deps.pytest = [ "pytest" ]

    [env.a.b.deps]
    coverage = [ "coverage" ]
    "#);
}

/// The comments around the older key are about the environment, so they survive its removal.
#[test]
fn test_use_develop_hands_every_comment_to_the_key_that_stays() {
    assert_snapshot!(
        format_toml_helper(
            concat!(
                "[env.test]\n",
                "# why the legacy setting is present\n",
                "use_develop = true # legacy spelling\n",
                "package = \"wheel\"  # the mode tox never reached\n",
            ),
            2
        ),
        @r#"
    [env.test]
    # why the legacy setting is present
    # legacy spelling
    package = "editable"  # the mode tox never reached
    "#
    );
}

/// A setting names a table of any depth, and the table it names may be written either way, so the
/// one it asks for is the one that comes back.
#[test]
fn test_expand_tables_reaches_a_child_written_as_dotted_keys() {
    let start = indoc! {r#"
        [env.test]
        set_env.A = "1"
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("env.test.set_env")],
        ..default_settings()
    };

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [env.test.set_env]
    A = "1"
    "#);
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

#[test]
fn test_collapse_tables_reaches_a_child_written_as_a_header() {
    let start = indoc! {r#"
        [env.test]

        [env.test.set_env]
        A = "1"
        "#};
    let settings = Settings {
        table_format: String::from("long"),
        collapse_tables: vec![String::from("env.test.set_env")],
        ..default_settings()
    };

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [env.test]
    set_env.A = "1"
    "#);
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

/// An environment the file quoted because its name holds a dot is one environment, so it sorts
/// among the others and stays ahead of the `[env]` table itself.
#[test]
fn test_quoted_environments_join_the_environment_order() {
    let start = indoc! {r#"
        [env]
        description = "the table itself"

        [env."b.c"]
        description = "b"

        [env."a.b"]
        description = "a"
        "#};
    let settings = Settings {
        table_format: String::from("long"),
        ..default_settings()
    };

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [env."a.b"]
    description = "a"

    [env."b.c"]
    description = "b"

    [env]
    description = "the table itself"
    "#);
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

/// The tables under a quoted environment read in the environment key order, the same as those under
/// any other.
#[test]
fn test_a_quoted_environment_orders_the_tables_under_it() {
    let start = indoc! {r#"
        [env."a.b"]
        description = "run tests"
        custom.a = 1
        set_env.A = "1"
        "#};
    let settings = Settings {
        table_format: String::from("long"),
        ..default_settings()
    };

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [env."a.b"]
    description = "run tests"
    [env."a.b".set_env]
    A = "1"
    [env."a.b".custom]
    a = 1
    "#);
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

/// An environment sorts by the name it was given, not by the quotes a header needs to write that
/// name down.
#[test]
fn test_a_quoted_environment_sorts_by_the_name_it_was_given() {
    let start = indoc! {r#"
        [env."z.z"]
        description = "z"

        [env.alpha]
        description = "a"
        "#};
    let settings = Settings {
        table_format: String::from("long"),
        ..default_settings()
    };

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [env.alpha]
    description = "a"

    [env."z.z"]
    description = "z"
    "#);
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

/// The table a setting names gets a header of its own even where the table above it keeps its keys
/// folded in.
#[test]
fn test_expand_tables_reaches_a_table_below_a_folded_one() {
    let start = indoc! {r#"
        [env.test]
        set_env.extra.A = "1"
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("env.test.set_env.extra")],
        ..default_settings()
    };

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [env.test.set_env.extra]
    A = "1"
    "#);
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

#[test]
fn test_collapse_tables_reaches_a_table_below_a_written_out_one() {
    let start = indoc! {r#"
        [env.test]

        [env.test.set_env.extra]
        A = "1"
        "#};
    let settings = Settings {
        table_format: String::from("long"),
        collapse_tables: vec![String::from("env.test.set_env.extra")],
        ..default_settings()
    };

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [env.test]
    [env.test.set_env]
    extra.A = "1"
    "#);
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

/// An environment written as dotted keys of the root table reads the way its own table does: the
/// keys of each environment stay together, in the environment key order.
#[test]
fn test_an_environment_written_as_root_keys_keeps_the_environment_order() {
    let start = indoc! {r#"
        env_run_base.commands = [["pytest"]]
        env_run_base.deps = ["a", "b"]
        env_run_base.runner = "uv-venv-lock-runner"
        env."a.b".description = "quoted"
        env."a.b".runner = "uv-venv-lock-runner"
        min_version = "4.0"
        "#};
    let settings = Settings {
        collapse_tables: vec![String::from("env_run_base"), String::from("env")],
        ..default_settings()
    };

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    min_version = "4.0"
    env."a.b".runner = "uv-venv-lock-runner"
    env."a.b".description = "quoted"
    env_run_base.runner = "uv-venv-lock-runner"
    env_run_base.deps = [ "a", "b" ]
    env_run_base.commands = [ [ "pytest" ] ]
    "#);
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

/// A table the file wrote with nothing under it says that the environment is there, so it stays,
/// and so does whatever was written above it.
#[test]
fn test_an_environment_written_with_nothing_under_it_stays() {
    let start = indoc! {r#"
        # why it is here
        [env.test]
        "#};
    let settings = default_settings();

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @"
    # why it is here
    [env.test]
    ");
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

#[test]
fn test_an_environment_written_with_nothing_under_it_stays_when_written_out() {
    let start = indoc! {r#"
        [env.test]

        [env.other]
        a = 1
        "#};
    let settings = Settings {
        table_format: String::from("long"),
        ..default_settings()
    };

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @"
    [env.other]
    a = 1

    [env.test]
    ");
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

/// tox reads a `set_env` table in the order it is written, so a key after `file` overrides what
/// that file said.
#[test]
fn test_set_env_keys_keep_their_order() {
    let start = indoc! {r#"
        [env.test]
        set_env.ZULU = "1"
        set_env.file = "vars.env"
        set_env.ALPHA = "2"
        "#};
    let settings = default_settings();

    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [env.test]
    set_env.ZULU = "1"
    set_env.file = "vars.env"
    set_env.ALPHA = "2"
    "#);
    assert_eq!(
        format_toml(&got, &settings).expect("the formatter reads its own output"),
        got
    );
}

/// pip reads this list the way it reads a requirements file, where a later `--index-url` replaces
/// the one before it.
#[test]
fn test_deps_holding_a_pip_option_keep_their_order() {
    let start = indoc! {r#"
        [env.test]
        deps = ["--index-url=https://one.example", "zebra>=1", "--index-url=https://two.example", "alpha>=1"]
        "#};

    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.test]
    deps = [
      "--index-url=https://one.example",
      "zebra>=1",
      "--index-url=https://two.example",
      "alpha>=1",
    ]
    "#);
}

/// pip installs a file it is handed by name, so an artifact written without a path is left as the
/// file spelled it and the list it sits in keeps its order.
#[test]
fn test_deps_naming_an_artifact_keep_it_as_written() {
    let start = indoc! {r#"
        [env.test]
        deps = ["package.whl", "pkg-1.0.tar.gz", "Zebra >= 1"]
        "#};

    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.test]
    deps = [ "package.whl", "pkg-1.0.tar.gz", "zebra>=1" ]
    "#);
}

/// A list naming nothing but requirements is a set of them, so it sorts.
#[test]
fn test_deps_of_plain_requirements_sort() {
    let start = indoc! {r#"
        [env.test]
        deps = ["Zebra >= 1.0.0", "alpha"]
        "#};

    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.test]
    deps = [ "alpha", "zebra>=1" ]
    "#);
}

/// Each constraint names a file tox hands to pip, not a requirement, so it is left as written.
#[test]
fn test_constraints_are_file_names_rather_than_requirements() {
    let start = indoc! {r#"
        [env.test]
        constraints = ["constraints.txt", "other_constraints.txt"]
        "#};

    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.test]
    constraints = [ "constraints.txt", "other_constraints.txt" ]
    "#);
}

/// A table whose every key is commented out is one the file wrote empty, and folding those keys
/// into the parent would leave nothing saying the table is there.
#[test]
fn test_a_table_of_only_commented_keys_stays_written_out() {
    let start = indoc! {r#"
        [env.test]
        deps = ["pytest"]

        [env.test.set_env]
        # PYTHONWARNINGS = "error"
        "#};
    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.test]
    deps = [ "pytest" ]

    [env.test.set_env]
    # PYTHONWARNINGS = "error"
    "#);
}

/// A key the file wrote is what says the table is there, so the commented one beside it folds in.
#[test]
fn test_a_table_with_a_key_beside_the_commented_one_folds() {
    let start = indoc! {r#"
        [env.test]
        deps = ["pytest"]

        [env.test.set_env]
        # PYTHONWARNINGS = "error"
        PYTHONHASHSEED = "0"
        "#};
    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.test]
    deps = [ "pytest" ]
    # set_env.PYTHONWARNINGS = "error"
    set_env.PYTHONHASHSEED = "0"
    "#);
}

/// A key the file wrote as a comment is not configuration tox reads, so neither the spelling it
/// uses nor the order of what it lists is rewritten.
#[test]
fn test_a_commented_key_is_left_as_the_file_wrote_it() {
    let start = indoc! {r#"
        [env.test]
        # setenv = { A = "1" }
        deps = ["b", "a"]
        # deps = ["z", "y"]
        "#};

    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.test]
    deps = [ "a", "b" ]
    # deps = [ "z", "y" ]
    # setenv = { A = "1" }
    "#);
}

/// Writing a table out empties the header its dotted keys were written under, and a comment the
/// file wrote there says what it says wherever the keys end up.
#[test]
fn test_a_header_comment_survives_writing_its_keys_out() {
    let start = indoc! {r#"
        # lead
        [env.test] # beside
        set_env.A = "1"
        "#};
    let settings = Settings {
        table_format: String::from("long"),
        ..default_settings()
    };
    let got = format_toml(start, &settings).expect("the formatter reads its own output");
    assert_valid_toml(&got);

    assert_snapshot!(got, @r#"
    # lead
    [env.test]  # beside
    [env.test.set_env]
    A = "1"
    "#);
    assert_eq!(
        format_toml(got.as_str(), &settings).expect("the formatter reads its own output"),
        got
    );

    let led =
        format_toml("# lead\n[env.test]\nset_env.A = \"1\"\n", &settings).expect("the formatter reads its own output");
    assert_valid_toml(&led);
    assert_snapshot!(led, @r#"
    # lead
    [env.test]
    [env.test.set_env]
    A = "1"
    "#);
}

/// A file no reader accepts is one the formatter has nothing to say about, so it says that rather
/// than handing back what it was given.
#[test]
fn test_a_file_that_is_not_a_document_is_rejected() {
    assert!(format_toml("key =\n", &default_settings()).is_err());
}

/// A `{ replace = "ref" }` names a key rather than holding text that looks like one, so a key that
/// moved to the spelling tox reads today takes the references to it along.
#[test]
fn test_a_reference_follows_the_key_it_names() {
    let start = indoc! {r#"
        envlist = ["docs"]

        [env_run_base]
        setenv = { A = "1" }

        [env.docs]
        setenv = { replace = "ref", of = ["env_run_base", "setenv"] }
        "#};

    assert_snapshot!(format_toml_helper(start, 2), @r#"
    env_list = [ "docs" ]

    [env_run_base]
    set_env = { A = "1" }

    [env.docs]
    set_env = { replace = "ref", of = [ "env_run_base", "set_env" ] }
    "#);
}

/// A reference naming no path of its own has no key for a rename to follow.
#[test]
fn test_a_reference_that_names_no_path() {
    let start = indoc! {r#"
        [env.docs]
        setenv = { A = "1" }
        a = { replace = "ref", of = "setenv" }
        b = { replace = "ref", of = [] }
        c = { replace = "ref", of = ["env_run_base", 1] }
        "#};

    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.docs]
    set_env = { A = "1" }
    a = { replace = "ref", of = "setenv" }
    b = { replace = "ref", of = [] }
    c = { replace = "ref", of = [ "env_run_base", 1 ] }
    "#);
}

/// An ordinary array that happens to hold the same word names no key, so nothing in it moves.
#[test]
fn test_an_array_that_is_not_a_reference_is_left_alone() {
    let start = indoc! {r#"
        [env.docs]
        setenv = { A = "1" }
        commands = [["echo", "setenv"]]
        "#};

    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.docs]
    set_env = { A = "1" }
    commands = [ [ "echo", "setenv" ] ]
    "#);
}

/// A reference names one table, not any whose path happens to end the same way: an environment
/// called `src` is not the root table of that name, and tox resolves the two separately.
#[test]
fn test_a_reference_names_the_table_it_spells_and_no_other() {
    let start = indoc! {r#"
        [src]
        setenv = { A = "root" }

        [env.src]
        setenv = { A = "env" }

        [env.docs]
        setenv = { replace = "ref", of = ["src", "setenv"] }
        other = { replace = "ref", of = ["env", "src", "setenv"] }
        "#};

    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env.docs]
    set_env = { replace = "ref", of = [ "src", "setenv" ] }
    other = { replace = "ref", of = [ "env", "src", "set_env" ] }

    [env.src]
    set_env = { A = "env" }

    [src]
    setenv = { A = "root" }
    "#);
}

/// An environment is the same environment however the file splits its path, so its requirements are
/// normalized and ordered either way.
#[test]
fn test_an_environment_is_formatted_however_the_file_splits_its_path() {
    assert_snapshot!(
        format_toml_helper("env.test.deps = [ \"z>=2.0.0\", \"A>=1.0.0\" ]\n", 2),
        @r#"env.test.deps = [ "a>=1", "z>=2" ]"#
    );
    assert_snapshot!(
        format_toml_helper("[env]\ntest.deps = [ \"z>=2.0.0\", \"A>=1.0.0\" ]\n", 2),
        @r#"
    [env.test]
    deps = [ "a>=1", "z>=2" ]
    "#
    );
}

/// A reusable base is an environment under another name, so it folds its own sub-tables in and its
/// keys are read the same way.
#[test]
fn test_a_reusable_base_is_read_as_the_environment_it_is() {
    let start = indoc! {r#"
        [env_base.shared]
        deps = ["Z>=2.0.0", "a>=1.0.0"]

        [env_base.shared.set_env]
        A = "1"
        "#};

    assert_snapshot!(format_toml_helper(start, 2), @r#"
    [env_base.shared]
    deps = [ "a>=1", "z>=2" ]
    set_env.A = "1"
    "#);
}

/// The root table is the same table however the file splits its path, so an older alias is moved
/// and a requirement list sorts either way.
#[test]
fn test_the_root_table_is_read_however_the_file_writes_it() {
    assert_snapshot!(
        format_toml_helper("minversion = \"4.0\"\nrequires = [ \"z\", \"a\" ]\n", 2),
        @r#"
    min_version = "4.0"
    requires = [ "a", "z" ]
    "#
    );
    // a table the file wrote as a value says the same thing
    assert_snapshot!(
        format_toml_helper("env = { test = { setenv = { A = \"1\" }, deps = [ \"z\", \"a\" ] } }\n", 2),
        @r#"env = { test = { deps = [ "a", "z" ], set_env = { A = "1" } } }"#
    );
}

fn format_toml_helper(start: &str, indent: usize) -> String {
    evaluate_settings(
        start,
        &Settings {
            indent,
            ..default_settings()
        },
    )
}

fn new_settings(settings: Settings) -> PyResult<Settings> {
    Python::attach(|python| Settings::new(Some(&settings_kwargs(python, settings)?)))
}

fn settings_kwargs(python: Python<'_>, settings: Settings) -> PyResult<Bound<'_, PyDict>> {
    let kwargs = PyDict::new(python);
    kwargs.set_item("column_width", settings.column_width)?;
    kwargs.set_item("indent", settings.indent)?;
    kwargs.set_item("table_format", settings.table_format)?;
    kwargs.set_item("sub_table_spacing", settings.sub_table_spacing)?;
    kwargs.set_item("separate_root_table", settings.separate_root_table)?;
    kwargs.set_item("expand_tables", settings.expand_tables)?;
    kwargs.set_item("collapse_tables", settings.collapse_tables)?;
    kwargs.set_item("skip_wrap_for_keys", settings.skip_wrap_for_keys)?;
    kwargs.set_item("pin_envs", settings.pin_envs)?;
    Ok(kwargs)
}
