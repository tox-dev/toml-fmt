use common::array::ensure_all_arrays_multiline;
use common::table::Tables;
use indoc::indoc;

use super::{collect_entries, format_syntax, parse};
use crate::build_system::fix;

fn format_build_systems_helper(start: &str, keep_full_version: bool) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let tables = Tables::from_ast(&root_ast);
    fix(&tables, keep_full_version);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    format_syntax(root_ast, 120)
}

#[test]
fn test_format_build_systems_no_build_system() {
    let start = indoc! {r""};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @"
");
}

#[test]
fn test_format_build_systems_build_system_requires_no_keep() {
    let start = indoc! {r#"
    [build-system]
    requires=["a>=1.0.0", "b.c>=1.5.0"]
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    requires = [ "a>=1", "b-c>=1.5" ]
    "#);
}

#[test]
fn test_format_build_systems_build_system_requires_keep() {
    let start = indoc! {r#"
    [build-system]
    requires=["a>=1.0.0", "b.c>=1.5.0"]
    "#};
    let res = format_build_systems_helper(start, true);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    requires = [ "a>=1.0.0", "b-c>=1.5.0" ]
    "#);
}

#[test]
fn test_format_build_systems_join() {
    let start = indoc! {r#"
    [build-system]
    requires=["a"]
    [build-system]
    build-backend = "hatchling.build"
    [[build-system.a]]
    name = "Hammer"
    [[build-system.a]]  # empty table within the array
    [[build-system.a]]
    name = "Nail"
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = "hatchling.build"[build-system]
    requires=["a"]
    [[build-system.a]]
    name = "Hammer"[[build-system.a]]  # empty table within the array
    [[build-system.a]]
    name = "Nail"
    "#);
}

#[test]
fn test_format_build_systems_issue_2_python_version_marker() {
    let start = indoc! {r#"
    [build-system]
    requires = [
      "cython==3.0.11",
      "numpy==1.22.2; python_version<'3.9'",
      "numpy>=2; python_version>='3.9'",
      "setuptools",
    ]
    "#};
    let res = format_build_systems_helper(start, true);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    requires = [
      "cython==3.0.11",
      "numpy==1.22.2; python_version<'3.9'",
      "numpy>=2; python_version>='3.9'",
      "setuptools",
    ]
    "#);
}

#[test]
fn test_format_build_systems_backend_path_sorting() {
    let start = indoc! {r#"
    [build-system]
    build-backend = "backend"
    backend-path = ["src", "lib", "another"]
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = "backend"
    backend-path = [ "another", "lib", "src" ]
    "#);
}
