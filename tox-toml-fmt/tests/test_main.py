from __future__ import annotations

from textwrap import dedent
from typing import TYPE_CHECKING

import pytest
from tox_toml_fmt.__main__ import runner as run

if TYPE_CHECKING:
    from pathlib import Path

    from pytest_mock import MockerFixture


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
            'requires = [ "tox>=4.22" ]\n',
            'requires = [ "tox>=4.22" ]\n',
            "no change for {0}\n",
            id="formatted",
        ),
        pytest.param(
            'requires = ["tox>=4.22"]\n',
            'requires = [ "tox>=4.22" ]\n',
            '--- {0}\n\n+++ {0}\n\n@@ -1 +1 @@\n\n-requires = ["tox>=4.22"]\n+requires = [ "tox>=4.22" ]\n',
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
    mocker: MockerFixture,
    cwd: bool,
    check: bool,
) -> None:
    mocker.patch("toml_fmt_common._color_diff", lambda t: t)
    if cwd:
        monkeypatch.chdir(tmp_path)
    pyproject_toml = tmp_path / "tox.toml"
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
        name = "tox.toml" if cwd else str(tmp_path / "tox.toml")
        output = output.format(name)
        assert pyproject_toml.read_text() == outcome
        assert out == output
    else:
        assert out == outcome


@pytest.mark.parametrize("indent", [0, 2, 4])
def test_indent(tmp_path: Path, indent: int) -> None:
    start = """\
    requires = [
     "a"
    ]
    """

    expected = f"""\
    requires = [
    {" " * indent}"a",
    ]
    """
    pyproject_toml = tmp_path / "tox.toml"
    pyproject_toml.write_text(dedent(start))
    args = [str(pyproject_toml), "--indent", str(indent)]
    run(args)
    output = pyproject_toml.read_text()
    assert output == dedent(expected)


def test_tox_toml_config(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    txt = """\
    requires = [
     "a",
    ]

    [tox-toml-fmt]
    indent = 6
    """
    filename = tmp_path / "tox.toml"
    filename.write_text(dedent(txt))
    run([str(filename)])

    expected = """\
    requires = [
          "a",
    ]

    [tox-toml-fmt]
    indent = 6
    """
    got = filename.read_text()
    assert got == dedent(expected)
    out, err = capsys.readouterr()
    assert out
    assert not err
