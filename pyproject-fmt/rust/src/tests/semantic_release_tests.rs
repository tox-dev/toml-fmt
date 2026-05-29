use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::semantic_release::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.semantic_release"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_semantic_release_order() {
    let start = indoc::indoc! {r#"
    [tool.semantic_release]
    version_toml = ["pyproject.toml:project.version"]
    assets = ["zebra.txt", "alpha.txt"]
    tag_format = "v{version}"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.semantic_release]
    tag_format = "v{version}"
    version_toml = [ "pyproject.toml:project.version" ]
    assets = [ "alpha.txt", "zebra.txt" ]
    "#);
}
