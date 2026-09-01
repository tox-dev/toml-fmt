use std::collections::HashSet;

use common::sections;
use indoc::indoc;

use super::default_settings;
use _pyproject_fmt::{format_toml, Settings, TableFormatConfig};

#[test]
fn test_project_no_project_section() {
    let start = "";
    let result = evaluate_project(start, false, (3, 9), true);
    insta::assert_snapshot!(result, @"");
}

#[test]
fn test_project_dependencies_normalize_no_keep() {
    let start = indoc! {r#"
        [project]
        dependencies=["a>=1.0.0", "b.c>=1.5.0"]
    "#};
    let result = evaluate_project(start, false, (3, 9), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    dependencies = [ "a>=1", "b-c>=1.5" ]
    "#);
}

#[test]
fn test_project_dependencies_normalize_keep_version() {
    let start = indoc! {r#"
        [project]
        dependencies=["a>=1.0.0", "b.c>=1.5.0"]
    "#};
    let result = evaluate_project(start, true, (3, 9), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    dependencies = [ "a>=1.0.0", "b-c>=1.5.0" ]
    "#);
}

#[test]
fn test_project_optional_dependencies() {
    let start = indoc! {r#"
        [project.optional-dependencies]
        dev = ["pytest>=7.0.0"]
        docs = ["sphinx>=4.0.0"]
    "#};
    let result = evaluate_project(start, false, (3, 13), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
      "Programming Language :: Python :: 3.13",
    ]
    optional-dependencies.dev = [ "pytest>=7" ]
    optional-dependencies.docs = [ "sphinx>=4" ]
    "#);
}

#[test]
fn test_project_classifiers_generated() {
    let start = indoc! {r#"
        [project]
        name = "test"
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
}

#[test]
fn test_project_classifiers_no_generation() {
    let start = indoc! {r#"
        [project]
        name = "test"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    "#);
}

#[test]
fn test_project_readme_inline_table() {
    let start = indoc! {r#"
        [project]
        readme = { file = "README.md", content-type = "text/markdown" }
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    readme = { file = "README.md", content-type = "text/markdown" }
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_license_inline_table() {
    let start = indoc! {r#"
        [project]
        license = { file = "LICENSE" }
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    license = { file = "LICENSE" }
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_urls_sorting() {
    let start = indoc! {r#"
        [project.urls]
        Repository = "https://github.com/example/repo"
        Documentation = "https://docs.example.com"
        Homepage = "https://example.com"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    urls.Documentation = "https://docs.example.com"
    urls.Homepage = "https://example.com"
    urls.Repository = "https://github.com/example/repo"
    "#);
}

/// A person reads name first however the file wrote the element, so folding one in does not leave
/// the email in front.
#[test]
fn test_project_people_keep_their_order_when_folded_in() {
    let start = indoc! {r#"
        [project]
        authors = [{ email = "john@example.com", name = "John Doe" }]
        maintainers = [{ email = "maintain@example.com", name = "Jane Smith" }]
    "#};
    let result = evaluate_project(start, false, (3, 9), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    maintainers = [ { name = "Jane Smith", email = "maintain@example.com" } ]
    authors = [ { name = "John Doe", email = "john@example.com" } ]
    "#);
}

#[test]
fn test_project_authors_maintainers() {
    let start = indoc! {r#"
        [project]
        authors = [
            { name = "John Doe", email = "john@example.com" },
            { name = "Jane Smith" }
        ]
        maintainers = [
            { email = "maintain@example.com" }
        ]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    maintainers = [ { email = "maintain@example.com" } ]
    authors = [ { name = "John Doe", email = "john@example.com" }, { name = "Jane Smith" } ]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_requires_python() {
    let start = indoc! {r#"
        [project]
        requires-python = ">=3.9"
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">=3.9"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
}

#[test]
fn test_project_keywords() {
    let start = indoc! {r#"
        [project]
        keywords = ["testing", "formatting", "toml"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    keywords = [ "formatting", "testing", "toml" ]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_entry_points() {
    let start = indoc! {r#"
        [project.entry-points."console_scripts"]
        mytool = "mypackage:main"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    entry-points.console_scripts.mytool = "mypackage:main"
    "#);
}

#[test]
fn test_project_scripts() {
    let start = indoc! {r#"
        [project.scripts]
        mytool = "mypackage:main"
        another = "mypackage.cli:run"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    scripts.another = "mypackage.cli:run"
    scripts.mytool = "mypackage:main"
    "#);
}

#[test]
fn test_project_gui_scripts() {
    let start = indoc! {r#"
        [project.gui-scripts]
        mygui = "mypackage.gui:main"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    gui-scripts.mygui = "mypackage.gui:main"
    "#);
}

#[test]
fn test_project_dynamic_fields() {
    let start = indoc! {r#"
        [project]
        dynamic = ["version", "description"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dynamic = [ "description", "version" ]
    "#);
}

#[test]
fn test_project_full_metadata() {
    let start = indoc! {r#"
        [project]
        name = "my-package"
        version = "1.0.0"
        description = "A test package"
        requires-python = ">=3.9"
        dependencies = ["requests>=2.28.0"]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "my-package"
    version = "1.0.0"
    description = "A test package"
    requires-python = ">=3.9"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    dependencies = [ "requests>=2.28" ]
    "#);
}

#[test]
fn test_project_classifiers_multiple_python_versions() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.9"
    "#};
    let result = evaluate_project(start, false, (3, 13), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.9"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
      "Programming Language :: Python :: 3.13",
    ]
    "#);
}

#[test]
fn test_project_optional_dependencies_multiple_groups() {
    let start = indoc! {r#"
        [project.optional-dependencies]
        dev = ["pytest>=7.0.0", "black>=22.0"]
        docs = ["sphinx>=4.0.0", "myst-parser>=0.18"]
        test = ["coverage>=6.0"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    optional-dependencies.dev = [ "black>=22", "pytest>=7" ]
    optional-dependencies.docs = [ "myst-parser>=0.18", "sphinx>=4" ]
    optional-dependencies.test = [ "coverage>=6" ]
    "#);
}

#[test]
fn test_project_authors_only_email() {
    let start = indoc! {r#"
        [project]
        authors = [{ email = "dev@example.com" }]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    authors = [ { email = "dev@example.com" } ]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_authors_only_name() {
    let start = indoc! {r#"
        [project]
        authors = [{ name = "Developer Name" }]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    authors = [ { name = "Developer Name" } ]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_license_text() {
    let start = indoc! {r#"
        [project]
        license = { text = "MIT" }
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    license = { text = "MIT" }
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_readme_string() {
    let start = indoc! {r#"
        [project]
        readme = "README.md"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    readme = "README.md"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_multiple_entry_point_groups() {
    let start = indoc! {r#"
        [project.entry-points."pytest11"]
        myplugin = "mypackage.plugin:pytest_plugin"
        [project.entry-points."console_scripts"]
        mytool = "mypackage:main"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    entry-points.console_scripts.mytool = "mypackage:main"
    entry-points.pytest11.myplugin = "mypackage.plugin:pytest_plugin"
    "#);
}

#[test]
fn test_project_version_classifiers_range() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.10,<3.14"
    "#};
    let result = evaluate_project(start, false, (3, 14), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.10,<3.14"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
      "Programming Language :: Python :: 3.13",
    ]
    "#);
}

#[test]
fn test_project_dependencies_with_extras() {
    let start = indoc! {r#"
        [project]
        dependencies = ["requests[security]>=2.28.0", "click[colorama]>=8.0"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = [ "click[colorama]>=8", "requests[security]>=2.28" ]
    "#);
}

#[test]
fn test_project_dependencies_with_markers() {
    let start = indoc! {r#"
        [project]
        dependencies = [
            "importlib-metadata>=4.0; python_version<'3.10'",
            "typing-extensions>=4.0; python_version<'3.11'"
        ]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = [ "importlib-metadata>=4; python_version<'3.10'", "typing-extensions>=4; python_version<'3.11'" ]
    "#);
}

#[test]
fn test_project_urls_multiple() {
    let start = indoc! {r#"
        [project.urls]
        Homepage = "https://example.com"
        Documentation = "https://docs.example.com"
        Repository = "https://github.com/example/repo"
        "Bug Tracker" = "https://github.com/example/repo/issues"
        Changelog = "https://github.com/example/repo/blob/main/CHANGELOG.md"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    urls."Bug Tracker" = "https://github.com/example/repo/issues"
    urls.Changelog = "https://github.com/example/repo/blob/main/CHANGELOG.md"
    urls.Documentation = "https://docs.example.com"
    urls.Homepage = "https://example.com"
    urls.Repository = "https://github.com/example/repo"
    "#);
}

#[test]
fn test_project_existing_classifiers_preserved() {
    let start = indoc! {r#"
        [project]
        name = "test"
        classifiers = [
            "Development Status :: 4 - Beta",
            "License :: OSI Approved :: MIT License"
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    classifiers = [
      "Development Status :: 4 - Beta",
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
}

#[test]
fn test_project_empty_dependencies() {
    let start = indoc! {r#"
        [project]
        dependencies = []
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = []
    "#);
}

#[test]
fn test_project_empty_optional_dependencies() {
    let start = indoc! {r#"
        [project.optional-dependencies]
        dev = []
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    optional-dependencies.dev = []
    "#);
}

#[test]
fn test_project_normalize_package_name_underscores() {
    let start = indoc! {r#"
        [project]
        dependencies = ["my_package>=1.0.0", "another.package>=2.0"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = [ "another-package>=2", "my-package>=1" ]
    "#);
}

#[test]
fn test_project_dependencies_git_urls() {
    let start = indoc! {r#"
        [project]
        dependencies = ["pkg @ git+https://github.com/user/repo.git@main"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = [ "pkg @ git+https://github.com/user/repo.git@main" ]
    "#);
}

/// A URL holds no whitespace and may hold a semicolon of its own, so a marker beside one is opened
/// by the space PEP 508 writes before it; text without that space is left as the file wrote it.
#[test]
fn test_project_dependencies_git_urls_with_marker() {
    let start = indoc! {r#"
        [project]
        dependencies = ["pkg @ git+https://github.com/user/repo.git@main ; python_version>='3.10'"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = [ "pkg @ git+https://github.com/user/repo.git@main ; python_version>='3.10'" ]
    "#);
}

#[test]
fn test_project_dependencies_local_paths() {
    let start = indoc! {r#"
        [project]
        dependencies = ["pkg @ file:///path/to/package"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = [ "pkg @ file:///path/to/package" ]
    "#);
}

/// The classifiers a project carries say which Python versions it runs on, and what the requirement
/// admits is what says which those are: a series the requirement leaves out gets no classifier, and
/// one it admits at any patch level gets one.
#[test]
fn the_generated_classifiers_say_what_the_requirement_admits() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "classifiers_with_requires_python",
            ">=3.9,<3.13",
            &[
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.9",
                "Programming Language :: Python :: 3.10",
                "Programming Language :: Python :: 3.11",
                "Programming Language :: Python :: 3.12",
            ],
        ),
        (
            "requires_python_greater_than",
            ">3.9",
            &[
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.9",
                "Programming Language :: Python :: 3.10",
                "Programming Language :: Python :: 3.11",
                "Programming Language :: Python :: 3.12",
            ],
        ),
        (
            "requires_python_less_than_or_equal",
            ">=3.9,<=3.11",
            &[
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.9",
                "Programming Language :: Python :: 3.10",
                "Programming Language :: Python :: 3.11",
            ],
        ),
        (
            "requires_python_not_equal",
            ">=3.9,!=3.10",
            &[
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.9",
                "Programming Language :: Python :: 3.10",
                "Programming Language :: Python :: 3.11",
                "Programming Language :: Python :: 3.12",
            ],
        ),
        (
            "requires_python_exact_version",
            "==3.11",
            &[
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.11",
            ],
        ),
        (
            "requires_python_compatible",
            "~=3.10",
            &[
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.10",
                "Programming Language :: Python :: 3.11",
                "Programming Language :: Python :: 3.12",
            ],
        ),
        (
            "requires_python_compatible_to_a_patch",
            "~=3.10.0",
            &[
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.10",
            ],
        ),
        (
            "requires_python_not_equal_to_a_series",
            ">=3.9,!=3.10.*",
            &[
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.9",
                "Programming Language :: Python :: 3.11",
                "Programming Language :: Python :: 3.12",
            ],
        ),
        (
            "requires_python_below_a_patch",
            "<3.10.1",
            &[
                "Programming Language :: Python :: 3.9",
                "Programming Language :: Python :: 3.10",
            ],
        ),
        ("requires_a_python_beyond_three", ">=4", &[]),
    ];
    for (name, requires_python, held) in cases {
        let start = format!(
            "[project]\nname = \"test\"\nrequires-python = \"{requires_python}\"\nclassifiers = [\"License :: OSI Approved :: MIT License\"]\n"
        );
        let listed: String = std::iter::once("License :: OSI Approved :: MIT License")
            .chain(held.iter().copied())
            .map(|held| format!("  \"{held}\",\n"))
            .collect();

        assert_eq!(
            evaluate_project(&start, false, (3, 12), true),
            format!(
                "[project]\nname = \"test\"\nrequires-python = \"{requires_python}\"\nclassifiers = [\n{listed}]\n"
            ),
            "{name}"
        );
    }
}

#[test]
fn test_project_python_version_3_8() {
    let start = indoc! {r#"
        [project]
        name = "test"
    "#};
    let result = evaluate_project(start, false, (3, 8), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    "#);
}

#[test]
fn test_project_python_version_3_14() {
    let start = indoc! {r#"
        [project]
        name = "test"
    "#};
    let result = evaluate_project(start, false, (3, 14), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
      "Programming Language :: Python :: 3.13",
      "Programming Language :: Python :: 3.14",
    ]
    "#);
}

#[test]
fn test_project_all_fields() {
    let start = indoc! {r#"
        [project]
        name = "my-package"
        version = "1.0.0"
        description = "A comprehensive test"
        readme = "README.md"
        requires-python = ">=3.9"
        license = { text = "MIT" }
        authors = [{ name = "Dev", email = "dev@example.com" }]
        maintainers = [{ name = "Maintainer" }]
        keywords = ["test", "example"]
        classifiers = ["Development Status :: 4 - Beta"]
        dependencies = ["requests>=2.28.0"]

        [project.optional-dependencies]
        dev = ["pytest>=7.0"]

        [project.urls]
        Homepage = "https://example.com"

        [project.scripts]
        mytool = "mypackage:main"
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "my-package"
    version = "1.0.0"
    description = "A comprehensive test"
    readme = "README.md"
    keywords = [ "example", "test" ]
    license = { text = "MIT" }
    maintainers = [ { name = "Maintainer" } ]
    authors = [ { name = "Dev", email = "dev@example.com" } ]
    requires-python = ">=3.9"
    classifiers = [
      "Development Status :: 4 - Beta",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    dependencies = [ "requests>=2.28" ]
    optional-dependencies.dev = [ "pytest>=7" ]
    urls.Homepage = "https://example.com"
    scripts.mytool = "mypackage:main"
    "#);
}

#[test]
fn test_project_optional_deps_with_underscores() {
    let start = indoc! {r#"
        [project.optional-dependencies]
        test_extra = ["pytest>=7.0"]
        dev_tools = ["black>=22.0"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    optional-dependencies.dev-tools = [ "black>=22" ]
    optional-dependencies.test-extra = [ "pytest>=7" ]
    "#);
}

#[test]
fn test_project_maintainers_multiple() {
    let start = indoc! {r#"
        [project]
        maintainers = [
            { name = "Alice", email = "alice@example.com" },
            { name = "Bob" },
            { email = "charlie@example.com" }
        ]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    maintainers = [ { name = "Alice", email = "alice@example.com" }, { name = "Bob" }, { email = "charlie@example.com" } ]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_complex_requires_python() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.9"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.9"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_dependencies_url_format() {
    let start = indoc! {r#"
        [project]
        dependencies = [
            "pkg @ https://example.com/pkg-1.0.tar.gz",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = [
      "pkg @ https://example.com/pkg-1.0.tar.gz",
    ]
    "#);
}

#[test]
fn test_project_entry_points_inline_tables() {
    let start = indoc! {r#"
        [[project.entry-points]]
        name = "console_scripts"
        value = { mytool = "mypackage:main", another = "mypackage:other" }
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    entry-points = [ { name = "console_scripts", value = { mytool = "mypackage:main", another = "mypackage:other" } } ]
    "#);
}

#[test]
fn test_project_scripts_inline_table() {
    let start = indoc! {r#"
        [project]
        scripts = { mytool = "mypackage:main", helper = "mypackage.cli:run" }
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    scripts = { mytool = "mypackage:main", helper = "mypackage.cli:run" }
    "#);
}

#[test]
fn test_project_gui_scripts_inline_table() {
    let start = indoc! {r#"
        [project]
        gui-scripts = { mygui = "mypackage.gui:main" }
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    gui-scripts = { mygui = "mypackage.gui:main" }
    "#);
}

#[test]
fn test_project_with_table_format_expand() {
    let start = indoc! {r#"
        [project]
        name = "test"
        version = "1.0.0"
        authors = [{ name = "Dev" }]

        [project.urls]
        Homepage = "https://example.com"
    "#};
    let result = evaluate_config(
        start,
        TableFormatConfig {
            default_collapse: false,
            expand: HashSet::new(),
            collapse: HashSet::new(),
        },
        true,
        (3, 12),
    );
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    version = "1.0.0"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    [[project.authors]]
    name = "Dev"
    [project.urls]
    Homepage = "https://example.com"
    "#);
}

#[test]
fn test_project_optional_deps_normalize_names() {
    let start = indoc! {r#"
        [project.optional-dependencies]
        Test_Extra = ["pytest>=7.0"]
        Dev-Tools = ["black>=22.0"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    optional-dependencies.dev-tools = [ "black>=22" ]
    optional-dependencies.test-extra = [ "pytest>=7" ]
    "#);
}

#[test]
fn test_project_classifiers_python_only() {
    let start = indoc! {r#"
        [project]
        name = "test"
        classifiers = [
            "Development Status :: 4 - Beta",
            "Programming Language :: Python :: 3.9",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    classifiers = [
      "Development Status :: 4 - Beta",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
}

#[test]
fn test_project_classifiers_add_python_3_only() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.10"
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.10"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
}

#[test]
fn test_project_min_python_equals_max() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.11"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.11"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_dependencies_complex_markers() {
    let start = indoc! {r#"
        [project]
        dependencies = [
            "pkg>=1.0; python_version<'3.10' and platform_system=='Linux'",
            "other>=2.0; (python_version>='3.10' or sys_platform!='win32')",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    dependencies = [
      "other>=2; (python_version>='3.10' or sys_platform!='win32')",
      "pkg>=1; python_version<'3.10' and platform_system=='Linux'",
    ]
    "#);
}

#[test]
fn test_project_dependencies_multiple_extras() {
    let start = indoc! {r#"
        [project]
        dependencies = ["pkg[extra1,extra2,extra3]>=1.0"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = [ "pkg[extra1,extra2,extra3]>=1" ]
    "#);
}

#[test]
fn test_project_urls_with_special_chars() {
    let start = indoc! {r#"
        [project.urls]
        "Bug Tracker" = "https://github.com/user/repo/issues"
        "Source Code" = "https://github.com/user/repo"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    urls."Bug Tracker" = "https://github.com/user/repo/issues"
    urls."Source Code" = "https://github.com/user/repo"
    "#);
}

#[test]
fn test_project_readme_content_type() {
    let start = indoc! {r#"
        [project]
        readme = { file = "README.rst", content-type = "text/x-rst" }
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    readme = { file = "README.rst", content-type = "text/x-rst" }
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_license_spdx() {
    let start = indoc! {r#"
        [project]
        license = { text = "Apache-2.0" }
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    license = { text = "Apache-2.0" }
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_dynamic_version_only() {
    let start = indoc! {r#"
        [project]
        name = "test"
        dynamic = ["version"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dynamic = [ "version" ]
    "#);
}

#[test]
fn test_project_dependencies_duplicate_handling() {
    let start = indoc! {r#"
        [project]
        dependencies = ["requests>=2.28", "requests>=2.30"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = [ "requests>=2.28", "requests>=2.30" ]
    "#);
}

#[test]
fn test_project_classifiers_preserve_order() {
    let start = indoc! {r#"
        [project]
        name = "test"
        classifiers = [
            "Development Status :: 5 - Production/Stable",
            "Intended Audience :: Developers",
            "License :: OSI Approved :: MIT License",
            "Programming Language :: Python :: 3",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    classifiers = [
      "Development Status :: 5 - Production/Stable",
      "Intended Audience :: Developers",
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
}

#[test]
fn test_project_license_and_or_with_keywords() {
    let start = indoc! {r#"
        [project]
        name = "test"
        license = "MIT and Apache-2.0 or GPL-2.0-only with Classpath-exception-2.0"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    license = "MIT AND Apache-2.0 OR GPL-2.0-only WITH Classpath-exception-2.0"
    "#);
}

#[test]
fn test_project_import_names() {
    let start = indoc! {r#"
        [project]
        name = "test"
        import-names = ["zebra", "alpha.sub ;private"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    import-names = [ "alpha.sub; private", "zebra" ]
    "#);
}

/// PEP 794 writes a dotted name of Python identifiers and lets `private` follow it. Anything else
/// is text this cannot read, which the file keeps as it wrote it.
#[test]
fn test_project_import_names_it_cannot_read() {
    let start = indoc! {r#"
        [project]
        name = "test"
        import-names = ["zebra;  extra", "9lives", "a..b", "held;private;private", "ok"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    import-names = [ "9lives", "a..b", "held;private;private", "ok", "zebra;  extra" ]
    "#);
}

#[test]
fn test_project_import_namespaces() {
    let start = indoc! {r#"
        [project]
        name = "test"
        import-namespaces = ["z.namespace", "a.namespace ;  private"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    import-namespaces = [ "a.namespace; private", "z.namespace" ]
    "#);
}

#[test]
fn test_project_description_multiline() {
    let start = indoc! {r#"
        [project]
        name = "test"
        description = """
        This is a   multiline
        description   with   extra   spaces
        """
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    description = "This is a multiline description with extra spaces"
    "#);
}

#[test]
fn test_project_keywords_dedupe() {
    let start = indoc! {r#"
        [project]
        name = "test"
        keywords = ["python", "Python", "PYTHON", "rust", "Rust"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    keywords = [ "python", "rust" ]
    "#);
}

#[test]
fn test_project_authors_preserve_order() {
    let start = indoc! {r#"
        [project]
        name = "test"
        authors = [
            {name = "Zoe", email = "zoe@example.com"},
            {name = "Alice", email = "alice@example.com"},
            {name = "Bob", email = "bob@example.com"},
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    authors = [
      { name = "Zoe", email = "zoe@example.com" },
      { name = "Alice", email = "alice@example.com" },
      { name = "Bob", email = "bob@example.com" },
    ]
    "#);
}

#[test]
fn test_project_authors_preserve_order_email_only() {
    let start = indoc! {r#"
        [project]
        name = "test"
        authors = [
            {email = "zoe@example.com"},
            {email = "alice@example.com"},
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    authors = [
      { email = "zoe@example.com" },
      { email = "alice@example.com" },
    ]
    "#);
}

#[test]
fn test_project_maintainers_preserve_order() {
    let start = indoc! {r#"
        [project]
        name = "test"
        maintainers = [
            {name = "Charlie"},
            {name = "Alice"},
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    maintainers = [
      { name = "Charlie" },
      { name = "Alice" },
    ]
    "#);
}

#[test]
fn test_project_requires_python_whitespace() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">= 3.9, < 4"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.9,<4"
    "#);
}

#[test]
fn test_project_name_normalization() {
    let start = indoc! {r#"
        [project]
        name = "My_Package.Name"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "my-package-name"
    "#);
}

#[test]
fn test_project_dependencies_same_package_different_markers() {
    let start = indoc! {r#"
        [project]
        name = "test"
        dependencies = [
            "pkg>=1.0; python_version<'3.10'",
            "pkg>=2.0; python_version>='3.10'",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    dependencies = [
      "pkg>=1; python_version<'3.10'",
      "pkg>=2; python_version>='3.10'",
    ]
    "#);
}

#[test]
fn test_project_optional_dependencies_same_package_different_markers() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [project.optional-dependencies]
        dev = [
            "pytest>=6.0; python_version<'3.10'",
            "pytest>=7.0; python_version>='3.10'",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    optional-dependencies.dev = [
      "pytest>=6; python_version<'3.10'",
      "pytest>=7; python_version>='3.10'",
    ]
    "#);
}

#[test]
fn test_project_dynamic_sorting() {
    let start = indoc! {r#"
        [project]
        name = "test"
        dynamic = ["version", "description", "authors"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    dynamic = [ "authors", "description", "version" ]
    "#);
}

#[test]
fn test_project_classifiers_sorting_and_dedup() {
    let start = indoc! {r#"
        [project]
        name = "test"
        classifiers = [
            "Development Status :: 4 - Beta",
            "License :: OSI Approved :: MIT License",
            "Development Status :: 4 - Beta",
            "Intended Audience :: Developers",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    classifiers = [
      "Development Status :: 4 - Beta",
      "Intended Audience :: Developers",
      "License :: OSI Approved :: MIT License",
    ]
    "#);
}

#[test]
fn test_project_entry_points_inline_table_expansion() {
    let start = indoc! {r#"
        [project]
        name = "test"
        entry-points.console_scripts = {foo = "pkg:main", bar = "pkg:bar"}
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    entry-points.console_scripts.bar = "pkg:bar"
    entry-points.console_scripts.foo = "pkg:main"
    "#);
}

#[test]
fn test_project_classifiers_filter_existing_python_versions() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.10"
        classifiers = [
            "Development Status :: 4 - Beta",
            "Programming Language :: Python :: 3.8",
            "Programming Language :: Python :: 3.9",
            "Programming Language :: Python :: 3.10",
            "Programming Language :: Python :: 3.11",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.10"
    classifiers = [
      "Development Status :: 4 - Beta",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_authors_same_name_preserve_order() {
    let start = indoc! {r#"
        [project]
        name = "test"
        authors = [
            {name = "Alice", email = "z@example.com"},
            {name = "Alice", email = "a@example.com"},
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    authors = [
      { name = "Alice", email = "z@example.com" },
      { name = "Alice", email = "a@example.com" },
    ]
    "#);
}

#[test]
fn test_project_scripts_multiple_sorting() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [project.scripts]
        zzz = "pkg:zzz"
        aaa = "pkg:aaa"
        mmm = "pkg:mmm"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    scripts.aaa = "pkg:aaa"
    scripts.mmm = "pkg:mmm"
    scripts.zzz = "pkg:zzz"
    "#);
}

#[test]
fn test_project_gui_scripts_sorting() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [project.gui-scripts]
        window_z = "pkg:z"
        window_a = "pkg:a"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    gui-scripts.window_a = "pkg:a"
    gui-scripts.window_z = "pkg:z"
    "#);
}

#[test]
fn test_project_urls_all_common() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [project.urls]
        Changelog = "https://example.com/changelog"
        Documentation = "https://example.com/docs"
        Homepage = "https://example.com"
        Repository = "https://github.com/user/repo"
        "Bug Tracker" = "https://github.com/user/repo/issues"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    urls."Bug Tracker" = "https://github.com/user/repo/issues"
    urls.Changelog = "https://example.com/changelog"
    urls.Documentation = "https://example.com/docs"
    urls.Homepage = "https://example.com"
    urls.Repository = "https://github.com/user/repo"
    "#);
}

#[test]
fn test_project_classifiers_implementation_cpython() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.9"
        classifiers = [
            "Programming Language :: Python :: Implementation :: CPython",
            "Programming Language :: Python :: 3.9",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.9"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: Implementation :: CPython",
    ]
    "#);
}

#[test]
fn test_project_classifiers_no_generation_keeps_existing() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.10"
        classifiers = [
            "Development Status :: 4 - Beta",
            "Programming Language :: Python :: 3.10",
            "Programming Language :: Python :: 3.11",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 11), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.10"
    classifiers = [
      "Development Status :: 4 - Beta",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_readme_file_path() {
    let start = indoc! {r#"
        [project]
        name = "test"
        readme = "README.md"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    readme = "README.md"
    "#);
}

#[test]
fn test_project_license_file() {
    let start = indoc! {r#"
        [project]
        name = "test"
        license = {file = "LICENSE"}
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    license = { file = "LICENSE" }
    "#);
}

#[test]
fn test_project_version_field() {
    let start = indoc! {r#"
        [project]
        name = "test"
        version = "1.2.3"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    version = "1.2.3"
    "#);
}

#[test]
fn test_project_version_calver_kept_verbatim() {
    let start = indoc! {r#"
        [project]
        version = "2026.08.10"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    version = "2026.08.10"
    "#);
}

#[test]
fn test_project_version_non_canonical_kept_verbatim() {
    let start = indoc! {r#"
        [project]
        version = "V1.0-Alpha.2-1.DEV+Ubuntu_01"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    version = "V1.0-Alpha.2-1.DEV+Ubuntu_01"
    "#);
}

#[test]
fn test_project_version_invalid() {
    let start = indoc! {r#"
        [project]
        version = "1.9.xyz"
    "#};
    let error = evaluate_project_error(start);
    insta::assert_snapshot!(error, @"project.version `1.9.xyz` is not a valid PEP 440 version");
}

#[test]
fn test_project_version_not_a_string() {
    let start = indoc! {r#"
        [project]
        version = 19
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @"
    [project]
    version = 19
    ");
}

#[test]
fn test_project_dependencies_empty_markers() {
    let start = indoc! {r#"
        [project]
        name = "test"
        dependencies = [
            "requests>=2.0",
            "urllib3",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    dependencies = [
      "requests>=2",
      "urllib3",
    ]
    "#);
}

#[test]
fn test_project_optional_deps_empty_group() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [project.optional-dependencies]
        dev = []
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    optional-dependencies.dev = []
    "#);
}

#[test]
fn test_project_array_of_tables_authors() {
    let start = indoc! {r#"
        [project]
        name = "test"

        [[project.authors]]
        name = "Alice"
        email = "alice@example.com"

        [[project.authors]]
        name = "Bob"
        email = "bob@example.com"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    authors = [ { name = "Alice", email = "alice@example.com" }, { name = "Bob", email = "bob@example.com" } ]
    "#);
}

#[test]
fn test_project_array_of_tables_maintainers() {
    let start = indoc! {r#"
        [project]
        name = "test"

        [[project.maintainers]]
        name = "Charlie"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    maintainers = [ { name = "Charlie" } ]
    "#);
}

#[test]
fn test_project_normalize_optional_deps_names() {
    let start = indoc! {r#"
        [project]
        name = "test"
        [project.optional-dependencies]
        Dev_Test = ["pytest>=7"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    optional-dependencies.dev-test = [ "pytest>=7" ]
    "#);
}

/// `~=3.10.0` names the 3.10 series, one component more precise than `~=3.10`.
/// `!=3.10` rules out that one release while `!=3.10.*` rules out the series it covers.
/// `<3.10` leaves the 3.10 series out, while `<3.10.1` keeps the releases below the one it names.
/// A project no Python 3 release satisfies is not a Python 3 project, so it is given none of the
/// classifiers that would say it is.
/// The clauses are met by one release or by none: a bound met by a later patch and one met by an
/// earlier patch leave no release between them.
#[test]
fn test_project_requires_python_clauses_that_meet_nowhere() {
    let contradictory = |written: &str| {
        let start = format!("[project]\nname = \"test\"\nrequires-python = \"{written}\"\n");
        let result = evaluate_project(&start, false, (3, 12), true);
        assert!(!result.contains("Programming Language"), "{result}");
    };

    contradictory(">=3.10.5,<3.10.5");
    contradictory("==3.10.1,!=3.10.1");
    // an ordinary Python release names no epoch, so none of them is at or above this bound
    contradictory(">=1!3");
}

/// Every clause narrows one window, so what is left is what one release of the series can be.
#[test]
fn test_project_requires_python_reads_the_whole_specifier_set() {
    let minors = |written: &str| {
        let start = format!("[project]\nname = \"test\"\nrequires-python = \"{written}\"\n");
        evaluate_project(&start, false, (3, 12), true)
            .lines()
            .filter_map(|line| line.trim().strip_prefix("\"Programming Language :: Python :: 3."))
            .map(|rest| rest.trim_end_matches("\",").to_owned())
            .collect::<Vec<String>>()
    };
    let named = |held: &[&str]| held.iter().map(|held| (*held).to_owned()).collect::<Vec<String>>();

    assert_eq!(minors("==3.10.*"), named(&["10"]));
    // `~=3.10.1.2` holds the series to `3.10.1`, which is below the bound it also names
    assert_eq!(minors("~=3.10.1.2"), named(&[]));
    assert_eq!(minors("~=3.10.1"), named(&["10"]));
    assert_eq!(minors("==4.*"), named(&[]));
    // a bound below an epoch is above every release that names none
    assert_eq!(minors("<2!1"), named(&["9", "10", "11", "12"]));
    assert_eq!(minors(">=3.10.5,<3.10.1"), named(&[]));
    assert_eq!(minors("<3.10.1,>=3.10.5"), named(&[]));
    assert_eq!(minors(">3.10.1,<=3.10.1"), named(&[]));
    // no release of three numbers lies between two adjacent patches
    assert_eq!(minors(">=3.10.1,>3.10.1,<3.10.2"), named(&[]));
    assert_eq!(minors(">3.10.1,<3.10.3"), named(&["10"]));
    assert_eq!(minors(">3.10.1,>=3.10.1,<=3.10.5,<3.10.5"), named(&["10"]));
    assert_eq!(minors("==3.10.1.*"), named(&["10"]));
    assert_eq!(minors(">=3.10.5,>=3.10.1"), named(&["10", "11", "12"]));
    assert_eq!(minors(">=3.10.1,>=3.10.5"), named(&["10", "11", "12"]));
    // a bound on a pre release is below the plain one, so the plain one is above it
    assert_eq!(minors("<3.10.1rc1,>=3.10.1"), named(&[]));
    assert_eq!(minors("<3.10.1rc1,>=3.10"), named(&["10"]));
    assert_eq!(minors("<=3.10.5,<=3.10.1"), named(&["9", "10"]));
    assert_eq!(minors("<=3.10.1,<=3.10.5"), named(&["9", "10"]));
    // an exclusion outside the series rules out nothing in it, and neither does a pre-release
    assert_eq!(minors("==3.10,!=4.10"), named(&["10"]));
    assert_eq!(minors("==3.10,!=3.10rc1"), named(&["10"]));
    assert_eq!(minors("!=4.*"), named(&["9", "10", "11", "12"]));
    assert_eq!(minors("!=3.*"), named(&[]));
    // `===` names the one text it was given, which no Python release is written as
    assert_eq!(minors("===foobar"), named(&[]));
    assert_eq!(minors("===v3.10"), named(&[]));
    // an interpreter says its version as three numbers, which is the text `===` compares
    assert_eq!(minors("===3.10"), named(&[]));
    assert_eq!(minors("===3.10.0"), named(&["10"]));
    // a release written after the plain one is not that release, and is above it
    assert_eq!(minors("==3.10.post1"), named(&[]));
    assert_eq!(minors("==3.10+vendor"), named(&[]));
    assert_eq!(minors("<3.10.post1"), named(&["9", "10"]));
    // a bound on a post release is above the plain one however the bound is written, so the plain
    // release is under it and a window closed on it holds nothing
    assert_eq!(minors(">=3.10.1.post1,<=3.10.1"), named(&[]));
    assert_eq!(minors(">3.10.1.post1,<=3.10.1"), named(&[]));
    assert_eq!(minors(">=3.10.1+vendor,<=3.10.1"), named(&[]));
    assert_eq!(minors(">=3.10.1,<=3.10.1"), named(&["10"]));
    // the plain release follows a pre or dev one, so a window closed on it still holds that release
    assert_eq!(minors(">3.10.1rc1,<=3.10.1"), named(&["10"]));
    assert_eq!(minors(">3.10.1.dev1,<=3.10.1"), named(&["10"]));
    // a later patch above the bound still satisfies it, so the series keeps its classifier
    assert_eq!(minors(">=3.10.1.post1,<3.10.3"), named(&["10"]));
    assert_eq!(minors(">=3.10.1.post1,<3.10.2"), named(&[]));
    assert_eq!(minors(">3.10.1rc1,<3.10.2"), named(&["10"]));
    assert_eq!(minors(">3.10.1.dev1,<3.10.2"), named(&["10"]));
    // an exclusion naming more numbers than a release has rules out no release
    assert_eq!(minors(">=3.10.1,<=3.10.1,!=3.10.1.5"), named(&["10"]));
    assert_eq!(minors(">=3.10.1,<=3.10.1,!=3.10.1"), named(&[]));
    // a wildcard rules out the releases opening with its numbers, and nothing else
    assert_eq!(minors("!=3.10.1.*"), named(&["9", "10", "11", "12"]));
    assert_eq!(minors("!=3.10.1.2.*"), named(&["9", "10", "11", "12"]));
    assert_eq!(minors("!=3.10.1.0.*,>=3.10.1,<=3.10.1"), named(&[]));
    assert_eq!(minors("!=3.11.1.*,>=3.11,<3.12"), named(&["11"]));
    // an exclusion no release of the series is written as rules out none of them
    assert_eq!(minors("==3.10.1,!=3.10.1.post1"), named(&["10"]));
    assert_eq!(minors("==3.10.1,!=3.10.1+local"), named(&["10"]));
    assert_eq!(minors("==3.10.1.*,!=3.10.1.post1"), named(&["10"]));
    // a bound that names only the series says its micro version is zero
    assert_eq!(minors(">=3.10,<3.10.1,!=3.10"), named(&[]));
    assert_eq!(minors(">=3.10,<3.10.2,!=3.10"), named(&["10"]));
    // `===` compares the text, which an interpreter writes without leading zeros
    assert_eq!(minors("===3.10.00"), named(&[]));
    // a release names as many digits as it likes, and the window counts every one of them
    assert_eq!(
        minors(">3.10.18446744073709551615,<3.10.18446744073709551617"),
        named(&["10"])
    );
    assert_eq!(
        minors(">3.10.18446744073709551615,<=3.10.18446744073709551615"),
        named(&[])
    );
    // `~=` opens where `>=` does, so a bound on a post release starts above the plain one
    assert_eq!(minors("<=3.10.1,~=3.10.1.post1"), named(&[]));
    assert_eq!(minors("==3.10.1,~=3.10.1.post1"), named(&[]));
    assert_eq!(minors("<3.10.2,~=3.10.1.post1"), named(&[]));
    assert_eq!(minors("<3.10.3,~=3.10.1.post1"), named(&["10"]));
    // a pre release stays under the final one whatever follows it, and a post release stays
    // above it even where a dev release follows
    assert_eq!(minors("<=3.10.1,>=3.10.1.post1.dev1"), named(&[]));
    assert_eq!(minors("<=3.10.1.post1.dev1,>=3.10.1"), named(&["10"]));
    assert_eq!(minors("<3.10.1,>3.10.1.post1.dev1"), named(&[]));
    assert_eq!(minors("<3.10.1.post1.dev1,>3.10.1"), named(&[]));
    assert_eq!(minors("<=3.10.1,>=3.10.1a1.post1"), named(&["10"]));
    assert_eq!(minors("==3.10.1a1.post1"), named(&[]));
    assert_eq!(minors("==3.10.1.post1.dev1"), named(&[]));
    assert_eq!(minors("===3.10.foo"), named(&[]));
}

/// `3 :: Only` says no major other than 3 runs the project, which a constraint a Python 2 release
/// satisfies does not.
#[test]
fn test_project_classifiers_for_a_range_that_is_not_python_3_only() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=2.7"
        classifiers = ["Programming Language :: Python :: 2.7"]
    "#};
    let result = evaluate_project(start, false, (3, 10), true);

    assert!(!result.contains("3 :: Only"), "{result}");
    assert!(result.contains("Programming Language :: Python :: 2.7"), "{result}");
    assert!(result.contains("Programming Language :: Python :: 3.10"), "{result}");
}

/// The configured minimum is the floor the formatter falls back to, not one it holds the project's
/// own constraint to.
#[test]
fn test_project_classifiers_below_the_configured_minimum() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.8,<3.11"
        classifiers = [
          "Programming Language :: Python :: 3.8",
          "Programming Language :: Python :: 3.9",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.8,<3.11"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.8",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
    ]
    "#);
}

/// A classifier is one of a fixed set of strings, so an invalid spelling beside the valid one is
/// two claims rather than one written twice.
#[test]
fn test_project_classifiers_that_differ_only_in_case() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = "==3.10"
        classifiers = [
          "programming language :: python :: 3.10",
          "Programming Language :: Python :: 3.10",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 10), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = "==3.10"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "programming language :: python :: 3.10",
      "Programming Language :: Python :: 3.10",
    ]
    "#);
}

/// A constraint this cannot read still says what the project supports, so the classifiers beside it
/// are left as the file wrote them and the text itself is not tidied into something else.
#[test]
fn test_project_requires_python_that_is_not_a_specifier() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = "three"
        classifiers = ["License :: OSI Approved :: MIT License"]
    "#};
    let result = evaluate_project(start, false, (3, 10), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = "three"
    classifiers = [ "License :: OSI Approved :: MIT License" ]
    "#);
}

/// Whitespace between the clauses says nothing, while whitespace inside one is what makes the text
/// something PEP 440 does not read.
#[test]
fn test_project_requires_python_holding_whitespace_inside_a_clause() {
    let written = |text: &str| {
        let start = format!("[project]\nname = \"test\"\nrequires-python = \"{text}\"\n");
        evaluate_project(&start, false, (3, 12), true)
    };

    insta::assert_snapshot!(written(">= 3.9, < 4"), @r#"
    [project]
    name = "test"
    requires-python = ">=3.9,<4"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
    for held in ["> = 3.10", ">=3 . 10", "=== foo bar"] {
        let result = written(held);
        assert!(result.contains(&format!("requires-python = \"{held}\"")), "{result}");
        assert!(!result.contains("Programming Language"), "{result}");
    }
}

#[test]
fn test_project_expand_authors_to_array_of_tables() {
    let start = indoc! {r#"
        [project]
        name = "test"
        authors = [{ name = "Alice", email = "alice@example.com" }, { name = "Bob" }]
    "#};
    let expand_tables = HashSet::from([common::sections::parse_name("project.authors")]);
    let result = evaluate_config(
        start,
        TableFormatConfig {
            default_collapse: false,
            expand: expand_tables,
            collapse: HashSet::new(),
        },
        false,
        (3, 12),
    );
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    [[project.authors]]
    name = "Alice"
    email = "alice@example.com"

    [[project.authors]]
    name = "Bob"
    "#);
}

#[test]
fn test_project_expand_maintainers_to_array_of_tables() {
    let start = indoc! {r#"
        [project]
        name = "test"
        maintainers = [{ name = "Charlie", email = "charlie@example.com" }]
    "#};
    let expand_tables = HashSet::from([common::sections::parse_name("project.maintainers")]);
    let result = evaluate_config(
        start,
        TableFormatConfig {
            default_collapse: false,
            expand: expand_tables,
            collapse: HashSet::new(),
        },
        false,
        (3, 12),
    );
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    [[project.maintainers]]
    name = "Charlie"
    email = "charlie@example.com"
    "#);
}

#[test]
fn test_project_classifiers_generated_from_requires_python() {
    let start = indoc! {r#"
        [project]
        requires-python = ">=3.9"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">=3.9"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_classifiers_preserve_existing() {
    let start = indoc! {r#"
        [project]
        requires-python = ">=3.9"
        classifiers = [
          "License :: OSI Approved :: MIT License",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 10), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">=3.9"
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
    ]
    "#);
}

#[test]
fn test_project_classifiers_filter_unsupported_versions() {
    let start = indoc! {r#"
        [project]
        requires-python = ">=3.9"
        classifiers = [
          "Programming Language :: Python :: 3.8",
          "Programming Language :: Python :: 3.12",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 10), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">=3.9"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
    ]
    "#);
}

#[test]
fn test_project_classifiers_with_comments() {
    let start = indoc! {r#"
        [project]
        requires-python = ">=3.9"
        classifiers = [
          # license comment
          "License :: OSI Approved :: MIT License", # inline comment
          "Programming Language :: Python :: 3.10", # version comment
        ]
    "#};
    let result = evaluate_project(start, false, (3, 10), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">=3.9"
    classifiers = [
      # license comment
      "License :: OSI Approved :: MIT License",      # inline comment
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",      # version comment
    ]
    "#);
}

#[test]
fn test_project_classifiers_greater_than() {
    let start = indoc! {r#"
        [project]
        requires-python = ">3.8"
    "#};
    let result = evaluate_project(start, false, (3, 10), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">3.8"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.8",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
    ]
    "#);
}

#[test]
fn test_project_classifiers_no_generate() {
    let start = indoc! {r#"
        [project]
        requires-python = ">=3.9"
    "#};
    let result = evaluate_project(start, false, (3, 10), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">=3.9"
    "#);
}

#[test]
fn test_project_classifiers_sort_and_dedupe() {
    let start = indoc! {r#"
        [project]
        requires-python = ">=3.9"
        classifiers = [
          "Programming Language :: Python :: 3.10",
          "License :: OSI Approved :: MIT License",
          "Programming Language :: Python :: 3.9",
          "Development Status :: 4 - Beta",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 10), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">=3.9"
    classifiers = [
      "Development Status :: 4 - Beta",
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
    ]
    "#);
}

#[test]
fn test_project_classifiers_wide_range() {
    let start = indoc! {r#"
        [project]
        requires-python = ">=3.9"
    "#};
    let result = evaluate_project(start, false, (3, 13), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">=3.9"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
      "Programming Language :: Python :: 3.13",
    ]
    "#);
}

#[test]
fn test_project_classifiers_single_line_array() {
    let start = indoc! {r#"
        [project]
        requires-python = ">=3.10"
        classifiers = ["License :: OSI Approved :: MIT License"]
    "#};
    let result = evaluate_project(start, false, (3, 11), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">=3.10"
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_classifiers_no_trailing_comma_multiline() {
    let start = indoc! {r#"
        [project]
        requires-python = ">=3.9"
        classifiers = [
            "License :: OSI Approved :: MIT License",
            "Development Status :: 5 - Production/Stable"
        ]
    "#};
    let result = evaluate_project(start, false, (3, 10), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    requires-python = ">=3.9"
    classifiers = [
      "Development Status :: 5 - Production/Stable",
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
    ]
    "#);
}

#[test]
fn test_project_classifiers_with_invalid_classifier() {
    let start = indoc! {r#"
        [project]
        name = "test"
        version = "0.0.1"
        classifiers = ["Programming Language :: Python :: 3", "a :: string"]
    "#};
    let result = evaluate_project(start, false, (3, 13), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    version = "0.0.1"
    classifiers = [
      "a :: string",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
      "Programming Language :: Python :: 3.13",
    ]
    "#);
}

#[test]
fn test_project_dependencies_group_markers() {
    let start = indoc! {r#"
        [project]
        name = "x"
        version = "1"
        dependencies = [
          # Group: web
          "flask",
          "django",
          # Group: db
          "sqlalchemy",
          "psycopg2",
        ]
    "#};
    let result = evaluate_project(start, false, (3, 9), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "x"
    version = "1"
    dependencies = [
      # Group: web
      "django",
      "flask",
      # Group: db
      "psycopg2",
      "sqlalchemy",
    ]
    "#);
}

/// A file this parser cannot read is still the user's file, so nothing may be dropped from it.
#[test]
fn test_project_keeps_a_requirement_it_cannot_read() {
    let start = indoc! {r#"
    [project]
    name = "alpha"
    version = "1.0"
    dependencies = ["good >= 1.0.0", "!! not a requirement !!", "other"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "alpha"
    version = "1.0"
    dependencies = [ "!! not a requirement !!", "good>=1", "other" ]
    "#);
}

/// One comment above the table is about the table, not about each key it becomes, and a comment
/// written beside a member belongs to that member.
#[test]
fn test_project_entry_points_carry_their_comments_once() {
    let start = indoc! {r#"
    [project]
    name = "demo"
    # plugins
    entry-points.group = { a = "demo:a", b = "demo:b" } # beside
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "demo"
    # plugins
    entry-points.group.a = "demo:a"
    entry-points.group.b = "demo:b"  # beside
    "#);
}

#[test]
fn test_project_entry_points_keep_comments_written_around_a_member() {
    let start = indoc! {r#"
    [project]
    name = "demo"
    entry-points.group = {
      # about a
      a = "demo:a", # beside a
      b = "demo:b",
    }
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "demo"
    # about a
    # beside a
    entry-points.group.a = "demo:a"
    entry-points.group.b = "demo:b"
    "#);
}

/// Two spellings of one extra are two keys until one is rewritten on top of the other.
#[test]
fn test_project_keeps_both_spellings_of_an_extra_that_would_collide() {
    let start = indoc! {r#"
    [project]
    name = "demo"
    optional-dependencies.My_Extra = ["a"]
    optional-dependencies.my-extra = ["b"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "demo"
    optional-dependencies.my-extra = [ "b" ]
    optional-dependencies.My_Extra = [ "a" ]
    "#);
}

/// `"name.foo"` is one key the file quoted whole, not the `name` field with a suffix, so no rule
/// written for `name` may touch it.
#[test]
fn test_project_leaves_a_quoted_key_holding_a_dot_alone() {
    let start = indoc! {r#"
    [project]
    name = "My_Project"
    "name.foo" = "My_Project"
    "version.bar" = "1.9.xyz"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "my-project"
    "name.foo" = "My_Project"
    "version.bar" = "1.9.xyz"
    "#);
}

/// The group's name is the one segment after the field, whatever characters it holds, so what is
/// normalized is that name rather than the way the file spelled the whole key.
#[test]
fn test_project_normalizes_a_quoted_extra_name_as_a_name() {
    let start = indoc! {r#"
    [project]
    name = "demo"
    optional-dependencies."My.Extra" = ["a"]
    optional-dependencies."Other_Extra" = ["b"]
    optional-dependencies."third-Extra" = ["c"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    assert!(result.parse::<toml::Table>().is_ok(), "{result}");
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "demo"
    optional-dependencies.my-extra = [ "a" ]
    optional-dependencies.other-extra = [ "b" ]
    optional-dependencies.third-extra = [ "c" ]
    "#);
}

/// A comment before the closing brace has no dotted key to move to, so the table stays as it is.
#[test]
fn test_project_entry_points_keep_a_comment_before_the_closing_brace() {
    let start = indoc! {r#"
    [project]
    name = "demo"
    entry-points.console = {
      command = "package:main",
      # explains the complete entry-point group
    }
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    assert!(result.contains("# explains the complete entry-point group"), "{result}");
    assert!(result.parse::<toml::Table>().is_ok(), "{result}");
}

/// A moved comment takes a line of its own however the file it came from ended.
#[test]
fn test_project_entry_points_expand_at_end_of_file_without_a_break() {
    let start = "[project]\nname = \"demo\"\nentry-points.console = { a = \"p:a\", b = \"p:b\" # beside b\n}";
    let result = evaluate_project(start, false, (3, 12), false);
    let read: toml::Table = result.parse().unwrap_or_else(|error| panic!("{error}: {result}"));

    assert!(result.contains("# beside b"), "{result}");
    assert_eq!(
        read["project"]["entry-points"]["console"]
            .as_table()
            .expect("a table")
            .len(),
        2,
        "{result}"
    );
}

/// The project's name is one distribution name, so text naming anything else is left as the file
/// wrote it.
#[test]
fn test_project_name_that_is_not_a_name_is_left_alone() {
    let start = indoc! {r#"
        [project]
        name = "pkg[feature]"
    "#};
    let result = evaluate_project(start, false, (3, 9), false);

    insta::assert_snapshot!(result, @r#"
    [project]
    name = "pkg[feature]"
    "#);
}

/// The deprecated license table holds a path and the license text itself, neither of which is an
/// SPDX expression.
#[test]
fn test_project_license_table_is_not_read_as_an_expression() {
    let start = indoc! {r#"
        [project]
        license.file = "LICENSE and NOTICE"
    "#};
    let result = evaluate_project(start, false, (3, 9), false);

    insta::assert_snapshot!(result, @r#"
    [project]
    license.file = "LICENSE and NOTICE"
    "#);
}

/// A field a file writes as dynamic is one its backend fills in, so the formatter does not write it
/// out as well.
#[test]
fn test_project_dynamic_classifiers_are_left_to_the_backend() {
    let classifiers = indoc! {r#"
        [project]
        name = "demo"
        dynamic = ["classifiers"]
    "#};
    let requires_python = indoc! {r#"
        [project]
        name = "demo"
        dynamic = ["requires-python"]
    "#};

    insta::assert_snapshot!(evaluate_project(classifiers, false, (3, 11), true), @r#"
    [project]
    name = "demo"
    dynamic = [ "classifiers" ]
    "#);
    insta::assert_snapshot!(evaluate_project(requires_python, false, (3, 11), true), @r#"
    [project]
    name = "demo"
    dynamic = [ "requires-python" ]
    "#);
}

/// `dynamic` names a list of fields, and a file writing anything else there has named none of them.
#[test]
fn test_project_dynamic_written_as_one_name_names_no_field() {
    let start = indoc! {r#"
        [project]
        name = "demo"
        dynamic = "classifiers"
        requires-python = ">=3.11"
    "#};

    insta::assert_snapshot!(evaluate_project(start, false, (3, 11), true), @r#"
    [project]
    name = "demo"
    requires-python = ">=3.11"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.11",
    ]
    dynamic = "classifiers"
    "#);
}

/// An SPDX expression names licenses joined by `AND`, `OR` and `WITH`; text that is not one is a
/// license description the file wrote, and uppercasing the words in it would rewrite what it says.
#[test]
fn test_project_license_that_is_not_an_expression() {
    let written = |held: &str| {
        let start = format!("[project]\nname = \"x\"\nlicense = \"{held}\"\n");
        evaluate_project(&start, false, (3, 10), false)
    };

    insta::assert_snapshot!(written("BSD 3-Clause New or Revised License"), @r#"
    [project]
    name = "x"
    license = "BSD 3-Clause New or Revised License"
    "#);
    insta::assert_snapshot!(written("(MIT or Apache-2.0) and BSD-3-Clause"), @r#"
    [project]
    name = "x"
    license = "(MIT OR Apache-2.0) AND BSD-3-Clause"
    "#);
    insta::assert_snapshot!(written("GPL-2.0-only with Classpath-exception-2.0"), @r#"
    [project]
    name = "x"
    license = "GPL-2.0-only WITH Classpath-exception-2.0"
    "#);

    // prose shaped like an expression names no license, and an identifier no register holds is not
    // one however the words around it are written
    for held in [
        "MIT or later",
        "foo and bar",
        "GPL-2.0-only with MIT",
        "mit or apache-2.0",
    ] {
        let start = format!("[project]\nname = \"x\"\nlicense = \"{held}\"\n");
        let result = evaluate_project(&start, false, (3, 10), false);
        assert!(result.contains(&format!("license = \"{held}\"")), "{result}");
    }

    // a parenthesis that opens where a license belongs, one that closes where none is open, and an
    // operator with nothing before it each say the text is not an expression
    for held in ["MIT (or Apache-2.0)", "MIT or Apache-2.0)", "or MIT"] {
        let start = format!("[project]\nname = \"x\"\nlicense = \"{held}\"\n");
        let result = evaluate_project(&start, false, (3, 10), false);
        assert!(result.contains(&format!("license = \"{held}\"")), "{result}");
    }
}

/// An extra names a distribution, so text that does not is left for the backend to report rather
/// than rewritten into something that is not one either.
#[test]
fn test_project_extra_names_outside_the_grammar() {
    let start = indoc! {r#"
        [project]
        name = "x"
        optional-dependencies."FOO🔥" = []
        optional-dependencies.".FOO" = []
        optional-dependencies.Dev_Tools = []
    "#};
    let result = evaluate_project(start, false, (3, 10), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "x"
    optional-dependencies.".FOO" = []
    optional-dependencies."FOO🔥" = []
    optional-dependencies.dev-tools = []
    "#);
}

/// A range naming only an upper bound still says how old a release may be, so a classifier it
/// admits stays even when the configured minimum sits above it.
#[test]
fn test_project_classifiers_an_upper_only_range_admits_are_kept() {
    let written = |bound: &str| {
        let start = format!(
            "[project]\nname = \"test\"\nrequires-python = \"{bound}\"\nclassifiers = [\
             \"Programming Language :: Python :: 3.8\", \"Programming Language :: Python :: 3 :: Only\"]\n"
        );
        evaluate_project(&start, false, (3, 11), true)
    };

    assert!(
        written("<=3.8").contains("Programming Language :: Python :: 3.8"),
        "{}",
        written("<=3.8")
    );
    assert!(
        !written("<3.8").contains("Programming Language :: Python :: 3.8"),
        "{}",
        written("<3.8")
    );
}

/// Where the range names no lower bound, the configured minimum decides how far back the file is
/// given classifiers it does not already name.
#[test]
fn test_project_classifiers_written_for_an_upper_only_range_start_at_the_configured_minimum() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = "<=3.11"
    "#};
    let result = evaluate_project(start, false, (3, 11), true);

    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = "<=3.11"
    classifiers = [
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

/// A bound on a post release is above the plain release with the same numbers, so a window closed
/// on that plain release holds no release of the series at all.
#[test]
fn test_project_classifiers_for_a_window_closed_on_a_post_release() {
    let written = |bound: &str| {
        let start = format!(
            "[project]\nname = \"test\"\nrequires-python = \"{bound}\"\n\
             classifiers = [\"Programming Language :: Python :: 3.10\"]\n"
        );
        evaluate_project(&start, false, (3, 12), true)
    };

    let closed = written(">=3.10.1.post1,<=3.10.1");
    assert!(!closed.contains("Programming Language :: Python :: 3.10"), "{closed}");
    let open = written(">=3.10.1.post1,<3.10.3");
    assert!(open.contains("Programming Language :: Python :: 3.10"), "{open}");
}

/// TOML gives every spelling of a table the same name, so the same rules run on each of them.
#[test]
fn test_a_table_is_formatted_however_the_file_splits_its_path() {
    let written = super::evaluate_full;

    assert_eq!(
        written("tool.black.target-version = [ \"py311\", \"py310\" ]\n"),
        "tool.black.target-version = [ \"py310\", \"py311\" ]\n"
    );
    assert_eq!(
        written("[tool]\nblack.target-version = [ \"py311\", \"py310\" ]\n"),
        "[tool]\nblack.target-version = [ \"py310\", \"py311\" ]\n"
    );
    assert_eq!(
        written("[tool.black]\ntarget-version = [ \"py311\", \"py310\" ]\n"),
        "[tool.black]\ntarget-version = [ \"py310\", \"py311\" ]\n"
    );
    assert_eq!(
        written("tool.black = { target-version = [ \"py311\", \"py310\" ] }\n"),
        "tool.black = { target-version = [ \"py310\", \"py311\" ] }\n"
    );
}

/// Every rule reads the table it names, wherever the file wrote its keys: before the first header,
/// inside a table written as a value, or under a header of its own.
#[test]
fn test_every_rule_reads_the_table_the_file_wrote() {
    fn default_settings() -> _pyproject_fmt::Settings {
        _pyproject_fmt::Settings {
            column_width: 120,
            indent: 2,
            keep_full_version: false,
            max_supported_python: (3, 12),
            min_supported_python: (3, 10),
            generate_python_version_classifiers: false,
            table_format: String::from("short"),
            sub_table_spacing: String::new(),
            separate_root_table: String::from("\n"),
            expand_tables: vec![],
            collapse_tables: vec![],
            skip_wrap_for_keys: vec![],
        }
    }
    let written = |start: &str| {
        use _pyproject_fmt::format_toml;

        let held = format_toml(start, &default_settings()).expect("the formatter accepts it");
        assert_eq!(
            format_toml(&held, &default_settings()).expect("the formatter accepts what it wrote"),
            held,
            "the formatter settled on its first pass"
        );
        held
    };

    // a rule that normalizes a value reaches it wherever the value is written
    assert_eq!(
        written("build-system.requires = [ \"Z>=2.0.0\", \"a>=1.0.0\" ]\n"),
        "build-system.requires = [ \"a>=1\", \"z>=2\" ]\n"
    );
    assert_eq!(
        written("dependency-groups.dev = [ \"Z>=2.0.0\", \"a>=1.0.0\" ]\n"),
        "dependency-groups.dev = [ \"a>=1\", \"z>=2\" ]\n"
    );
    assert_eq!(
        written("tool = { ruff = { lint = { select = [ \"W\", \"E\" ] } } }\n"),
        "tool = { ruff = { lint = { select = [ \"E\", \"W\" ] } } }\n"
    );
    assert_eq!(
        written("project = { optional-dependencies = { Dev = [ \"Z>=2.0.0\", \"a>=1.0.0\" ] } }\n"),
        "project = { optional-dependencies = { dev = [ \"a>=1\", \"z>=2\" ] } }\n"
    );
    // a rule that orders keys reaches them the same way
    assert_eq!(
        written("tool = { black = { exclude = \"x\", required-version = \"1\" } }\n"),
        "tool = { black = { required-version = \"1\", exclude = \"x\" } }\n"
    );
    assert_eq!(
        written("[tool.ruff.lint]\nignore = [ \"E\" ]\nselect = [ \"W\" ]\n"),
        "[tool.ruff]\nlint.select = [ \"W\" ]\nlint.ignore = [ \"E\" ]\n"
    );
    // a group the file marked is a boundary keys do not cross
    assert_eq!(
        written("tool.black.z = 1\n# Group: later\ntool.black.a = 2\n"),
        "tool.black.z = 1\n# Group: later\ntool.black.a = 2\n"
    );
}

/// Every rule reads the table it names, so a set-like array sorts wherever the file wrote it.
#[test]
fn test_every_tool_reads_the_table_the_file_wrote() {
    fn held(start: &str) -> String {
        let written = _pyproject_fmt::format_toml(start, &wide_settings()).expect("the formatter accepts it");
        assert_eq!(
            _pyproject_fmt::format_toml(&written, &wide_settings()).expect("the formatter accepts what it wrote"),
            written,
            "the formatter settled on its first pass"
        );
        written
    }

    for name in [
        "tool.towncrier.ignore",
        "tool.cibuildwheel.enable",
        "tool.pdm.plugins",
        "tool.setuptools.platforms",
        "tool.pyright.include",
        "tool.mypy.exclude",
        "tool.hatch.workspace.members",
    ] {
        // the file keeps the shape its author gave it, and the list reads the same way in each
        let (table, key) = name.rsplit_once('.').expect("the name carries its table");
        for start in [
            format!("{name} = [ \"z\", \"a\" ]\n"),
            format!("[{table}]\n{key} = [ \"z\", \"a\" ]\n"),
        ] {
            let written = held(&start);
            assert!(written.contains("[ \"a\", \"z\" ]"), "{name}: {written}");
        }
    }
}

/// A project written only as a child header is the same project, so its extras and requirements are
/// read the way they are under the parent the file left implicit.
#[test]
fn test_a_project_written_only_below_its_own_header_is_still_read() {
    let start = indoc! {r#"
        [project.optional-dependencies]
        My_Extra = ["pytest>=7.0"]
    "#};

    insta::assert_snapshot!(evaluate_project(start, false, (3, 12), false), @r#"
    [project]
    optional-dependencies.my-extra = [ "pytest>=7" ]
    "#);
}

/// A project written only under a table of its own is still a project, and the key that says what
/// it supports goes where the file can hold it rather than into a table that says something else.
#[test]
fn test_classifiers_reach_a_project_written_only_below_its_own_header() {
    let written = |start: &str| {
        let held = evaluate_project(start, false, (3, 12), true);
        assert_eq!(
            evaluate_project(&held, false, (3, 12), true),
            held,
            "the pass settled on its first run"
        );
        held
    };

    insta::assert_snapshot!(written("[project.urls]\nhomepage = \"https://example.invalid\"\n"), @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    urls.homepage = "https://example.invalid"
    "#);
    insta::assert_snapshot!(written("[project.optional-dependencies]\ndocs = [ \"a\" ]\n"), @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    optional-dependencies.docs = [ "a" ]
    "#);
    // a key the file wrote as a comment declares nothing, so nothing is written for it
    insta::assert_snapshot!(written("# project.name = \"x\"\n"), @r##"# project.name = "x""##);

    // written out, no run of the file holds a key of the project, so the key that says what it
    // supports goes before the first header and names the whole path it belongs to
    let expanded = _pyproject_fmt::format_toml(
        "[project.urls]\nhomepage = \"https://example.invalid\"\n",
        &_pyproject_fmt::Settings {
            column_width: 120,
            indent: 2,
            keep_full_version: false,
            max_supported_python: (3, 12),
            min_supported_python: (3, 10),
            generate_python_version_classifiers: true,
            table_format: String::from("long"),
            sub_table_spacing: String::new(),
            separate_root_table: String::from("\n"),
            expand_tables: vec![],
            collapse_tables: vec![],
            skip_wrap_for_keys: vec![],
        },
    )
    .expect("the formatter accepts it");

    insta::assert_snapshot!(expanded, @r#"
    project.classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]

    [project.urls]
    homepage = "https://example.invalid"
    "#);
}

/// The key that says what a project supports goes inside the table the file wrote for it, whichever
/// form that is: a header the file left empty, or a table written as a value.
#[test]
fn test_classifiers_go_inside_the_table_the_file_wrote() {
    let written = |start: &str| {
        let held = evaluate_project(start, false, (3, 12), true);
        assert_eq!(
            evaluate_project(&held, false, (3, 12), true),
            held,
            "the pass settled on its first run"
        );
        held
    };

    insta::assert_snapshot!(written("project = { name = \"x\" }\n"), @r#"
    project = { name = "x", classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ] }
    "#);
    insta::assert_snapshot!(written("project = {}\n"), @r#"
    project = { classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ] }
    "#);
    insta::assert_snapshot!(written("[project]\n"), @r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
    // a project written before the first header keeps its keys there
    insta::assert_snapshot!(written("project.name = \"x\"\nproject.requires-python = \">=3.11\"\n"), @r#"
    project.name = "x"
    project.requires-python = ">=3.11"
    project.classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
    // a value that is not a table is no place to write metadata into
    insta::assert_snapshot!(written("project = 1\n"), @"project = 1");
}

/// A dependency this parser cannot read is left where it is, so a list holding something that is
/// not a requirement at all keeps the order the file gave it.
#[test]
fn test_project_dependencies_holding_something_that_is_not_a_string() {
    let start = indoc! {r#"
        [project]
        name = "test"
        dependencies = ["z", 1, "a"]
        "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    dependencies = [ "z", 1, "a" ]
    "#);
}

/// The classifiers a project gets are written into the table however the file wrote one: as a
/// header, or as a value on a line of its own.
#[test]
fn test_project_classifiers_reach_a_table_written_as_a_value() {
    let start = indoc! {r#"
        project = { name = "test", requires-python = ">=3.12" }
        "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    project = { name = "test", requires-python = ">=3.12", classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ] }
    "#);
}

/// The classifiers go with the keys of the project wherever the file wrote them, and a table below
/// the project holds keys of its own rather than of the project.
#[test]
fn test_project_classifiers_go_with_the_keys_of_the_project_not_a_table_below_it() {
    let start = indoc! {r#"
        project.name = "test"
        project.requires-python = ">=3.12"

        [project.urls]
        homepage = "https://example.com"
        "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    project.name = "test"
    project.requires-python = ">=3.12"
    project.classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]

    [project.urls]
    homepage = "https://example.com"
    "#);
}

fn wide_settings() -> _pyproject_fmt::Settings {
    _pyproject_fmt::Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 12),
        min_supported_python: (3, 10),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        sub_table_spacing: String::new(),
        separate_root_table: String::from("\n"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
    }
}

fn evaluate_project(
    start: &str,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
) -> String {
    run_project_fix(
        start,
        keep_full_version,
        max_supported_python,
        generate_python_version_classifiers,
    )
    .expect("the formatter accepts it")
}

fn evaluate_project_error(start: &str) -> String {
    run_project_fix(start, false, (3, 12), false).expect_err("the formatter rejects it")
}

fn evaluate_config(
    start: &str,
    table_config: TableFormatConfig,
    classifiers: bool,
    max_supported_python: (u8, u8),
) -> String {
    let named = |held: &HashSet<Vec<String>>| held.iter().map(|name| sections::dotted_name(name)).collect();
    evaluate_project_settings(&Settings {
        table_format: String::from(if table_config.default_collapse { "short" } else { "long" }),
        expand_tables: named(&table_config.expand),
        collapse_tables: named(&table_config.collapse),
        generate_python_version_classifiers: classifiers,
        max_supported_python,
        ..default_settings()
    })(start)
}

fn run_project_fix(
    start: &str,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
) -> Result<String, String> {
    let settings = Settings {
        keep_full_version,
        max_supported_python,
        generate_python_version_classifiers,
        ..default_settings()
    };
    let written = format_toml(start, &settings)?;
    super::assert_valid_toml(&written);
    Ok(written)
}

fn evaluate_project_settings(settings: &Settings) -> impl Fn(&str) -> String + '_ {
    move |start| {
        let written = format_toml(start, settings).expect("the formatter accepts it");
        super::assert_valid_toml(&written);
        written
    }
}
