use super::evaluate_full as evaluate;

#[test]
fn test_maturin_top_level_order() {
    let start = indoc::indoc! {r#"
    [tool.maturin]
    compatibility = "manylinux2014"
    features = ["pyo3/extension-module"]
    python-source = "python"
    bindings = "pyo3"
    module-name = "my_pkg._lib"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.maturin]
    module-name = "my_pkg._lib"
    bindings = "pyo3"
    python-source = "python"
    features = [ "pyo3/extension-module" ]
    compatibility = "manylinux2014"
    "#);
}

#[test]
fn test_maturin_arrays_sorted_but_the_exclusions() {
    let start = indoc::indoc! {r#"
    [tool.maturin]
    features = ["zebra", "alpha", "mike"]
    include = ["src/**/*.rs", "*.toml"]
    exclude = ["zeta/*", "alpha/*"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.maturin]
    include = [ "*.toml", "src/**/*.rs" ]
    exclude = [ "zeta/*", "alpha/*" ]
    features = [ "alpha", "mike", "zebra" ]
    "#);
}

#[test]
fn test_maturin_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.maturin]
    bindings = "pyo3"
    features = [ "extension-module" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

/// maturin reads `exclude` as an ordered override program, where a `!pattern` after a broader one
/// takes back what it matched.
#[test]
fn test_maturin_exclude_keeps_its_order() {
    let start = indoc::indoc! {r#"
    [tool.maturin]
    exclude = ["*", "!keep.so"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.maturin]
    exclude = [ "*", "!keep.so" ]
    "#);
}

/// maturin hands `rustc-args` and `unstable-flags` to cargo as argv, and an `include` entry may
/// name the format it belongs to rather than a path.
#[test]
fn test_maturin_reads_the_names_it_spells_today() {
    let start = indoc::indoc! {r#"
    [tool.maturin]
    use-base-python = true
    rustc-args = ["-C", "target-cpu=native"]
    include = [{ path = "z.txt", format = "sdist" }, "a.txt"]
    unstable-flags = ["-Zbuild-std"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.maturin]
    include = [ { path = "z.txt", format = "sdist" }, "a.txt" ]
    rustc-args = [ "-C", "target-cpu=native" ]
    unstable-flags = [ "-Zbuild-std" ]
    use-base-python = true
    "#);
}
