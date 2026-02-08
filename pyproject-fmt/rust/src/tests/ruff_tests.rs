use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use insta::assert_snapshot;

use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{collect_entries, format_syntax, parse};
use crate::ruff::fix;

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("rust")
        .join("src")
        .join("tests")
        .join("data")
}

use super::assert_valid_toml;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.ruff"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_order_ruff() {
    let data = data_dir();
    let start = read_to_string(data.join("ruff-order.toml")).unwrap();
    let result = evaluate(&start);
    assert_snapshot!(result);
}

#[test]
fn test_ruff_comment_21() {
    let start = indoc::indoc! {r#"
    [tool.ruff.lint]
    select = ["ALL"]

    ignore = [
        # Missing type annotation for **{name}.
        "ANN003",
    ]

    # Do not automatically remove commented out code.
    # We comment out code during development, and with VSCode auto-save, this code
    # is sometimes annoyingly removed.
    unfixable = ["ERA001"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.ruff]
    lint.select = [ "ALL" ]
    lint.ignore = [
      # Missing type annotation for **{name}.
      "ANN003",
    ]
    # Do not automatically remove commented out code.
    # We comment out code during development, and with VSCode auto-save, this code
    # is sometimes annoyingly removed.
    lint.unfixable = [ "ERA001" ]
    "#);
}

#[test]
fn test_ruff_inline_comments_preserved() {
    let start = indoc::indoc! {r#"
    [tool.ruff.lint]
    ignore = [
      "COM812",  # Conflict with formatter
      "CPY",  # No copyright statements
      "D203",  # Blank line before class
    ]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.ruff]
    lint.ignore = [
      "COM812",  # Conflict with formatter
      "CPY",  # No copyright statements
      "D203",  # Blank line before class
    ]
    "#);
}

#[test]
fn test_ruff_per_file_ignores() {
    let start = indoc::indoc! {r#"
    [tool.ruff]
    lint.per-file-ignores."tests/**/*.py" = ["S101", "D103", "ARG001"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.ruff]
    lint.per-file-ignores."tests/**/*.py" = [ "ARG001", "D103", "S101" ]
    "#);
}

#[test]
fn test_ruff_extend_per_file_ignores() {
    let start = indoc::indoc! {r#"
    [tool.ruff]
    lint.extend-per-file-ignores."docs/*.py" = ["E501", "D100"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.ruff]
    lint.extend-per-file-ignores."docs/*.py" = [ "D100", "E501" ]
    "#);
}
