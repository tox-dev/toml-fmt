from __future__ import annotations

import tomllib
from pathlib import Path
from textwrap import dedent
from typing import Final

import pytest
from trove_classifiers import classifiers

from pyproject_fmt import build_parser, format_example, run


def test_build_parser_uses_program_name() -> None:
    assert build_parser().prog == "pyproject-fmt"


def test_the_default_max_supported_python_names_a_published_classifier() -> None:
    # PyPI turns away an upload naming a classifier it does not publish, so the default writes none.
    major, minor = build_parser().get_default("max_supported_python")

    assert f"Programming Language :: Python :: {major}.{minor}" in classifiers


def test_the_default_max_supported_python_trails_the_newest_classifier_by_one() -> None:
    # A classifier is published while its release is still in beta, so the newest one runs a release
    # ahead of the newest that ships. Two ahead says a release has shipped that the default leaves out,
    # which is what asks for the bump.
    major, minor = build_parser().get_default("max_supported_python")
    published = max(
        int(held)
        for name in classifiers
        if (held := name.removeprefix(f"Programming Language :: Python :: {major}.")).isdigit()
        and name.startswith(f"Programming Language :: Python :: {major}.")
    )

    assert published - minor <= 1, (
        f"Python {major}.{published} has a classifier, so the default may name {major}.{published - 1}"
    )


_FORMATTED: Final[str] = '[build-system]\nrequires = [\n  "hatchling>=0.14",\n]\n'
_UNFORMATTED: Final[str] = '[build-system]\nrequires = ["hatchling>=0.14.0"]'
_REFORMATTED: Final[str] = '[build-system]\nrequires = [ "hatchling>=0.14" ]\n'
_CASES: Final[list[pytest.ParameterSet]] = [
    pytest.param(_FORMATTED, _FORMATTED, "no change for {0}\n", 0, id="formatted"),
    pytest.param(
        _UNFORMATTED,
        _REFORMATTED,
        "--- {0}\n\n+++ {0}\n\n@@ -1,2 +1,2 @@\n\n [build-system]\n-requires = "
        '["hatchling>=0.14.0"]\n+requires = [ "hatchling>=0.14" ]\n',
        1,
        id="format",
    ),
]


def test_format_example_uses_docs_settings() -> None:
    assert (
        format_example(
            _UNFORMATTED,
            {
                "column_width": 120,
                "indent": 2,
                "keep_full_version": False,
                "max_supported_python": (3, 15),
                "min_supported_python": (3, 10),
                "generate_python_version_classifiers": True,
                "table_format": "short",
                "sub_table_spacing": "",
                "separate_root_table": "\n",
                "expand_tables": (),
                "collapse_tables": (),
                "skip_wrap_for_keys": (),
            },
        )
        == _REFORMATTED
    )


@pytest.mark.parametrize(("start", "outcome", "output", "exit_code"), _CASES)
def test_main_in_place(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    start: str,
    outcome: str,
    output: str,
    exit_code: int,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("NO_COLOR", "1")
    monkeypatch.chdir(tmp_path)
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(start)

    assert run(["pyproject.toml"]) == exit_code

    out, err = capsys.readouterr()
    assert not err
    assert pyproject_toml.read_text() == outcome
    assert out == output.format("pyproject.toml")


@pytest.mark.parametrize(("start", "outcome", "output", "exit_code"), _CASES)
def test_main_in_place_shows_an_absolute_path_outside_cwd(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    start: str,
    outcome: str,
    output: str,
    exit_code: int,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("NO_COLOR", "1")
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(start)

    assert run([str(pyproject_toml)]) == exit_code

    out, err = capsys.readouterr()
    assert not err
    assert pyproject_toml.read_text() == outcome
    assert out == output.format(pyproject_toml)


@pytest.mark.parametrize(
    ("start", "exit_code"),
    [pytest.param(_FORMATTED, 0, id="formatted"), pytest.param(_UNFORMATTED, 1, id="format")],
)
def test_main_check_leaves_the_file_unchanged(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    start: str,
    exit_code: int,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("NO_COLOR", "1")
    monkeypatch.chdir(tmp_path)
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(start)

    assert run(["pyproject.toml", "--check"]) == exit_code

    _, err = capsys.readouterr()
    assert not err
    assert pyproject_toml.read_text() == start


@pytest.mark.parametrize("start", [pytest.param(_FORMATTED, id="formatted"), pytest.param(_UNFORMATTED, id="format")])
def test_main_check_refuses_stdout(tmp_path: Path, start: str) -> None:
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(start)

    with pytest.raises(SystemExit):
        run([str(pyproject_toml), "--stdout", "--check"])
    assert pyproject_toml.read_text() == start


@pytest.mark.parametrize(
    ("start", "outcome", "exit_code"),
    [
        pytest.param(_FORMATTED, _FORMATTED, 0, id="formatted"),
        pytest.param(_UNFORMATTED, _REFORMATTED, 1, id="format"),
    ],
)
def test_main_prints_to_stdout(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    start: str,
    outcome: str,
    exit_code: int,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("NO_COLOR", "1")
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(start)

    assert run([str(pyproject_toml), "--stdout"]) == exit_code

    out, err = capsys.readouterr()
    assert not err
    assert out == outcome


@pytest.mark.parametrize("indent", [pytest.param(0, id="zero"), pytest.param(2, id="two"), pytest.param(4, id="four")])
def test_indent(tmp_path: Path, indent: int) -> None:

    expected = f"""\
    [build-system]
    requires = [
    {" " * indent}"a",
    ]
    """
    pyproject_toml = tmp_path / "pyproject.toml"
    start = """\
    [build-system]
    requires = [
        "A",
    ]
    """
    pyproject_toml.write_text(dedent(start))
    run([str(pyproject_toml), "--indent", str(indent)])
    output = pyproject_toml.read_text()
    assert output == dedent(expected)


@pytest.mark.parametrize(
    ("flag", "message"),
    [
        pytest.param("--max-supported-python=3.10", "must not precede 3.11", id="maximum-below-the-minimum"),
        pytest.param("--max-supported-python=4.0", "must name a Python 3 minor", id="another-major"),
        pytest.param("--max-supported-python=3.256", "must name a Python 3 minor", id="minor-too-large"),
        pytest.param("--indent=-1", "must not be negative", id="negative-indent"),
        pytest.param("--column-width=-1", "must not be negative", id="negative-width"),
    ],
)
def test_a_setting_the_formatter_cannot_hold(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    flag: str,
    message: str,
) -> None:
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text('[project]\nname = "x"\n')

    with pytest.raises(SystemExit):
        run([str(pyproject_toml), flag])

    assert message in capsys.readouterr().err


@pytest.mark.parametrize(
    ("setting", "message"),
    [
        pytest.param("indent = -1", "must not be negative", id="negative-count"),
        pytest.param('table_format = "wide"', "invalid choice", id="unknown-table-format"),
        pytest.param("column_width = true", "is not written as int", id="boolean-count"),
        pytest.param('keep_full_version = "false"', "is not written as bool", id="text-flag"),
        pytest.param("expand_tables = 1", "is not written as list", id="number-list"),
        pytest.param("sub_table_spacing = 1", "is not written as str", id="number-spacing"),
        pytest.param("skip_wrap_for_keys = [ 1 ]", "every entry names one thing", id="list-of-numbers"),
        pytest.param("colm_width = 80", "unknown setting", id="misspelled"),
        pytest.param("check = true", "unknown setting", id="run-mode"),
        pytest.param("column_width = 100000", "must be at most", id="count-beyond-a-line"),
        pytest.param('max_supported_python = "3.10"', "must not precede 3.11", id="maximum-below-the-minimum"),
        pytest.param("[tool.pyproject-fmt.deeper]\nheld = 1", "unknown setting", id="table-below-the-settings"),
    ],
)
def test_a_configured_setting_the_formatter_cannot_hold(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    setting: str,
    message: str,
) -> None:
    pyproject_toml = tmp_path / "pyproject.toml"
    start = f'[project]\nname = "x"\n\n[tool.pyproject-fmt]\n{setting}\n'
    pyproject_toml.write_text(start)

    with pytest.raises(SystemExit):
        run([str(pyproject_toml)])

    assert message in capsys.readouterr().err
    assert pyproject_toml.read_text() == start


def test_a_carriage_return_toml_does_not_read(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    pyproject_toml = tmp_path / "pyproject.toml"
    start = b'[project]\rname = "x"\n'
    pyproject_toml.write_bytes(start)

    assert run([str(pyproject_toml)]) == 1

    assert "carriage return" in capsys.readouterr().err
    assert pyproject_toml.read_bytes() == start


def test_keep_full_version_cli(tmp_path: Path) -> None:
    pyproject_toml = tmp_path / "pyproject.toml"
    start = """\
    [build-system]
    requires = [
      "a==1.0.0",
    ]

    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.11",
    ]
    dependencies = [
      "a==1.0.0",
    ]
    optional-dependencies.docs = [
      "b==2.0.0",
    ]
    """
    pyproject_toml.write_text(dedent(start).replace('"a==1.0.0"', '"A==1.0.0"'))
    run([str(pyproject_toml), "--keep-full-version", "--max-supported-python", "3.11"])
    output = pyproject_toml.read_text()
    assert output == dedent(start)


def test_pyproject_toml_config(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    filename = tmp_path / "pyproject.toml"
    txt = """
    [project]
    keywords = [
      "A",
    ]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
    ]
    dynamic = [
      "B",
    ]
    dependencies = [
      "requests>=2.0",
    ]

    [tool.pyproject-fmt]
    column_width = 120
    indent = 4
    keep_full_version = true
    max_supported_python = "3.11"
    """
    filename.write_text(dedent(txt))
    run([str(filename)])

    got = filename.read_text()
    expected = """\
    [project]
    keywords = [
        "A",
    ]
    classifiers = [
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Python :: 3.11",
    ]
    dynamic = [
        "B",
    ]
    dependencies = [
        "requests>=2.0",
    ]

    [tool.pyproject-fmt]
    column_width = 120
    indent = 4
    keep_full_version = true
    max_supported_python = "3.11"
    """
    assert got == dedent(expected)
    out, err = capsys.readouterr()
    assert out
    assert not err


def test_pyproject_fmt_api_changed(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    filename = tmp_path / "pyproject.toml"
    txt = """
    [project]
    requires-python = "==3.12"
    """
    filename.write_text(dedent(txt))
    res = run([str(filename), "--no-print-diff", "--column-width", "120"])

    assert res == 1

    got = filename.read_text()
    expected = """\
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    """
    assert got == dedent(expected)

    out, err = capsys.readouterr()
    assert not out
    assert not err


def test_pyproject_fmt_api_no_change(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    filename = tmp_path / "pyproject.toml"
    txt = """\
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    """
    filename.write_text(dedent(txt))
    res = run([str(filename), "--no-print-diff"])

    assert res == 0

    got = filename.read_text()

    assert got == dedent(txt)

    out, err = capsys.readouterr()
    assert not out
    assert not err


def test_no_generate_python_version_classifiers(tmp_path: Path) -> None:
    filename = tmp_path / "pyproject.toml"
    txt = """\
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    """
    filename.write_text(dedent(txt))
    res = run([str(filename), "--no-print-diff", "--no-generate-python-version-classifiers"])

    assert res == 0

    got = filename.read_text()

    expected = """\
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    """
    assert got == dedent(expected)


def test_table_format_long_expands_sub_tables(tmp_path: Path) -> None:
    # Test that --table-format long expands sub-tables to [table.subtable] format.
    filename = tmp_path / "pyproject.toml"
    txt = """\
    [project]
    name = "myproject"
    urls.homepage = "https://example.com"
    urls.repository = "https://github.com/example"
    """
    filename.write_text(dedent(txt))
    res = run([str(filename), "--table-format", "long", "--no-generate-python-version-classifiers"])

    assert res == 1

    got = filename.read_text()
    # Verify sub-tables are expanded
    assert "[project.urls]" in got
    # Verify dotted keys are removed
    assert "urls.homepage =" not in got
    assert "homepage =" in got
    assert "repository =" in got


def test_table_format_short_collapses_sub_tables(tmp_path: Path) -> None:
    # Test that --table-format short collapses [table.subtable] to dotted keys.
    filename = tmp_path / "pyproject.toml"
    txt = """\
    [project]
    name = "myproject"

    [project.urls]
    homepage = "https://example.com"
    repository = "https://github.com/example"
    """
    filename.write_text(dedent(txt))
    res = run([str(filename), "--table-format", "short", "--no-generate-python-version-classifiers"])

    assert res == 1

    got = filename.read_text()
    # Verify sub-tables are collapsed
    assert "urls.homepage =" in got
    assert "urls.repository =" in got
    # Verify expanded tables are removed
    assert "[project.urls]" not in got


def test_table_format_config_in_pyproject_toml(tmp_path: Path) -> None:
    # Test that table_format can be configured via pyproject.toml.
    filename = tmp_path / "pyproject.toml"
    txt = """\
    [project]
    name = "myproject"
    urls.homepage = "https://example.com"

    [tool.pyproject-fmt]
    table_format = "long"
    """
    filename.write_text(dedent(txt))
    res = run([str(filename), "--no-generate-python-version-classifiers"])

    assert res == 1

    got = filename.read_text()
    # Verify sub-tables are expanded
    assert "[project.urls]" in got
    assert "homepage =" in got


def test_expand_tables_override(tmp_path: Path) -> None:
    # Test that --expand-tables overrides the default table format.
    filename = tmp_path / "pyproject.toml"
    txt = """\
    [project]
    name = "myproject"
    urls.homepage = "https://example.com"
    scripts.main = "pkg:main"
    """
    filename.write_text(dedent(txt))
    # Use short format but expand project tables
    res = run([
        str(filename),
        "--table-format",
        "short",
        "--expand-tables",
        "project",
        "--no-generate-python-version-classifiers",
    ])

    assert res == 1

    got = filename.read_text()
    # Verify sub-tables are expanded despite short format
    assert "[project.urls]" in got or "[project.scripts]" in got


def test_collapse_tables_override(tmp_path: Path) -> None:
    filename = tmp_path / "pyproject.toml"
    txt = """\
    [project]
    name = "myproject"

    [project.urls]
    homepage = "https://example.com"
    """
    filename.write_text(dedent(txt))
    # Use long format, set expand, but collapse overrides
    res = run([
        str(filename),
        "--table-format",
        "long",
        "--expand-tables",
        "project",
        "--collapse-tables",
        "project",
        "--no-generate-python-version-classifiers",
    ])

    assert res == 1

    got = filename.read_text()
    # Verify sub-tables are collapsed due to collapse override
    assert "urls.homepage =" in got
    assert "[project.urls]" not in got


def test_pyproject_fmt_self_config_normalized(tmp_path: Path) -> None:
    # The tool.pyproject-fmt table is key-ordered and its list values sorted and deduplicated.
    filename = tmp_path / "pyproject.toml"
    txt = """\
    [project]
    name = "myproject"

    [tool.pyproject-fmt]
    skip_wrap_for_keys = ["z.parse", "a.parse", "a.parse"]
    indent = 2
    column_width = 120
    expand_tables = ["tool.ruff", "tool.black", "tool.ruff"]
    keep_full_version = true
    """
    filename.write_text(dedent(txt))
    res = run([str(filename), "--no-generate-python-version-classifiers"])

    assert res == 1

    expected = """\
    [project]
    name = "myproject"

    [tool.pyproject-fmt]
    column_width = 120
    indent = 2
    keep_full_version = true
    expand_tables = [ "tool.black", "tool.ruff" ]
    skip_wrap_for_keys = [ "a.parse", "z.parse" ]
    """
    assert filename.read_text() == dedent(expected)


def test_invalid_project_version(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    filename = tmp_path / "pyproject.toml"
    txt = """\
    [project]
    version = "1.9.xyz"
    """
    filename.write_text(dedent(txt))

    assert run([str(filename)]) == 1

    assert filename.read_text() == dedent(txt)
    out, err = capsys.readouterr()
    assert not out
    assert err == f"{filename}: project.version `1.9.xyz` is not a valid PEP 440 version\n"


def test_project_version_kept_verbatim(tmp_path: Path) -> None:
    filename = tmp_path / "pyproject.toml"
    txt = """\
    [project]
    version = "2026.08.10"
    """
    filename.write_text(dedent(txt))

    assert run([str(filename), "--no-print-diff", "--no-generate-python-version-classifiers"]) == 0

    assert filename.read_text() == dedent(txt)


def test_settings_are_read_beside_a_value_only_toml_1_1_reads(tmp_path: Path) -> None:
    # A file the formatter reads is one its own settings are read from, TOML 1.1 values included.
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(
        'when = 12:30\n\n[project]\nname = "demo"\ndescription = "one two three four five six seven"\n'
        "\n[tool.pyproject-fmt]\ncolumn_width = 30\n"
    )

    run([str(pyproject_toml)])

    assert '"""\\' in pyproject_toml.read_text()


def test_a_setting_beside_a_value_only_toml_1_1_reads_is_still_checked(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    pyproject_toml = tmp_path / "pyproject.toml"
    start = 'when = 12:30\n\n[project]\nname = "demo"\n\n[tool.pyproject-fmt]\ncolumn_width = "wide"\n'
    pyproject_toml.write_text(start)

    with pytest.raises(SystemExit):
        run([str(pyproject_toml)])

    assert "is not written as int" in capsys.readouterr().err
    assert pyproject_toml.read_text() == start


def test_settings_are_read_from_a_file_that_opens_with_a_byte_order_mark(tmp_path: Path) -> None:
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(
        '﻿[project]\nname = "demo"\ndescription = "one two three four five six seven"\n'
        "\n[tool.pyproject-fmt]\ncolumn_width = 30\n",
        encoding="utf-8",
    )

    run([str(pyproject_toml)])

    assert '"""\\' in pyproject_toml.read_text()


def test_a_value_nested_deeper_than_a_python_reader_goes(tmp_path: Path) -> None:
    # Reading the settings must reach as far into a document as the formatter does.
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(f"a = {'[' * 200}1{']' * 200}\n")

    assert run([str(pyproject_toml)]) == 1
    assert pyproject_toml.read_text().startswith("a = [\n  [\n")


def test_a_value_nested_deeper_than_the_formatter_reads(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # A value past what a stack holds is reported, and the file is left as its author wrote it.
    pyproject_toml = tmp_path / "pyproject.toml"
    start = f"a = {'[' * 12_000}1{']' * 12_000}\n"
    pyproject_toml.write_text(start)

    assert run([str(pyproject_toml)]) == 1

    assert "nested deeper" in capsys.readouterr().err
    assert pyproject_toml.read_text() == start


def test_every_python_environment_runs_the_tests() -> None:
    # A Python environment that defines no command of its own passes without running a test.
    config = tomllib.loads((Path(__file__).parent.parent / "tox.toml").read_text(encoding="utf-8"))

    assert config["env_run_base"]["commands"][0][0] == "pytest"
    named = [name for name in config["env_list"] if name[0].isdigit()]
    assert named
    for name in named:
        assert "commands" not in config.get("env", {}).get(name, {}), name


@pytest.mark.parametrize(
    "settings",
    [
        pytest.param("[tool.pyproject-fmt]\ncolumn_width = 30", id="a-header"),
        pytest.param("tool.pyproject-fmt.column_width = 30", id="a-dotted-name"),
        pytest.param("[tool]\npyproject-fmt = { column_width = 30 }", id="a-table-written-as-a-value"),
        pytest.param('[tool]\npyproject-fmt = { "column_width" = 30 }', id="a-name-in-quotes"),
        pytest.param("tool = { pyproject-fmt = { column_width = 30 } }", id="a-table-inside-a-value"),
    ],
)
def test_a_setting_is_read_however_the_file_writes_its_table(tmp_path: Path, settings: str) -> None:
    # TOML maps dotted and explicit table spellings to one path; settings follow that path.
    pyproject_toml = tmp_path / "pyproject.toml"
    # a dotted key writes its own table, so it stands before the first header rather than under it
    pyproject_toml.write_text(
        f'{settings}\n\n[project]\nname = "demo"\ndescription = "one two three four five six seven"\n'
    )

    run([str(pyproject_toml)])

    assert '"""\\' in pyproject_toml.read_text()


def test_settings_under_a_repeated_table_are_no_settings(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # A table the file repeats is a list of them, and what its elements write is no one table.
    pyproject_toml = tmp_path / "pyproject.toml"
    start = '[project]\nname = "x"\n\n[[tool]]\npyproject-fmt = { column_width = 30 }\n'
    pyproject_toml.write_text(start)

    with pytest.raises(SystemExit):
        run([str(pyproject_toml)])

    assert "an array of tables holds no settings" in capsys.readouterr().err
    assert pyproject_toml.read_text() == start


def test_the_first_setting_the_formatter_cannot_hold_is_the_one_reported(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    # Report the first invalid setting in source order.
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text('[project]\nname = "x"\n\n[tool.pyproject-fmt]\nz_bad = 1\na_bad = 2\n')

    with pytest.raises(SystemExit):
        run([str(pyproject_toml)])

    assert "z_bad: unknown setting" in capsys.readouterr().err


def test_a_selector_that_names_no_table(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # A setting that names a table TOML cannot read asks for nothing, and is told so.
    pyproject_toml = tmp_path / "pyproject.toml"
    start = '[project]\nname = "x"\n\n[tool.pyproject-fmt]\nexpand_tables = [ "project.\\"urls" ]\n'
    pyproject_toml.write_text(start)

    assert run([str(pyproject_toml)]) == 1

    assert 'expand_tables: project."urls is not a table name' in capsys.readouterr().err
    assert pyproject_toml.read_text() == start


@pytest.mark.parametrize(
    ("config", "args", "child"),
    [
        pytest.param("\n[tool.pyproject-fmt]\nexpand_tables = [ 'tool.\"a,b\"' ]\n", [], True, id="configuration"),
        pytest.param("", ["--expand-tables", 'tool."a,b"'], True, id="command-line"),
        pytest.param("", [], False, id="control"),
    ],
)
def test_a_quoted_selector_names_the_same_table_either_way(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    config: str,
    args: list[str],
    child: bool,
) -> None:
    # A name TOML quotes may hold a comma of its own, which separates nothing.
    pyproject_toml = tmp_path / "pyproject.toml"
    held = '[project]\nname = "My_Package"\n\n[tool."a,b".child]\nx = 1\n'
    pyproject_toml.write_text(f"{held}{config}")
    assert run([str(pyproject_toml), "--no-print-diff", *args]) == 1

    assert not capsys.readouterr().err
    written = pyproject_toml.read_text()
    # the formatter ran: the project name is written the way a distribution name is spelled
    assert 'name = "my-package"' in written
    # the selector is what holds the table open, so without one it folds into its parent
    assert ('[tool."a,b".child]' in written) is child
    assert ('[tool."a,b"]' in written) is (not child)
