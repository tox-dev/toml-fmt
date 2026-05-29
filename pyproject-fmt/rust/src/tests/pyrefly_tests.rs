use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::pyrefly::fix;
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.pyrefly"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
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
        max_supported_python: (3, 9),
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

fn long_settings() -> Settings {
    Settings {
        table_format: String::from("long"),
        ..default_settings()
    }
}

fn evaluate_long(start: &str) -> String {
    let result = format_toml(start, &long_settings());
    assert_valid_toml(&result);
    result
}

#[test]
fn test_pyrefly_order() {
    let start = indoc::indoc! {r#"
    [tool.pyrefly]
    project_includes = ["src"]
    python_version = "3.12"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyrefly]
    python_version = "3.12"
    project_includes = [ "src" ]
    "#);
}

#[test]
fn test_pyrefly_no_table_noop() {
    let start = indoc::indoc! {r#"
    [project]
    name = "x"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "x"
    "#);
}

#[test]
fn test_pyrefly_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pyrefly]
    project_includes = ["zeta/**", "alpha/**"]
    project_excludes = ["zbuild", "abuild"]
    search_path = ["z_path", "a_path"]
    site_package_path = ["z_site", "a_site"]
    replace_imports_with_any = ["z_pkg", "a_pkg"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyrefly]
    project_includes = [ "alpha/**", "zeta/**" ]
    project_excludes = [ "abuild", "zbuild" ]
    search_path = [ "a_path", "z_path" ]
    site_package_path = [ "a_site", "z_site" ]
    replace_imports_with_any = [ "a_pkg", "z_pkg" ]
    "#);
}

#[test]
fn test_pyrefly_non_sortable_preserved() {
    let start = indoc::indoc! {r#"
    [tool.pyrefly]
    python_interpreter = "/usr/bin/python3"
    use_untyped_imports = true
    python_version = "3.12"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyrefly]
    python_version = "3.12"
    python_interpreter = "/usr/bin/python3"
    use_untyped_imports = true
    "#);
}

#[test]
fn test_pyrefly_long_format() {
    let start = indoc::indoc! {r#"
    [tool.pyrefly]
    project_includes = ["zeta/**", "alpha/**"]
    python_version = "3.12"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.pyrefly]"));
    assert!(result.find("alpha").unwrap() < result.find("zeta").unwrap());
    assert!(result.find("python_version").unwrap() < result.find("project_includes").unwrap());
}
