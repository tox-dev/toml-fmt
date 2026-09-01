use super::{evaluate_full as evaluate, evaluate_long};

#[test]
fn test_pdm_top_level_order() {
    let start = indoc::indoc! {r#"
    [tool.pdm.build]
    includes = ["src/**"]

    [tool.pdm.version]
    source = "scm"

    [tool.pdm]
    distribution = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"

    [tool.pdm]
    distribution = true
    version.source = "scm"
    build.includes = [ "src/**" ]
    "#);
}

#[test]
fn test_pdm_build_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pdm.build]
    includes = ["zebra/**", "alpha/**"]
    excludes = ["tests/*"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pdm]
    build.includes = [ "alpha/**", "zebra/**" ]
    build.excludes = [ "tests/*" ]
    "#);
}

#[test]
fn test_pdm_dev_dependencies_inner_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pdm.dev-dependencies]
    test = ["pytest", "coverage"]
    lint = ["ruff", "black"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pdm]
    dev-dependencies.lint = [ "black", "ruff" ]
    dev-dependencies.test = [ "coverage", "pytest" ]
    "#);
}

#[test]
fn test_pdm_source_aot_key_order() {
    let start = indoc::indoc! {r#"
    [[tool.pdm.source]]
    verify_ssl = false
    type = "find_links"
    url = "https://example.com/links"
    name = "internal"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pdm]
    source = [ { name = "internal", url = "https://example.com/links", type = "find_links", verify_ssl = false } ]
    "#);
}

/// The same source written out as its own header reads the same way.
#[test]
fn test_pdm_source_aot_key_order_written_out() {
    let start = indoc::indoc! {r#"
    [[tool.pdm.source]]
    verify_ssl = false
    type = "find_links"
    url = "https://example.com/links"
    name = "internal"
    "#};
    let result = evaluate_long(start);
    insta::assert_snapshot!(result, @r#"
    [[tool.pdm.source]]
    name = "internal"
    url = "https://example.com/links"
    type = "find_links"
    verify_ssl = false
    "#);
}

/// A package list of a source sorts whichever form the source is written in.
#[test]
fn test_pdm_source_packages_sorted_in_both_forms() {
    let start = indoc::indoc! {r#"
    [[tool.pdm.source]]
    name = "internal"
    include_packages = ["zebra", "alpha"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pdm]
    source = [ { name = "internal", include_packages = [ "alpha", "zebra" ] } ]
    "#);

    let written_out = evaluate_long(start);
    insta::assert_snapshot!(written_out, @r#"
    [[tool.pdm.source]]
    name = "internal"
    include_packages = [ "alpha", "zebra" ]
    "#);
}

#[test]
fn test_pdm_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.pdm]
    distribution = true
    version.source = "scm"
    build.includes = [ "src/**" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_pdm_no_table_is_noop() {
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

#[test]
fn test_pdm_long_format_scripts() {
    let start = indoc::indoc! {r#"
    [tool.pdm.scripts]
    test = "pytest"
    lint = "ruff check"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.pdm.scripts]"));
    assert!(result.contains("test = \"pytest\""));
    assert!(result.contains("lint = \"ruff check\""));
}

#[test]
fn test_pdm_long_format_dev_dependencies() {
    let start = indoc::indoc! {r#"
    [tool.pdm.dev-dependencies]
    test = ["pytest", "coverage"]
    lint = ["ruff", "black"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.pdm.dev-dependencies]"));
    let block = result.split("[tool.pdm.dev-dependencies]").nth(1).unwrap();
    assert!(block.find("\"black\"").unwrap() < block.find("\"ruff\"").unwrap());
    assert!(block.find("\"coverage\"").unwrap() < block.find("\"pytest\"").unwrap());
}

#[test]
fn test_pdm_long_format_source_aot() {
    let start = indoc::indoc! {r#"
    [[tool.pdm.source]]
    verify_ssl = false
    exclude_packages = ["zeta", "alpha"]
    type = "find_links"
    include_packages = ["zinc", "amber"]
    url = "https://example.com/links"
    name = "internal"
    extra = "value"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.pdm.source]]"));
    let block = result.split("[[tool.pdm.source]]").nth(1).unwrap();
    assert!(block.find("name").unwrap() < block.find("url").unwrap());
    assert!(block.find("url").unwrap() < block.find("type").unwrap());
    assert!(block.find("type").unwrap() < block.find("verify_ssl").unwrap());
    assert!(block.find("verify_ssl").unwrap() < block.find("include_packages").unwrap());
    assert!(block.find("\"amber\"").unwrap() < block.find("\"zinc\"").unwrap());
    assert!(block.find("\"alpha\"").unwrap() < block.find("\"zeta\"").unwrap());
}
