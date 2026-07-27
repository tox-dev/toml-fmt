use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::pyproject_fmt::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.pyproject-fmt"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_pyproject_fmt_reorders_keys() {
    let start = indoc::indoc! {r#"
    [tool.pyproject-fmt]
    skip_wrap_for_keys = ["a"]
    keep_full_version = true
    table_format = "short"
    indent = 4
    column_width = 120
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyproject-fmt]
    column_width = 120
    indent = 4
    keep_full_version = true
    table_format = "short"
    skip_wrap_for_keys = [ "a" ]
    "#);
}

#[test]
fn test_pyproject_fmt_unknown_keys_last() {
    let start = indoc::indoc! {r#"
    [tool.pyproject-fmt]
    zzz = 1
    indent = 2
    aaa = 2
    column_width = 120
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyproject-fmt]
    column_width = 120
    indent = 2
    aaa = 2
    zzz = 1
    "#);
}

#[test]
fn test_pyproject_fmt_sorts_and_dedupes_arrays() {
    let start = indoc::indoc! {r#"
    [tool.pyproject-fmt]
    expand_tables = ["tool.ruff", "tool.black", "tool.ruff"]
    collapse_tables = ["b", "a"]
    skip_wrap_for_keys = ["z.parse", "a.parse", "z.parse"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyproject-fmt]
    expand_tables = [ "tool.black", "tool.ruff" ]
    collapse_tables = [ "a", "b" ]
    skip_wrap_for_keys = [ "a.parse", "z.parse" ]
    "#);
}

#[test]
fn test_pyproject_fmt_dedup_case_sensitive() {
    let start = indoc::indoc! {r#"
    [tool.pyproject-fmt]
    expand_tables = ["tool.Ruff", "tool.ruff"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyproject-fmt]
    expand_tables = [ "tool.Ruff", "tool.ruff" ]
    "#);
}

#[test]
fn test_pyproject_fmt_preserves_escape_strings() {
    let start = indoc::indoc! {r#"
    [tool.pyproject-fmt]
    separate_root_table = "\n"
    sub_table_spacing = ""
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyproject-fmt]
    sub_table_spacing = ""
    separate_root_table = "\n"
    "#);
}

#[test]
fn test_pyproject_fmt_absent_table_noop() {
    let start = indoc::indoc! {r#"
    [project]
    name = "foo"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "foo"
    "#);
}
