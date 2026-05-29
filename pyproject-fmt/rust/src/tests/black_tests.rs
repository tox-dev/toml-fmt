use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::black::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.black"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_black_top_level_key_order() {
    let start = indoc::indoc! {r#"
    [tool.black]
    workers = 4
    preview = true
    skip-magic-trailing-comma = false
    skip-string-normalization = false
    extend-exclude = "/tests/"
    target-version = ["py311", "py310"]
    line-length = 88
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.black]
    target-version = [ "py310", "py311" ]
    line-length = 88
    extend-exclude = "/tests/"
    skip-string-normalization = false
    skip-magic-trailing-comma = false
    preview = true
    workers = 4
    "#);
}

#[test]
fn test_black_target_version_sorted() {
    let start = indoc::indoc! {r#"
    [tool.black]
    target-version = ["py313", "py39", "py312", "py310", "py311"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.black]
    target-version = [ "py39", "py310", "py311", "py312", "py313" ]
    "#);
}

#[test]
fn test_black_required_version_first() {
    let start = indoc::indoc! {r#"
    [tool.black]
    line-length = 88
    required-version = "24.10"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.black]
    required-version = "24.10"
    line-length = 88
    "#);
}

#[test]
fn test_black_unknown_keys_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.black]
    zzz_unknown = true
    aaa_unknown = false
    line-length = 88
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.black]
    line-length = 88
    aaa_unknown = false
    zzz_unknown = true
    "#);
}

#[test]
fn test_black_comments_preserved() {
    let start = indoc::indoc! {r#"
    [tool.black]
    # Target older Pythons
    target-version = ["py39"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.black]
    # Target older Pythons
    target-version = [ "py39" ]
    "#);
}

#[test]
fn test_black_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.black]
    target-version = [ "py310", "py311" ]
    line-length = 88
    extend-exclude = "/tests/"
    preview = true
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_black_no_table_is_noop() {
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
