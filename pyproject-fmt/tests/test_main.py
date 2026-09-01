from __future__ import annotations

import sys
from pathlib import Path
from textwrap import dedent

if sys.version_info >= (3, 11):  # pragma: >=3.11 cover
    import tomllib
else:  # pragma: <3.11 cover
    import tomli as tomllib

import pytest

from pyproject_fmt import build_parser, run


def test_build_parser_uses_program_name() -> None:
    assert build_parser().prog == "pyproject-fmt"


@pytest.mark.parametrize(
    "in_place",
    [
        True,
        False,
    ],
    ids=("in_place", "print"),
)
@pytest.mark.parametrize(
    "check",
    [
        True,
        False,
    ],
    ids=["check", "no_check"],
)
@pytest.mark.parametrize(
    "cwd",
    [
        True,
        False,
    ],
    ids=["cwd", "absolute"],
)
@pytest.mark.parametrize(
    ("start", "outcome", "output"),
    [
        pytest.param(
            '[build-system]\nrequires = [\n  "hatchling>=0.14",\n]\n',
            '[build-system]\nrequires = [\n  "hatchling>=0.14",\n]\n',
            "no change for {0}\n",
            id="formatted",
        ),
        pytest.param(
            '[build-system]\nrequires = ["hatchling>=0.14.0"]',
            '[build-system]\nrequires = [ "hatchling>=0.14" ]\n',
            "--- {0}\n\n+++ {0}\n\n@@ -1,2 +1,2 @@\n\n [build-system]\n-requires = "
            '["hatchling>=0.14.0"]\n+requires = [ "hatchling>=0.14" ]\n',
            id="format",
        ),
    ],
)
def test_main(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    in_place: bool,
    start: str,
    outcome: str,
    output: str,
    monkeypatch: pytest.MonkeyPatch,
    cwd: bool,
    check: bool,
) -> None:
    monkeypatch.setenv("NO_COLOR", "1")
    if cwd:
        monkeypatch.chdir(tmp_path)
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(start)
    args = [str(pyproject_toml)]
    if not in_place:
        args.append("--stdout")

    if check:
        args.append("--check")

        if not in_place:
            with pytest.raises(SystemExit):
                run(args)
            assert pyproject_toml.read_text() == start
            return

    result = run(args)
    assert result == (0 if start == outcome else 1)

    out, err = capsys.readouterr()
    assert not err

    if check:
        assert pyproject_toml.read_text() == start
    elif in_place:
        name = "pyproject.toml" if cwd else str(tmp_path / "pyproject.toml")
        output = output.format(name)
        assert pyproject_toml.read_text() == outcome
        assert out == output
    else:
        assert out == outcome


@pytest.mark.parametrize("indent", [0, 2, 4])
def test_indent(tmp_path: Path, indent: int) -> None:
    start = """\
    [build-system]
    requires = [
        "A",
    ]
    """

    expected = f"""\
    [build-system]
    requires = [
    {" " * indent}"a",
    ]
    """
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(dedent(start))
    args = [str(pyproject_toml), "--indent", str(indent)]
    run(args)
    output = pyproject_toml.read_text()
    assert output == dedent(expected)


@pytest.mark.parametrize(
    ("flag", "message"),
    [
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
        pytest.param('max_supported_python = "3.9"', "must not precede 3.10", id="maximum-below-the-minimum"),
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
    start = """\
    [build-system]
    requires = [
      "a==1.0.0",
    ]

    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.10",
    ]
    dependencies = [
      "a==1.0.0",
    ]
    optional-dependencies.docs = [
      "b==2.0.0",
    ]
    """
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text(dedent(start))
    args = [str(pyproject_toml), "--keep-full-version", "--max-supported-python", "3.10"]
    run(args)
    output = pyproject_toml.read_text()
    assert output == dedent(start)


def test_pyproject_toml_config(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
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
    filename = tmp_path / "pyproject.toml"
    filename.write_text(dedent(txt))
    run([str(filename)])

    expected = """\
    [project]
    keywords = [
        "A",
    ]
    classifiers = [
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Python :: 3.10",
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
    got = filename.read_text()
    assert got == dedent(expected)
    out, err = capsys.readouterr()
    assert out
    assert not err


def test_pyproject_fmt_api_changed(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    txt = """
    [project]
    requires-python = "==3.12"
    """
    filename = tmp_path / "pyproject.toml"
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
    txt = """\
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    """
    filename = tmp_path / "pyproject.toml"
    filename.write_text(dedent(txt))
    res = run([str(filename), "--no-print-diff"])

    assert res == 0

    got = filename.read_text()

    assert got == dedent(txt)

    out, err = capsys.readouterr()
    assert not out
    assert not err


def test_no_generate_python_version_classifiers(tmp_path: Path) -> None:
    txt = """\
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    """
    filename = tmp_path / "pyproject.toml"
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
    """Test that --table-format long expands sub-tables to [table.subtable] format."""
    txt = """\
    [project]
    name = "myproject"
    urls.homepage = "https://example.com"
    urls.repository = "https://github.com/example"
    """
    filename = tmp_path / "pyproject.toml"
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
    """Test that --table-format short collapses [table.subtable] to dotted keys."""
    txt = """\
    [project]
    name = "myproject"

    [project.urls]
    homepage = "https://example.com"
    repository = "https://github.com/example"
    """
    filename = tmp_path / "pyproject.toml"
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
    """Test that table_format can be configured via pyproject.toml."""
    txt = """\
    [project]
    name = "myproject"
    urls.homepage = "https://example.com"

    [tool.pyproject-fmt]
    table_format = "long"
    """
    filename = tmp_path / "pyproject.toml"
    filename.write_text(dedent(txt))
    res = run([str(filename), "--no-generate-python-version-classifiers"])

    assert res == 1

    got = filename.read_text()
    # Verify sub-tables are expanded
    assert "[project.urls]" in got
    assert "homepage =" in got


def test_expand_tables_override(tmp_path: Path) -> None:
    """Test that --expand-tables overrides the default table format."""
    txt = """\
    [project]
    name = "myproject"
    urls.homepage = "https://example.com"
    scripts.main = "pkg:main"
    """
    filename = tmp_path / "pyproject.toml"
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
    """Test that --collapse-tables overrides expand-tables."""
    txt = """\
    [project]
    name = "myproject"

    [project.urls]
    homepage = "https://example.com"
    """
    filename = tmp_path / "pyproject.toml"
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
    """The tool.pyproject-fmt table is key-ordered and its list values sorted and deduplicated."""
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
    filename = tmp_path / "pyproject.toml"
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
    txt = """\
    [project]
    version = "1.9.xyz"
    """
    filename = tmp_path / "pyproject.toml"
    filename.write_text(dedent(txt))

    assert run([str(filename)]) == 1

    assert filename.read_text() == dedent(txt)
    out, err = capsys.readouterr()
    assert not out
    assert err == f"{filename}: project.version `1.9.xyz` is not a valid PEP 440 version\n"


def test_project_version_kept_verbatim(tmp_path: Path) -> None:
    txt = """\
    [project]
    version = "2026.08.10"
    """
    filename = tmp_path / "pyproject.toml"
    filename.write_text(dedent(txt))

    assert run([str(filename), "--no-print-diff", "--no-generate-python-version-classifiers"]) == 0

    assert filename.read_text() == dedent(txt)


def test_settings_are_read_beside_a_value_only_toml_1_1_reads(tmp_path: Path) -> None:
    """A file the formatter reads is one its own settings are read from, TOML 1.1 values included."""
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
    """Reading the settings must reach as far into a document as the formatter does."""
    pyproject_toml = tmp_path / "pyproject.toml"
    start = f"a = {'[' * 200}1{']' * 200}\n"
    pyproject_toml.write_text(start)

    assert run([str(pyproject_toml)]) == 1
    assert pyproject_toml.read_text().startswith("a = [\n  [\n")


def test_a_value_nested_deeper_than_the_formatter_reads(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    """A value past what a stack holds is reported, and the file is left as its author wrote it."""
    pyproject_toml = tmp_path / "pyproject.toml"
    start = f"a = {'[' * 12_000}1{']' * 12_000}\n"
    pyproject_toml.write_text(start)

    assert run([str(pyproject_toml)]) == 1

    assert "nested deeper" in capsys.readouterr().err
    assert pyproject_toml.read_text() == start


def test_every_python_environment_runs_the_tests() -> None:
    """A Python environment that defines no command of its own passes without running a test."""
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
    """TOML maps dotted and explicit table spellings to one path; settings follow that path."""
    pyproject_toml = tmp_path / "pyproject.toml"
    # a dotted key writes its own table, so it stands before the first header rather than under it
    pyproject_toml.write_text(
        f'{settings}\n\n[project]\nname = "demo"\ndescription = "one two three four five six seven"\n'
    )

    run([str(pyproject_toml)])

    assert '"""\\' in pyproject_toml.read_text()


def test_settings_under_a_repeated_table_are_no_settings(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    """A table the file repeats is a list of them, and what its elements write is no one table."""
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
    """Report the first invalid setting in source order."""
    pyproject_toml = tmp_path / "pyproject.toml"
    pyproject_toml.write_text('[project]\nname = "x"\n\n[tool.pyproject-fmt]\nz_bad = 1\na_bad = 2\n')

    with pytest.raises(SystemExit):
        run([str(pyproject_toml)])

    assert "z_bad: unknown setting" in capsys.readouterr().err


def test_a_selector_that_names_no_table(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    """A setting that names a table TOML cannot read asks for nothing, and is told so."""
    pyproject_toml = tmp_path / "pyproject.toml"
    start = '[project]\nname = "x"\n\n[tool.pyproject-fmt]\nexpand_tables = [ "project.\\"urls" ]\n'
    pyproject_toml.write_text(start)

    assert run([str(pyproject_toml)]) == 1

    assert 'expand_tables: project."urls is not a table name' in capsys.readouterr().err
    assert pyproject_toml.read_text() == start


@pytest.mark.parametrize("through", ["cli", "file", "neither"], ids=["command-line", "configuration", "control"])
def test_a_quoted_selector_names_the_same_table_either_way(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    through: str,
) -> None:
    """A name TOML quotes may hold a comma of its own, which separates nothing."""
    pyproject_toml = tmp_path / "pyproject.toml"
    held = '[project]\nname = "My_Package"\n\n[tool."a,b".child]\nx = 1\n'
    pyproject_toml.write_text(held)
    if through == "cli":
        assert run([str(pyproject_toml), "--no-print-diff", "--expand-tables", 'tool."a,b"']) == 1
    elif through == "file":
        pyproject_toml.write_text(f"{held}\n[tool.pyproject-fmt]\nexpand_tables = [ 'tool.\"a,b\"' ]\n")
        assert run([str(pyproject_toml), "--no-print-diff"]) == 1
    else:
        assert run([str(pyproject_toml), "--no-print-diff"]) == 1

    assert not capsys.readouterr().err
    written = pyproject_toml.read_text()
    # the formatter ran: the project name is written the way a distribution name is spelled
    assert 'name = "my-package"' in written
    # the selector is what holds the table open, so without one it folds into its parent
    assert ('[tool."a,b".child]' in written) is (through != "neither")
    assert ('[tool."a,b"]' in written) is (through == "neither")
