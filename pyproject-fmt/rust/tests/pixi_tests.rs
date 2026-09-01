use super::{evaluate_full as evaluate, evaluate_long};
use indoc::indoc;
use insta::assert_snapshot;

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
    workspace.channels = [ "conda-forge", "bioconda" ]
    workspace.platforms = [ "linux-64", "osx-arm64" ]
    workspace.requires-pixi = ">=0.30"
    "#);
}

/// A channel earlier in the list wins the packages it holds, so the order is what the workspace
/// says rather than how it reads.
#[test]
fn test_workspace_channels_keep_their_order() {
    let start = indoc! {r#"
    [tool.pixi]
    workspace.channels = ["pytorch", "conda-forge", "bioconda"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.pixi]
    workspace.channels = [ "pytorch", "conda-forge", "bioconda" ]
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
    let result = evaluate_long(start);
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

/// A rich platform is written as a table, which names no platform this can sort by, so the list it
/// sits in keeps the order the workspace declared: pixi runs the first entry a host satisfies.
#[test]
fn test_pixi_platforms_holding_a_table_keep_their_order() {
    let start = indoc! {r#"
    [tool.pixi]
    workspace.platforms = [
      { name = "cuda-12", platform = "linux-64", cuda = "12" },
      "win-64",
      { name = "cuda-13", platform = "linux-64", cuda = "13" },
      "linux-64",
    ]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.pixi]
    workspace.platforms = [
      { name = "cuda-12", platform = "linux-64", cuda = "12" },
      "win-64",
      { name = "cuda-13", platform = "linux-64", cuda = "13" },
      "linux-64",
    ]
    "#);
}
