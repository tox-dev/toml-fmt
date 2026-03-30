use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};
use indoc::indoc;
use insta::assert_snapshot;

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::pixi::fix;

fn evaluate_core(start: &str, collapse: bool) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| collapse, &["tool.pixi"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    format_syntax(root_ast, 120)
}

fn evaluate(start: &str) -> String {
    let result = evaluate_core(start, true);
    assert_valid_toml(&result);
    result
}

#[test]
fn test_no_pixi_section() {
    let start = indoc! {r#"
    [tool.ruff]
    line-length = 120
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @"
    [tool.ruff]
    line-length = 120
    ");
}

#[test]
fn test_order_pixi_top_level() {
    let start = indoc! {r#"
    [tool.pixi]
    environments.default = { solve-group = "default" }
    tasks.test = "pytest"
    activation.scripts = ["setup.sh"]
    dependencies.python = ">=3.11"
    pypi-dependencies.requests = ">=2"
    workspace.channels = ["conda-forge"]
    workspace.platforms = ["linux-64"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.pixi]
    workspace.channels = [ "conda-forge" ]
    workspace.platforms = [ "linux-64" ]
    dependencies.python = ">=3.11"
    pypi-dependencies.requests = ">=2"
    activation.scripts = [ "setup.sh" ]
    tasks.test = "pytest"
    environments.default = { solve-group = "default" }
    "#);
}

#[test]
fn test_order_pixi_workspace_collapsed() {
    let start = indoc! {r#"
    [tool.pixi]
    workspace.platforms = ["osx-arm64", "linux-64"]
    workspace.channels = ["conda-forge", "bioconda"]
    workspace.name = "my-project"
    workspace.requires-pixi = ">=0.30"
    workspace.version = "1.0.0"
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.pixi]
    workspace.name = "my-project"
    workspace.version = "1.0.0"
    workspace.channels = [ "bioconda", "conda-forge" ]
    workspace.platforms = [ "linux-64", "osx-arm64" ]
    workspace.requires-pixi = ">=0.30"
    "#);
}

#[test]
fn test_sort_workspace_channels() {
    let start = indoc! {r#"
    [tool.pixi]
    workspace.channels = ["pytorch", "conda-forge", "bioconda"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.pixi]
    workspace.channels = [ "bioconda", "conda-forge", "pytorch" ]
    "#);
}

#[test]
fn test_sort_workspace_platforms() {
    let start = indoc! {r#"
    [tool.pixi]
    workspace.platforms = ["win-64", "linux-64", "osx-arm64", "osx-64"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.pixi]
    workspace.platforms = [ "linux-64", "osx-64", "osx-arm64", "win-64" ]
    "#);
}

#[test]
fn test_sort_workspace_preview() {
    let start = indoc! {r#"
    [tool.pixi]
    workspace.preview = ["pixi-build", "conda-build"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.pixi]
    workspace.preview = [ "conda-build", "pixi-build" ]
    "#);
}

#[test]
fn test_pixi_workspace_expanded_table() {
    let start = indoc! {r#"
    [tool.pixi.workspace]
    platforms = ["osx-arm64", "linux-64"]
    channels = ["conda-forge"]
    name = "my-project"
    documentation = "https://docs.example.com"
    homepage = "https://example.com"
    "#};
    let result = evaluate_core(start, false);
    assert_valid_toml(&result);
    assert_snapshot!(result, @r#"
    [tool.pixi.workspace]
    name = "my-project"
    homepage = "https://example.com"
    documentation = "https://docs.example.com"
    channels = [ "conda-forge" ]
    platforms = [ "linux-64", "osx-arm64" ]
    "#);
}

#[test]
fn test_pixi_preserves_subtables() {
    let start = indoc! {r#"
    [tool.pixi]
    workspace.channels = ["conda-forge"]
    workspace.platforms = ["linux-64"]

    [tool.pixi.dependencies]
    python = ">=3.11"
    numpy = ">=1.24"

    [tool.pixi.tasks]
    test = "pytest"
    lint = "ruff check ."
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.pixi]
    workspace.channels = [ "conda-forge" ]
    workspace.platforms = [ "linux-64" ]
    dependencies.numpy = ">=1.24"
    dependencies.python = ">=3.11"
    tasks.lint = "ruff check ."
    tasks.test = "pytest"
    "#);
}
