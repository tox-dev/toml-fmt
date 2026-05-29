use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::setuptools::{fix, reorder_inline_tables};
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.setuptools", "tool.setuptools_scm"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    reorder_inline_tables(&root_ast);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

fn default_settings() -> Settings {
    Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 13),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        sub_table_spacing: String::new(),
        separate_root_table: String::from("\n"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
    }
}

fn evaluate_full(start: &str) -> String {
    let r = format_toml(start, &default_settings());
    assert_valid_toml(&r);
    r
}

#[test]
fn test_setuptools_top_level_key_order() {
    let start = indoc::indoc! {r#"
    [tool.setuptools]
    zip-safe = false
    license-files = ["LICENSE"]
    platforms = ["any"]
    include-package-data = true
    packages = ["my_pkg"]
    py-modules = ["foo", "bar"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools]
    py-modules = [ "bar", "foo" ]
    packages = [ "my_pkg" ]
    include-package-data = true
    platforms = [ "any" ]
    license-files = [ "LICENSE" ]
    zip-safe = false
    "#);
}

#[test]
fn test_setuptools_sortable_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.setuptools]
    py-modules = ["zebra", "alpha"]
    platforms = ["windows", "linux", "darwin"]
    provides = ["zeta", "alpha"]
    obsoletes = ["legacy_z", "legacy_a"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools]
    py-modules = [ "alpha", "zebra" ]
    platforms = [ "darwin", "linux", "windows" ]
    provides = [ "alpha", "zeta" ]
    obsoletes = [ "legacy_a", "legacy_z" ]
    "#);
}

#[test]
fn test_setuptools_packages_preserve_order() {
    let start = indoc::indoc! {r#"
    [tool.setuptools]
    packages = ["main_pkg", "alpha_pkg", "zebra_pkg"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools]
    packages = [ "main_pkg", "alpha_pkg", "zebra_pkg" ]
    "#);
}

#[test]
fn test_setuptools_packages_find_inner_order() {
    let start = indoc::indoc! {r#"
    [tool.setuptools.packages.find]
    namespaces = true
    exclude = ["tests*", "docs*"]
    include = ["my_pkg*"]
    where = ["src"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools]
    packages.find.where = [ "src" ]
    packages.find.include = [ "my_pkg*" ]
    packages.find.exclude = [ "docs*", "tests*" ]
    packages.find.namespaces = true
    "#);
}

#[test]
fn test_setuptools_package_data_star_first_then_alpha() {
    let start = indoc::indoc! {r#"
    [tool.setuptools.package-data]
    "zebra_pkg" = ["*.txt", "*.md"]
    "alpha_pkg" = ["*.json"]
    "*" = ["*.cfg"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools]
    package-data."*" = [ "*.cfg" ]
    package-data."alpha_pkg" = [ "*.json" ]
    package-data."zebra_pkg" = [ "*.md", "*.txt" ]
    "#);
}

#[test]
fn test_setuptools_exclude_package_data_inner_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.setuptools.exclude-package-data]
    "*" = ["zebra.txt", "alpha.txt"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools]
    exclude-package-data."*" = [ "alpha.txt", "zebra.txt" ]
    "#);
}

#[test]
fn test_setuptools_dynamic_field_order_alpha() {
    let start = indoc::indoc! {r#"
    [tool.setuptools.dynamic]
    version = { attr = "my_pkg.__version__" }
    readme = { file = "README.md" }
    dependencies = { file = "requirements.txt" }
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools]
    dynamic.dependencies = { file = "requirements.txt" }
    dynamic.readme = { file = "README.md" }
    dynamic.version = { attr = "my_pkg.__version__" }
    "#);
}

#[test]
fn test_setuptools_dynamic_readme_inline_key_order() {
    let start = indoc::indoc! {r#"
    [tool.setuptools.dynamic]
    readme = { content-type = "text/markdown", file = "README.md" }
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools]
    dynamic.readme = { file = "README.md", content-type = "text/markdown" }
    "#);
}

#[test]
fn test_setuptools_scm_top_level_key_order() {
    let start = indoc::indoc! {r#"
    [tool.setuptools_scm]
    fallback_version = "0.0.0"
    root = ".."
    local_scheme = "no-local-version"
    version_scheme = "release-branch-semver"
    version_file = "src/my_pkg/_version.py"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools_scm]
    version_file = "src/my_pkg/_version.py"
    version_scheme = "release-branch-semver"
    local_scheme = "no-local-version"
    root = ".."
    fallback_version = "0.0.0"
    "#);
}

#[test]
fn test_setuptools_scm_deprecated_keys_last() {
    let start = indoc::indoc! {r#"
    [tool.setuptools_scm]
    write_to = "src/_version.py"
    version_scheme = "guess-next-dev"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools_scm]
    version_scheme = "guess-next-dev"
    write_to = "src/_version.py"
    "#);
}

#[test]
fn test_setuptools_scm_git_sub_table() {
    let start = indoc::indoc! {r#"
    [tool.setuptools_scm.scm.git]
    describe_command = "git describe --dirty --tags --long --match *.*.*"
    pre_parse = "warn_on_shallow"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools_scm]
    scm.git.pre_parse = "warn_on_shallow"
    scm.git.describe_command = "git describe --dirty --tags --long --match *.*.*"
    "#);
}

#[test]
fn test_setuptools_unknown_keys_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.setuptools]
    zzz_unknown = true
    aaa_unknown = false
    py-modules = ["foo"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools]
    py-modules = [ "foo" ]
    aaa_unknown = false
    zzz_unknown = true
    "#);
}

#[test]
fn test_setuptools_comments_preserved() {
    let start = indoc::indoc! {r#"
    [tool.setuptools]
    # Top-level module list
    py-modules = ["foo"]
    # Include everything
    include-package-data = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.setuptools]
    # Top-level module list
    py-modules = [ "foo" ]
    # Include everything
    include-package-data = true
    "#);
}

#[test]
fn test_setuptools_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.setuptools]
    py-modules = [ "alpha", "beta" ]
    include-package-data = true

    [tool.setuptools.packages.find]
    where = [ "src" ]
    include = [ "pkg_*" ]

    [tool.setuptools.dynamic]
    version = { attr = "pkg.__version__" }
    readme = { file = "README.md", content-type = "text/markdown" }

    [tool.setuptools_scm]
    version_file = "src/pkg/_version.py"
    "#};
    let once = evaluate_full(start);
    let twice = evaluate_full(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_setuptools_no_table_is_noop() {
    let start = indoc::indoc! {r#"
    [project]
    name = "demo"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "demo"
    "#);
}

fn long_settings() -> Settings {
    Settings {
        table_format: String::from("long"),
        ..default_settings()
    }
}

fn evaluate_long(start: &str) -> String {
    let r = format_toml(start, &long_settings());
    assert_valid_toml(&r);
    r
}

#[test]
fn test_setuptools_long_format_packages_find() {
    let start = indoc::indoc! {r#"
    [tool.setuptools.packages.find]
    namespaces = true
    exclude = ["tests*"]
    include = ["my_pkg*"]
    where = ["src"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.setuptools.packages.find]"));
    let w = result.find("where = ").expect("where");
    let i = result.find("include = ").expect("include");
    let e = result.find("exclude = ").expect("exclude");
    let n = result.find("namespaces = ").expect("namespaces");
    assert!(w < i && i < e && e < n, "key order wrong:\n{result}");
}

#[test]
fn test_setuptools_long_format_dynamic_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.setuptools.dynamic]
    version = { attr = "pkg.__version__" }
    dependencies = { file = "requirements.txt" }
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.setuptools.dynamic]"));
    let d = result.find("dependencies = ").expect("dependencies");
    let v = result.find("version = ").expect("version");
    assert!(d < v, "dynamic fields not alphabetized:\n{result}");
}

#[test]
fn test_setuptools_long_format_package_data_star_first() {
    let start = indoc::indoc! {r#"
    [tool.setuptools.package-data]
    "zebra" = ["*.txt"]
    "alpha" = ["*.json"]
    "*" = ["*.cfg"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.setuptools.package-data]"));
    let star = result.find("\"*\" = ").expect("star");
    let alpha = result.find("alpha = ").expect("alpha");
    let zebra = result.find("zebra = ").expect("zebra");
    assert!(star < alpha && alpha < zebra, "package-data key order wrong:\n{result}");
}

#[test]
fn test_setuptools_long_format_cmdclass_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.setuptools.cmdclass]
    zeta_cmd = "z:Z"
    alpha_cmd = "a:A"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.setuptools.cmdclass]"));
    let a = result.find("alpha_cmd = ").expect("alpha_cmd");
    let z = result.find("zeta_cmd = ").expect("zeta_cmd");
    assert!(a < z, "cmdclass not alphabetized:\n{result}");
}
