use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use indoc::indoc;
use rstest::rstest;

use crate::global::reorder_tables;
use common::table::Tables;

#[rstest]
#[case::reorder_no_env_list(
    // Without env_list, env tables are sorted alphabetically (docs before type)
    indoc ! {r#"
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

    "#},
    indoc ! {r#"
        # comment
        requires = ["tox>=4.22"]

        [env_run_base]
        description = "base"

        [env.docs]
        description = "docs"

        [env.type]
        description = "type"

        [demo]
        desc = "demo"
    "#},
)]
#[case::reorder_with_env_list(
    // With env_list, env tables follow env_list order
    indoc ! {r#"
        env_list = ["docs", "type", "lint"]

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"

        [env.lint]
        description = "lint"

    "#},
    indoc ! {r#"
        env_list = ["docs", "type", "lint"]

        [env.docs]
        description = "docs"

        [env.type]
        description = "type"

        [env.lint]
        description = "lint"
    "#},
)]
#[case::reorder_env_list_partial(
    // env tables in env_list come first in env_list order,
    // then any not in env_list preserve file order
    indoc ! {r#"
        env_list = ["type"]

        [env.lint]
        description = "lint"

        [env.docs]
        description = "docs"

        [env.type]
        description = "type"

    "#},
    indoc ! {r#"
        env_list = ["type"]

        [env.type]
        description = "type"

        [env.lint]
        description = "lint"

        [env.docs]
        description = "docs"
    "#},
)]
fn test_reorder_table(#[case] start: &str, #[case] expected: &str) {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let opt = Options {
        column_width: 120,
        ..Options::default()
    };
    let got = format_syntax(root_ast, opt);
    assert_eq!(got, expected);
}

#[test]
fn test_reorder_no_root_table() {
    // File with only sections, no root entries - should still work
    let start = indoc! {r#"
        [env.test]
        description = "test"
    "#};
    let expected = indoc! {r#"
        [env.test]
        description = "test"
    "#};
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let opt = Options {
        column_width: 120,
        ..Options::default()
    };
    let got = format_syntax(root_ast, opt);
    assert_eq!(got, expected);
}

#[test]
fn test_reorder_root_table_no_env_list_key() {
    // Root table exists but has other keys, not env_list
    let start = indoc! {r#"
        requires = ["tox>=4"]

        [env.test]
        description = "test"
    "#};
    let expected = indoc! {r#"
        requires = ["tox>=4"]

        [env.test]
        description = "test"
    "#};
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let opt = Options {
        column_width: 120,
        ..Options::default()
    };
    let got = format_syntax(root_ast, opt);
    assert_eq!(got, expected);
}

#[test]
fn test_reorder_env_list_not_array() {
    // env_list is a string instead of an array - should be ignored (treated as no env_list)
    let start = indoc! {r#"
        env_list = "test"

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"
    "#};
    let expected = indoc! {r#"
        env_list = "test"

        [env.docs]
        description = "docs"
        [env.type]
        description = "type"
    "#};
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let opt = Options {
        column_width: 120,
        ..Options::default()
    };
    let got = format_syntax(root_ast, opt);
    assert_eq!(got, expected);
}

#[test]
fn test_reorder_empty_env_list() {
    // Empty env_list - should behave like no env_list (alphabetical order)
    let start = indoc! {r#"
        env_list = []

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"
    "#};
    let expected = indoc! {r#"
        env_list = []

        [env.docs]
        description = "docs"
        [env.type]
        description = "type"
    "#};
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let opt = Options {
        column_width: 120,
        ..Options::default()
    };
    let got = format_syntax(root_ast, opt);
    assert_eq!(got, expected);
}

#[test]
fn test_reorder_env_list_with_env_run_base() {
    // env_list with env_run_base - env_run_base should come before env.* tables
    let start = indoc! {r#"
        env_list = ["test"]

        [env.test]
        description = "test"

        [env_run_base]
        commands = ["pytest"]
    "#};
    let expected = indoc! {r#"
        env_list = ["test"]

        [env_run_base]
        commands = ["pytest"]

        [env.test]
        description = "test"
    "#};
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let opt = Options {
        column_width: 120,
        ..Options::default()
    };
    let got = format_syntax(root_ast, opt);
    assert_eq!(got, expected);
}
