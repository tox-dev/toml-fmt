use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::djlint::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.djlint"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_djlint_order_and_sort() {
    let start = indoc::indoc! {r#"
    [tool.djlint]
    max_line_length = 120
    indent = 2
    ignore = ["H006", "H013", "H005"]
    profile = "django"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.djlint]
    profile = "django"
    indent = 2
    max_line_length = 120
    ignore = [ "H005", "H006", "H013" ]
    "#);
}

#[test]
fn test_djlint_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.djlint]
    profile = "django"
    indent = 2
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}
