use indoc::indoc;
use insta::assert_snapshot;

use super::{assert_valid_toml, format_syntax, parse};
use crate::global::reorder_tables;
use common::table::Tables;

fn reorder_table_helper(start: &str) -> String {
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_reorder_table_reorder_no_env_list() {
    let start = indoc! {r#"
        # comment
        requires = ["tox>=4.22"]

        [demo]
        desc = "demo"

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"

        [env_run_base]
        description = "base"

    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    # comment
    requires = [ "tox>=4.22" ]

    [env_run_base]
    description = "base"

    [env.docs]
    description = "docs"

    [env.type]
    description = "type"

    [demo]
    desc = "demo"
    "#);
}

#[test]
fn test_reorder_table_reorder_with_env_list() {
    let start = indoc! {r#"
        env_list = ["docs", "type", "lint"]

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"

        [env.lint]
        description = "lint"

    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    env_list = [ "docs", "type", "lint" ]

    [env.docs]
    description = "docs"

    [env.type]
    description = "type"

    [env.lint]
    description = "lint"
    "#);
}

#[test]
fn test_reorder_table_reorder_env_list_partial() {
    let start = indoc! {r#"
        env_list = ["type"]

        [env.lint]
        description = "lint"

        [env.docs]
        description = "docs"

        [env.type]
        description = "type"

    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    env_list = [ "type" ]

    [env.type]
    description = "type"

    [env.lint]
    description = "lint"

    [env.docs]
    description = "docs"
    "#);
}

#[test]
fn test_reorder_no_root_table() {
    let start = indoc! {r#"
        [env.test]
        description = "test"
    "#};
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let got = format_syntax(root_ast, 120);
    assert_snapshot!(got, @r#"
    [env.test]
    description = "test"
    "#);
}

#[test]
fn test_reorder_root_table_no_env_list_key() {
    let start = indoc! {r#"
        requires = ["tox>=4"]

        [env.test]
        description = "test"
    "#};
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let got = format_syntax(root_ast, 120);
    assert_snapshot!(got, @r#"
    requires = [ "tox>=4" ]

    [env.test]
    description = "test"
    "#);
}

#[test]
fn test_reorder_env_list_not_array() {
    let start = indoc! {r#"
        env_list = "test"

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"
    "#};
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let got = format_syntax(root_ast, 120);
    assert_snapshot!(got, @r#"
    env_list = "test"

    [env.docs]
    description = "docs"

    [env.type]
    description = "type"
    "#);
}

#[test]
fn test_reorder_empty_env_list() {
    let start = indoc! {r#"
        env_list = []

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"
    "#};
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let got = format_syntax(root_ast, 120);
    assert_snapshot!(got, @r#"
    env_list = []

    [env.docs]
    description = "docs"

    [env.type]
    description = "type"
    "#);
}

#[test]
fn test_reorder_env_list_with_env_run_base() {
    let start = indoc! {r#"
        env_list = ["test"]

        [env.test]
        description = "test"

        [env_run_base]
        commands = ["pytest"]
    "#};
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let got = format_syntax(root_ast, 120);
    assert_snapshot!(got, @r#"
    env_list = [ "test" ]

    [env_run_base]
    commands = [ "pytest" ]

    [env.test]
    description = "test"
    "#);
}
