use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::bandit::fix;
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.bandit"], 120);
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
fn test_bandit_top_level_order() {
    let start = indoc::indoc! {r#"
    [tool.bandit]
    skips = ["B101"]
    tests = ["B201"]
    targets = ["src"]
    exclude_dirs = ["tests"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    exclude_dirs = [ "tests" ]
    targets = [ "src" ]
    tests = [ "B201" ]
    skips = [ "B101" ]
    "#);
}

#[test]
fn test_bandit_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit]
    skips = ["B311", "B101", "B201"]
    tests = ["B999", "B101"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    tests = [ "B101", "B999" ]
    skips = [ "B101", "B201", "B311" ]
    "#);
}

#[test]
fn test_bandit_assert_used_inner() {
    let start = indoc::indoc! {r#"
    [tool.bandit.assert_used]
    skips = ["*_test.py", "test_*.py"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    assert_used.skips = [ "*_test.py", "test_*.py" ]
    "#);
}

#[test]
fn test_bandit_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.bandit]
    exclude_dirs = [ "tests" ]
    skips = [ "B101", "B201" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_bandit_no_table_noop() {
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
fn test_bandit_inner_tmp_dirs_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit.hardcoded_tmp_directory]
    tmp_dirs = ["/var/tmp", "/tmp", "/dev/shm"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    hardcoded_tmp_directory.tmp_dirs = [ "/dev/shm", "/tmp", "/var/tmp" ]
    "#);
}

#[test]
fn test_bandit_inner_no_shell_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit.any_other_function_with_shell_equals_true]
    no_shell = ["os.system", "os.popen"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    any_other_function_with_shell_equals_true.no_shell = [ "os.popen", "os.system" ]
    "#);
}

#[test]
fn test_bandit_inner_shell_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit.shell_injection]
    shell = ["zsh", "bash", "sh"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    shell_injection.shell = [ "bash", "sh", "zsh" ]
    "#);
}

#[test]
fn test_bandit_inner_subprocess_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit.shell_injection]
    subprocess = ["subprocess.run", "subprocess.Popen"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    shell_injection.subprocess = [ "subprocess.Popen", "subprocess.run" ]
    "#);
}

#[test]
fn test_bandit_inner_tests_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit.some_plugin]
    tests = ["B999", "B101"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    some_plugin.tests = [ "B101", "B999" ]
    "#);
}

#[test]
fn test_bandit_inner_non_matching_preserved() {
    let start = indoc::indoc! {r#"
    [tool.bandit.assert_used]
    word_list = ["zeta", "alpha"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    assert_used.word_list = [ "zeta", "alpha" ]
    "#);
}

#[test]
fn test_bandit_long_format() {
    let start = indoc::indoc! {r#"
    [tool.bandit]
    skips = ["B311", "B101"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.bandit]"));
    assert!(result.find("B101").unwrap() < result.find("B311").unwrap());
}
