from __future__ import annotations

import os
from io import StringIO
from typing import TYPE_CHECKING

import pytest

from toml_fmt_common import GREEN, RED, RESET, ArgumentGroup, FmtNamespace, TOMLFormatter, run

if TYPE_CHECKING:
    from pathlib import Path

    from pytest_mock import MockerFixture


class DumpNamespace(FmtNamespace):
    extra: str
    tuple_magic: tuple[str, ...]


class Dumb(TOMLFormatter[DumpNamespace]):
    def __init__(self) -> None:
        super().__init__(DumpNamespace())

    @property
    def prog(self) -> str:
        return "toml-fmt-common"

    @property
    def filename(self) -> str:
        return "dumb.toml"

    @property
    def override_cli_from_section(self) -> tuple[str, ...]:
        return "start", "sub"

    def add_format_flags(self, parser: ArgumentGroup) -> None:  # noqa: PLR6301
        parser.add_argument("extra", help="this is something extra")
        parser.add_argument("-t", "--tuple-magic", default=(), type=lambda t: tuple(t.split(".")))

    def format(self, text: str, opt: DumpNamespace) -> str:  # noqa: PLR6301
        if os.environ.get("NO_FMT"):
            return text
        return "\n".join([
            text,
            f"extras = {opt.extra!r}",
            *([f"magic = {','.join(opt.tuple_magic)!r}"] if opt.tuple_magic else []),
        ])


def test_dumb_help(capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit) as exc:
        run(Dumb(), ["--help"])

    assert exc.value.code == 0

    out, err = capsys.readouterr()
    assert not err
    assert "this is something extra" in out


def test_dumb_format_with_override(capsys: pytest.CaptureFixture[str], tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("[start.sub]\nextra = 'B'")

    exit_code = run(Dumb(), ["E", str(dumb)])
    assert exit_code == 1

    assert dumb.read_text() == "[start.sub]\nextra = 'B'\nextras = 'B'"

    out, err = capsys.readouterr()
    assert not err
    assert out.splitlines() == [
        f"{RED}--- {dumb}",
        f"{RESET}",
        f"{GREEN}+++ {dumb}",
        f"{RESET}",
        "@@ -1,2 +1,3 @@",
        "",
        " [start.sub]",
        " extra = 'B'",
        f"{GREEN}+extras = 'B'{RESET}",
    ]


def test_dumb_format_with_override_custom_type(capsys: pytest.CaptureFixture[str], tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("[start.sub]\ntuple_magic = '1.2.3'")

    exit_code = run(Dumb(), ["E", str(dumb)])
    assert exit_code == 1

    assert dumb.read_text() == "[start.sub]\ntuple_magic = '1.2.3'\nextras = 'E'\nmagic = '1,2,3'"

    out, err = capsys.readouterr()
    assert not err
    assert out.splitlines() == [
        f"{RED}--- {dumb}",
        f"{RESET}",
        f"{GREEN}+++ {dumb}",
        f"{RESET}",
        "@@ -1,2 +1,4 @@",
        "",
        " [start.sub]",
        " tuple_magic = '1.2.3'",
        f"{GREEN}+extras = 'E'{RESET}",
        f"{GREEN}+magic = '1,2,3'{RESET}",
    ]


def test_dumb_format_no_print_diff(capsys: pytest.CaptureFixture[str], tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("[start.sub]\nextra = 'B'")

    exit_code = run(Dumb(), ["E", str(dumb), "--no-print-diff"])
    assert exit_code == 1

    assert dumb.read_text() == "[start.sub]\nextra = 'B'\nextras = 'B'"

    out, err = capsys.readouterr()
    assert not err
    assert out.splitlines() == []


def test_dumb_format_already_good(
    capsys: pytest.CaptureFixture[str], tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("NO_FMT", "1")
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("[start.sub]\nextra = 'B'")

    exit_code = run(Dumb(), ["E", str(dumb)])
    assert exit_code == 0

    assert dumb.read_text() == "[start.sub]\nextra = 'B'"

    out, err = capsys.readouterr()
    assert not err
    assert out.splitlines() == [f"no change for {dumb}"]


def test_dumb_format_via_folder(
    capsys: pytest.CaptureFixture[str], tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.chdir(tmp_path)
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")

    exit_code = run(Dumb(), ["E", "."])
    assert exit_code == 1

    assert dumb.read_text() == "\nextras = 'E'"

    out, err = capsys.readouterr()
    assert not err
    assert out.splitlines() == [
        f"{RED}--- dumb.toml",
        f"{RESET}",
        f"{GREEN}+++ dumb.toml",
        f"{RESET}",
        "@@ -0,0 +1,2 @@",
        "",
        f"{GREEN}+{RESET}",
        f"{GREEN}+extras = 'E'{RESET}",
    ]


def test_dumb_format_override_non_dict_result(capsys: pytest.CaptureFixture[str], tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("[start]\nsub = 'B'")

    exit_code = run(Dumb(), ["E", str(dumb)])
    assert exit_code == 1

    assert dumb.read_text() == "[start]\nsub = 'B'\nextras = 'E'"

    out, err = capsys.readouterr()
    assert not err
    assert out.splitlines() == [
        f"{RED}--- {dumb}",
        f"{RESET}",
        f"{GREEN}+++ {dumb}",
        f"{RESET}",
        "@@ -1,2 +1,3 @@",
        "",
        " [start]",
        " sub = 'B'",
        f"{GREEN}+extras = 'E'{RESET}",
    ]


def test_dumb_format_override_non_dict_part(capsys: pytest.CaptureFixture[str], tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("start = 'B'")

    exit_code = run(Dumb(), ["E", str(dumb)])
    assert exit_code == 1

    assert dumb.read_text() == "start = 'B'\nextras = 'E'"

    out, err = capsys.readouterr()
    assert not err
    assert out.splitlines() == [
        f"{RED}--- {dumb}",
        f"{RESET}",
        f"{GREEN}+++ {dumb}",
        f"{RESET}",
        "@@ -1 +1,2 @@",
        "",
        " start = 'B'",
        f"{GREEN}+extras = 'E'{RESET}",
    ]


def test_dumb_stdin(capsys: pytest.CaptureFixture[str], mocker: MockerFixture) -> None:
    mocker.patch("sys.stdin", StringIO("ok = 1"))

    exit_code = run(Dumb(), ["E", "-"])
    assert exit_code == 1

    out, err = capsys.readouterr()
    assert not err
    assert out.splitlines() == ["ok = 1", "extras = 'E'"]


def test_dumb_path_missing(capsys: pytest.CaptureFixture[str], tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.chdir(tmp_path)

    with pytest.raises(SystemExit):
        run(Dumb(), ["E", "dumb.toml"])

    out, err = capsys.readouterr()
    assert "\ntoml-fmt-common: error: argument inputs: path does not exist\n" in err
    assert not out


def test_dumb_path_is_folder(capsys: pytest.CaptureFixture[str], tmp_path: Path) -> None:
    toml = tmp_path / "dumb.toml"
    os.mkfifo(toml)

    with pytest.raises(SystemExit):
        run(Dumb(), ["E", str(toml)])

    out, err = capsys.readouterr()
    assert "\ntoml-fmt-common: error: argument inputs: path is not a file\n" in err
    assert not out


def test_dumb_path_no_read(capsys: pytest.CaptureFixture[str], tmp_path: Path) -> None:
    toml = tmp_path / "dumb.toml"
    toml.write_text("")
    start = toml.stat().st_mode
    toml.chmod(0o000)

    try:
        with pytest.raises(SystemExit):
            run(Dumb(), ["E", str(toml)])
    finally:
        toml.chmod(start)

    out, err = capsys.readouterr()
    assert "\ntoml-fmt-common: error: argument inputs: cannot read path\n" in err
    assert not out


def test_dumb_path_no_write(capsys: pytest.CaptureFixture[str], tmp_path: Path) -> None:
    toml = tmp_path / "dumb.toml"
    toml.write_text("")
    start = toml.stat().st_mode
    toml.chmod(0o400)

    try:
        with pytest.raises(SystemExit):
            run(Dumb(), ["E", str(toml)])
    finally:
        toml.chmod(start)

    out, err = capsys.readouterr()
    assert "\ntoml-fmt-common: error: argument inputs: cannot write path\n" in err
    assert not out


def test_writes_lf_line_endings(tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")

    run(Dumb(), ["E", str(dumb)])

    raw = dumb.read_bytes()
    assert b"\r\n" not in raw
    assert b"\n" in raw
