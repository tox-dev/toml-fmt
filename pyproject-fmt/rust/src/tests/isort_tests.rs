use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::isort::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.isort"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_isort_profile_first() {
    let start = indoc::indoc! {r#"
    [tool.isort]
    line_length = 100
    multi_line_output = 3
    profile = "black"
    include_trailing_comma = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.isort]
    profile = "black"
    line_length = 100
    multi_line_output = 3
    include_trailing_comma = true
    "#);
}

#[test]
fn test_isort_known_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.isort]
    known_first_party = ["my_pkg", "another_pkg", "alpha"]
    known_third_party = ["zebra_lib", "alpha_lib"]
    known_standard_library = ["typing", "asyncio"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.isort]
    known_standard_library = [ "asyncio", "typing" ]
    known_third_party = [ "alpha_lib", "zebra_lib" ]
    known_first_party = [ "alpha", "another_pkg", "my_pkg" ]
    "#);
}

#[test]
fn test_isort_skip_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.isort]
    skip = ["zebra.py", "alpha.py"]
    skip_glob = ["**/migrations/*", "**/vendor/*"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.isort]
    skip = [ "alpha.py", "zebra.py" ]
    skip_glob = [ "**/migrations/*", "**/vendor/*" ]
    "#);
}

#[test]
fn test_isort_sections_preserve_order() {
    let start = indoc::indoc! {r#"
    [tool.isort]
    sections = ["FUTURE", "STDLIB", "THIRDPARTY", "FIRSTPARTY", "LOCALFOLDER"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.isort]
    sections = [ "FUTURE", "STDLIB", "THIRDPARTY", "FIRSTPARTY", "LOCALFOLDER" ]
    "#);
}

#[test]
fn test_isort_add_imports_preserve_order() {
    let start = indoc::indoc! {r#"
    [tool.isort]
    add_imports = ["from __future__ import annotations", "import sys"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.isort]
    add_imports = [ "from __future__ import annotations", "import sys" ]
    "#);
}

#[test]
fn test_isort_section_headings_after_known_sources() {
    let start = indoc::indoc! {r#"
    [tool.isort]
    import_heading_stdlib = "Standard library"
    profile = "black"
    known_first_party = ["my_pkg"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.isort]
    profile = "black"
    known_first_party = [ "my_pkg" ]
    import_heading_stdlib = "Standard library"
    "#);
}

#[test]
fn test_isort_unknown_keys_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.isort]
    zzz_unknown = true
    aaa_unknown = false
    profile = "black"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.isort]
    profile = "black"
    aaa_unknown = false
    zzz_unknown = true
    "#);
}

#[test]
fn test_isort_comments_preserved() {
    let start = indoc::indoc! {r#"
    [tool.isort]
    # Match black
    profile = "black"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.isort]
    # Match black
    profile = "black"
    "#);
}

#[test]
fn test_isort_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.isort]
    profile = "black"
    line_length = 88
    known_first_party = [ "my_pkg", "another_pkg" ]
    skip = [ "build", "dist" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_isort_no_table_is_noop() {
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
