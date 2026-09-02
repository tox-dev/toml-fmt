use super::{default_settings, evaluate_full as evaluate};

#[test]
fn test_hatch_version_first_then_build() {
    let start = indoc::indoc! {r#"
    [tool.hatch.build]
    include = ["src/**/*.py"]

    [tool.hatch.version]
    path = "src/my_pkg/__init__.py"
    source = "regex"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    version.source = "regex"
    version.path = "src/my_pkg/__init__.py"
    build.include = [ "src/**/*.py" ]
    "#);
}

/// hatch reads `include` and `exclude` the way a gitignore is read, so those keep their order while
/// the name lists sort.
#[test]
fn test_hatch_build_arrays_sorted_but_the_patterns() {
    let start = indoc::indoc! {r#"
    [tool.hatch.build]
    include = ["zebra/**", "alpha/**", "beta/**"]
    exclude = ["zeta_tests/*", "alpha_tests/*"]
    packages = ["src/zebra_pkg", "src/alpha_pkg"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    build.packages = [ "src/alpha_pkg", "src/zebra_pkg" ]
    build.include = [ "zebra/**", "alpha/**", "beta/**" ]
    build.exclude = [ "zeta_tests/*", "alpha_tests/*" ]
    "#);
}

#[test]
fn test_hatch_env_inner_key_order() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.test]
    scripts.run = "pytest"
    dependencies = ["pytest", "coverage"]
    python = "3.11"
    detached = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    envs.test.detached = true
    envs.test.python = "3.11"
    envs.test.dependencies = [ "coverage", "pytest" ]
    envs.test.scripts.run = "pytest"
    "#);
}

#[test]
fn test_hatch_env_dependencies_sorted() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.default]
    dependencies = ["pytest", "black", "mypy", "ruff"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    envs.default.dependencies = [ "black", "mypy", "pytest", "ruff" ]
    "#);
}

#[test]
fn test_hatch_metadata_keys_grouped() {
    let start = indoc::indoc! {r#"
    [tool.hatch.metadata]
    allow-ambiguous-features = true
    allow-direct-references = true
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @"
    [tool.hatch]
    metadata.allow-direct-references = true
    metadata.allow-ambiguous-features = true
    ");
}

#[test]
fn test_hatch_targets_wheel_after_build_top_level() {
    let start = indoc::indoc! {r#"
    [tool.hatch.build.targets.wheel]
    packages = ["src/my_pkg"]

    [tool.hatch.build]
    include = ["src/**"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    build.include = [ "src/**" ]
    build.targets.wheel.packages = [ "src/my_pkg" ]
    "#);
}

#[test]
fn test_hatch_multiple_envs_handled() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.lint]
    dependencies = ["ruff"]

    [tool.hatch.envs.test]
    dependencies = ["pytest"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    envs.lint.dependencies = [ "ruff" ]
    envs.test.dependencies = [ "pytest" ]
    "#);
}

#[test]
fn test_hatch_comments_preserved() {
    let start = indoc::indoc! {r#"
    [tool.hatch.version]
    # Read version from __init__.py
    path = "src/pkg/__init__.py"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    # Read version from __init__.py
    version.path = "src/pkg/__init__.py"
    "#);
}

#[test]
fn test_hatch_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.hatch.version]
    path = "src/pkg/__init__.py"
    source = "regex"

    [tool.hatch.build]
    include = ["src/**"]
    packages = ["src/pkg"]

    [tool.hatch.envs.default]
    dependencies = ["pytest"]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_hatch_no_table_is_noop() {
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
fn test_hatch_long_format_env_table_keys_reordered() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.default]
    scripts = { test = "pytest" }
    dependencies = ["zeta", "alpha"]
    python = "3.12"
    type = "virtual"
    extra-dependencies = ["zinc", "amber"]
    features = ["dev", "test"]
    platforms = ["linux", "macos"]
    pre-install-commands = ["echo pre"]
    post-install-commands = ["echo post"]
    env-include = ["FOO_*"]
    env-exclude = ["BAR_*"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.hatch.envs.default]"));
    let block = result.split("[tool.hatch.envs.default]").nth(1).unwrap();
    assert!(block.find("type").unwrap() < block.find("python").unwrap());
    assert!(block.find("python").unwrap() < block.find("dependencies").unwrap());
    assert!(block.find("\"alpha\"").unwrap() < block.find("\"zeta\"").unwrap());
    assert!(block.find("\"amber\"").unwrap() < block.find("\"zinc\"").unwrap());
}

#[test]
fn test_hatch_long_format_env_scripts_subtable() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.default.scripts]
    test = "pytest"
    lint = "ruff check"

    [tool.hatch.envs.default.env-vars]
    FOO = "1"
    BAR = "2"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.hatch.envs.default.scripts]"));
    assert!(result.contains("[tool.hatch.envs.default.env-vars]"));
    assert!(result.contains("test = \"pytest\""));
    assert!(result.contains("FOO = \"1\""));
}

#[test]
fn test_hatch_envs_key_without_inner_segment() {
    let start = indoc::indoc! {r#"
    [tool.hatch]
    envs.bare = "value"
    "#};
    let result = evaluate(start);
    assert!(result.contains("envs.bare"));
}

#[test]
fn test_hatch_long_format_matrix_aot() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.default]
    python = "3.12"

    [[tool.hatch.envs.default.matrix]]
    python = ["3.12", "3.11"]

    [[tool.hatch.envs.default.matrix]]
    python = ["3.10"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[[tool.hatch.envs.default.matrix]]"));
}

#[test]
fn test_disabled_keys_reorder_and_stay_valid_comments_issue_390() {
    let start = indoc::indoc! {r#"
        [tool.hatch]
        # TODO: re-activate after https://github.com/pypa/hatch/issues/2252
        # metadata.hooks.docstring-description = {}
        # metadata.hooks.fancy-pypi-readme.fragments = [ { path = "README.rst", start-after = ".. begin" } ]
        version.source = "vcs"
        version.raw-options = { local_scheme = "no-local-version" }  # be able to publish dev version
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    version.source = "vcs"
    version.raw-options = { local_scheme = "no-local-version" }  # be able to publish dev version
    # TODO: re-activate after https://github.com/pypa/hatch/issues/2252
    # metadata.hooks.docstring-description = {}
    # metadata.hooks.fancy-pypi-readme.fragments = [
    #   { path = "README.rst", start-after = ".. begin" }
    # ]
    "#);
    assert_eq!(evaluate(&result), result, "idempotent");
}

/// An environment name the file quoted because it holds a dot is one segment, and the tables under
/// it are found by that name rather than by the two a dotted path would read.
#[test]
fn test_hatch_environment_name_holding_a_dot_is_still_ordered() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs."a.b"]
    dependencies = ["zeta", "alpha"]
    python = "3.12"

    [tool.hatch.envs."a.b".scripts]
    zzz = "z"
    aaa = "a"
    "#};
    let result = evaluate_long(start);
    assert!(result.parse::<toml::Table>().is_ok(), "{result}");
    insta::assert_snapshot!(result, @r#"
    [tool.hatch.envs."a.b"]
    python = "3.12"
    dependencies = [ "alpha", "zeta" ]
    [tool.hatch.envs."a.b".scripts]
    aaa = "a"
    zzz = "z"
    "#);
}

/// A dotted environment name folded into `[tool.hatch]` keeps the hatch order for its keys.
#[test]
fn test_hatch_collapsed_environment_name_holding_a_dot_keeps_its_order() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs."a.b"]
    dependencies = ["zeta", "alpha"]
    description = "d"
    python = "3.12"
    "#};
    let result = evaluate(start);
    assert!(result.parse::<toml::Table>().is_ok(), "{result}");
    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    envs."a.b".description = "d"
    envs."a.b".python = "3.12"
    envs."a.b".dependencies = [ "alpha", "zeta" ]
    "#);
}

/// hatch reads a matrix element in the order it is written to build the names it generates, so the
/// variables keep that order.
#[test]
fn test_hatch_quoted_environment_matrix_keys_keep_their_order() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs."a.b"]
    python = "3.12"

    [[tool.hatch.envs."a.b".matrix]]
    zzz = ["1"]
    aaa = ["2"]
    "#};
    let result = evaluate_long(start);
    assert!(result.parse::<toml::Table>().is_ok(), "{result}");
    insta::assert_snapshot!(result, @r#"
    [tool.hatch.envs."a.b"]
    python = "3.12"
    [[tool.hatch.envs."a.b".matrix]]
    zzz = [ "1" ]
    aaa = [ "2" ]
    "#);
}

/// Hatch runs these commands in the order they are listed, and one can depend on what the one
/// before it made, so their order is what the environment says.
#[test]
fn test_hatch_install_commands_keep_their_order() {
    let start = indoc::indoc! {r#"
    [tool.hatch.envs.test]
    pre-install-commands = ["python generate.py", "python check.py"]
    post-install-commands = ["python publish.py", "python archive.py"]
    "#};
    let result = super::evaluate_long(start);

    insta::assert_snapshot!(result, @r#"
    [tool.hatch.envs.test]
    pre-install-commands = [ "python generate.py", "python check.py" ]
    post-install-commands = [ "python publish.py", "python archive.py" ]
    "#);
}

/// Hatch reads these the way a gitignore is read, where a `!pattern` after a broader one takes back
/// what it matched.
#[test]
fn test_hatch_artifacts_keep_their_order() {
    let start = indoc::indoc! {r#"
    [tool.hatch]
    build.artifacts = ["*.so", "!/foo/*.so"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.hatch]
    build.artifacts = [ "*.so", "!/foo/*.so" ]
    "#);
}

/// hatch runs the hooks and applies the overrides in the order they are written, and one sees what
/// the one before it left behind.
#[test]
fn test_hatch_hooks_and_overrides_keep_their_order() {
    let start = indoc::indoc! {r#"
    [tool.hatch.build.hooks.z-last]
    x = 1

    [tool.hatch.build.hooks.a-first]
    y = 2

    [tool.hatch.envs.demo.overrides.name."z.*"]
    set-dependencies = ["from-z"]

    [tool.hatch.envs.demo.overrides.name.".*"]
    set-dependencies = ["from-all"]
    "#};

    insta::assert_snapshot!(evaluate(start), @r#"
    [tool.hatch]
    build.hooks.z-last.x = 1
    build.hooks.a-first.y = 2
    envs.demo.overrides.name."z.*".set-dependencies = [ "from-z" ]
    envs.demo.overrides.name.".*".set-dependencies = [ "from-all" ]
    "#);
    insta::assert_snapshot!(evaluate_long(start), @r#"
    [tool.hatch.build.hooks.z-last]
    x = 1
    [tool.hatch.build.hooks.a-first]
    y = 2
    [tool.hatch.envs.demo.overrides.name."z.*"]
    set-dependencies = [ "from-z" ]
    [tool.hatch.envs.demo.overrides.name.".*"]
    set-dependencies = [ "from-all" ]
    "#);
}

fn evaluate_long(start: &str) -> String {
    super::evaluate_settings(
        start,
        &_pyproject_fmt::Settings {
            table_format: String::from("long"),
            ..default_settings()
        },
    )
}
