use std::collections::HashSet;

use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};
use indoc::indoc;

use tombi_config::TomlVersion;

use crate::project::fix;
use crate::tests::{assert_valid_toml, collect_entries, format_syntax, format_toml_str, parse};
use crate::TableFormatConfig;

fn evaluate_project(
    start: &str,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    let table_config = TableFormatConfig {
        default_collapse: true,
        expand_tables: HashSet::new(),
        collapse_tables: HashSet::new(),
    };
    apply_table_formatting(
        &mut tables,
        |name| table_config.should_collapse(name),
        &["project"],
        120,
    );
    fix(
        &mut tables,
        keep_full_version,
        max_supported_python,
        (3, 9),
        generate_python_version_classifiers,
        &table_config,
    );

    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

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
    entry-points."console_scripts".mytool = "mypackage:main"
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
    entry-points."console_scripts".mytool = "mypackage:main"
    entry-points."pytest11".myplugin = "mypackage.plugin:pytest_plugin"
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

#[test]
fn test_project_classifiers_with_requires_python() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.9,<3.13"
        classifiers = ["License :: OSI Approved :: MIT License"]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.9,<3.13"
    classifiers = [
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
fn test_project_python_version_3_8() {
    let start = indoc! {r#"
        [project]
        name = "test"
    "#};
    let result = evaluate_project(start, false, (3, 8), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
    ]
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
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);
    let table_config = TableFormatConfig {
        default_collapse: false,
        expand_tables: HashSet::new(),
        collapse_tables: HashSet::new(),
    };
    apply_table_formatting(
        &mut tables,
        |name| table_config.should_collapse(name),
        &["project"],
        120,
    );
    fix(&mut tables, false, (3, 12), (3, 9), true, &table_config);
    let intermediate = root_ast.to_string();
    let result = format_toml_str(&intermediate, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    version = "1.0.0"
    authors = [ { name = "Dev" } ]

    [project.urls]
    Homepage = "https://example.com"
    "#);
}

#[test]
fn test_project_with_collapse_specific_table() {
    let start = indoc! {r#"
        [project]
        name = "test"

        [project.urls]
        Homepage = "https://example.com"
        Repository = "https://github.com/user/repo"
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);
    let mut collapse_tables = HashSet::new();
    collapse_tables.insert("project.urls".to_string());
    let table_config = TableFormatConfig {
        default_collapse: false,
        expand_tables: HashSet::new(),
        collapse_tables,
    };
    apply_table_formatting(
        &mut tables,
        |name| table_config.should_collapse(name),
        &["project"],
        120,
    );
    fix(&mut tables, false, (3, 11), (3, 9), true, &table_config);
    let intermediate = root_ast.to_string();
    let result = format_toml_str(&intermediate, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"

    [project.urls]
    urls.Homepage = "https://example.com"
    urls.Repository = "https://github.com/user/repo"
    "#);
}

#[test]
fn test_project_with_expand_specific_table() {
    let start = indoc! {r#"
        [project]
        name = "test"
        urls.Homepage = "https://example.com"
        urls.Repository = "https://github.com/user/repo"
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);
    let mut expand_tables = HashSet::new();
    expand_tables.insert("project.urls".to_string());
    let table_config = TableFormatConfig {
        default_collapse: true,
        expand_tables,
        collapse_tables: HashSet::new(),
    };
    apply_table_formatting(
        &mut tables,
        |name| table_config.should_collapse(name),
        &["project"],
        120,
    );
    fix(&mut tables, false, (3, 11), (3, 9), true, &table_config);
    let intermediate = root_ast.to_string();
    let result = format_toml_str(&intermediate, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    urls.Homepage = "https://example.com"
    urls.Repository = "https://github.com/user/repo"
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
        license = "mit and apache-2.0 or gpl with classpath-exception"
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    license = "mit AND apache-2.0 OR gpl WITH classpath-exception"
    "#);
}

#[test]
fn test_project_import_names() {
    let start = indoc! {r#"
        [project]
        name = "test"
        import-names = ["zebra;  extra", "alpha ; more"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    import-names = [ "alpha; more", "zebra; extra" ]
    "#);
}

#[test]
fn test_project_import_namespaces() {
    let start = indoc! {r#"
        [project]
        name = "test"
        import-namespaces = ["z.namespace;extra", "a.namespace ;  data"]
    "#};
    let result = evaluate_project(start, false, (3, 12), false);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    import-namespaces = [ "a.namespace; data", "z.namespace; extra" ]
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

#[test]
fn test_project_requires_python_greater_than() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">3.9"
        classifiers = ["License :: OSI Approved :: MIT License"]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">3.9"
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
}

#[test]
fn test_project_requires_python_less_than_or_equal() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.9,<=3.11"
        classifiers = ["License :: OSI Approved :: MIT License"]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.9,<=3.11"
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_requires_python_not_equal() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = ">=3.9,!=3.10"
        classifiers = ["License :: OSI Approved :: MIT License"]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = ">=3.9,!=3.10"
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#);
}

#[test]
fn test_project_requires_python_exact_version() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = "==3.11"
        classifiers = ["License :: OSI Approved :: MIT License"]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = "==3.11"
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.11",
    ]
    "#);
}

#[test]
fn test_project_requires_python_compatible() {
    let start = indoc! {r#"
        [project]
        name = "test"
        requires-python = "~=3.10"
        classifiers = ["License :: OSI Approved :: MIT License"]
    "#};
    let result = evaluate_project(start, false, (3, 12), true);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    requires-python = "~=3.10"
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.10",
    ]
    "#);
}

#[test]
fn test_project_expand_authors_to_array_of_tables() {
    let start = indoc! {r#"
        [project]
        name = "test"
        authors = [{ name = "Alice", email = "alice@example.com" }, { name = "Bob" }]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);
    let mut expand_tables = HashSet::new();
    expand_tables.insert("project.authors".to_string());
    let table_config = TableFormatConfig {
        default_collapse: false,
        expand_tables,
        collapse_tables: HashSet::new(),
    };
    apply_table_formatting(
        &mut tables,
        |name| table_config.should_collapse(name),
        &["project"],
        120,
    );
    fix(&mut tables, false, (3, 12), (3, 9), false, &table_config);
    let intermediate = root_ast.to_string();
    let result = format_toml_str(&intermediate, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    authors = [ { name = "Alice", email = "alice@example.com" }, { name = "Bob" } ]
    "#);
}

#[test]
fn test_project_expand_maintainers_to_array_of_tables() {
    let start = indoc! {r#"
        [project]
        name = "test"
        maintainers = [{ name = "Charlie", email = "charlie@example.com" }]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);
    let mut expand_tables = HashSet::new();
    expand_tables.insert("project.maintainers".to_string());
    let table_config = TableFormatConfig {
        default_collapse: false,
        expand_tables,
        collapse_tables: HashSet::new(),
    };
    apply_table_formatting(
        &mut tables,
        |name| table_config.should_collapse(name),
        &["project"],
        120,
    );
    fix(&mut tables, false, (3, 12), (3, 9), false, &table_config);
    let intermediate = root_ast.to_string();
    let result = format_toml_str(&intermediate, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    maintainers = [ { name = "Charlie", email = "charlie@example.com" } ]
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
      "License :: OSI Approved :: MIT License",  # inline comment
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",  # version comment
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
