use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::mypy::{fix, reorder_inline_tables};
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.mypy"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    reorder_inline_tables(&root_ast);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

fn default_settings() -> Settings {
    Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 13),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        sub_table_spacing: String::new(),
        separate_root_table: String::from("\n"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
    }
}

fn evaluate_full(start: &str) -> String {
    let r = format_toml(start, &default_settings());
    assert_valid_toml(&r);
    r
}

#[test]
fn test_mypy_top_level_key_order_groups_sections() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    warn_unused_configs = true
    strict = true
    cache_dir = ".mypy_cache"
    plugins = ["pydantic.mypy"]
    pretty = true
    python_version = "3.11"
    files = ["src"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    files = [ "src" ]
    python_version = "3.11"
    strict = true
    pretty = true
    cache_dir = ".mypy_cache"
    plugins = [ "pydantic.mypy" ]
    warn_unused_configs = true
    "#);
}

#[test]
fn test_mypy_import_discovery_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    packages = ["z_pkg", "a_pkg", "m_pkg"]
    modules = ["zebra", "alpha"]
    files = ["tests", "src", "examples"]
    exclude = ["build/", "dist/", "\\.tox/"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    files = [ "examples", "src", "tests" ]
    modules = [ "alpha", "zebra" ]
    packages = [ "a_pkg", "m_pkg", "z_pkg" ]
    exclude = [ "\\.tox/", "build/", "dist/" ]
    "#);
}

#[test]
fn test_mypy_plugins_order_preserved() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    plugins = ["pydantic.mypy", "numpy.typing.mypy_plugin", "z_first"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    plugins = [ "pydantic.mypy", "numpy.typing.mypy_plugin", "z_first" ]
    "#);
}

#[test]
fn test_mypy_path_order_preserved() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    mypy_path = ["./stubs", "./src", "./vendor"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    mypy_path = [ "./stubs", "./src", "./vendor" ]
    "#);
}

#[test]
fn test_mypy_error_code_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    disable_error_code = ["import-untyped", "attr-defined", "no-untyped-def"]
    enable_error_code = ["truthy-bool", "redundant-expr"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    disable_error_code = [ "attr-defined", "import-untyped", "no-untyped-def" ]
    enable_error_code = [ "redundant-expr", "truthy-bool" ]
    "#);
}

#[test]
fn test_mypy_always_true_false_sorted() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    always_true = ["FEATURE_Z", "FEATURE_A"]
    always_false = ["DEBUG", "BETA"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    always_true = [ "FEATURE_A", "FEATURE_Z" ]
    always_false = [ "BETA", "DEBUG" ]
    "#);
}

#[test]
fn test_mypy_overrides_aot_key_order() {
    let start = indoc::indoc! {r#"
    [[tool.mypy.overrides]]
    ignore_missing_imports = true
    disable_error_code = ["attr-defined"]
    module = "third_party.*"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    overrides = [ { module = "third_party.*", ignore_missing_imports = true, disable_error_code = [ "attr-defined" ] } ]
    "#);
}

#[test]
fn test_mypy_overrides_module_as_array() {
    let start = indoc::indoc! {r#"
    [[tool.mypy.overrides]]
    ignore_missing_imports = true
    module = ["zlib_mod", "alpha_mod", "mike_mod"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    overrides = [ { module = [ "alpha_mod", "mike_mod", "zlib_mod" ], ignore_missing_imports = true } ]
    "#);
}

#[test]
fn test_mypy_overrides_disable_error_code_sorted_inside_inline() {
    let start = indoc::indoc! {r#"
    [[tool.mypy.overrides]]
    module = "tests.*"
    disable_error_code = ["no-untyped-def", "attr-defined", "import-untyped"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    overrides = [ { module = "tests.*", disable_error_code = [ "attr-defined", "import-untyped", "no-untyped-def" ] } ]
    "#);
}

#[test]
fn test_mypy_multiple_overrides_preserved_order() {
    let start = indoc::indoc! {r#"
    [[tool.mypy.overrides]]
    module = "third_party.*"
    ignore_missing_imports = true

    [[tool.mypy.overrides]]
    module = "tests.*"
    disable_error_code = ["attr-defined"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    overrides = [
      { module = "third_party.*", ignore_missing_imports = true },
      { module = "tests.*", disable_error_code = [ "attr-defined" ] },
    ]
    "#);
}

#[test]
fn test_mypy_unknown_keys_alphabetized_at_end() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    zzz_unknown = true
    aaa_unknown = false
    strict = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    strict = true
    aaa_unknown = false
    zzz_unknown = true
    "#);
}

#[test]
fn test_mypy_comments_preserved() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    # Python target version
    python_version = "3.12"
    # Be strict
    strict = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    # Python target version
    python_version = "3.12"
    # Be strict
    strict = true
    "#);
}

#[test]
fn test_mypy_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    python_version = "3.11"
    files = [ "src", "tests" ]
    strict = true
    disable_error_code = [ "attr-defined" ]
    plugins = [ "pydantic.mypy" ]

    [[tool.mypy.overrides]]
    module = "third_party.*"
    ignore_missing_imports = true
    "#};
    let once = evaluate_full(start);
    let twice = evaluate_full(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_mypy_no_table_is_noop() {
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
fn test_mypy_does_not_reorder_unrelated_inline_tables() {
    // [[project.authors]] has `{ name, email }`. Our schemas key on mypy-specific discriminators, so authors stay
    // untouched.
    let start = indoc::indoc! {r#"
    [project]
    name = "demo"
    authors = [ { email = "alice@example.com", name = "Alice" } ]
    "#};
    let result = evaluate_full(start);
    assert!(
        result.contains(r#"{ email = "alice@example.com", name = "Alice" }"#)
            || result.contains(r#"{ name = "Alice", email = "alice@example.com" }"#),
        "authors inline-table should not be reordered by mypy schemas, got:\n{result}"
    );
}

#[test]
fn test_mypy_warnings_group_together() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    warn_unreachable = true
    warn_redundant_casts = true
    warn_return_any = true
    warn_unused_ignores = true
    warn_no_return = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    warn_redundant_casts = true
    warn_unused_ignores = true
    warn_no_return = true
    warn_return_any = true
    warn_unreachable = true
    "#);
}

#[test]
fn test_mypy_report_keys_grouped() {
    let start = indoc::indoc! {r#"
    [tool.mypy]
    txt_report = "reports/txt"
    html_report = "reports/html"
    xml_report = "reports/xml"
    strict = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.mypy]
    strict = true
    html_report = "reports/html"
    txt_report = "reports/txt"
    xml_report = "reports/xml"
    "#);
}

fn long_settings() -> Settings {
    Settings {
        table_format: String::from("long"),
        ..default_settings()
    }
}

fn evaluate_long(start: &str) -> String {
    let r = format_toml(start, &long_settings());
    assert_valid_toml(&r);
    r
}

#[test]
fn test_mypy_long_format_overrides_expanded() {
    let start = indoc::indoc! {r#"
    [[tool.mypy.overrides]]
    ignore_missing_imports = true
    disable_error_code = ["import-untyped", "attr-defined"]
    module = "third_party.*"
    "#};
    let result = evaluate_long(start);
    assert!(
        result.contains("[[tool.mypy.overrides]]"),
        "expected AoT preserved:\n{result}"
    );
    let m = result.find("module = ").expect("module");
    let i = result
        .find("ignore_missing_imports = ")
        .expect("ignore_missing_imports");
    let d = result.find("disable_error_code = ").expect("disable_error_code");
    assert!(m < i && i < d, "key order wrong:\n{result}");
    assert!(
        result.contains(r#"disable_error_code = [ "attr-defined", "import-untyped" ]"#),
        "disable_error_code not sorted:\n{result}"
    );
}

#[test]
fn test_mypy_long_format_overrides_module_array_sorted() {
    let start = indoc::indoc! {r#"
    [[tool.mypy.overrides]]
    module = ["zlib_mod", "alpha_mod"]
    ignore_missing_imports = true
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.mypy.overrides]]"));
    assert!(
        result.contains(r#"module = [ "alpha_mod", "zlib_mod" ]"#),
        "module not sorted:\n{result}"
    );
}
