use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use insta::assert_snapshot;

use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{collect_entries, format_syntax, parse};
use crate::uv::fix;

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("rust")
        .join("src")
        .join("tests")
        .join("data")
}

fn evaluate(start: &str) -> String {
    evaluate_with_collapse(start, true)
}

fn evaluate_with_collapse(start: &str, collapse: bool) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| collapse, &["tool.uv"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    ensure_all_arrays_multiline(&root_ast, 120);
    format_syntax(root_ast, 120)
}

#[test]
fn test_order_uv() {
    let data = data_dir();
    let start = read_to_string(data.join("uv-order.toml")).unwrap();
    let result = evaluate(&start);
    assert_snapshot!(result);
}

#[test]
fn test_uv_sources_sorting() {
    let start = indoc::indoc! {r#"
    [tool.uv.sources]
    zebra = { git = "https://github.com/example/zebra" }
    alpha = { path = "../alpha" }
    mango = { workspace = true }
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.uv]
    sources.alpha = { path = "../alpha" }
    sources.mango = { workspace = true }
    sources.zebra = { git = "https://github.com/example/zebra" }
    "#);
}

#[test]
fn test_uv_dependency_arrays() {
    let start = indoc::indoc! {r#"
    [tool.uv]
    dev-dependencies = ["pytest", "black", "mypy"]
    constraint-dependencies = ["requests<3", "flask>=2"]
    override-dependencies = ["numpy==1.24", "pandas>=2"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.uv]
    dev-dependencies = [ "black", "mypy", "pytest" ]
    constraint-dependencies = [ "flask>=2", "requests<3" ]
    override-dependencies = [ "numpy==1.24", "pandas>=2" ]
    "#);
}

#[test]
fn test_uv_package_arrays() {
    let start = indoc::indoc! {r#"
    [tool.uv]
    no-binary-package = ["scipy", "numpy", "pillow"]
    no-build-package = ["torch", "tensorflow"]
    reinstall-package = ["mypy", "black"]
    upgrade-package = ["requests", "flask"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.uv]
    no-binary-package = [ "numpy", "pillow", "scipy" ]
    no-build-package = [ "tensorflow", "torch" ]
    reinstall-package = [ "black", "mypy" ]
    upgrade-package = [ "flask", "requests" ]
    "#);
}

#[test]
fn test_uv_workspace_members() {
    let start = indoc::indoc! {r#"
    [tool.uv]
    workspace.members = ["packages/zebra", "packages/alpha", "packages/beta"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.uv]
    workspace.members = [ "packages/alpha", "packages/beta", "packages/zebra" ]
    "#);
}

#[test]
fn test_uv_workspace_exclude() {
    let start = indoc::indoc! {r#"
    [tool.uv]
    workspace.exclude = ["examples/demo", "examples/alpha"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.uv]
    workspace.exclude = [ "examples/alpha", "examples/demo" ]
    "#);
}

#[test]
fn test_uv_preserve_comments() {
    let start = indoc::indoc! {r#"
    [tool.uv]
    dev-dependencies = [
        # Testing tools
        "pytest",
        "coverage",
    ]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.uv]
    dev-dependencies = [
      # Testing tools
      "pytest",
      "coverage",
    ]
    "#);
}

#[test]
fn test_uv_no_uv_section() {
    let start = indoc::indoc! {r#"
    [tool.ruff]
    line-length = 120
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.ruff]
    line-length = 120
    "#);
}

#[test]
fn test_uv_pip_subsection() {
    let start = indoc::indoc! {r#"
    [tool.uv.pip]
    no-binary-package = ["scipy", "numpy"]
    extra = ["dev", "test", "docs"]
    index-url = "https://pypi.org/simple"
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.uv]
    pip.extra = [ "dev", "docs", "test" ]
    pip.index-url = "https://pypi.org/simple"
    pip.no-binary-package = [ "numpy", "scipy" ]
    "#);
}

#[test]
fn test_uv_environments() {
    let start = indoc::indoc! {r#"
    [tool.uv]
    environments = ["sys_platform == 'darwin'", "sys_platform == 'linux'", "python_version >= '3.10'"]
    required-environments = ["sys_platform == 'win32'", "python_version >= '3.8'"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.uv]
    environments = [ "python_version >= '3.10'", "sys_platform == 'darwin'", "sys_platform == 'linux'" ]
    required-environments = [ "python_version >= '3.8'", "sys_platform == 'win32'" ]
    "#);
}

#[test]
fn test_uv_network_arrays() {
    let start = indoc::indoc! {r#"
    [tool.uv]
    allow-insecure-host = ["internal.corp.com", "dev.local", "build.corp.com"]
    no-proxy = ["localhost", "*.internal.com", "127.0.0.1"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.uv]
    allow-insecure-host = [ "build.corp.com", "dev.local", "internal.corp.com" ]
    no-proxy = [ "*.internal.com", "127.0.0.1", "localhost" ]
    "#);
}

#[test]
fn test_uv_pip_table_no_collapse() {
    let start = indoc::indoc! {r#"
    [tool.uv.pip]
    upgrade-package = ["requests", "flask"]
    no-binary-package = ["scipy", "numpy"]
    extra = ["dev", "test", "docs"]
    index-url = "https://pypi.org/simple"
    "#};
    let result = evaluate_with_collapse(start, false);
    assert_snapshot!(result, @r#"
    index-url = "https://pypi.org/simple"no-binary-package = ["numpy","scipy" ]
    extra = ["dev", "docs","test" ]
    [tool.uv.pip]
    upgrade-package = ["flask","requests" ]
    "#);
}

#[test]
fn test_uv_sources_table_no_collapse() {
    let start = indoc::indoc! {r#"
    [tool.uv.sources]
    zebra = { git = "https://github.com/example/zebra" }
    alpha = { path = "../alpha" }
    mango = { workspace = true }
    "#};
    let result = evaluate_with_collapse(start, false);
    assert_snapshot!(result, @r#"
    alpha = { path = "../alpha" }
    mango = { workspace = true }[tool.uv.sources]
    zebra = { git = "https://github.com/example/zebra" }
    "#);
}
