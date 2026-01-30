use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use indoc::indoc;
use rstest::{fixture, rstest};

use crate::{format_toml, Settings};

#[rstest]
#[case::simple(
        indoc ! {r#"
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
    "#},
        indoc ! {r#"
    # comment
    a = "b"

    [build-system]
    build-backend = "backend"
    requires = [
      "c>=1.5",
      "d==2",
    ]

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
    dependencies = [
      "e>=1.5",
    ]

    [dependency-groups]
    test = [
      "p>1",
    ]

    [tool.mypy]
    mk = "mv"
    "#},
        2,
        false,
        (3, 13),
)]
#[case::empty(
        indoc ! {r""},
        "\n",
        2,
        true,
        (3, 13)
)]
#[case::scripts(
        indoc ! {r#"
    [project.scripts]
    c = "d"
    a = "b"
    "#},
        indoc ! {r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    scripts.a = "b"
    scripts.c = "d"
    "#},
        2,
        true,
        (3, 9)
)]
#[case::subsubtable(
        indoc ! {r"
    [project]
    [tool.coverage.report]
    a = 2
    [tool.coverage]
    a = 0
    [tool.coverage.paths]
    a = 1
    [tool.coverage.run]
    a = 3
    "},
        indoc ! {r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]

    [tool.coverage]
    a = 0
    [tool.coverage.paths]
    a = 1
    [tool.coverage.report]
    a = 2
    [tool.coverage.run]
    a = 3
    "#},
        2,
        true,
        (3, 9)
)]
#[case::array_of_tables(
        indoc ! {r#"
        [tool.commitizen]
        name = "cz_customize"

        [tool.commitizen.customize]
        message_template = ""

        [[tool.commitizen.customize.questions]]
        type = "list"
        [[tool.commitizen.customize.questions]]
        type = "input"
    "#},
        indoc ! {r#"
    [tool.commitizen]
    name = "cz_customize"

    [tool.commitizen.customize]
    message_template = ""

    [[tool.commitizen.customize.questions]]
    type = "list"

    [[tool.commitizen.customize.questions]]
    type = "input"
    "#},
        2,
        true,
        (3, 9)
)]
#[case::unstable_issue_18(
        indoc ! {r#"
    [project]
    requires-python = "==3.12"
    classifiers = [
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Python :: 3.12",
    ]
    [project.urls]
    Source = "https://github.com/VWS-Python/vws-python-mock"

    [tool.setuptools]
    zip-safe = false
    "#},
        indoc ! {r#"
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    urls.Source = "https://github.com/VWS-Python/vws-python-mock"

    [tool.setuptools]
    zip-safe = false
    "#},
        2,
        true,
        (3, 9)
)]
fn test_format_toml(
    #[case] start: &str,
    #[case] expected: &str,
    #[case] indent: usize,
    #[case] keep_full_version: bool,
    #[case] max_supported_python: (u8, u8),
) {
    let settings = Settings {
        column_width: 1,
        indent,
        keep_full_version,
        max_supported_python,
        min_supported_python: (3, 9),
        generate_python_version_classifiers: true,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

#[fixture]
fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("rust")
        .join("src")
        .join("data")
}

#[rstest]
fn test_issue_24(data: PathBuf) {
    let start = read_to_string(data.join("ruff-order.start.toml")).unwrap();
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: true,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start.as_str(), &settings);
    let expected = read_to_string(data.join("ruff-order.expected.toml")).unwrap();
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test that the column width is respected,
/// and that arrays are neither exploded nor collapsed without reason
#[rstest]
fn test_column_width() {
    let start = indoc! {r#"
        [build-system]
        build-backend = "backend"
        requires = ["c>=1.5", "d == 2" ]

        [project]
        name = "beta"
        dependencies = [
        "e>=1.5",
        ]
        "#};
    let settings = Settings {
        column_width: 80,
        indent: 4,
        keep_full_version: false,
        max_supported_python: (3, 13),
        min_supported_python: (3, 13),
        generate_python_version_classifiers: true,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [build-system]
        build-backend = "backend"
        requires = [ "c>=1.5", "d==2" ]

        [project]
        name = "beta"
        classifiers = [
            "Programming Language :: Python :: 3 :: Only",
            "Programming Language :: Python :: 3.13",
        ]
        dependencies = [
            "e>=1.5",
        ]
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test table_format="long" expands sub-tables
#[rstest]
fn test_table_format_long_expands_project_sub_tables() {
    let start = indoc! {r#"
        [project]
        name = "myproject"
        urls.homepage = "https://example.com"
        urls.repository = "https://github.com/example"
        scripts.mycli = "mypackage:main"
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    // Verify sub-tables are expanded (order may vary)
    assert!(got.contains("[project.urls]"));
    assert!(got.contains("[project.scripts]"));
    assert!(got.contains("homepage = "));
    assert!(got.contains("repository = "));
    assert!(got.contains("mycli = "));
    // Verify dotted keys are removed
    assert!(!got.contains("urls.homepage ="));
    assert!(!got.contains("scripts.mycli ="));
    // Verify idempotency
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test table_format="short" collapses sub-tables (default behavior)
#[rstest]
fn test_table_format_short_collapses_project_sub_tables() {
    let start = indoc! {r#"
        [project]
        name = "myproject"

        [project.urls]
        homepage = "https://example.com"
        repository = "https://github.com/example"

        [project.scripts]
        mycli = "mypackage:main"
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    // Verify sub-tables are collapsed
    assert!(got.contains("urls.homepage ="));
    assert!(got.contains("urls.repository ="));
    assert!(got.contains("scripts.mycli ="));
    // Verify expanded tables are removed
    assert!(!got.contains("[project.urls]"));
    assert!(!got.contains("[project.scripts]"));
    // Verify idempotency
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test expand_tables override takes priority over table_format="short"
#[rstest]
fn test_expand_tables_override() {
    let start = indoc! {r#"
        [project]
        name = "myproject"
        urls.homepage = "https://example.com"
        scripts.mycli = "mypackage:main"
        "#};
    let settings = Settings {
        column_width: 1,
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
    // With expand_tables override, project sub-tables should be expanded
    assert!(got.contains("[project.urls]") || got.contains("[project.scripts]"));
}

/// Test collapse_tables override takes priority over expand_tables
#[rstest]
fn test_collapse_tables_priority_over_expand() {
    let start = indoc! {r#"
        [project]
        name = "myproject"

        [project.urls]
        homepage = "https://example.com"
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![String::from("project")],
        collapse_tables: vec![String::from("project")],
    };
    let got = format_toml(start, &settings);
    // collapse_tables takes priority, so urls should be collapsed
    assert!(got.contains("urls.homepage ="));
}

/// Test table_format="long" expands ruff sub-tables
#[rstest]
fn test_table_format_long_expands_ruff_sub_tables() {
    let start = indoc! {r#"
        [tool.ruff]
        lint.select = ["E", "F"]
        lint.ignore = ["E501"]
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    // Verify sub-table is expanded
    assert!(got.contains("[tool.ruff.lint]"));
    assert!(got.contains("select ="));
    assert!(got.contains("ignore ="));
    // Verify dotted keys are removed
    assert!(!got.contains("lint.select ="));
    assert!(!got.contains("lint.ignore ="));
    // Verify idempotency
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test TableFormatConfig.should_collapse priority
#[rstest]
fn test_table_format_config_should_collapse() {
    use crate::TableFormatConfig;
    use std::collections::HashSet;

    // Test default collapse with table_format="short"
    let config = TableFormatConfig {
        default_collapse: true,
        expand_tables: HashSet::new(),
        collapse_tables: HashSet::new(),
    };
    assert!(config.should_collapse("project"));

    // Test default expand with table_format="long"
    let config = TableFormatConfig {
        default_collapse: false,
        expand_tables: HashSet::new(),
        collapse_tables: HashSet::new(),
    };
    assert!(!config.should_collapse("project"));

    // Test expand_tables override
    let mut expand = HashSet::new();
    expand.insert(String::from("project"));
    let config = TableFormatConfig {
        default_collapse: true,
        expand_tables: expand,
        collapse_tables: HashSet::new(),
    };
    assert!(!config.should_collapse("project"));

    // Test collapse_tables priority over expand_tables
    let mut expand = HashSet::new();
    expand.insert(String::from("project"));
    let mut collapse = HashSet::new();
    collapse.insert(String::from("project"));
    let config = TableFormatConfig {
        default_collapse: false,
        expand_tables: expand,
        collapse_tables: collapse,
    };
    assert!(config.should_collapse("project"));
}

/// Test table_format="long" with project dependencies
#[rstest]
fn test_table_format_long_with_dependencies() {
    let start = indoc! {r#"
        [project]
        name = "example"
        dependencies = ["requests>=2.0"]
        optional-dependencies.dev = ["pytest"]
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [project]
        name = "example"
        dependencies = [
          "requests>=2",
        ]
        [project.optional-dependencies]
        dev = [
          "pytest",
        ]
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test table_format="short" with project dependencies
#[rstest]
fn test_table_format_short_with_dependencies() {
    let start = indoc! {r#"
        [project]
        name = "example"
        dependencies = ["requests>=2.0"]
        [project.optional-dependencies]
        dev = ["pytest"]
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [project]
        name = "example"
        dependencies = [
          "requests>=2",
        ]
        optional-dependencies.dev = [
          "pytest",
        ]
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test expand_tables with main table
#[rstest]
fn test_expand_tables_with_project() {
    let start = indoc! {r#"
        [project]
        name = "example"
        optional-dependencies.dev = ["pytest"]
        urls.homepage = "https://example.com"
        "#};
    let settings = Settings {
        column_width: 1,
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
    let expected = indoc! {r#"
        [project]
        name = "example"
        [project.optional-dependencies]
        dev = [
          "pytest",
        ]
        [project.urls]
        homepage = "https://example.com"
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test collapse_tables with tool.ruff
#[rstest]
fn test_collapse_tables_with_ruff() {
    let start = indoc! {r#"
        [tool.ruff]
        [tool.ruff.lint]
        select = ["E", "F"]
        ignore = ["E501"]
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![],
        collapse_tables: vec![String::from("tool.ruff")],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [tool.ruff]
        lint.select = [
          "E",
          "F",
        ]
        lint.ignore = [
          "E501",
        ]
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test table_format="long" with authors array
#[rstest]
fn test_table_format_long_with_authors() {
    let start = indoc! {r#"
        [project]
        name = "example"
        [[project.authors]]
        name = "John Doe"
        email = "john@example.com"
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [project]
        name = "example"
        [[project.authors]]
        name = "John Doe"
        email = "john@example.com"
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test collapse_tables with project.authors
#[rstest]
fn test_collapse_project_authors() {
    let start = indoc! {r#"
        [project]
        name = "example"
        [[project.authors]]
        name = "John Doe"
        email = "john@example.com"
        "#};
    let settings = Settings {
        column_width: 1,
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
    let expected = indoc! {r#"
        [project]
        name = "example"
        authors = [
          { name = "John Doe", email = "john@example.com" },
        ]
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test collapse_tables with project.maintainers
#[rstest]
fn test_collapse_project_maintainers() {
    let start = indoc! {r#"
        [project]
        name = "example"
        [[project.maintainers]]
        name = "Jane Doe"
        email = "jane@example.com"
        "#};
    let settings = Settings {
        column_width: 1,
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
    let expected = indoc! {r#"
        [project]
        name = "example"
        maintainers = [
          { name = "Jane Doe", email = "jane@example.com" },
        ]
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test table_format="long" with entry-points
#[rstest]
fn test_table_format_long_with_entry_points() {
    let start = indoc! {r#"
        [project]
        name = "example"
        entry-points."console_scripts".mycli = "pkg:main"
        entry-points."console_scripts".othercli = "pkg:other"
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("long"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [project]
        name = "example"
        [project.entry-points]
        "console_scripts".mycli = "pkg:main"
        "console_scripts".othercli = "pkg:other"
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test expand_tables with project.authors
#[rstest]
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
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![String::from("project.authors")],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [project]
        name = "example"
        [[project.authors]]
        name = "John Doe"
        email = "john@example.com"

        [[project.authors]]
        name = "Jane Doe"
        email = "jane@example.com"
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test expand_tables with project.maintainers
#[rstest]
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
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![String::from("project.maintainers")],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [project]
        name = "example"
        [[project.maintainers]]
        name = "Bob Smith"
        email = "bob@example.com"

        [[project.maintainers]]
        name = "Alice Jones"
        email = "alice@example.com"
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test expand single author
#[rstest]
fn test_expand_single_author() {
    let start = indoc! {r#"
        [project]
        name = "example"
        authors = [
          { name = "John Doe", email = "john@example.com" },
        ]
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![String::from("project.authors")],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [project]
        name = "example"
        [[project.authors]]
        name = "John Doe"
        email = "john@example.com"
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}
/// Test collapse authors with custom url field (covers line 640 in project.rs)
#[rstest]
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
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [project]
        name = "test"
        authors = [
          { name = "Alice", email = "alice@example.com" },
          { name = "Bob", email = "bob@example.com", url = "https://bob.com" },
        ]
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}
/// Test collapse empty authors (covers line 653 in project.rs)
#[rstest]
fn test_collapse_empty_authors() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [[project.authors]]
        [[project.authors]]
        "#};
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    let expected = indoc! {r#"
        [project]
        name = "test"
        "#};
    assert_eq!(got, expected);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got);
}

/// Test collapse authors when parent doesn't end with newline (covers line 664)
#[rstest]
fn test_collapse_authors_without_trailing_newline() {
    let start = "[project]\nname = \"test\"\n[[project.authors]]\nname = \"Alice\"\nemail = \"alice@example.com\"";
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    assert!(got.contains("authors = ["));
    assert!(got.contains("{ name = \"Alice\", email = \"alice@example.com\" }"));
}

/// Test collapse authors with compact parent table
#[rstest]
fn test_collapse_authors_compact_parent() {
    let start =
        "[project]\nname=\"test\"\nversion=\"1.0\"\n[[project.authors]]\nname=\"Alice\"\nemail=\"alice@example.com\"";
    let settings = Settings {
        column_width: 1,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
    };
    let got = format_toml(start, &settings);
    assert!(got.contains("authors = ["));
}

/// Test expand when authors already in array of tables format (covers line 686)
#[rstest]
fn test_expand_authors_already_expanded() {
    let start = indoc! {r#"
        [project]
        name = "example"
        [[project.authors]]
        name = "John Doe"
        email = "john@example.com"
        "#};
    let settings = Settings {
        column_width: 1,
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
    assert!(got.contains("[[project.authors]]"));
    assert!(got.contains("name = \"John Doe\""));
}
