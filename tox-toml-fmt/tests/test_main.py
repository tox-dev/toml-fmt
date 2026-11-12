from __future__ import annotations

import tomllib
from pathlib import Path
from textwrap import dedent
from typing import Final

import pytest

from tox_toml_fmt import build_parser, format_example, run


def test_build_parser_uses_program_name() -> None:
    assert build_parser().prog == "tox-toml-fmt"


_FORMATTED: Final[str] = 'requires = [ "tox>=4.22" ]\n'
_UNFORMATTED: Final[str] = "requires = ['tox>=4.22']\n"
_CASES: Final[list[pytest.ParameterSet]] = [
    pytest.param(_FORMATTED, _FORMATTED, "no change for {0}\n", 0, id="formatted"),
    pytest.param(
        _UNFORMATTED,
        _FORMATTED,
        "--- {0}\n\n+++ {0}\n\n@@ -1 +1 @@\n\n-requires = ['tox>=4.22']\n+requires = [ \"tox>=4.22\" ]\n",
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
                "table_format": "short",
                "sub_table_spacing": "",
                "separate_root_table": "\n",
                "expand_tables": (),
                "collapse_tables": (),
                "skip_wrap_for_keys": (),
                "pin_envs": (),
            },
        )
        == _FORMATTED
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
    tox_toml = tmp_path / "tox.toml"
    tox_toml.write_text(start)

    assert run(["tox.toml"]) == exit_code

    out, err = capsys.readouterr()
    assert not err
    assert tox_toml.read_text() == outcome
    assert out == output.format("tox.toml")


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
    tox_toml = tmp_path / "tox.toml"
    tox_toml.write_text(start)

    assert run([str(tox_toml)]) == exit_code

    out, err = capsys.readouterr()
    assert not err
    assert tox_toml.read_text() == outcome
    assert out == output.format(tox_toml)


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
    tox_toml = tmp_path / "tox.toml"
    tox_toml.write_text(start)

    assert run(["tox.toml", "--check"]) == exit_code

    _, err = capsys.readouterr()
    assert not err
    assert tox_toml.read_text() == start


@pytest.mark.parametrize("start", [pytest.param(_FORMATTED, id="formatted"), pytest.param(_UNFORMATTED, id="format")])
def test_main_check_refuses_stdout(tmp_path: Path, start: str) -> None:
    tox_toml = tmp_path / "tox.toml"
    tox_toml.write_text(start)

    with pytest.raises(SystemExit):
        run([str(tox_toml), "--stdout", "--check"])
    assert tox_toml.read_text() == start


@pytest.mark.parametrize(
    ("start", "outcome", "exit_code"),
    [
        pytest.param(_FORMATTED, _FORMATTED, 0, id="formatted"),
        pytest.param(_UNFORMATTED, _FORMATTED, 1, id="format"),
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
    tox_toml = tmp_path / "tox.toml"
    tox_toml.write_text(start)

    assert run([str(tox_toml), "--stdout"]) == exit_code

    out, err = capsys.readouterr()
    assert not err
    assert out == outcome


@pytest.mark.parametrize("indent", [pytest.param(0, id="zero"), pytest.param(2, id="two"), pytest.param(4, id="four")])
def test_indent(tmp_path: Path, indent: int) -> None:

    expected = f"""\
    requires = [
    {" " * indent}"packaging>=24",
    {" " * indent}"tox>=4.22"
    ]
    """
    pyproject_toml = tmp_path / "tox.toml"
    start = """\
    requires = [
     "tox>=4.22",
     "packaging>=24"
    ]
    """
    pyproject_toml.write_text(dedent(start))
    run([str(pyproject_toml), "--indent", str(indent), "--column-width", "40"])
    output = pyproject_toml.read_text()
    assert output == dedent(expected)


def test_pin_env_cli(tmp_path: Path) -> None:
    # A pin leads `env_list` and writes the tables in the same order.
    filename = tmp_path / "tox.toml"
    txt = 'env_list = ["lint", "fix"]\n\n[env.lint]\ndescription = "lint"\n\n[env.fix]\ndescription = "fix"\n'
    filename.write_text(txt)

    run([str(filename), "--pin-env", "fix"])

    assert filename.read_text() == (
        'env_list = [ "fix", "lint" ]\n\n[env.fix]\ndescription = "fix"\n\n[env.lint]\ndescription = "lint"\n'
    )


def test_pin_env_config(tmp_path: Path) -> None:
    filename = tmp_path / "tox.toml"
    filename.write_text("""\
env_list = ["lint", "fix"]

[tox-toml-fmt]
pin_envs = ["fix"]

[env.lint]
description = "lint"

[env.fix]
description = "fix"
""")

    run([str(filename)])

    expected = """\
env_list = [ "fix", "lint" ]

[env.fix]
description = "fix"

[env.lint]
description = "lint"

[tox-toml-fmt]
pin_envs = [ "fix" ]
"""
    assert filename.read_text() == expected


def test_tox_toml_config(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    filename = tmp_path / "tox.toml"
    txt = """\
    requires = [
     "a",
    ]

    [tox-toml-fmt]
    indent = 6
    """
    filename.write_text(dedent(txt))
    run([str(filename)])

    got = filename.read_text()
    expected = """\
    requires = [
          "a",
    ]

    [tox-toml-fmt]
    indent = 6
    """
    assert got == dedent(expected)
    out, err = capsys.readouterr()
    assert out
    assert not err


def test_settings_are_read_beside_a_value_only_toml_1_1_reads(tmp_path: Path) -> None:
    # A file the formatter reads is one its own settings are read from, TOML 1.1 values included.
    tox_toml = tmp_path / "tox.toml"
    tox_toml.write_text(
        'when = 12:30\n\n[env.test]\ndescription = "run the whole unit test suite with coverage"\n'
        "\n[tox-toml-fmt]\ncolumn_width = 30\n"
    )

    run([str(tox_toml)])

    assert '"""\\' in tox_toml.read_text()


def test_a_setting_beside_a_value_only_toml_1_1_reads_is_still_checked(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    tox_toml = tmp_path / "tox.toml"
    start = 'when = 12:30\n\n[env.test]\ndeps = [ "a" ]\n\n[tox-toml-fmt]\ncolumn_width = "wide"\n'
    tox_toml.write_text(start)

    with pytest.raises(SystemExit):
        run([str(tox_toml)])

    assert "is not written as int" in capsys.readouterr().err
    assert tox_toml.read_text() == start


def test_settings_are_read_from_a_file_that_opens_with_a_byte_order_mark(tmp_path: Path) -> None:
    tox_toml = tmp_path / "tox.toml"
    tox_toml.write_text(
        '﻿[env.test]\ndescription = "run the whole unit test suite with coverage"\n'
        "\n[tox-toml-fmt]\ncolumn_width = 30\n",
        encoding="utf-8",
    )

    run([str(tox_toml)])

    assert '"""\\' in tox_toml.read_text()


def test_a_value_nested_deeper_than_a_python_reader_goes(tmp_path: Path) -> None:
    # Reading the settings must reach as far into a document as the formatter does.
    tox_toml = tmp_path / "tox.toml"
    tox_toml.write_text(f"a = {'[' * 200}1{']' * 200}\n")

    assert run([str(tox_toml)]) == 1
    assert tox_toml.read_text().startswith("a = [\n  [\n")


def test_a_value_nested_deeper_than_the_formatter_reads(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # A value past what a stack holds is reported, and the file is left as its author wrote it.
    tox_toml = tmp_path / "tox.toml"
    start = f"a = {'[' * 12_000}1{']' * 12_000}\n"
    tox_toml.write_text(start)

    assert run([str(tox_toml)]) == 1

    assert "nested deeper" in capsys.readouterr().err
    assert tox_toml.read_text() == start


@pytest.mark.parametrize(
    ("setting", "message"),
    [
        pytest.param("indent = -1", "must not be negative", id="negative-count"),
        pytest.param('table_format = "wide"', "invalid choice", id="unknown-table-format"),
        pytest.param("column_width = true", "is not written as int", id="boolean-count"),
        pytest.param("expand_tables = 1", "is not written as list", id="number-list"),
        pytest.param("sub_table_spacing = 1", "is not written as str", id="number-spacing"),
        pytest.param("skip_wrap_for_keys = [ 1 ]", "every entry names one thing", id="list-of-numbers"),
        pytest.param("colm_width = 80", "unknown setting", id="misspelled"),
        pytest.param("check = true", "unknown setting", id="run-mode"),
        pytest.param("column_width = 100000", "must be at most", id="count-beyond-a-line"),
        pytest.param("[tox-toml-fmt.deeper]\nheld = 1", "unknown setting", id="table-below-the-settings"),
    ],
)
def test_a_configured_setting_the_formatter_cannot_hold(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    setting: str,
    message: str,
) -> None:
    tox_toml = tmp_path / "tox.toml"
    start = f'[env.test]\ndeps = [ "a" ]\n\n[tox-toml-fmt]\n{setting}\n'
    tox_toml.write_text(start)

    with pytest.raises(SystemExit):
        run([str(tox_toml)])

    assert message in capsys.readouterr().err
    assert tox_toml.read_text() == start


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
        pytest.param("[tox-toml-fmt]\ncolumn_width = 30", id="a-header"),
        pytest.param("tox-toml-fmt.column_width = 30", id="a-dotted-name"),
        pytest.param('tox-toml-fmt = { "column_width" = 30 }', id="a-name-in-quotes"),
        pytest.param("tox-toml-fmt = { column_width = 30 }", id="a-table-written-as-a-value"),
    ],
)
def test_a_setting_is_read_however_the_file_writes_its_table(tmp_path: Path, settings: str) -> None:
    # TOML maps dotted and explicit table spellings to one path; settings follow that path.
    tox_toml = tmp_path / "tox.toml"
    # a dotted key writes its own table, so it stands before the first header rather than under it
    tox_toml.write_text(f'{settings}\n\n[env.test]\ndescription = "run the whole unit test suite with coverage"\n')

    run([str(tox_toml)])

    assert '"""\\' in tox_toml.read_text()


def test_settings_under_a_repeated_table_are_no_settings(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # A table the file repeats is a list of them, and what its elements write is no one table.
    tox_toml = tmp_path / "tox.toml"
    start = '[[tox-toml-fmt]]\ncolumn_width = 30\n\n[env.test]\ndeps = [ "a" ]\n'
    tox_toml.write_text(start)

    with pytest.raises(SystemExit):
        run([str(tox_toml)])

    assert "an array of tables holds no settings" in capsys.readouterr().err
    assert tox_toml.read_text() == start


def test_the_first_setting_the_formatter_cannot_hold_is_the_one_reported(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    # Report the first invalid setting in source order.
    tox_toml = tmp_path / "tox.toml"
    tox_toml.write_text('[env.test]\ndeps = [ "a" ]\n\n[tox-toml-fmt]\nz_bad = 1\na_bad = 2\n')

    with pytest.raises(SystemExit):
        run([str(tox_toml)])

    assert "z_bad: unknown setting" in capsys.readouterr().err


def test_a_selector_that_names_no_table(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # A setting that names a table TOML cannot read asks for nothing, and is told so.
    tox_toml = tmp_path / "tox.toml"
    start = '[env.test]\ndeps = [ "a" ]\n\n[tox-toml-fmt]\nexpand_tables = [ "env.\\"test" ]\n'
    tox_toml.write_text(start)

    assert run([str(tox_toml)]) == 1

    assert 'expand_tables: env."test is not a table name' in capsys.readouterr().err
    assert tox_toml.read_text() == start


@pytest.mark.parametrize(
    ("held", "first", "second"),
    [
        pytest.param(["--pin-env", '"a,b"'], '[env."a,b"]', "[env.z]", id="pinned"),
        pytest.param([], '[env."a,b"]', "[env.z]", id="by-name"),
        pytest.param(["--pin-env", "z"], "[env.z]", '[env."a,b"]', id="pinned-past-the-name"),
    ],
)
def test_a_pinned_environment_may_hold_a_comma(
    tmp_path: Path, capsys: pytest.CaptureFixture[str], held: list[str], first: str, second: str
) -> None:
    # A pin names one environment, and a name TOML quotes may hold a comma of its own.
    tox_toml = tmp_path / "tox.toml"
    tox_toml.write_text('env_list = [ "z", "a,b" ]\n\n[env.z]\ndeps = [ "y" ]\n\n[env."a,b"]\ndeps = [ "x" ]\n')

    run([str(tox_toml), "--no-print-diff", *held])

    assert not capsys.readouterr().err
    written = tox_toml.read_text()
    assert written.index(first) < written.index(second)
