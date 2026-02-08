use indoc::indoc;
use insta::assert_snapshot;

use super::assert_valid_toml;
use crate::{format_toml, Settings};

fn default_settings() -> Settings {
    Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    }
}

fn long_format_settings() -> Settings {
    Settings {
        table_format: String::from("long"),
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
    let result = format_toml(start, &settings);
    assert_valid_toml(&result);
    result
}

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

/// Test expand_tables with main table
#[test]
fn test_expand_tables_with_project() {
    let start = indoc! {r#"
        [project]
        name = "example"
        optional-dependencies.dev = ["pytest"]
        urls.homepage = "https://example.com"
        "#};
    let settings = Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![String::from("project")],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"

    [project.optional-dependencies]
    dev = [ "pytest" ]

    [project.urls]
    homepage = "https://example.com"
    "#);
}

/// Test collapse_tables with project.authors
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
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![],
        collapse_tables: vec![String::from("project.authors")],
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    authors = [ { name = "John Doe", email = "john@example.com" } ]
    "#);
}

/// Test collapse_tables with project.maintainers
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
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![],
        collapse_tables: vec![String::from("project.maintainers")],
    };
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    maintainers = [ { name = "Jane Doe", email = "jane@example.com" } ]
    "#);
}

/// Test table_format="long" with entry-points
#[test]
fn test_table_format_long_with_entry_points() {
    let start = indoc! {r#"
        [project]
        name = "example"
        entry-points."console_scripts".mycli = "pkg:main"
        entry-points."console_scripts".othercli = "pkg:other"
        "#};
    let got = format_toml(start, &long_format_settings());
    assert_snapshot!(got, @r#"
    [project]
    name = "example"
    [project.entry-points]
    "console_scripts".mycli = "pkg:main"
    "console_scripts".othercli = "pkg:other"
    "#);
}

/// Test expand_tables with project.authors
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
    let got = format_toml(start, &settings);
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

/// Test expand_tables with project.maintainers
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
    let got = format_toml(start, &settings);
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

/// Test expand single author
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
    let got = format_toml(start, &settings);
    assert_snapshot!(got, @r#"
    [project]
    name = "example"

    [[project.authors]]
    name = "John Doe"
    email = "john@example.com"
    "#);
}
/// Test collapse authors with custom url field (covers line 640 in project.rs)
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
    let got = format_toml(start, &default_settings());
    assert_snapshot!(got, @r#"
    [project]
    name = "test"
    authors = [
      { name = "Alice", email = "alice@example.com" },
      { name = "Bob", email = "bob@example.com", url = "https://bob.com" }
    ]
    "#);
}
/// Test collapse empty authors (covers line 653 in project.rs)
#[test]
fn test_collapse_empty_authors() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [[project.authors]]
        [[project.authors]]
        "#};
    let got = format_toml(start, &default_settings());
    assert_snapshot!(got, @r#"
    [project]
    name = "test"

    [[project.authors]]

    [[project.authors]]
    "#);
}

/// Test collapse authors when parent doesn't end with newline (covers line 664)
#[test]
fn test_collapse_authors_without_trailing_newline() {
    let start = "[project]\nname = \"test\"\n[[project.authors]]\nname = \"Alice\"\nemail = \"alice@example.com\"";
    let got = format_toml(start, &default_settings());
    assert!(got.contains("authors = ["));
    assert!(got.contains("{ name = \"Alice\", email = \"alice@example.com\" }"));
}

/// Test collapse authors with compact parent table
#[test]
fn test_collapse_authors_compact_parent() {
    let start =
        "[project]\nname=\"test\"\nversion=\"1.0\"\n[[project.authors]]\nname=\"Alice\"\nemail=\"alice@example.com\"";
    let got = format_toml(start, &default_settings());
    assert!(got.contains("authors = ["));
}

/// Test expand when authors already in array of tables format (covers line 686)
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
    let got = format_toml(start, &settings);
    assert!(got.contains("[[project.authors]]"));
    assert!(got.contains("name = \"John Doe\""));
}

/// Test issue 146: expand_tables keeps specific sub-table expanded while others collapse
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
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![String::from("project.optional-dependencies")],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    assert!(
        got.contains("[project.optional-dependencies]"),
        "optional-dependencies should stay expanded"
    );
    assert!(got.contains("urls.homepage ="), "urls should be collapsed");
}

/// Test CSS-like specificity: more specific selector wins
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
        column_width: 120,
        indent: 4,
        keep_full_version: true,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![String::from("project.urls")],
        collapse_tables: vec![String::from("project")],
    };
    let got = format_toml(start, &settings);
    assert!(
        got.contains("[project.urls]"),
        "project.urls should be expanded (specific)"
    );
    assert!(
        got.contains("optional-dependencies.dev ="),
        "optional-dependencies should be collapsed (inherits project)"
    );
}

/// Test nested table specificity: project.entry-points.tox can be different from project.entry-points
#[test]
fn test_nested_table_specificity() {
    use crate::TableFormatConfig;
    use std::collections::HashSet;

    let mut expand = HashSet::new();
    expand.insert(String::from("project.entry-points.special"));

    let mut collapse = HashSet::new();
    collapse.insert(String::from("project.entry-points"));

    let config = TableFormatConfig {
        default_collapse: false,
        expand_tables: expand,
        collapse_tables: collapse,
    };

    assert!(
        config.should_collapse("project.entry-points"),
        "project.entry-points should collapse"
    );
    assert!(
        config.should_collapse("project.entry-points.tox"),
        "project.entry-points.tox inherits collapse"
    );
    assert!(
        !config.should_collapse("project.entry-points.special"),
        "project.entry-points.special should expand"
    );
}

/// Test parent inheritance: sub-table inherits from parent setting
#[test]
fn test_parent_inheritance() {
    use crate::TableFormatConfig;
    use std::collections::HashSet;

    let mut expand = HashSet::new();
    expand.insert(String::from("project"));

    let config = TableFormatConfig {
        default_collapse: true,
        expand_tables: expand,
        collapse_tables: HashSet::new(),
    };

    assert!(!config.should_collapse("project"), "project should expand");
    assert!(
        !config.should_collapse("project.urls"),
        "project.urls inherits expand from project"
    );
    assert!(
        !config.should_collapse("project.optional-dependencies"),
        "project.optional-dependencies inherits expand"
    );
}

/// Test that default_collapse is used when no specific setting exists
#[test]
fn test_default_collapse_fallback() {
    use crate::TableFormatConfig;
    use std::collections::HashSet;

    let config = TableFormatConfig {
        default_collapse: true,
        expand_tables: HashSet::new(),
        collapse_tables: HashSet::new(),
    };

    assert!(config.should_collapse("project"));
    assert!(config.should_collapse("project.urls"));
    assert!(config.should_collapse("tool.ruff.lint"));
}

/// Test issue 146 with deeply nested ruff table: expand_tables works for deep paths
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
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![String::from("tool.ruff.lint.flake8-tidy-imports.banned-api")],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
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
    let got = format_toml(start, &default_settings());
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
    let got = format_toml(start, &long_format_settings());
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
    let got = format_toml(start, &long_format_settings());
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
    let got = format_toml(start, &long_format_settings());
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
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![String::from("project.authors")],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
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
    let got = format_toml(start, &default_settings());
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
    let got = format_toml(start, &long_format_settings());
    assert_snapshot!(got, @"
    [tool.coverage.report]
    precision = 2
    [tool.coverage.run]
    branch = true
    ");
}

#[test]
fn test_should_collapse_with_no_dot_in_name() {
    use crate::TableFormatConfig;
    use std::collections::HashSet;

    let config = TableFormatConfig {
        default_collapse: true,
        expand_tables: HashSet::new(),
        collapse_tables: HashSet::new(),
    };

    assert!(config.should_collapse("project"));
    assert!(config.should_collapse("build-system"));
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
    let got = format_toml(start, &long_format_settings());
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
    let settings = Settings::new(
        120,
        4,
        true,
        (3, 13),
        (3, 9),
        true,
        String::from("short"),
        vec![String::from("project.urls")],
        vec![String::from("project.authors")],
    );
    assert_eq!(settings.column_width, 120);
    assert_eq!(settings.indent, 4);
    assert!(settings.keep_full_version);
    assert_eq!(settings.max_supported_python, (3, 13));
    assert_eq!(settings.min_supported_python, (3, 9));
    assert!(settings.generate_python_version_classifiers);
    assert_eq!(settings.table_format, "short");
    assert_eq!(settings.expand_tables, vec!["project.urls"]);
    assert_eq!(settings.collapse_tables, vec!["project.authors"]);
}

#[test]
fn test_table_format_config_from_settings() {
    use crate::TableFormatConfig;

    let settings = Settings::new(
        120,
        2,
        false,
        (3, 12),
        (3, 9),
        false,
        String::from("short"),
        vec![String::from("tool.ruff")],
        vec![String::from("project")],
    );
    let config = TableFormatConfig::from_settings(&settings);
    assert!(config.default_collapse);
    assert!(config.expand_tables.contains("tool.ruff"));
    assert!(config.collapse_tables.contains("project"));
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
fn test_idempotent_formatting() {
    let start = indoc! {r#"
        [project]
        name = "test"
        description = "This is a long description string that needs to exceed the default column width of one hundred and twenty characters to trigger wrapping."
    "#};
    let settings = Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let first = format_toml(start, &settings);
    let second = format_toml(&first, &settings);
    let third = format_toml(&second, &settings);
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
    let got = format_toml(start, &default_settings());
    assert_snapshot!(got, @r#"
    [tool.something]
    items = [
      "first",
      # A comment
      "second",
    ]
    "#);
}
