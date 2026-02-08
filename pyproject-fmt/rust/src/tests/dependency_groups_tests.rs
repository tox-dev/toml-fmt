use common::array::ensure_all_arrays_multiline;
use common::table::Tables;
use indoc::indoc;

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::dependency_groups::fix;

fn format_dependency_groups_helper(start: &str, keep_full_version: bool) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    fix(&mut tables, keep_full_version);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_format_dependency_groups_no_groups() {
    let start = indoc! {r""};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @"");
}

#[test]
fn test_format_dependency_groups_single_group_single_dep() {
    let start = indoc! {r#"
    [dependency-groups]
    test=["a>1.0.0"]
    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    test = [ "a>1" ]
    "#);
}

#[test]
fn test_format_dependency_groups_single_group_single_dep_full_version() {
    let start = indoc! {r#"
    [dependency-groups]
    test=["a>1.0.0"]
    "#};
    let res = format_dependency_groups_helper(start, true);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    test = [ "a>1.0.0" ]
    "#);
}

#[test]
fn test_format_dependency_groups_single_group_multiple_deps() {
    let start = indoc! {r#"
    [dependency-groups]
    test=["b==2.0.*", "a>1"]
    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    test = [ "a>1", "b==2.0.*" ]
    "#);
}

#[test]
fn test_format_dependency_groups_multiple_groups() {
    let start = indoc! {r#"
    [dependency-groups]
    example=["c<1"]
    docs=["b==1"]
    test=["a>1"]
    dev=["d>=2"]
    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    dev = [ "d>=2" ]
    test = [ "a>1" ]
    docs = [ "b==1" ]
    example = [ "c<1" ]
    "#);
}

#[test]
fn test_format_dependency_groups_multiple_groups_and_extra_line() {
    let start = indoc! {r#"
    [dependency-groups]
    example = [ "c<1" ]
    coverage = ["b<2"]
    type = [ "a>1" ]

    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    type = [ "a>1" ]
    coverage = [ "b<2" ]
    example = [ "c<1" ]
    "#);
}

#[test]
fn test_format_dependency_groups_include_single_group() {
    let start = indoc! {r#"
    [dependency-groups]
    docs=["b==1"]
    test=["a>1",{include-group="docs"}]
    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    test = [ "a>1", { include-group = "docs" } ]
    docs = [ "b==1" ]
    "#);
}

#[test]
fn test_format_dependency_groups_include_many_groups() {
    let start = indoc! {r#"
    [dependency-groups]
    all=['c<1', {include-group='test'}, {include-group='docs'}, 'd>1']
    docs = ['b==1']
    test = ['a>1']
    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    test = [ "a>1" ]
    docs = [ "b==1" ]
    all = [ { include-group = "docs" }, "c<1", { include-group = "test" }, "d>1" ]
    "#);
}

#[test]
fn test_format_dependency_groups_duplicate_package_names() {
    let start = indoc! {r#"
    [dependency-groups]
    test=["pkg>=2.0","pkg>=1.0","other"]
    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    test = [ "other", "pkg>=1", "pkg>=2" ]
    "#);
}

#[test]
fn test_format_dependency_groups_inline_table_without_include_group() {
    let start = indoc! {r#"
    [dependency-groups]
    test=["pkg>=1.0",{some-key="value"}]
    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    test = [ "pkg>=1", { some-key = "value" } ]
    "#);
}

#[test]
fn test_format_dependency_groups_multiple_include_groups_sorted() {
    let start = indoc! {r#"
    [dependency-groups]
    test = ['a>=1']
    docs = ['b>=1']
    dev = ['c>=1']
    all = [
      {include-group='test'},
      {include-group='docs'},
      {include-group='dev'},
    ]
    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    dev = [ "c>=1" ]
    test = [ "a>=1" ]
    docs = [ "b>=1" ]
    all = [
      { include-group = "dev" },
      { include-group = "docs" },
      { include-group = "test" },
    ]
    "#);
}

#[test]
fn test_format_dependency_groups_array_with_comments() {
    let start = indoc! {r#"
    [dependency-groups]
    test = [
      "pytest>=7.0",
      # This is a comment
      "black>=23.0",
      { include-group = "docs" },
      # Another comment
    ]
    docs = ["sphinx>=5.0"]
    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    test = [
      # This is a comment
      "black>=23",
      "pytest>=7",
      { include-group = "docs" },
      # Another comment
    ]
    docs = [ "sphinx>=5" ]
    "#);
}

#[test]
fn test_format_dependency_groups_mixed_packages_and_include_groups() {
    let start = indoc! {r#"
    [dependency-groups]
    base = ['requests>=2']
    extended = [
      {include-group='base'},
      'pytest>=7',
      {include-group='base'},
      'black>=23',
    ]
    "#};
    let res = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [dependency-groups]
    base = [ "requests>=2" ]
    extended = [
      { include-group = "base" },
      "pytest>=7",
      { include-group = "base" },
      "black>=23",
    ]
    "#);
}

#[test]
fn test_dependency_groups_with_integer() {
    let start = indoc! {r#"
        [dependency-groups]
        test = [
          "pkg>=1.0",
          42,
        ]
        "#};
    let result = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    test = [
      "pkg>=1",
      42,
    ]
    "#);
}

#[test]
fn test_dependency_groups_same_package_different_markers() {
    let start = indoc! {r#"
        [dependency-groups]
        test = [
          "pkg>=2.0; python_version>='3.10'",
          "pkg>=1.0; python_version<'3.10'",
        ]
        "#};
    let result = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    test = [
      "pkg>=1; python_version<'3.10'",
      "pkg>=2; python_version>='3.10'",
    ]
    "#);
}

#[test]
fn test_dependency_groups_ordering_dev_test_docs() {
    let start = indoc! {r#"
        [dependency-groups]
        docs = ["sphinx>=5"]
        dev = ["ruff>=0.4"]
        test = ["pytest>=7"]
        "#};
    let result = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    dev = [ "ruff>=0.4" ]
    test = [ "pytest>=7" ]
    docs = [ "sphinx>=5" ]
    "#);
}

#[test]
fn test_dependency_groups_unknown_group_name() {
    let start = indoc! {r#"
        [dependency-groups]
        custom = ["pkg>=1"]
        zebra = ["other>=2"]
        alpha = ["third>=3"]
        "#};
    let result = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    alpha = [ "third>=3" ]
    custom = [ "pkg>=1" ]
    zebra = [ "other>=2" ]
    "#);
}

#[test]
fn test_dependency_groups_inline_table_without_known_key() {
    let start = indoc! {r#"
        [dependency-groups]
        test = [
          "pkg>=1.0",
          {other-key="value"},
        ]
        "#};
    let result = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    test = [
      "pkg>=1",
      { other-key = "value" },
    ]
    "#);
}

#[test]
fn test_dependency_groups_literal_string_package() {
    let start = indoc! {r#"
        [dependency-groups]
        test = [
          'single-quoted>=1.0',
        ]
        "#};
    let result = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    test = [
      "single-quoted>=1",
    ]
    "#);
}

#[test]
fn test_dependency_groups_empty_group() {
    let start = indoc! {r#"
        [dependency-groups]
        empty = []
        test = ["pkg>=1"]
        "#};
    let result = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    test = [ "pkg>=1" ]
    empty = []
    "#);
}

#[test]
fn test_dependency_groups_tertiary_sort_same_package_same_canonical() {
    let start = indoc! {r#"
        [dependency-groups]
        test = [
          "pkg>=2.0",
          "pkg>=1.0",
        ]
        "#};
    let result = format_dependency_groups_helper(start, true);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    test = [
      "pkg>=1.0",
      "pkg>=2.0",
    ]
    "#);
}

#[test]
fn test_dependency_groups_with_nested_inline_table() {
    let start = indoc! {r#"
        [dependency-groups]
        test = [
          "pkg>=1",
          {include-group = "base"},
          {include-group = "docs"},
        ]
        base = ["base-pkg"]
        docs = ["sphinx"]
        "#};
    let result = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    test = [
      "pkg>=1",
      { include-group = "base" },
      { include-group = "docs" },
    ]
    docs = [ "sphinx" ]
    base = [ "base-pkg" ]
    "#);
}

#[test]
fn test_dependency_groups_include_groups_sorted_alphabetically() {
    let start = indoc! {r#"
        [dependency-groups]
        all = [
          {include-group = "zebra"},
          {include-group = "alpha"},
          {include-group = "beta"},
        ]
        alpha = ["a"]
        beta = ["b"]
        zebra = ["z"]
        "#};
    let result = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    all = [
      { include-group = "alpha" },
      { include-group = "beta" },
      { include-group = "zebra" },
    ]
    alpha = [ "a" ]
    beta = [ "b" ]
    zebra = [ "z" ]
    "#);
}

#[test]
fn test_dependency_groups_packages_before_include_groups() {
    let start = indoc! {r#"
        [dependency-groups]
        all = [
          {include-group = "base"},
          "zz-pkg",
          "aa-pkg",
        ]
        base = ["x"]
        "#};
    let result = format_dependency_groups_helper(start, false);
    insta::assert_snapshot!(result, @r#"
    [dependency-groups]
    all = [
      "aa-pkg",
      "zz-pkg",
      { include-group = "base" },
    ]
    base = [ "x" ]
    "#);
}
