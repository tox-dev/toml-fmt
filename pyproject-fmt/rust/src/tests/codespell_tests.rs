use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::codespell::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.codespell"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_codespell_order_and_sorted_arrays() {
    let start = indoc::indoc! {r#"
    [tool.codespell]
    write-changes = true
    skip = ["./vendor", "./build"]
    ignore-words-list = ["fo", "ba"]
    builtin = "clear,rare"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.codespell]
    builtin = "clear,rare"
    ignore-words-list = [ "ba", "fo" ]
    skip = [ "./build", "./vendor" ]
    write-changes = true
    "#);
}

#[test]
fn test_codespell_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.codespell]
    skip = [ "build", "dist" ]
    ignore-words-list = [ "fo", "ba" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}
