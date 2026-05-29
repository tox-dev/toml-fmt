use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::maturin::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.maturin"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_maturin_top_level_order() {
    let start = indoc::indoc! {r#"
    [tool.maturin]
    compatibility = "manylinux2014"
    features = ["pyo3/extension-module"]
    python-source = "python"
    bindings = "pyo3"
    module-name = "my_pkg._lib"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.maturin]
    module-name = "my_pkg._lib"
    bindings = "pyo3"
    python-source = "python"
    features = [ "pyo3/extension-module" ]
    compatibility = "manylinux2014"
    "#);
}

#[test]
fn test_maturin_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.maturin]
    features = ["zebra", "alpha", "mike"]
    include = ["src/**/*.rs", "*.toml"]
    exclude = ["zeta/*", "alpha/*"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.maturin]
    include = [ "*.toml", "src/**/*.rs" ]
    exclude = [ "alpha/*", "zeta/*" ]
    features = [ "alpha", "mike", "zebra" ]
    "#);
}

#[test]
fn test_maturin_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.maturin]
    bindings = "pyo3"
    features = [ "extension-module" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}
