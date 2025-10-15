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
    [tool.coverage.report]
    a = 2
    [tool.coverage.paths]
    a = 1
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
