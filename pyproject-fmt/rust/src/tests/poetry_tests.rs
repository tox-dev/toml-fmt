use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

use super::{assert_valid_toml, collect_entries, format_syntax, parse};
use crate::poetry::{fix, reorder_inline_tables};
use crate::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    let root_ast = parse(start);
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["tool.poetry"], 120);
    fix(&mut tables);
    let entries = collect_entries(&tables);
    root_ast.splice_children(0..count, entries);
    // Inline-table reordering operates AST-wide, so run it after the root children have
    // been restored from the table_set (mirrors what format_toml does in main.rs).
    reorder_inline_tables(&root_ast);
    ensure_all_arrays_multiline(&root_ast, 120);
    let result = format_syntax(root_ast, 120);
    assert_valid_toml(&result);
    result
}

fn default_poetry_settings() -> Settings {
    Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
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
    let result = format_toml(start, &default_poetry_settings());
    assert_valid_toml(&result);
    result
}

fn long_format_settings() -> Settings {
    Settings {
        table_format: String::from("long"),
        ..default_poetry_settings()
    }
}

fn evaluate_long(start: &str) -> String {
    let result = format_toml(start, &long_format_settings());
    assert_valid_toml(&result);
    result
}

#[test]
fn test_poetry_top_level_key_order() {
    let start = indoc::indoc! {r#"
    [tool.poetry]
    requires-poetry = ">=2.0"
    classifiers = ["License :: OSI Approved :: MIT License"]
    keywords = ["toml", "format"]
    homepage = "https://example.com"
    documentation = "https://docs.example.com"
    repository = "https://github.com/example/example"
    readme = "README.md"
    license = "MIT"
    description = "Example package"
    version = "1.0.0"
    name = "example"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    name = "example"
    version = "1.0.0"
    description = "Example package"
    license = "MIT"
    readme = "README.md"
    homepage = "https://example.com"
    repository = "https://github.com/example/example"
    documentation = "https://docs.example.com"
    keywords = [ "format", "toml" ]
    classifiers = [ "License :: OSI Approved :: MIT License" ]
    requires-poetry = ">=2.0"
    "#);
}

#[test]
fn test_poetry_keywords_sorted_and_deduped() {
    let start = indoc::indoc! {r#"
    [tool.poetry]
    keywords = ["zebra", "alpha", "Alpha", "mike", "alpha"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    keywords = [ "alpha", "mike", "zebra" ]
    "#);
}

#[test]
fn test_poetry_classifiers_sorted_and_deduped() {
    let start = indoc::indoc! {r#"
    [tool.poetry]
    classifiers = [
      "Programming Language :: Python :: 3.12",
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.11",
    ]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12"
    ]
    "#);
}

#[test]
fn test_poetry_exclude_sorted() {
    let start = indoc::indoc! {r#"
    [tool.poetry]
    exclude = ["tests/*", "docs/*", "build/*"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    exclude = [ "build/*", "docs/*", "tests/*" ]
    "#);
}

#[test]
fn test_poetry_authors_preserve_order() {
    let start = indoc::indoc! {r#"
    [tool.poetry]
    authors = ["Bob <bob@example.com>", "Alice <alice@example.com>"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    authors = [ "Bob <bob@example.com>", "Alice <alice@example.com>" ]
    "#);
}

#[test]
fn test_poetry_dependencies_python_first_then_alpha() {
    let start = indoc::indoc! {r#"
    [tool.poetry.dependencies]
    zebra = "^1.0"
    alpha = "^2.0"
    python = "^3.11"
    mike = "^1.5"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    dependencies.python = "^3.11"
    dependencies.alpha = "^2.0"
    dependencies.mike = "^1.5"
    dependencies.zebra = "^1.0"
    "#);
}

#[test]
fn test_poetry_dev_dependencies_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.poetry.dev-dependencies]
    pytest = "^8.0"
    black = "^25.0"
    mypy = "^1.10"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    dev-dependencies.black = "^25.0"
    dev-dependencies.mypy = "^1.10"
    dev-dependencies.pytest = "^8.0"
    "#);
}

#[test]
fn test_poetry_extras_keys_and_values_sorted() {
    let start = indoc::indoc! {r#"
    [tool.poetry.extras]
    web = ["uvicorn", "starlette", "fastapi"]
    cli = ["typer", "rich"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    extras.cli = [ "rich", "typer" ]
    extras.web = [ "fastapi", "starlette", "uvicorn" ]
    "#);
}

#[test]
fn test_poetry_group_key_order_and_include_groups_sorted() {
    let start = indoc::indoc! {r#"
    [tool.poetry.group.dev]
    include-groups = ["lint", "test", "docs"]
    optional = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    group.dev.optional = true
    group.dev.include-groups = [ "docs", "lint", "test" ]
    "#);
}

#[test]
fn test_poetry_group_dependencies_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.poetry.group.test.dependencies]
    pytest-cov = "^5.0"
    pytest = "^8.0"
    coverage = "^7.5"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    group.test.dependencies.coverage = "^7.5"
    group.test.dependencies.pytest = "^8.0"
    group.test.dependencies.pytest-cov = "^5.0"
    "#);
}

#[test]
fn test_poetry_scripts_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.poetry.scripts]
    my-cli = "my_pkg.cli:main"
    my-server = "my_pkg.server:run"
    my-admin = "my_pkg.admin:run"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    scripts.my-admin = "my_pkg.admin:run"
    scripts.my-cli = "my_pkg.cli:main"
    scripts.my-server = "my_pkg.server:run"
    "#);
}

#[test]
fn test_poetry_urls_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.poetry.urls]
    "Source" = "https://github.com/example/example"
    "Bug Tracker" = "https://github.com/example/example/issues"
    "Funding" = "https://github.com/sponsors/example"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    urls."Bug Tracker" = "https://github.com/example/example/issues"
    urls."Funding" = "https://github.com/sponsors/example"
    urls."Source" = "https://github.com/example/example"
    "#);
}

#[test]
fn test_poetry_plugins_inner_keys_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.poetry.plugins."poetry.application.plugin"]
    zebra = "z_pkg:Z"
    alpha = "a_pkg:A"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    plugins."poetry.application.plugin".alpha = "a_pkg:A"
    plugins."poetry.application.plugin".zebra = "z_pkg:Z"
    "#);
}

#[test]
fn test_poetry_source_aot_key_order_preserves_array_order() {
    let start = indoc::indoc! {r#"
    [[tool.poetry.source]]
    priority = "primary"
    url = "https://pypi.example.com/simple"
    name = "private"

    [[tool.poetry.source]]
    url = "https://pypi.org/simple"
    name = "PyPI"
    priority = "supplemental"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    source = [
      { name = "private", url = "https://pypi.example.com/simple", priority = "primary" },
      { name = "PyPI", url = "https://pypi.org/simple", priority = "supplemental" },
    ]
    "#);
}

#[test]
fn test_poetry_source_deprecated_keys_kept_last() {
    let start = indoc::indoc! {r#"
    [[tool.poetry.source]]
    secondary = true
    name = "legacy"
    url = "https://legacy.example.com/simple"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    source = [ { name = "legacy", url = "https://legacy.example.com/simple", secondary = true } ]
    "#);
}

#[test]
fn test_poetry_build_inline_table_key_order() {
    let start = indoc::indoc! {r#"
    [tool.poetry.build]
    generate-setup-file = true
    script = "build.py"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    build.script = "build.py"
    build.generate-setup-file = true
    "#);
}

#[test]
fn test_poetry_requires_plugins_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.poetry.requires-plugins]
    poetry-plugin-export = ">=1.8"
    poetry-dynamic-versioning = ">=1.0"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    requires-plugins.poetry-dynamic-versioning = ">=1.0"
    requires-plugins.poetry-plugin-export = ">=1.8"
    "#);
}

#[test]
fn test_poetry_build_constraints_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.poetry.build-constraints]
    zeta = { setuptools = "<78" }
    alpha = { setuptools = "<79" }
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    build-constraints.alpha = { setuptools = "<79" }
    build-constraints.zeta = { setuptools = "<78" }
    "#);
}

#[test]
fn test_poetry_dependency_subtable_extras_sorted() {
    let start = indoc::indoc! {r#"
    [tool.poetry.dependencies.fastapi]
    extras = ["all", "standard"]
    version = "^0.110"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    dependencies.fastapi.extras = [ "all", "standard" ]
    dependencies.fastapi.version = "^0.110"
    "#);
}

#[test]
fn test_poetry_dependency_subtable_unsorted_extras_sorted() {
    let start = indoc::indoc! {r#"
    [tool.poetry.dependencies.fastapi]
    version = "^0.110"
    extras = ["standard", "all"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    dependencies.fastapi.extras = [ "all", "standard" ]
    dependencies.fastapi.version = "^0.110"
    "#);
}

#[test]
fn test_poetry_comments_preserved_around_reordered_keys() {
    let start = indoc::indoc! {r#"
    [tool.poetry]
    # Package name
    name = "example"
    # License
    license = "MIT"
    # Version
    version = "1.0.0"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    # Package name
    name = "example"
    # Version
    version = "1.0.0"
    # License
    license = "MIT"
    "#);
}

#[test]
fn test_poetry_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.poetry]
    name = "example"
    version = "1.0.0"
    description = "Example package"
    license = "MIT"
    authors = [ "Alice <alice@example.com>" ]
    keywords = [ "format", "toml" ]
    classifiers = [ "License :: OSI Approved :: MIT License" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_poetry_no_table_is_noop() {
    let start = indoc::indoc! {r#"
    [project]
    name = "example"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "example"
    "#);
}

#[test]
fn test_poetry_source_inline_key_order() {
    let start = indoc::indoc! {r#"
    [[tool.poetry.source]]
    priority = "primary"
    url = "https://pypi.example.com/simple"
    name = "private"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    source = [ { name = "private", url = "https://pypi.example.com/simple", priority = "primary" } ]
    "#);
}

#[test]
fn test_poetry_source_deprecated_secondary_inline_key_order() {
    let start = indoc::indoc! {r#"
    [[tool.poetry.source]]
    secondary = true
    name = "legacy"
    url = "https://legacy.example.com/simple"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    source = [ { name = "legacy", url = "https://legacy.example.com/simple", secondary = true } ]
    "#);
}

#[test]
fn test_poetry_git_dependency_inline_key_order() {
    let start = indoc::indoc! {r#"
    [tool.poetry.dependencies]
    foo = { branch = "main", git = "https://github.com/example/foo", subdirectory = "pkg" }
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    dependencies.foo = { git = "https://github.com/example/foo", branch = "main", subdirectory = "pkg" }
    "#);
}

#[test]
fn test_poetry_path_dependency_inline_key_order() {
    let start = indoc::indoc! {r#"
    [tool.poetry.dependencies]
    foo = { develop = true, path = "../foo", extras = ["all"] }
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    dependencies.foo = { path = "../foo", develop = true, extras = [ "all" ] }
    "#);
}

#[test]
fn test_poetry_file_dependency_inline_key_order() {
    let start = indoc::indoc! {r#"
    [tool.poetry.dependencies]
    foo = { python = ">=3.10", file = "./wheels/foo-1.0.whl" }
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.poetry]
    dependencies.foo = { file = "./wheels/foo-1.0.whl", python = ">=3.10" }
    "#);
}

#[test]
fn test_poetry_full_pipeline_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.poetry]
    name = "example"
    version = "1.0.0"
    description = "Example package"
    keywords = ["b", "a"]

    [[tool.poetry.source]]
    priority = "primary"
    name = "private"
    url = "https://pypi.example.com/simple"

    [tool.poetry.dependencies]
    requests = { extras = ["socks"], version = "^2.31" }
    foo = { git = "https://github.com/example/foo", branch = "main" }
    "#};
    let once = evaluate_full(start);
    let twice = evaluate_full(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_poetry_does_not_reorder_unrelated_inline_tables() {
    // [project.authors] uses `{ name, email }` inline tables. Our schemas key on
    // poetry-specific discriminators (priority, git, path, file, etc.), so authors
    // should be left untouched.
    let start = indoc::indoc! {r#"
    [project]
    name = "demo"
    authors = [ { email = "alice@example.com", name = "Alice" } ]
    "#};
    let result = evaluate_full(start);
    assert!(
        result.contains(r#"{ email = "alice@example.com", name = "Alice" }"#)
            || result.contains(r#"{ name = "Alice", email = "alice@example.com" }"#),
        "authors inline-table should not be reordered by Poetry schemas, got:\n{result}"
    );
}

#[test]
fn test_poetry_long_format_dependencies_expanded() {
    let start = indoc::indoc! {r#"
    [tool.poetry.dependencies]
    zebra = "^1.0"
    python = "^3.11"
    alpha = "^2.0"
    "#};
    let result = evaluate_long(start);
    // python first, then alphabetized
    assert!(
        result.contains("[tool.poetry.dependencies]"),
        "expected expanded form, got:\n{result}"
    );
    let py_pos = result.find("python = ").expect("python present");
    let alpha_pos = result.find("alpha = ").expect("alpha present");
    let zebra_pos = result.find("zebra = ").expect("zebra present");
    assert!(py_pos < alpha_pos && alpha_pos < zebra_pos, "ordering wrong:\n{result}");
}

#[test]
fn test_poetry_long_format_extras_expanded() {
    let start = indoc::indoc! {r#"
    [tool.poetry.extras]
    web = ["uvicorn", "fastapi", "starlette"]
    cli = ["typer", "rich"]
    "#};
    let result = evaluate_long(start);
    assert!(
        result.contains("[tool.poetry.extras]"),
        "expected expanded form, got:\n{result}"
    );
    // Inner arrays alphabetized
    assert!(
        result.contains(r#"cli = [ "rich", "typer" ]"#),
        "cli sort failed:\n{result}"
    );
    assert!(
        result.contains(r#"web = [ "fastapi", "starlette", "uvicorn" ]"#),
        "web sort failed:\n{result}"
    );
}

#[test]
fn test_poetry_long_format_scripts_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.poetry.scripts]
    z-cli = "z:main"
    a-cli = "a:main"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.poetry.scripts]"));
    let a_pos = result.find("a-cli = ").expect("a-cli");
    let z_pos = result.find("z-cli = ").expect("z-cli");
    assert!(a_pos < z_pos, "scripts not alphabetized:\n{result}");
}

#[test]
fn test_poetry_long_format_urls_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.poetry.urls]
    "Source" = "https://example.com"
    "Bug Tracker" = "https://example.com/issues"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.poetry.urls]"));
    let bug = result.find("Bug Tracker").expect("Bug Tracker");
    let src = result.find("Source").expect("Source");
    assert!(bug < src, "urls not alphabetized:\n{result}");
}

#[test]
fn test_poetry_long_format_plugins_unquoted_inner() {
    // Unquoted plugin group names (no dots) — fix_expanded_plugins handles these.
    let start = indoc::indoc! {r#"
    [tool.poetry.plugins.console_scripts]
    zebra = "z:Z"
    alpha = "a:A"
    "#};
    let result = evaluate_long(start);
    let a_pos = result.find("alpha = ").expect("alpha");
    let z_pos = result.find("zebra = ").expect("zebra");
    assert!(a_pos < z_pos, "plugins inner keys not sorted:\n{result}");
}

#[test]
fn test_poetry_long_format_group_inner_order() {
    let start = indoc::indoc! {r#"
    [tool.poetry.group.dev]
    include-groups = ["test", "lint", "docs"]
    optional = true
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.poetry.group.dev]"));
    let opt_pos = result.find("optional = ").expect("optional");
    let inc_pos = result.find("include-groups = ").expect("include-groups");
    assert!(opt_pos < inc_pos, "group key order wrong:\n{result}");
    // include-groups sorted
    assert!(
        result.contains(r#"include-groups = [ "docs", "lint", "test" ]"#),
        "include-groups not sorted:\n{result}"
    );
}

#[test]
fn test_poetry_long_format_group_dependencies_expanded() {
    let start = indoc::indoc! {r#"
    [tool.poetry.group.test.dependencies]
    zebra = "^1.0"
    python = "^3.11"
    alpha = "^2.0"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.poetry.group.test.dependencies]"));
    let py_pos = result.find("python = ").expect("python");
    let alpha_pos = result.find("alpha = ").expect("alpha");
    let zebra_pos = result.find("zebra = ").expect("zebra");
    assert!(py_pos < alpha_pos && alpha_pos < zebra_pos);
}

#[test]
fn test_poetry_long_format_source_aot_preserved() {
    let start = indoc::indoc! {r#"
    [[tool.poetry.source]]
    priority = "primary"
    url = "https://pypi.example.com/simple"
    name = "private"
    "#};
    let result = evaluate_long(start);
    assert!(
        result.contains("[[tool.poetry.source]]"),
        "expected AoT preserved, got:\n{result}"
    );
    // name → url → priority
    let n = result.find("name = ").expect("name");
    let u = result.find("url = ").expect("url");
    let p = result.find("priority = ").expect("priority");
    assert!(n < u && u < p, "source key order wrong:\n{result}");
}

#[test]
fn test_poetry_long_format_build_inline_table() {
    let start = indoc::indoc! {r#"
    [tool.poetry.build]
    generate-setup-file = true
    script = "build.py"
    "#};
    let result = evaluate_long(start);
    let s = result.find("script = ").expect("script");
    let g = result.find("generate-setup-file = ").expect("generate-setup-file");
    assert!(s < g, "build key order wrong:\n{result}");
}

#[test]
fn test_poetry_long_format_dev_dependencies_alphabetized() {
    let start = indoc::indoc! {r#"
    [tool.poetry.dev-dependencies]
    pytest = "^8.0"
    black = "^25.0"
    mypy = "^1.10"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.poetry.dev-dependencies]"));
    let b = result.find("black = ").expect("black");
    let m = result.find("mypy = ").expect("mypy");
    let p = result.find("pytest = ").expect("pytest");
    assert!(b < m && m < p);
}

#[test]
fn test_poetry_long_format_requires_plugins_expanded() {
    let start = indoc::indoc! {r#"
    [tool.poetry.requires-plugins]
    z-plug = ">=1"
    a-plug = ">=1"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.poetry.requires-plugins]"));
    let a = result.find("a-plug = ").expect("a-plug");
    let z = result.find("z-plug = ").expect("z-plug");
    assert!(a < z);
}

#[test]
fn test_poetry_long_format_build_constraints_expanded() {
    let start = indoc::indoc! {r#"
    [tool.poetry.build-constraints]
    zeta = { setuptools = "<78" }
    alpha = { setuptools = "<79" }
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.poetry.build-constraints]"));
    let a = result.find("alpha = ").expect("alpha");
    let z = result.find("zeta = ").expect("zeta");
    assert!(a < z);
}

#[test]
fn test_poetry_long_format_plugins_top_alphabetized() {
    // When there are multiple plugin groups, the parent plugins table's keys are
    // ordered via fix_expanded_plugins.
    let start = indoc::indoc! {r#"
    [tool.poetry.plugins]
    zeta = "z:Z"
    alpha = "a:A"
    "#};
    let result = evaluate_long(start);
    let a = result.find("alpha = ").expect("alpha");
    let z = result.find("zeta = ").expect("zeta");
    assert!(a < z, "top-level plugins not alphabetized:\n{result}");
}
