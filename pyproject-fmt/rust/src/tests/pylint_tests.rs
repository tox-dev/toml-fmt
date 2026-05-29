use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::pylint::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.pylint"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_pylint_main_before_messages_control() {
    let start = indoc::indoc! {r#"
    [tool.pylint.format]
    max-line-length = 120

    [tool.pylint.messages_control]
    disable = ["C0114", "C0115"]

    [tool.pylint.main]
    jobs = 4
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pylint]
    main.jobs = 4
    messages_control.disable = [ "C0114", "C0115" ]
    format.max-line-length = 120
    "#);
}

#[test]
fn test_pylint_disable_enable_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pylint.messages_control]
    disable = ["W0621", "C0114", "R0903", "C0115"]
    enable = ["W0612", "C0103"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pylint]
    messages_control.disable = [ "C0114", "C0115", "R0903", "W0621" ]
    messages_control.enable = [ "C0103", "W0612" ]
    "#);
}

#[test]
fn test_pylint_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.pylint.main]
    jobs = 4
    [tool.pylint.messages_control]
    disable = [ "C0114", "C0115" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}
