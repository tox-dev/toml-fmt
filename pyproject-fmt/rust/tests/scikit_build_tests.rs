use super::{evaluate_full as evaluate, evaluate_long};

#[test]
fn test_scikit_build_order() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    wheel.packages = ["src/foo"]
    cmake.version = ">=3.20"
    minimum-version = "0.9"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    minimum-version = "0.9"
    cmake.version = ">=3.20"
    wheel.packages = [ "src/foo" ]
    "#);
}

#[test]
fn test_scikit_build_no_table_noop() {
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
fn test_scikit_build_args_preserved() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    cmake.args = ["-DZ=1", "-DA=2"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    cmake.args = [ "-DZ=1", "-DA=2" ]
    "#);
}

#[test]
fn test_scikit_build_define_preserved() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build.cmake]
    define = { Z_FLAG = "z", A_FLAG = "a" }
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    cmake.define = { Z_FLAG = "z", A_FLAG = "a" }
    "#);
}

#[test]
fn test_scikit_build_packages_keep_their_order() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    wheel.packages = ["src/zeta", "src/alpha", "src/beta"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    wheel.packages = [ "src/zeta", "src/alpha", "src/beta" ]
    "#);
}

#[test]
fn test_scikit_build_exclude_keeps_its_order() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    sdist.exclude = ["zeta/**", "alpha/**"]
    sdist.include = ["zinc/*", "amber/*"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    sdist.exclude = [ "zeta/**", "alpha/**" ]
    sdist.include = [ "zinc/*", "amber/*" ]
    "#);
}

#[test]
fn test_scikit_build_files_sorted_but_the_ordered_lists() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    install.components = ["zlib", "alib"]
    wheel.exclude-fields = ["metadata.version", "metadata.author"]
    install.targets = ["zt", "at"]
    sdist.files = ["zf", "af"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.scikit-build]
    sdist.files = [ "af", "zf" ]
    wheel.exclude-fields = [ "metadata.author", "metadata.version" ]
    install.components = [ "zlib", "alib" ]
    install.targets = [ "zt", "at" ]
    "#);
}

#[test]
fn test_scikit_build_long_format() {
    let start = indoc::indoc! {r#"
    [tool.scikit-build]
    wheel.packages = ["src/zeta", "src/alpha"]
    cmake.args = ["-DZ=1", "-DA=2"]
    minimum-version = "0.9"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.scikit-build]"));
    assert!(result.contains("minimum-version = \"0.9\""));
}
