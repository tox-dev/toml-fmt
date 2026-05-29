use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::commitizen::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.commitizen"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_commitizen_order() {
    let start = indoc::indoc! {r#"
    [tool.commitizen]
    update_changelog_on_bump = true
    version_files = ["src/pkg/__init__.py", "pyproject.toml"]
    tag_format = "v$version"
    version = "1.0.0"
    name = "cz_conventional_commits"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.commitizen]
    name = "cz_conventional_commits"
    version = "1.0.0"
    version_files = [ "pyproject.toml", "src/pkg/__init__.py" ]
    tag_format = "v$version"
    update_changelog_on_bump = true
    "#);
}

#[test]
fn test_commitizen_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.commitizen]
    name = "cz_conventional_commits"
    version = "1.0.0"
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}
