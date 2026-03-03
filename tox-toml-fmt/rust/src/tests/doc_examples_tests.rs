use indoc::indoc;
use insta::assert_snapshot;

use super::assert_valid_toml;
use crate::{format_toml, Settings};

fn format_toml_helper(start: &str, indent: usize) -> String {
    let settings = Settings {
        column_width: 120,
        indent,
        table_format: String::from("short"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    };
    let got = format_toml(start, &settings);
    assert_valid_toml(&got);
    let second = format_toml(got.as_str(), &settings);
    assert_eq!(second, got, "formatting should be idempotent");
    got
}

// --- reference/config.rst examples ---

#[test]
fn test_doc_conditional_deps_and_commands() {
    let start = indoc! {r#"
        [env_run_base]
        deps = [
            "pytest",
            { replace = "if", condition = "factor.django50", then = ["Django>=5.0,<5.1"] },
            { replace = "if", condition = "factor.django42", then = ["Django>=4.2,<4.3"] },
            { replace = "if", condition = "not factor.lint", then = ["coverage"] },
        ]
        commands = [
            { replace = "if", condition = "factor.linux", then = [["python", "-c", "print('on linux')"]] },
            { replace = "if", condition = "factor.darwin", then = [["python", "-c", "print('on mac')"]] },
            { replace = "if", condition = "factor.win32", then = [["python", "-c", "print('on windows')"]] },
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [
      "pytest",
      { replace = "if", condition = "factor.django50", then = [ "Django>=5.0,<5.1" ] },
      { replace = "if", condition = "factor.django42", then = [ "Django>=4.2,<4.3" ] },
      { replace = "if", condition = "not factor.lint", then = [ "coverage" ] },
    ]
    commands = [
      { replace = "if", condition = "factor.linux", then = [ [ "python", "-c", "print('on linux')" ] ] },
      { replace = "if", condition = "factor.darwin", then = [ [ "python", "-c", "print('on mac')" ] ] },
      { replace = "if", condition = "factor.win32", then = [ [ "python", "-c", "print('on windows')" ] ] },
    ]
    "#);
}

#[test]
fn test_doc_cartesian_product_env_list() {
    let start = indoc! {r#"
        env_list = [
            { product = [["py312", "py313", "py314"], ["django42", "django50"]] },
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      { product = [ [ "py312", "py313", "py314" ], [ "django42", "django50" ] ] },
    ]
    "#);
}

#[test]
fn test_doc_env_base_with_conditional() {
    let start = indoc! {r#"
        [env_base.build]
        factors = [["py312", "py313"], ["x86", "x64"]]
        env_dir = { replace = "if", condition = "factor.x86", then = ".venv-x86", "else" = ".venv-x64" }
        commands = [["python", "-c", "print('ok')"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_base.build]
    factors = [ [ "py312", "py313" ], [ "x86", "x64" ] ]
    commands = [ [ "python", "-c", "print('ok')" ] ]
    env_dir = { replace = "if", condition = "factor.x86", then = ".venv-x86", else = ".venv-x64" }
    "#);
}

#[test]
fn test_doc_simple_env_base_template() {
    let start = indoc! {r#"
        [env_base.test]
        factors = [["3.13", "3.14"]]
        deps = ["pytest>=8"]
        commands = [["pytest"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_base.test]
    factors = [ [ "3.13", "3.14" ] ]
    deps = [ "pytest>=8" ]
    commands = [ [ "pytest" ] ]
    "#);
}

#[test]
fn test_doc_multi_dimensional_env_base() {
    let start = indoc! {r#"
        [env_base.django]
        factors = [["py312", "py313"], ["django42", "django50"]]
        deps = [
            "pytest",
            { replace = "if", condition = "factor.django42", then = ["Django>=4.2,<4.3"] },
            { replace = "if", condition = "factor.django50", then = ["Django>=5.0,<5.1"] },
        ]
        commands = [["pytest"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_base.django]
    factors = [ [ "py312", "py313" ], [ "django42", "django50" ] ]
    deps = [
      "pytest",
      { replace = "if", condition = "factor.django42", then = [ "Django>=4.2,<4.3" ] },
      { replace = "if", condition = "factor.django50", then = [ "Django>=5.0,<5.1" ] },
    ]
    commands = [ [ "pytest" ] ]
    "#);
}

#[test]
fn test_doc_tox_toml_dedicated() {
    let start = indoc! {r#"
        requires = ["tox>=4.19"]
        env_list = ["3.14t", "3.14", "3.13", "3.12", "type"]

        [env_run_base]
        description = "Run test under {base_python}"
        commands = [["pytest"]]

        [env.type]
        description = "run type check on code base"
        deps = ["mypy==1.18.2", "types-cachetools>=5.5.0.20240820", "types-chardet>=5.0.4.6"]
        commands = [["mypy", "src{/}tox"], ["mypy", "tests"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    requires = [ "tox>=4.19" ]
    env_list = [ "3.14", "3.13", "3.12", "3.14t", "type" ]

    [env_run_base]
    description = "Run test under {base_python}"
    commands = [ [ "pytest" ] ]

    [env.type]
    description = "run type check on code base"
    deps = [ "mypy==1.18.2", "types-cachetools>=5.5.0.20240820", "types-chardet>=5.0.4.6" ]
    commands = [ [ "mypy", "src{/}tox" ], [ "mypy", "tests" ] ]
    "#);
}

#[test]
fn test_doc_requires_multi_deps() {
    let start = indoc! {r#"
        requires = [
          "tox>=4",
          "virtualenv>20.2",
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    requires = [
      "tox>=4",
      "virtualenv>20.2",
    ]
    "#);
}

#[test]
fn test_doc_labels() {
    let start = indoc! {r#"
        labels = { test = ["3.14t", "3.14", "3.13", "3.12"], static = ["ruff", "mypy"] }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"labels = { test = [ "3.14t", "3.14", "3.13", "3.12" ], static = [ "ruff", "mypy" ] }"#);
}

#[test]
fn test_doc_pass_env() {
    let start = indoc! {r#"
        [env_run_base]
        pass_env = ["FOO_*"]
        disallow_pass_env = ["FOO_SECRET"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    pass_env = [ "FOO_*" ]
    disallow_pass_env = [ "FOO_SECRET" ]
    "#);
}

#[test]
fn test_doc_set_env_marker() {
    let start = indoc! {r#"
        [env_run_base]
        set_env.LINUX_VAR = { value = "1", marker = "sys_platform == 'linux'" }
        set_env.WIN_VAR = { value = "1", marker = "sys_platform == 'win32'" }
        set_env.CONDITIONAL = { replace = "env", name = "MY_VAR", default = "fallback", marker = "sys_platform == 'linux'" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env.CONDITIONAL = { replace = "env", name = "MY_VAR", default = "fallback", marker = "sys_platform == 'linux'" }
    set_env.LINUX_VAR = { value = "1", marker = "sys_platform == 'linux'" }
    set_env.WIN_VAR = { value = "1", marker = "sys_platform == 'win32'" }
    "#);
}

#[test]
fn test_doc_env_labels() {
    let start = indoc! {r#"
        [env_run_base]
        labels = ["test", "core"]

        [env.flake8]
        labels = ["mypy"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    labels = [ "core", "test" ]

    [env.flake8]
    labels = [ "mypy" ]
    "#);
}

#[test]
fn test_doc_depends() {
    let start = indoc! {r#"
        [env.coverage]
        depends = ["3.*"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.coverage]
    depends = [ "3.*" ]
    "#);
}

#[test]
fn test_doc_extra_setup_commands() {
    let start = indoc! {r#"
        [env_run_base]
        deps = ["pre-commit"]
        extra_setup_commands = [
          ["pre-commit", "install-hooks"],
        ]
        commands = [
          ["pre-commit", "run", "--all-files"],
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [ "pre-commit" ]
    extra_setup_commands = [
      [ "pre-commit", "install-hooks" ],
    ]
    commands = [
      [ "pre-commit", "run", "--all-files" ],
    ]
    "#);
}

#[test]
fn test_doc_recreate_commands() {
    let start = indoc! {r#"
        [env_run_base]
        deps = ["pre-commit"]
        recreate_commands = [["{env_python}", "-Im", "pre_commit", "clean"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [ "pre-commit" ]
    recreate_commands = [ [ "{env_python}", "-Im", "pre_commit", "clean" ] ]
    "#);
}

#[test]
fn test_doc_commands_retry() {
    let start = indoc! {r#"
        [env_run_base]
        commands_retry = 2
        commands = [["pytest", "tests"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    commands_retry = 2
    commands = [ [ "pytest", "tests" ] ]
    "#);
}

#[test]
fn test_doc_default_base_python() {
    let start = indoc! {r#"
        [env_run_base]
        default_base_python = ["3.14", "3.13"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    default_base_python = [ "3.14", "3.13" ]
    "#);
}

#[test]
fn test_doc_pylock() {
    let start = indoc! {r#"
        [env_run_base]
        pylock = "pylock.toml"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    pylock = "pylock.toml"
    "#);
}

#[test]
fn test_doc_deps_with_requirements_file() {
    let start = indoc! {r#"
        [env_run_base]
        deps = [
          "pytest>=8",
          "-r requirements.txt",
          "-c constraints.txt",
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [
      "-c constraints.txt",
      "-r requirements.txt",
      "pytest>=8",
    ]
    "#);
}

#[test]
fn test_doc_ref_extend_env_key() {
    let start = indoc! {r#"
        [env.src]
        extras = ["A", "{env_name}"]
        [env.dest]
        extras = [{ replace = "ref", env = "src", key = "extras", extend = true }, "B"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.dest]
    extras = [ { replace = "ref", env = "src", key = "extras", extend = true }, "B" ]

    [env.src]
    extras = [ "{env_name}", "A" ]
    "#);
}

#[test]
fn test_doc_ref_extend_of() {
    let start = indoc! {r#"
        [env.src]
        extras = ["A", "{env_name}"]
        [env.dest]
        extras = [{ replace = "ref", of = ["env", "extras"], extend = true }, "B"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.dest]
    extras = [ { replace = "ref", of = [ "env", "extras" ], extend = true }, "B" ]

    [env.src]
    extras = [ "{env_name}", "A" ]
    "#);
}

#[test]
fn test_doc_posargs_default_extend() {
    let start = indoc! {r#"
        [env.A]
        commands = [["python", { replace = "posargs", default = ["a", "b"], extend = true } ]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    commands = [ [ "python", { replace = "posargs", default = [ "a", "b" ], extend = true } ] ]
    "#);
}

#[test]
fn test_doc_posargs_as_default_command() {
    let start = indoc! {r#"
        [env.A]
        commands = [
            { replace = "posargs", default = ["python", "patch.py"]},
            ["pytest"]
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    commands = [ { replace = "posargs", default = [ "python", "patch.py" ] }, [ "pytest" ] ]
    "#);
}

#[test]
fn test_doc_env_replacement() {
    let start = indoc! {r#"
        [env.A]
        set_env.COVERAGE_FILE = { replace = "env", name = "COVERAGE_FILE", default = "ok" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    set_env.COVERAGE_FILE = { replace = "env", name = "COVERAGE_FILE", default = "ok" }
    "#);
}

#[test]
fn test_doc_ref_merging() {
    let start = indoc! {r#"
        [env_run_base]
        set_env = { A = "1", B = "2"}

        [env.magic]
        set_env = [
            { replace = "ref", of = ["env_run_base", "set_env"]},
            { C = "3", D = "4"},
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env = { A = "1", B = "2" }

    [env.magic]
    set_env = [
      { replace = "ref", of = [ "env_run_base", "set_env" ] },
      { C = "3", D = "4" },
    ]
    "#);
}

#[test]
fn test_doc_glob_pattern() {
    let start = indoc! {r#"
        [env.A]
        commands = [["twine", "upload", { replace = "glob", pattern = "dist/*.whl", extend = true }]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    commands = [ [ "twine", "upload", { replace = "glob", pattern = "dist/*.whl", extend = true } ] ]
    "#);
}

#[test]
fn test_doc_glob_with_default() {
    let start = indoc! {r#"
        [env.A]
        commands = [["twine", "upload", { replace = "glob", pattern = "dist/*.whl", default = ["fallback.whl"], extend = true }]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    commands = [
      [ "twine", "upload", { replace = "glob", pattern = "dist/*.whl", default = [ "fallback.whl" ], extend = true } ]
    ]
    "#);
}

#[test]
fn test_doc_if_env_var() {
    let start = indoc! {r#"
        [env.A]
        set_env.MATURITY = { replace = "if", condition = "env.TAG_NAME", then = "production", "else" = "testing" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    set_env.MATURITY = { replace = "if", condition = "env.TAG_NAME", then = "production", else = "testing" }
    "#);
}

#[test]
fn test_doc_if_equality_check() {
    let start = indoc! {r#"
        [env.A]
        set_env.MODE = { replace = "if", condition = "env.CI == 'true'", then = "ci", "else" = "local" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    set_env.MODE = { replace = "if", condition = "env.CI == 'true'", then = "ci", else = "local" }
    "#);
}

#[test]
fn test_doc_if_boolean_logic() {
    let start = indoc! {r#"
        [env.A]
        description = { replace = "if", condition = "env.CI and env.DEPLOY", then = "deploying", "else" = "skipped" }

        [env.B]
        description = { replace = "if", condition = "env.CI or env.LOCAL", then = "active", "else" = "inactive" }

        [env.C]
        description = { replace = "if", condition = "not env.CI", then = "local dev", "else" = "CI build" }

        [env.D]
        description = { replace = "if", condition = "env.MODE != 'prod'", then = "non-production", "else" = "production" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    description = { replace = "if", condition = "env.CI and env.DEPLOY", then = "deploying", else = "skipped" }

    [env.B]
    description = { replace = "if", condition = "env.CI or env.LOCAL", then = "active", else = "inactive" }

    [env.C]
    description = { replace = "if", condition = "not env.CI", then = "local dev", else = "CI build" }

    [env.D]
    description = { replace = "if", condition = "env.MODE != 'prod'", then = "non-production", else = "production" }
    "#);
}

#[test]
fn test_doc_if_no_else() {
    let start = indoc! {r#"
        [env.A]
        description = { replace = "if", condition = "env.DEPLOY", then = "deployment mode" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    description = { replace = "if", condition = "env.DEPLOY", then = "deployment mode" }
    "#);
}

#[test]
fn test_doc_if_with_env_name() {
    let start = indoc! {r#"
        [env.A]
        description = { replace = "if", condition = "env.DEPLOY", then = "{env_name}", "else" = "none" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    description = { replace = "if", condition = "env.DEPLOY", then = "{env_name}", else = "none" }
    "#);
}

#[test]
fn test_doc_if_list_values_extend() {
    let start = indoc! {r#"
        [env.A]
        commands = [["pytest", { replace = "if", condition = "env.VERBOSE", then = ["--verbose", "--debug"], "else" = ["--quiet"], extend = true }]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.A]
    commands = [
      [
        "pytest",
        { replace = "if", condition = "env.VERBOSE", then = [ "--verbose", "--debug" ], else = [ "--quiet" ], extend = true },
      ],
    ]
    "#);
}

#[test]
fn test_doc_factor_based_commands() {
    let start = indoc! {r#"
        [env_run_base]
        commands = [
            { replace = "if", condition = "factor.linux", then = [["pytest", "--numprocesses=auto"]] },
            { replace = "if", condition = "not factor.linux", then = [["pytest"]] },
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    commands = [
      { replace = "if", condition = "factor.linux", then = [ [ "pytest", "--numprocesses=auto" ] ] },
      { replace = "if", condition = "not factor.linux", then = [ [ "pytest" ] ] },
    ]
    "#);
}

#[test]
fn test_doc_factor_and_env_combined() {
    let start = indoc! {r#"
        [env_run_base]
        commands = [["pytest", { replace = "if", condition = "factor.linux and env.CI", then = ["--numprocesses=auto"], "else" = [], extend = true }]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    commands = [
      [
        "pytest",
        { replace = "if", condition = "factor.linux and env.CI", then = [ "--numprocesses=auto" ], else = [], extend = true },
      ],
    ]
    "#);
}

#[test]
fn test_doc_complex_matrix_three_factors() {
    let start = indoc! {r#"
        env_list = [
            "lint",
            { product = [
                { prefix = "py3", start = 9, stop = 11 },
                ["django41", "django40"],
                ["sqlite", "mysql"],
            ] },
        ]

        [env_run_base]
        deps = [
            { replace = "if", condition = "factor.django41", then = ["Django>=4.1,<4.2"] },
            { replace = "if", condition = "factor.django40", then = ["Django>=4.0,<4.1"] },
            { replace = "if", condition = "factor.py311 and factor.mysql", then = ["PyMySQL"] },
            { replace = "if", condition = "factor.py311 or factor.py310", then = ["urllib3"] },
            { replace = "if", condition = "(factor.py311 or factor.py310) and factor.sqlite", then = ["mock"] },
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      "lint",
      { product = [
        { prefix = "py3", start = 9, stop = 11 },
        [ "django41", "django40" ],
        [ "sqlite", "mysql" ],
      ] },
    ]

    [env_run_base]
    deps = [
      { replace = "if", condition = "factor.django41", then = [ "Django>=4.1,<4.2" ] },
      { replace = "if", condition = "factor.django40", then = [ "Django>=4.0,<4.1" ] },
      { replace = "if", condition = "factor.py311 and factor.mysql", then = [ "PyMySQL" ] },
      { replace = "if", condition = "factor.py311 or factor.py310", then = [ "urllib3" ] },
      { replace = "if", condition = "(factor.py311 or factor.py310) and factor.sqlite", then = [ "mock" ] },
    ]
    "#);
}

#[test]
fn test_doc_range_env_list() {
    let start = indoc! {r#"
        env_list = [
            { product = [{ prefix = "py3", start = 10 }, ["django42"]] },
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      { product = [ { prefix = "py3", start = 10 }, [ "django42" ] ] },
    ]
    "#);
}

#[test]
fn test_doc_product_with_exclude() {
    let start = indoc! {r#"
        env_list = [
            { product = [["py312", "py313"], ["django42", "django50"]], exclude = ["py312-django50"] },
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      { product = [ [ "py312", "py313" ], [ "django42", "django50" ] ], exclude = [ "py312-django50" ] },
    ]
    "#);
}

#[test]
fn test_doc_arch_specific_config() {
    let start = indoc! {r#"
        [env_base.py311-venv]
        factors = [["x86", "x64"]]
        base_python = { replace = "if", condition = "factor.x86", then = "python3.11-32", "else" = "python3.11-64" }
        env_dir = { replace = "if", condition = "factor.x86", then = ".venv-x86", "else" = ".venv-x64" }
        commands = [["pytest"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_base.py311-venv]
    factors = [ [ "x86", "x64" ] ]
    base_python = { replace = "if", condition = "factor.x86", then = "python3.11-32", else = "python3.11-64" }
    commands = [ [ "pytest" ] ]
    env_dir = { replace = "if", condition = "factor.x86", then = ".venv-x86", else = ".venv-x64" }
    "#);
}

#[test]
fn test_doc_env_pkg_base_pass_env() {
    let start = indoc! {r#"
        [env_pkg_base]
        pass_env = ["PKG_CONFIG", "PKG_CONFIG_PATH", "PKG_CONFIG_SYSROOT_DIR"]

        [env.".pkg-cpython311"]
        pass_env = ["PKG_CONFIG", "PKG_CONFIG_PATH", "PKG_CONFIG_SYSROOT_DIR", "IS_311"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_pkg_base]
    pass_env = [ "PKG_CONFIG", "PKG_CONFIG_PATH", "PKG_CONFIG_SYSROOT_DIR" ]

    [env.".pkg-cpython311"]
    pass_env = [ "IS_311", "PKG_CONFIG", "PKG_CONFIG_PATH", "PKG_CONFIG_SYSROOT_DIR" ]
    "#);
}

#[test]
fn test_doc_virtualenv_spec() {
    let start = indoc! {r#"
        [env.legacy]
        base_python = ["python3.6"]
        virtualenv_spec = "virtualenv<20.22.0"
        commands = [["python", "-c", "import sys; print(sys.version)"]]

        [env.modern]
        base_python = ["python3.15"]
        commands = [["python", "-c", "import sys; print(sys.version)"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.legacy]
    base_python = [ "python3.6" ]
    virtualenv_spec = "virtualenv<20.22.0"
    commands = [ [ "python", "-c", "import sys; print(sys.version)" ] ]

    [env.modern]
    base_python = [ "python3.15" ]
    commands = [ [ "python", "-c", "import sys; print(sys.version)" ] ]
    "#);
}

// --- how-to/usage.rst examples ---

#[test]
fn test_doc_basic_pytest() {
    let start = indoc! {r#"
        env_list = ["3.13", "3.12"]

        [env_run_base]
        deps = ["pytest>=8"]
        commands = [["pytest", { replace = "posargs", default = ["tests"], extend = true }]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [ "3.13", "3.12" ]

    [env_run_base]
    deps = [ "pytest>=8" ]
    commands = [ [ "pytest", { replace = "posargs", default = [ "tests" ], extend = true } ] ]
    "#);
}

#[test]
fn test_doc_coverage_collection() {
    let start = indoc! {r#"
        env_list = ["3.13", "3.12", "coverage"]

        [env_run_base]
        deps = ["pytest", "coverage[toml]"]
        commands = [["coverage", "run", "-p", "-m", "pytest", "tests"]]

        [env.coverage]
        skip_install = true
        deps = ["coverage[toml]"]
        depends = ["3.*"]
        commands = [
            ["coverage", "combine"],
            ["coverage", "report", "--fail-under=80"],
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [ "3.13", "3.12", "coverage" ]

    [env_run_base]
    deps = [ "coverage[toml]", "pytest" ]
    commands = [ [ "coverage", "run", "-p", "-m", "pytest", "tests" ] ]

    [env.coverage]
    skip_install = true
    deps = [ "coverage[toml]" ]
    commands = [
      [ "coverage", "combine" ],
      [ "coverage", "report", "--fail-under=80" ],
    ]
    depends = [ "3.*" ]
    "#);
}

#[test]
fn test_doc_labels_grouping() {
    let start = indoc! {r#"
        env_list = ["3.13", "3.12", "lint", "type"]

        [env_run_base]
        labels = ["test"]
        commands = [["pytest", "tests"]]

        [env.lint]
        labels = ["check"]
        skip_install = true
        deps = ["ruff"]
        commands = [["ruff", "check", "."]]

        [env.type]
        labels = ["check"]
        deps = ["mypy"]
        commands = [["mypy", "src"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [ "3.13", "3.12", "lint", "type" ]

    [env_run_base]
    commands = [ [ "pytest", "tests" ] ]
    labels = [ "test" ]

    [env.lint]
    skip_install = true
    deps = [ "ruff" ]
    commands = [ [ "ruff", "check", "." ] ]
    labels = [ "check" ]

    [env.type]
    deps = [ "mypy" ]
    commands = [ [ "mypy", "src" ] ]
    labels = [ "check" ]
    "#);
}

#[test]
fn test_doc_platform_specific_deps() {
    let start = indoc! {r#"
        [env_run_base]
        deps = [
            "pytest",
            { replace = "if", condition = "factor.linux or factor.darwin", then = ["platformdirs>=3"] },
            { replace = "if", condition = "factor.win32", then = ["platformdirs>=2"] },
        ]
        commands = [
            { replace = "if", condition = "factor.linux", then = [["python", "-c", "print('Running on Linux')"]] },
            { replace = "if", condition = "factor.darwin", then = [["python", "-c", "print('Running on macOS')"]] },
            { replace = "if", condition = "factor.win32", then = [["python", "-c", "print('Running on Windows')"]] },
            ["python", "-m", "pytest"],
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [
      "pytest",
      { replace = "if", condition = "factor.linux or factor.darwin", then = [ "platformdirs>=3" ] },
      { replace = "if", condition = "factor.win32", then = [ "platformdirs>=2" ] },
    ]
    commands = [
      { replace = "if", condition = "factor.linux", then = [ [ "python", "-c", "print('Running on Linux')" ] ] },
      { replace = "if", condition = "factor.darwin", then = [ [ "python", "-c", "print('Running on macOS')" ] ] },
      { replace = "if", condition = "factor.win32", then = [ [ "python", "-c", "print('Running on Windows')" ] ] },
      [ "python", "-m", "pytest" ],
    ]
    "#);
}

#[test]
fn test_doc_multi_dim_platform_django() {
    let start = indoc! {r#"
        env_list = [
            { product = [["py312", "py313"], ["django42", "django50"]] },
        ]

        [env_run_base]
        deps = [
            { replace = "if", condition = "factor.django42", then = ["Django>=4.2,<4.3"] },
            { replace = "if", condition = "factor.django50", then = ["Django>=5.0,<5.1"] },
            { replace = "if", condition = "factor.py312 and factor.linux", then = ["pytest-xdist"] },
            { replace = "if", condition = "factor.darwin", then = ["pyobjc-framework-Cocoa"] },
        ]
        commands = [
            { replace = "if", condition = "factor.win32", then = [["python", "-c", "import winreg"]] },
            ["pytest"],
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      { product = [ [ "py312", "py313" ], [ "django42", "django50" ] ] },
    ]

    [env_run_base]
    deps = [
      { replace = "if", condition = "factor.django42", then = [ "Django>=4.2,<4.3" ] },
      { replace = "if", condition = "factor.django50", then = [ "Django>=5.0,<5.1" ] },
      { replace = "if", condition = "factor.py312 and factor.linux", then = [ "pytest-xdist" ] },
      { replace = "if", condition = "factor.darwin", then = [ "pyobjc-framework-Cocoa" ] },
    ]
    commands = [
      { replace = "if", condition = "factor.win32", then = [ [ "python", "-c", "import winreg" ] ] },
      [ "pytest" ],
    ]
    "#);
}

#[test]
fn test_doc_negated_platform_factors() {
    let start = indoc! {r#"
        [env_run_base]
        deps = [
            { replace = "if", condition = "not factor.win32", then = ["uvloop"] },
            { replace = "if", condition = "not factor.darwin", then = ["pyinotify"] },
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [
      { replace = "if", condition = "not factor.win32", then = [ "uvloop" ] },
      { replace = "if", condition = "not factor.darwin", then = [ "pyinotify" ] },
    ]
    "#);
}

#[test]
fn test_doc_platform_specific_commands() {
    let start = indoc! {r#"
        [env_run_base]
        commands = [
            { replace = "if", condition = "factor.linux", then = [["pytest", "--numprocesses=auto"]] },
            { replace = "if", condition = "factor.darwin or factor.win32", then = [["pytest"]] },
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    commands = [
      { replace = "if", condition = "factor.linux", then = [ [ "pytest", "--numprocesses=auto" ] ] },
      { replace = "if", condition = "factor.darwin or factor.win32", then = [ [ "pytest" ] ] },
    ]
    "#);
}

#[test]
fn test_doc_conditional_set_env() {
    let start = indoc! {r#"
        [env_run_base]
        set_env.MATURITY = { replace = "if", condition = "env.CI", then = "release", "else" = "dev" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env.MATURITY = { replace = "if", condition = "env.CI", then = "release", else = "dev" }
    "#);
}

#[test]
fn test_doc_conditional_command_args() {
    let start = indoc! {r#"
        [env_run_base]
        commands = [["pytest", { replace = "if", condition = "env.DEBUG", then = ["-vv", "--tb=long"], "else" = [], extend = true }]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    commands = [
      [ "pytest", { replace = "if", condition = "env.DEBUG", then = [ "-vv", "--tb=long" ], else = [], extend = true } ]
    ]
    "#);
}

#[test]
fn test_doc_conditional_deps_with_else() {
    let start = indoc! {r#"
        [env_run_base]
        deps = [
            "pytest",
            { replace = "if", condition = "factor.django50", then = ["Django>=5.0,<5.1"], "else" = ["Django>=4.2,<4.3"] },
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [
      "pytest",
      { replace = "if", condition = "factor.django50", then = [ "Django>=5.0,<5.1" ], else = [ "Django>=4.2,<4.3" ] },
    ]
    "#);
}

#[test]
fn test_doc_complex_boolean_conditions() {
    let start = indoc! {r#"
        [env.deploy]
        commands = [["deploy", { replace = "if", condition = "env.CI and env.TAG_NAME != ''", then = ["--production"], "else" = ["--dry-run"], extend = true }]]

        [env_run_base]
        commands = [["pytest", { replace = "if", condition = "factor.linux and not env.CI", then = ["--numprocesses=auto"], "else" = [], extend = true }]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    commands = [
      [
        "pytest",
        { replace = "if", condition = "factor.linux and not env.CI", then = [ "--numprocesses=auto" ], else = [], extend = true },
      ],
    ]

    [env.deploy]
    commands = [
      [
        "deploy",
        { replace = "if", condition = "env.CI and env.TAG_NAME != ''", then = [ "--production" ], else = [ "--dry-run" ], extend = true },
      ],
    ]
    "#);
}

#[test]
fn test_doc_custom_pypi_with_env_fallback() {
    let start = indoc! {r#"
        [env_run_base]
        set_env.PIP_INDEX_URL = { replace = "env", name = "PIP_INDEX_URL", default = "https://my.pypi.example/simple" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env.PIP_INDEX_URL = { replace = "env", name = "PIP_INDEX_URL", default = "https://my.pypi.example/simple" }
    "#);
}

#[test]
fn test_doc_multiple_pypi_servers() {
    let start = indoc! {r#"
        [env_run_base]
        set_env.PIP_INDEX_URL = { replace = "env", name = "PIP_INDEX_URL", default = "https://primary.example/simple" }
        set_env.PIP_EXTRA_INDEX_URL = { replace = "env", name = "PIP_EXTRA_INDEX_URL", default = "https://secondary.example/simple" }
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env.PIP_EXTRA_INDEX_URL = { replace = "env", name = "PIP_EXTRA_INDEX_URL", default = "https://secondary.example/simple" }
    set_env.PIP_INDEX_URL = { replace = "env", name = "PIP_INDEX_URL", default = "https://primary.example/simple" }
    "#);
}

#[test]
fn test_doc_extras() {
    let start = indoc! {r#"
        [env_run_base]
        extras = ["testing"]

        [env.docs]
        extras = ["docs"]
        commands = [["sphinx-build", "-W", "docs", "docs/_build/html"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    extras = [ "testing" ]

    [env.docs]
    extras = [ "docs" ]
    commands = [ [ "sphinx-build", "-W", "docs", "docs/_build/html" ] ]
    "#);
}

#[test]
fn test_doc_generative_matrix_with_range() {
    let start = indoc! {r#"
        env_list = [
            "lint",
            { product = [
                { prefix = "py3", start = 12, stop = 14 },
                ["django42", "django50"],
            ] },
        ]

        [env_run_base]
        package = "skip"
        deps = [
            "pytest",
            { replace = "if", condition = "factor.django42", then = ["Django>=4.2,<4.3"] },
            { replace = "if", condition = "factor.django50", then = ["Django>=5.0,<5.1"] },
        ]
        commands = [["pytest"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      "lint",
      { product = [
        { prefix = "py3", start = 12, stop = 14 },
        [ "django42", "django50" ],
      ] },
    ]

    [env_run_base]
    package = "skip"
    deps = [
      "pytest",
      { replace = "if", condition = "factor.django42", then = [ "Django>=4.2,<4.3" ] },
      { replace = "if", condition = "factor.django50", then = [ "Django>=5.0,<5.1" ] },
    ]
    commands = [ [ "pytest" ] ]
    "#);
}

#[test]
fn test_doc_env_base_range_factors() {
    let start = indoc! {r#"
        [env_base.django]
        factors = [
            { prefix = "py3", start = 13, stop = 14 },
            ["django42", "django50"],
        ]
        package = "skip"
        deps = [
            "pytest",
            { replace = "if", condition = "factor.django42", then = ["Django>=4.2,<4.3"] },
            { replace = "if", condition = "factor.django50", then = ["Django>=5.0,<5.1"] },
        ]
        commands = [["pytest"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_base.django]
    factors = [
      { prefix = "py3", start = 13, stop = 14 },
      [ "django42", "django50" ],
    ]
    package = "skip"
    deps = [
      "pytest",
      { replace = "if", condition = "factor.django42", then = [ "Django>=4.2,<4.3" ] },
      { replace = "if", condition = "factor.django50", then = [ "Django>=5.0,<5.1" ] },
    ]
    commands = [ [ "pytest" ] ]
    "#);
}

#[test]
fn test_doc_open_ended_range_start() {
    let start = indoc! {r#"
        env_list = [
            { product = [{ prefix = "py3", start = 10 }] },
            "lint",
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      { product = [ { prefix = "py3", start = 10 } ] },
      "lint",
    ]
    "#);
}

#[test]
fn test_doc_open_ended_range_stop() {
    let start = indoc! {r#"
        env_list = [
            { product = [{ prefix = "py3", stop = 13 }] },
            "lint",
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [
      { product = [ { prefix = "py3", stop = 13 } ] },
      "lint",
    ]
    "#);
}

#[test]
fn test_doc_ignore_exit_code() {
    let start = indoc! {r#"
        [env_run_base]
        commands = [
            ["-", "python", "-c", "import sys; sys.exit(1)"],
            ["python", "--version"],
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    commands = [
      [ "-", "python", "-c", "import sys; sys.exit(1)" ],
      [ "python", "--version" ],
    ]
    "#);
}

#[test]
fn test_doc_invert_exit_code() {
    let start = indoc! {r#"
        [env_run_base]
        commands = [
            ["!", "python", "-c", "import sys; sys.exit(1)"],
            ["python", "--version"],
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    commands = [
      [ "!", "python", "-c", "import sys; sys.exit(1)" ],
      [ "python", "--version" ],
    ]
    "#);
}

#[test]
fn test_doc_sphinx_build() {
    let start = indoc! {r#"
        [env.docs]
        description = "build documentation"
        deps = ["sphinx>=7"]
        commands = [
            ["sphinx-build", "-d", "{env_tmp_dir}/doctree", "docs", "{work_dir}/docs_out", "--color", "-b", "html"],
        ]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.docs]
    description = "build documentation"
    deps = [ "sphinx>=7" ]
    commands = [
      [
        "sphinx-build",
        "-d",
        "{env_tmp_dir}/doctree",
        "docs",
        "{work_dir}/docs_out",
        "--color",
        "-b",
        "html"
      ],
    ]
    "#);
}

#[test]
fn test_doc_mkdocs() {
    let start = indoc! {r#"
        [env.docs]
        description = "run a development server for documentation"
        deps = [
            "mkdocs>=1.3",
            "mkdocs-material",
        ]
        commands = [
            ["mkdocs", "build", "--clean"],
            ["mkdocs", "serve", "-a", "localhost:8080"],
        ]

        [env.docs-deploy]
        description = "build and deploy documentation"
        deps = [
            "mkdocs>=1.3",
            "mkdocs-material",
        ]
        commands = [["mkdocs", "gh-deploy", "--clean"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.docs]
    description = "run a development server for documentation"
    deps = [
      "mkdocs>=1.3",
      "mkdocs-material",
    ]
    commands = [
      [ "mkdocs", "build", "--clean" ],
      [ "mkdocs", "serve", "-a", "localhost:8080" ],
    ]

    [env.docs-deploy]
    description = "build and deploy documentation"
    deps = [
      "mkdocs>=1.3",
      "mkdocs-material",
    ]
    commands = [ [ "mkdocs", "gh-deploy", "--clean" ] ]
    "#);
}

#[test]
fn test_doc_virtualenv_per_env() {
    let start = indoc! {r#"
        env_list = ["3.6", "3.15", "3.13"]

        [env_run_base]
        deps = ["pytest"]
        commands = [["pytest"]]

        [env."3.6"]
        virtualenv_spec = "virtualenv<20.22.0"
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [ "3.15", "3.13", "3.6" ]

    [env_run_base]
    deps = [ "pytest" ]
    commands = [ [ "pytest" ] ]

    [env."3.6"]
    virtualenv_spec = "virtualenv<20.22.0"
    "#);
}

#[test]
fn test_doc_pylock_with_extras_and_groups() {
    let start = indoc! {r#"
        [env.docs]
        pylock = "pylock.toml"
        extras = ["docs"]

        [env.dev]
        pylock = "pylock.toml"
        dependency_groups = ["dev"]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env.dev]
    dependency_groups = [ "dev" ]
    pylock = "pylock.toml"

    [env.docs]
    pylock = "pylock.toml"
    extras = [ "docs" ]
    "#);
}

#[test]
fn test_doc_clean_cache_recreate() {
    let start = indoc! {r#"
        [env_run_base]
        deps = ["pre-commit"]
        recreate_commands = [["{env_python}", "-Im", "pre_commit", "clean"]]
        commands = [["pre-commit", "run", "--all-files"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_run_base]
    deps = [ "pre-commit" ]
    recreate_commands = [ [ "{env_python}", "-Im", "pre_commit", "clean" ] ]
    commands = [ [ "pre-commit", "run", "--all-files" ] ]
    "#);
}

#[test]
fn test_doc_arch_specific_interpreters() {
    let start = indoc! {r#"
        env_list = ["arm64", "x86_64"]

        [env.arm64]
        base_python = ["cpython3.12-64-arm64"]
        commands = [["pytest"]]

        [env.x86_64]
        base_python = ["cpython3.12-64-x86_64"]
        commands = [["pytest"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [ "arm64", "x86_64" ]

    [env.arm64]
    base_python = [ "cpython3.12-64-arm64" ]
    commands = [ [ "pytest" ] ]

    [env.x86_64]
    base_python = [ "cpython3.12-64-x86_64" ]
    commands = [ [ "pytest" ] ]
    "#);
}

// --- tutorial/getting-started.rst examples ---

#[test]
fn test_doc_getting_started() {
    let start = indoc! {r#"
        env_list = ["3.13", "3.12", "lint"]

        [env_run_base]
        description = "run the test suite with pytest"
        deps = [
            "pytest>=8",
        ]
        commands = [["pytest", { replace = "posargs", default = ["tests"], extend = true }]]

        [env.lint]
        description = "run linters"
        skip_install = true
        deps = ["ruff"]
        commands = [["ruff", "check", { replace = "posargs", default = ["."], extend = true }]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    env_list = [ "3.13", "3.12", "lint" ]

    [env_run_base]
    description = "run the test suite with pytest"
    deps = [
      "pytest>=8",
    ]
    commands = [ [ "pytest", { replace = "posargs", default = [ "tests" ], extend = true } ] ]

    [env.lint]
    description = "run linters"
    skip_install = true
    deps = [ "ruff" ]
    commands = [ [ "ruff", "check", { replace = "posargs", default = [ "." ], extend = true } ] ]
    "#);
}

#[test]
fn test_doc_full_tox_toml_structure() {
    let start = indoc! {r#"
        # tox.toml - values at root level are core settings
        requires = ["tox>=4.20"]
        env_list = ["3.13", "3.12", "lint"]

        # base settings for run environments
        [env_run_base]
        deps = ["pytest>=8"]
        commands = [["pytest", "tests"]]

        # environment-specific overrides
        [env.lint]
        skip_install = true
        deps = ["ruff"]
        commands = [["ruff", "check", "."]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    # tox.toml - values at root level are core settings
    requires = [ "tox>=4.20" ]
    env_list = [ "3.13", "3.12", "lint" ]

    # base settings for run environments
    [env_run_base]
    deps = [ "pytest>=8" ]
    commands = [ [ "pytest", "tests" ] ]

    # environment-specific overrides
    [env.lint]
    skip_install = true
    deps = [ "ruff" ]
    commands = [ [ "ruff", "check", "." ] ]
    "#);
}

#[test]
fn test_doc_env_base_tutorial() {
    let start = indoc! {r#"
        [env_base.test]
        factors = [["3.13", "3.14"]]
        deps = ["pytest>=8"]
        commands = [["pytest"]]
        "#};
    let got = format_toml_helper(start, 2);
    assert_snapshot!(got, @r#"
    [env_base.test]
    factors = [ [ "3.13", "3.14" ] ]
    deps = [ "pytest>=8" ]
    commands = [ [ "pytest" ] ]
    "#);
}
