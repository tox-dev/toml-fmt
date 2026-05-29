use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::tox::fix;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.tox"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_tox_root_order() {
    let start = indoc::indoc! {r#"
    [tool.tox]
    skip_missing_interpreters = true
    env_list = ["py312", "py311"]
    min_version = "4.0"
    requires = ["tox-uv"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.tox]
    min_version = "4.0"
    requires = [ "tox-uv" ]
    env_list = [ "py312", "py311" ]
    skip_missing_interpreters = true
    "#);
}

#[test]
fn test_tox_env_run_base_inner_order() {
    let start = indoc::indoc! {r#"
    [tool.tox.env_run_base]
    commands = [["pytest"]]
    extras = ["test", "all"]
    deps = ["pytest>=7", "coverage"]
    runner = "uv-venv-lock-runner"
    package = "wheel"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.tox]
    env_run_base.commands = [ [ "pytest" ] ]
    env_run_base.deps = [ "pytest>=7", "coverage" ]
    env_run_base.extras = [ "test", "all" ]
    env_run_base.package = "wheel"
    env_run_base.runner = "uv-venv-lock-runner"
    "#);
}

#[test]
fn test_tox_per_env_order() {
    let start = indoc::indoc! {r#"
    [tool.tox.env.lint]
    commands = [["ruff", "check"]]
    deps = ["ruff"]
    runner = "uv-venv-lock-runner"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.tox]
    env.lint.commands = [ [ "ruff", "check" ] ]
    env.lint.deps = [ "ruff" ]
    env.lint.runner = "uv-venv-lock-runner"
    "#);
}

#[test]
fn test_tox_env_list_sorted() {
    let start = indoc::indoc! {r#"
    [tool.tox]
    env_list = ["py312", "py310", "py311", "py313"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.tox]
    env_list = [ "py313", "py312", "py311", "py310" ]
    "#);
}

#[test]
fn test_tox_deps_sorted_per_env() {
    let start = indoc::indoc! {r#"
    [tool.tox.env.test]
    deps = ["zebra", "alpha", "mike"]
    pass_env = ["HOME", "PATH"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.tox]
    env.test.deps = [ "zebra", "alpha", "mike" ]
    env.test.pass_env = [ "HOME", "PATH" ]
    "#);
}

#[test]
fn test_tox_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.tox]
    requires = [ "tox-uv" ]
    env_list = [ "py311", "py312" ]

    [tool.tox.env_run_base]
    runner = "uv-venv-lock-runner"
    deps = [ "coverage", "pytest" ]
    commands = [ [ "pytest" ] ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_tox_no_table_is_noop() {
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
