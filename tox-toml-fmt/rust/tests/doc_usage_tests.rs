//! The examples `docs/how-to/usage.rst` shows, formatted the way the docs print them.

use indoc::indoc;
use insta::assert_snapshot;

use super::format_doc_example;

#[test]
fn test_doc_basic_pytest() {
    let start = indoc! {r#"
        env_list = ["3.13", "3.12"]

        [env_run_base]
        deps = ["pytest>=8"]
        commands = [["pytest", { replace = "posargs", default = ["tests"], extend = true }]]
        "#};
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
    assert_snapshot!(got, @r#"
    [env_run_base]
    set_env.PIP_INDEX_URL = { replace = "env", name = "PIP_INDEX_URL", default = "https://primary.example/simple" }
    set_env.PIP_EXTRA_INDEX_URL = { replace = "env", name = "PIP_EXTRA_INDEX_URL", default = "https://secondary.example/simple" }
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
    assert_snapshot!(got, @r#"
    [env.docs]
    description = "build documentation"
    deps = [ "sphinx>=7" ]
    commands = [
      [ "sphinx-build", "-d", "{env_tmp_dir}/doctree", "docs", "{work_dir}/docs_out", "--color", "-b", "html" ],
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
    let got = format_doc_example(start);
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
