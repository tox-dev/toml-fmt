use super::{evaluate_full as evaluate, evaluate_long};

#[test]
fn test_cibw_selection_first() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    test-command = "pytest {project}/tests"
    archs = ["x86_64", "arm64"]
    skip = ["pp*"]
    build = "cp3{10,11,12}-*"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    build = "cp3{10,11,12}-*"
    skip = [ "pp*" ]
    archs = [ "x86_64", "arm64" ]
    test-command = "pytest {project}/tests"
    "#);
}

#[test]
fn test_cibw_enable_sorted() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    enable = ["pypy", "cpython-prerelease", "cpython-freethreading"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    enable = [ "cpython-freethreading", "cpython-prerelease", "pypy" ]
    "#);
}

#[test]
fn test_cibw_addopts_preserve_order() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    before-all = "bash setup.sh"
    test-requires = ["pytest", "pytest-cov"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    before-all = "bash setup.sh"
    test-requires = [ "pytest", "pytest-cov" ]
    "#);
}

#[test]
fn test_cibw_per_platform_collapsed() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel.linux]
    before-all = "yum install -y openssl"
    archs = ["x86_64"]

    [tool.cibuildwheel.macos]
    archs = ["x86_64", "arm64"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    linux.archs = [ "x86_64" ]
    linux.before-all = "yum install -y openssl"
    macos.archs = [ "x86_64", "arm64" ]
    "#);
}

#[test]
fn test_cibw_overrides_aot_select_first() {
    let start = indoc::indoc! {r#"
    [[tool.cibuildwheel.overrides]]
    test-command = "pytest {project}/tests/cpython"
    select = "cp3{10,11}-*"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    overrides = [ { select = "cp3{10,11}-*", test-command = "pytest {project}/tests/cpython" } ]
    "#);
}

#[test]
fn test_cibw_overrides_inline_select_first() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    overrides = [
      { test-extras = ["z", "a"], test-command = "pytest", select = "cp310-*" },
      { before-all = "make", select = "cp311-*" },
    ]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    overrides = [
      { select = "cp310-*", test-command = "pytest", test-extras = [ "a", "z" ] },
      { select = "cp311-*", before-all = "make" },
    ]
    "#);
}

#[test]
fn test_cibw_overrides_inline_scalar_sort_key_kept() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    overrides = [ { test-extras = "test", select = "cp310-*" } ]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    overrides = [ { select = "cp310-*", test-extras = "test" } ]
    "#);
}

#[test]
fn test_cibw_overrides_inline_without_select_untouched() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    overrides = [ { test-command = "pytest", before-all = "make" } ]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    overrides = [ { test-command = "pytest", before-all = "make" } ]
    "#);
}

#[test]
fn test_cibw_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    build = "cp3{10,11,12}-*"
    skip = [ "pp*" ]
    archs = [ "x86_64" ]
    enable = [ "cpython-freethreading", "pypy" ]
    test-command = "pytest"
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_cibw_no_table_noop() {
    let start = indoc::indoc! {r#"
    [project]
    name = "x"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "x"
    "#);
}

#[test]
fn test_cibw_long_format_per_platform_tables() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel.linux]
    before-all = "yum install -y openssl"
    enable = ["pypy", "cpython-freethreading"]
    archs = ["x86_64"]

    [tool.cibuildwheel.macos]
    enable = ["pypy", "cpython-freethreading"]
    archs = ["x86_64", "arm64"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.cibuildwheel.linux]"));
    assert!(result.contains("[tool.cibuildwheel.macos]"));
    assert!(result.find("cpython-freethreading").unwrap() < result.find("pypy").unwrap());
}

#[test]
fn test_cibw_long_format_overrides_aot() {
    let start = indoc::indoc! {r#"
    [[tool.cibuildwheel.overrides]]
    test-command = "pytest {project}/tests/cpython"
    select = "cp3{10,11}-*"
    enable = ["pypy", "cpython-freethreading"]
    build = "cp3*-*"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.cibuildwheel.overrides]]"));
    let block = result.split("[[tool.cibuildwheel.overrides]]").nth(1).unwrap();
    assert!(block.find("select").unwrap() < block.find("build").unwrap());
    assert!(block.find("build").unwrap() < block.find("enable").unwrap());
    assert!(block.find("cpython-freethreading").unwrap() < block.find("pypy").unwrap());
}

#[test]
fn test_cibuildwheel_override_member_that_is_not_a_table() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    overrides = [ "none" ]
    "#};
    insta::assert_snapshot!(evaluate(start), @r#"
    [tool.cibuildwheel]
    overrides = [ "none" ]
    "#);
}

/// `overrides` names a list of tables, and a value that is not one holds nothing the rule reads.
#[test]
fn test_cibuildwheel_overrides_that_is_not_an_array() {
    let start = indoc::indoc! {r#"
    [tool.cibuildwheel]
    overrides = "not-an-array"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    overrides = "not-an-array"
    "#);
}
