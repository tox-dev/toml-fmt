use super::{evaluate_full as evaluate, evaluate_long};

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
    env_run_base.runner = "uv-venv-lock-runner"
    env_run_base.package = "wheel"
    env_run_base.deps = [ "coverage", "pytest>=7" ]
    env_run_base.extras = [ "all", "test" ]
    env_run_base.commands = [ [ "pytest" ] ]
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
    env.lint.runner = "uv-venv-lock-runner"
    env.lint.deps = [ "ruff" ]
    env.lint.commands = [ [ "ruff", "check" ] ]
    "#);
}

/// `env_list` is written newest interpreter first, embedded in a `pyproject.toml` just as in a
/// `tox.toml`.
#[test]
fn test_tox_env_list_is_written_newest_interpreter_first() {
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
    env.test.deps = [ "alpha", "mike", "zebra" ]
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

#[test]
fn test_tox_leaves_the_tables_of_other_tools_alone() {
    let start = indoc::indoc! {r#"
    [project]
    name = "example"

    [tool.tox.env.docs]
    description = "docs"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "example"

    [tool.tox]
    env.docs.description = "docs"
    "#);
}

/// The short format folds every environment into `[tool.tox]`, and each rule still reaches it: an
/// alias is renamed, `use_develop` migrates, a requirement is normalized, and a list sorts.
#[test]
fn test_tox_env_rules_reach_a_folded_environment() {
    let start = indoc::indoc! {r#"
    [tool.tox.env.test]
    setenv = { A = "1" }
    usedevelop = true
    deps = ["Zebra >= 1.0", "alpha"]
    passenv = ["PATH", "HOME"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.tox]
    env.test.package = "editable"
    env.test.deps = [ "alpha", "zebra>=1" ]
    env.test.pass_env = [ "HOME", "PATH" ]
    env.test.set_env = { A = "1" }
    "#);
}

/// The same file in the format that keeps a header per environment reads the same way.
#[test]
fn test_tox_env_rules_read_the_same_written_out() {
    let start = indoc::indoc! {r#"
    [tool.tox.env.test]
    setenv = { A = "1" }
    usedevelop = true
    deps = ["Zebra >= 1.0", "alpha"]
    passenv = ["PATH", "HOME"]
    "#};
    let result = evaluate_long(start);
    insta::assert_snapshot!(result, @r#"
    [tool.tox.env.test]
    package = "editable"
    deps = [ "alpha", "zebra>=1" ]
    pass_env = [ "HOME", "PATH" ]
    set_env = { A = "1" }
    "#);
}

#[test]
fn test_tox_folded_environment_rules_are_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.tox.env.test]
    deps = ["zebra", "alpha"]
    runner = "uv-venv-lock-runner"
    "#};
    let once = evaluate(start);

    assert_eq!(evaluate(&once), once);
}

/// The tox table embedded in a `pyproject.toml` is the same table however the file splits its path.
#[test]
fn test_the_embedded_tox_table_is_read_however_the_file_writes_it() {
    let held = |start: &str| {
        _pyproject_fmt::format_toml(
            start,
            &_pyproject_fmt::Settings {
                column_width: 120,
                indent: 2,
                keep_full_version: false,
                max_supported_python: (3, 12),
                min_supported_python: (3, 10),
                generate_python_version_classifiers: false,
                table_format: String::from("short"),
                sub_table_spacing: String::new(),
                separate_root_table: String::from("\n"),
                expand_tables: vec![],
                collapse_tables: vec![],
                skip_wrap_for_keys: vec![],
            },
        )
        .expect("the formatter accepts it")
    };

    insta::assert_snapshot!(held("tool.tox.minversion = \"4.0\"\n"), @r#"tool.tox.min_version = "4.0""#);
    insta::assert_snapshot!(
        held("tool = { tox = { requires = [ \"z\", \"a\" ] } }\n"),
        @r#"tool = { tox = { requires = [ "a", "z" ] } }"#
    );
}
