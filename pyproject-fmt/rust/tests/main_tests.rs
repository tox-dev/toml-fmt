use std::collections::HashSet;

use indoc::indoc;
use insta::assert_snapshot;
use pyo3::types::{PyDict, PyDictMethods};
use pyo3::{PyResult, Python};

use super::{assert_valid_toml, default_settings};
use _pyproject_fmt::{format_toml, Settings};

#[test]
fn test_format_toml_simple() {
    let start = indoc! {r#"
    # comment
    a= "b"
    [project]
    name="alpha"
    dependencies=[" e >= 1.5.0"]
    [build-system]
    build-backend="backend"
    requires=[" c >= 1.5.0", "d == 2.0.0"]
    [dependency-groups]
    test=["p>1.0.0"]
    [tool.mypy]
    mk="mv"
    "#};
    let res = format_toml_helper(start, 2, false, (3, 13), true);
    assert_snapshot!(res, @r#"
    # comment
    a = "b"

    [build-system]
    build-backend = "backend"
    requires = [ "c>=1.5", "d==2" ]

    [project]
    name = "alpha"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
      "Programming Language :: Python :: 3.13",
    ]
    dependencies = [ "e>=1.5" ]

    [dependency-groups]
    test = [ "p>1" ]

    [tool.mypy]
    mk = "mv"
    "#);
}

#[test]
fn test_format_toml_scripts() {
    let start = indoc! {r#"
    [project.scripts]
    c = "d"
    a = "b"
    "#};
    let res = format_toml_helper(start, 2, true, (3, 9), true);
    assert_snapshot!(res, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    scripts.a = "b"
    scripts.c = "d"
    "#);
}

#[test]
fn test_expand_tables_with_project() {
    let start = indoc! {r#"
        [project]
        name = "example"
        optional-dependencies.dev = ["pytest"]
        urls.homepage = "https://example.com"
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"

    [project.optional-dependencies]
    dev = [ "pytest" ]

    [project.urls]
    homepage = "https://example.com"
    "#);
}

#[test]
fn test_collapse_project_authors() {
    let start = indoc! {r#"
        [project]
        name = "example"
        [[project.authors]]
        name = "John Doe"
        email = "john@example.com"
        "#};
    let settings = Settings {
        collapse_tables: vec![String::from("project.authors")],
        ..long_format_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    authors = [ { name = "John Doe", email = "john@example.com" } ]
    "#);
}

#[test]
fn test_collapse_project_maintainers() {
    let start = indoc! {r#"
        [project]
        name = "example"
        [[project.maintainers]]
        name = "Jane Doe"
        email = "jane@example.com"
        "#};
    let settings = Settings {
        collapse_tables: vec![String::from("project.maintainers")],
        ..long_format_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    maintainers = [ { name = "Jane Doe", email = "jane@example.com" } ]
    "#);
}

#[test]
fn test_table_format_long_with_entry_points() {
    let start = indoc! {r#"
        [project]
        name = "example"
        entry-points."console_scripts".mycli = "pkg:main"
        entry-points."console_scripts".othercli = "pkg:other"
        "#};
    let got = format_toml(start, &long_format_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    [project.entry-points]
    console_scripts.mycli = "pkg:main"
    console_scripts.othercli = "pkg:other"
    "#);
}

#[test]
fn test_expand_project_authors() {
    let start = indoc! {r#"
        [project]
        name = "example"
        authors = [
          { name = "John Doe", email = "john@example.com" },
          { name = "Jane Doe", email = "jane@example.com" },
        ]
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"

    [[project.authors]]
    name = "John Doe"
    email = "john@example.com"

    [[project.authors]]
    name = "Jane Doe"
    email = "jane@example.com"
    "#);
}

#[test]
fn test_expand_project_maintainers() {
    let start = indoc! {r#"
        [project]
        name = "example"
        maintainers = [
          { name = "Bob Smith", email = "bob@example.com" },
          { name = "Alice Jones", email = "alice@example.com" },
        ]
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.maintainers")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"

    [[project.maintainers]]
    name = "Bob Smith"
    email = "bob@example.com"

    [[project.maintainers]]
    name = "Alice Jones"
    email = "alice@example.com"
    "#);
}

#[test]
fn test_expand_single_author() {
    let start = indoc! {r#"
        [project]
        name = "example"
        authors = [
          { name = "John Doe", email = "john@example.com" },
        ]
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"

    [[project.authors]]
    name = "John Doe"
    email = "john@example.com"
    "#);
}
#[test]
fn test_collapse_authors_with_url_field() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [[project.authors]]
        name = "Bob"
        email = "bob@example.com"
        url = "https://bob.com"
        [[project.authors]]
        name = "Alice"
        email = "alice@example.com"
        "#};
    let got = format_toml(start, &default_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "test"
    authors = [
      { name = "Bob", email = "bob@example.com", url = "https://bob.com" },
      { name = "Alice", email = "alice@example.com" }
    ]
    "#);
}
#[test]
fn test_collapse_empty_authors() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [[project.authors]]
        [[project.authors]]
        "#};
    let got = format_toml(start, &default_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "test"

    [[project.authors]]

    [[project.authors]]
    "#);
}

#[test]
fn test_collapse_authors_without_trailing_newline() {
    let start = "[project]\nname = \"test\"\n[[project.authors]]\nname = \"Alice\"\nemail = \"alice@example.com\"";
    let got = format_toml(start, &default_settings()).unwrap();
    assert!(got.contains("authors = ["));
    assert!(got.contains("{ name = \"Alice\", email = \"alice@example.com\" }"));
}

#[test]
fn test_collapse_authors_compact_parent() {
    let start =
        "[project]\nname=\"test\"\nversion=\"1.0\"\n[[project.authors]]\nname=\"Alice\"\nemail=\"alice@example.com\"";
    let got = format_toml(start, &default_settings()).unwrap();
    assert!(got.contains("authors = ["));
}

#[test]
fn test_expand_authors_already_expanded() {
    let start = indoc! {r#"
        [project]
        name = "example"
        [[project.authors]]
        name = "John Doe"
        email = "john@example.com"
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..long_format_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert!(got.contains("[[project.authors]]"));
    assert!(got.contains("name = \"John Doe\""));
}

#[test]
fn test_issue_146_expand_specific_subtable() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [project.optional-dependencies]
        a = ["b", "c"]
        [project.urls]
        homepage = "https://example.com"
        "#};
    let settings = Settings {
        column_width: 120,
        indent: 4,
        keep_full_version: true,
        max_supported_python: (3, 14),
        min_supported_python: (3, 14),
        expand_tables: vec![String::from("project.optional-dependencies")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert!(
        got.contains("[project.optional-dependencies]"),
        "optional-dependencies should stay expanded"
    );
    assert!(got.contains("urls.homepage ="), "urls should be collapsed");
}

#[test]
fn test_css_specificity_more_specific_wins() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [project.urls]
        homepage = "https://example.com"
        [project.optional-dependencies]
        dev = ["pytest"]
        "#};
    let settings = Settings {
        indent: 4,
        keep_full_version: true,
        expand_tables: vec![String::from("project.urls")],
        collapse_tables: vec![String::from("project")],
        ..long_format_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert!(
        got.contains("[project.urls]"),
        "project.urls should be expanded (specific)"
    );
    assert!(
        got.contains("optional-dependencies.dev ="),
        "optional-dependencies should be collapsed (inherits project)"
    );
}

#[test]
fn test_nested_table_specificity() {
    use _pyproject_fmt::TableFormatConfig;

    let expand: HashSet<Vec<String>> = HashSet::from([common::sections::parse_name("project.entry-points.special")]);

    let collapse: HashSet<Vec<String>> = HashSet::from([common::sections::parse_name("project.entry-points")]);

    let config = TableFormatConfig {
        default_collapse: false,
        expand,
        collapse,
    };

    assert!(
        config.should_collapse(&["project", "entry-points"].map(str::to_owned)),
        "project.entry-points should collapse"
    );
    assert!(
        config.should_collapse(&["project", "entry-points", "tox"].map(str::to_owned)),
        "project.entry-points.tox inherits collapse"
    );
    assert!(
        !config.should_collapse(&["project", "entry-points", "special"].map(str::to_owned)),
        "project.entry-points.special should expand"
    );
}

#[test]
fn test_parent_inheritance() {
    use _pyproject_fmt::TableFormatConfig;

    let expand: HashSet<Vec<String>> = HashSet::from([common::sections::parse_name("project")]);

    let config = TableFormatConfig {
        default_collapse: true,
        expand,
        collapse: HashSet::new(),
    };

    assert!(
        !config.should_collapse(&["project"].map(str::to_owned)),
        "project should expand"
    );
    assert!(
        !config.should_collapse(&["project", "urls"].map(str::to_owned)),
        "project.urls inherits expand from project"
    );
    assert!(
        !config.should_collapse(&["project", "optional-dependencies"].map(str::to_owned)),
        "project.optional-dependencies inherits expand"
    );
}

#[test]
fn test_default_collapse_fallback() {
    use _pyproject_fmt::TableFormatConfig;

    let config = TableFormatConfig {
        default_collapse: true,
        expand: HashSet::new(),
        collapse: HashSet::new(),
    };

    assert!(config.should_collapse(&["project"].map(str::to_owned)));
    assert!(config.should_collapse(&["project", "urls"].map(str::to_owned)));
    assert!(config.should_collapse(&["tool", "ruff", "lint"].map(str::to_owned)));
}

#[test]
fn test_issue_146_deeply_nested_ruff_table() {
    let start = indoc! {r#"
        [tool.ruff.lint.flake8-tidy-imports.banned-api]
        "collections.namedtuple".msg = "Use typing.NamedTuple instead"
        "#};
    let settings = Settings {
        column_width: 120,
        indent: 4,
        keep_full_version: true,
        max_supported_python: (3, 14),
        min_supported_python: (3, 14),
        expand_tables: vec![String::from("tool.ruff.lint.flake8-tidy-imports.banned-api")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert!(
        got.contains("[tool.ruff.lint.flake8-tidy-imports.banned-api]"),
        "deeply nested ruff table should stay expanded. Got:\n{got}"
    );
}

#[test]
fn test_no_duplicate_requires() {
    let start = indoc! {r#"
        [build-system]
        build-backend = "backend"
        requires = ["c", "d"]
    "#};
    let got = format_toml(start, &default_settings()).unwrap();
    let count = got.matches("requires").count();
    assert_eq!(count, 1, "requires should appear exactly once, but got:\n{}", got);
}

#[test]
fn test_table_format_long_removes_blank_lines_between_same_group() {
    let start = indoc! {r#"
        [project]
        name = "test"

        [project.urls]
        homepage = "https://example.com"

        [project.optional-dependencies]
        dev = ["pytest"]
        "#};
    let got = format_toml(start, &long_format_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "test"
    [project.optional-dependencies]
    dev = [ "pytest" ]
    [project.urls]
    homepage = "https://example.com"
    "#);
}

#[test]
fn test_table_format_long_with_tool_tables() {
    let start = indoc! {r#"
        [tool.ruff]
        line-length = 120

        [tool.ruff.lint]
        select = ["E", "W"]

        [tool.mypy]
        strict = true
        "#};
    let got = format_toml(start, &long_format_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [tool.ruff]
    line-length = 120
    [tool.ruff.lint]
    select = [ "E", "W" ]

    [tool.mypy]
    strict = true
    "#);
}

#[test]
fn test_table_format_long_preserves_blank_lines_between_different_groups() {
    let start = indoc! {r#"
        [build-system]
        requires = ["setuptools"]

        [project]
        name = "test"
        "#};
    let got = format_toml(start, &long_format_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [build-system]
    requires = [ "setuptools" ]

    [project]
    name = "test"
    "#);
}

#[test]
fn test_extract_table_names_from_array_tables() {
    let start = indoc! {r#"
        [project]
        name = "test"

        [[project.authors]]
        name = "John"
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..long_format_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "test"
    [[project.authors]]
    name = "John"
    "#);
}

#[test]
fn test_format_with_trailing_newline_preserved() {
    let start = "[project]\nname = \"test\"\n";
    let got = format_toml(start, &default_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "test"
    "#);
}

#[test]
fn test_tool_prefix_extraction_with_dotted_keys() {
    let start = indoc! {r#"
        [tool.coverage.run]
        branch = true

        [tool.coverage.report]
        precision = 2
        "#};
    let got = format_toml(start, &long_format_settings()).unwrap();
    assert_snapshot!(got, @"
    [tool.coverage.run]
    branch = true
    [tool.coverage.report]
    precision = 2
    ");
}

#[test]
fn test_should_collapse_with_no_dot_in_name() {
    use _pyproject_fmt::TableFormatConfig;

    let config = TableFormatConfig {
        default_collapse: true,
        expand: HashSet::new(),
        collapse: HashSet::new(),
    };

    assert!(config.should_collapse(&["project"].map(str::to_owned)));
    assert!(config.should_collapse(&["build-system"].map(str::to_owned)));
}

#[test]
fn test_format_with_non_table_lines_between_headers() {
    let start = indoc! {r#"
        [project]
        name = "test"
        version = "1.0"

        [project.urls]
        homepage = "https://example.com"
        "#};
    let got = format_toml(start, &long_format_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "test"
    version = "1.0"
    [project.urls]
    homepage = "https://example.com"
    "#);
}

#[test]
fn test_settings_new() {
    let settings = new_settings(Settings {
        column_width: 120,
        indent: 4,
        keep_full_version: true,
        max_supported_python: (3, 13),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: true,
        table_format: String::from("short"),
        sub_table_spacing: String::from("\n"),
        separate_root_table: String::from("\n\n"),
        expand_tables: vec![String::from("project.urls")],
        collapse_tables: vec![String::from("project.authors")],
        skip_wrap_for_keys: vec![],
    })
    .expect("Python 3 bounds");
    assert_eq!(settings.column_width, 120);
    assert_eq!(settings.indent, 4);
    assert!(settings.keep_full_version);
    assert_eq!(settings.max_supported_python, (3, 13));
    assert_eq!(settings.min_supported_python, (3, 9));
    assert!(settings.generate_python_version_classifiers);
    assert_eq!(settings.table_format, "short");
    assert_eq!(settings.sub_table_spacing, "\n");
    assert_eq!(settings.separate_root_table, "\n\n");
    assert_eq!(settings.expand_tables, vec!["project.urls"]);
    assert_eq!(settings.collapse_tables, vec!["project.authors"]);
}

#[test]
fn test_table_format_config_from_settings() {
    use _pyproject_fmt::TableFormatConfig;

    let settings = new_settings(Settings {
        max_supported_python: (3, 12),
        expand_tables: vec![String::from("tool.ruff")],
        collapse_tables: vec![String::from("project")],
        ..default_settings()
    })
    .expect("Python 3 bounds");
    let config = TableFormatConfig::new(
        &settings.table_format,
        &settings.expand_tables,
        &settings.collapse_tables,
    );
    assert!(config.default_collapse);
    assert!(config.expand.contains(&common::sections::parse_name("tool.ruff")));
    assert!(config.collapse.contains(&common::sections::parse_name("project")));
}

#[test]
fn test_lib_module_registration() {
    use pyo3::types::PyAnyMethods;

    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let module = pyo3::types::PyModule::new(py, "_lib").unwrap();
        _pyproject_fmt::_lib(&module.as_borrowed()).unwrap();

        assert!(module.hasattr("format_toml").unwrap());
        assert!(module.hasattr("Settings").unwrap());
    });
}

#[test]
fn test_idempotent_formatting() {
    let start = indoc! {r#"
        [project]
        name = "test"
        description = "This is a long description string that needs to exceed the default column width of one hundred and twenty characters to trigger wrapping."
    "#};
    let settings = default_settings();
    let first = format_toml(start, &settings).unwrap();
    let second = format_toml(&first, &settings).unwrap();
    let third = format_toml(&second, &settings).unwrap();
    assert_eq!(first, second, "formatting should be idempotent (first->second)");
    assert_eq!(second, third, "formatting should be idempotent (second->third)");
}

#[test]
fn test_issue_186_single_quote_with_comments() {
    let start = indoc! {r#"
    [tool.something]
    items = [
        'first',
        # A comment
        'second',
    ]
    "#};
    let got = format_toml(start, &default_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [tool.something]
    items = [
      "first",
      # A comment
      "second",
    ]
    "#);
}

#[test]
fn test_remove_blank_lines_between_same_group_tables_long_format() {
    let start = indoc! {r#"
    [tool.ruff]
    line-length = 100

    [tool.ruff.lint]
    select = ["ALL"]

    [tool.ruff.format]
    quote-style = "double"
    "#};
    let got = format_toml(start, &long_format_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [tool.ruff]
    line-length = 100
    [tool.ruff.format]
    quote-style = "double"
    [tool.ruff.lint]
    select = [ "ALL" ]
    "#);
}

#[test]
fn test_table_key_without_prefix_match_long_format() {
    let start = indoc! {r#"
    [custom]
    key = "value"

    [custom.nested]
    other = "data"
    "#};
    let got = format_toml(start, &long_format_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [custom]
    key = "value"
    [custom.nested]
    other = "data"
    "#);
}

#[test]
fn test_sub_table_spacing_blank_line() {
    let start = indoc! {r#"
        [tool.ruff]
        line-length = 120

        [tool.ruff.lint]
        select = ["E", "W"]

        [tool.mypy]
        strict = true
        "#};
    let settings = Settings {
        sub_table_spacing: String::from("\n"),
        ..long_format_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [tool.ruff]
    line-length = 120

    [tool.ruff.lint]
    select = [ "E", "W" ]

    [tool.mypy]
    strict = true
    "#);
}

#[test]
fn test_sub_table_spacing_with_project_tables() {
    let start = indoc! {r#"
        [project]
        name = "test"

        [project.urls]
        homepage = "https://example.com"

        [project.optional-dependencies]
        dev = ["pytest"]
        "#};
    let settings = Settings {
        sub_table_spacing: String::from("\n"),
        ..long_format_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "test"

    [project.optional-dependencies]
    dev = [ "pytest" ]

    [project.urls]
    homepage = "https://example.com"
    "#);
}

#[test]
fn test_issue_402_sub_table_spacing_two_blank_lines() {
    let start = indoc! {r#"
        [tool.uv.sources]
        pkg = { workspace = true }

        [tool.uv.workspace]
        members = ["a", "b"]

        [tool.pyproject-fmt]
        indent = 4
        "#};
    let settings = Settings {
        sub_table_spacing: String::from("\n\n"),
        ..long_format_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [tool.uv.sources]
    pkg = { workspace = true }


    [tool.uv.workspace]
    members = [ "a", "b" ]

    [tool.pyproject-fmt]
    indent = 4
    "#);
}

#[test]
fn test_issue_402_separate_root_table_two_blank_lines() {
    let start = indoc! {r#"
        [build-system]
        requires = ["hatchling"]

        [project]
        name = "test"
        "#};
    let settings = Settings {
        separate_root_table: String::from("\n\n"),
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [build-system]
    requires = [ "hatchling" ]


    [project]
    name = "test"
    "#);
}

#[test]
fn test_issue_217_mixed_quotes_idempotent() {
    let start = indoc! {r#"
    [project]
    name = "flexget"
    requires-python = ">=3.14"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.14",
    ]

    [tool.ruff]
    lint.per-file-ignores.'docs/scripts/*' = [ "T20" ]
    lint.per-file-ignores.'tests/*' = [ "T20" ]
    lint.per-file-ignores."flexget/*" = [ "PTH" ] # TODO
    "#};
    let settings = Settings {
        keep_full_version: true,
        max_supported_python: (3, 14),
        min_supported_python: (3, 14),
        ..default_settings()
    };
    let first = format_toml(start, &settings).unwrap();
    assert_valid_toml(&first);
    let second = format_toml(&first, &settings).unwrap();
    assert_eq!(first, second, "formatting should be idempotent");
    assert_snapshot!(first, @r#"
    [project]
    name = "flexget"
    requires-python = ">=3.14"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.14",
    ]

    [tool.ruff]
    lint.per-file-ignores."docs/scripts/*" = [ "T20" ]
    lint.per-file-ignores."flexget/*" = [ "PTH" ]  # TODO
    lint.per-file-ignores."tests/*" = [ "T20" ]
    "#);
}

#[test]
fn test_issue_217_full_pyproject_idempotent() {
    let start = std::fs::read_to_string(super::data_dir().join("issue-217.toml")).expect("the case reads its input");
    let settings = Settings {
        keep_full_version: true,
        max_supported_python: (3, 14),
        min_supported_python: (3, 10),
        ..default_settings()
    };
    let first = format_toml(&start, &settings).unwrap();
    assert_valid_toml(&first);
    let second = format_toml(&first, &settings).unwrap();
    assert_eq!(first, second, "formatting should be idempotent");
    assert_snapshot!(first);
}

#[test]
fn test_issue_299_pixi_workspace_collapse_with_keys() {
    let start = indoc! {r#"
    [project]
    name = "x"
    version = "0.1.0"

    [tool.pixi.workspace]
    name = "my-project"
    "#};
    let got = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "x"
    version = "0.1.0"

    [tool.pixi]
    workspace.name = "my-project"
    "#);
}

#[test]
fn test_issue_299_pixi_workspace_collapse_empty() {
    let start = indoc! {r#"
    [project]
    name = "x"
    version = "0.1.0"

    [tool.pixi.workspace]
    "#};
    let got = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "x"
    version = "0.1.0"

    [tool.pixi]
    workspace = {}
    "#);
}

#[test]
fn test_issue_202_preserve_inline_comment_after_array() {
    let start = indoc! {r#"
    [tool.uv]
    lint.per-file-ignores."docs/**/*.py" = [ "INP001" ] # No __init__.py in docs
    "#};
    let got = format_toml(start, &default_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [tool.uv]
    lint.per-file-ignores."docs/**/*.py" = [ "INP001" ]  # No __init__.py in docs
    "#);
}

#[test]
fn test_issue_376_collapse_with_comments_stays_valid() {
    let start = indoc! {r#"
        [[tool.uv.index]]
        name = "pypi"
        url = "https://pypi.org/simple"
        # TODO: uncomment once ready
        # default = true
        authenticate = "never"

        [[tool.uv.index]]
        name = "company-master"
        url = "https://dl.cloudsmith.io/x"
        # ignore-error-codes = [400, 401, 403]
        authenticate = "always"
    "#};
    let mut settings = default_settings();
    settings.collapse_tables = vec![String::from("tool.uv.index")];
    let result = format_toml(start, &settings).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [[tool.uv.index]]
    name = "pypi"
    url = "https://pypi.org/simple"
    # TODO: uncomment once ready
    # default = true
    authenticate = "never"

    [[tool.uv.index]]
    name = "company-master"
    url = "https://dl.cloudsmith.io/x"
    # ignore-error-codes = [ 400, 401, 403 ]
    authenticate = "always"
    "#);
}

#[test]
fn test_wide_array_of_tables_under_implicit_parent() {
    let start = indoc! {r#"
        [[tool.demo.labels.file-rules]]
        any-glob-to-any-file = ["src/managers/apt*", "src/managers/dpkg*", "src/managers/opkg*", "tests/*apt*", "tests/*dpkg*", "tests/*opkg*"]
    "#};
    let result = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [[tool.demo.labels.file-rules]]
    any-glob-to-any-file = [
      "src/managers/apt*",
      "src/managers/dpkg*",
      "src/managers/opkg*",
      "tests/*apt*",
      "tests/*dpkg*",
      "tests/*opkg*"
    ]
    "#);
}

#[test]
fn test_wide_array_of_tables_under_explicit_empty_parent() {
    let start = indoc! {r#"
        [tool.demo.labels]
        [[tool.demo.labels.file-rules]]
        any-glob-to-any-file = ["src/managers/apt*", "src/managers/dpkg*", "src/managers/opkg*", "tests/*apt*", "tests/*dpkg*", "tests/*opkg*"]
    "#};
    let result = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [tool.demo.labels]
    [[tool.demo.labels.file-rules]]
    any-glob-to-any-file = [
      "src/managers/apt*",
      "src/managers/dpkg*",
      "src/managers/opkg*",
      "tests/*apt*",
      "tests/*dpkg*",
      "tests/*opkg*"
    ]
    "#);
}

#[test]
fn test_format_toml_rejects_invalid_project_version() {
    let start = indoc! {r#"
    [project]
    name = "alpha"
    version = "1.9.xyz"
    "#};
    let error = format_toml(start, &default_settings()).unwrap_err();
    assert_snapshot!(error, @"project.version `1.9.xyz` is not a valid PEP 440 version");
}

#[test]
fn test_lib_format_toml_returns_formatted_content() {
    use pyo3::types::PyAnyMethods;

    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let module = pyo3::types::PyModule::new(py, "_lib").unwrap();
        _pyproject_fmt::_lib(&module.as_borrowed()).unwrap();
        let settings = pyo3::Py::new(py, default_settings()).unwrap();

        let got = module
            .getattr("format_toml")
            .unwrap()
            .call1(("[project]\nname=\"My_Package\"\n", settings))
            .unwrap()
            .extract::<String>()
            .unwrap();

        assert_snapshot!(got, @r#"
        [project]
        name = "my-package"
        "#);
    });
}

#[test]
fn test_lib_format_toml_raises_on_invalid_version() {
    use pyo3::types::PyAnyMethods;

    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let module = pyo3::types::PyModule::new(py, "_lib").unwrap();
        _pyproject_fmt::_lib(&module.as_borrowed()).unwrap();
        let settings = pyo3::Py::new(py, default_settings()).unwrap();

        let error = module
            .getattr("format_toml")
            .unwrap()
            .call1(("[project]\nversion=\"1.9.xyz\"\n", settings))
            .unwrap_err();

        assert!(error.is_instance_of::<pyo3::exceptions::PyValueError>(py));
        assert_snapshot!(error.value(py).to_string(), @"project.version `1.9.xyz` is not a valid PEP 440 version");
    });
}

#[test]
fn test_expand_authors_that_is_not_an_array() {
    let start = indoc! {r#"
        [project]
        name = "example"
        authors = "John Doe"
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    authors = "John Doe"
    "#);
}

#[test]
fn test_expand_authors_holding_nothing_worth_a_table() {
    let start = indoc! {r#"
        [project]
        name = "example"
        authors = [ "John Doe", {} ]
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    authors = [ "John Doe", {} ]
    "#);
}

#[test]
fn test_import_names_that_is_not_an_array() {
    let start = indoc! {r#"
        [project]
        name = "example"
        import-names = "pkg"
        "#};
    let got = format_toml(start, &default_settings()).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    import-names = "pkg"
    "#);
}

#[test]
fn test_requires_python_that_is_not_a_string() {
    let start = indoc! {r#"
        [project]
        name = "example"
        requires-python = [ ">=3.10" ]
        classifiers = [ "Programming Language :: Python :: 3.10" ]
        "#};
    let got = format_toml(start, &generating_classifiers()).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    requires-python = [ ">=3.10" ]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_requires_python_bound_beyond_a_minor_version() {
    let start = indoc! {r#"
        [project]
        name = "example"
        requires-python = ">=3.999"
        "#};
    let got = format_toml(start, &generating_classifiers()).unwrap();
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    requires-python = ">=3.999"
    "#);
}

#[test]
fn test_classifiers_that_are_not_an_array() {
    let start = indoc! {r#"
        [project]
        name = "example"
        requires-python = ">=3.10"
        classifiers = "none"
        "#};
    let got = format_toml(start, &generating_classifiers()).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    requires-python = ">=3.10"
    classifiers = "none"
    "#);
}

/// Writing out only the members that convert would drop the rest of what the file says, so an
/// array holding anything else stays as it is.
#[test]
fn test_expand_authors_holding_something_with_no_written_form() {
    let start = indoc! {r#"
        [project]
        name = "example"
        authors = [ { name = "Alice" }, {}, "Bob" ]
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    authors = [ { name = "Alice" }, {}, "Bob" ]
    "#);
}

#[test]
fn test_expand_authors_writes_an_empty_member_out_as_an_empty_element() {
    let start = indoc! {r#"
        [project]
        name = "example"
        authors = [ { name = "Alice" }, {} ]
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"

    [[project.authors]]
    name = "Alice"

    [[project.authors]]
    "#);
}

/// Comments are what the file says. A comment with no one place to go among the headers the array
/// becomes holds the array as it is.
#[test]
fn test_expand_authors_holding_comments_leaves_the_array_alone() {
    let start = indoc! {r#"
        [project]
        name = "example"
        # people published in package metadata
        authors = [
          # primary contact
          { name = "Alice" }, # owns releases
        ] # keep this provenance
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert!(got.contains("# primary contact"), "{got}");
    assert!(got.contains("# owns releases"), "{got}");
    assert!(got.contains("# keep this provenance"), "{got}");
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    # people published in package metadata
    authors = [
      # primary contact
      { name = "Alice" }, # owns releases
    ]  # keep this provenance
    "#);
}

#[test]
fn test_expand_authors_carries_the_comment_that_led_the_entry() {
    let start = indoc! {r#"
        [project]
        name = "example"
        # people published in package metadata
        authors = [ { name = "Alice" } ]
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"

    # people published in package metadata
    [[project.authors]]
    name = "Alice"
    "#);
}

/// `3.0` has no version below it and `3.255` none above, and a bound outside the supported range
/// narrows nothing.
#[test]
fn test_requires_python_bounds_at_the_numeric_edges() {
    let generated = |requires: &str| {
        let start = format!("[project]\nname = \"example\"\nrequires-python = \"{requires}\"\n");
        let got = format_toml(&start, &generating_classifiers()).unwrap();
        assert_valid_toml(&got);
        got.lines()
            .filter(|line| line.contains("Programming Language :: Python :: 3."))
            .count()
    };

    assert_eq!(generated("<3.0"), 0);
    assert_eq!(generated(">3.255"), 0);
    assert_eq!(generated(">=3.11,<3.10"), 0);
    assert_eq!(generated(">=3.10,<=3.11"), 2);
}

#[test]
fn test_expand_authors_that_hold_nothing() {
    let start = indoc! {r#"
        [project]
        name = "example"
        authors = []
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    authors = []
    "#);
}

#[test]
fn test_expand_authors_with_a_comment_only_inside_a_member() {
    let start = indoc! {r#"
        [project]
        name = "example"
        authors = [ { name = "Alice" }, { name = "Bob" } ]
        maintainers = [
          { name = "Carol" }, # release manager
        ]
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.authors"), String::from("project.maintainers")],
        ..default_settings()
    };
    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert!(got.contains("# release manager"), "{got}");
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    maintainers = [
      { name = "Carol" }, # release manager
    ]

    [[project.authors]]
    name = "Alice"

    [[project.authors]]
    name = "Bob"
    "#);
}

/// The section pass runs after each tool's own, so an order a tool built from the file has to
/// survive it.
#[test]
fn test_a_tool_order_built_from_the_file_survives_the_section_pass() {
    let poetry = format_toml(
        "[tool.poetry]\ngroup.dev.dependencies.pytest = \"^8\"\ngroup.dev.optional = true\n",
        &default_settings(),
    )
    .unwrap();
    assert_snapshot!(poetry, @r#"
    [tool.poetry]
    group.dev.optional = true
    group.dev.dependencies.pytest = "^8"
    "#);

    let pyright = format_toml(
        "[tool.pyright]\nexecutionEnvironments = [ ]\nreportMissingImports = true\n",
        &default_settings(),
    )
    .unwrap();
    assert_snapshot!(pyright, @"
    [tool.pyright]
    reportMissingImports = true
    executionEnvironments = []
    ");
}

/// A setting names a table of any depth, and the table it names may be written either way, so the
/// one it asks for is the one that comes back.
#[test]
fn test_expand_tables_reaches_a_child_written_as_dotted_keys() {
    let start = indoc! {r#"
        [project]
        name = "example"
        urls.Homepage = "https://example.com"
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.urls")],
        ..default_settings()
    };

    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"

    [project.urls]
    Homepage = "https://example.com"
    "#);
    assert_eq!(format_toml(&got, &settings).unwrap(), got);
}

#[test]
fn test_collapse_tables_reaches_a_child_written_as_a_header() {
    let start = indoc! {r#"
        [project]
        name = "example"

        [project.urls]
        Homepage = "https://example.com"
        "#};
    let settings = Settings {
        collapse_tables: vec![String::from("project.urls")],
        ..long_format_settings()
    };

    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    urls.Homepage = "https://example.com"
    "#);
    assert_eq!(format_toml(&got, &settings).unwrap(), got);
}

/// The table a setting names gets a header of its own even where the table above it keeps its keys
/// folded in.
#[test]
fn test_expand_tables_reaches_a_table_below_a_folded_one() {
    let start = indoc! {r#"
        [project]
        name = "example"
        entry-points.special.example = "package:main"
        "#};
    let settings = Settings {
        expand_tables: vec![String::from("project.entry-points.special")],
        ..default_settings()
    };

    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"

    [project.entry-points.special]
    example = "package:main"
    "#);
    assert_eq!(format_toml(&got, &settings).unwrap(), got);
}

#[test]
fn test_collapse_tables_reaches_a_table_below_a_written_out_one() {
    let start = indoc! {r#"
        [project]
        name = "example"

        [project.entry-points.special]
        example = "package:main"
        "#};
    let settings = Settings {
        collapse_tables: vec![String::from("project.entry-points.special")],
        ..long_format_settings()
    };

    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    [project.entry-points]
    special.example = "package:main"
    "#);
    assert_eq!(format_toml(&got, &settings).unwrap(), got);
}

/// A table whose name merely opens with a namespace is its own table, so it keeps the place the
/// file gave it.
#[test]
fn test_a_name_that_only_starts_like_a_namespace_keeps_its_place() {
    let start = indoc! {r#"
        [aaa]
        z = 1

        [toolbox]
        y = 2
        "#};
    let settings = default_settings();

    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @"
    [aaa]
    z = 1

    [toolbox]
    y = 2
    ");
    assert_eq!(format_toml(&got, &settings).unwrap(), got);
}

/// The empty name is a name a file can write, and nothing the formatter orders by is spelled that
/// way, so it is ordered like any other unknown name.
#[test]
fn test_an_empty_name_is_ordered_like_any_other() {
    let start = indoc! {r#"
        [aaa]
        z = 1

        [""]
        e = 1

        [tool.black]
        line-length = 1
        "" = 2
        "#};
    let settings = default_settings();

    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [tool.black]
    line-length = 1
    "" = 2

    [aaa]
    z = 1

    [""]
    e = 1
    "#);
    assert_eq!(format_toml(&got, &settings).unwrap(), got);
}

/// The caller hears why a file was rejected, and hears it before anything is rewritten.
#[test]
fn test_a_source_that_is_not_a_document_is_reported() {
    let refused = format_toml("[project]\nname = \"x\"\nname = \"y\"\n", &default_settings());

    assert_eq!(refused, Err(String::from("`name` is written twice at byte 21")));
}

/// A discriminator names a key one tool writes, not one no other tool may, so an inline table under
/// another tool keeps the order the file gave it.
#[test]
fn test_an_inline_table_under_another_tool_keeps_its_order() {
    let start = indoc! {r#"
        [tool.other]
        config = { z = 1, replace = true, a = 2 }
        source = { z = 1, path = "x", a = 2 }
        overrides = { z = 1, module = "m", a = 2 }
        "#};
    let settings = default_settings();

    let got = format_toml(start, &settings).unwrap();
    assert_valid_toml(&got);
    assert_snapshot!(got, @r#"
    [tool.other]
    config = { z = 1, replace = true, a = 2 }
    source = { z = 1, path = "x", a = 2 }
    overrides = { z = 1, module = "m", a = 2 }
    "#);
    assert_eq!(format_toml(&got, &settings).unwrap(), got);
}

/// A table whose every key is commented out is one the file wrote empty, and folding those keys
/// into the parent would leave nothing saying the table is there.
#[test]
fn test_a_table_of_only_commented_keys_stays_written_out() {
    let start = indoc! {r#"
        [tool.hatch.envs.default]
        dependencies = ["a"]

        [tool.hatch.envs.default.scripts]
        # run = "x"
    "#};
    let result = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    envs.default.dependencies = [ "a" ]

    [tool.hatch.envs.default.scripts]
    # run = "x"
    "#);
    assert_eq!(format_toml(&result, &default_settings()).unwrap(), result);
}

/// A key the file wrote is what says the table is there, so the commented one beside it folds in.
#[test]
fn test_a_table_with_a_key_beside_the_commented_one_folds() {
    let start = indoc! {r#"
        [tool.hatch.envs.default]
        dependencies = ["a"]

        [tool.hatch.envs.default.scripts]
        # run = "x"
        test = "pytest"
    "#};
    let result = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    envs.default.dependencies = [ "a" ]
    # envs.default.scripts.run = "x"
    envs.default.scripts.test = "pytest"
    "#);
    assert_eq!(format_toml(&result, &default_settings()).unwrap(), result);
}

/// A field the file wrote as a comment names nothing the backend fills in, so the classifiers the
/// active metadata implies are still generated.
#[test]
fn test_a_commented_dynamic_leaves_the_active_metadata_alone() {
    let start = indoc! {r#"
        [project]
        name = "demo"
        requires-python = ">=3.11"
        # dynamic = ["classifiers"]
    "#};
    let mut settings = default_settings();
    settings.max_supported_python = (3, 12);
    settings.generate_python_version_classifiers = true;
    let result = format_toml(start, &settings).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "demo"
    requires-python = ">=3.11"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    # dynamic = [ "classifiers" ]
    "#);
    assert_eq!(format_toml(&result, &settings).unwrap(), result);
}

/// A comment written inside a multi-line inline table would swallow the brace if the member it
/// closes the line of moved to the end, so the line it left open is closed where it now sits.
#[test]
fn test_reordering_a_commented_inline_table_stays_readable() {
    let start = "[tool.cibuildwheel]\noverrides = [{ test-command = \"pytest\", # command\n select = \"cp310-*\" }]\n";
    let result = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    overrides = [ {
     select = "cp310-*" , test-command = "pytest" # command
    } ]
    "#);
    assert_eq!(format_toml(&result, &default_settings()).unwrap(), result);
}

/// A key the file wrote as a comment reserves no name, so the extra beside it is still written the
/// way an extra name compares.
#[test]
fn test_a_commented_extra_holds_no_name_back() {
    let start = indoc! {r#"
        [project]
        name = "demo"
        optional-dependencies.Dev_Test = ["pytest"]
        # optional-dependencies.dev-test = ["nose"]
    "#};
    let result = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "demo"
    optional-dependencies.dev-test = [ "pytest" ]
    # optional-dependencies.dev-test = [ "nose" ]
    "#);
    assert_eq!(format_toml(&result, &default_settings()).unwrap(), result);
}

/// A field the file wrote as a comment is not configuration a rule reads, so the members of the
/// inline table it names keep the order the file gave them.
#[test]
fn test_a_commented_field_keeps_the_order_it_was_written_in() {
    let start = indoc! {r#"
        [project]
        name = "demo"
        # authors = [{ email = "a@example.com", name = "Alice" }]
    "#};
    let result = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "demo"
    # authors = [ { email = "a@example.com", name = "Alice" } ]
    "#);
    assert_eq!(format_toml(&result, &default_settings()).unwrap(), result);
}

/// An `overrides` key belongs to the tool whose table it sits in, so mypy's rule for its own
/// overrides leaves another tool's alone.
#[test]
fn test_overrides_under_another_tool_keep_their_order() {
    let start = indoc! {r#"
        [tool.other]
        overrides = [{ module = ["z", "a"] }]

        [tool.mypy]
        overrides = [{ module = ["z", "a"] }]
    "#};
    let result = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    overrides = [ { module = [ "a", "z" ] } ]

    [tool.other]
    overrides = [ { module = [ "z", "a" ] } ]
    "#);
    assert_eq!(format_toml(&result, &default_settings()).unwrap(), result);
}

/// The classifiers the formatter generates name Python 3, so a bound naming another major is one it
/// cannot act on rather than one it asserts against.
#[test]
fn test_settings_reject_a_python_beyond_three() {
    let built = new_settings(Settings {
        max_supported_python: (4, 0),
        generate_python_version_classifiers: true,
        ..default_settings()
    });

    assert!(built.is_err());
}

/// A selector names a table the way TOML names one, so a setting asking for a name no key spells is
/// told rather than read as some other table.
#[test]
fn test_settings_reject_a_selector_that_names_no_table() {
    let built = new_settings(Settings {
        max_supported_python: (3, 12),
        generate_python_version_classifiers: true,
        expand_tables: vec![String::from("project.\"urls")],
        ..default_settings()
    });

    let why = built.err().map(|error| error.to_string()).unwrap_or_default();
    assert!(
        why.contains("expand_tables: project.\"urls is not a table name"),
        "{why}"
    );
}

/// A pattern names a key, which a list holding nothing does not.
#[test]
fn test_settings_reject_a_pattern_written_as_nothing() {
    let built = new_settings(Settings {
        max_supported_python: (3, 12),
        generate_python_version_classifiers: true,
        skip_wrap_for_keys: vec![String::from(" ")],
        ..default_settings()
    });

    let why = built.err().map(|error| error.to_string()).unwrap_or_default();
    assert!(why.contains("skip_wrap_for_keys"), "{why}");
}

/// A disabled key is a comment the file wrote, and the blank lines around it are part of what that
/// comment says: moving one would hand the block to the table below it on the next run.
#[test]
fn test_a_disabled_block_keeps_the_lines_written_around_it() {
    let start = indoc! {r#"
        [build-system]
        requires = [
        ]

        # [tool.setuptools.packages.find]

        # namespaces = false

        [tool.ruff.lint.per-file-ignores]
    "#};
    let result = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [build-system]
    requires = []

    # [tool.setuptools.packages.find]

    # namespaces = false

    [tool.ruff]
    lint.per-file-ignores = {}
    "#);
    assert_eq!(format_toml(&result, &default_settings()).unwrap(), result);
}

/// The whole description is read as one run of words, so a paragraph break says what a space says
/// and the second run finds nothing left to change.
#[test]
fn test_a_description_spread_over_paragraphs_settles_at_once() {
    let start = "[project]\nname = \"demo\"\ndescription = \"\"\"\nfirst paragraph.\n\nsecond paragraph.\n\"\"\"\n";
    let result = format_toml(start, &default_settings()).unwrap();
    assert_valid_toml(&result);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "demo"
    description = "first paragraph. second paragraph."
    "#);
    assert_eq!(format_toml(&result, &default_settings()).unwrap(), result);
}

/// What a folded table takes is read from the way the layout will write it, so how the file
/// happened to indent a nested array does not decide whether the array of tables collapses.
#[test]
fn test_an_array_of_tables_folds_by_what_it_will_be_written_as() {
    let source = |indent: &str| {
        format!(
            "[[tool.mypy.overrides]]\nmodule = [\n{indent}\"one.long.module.name\",\n{indent}\"two.long.module.name\",\n]\nignore_missing_imports = true\n"
        )
    };
    let held = |indent: &str| format_toml(&source(indent), &default_settings()).unwrap();

    assert_eq!(held("  "), held("    "));
    assert_eq!(format_toml(&held("  "), &default_settings()).unwrap(), held("  "));
}

#[test]
fn test_lib_settings_in_reads_the_table_it_is_given() {
    use pyo3::types::PyAnyMethods;

    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let module = pyo3::types::PyModule::new(py, "_lib").unwrap();
        _pyproject_fmt::_lib(&module.as_borrowed()).unwrap();
        let read = module.getattr("settings_in").unwrap();
        let path = vec![String::from("tool"), String::from("pyproject-fmt")];

        let held = read
            .call1((
                "[tool.pyproject-fmt]\ncolumn_width = 30\nexpand_tables = [ \"a\" ]\n",
                path.clone(),
            ))
            .unwrap();
        assert_eq!(held.get_item("column_width").unwrap().to_string(), "30");
        assert_eq!(held.get_item("expand_tables").unwrap().to_string(), "['a']");

        assert!(read
            .call1(("[project]\nname = \"x\"\n", path.clone()))
            .unwrap()
            .is_none());

        let error = read.call1(("key =\n", path.clone())).unwrap_err();
        assert!(error.is_instance_of::<pyo3::exceptions::PySyntaxError>(py));

        let error = read
            .call1(("[tool.pyproject-fmt]\ncolumn_width = 12:30\n", path))
            .unwrap_err();
        assert!(error.is_instance_of::<pyo3::exceptions::PyValueError>(py));
        assert_snapshot!(error.value(py).to_string(), @"column_width: 12:30 is not a setting");
    });
}

fn generating_classifiers() -> Settings {
    Settings {
        generate_python_version_classifiers: true,
        max_supported_python: (3, 11),
        min_supported_python: (3, 10),
        ..default_settings()
    }
}

fn format_toml_helper(
    start: &str,
    indent: usize,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
) -> String {
    let settings = Settings {
        indent,
        keep_full_version,
        max_supported_python,
        generate_python_version_classifiers,
        ..default_settings()
    };
    let result = format_toml(start, &settings).unwrap();
    assert_valid_toml(&result);
    result
}

fn long_format_settings() -> Settings {
    Settings {
        table_format: String::from("long"),
        ..default_settings()
    }
}

fn new_settings(settings: Settings) -> PyResult<Settings> {
    Python::attach(|python| {
        let kwargs = PyDict::new(python);
        kwargs.set_item("column_width", settings.column_width)?;
        kwargs.set_item("indent", settings.indent)?;
        kwargs.set_item("keep_full_version", settings.keep_full_version)?;
        kwargs.set_item("max_supported_python", settings.max_supported_python)?;
        kwargs.set_item("min_supported_python", settings.min_supported_python)?;
        kwargs.set_item(
            "generate_python_version_classifiers",
            settings.generate_python_version_classifiers,
        )?;
        kwargs.set_item("table_format", settings.table_format)?;
        kwargs.set_item("sub_table_spacing", settings.sub_table_spacing)?;
        kwargs.set_item("separate_root_table", settings.separate_root_table)?;
        kwargs.set_item("expand_tables", settings.expand_tables)?;
        kwargs.set_item("collapse_tables", settings.collapse_tables)?;
        kwargs.set_item("skip_wrap_for_keys", settings.skip_wrap_for_keys)?;
        Settings::new(Some(&kwargs))
    })
}
