from __future__ import annotations

import os
from io import StringIO
from typing import TYPE_CHECKING

import pytest

from toml_fmt_common import GREEN, RED, RESET, ArgumentGroup, FmtNamespace, TOMLFormatter, _build_cli, build_cli, run

if TYPE_CHECKING:
    from pathlib import Path

    from pytest_mock import MockerFixture


class DumpNamespace(FmtNamespace):
    extra: str
    tuple_magic: tuple[str, ...]


class Dumb(TOMLFormatter[DumpNamespace]):
    def __init__(self) -> None:
        super().__init__(DumpNamespace())
        self.last_format_opt: DumpNamespace | None = None

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

    def format(self, text: str, opt: DumpNamespace) -> str:
        self.last_format_opt = opt
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
    assert "cannot write path" in err
    assert not out


def test_dumb_path_no_write_check_mode(
    capsys: pytest.CaptureFixture[str], tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("NO_FMT", "1")
    toml = tmp_path / "dumb.toml"
    toml.write_text("")
    start = toml.stat().st_mode
    toml.chmod(0o400)

    try:
        exit_code = run(Dumb(), ["E", "--check", str(toml)])
    finally:
        toml.chmod(start)

    assert exit_code == 0
    _out, err = capsys.readouterr()
    assert not err


def test_dumb_path_no_write_stdout_mode(
    capsys: pytest.CaptureFixture[str], tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("NO_FMT", "1")
    toml = tmp_path / "dumb.toml"
    toml.write_text("")
    start = toml.stat().st_mode
    toml.chmod(0o400)

    try:
        exit_code = run(Dumb(), ["E", "--stdout", str(toml)])
    finally:
        toml.chmod(start)

    assert exit_code == 0
    _out, err = capsys.readouterr()
    assert not err


def test_writes_lf_line_endings(tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")

    run(Dumb(), ["E", str(dumb)])

    raw = dumb.read_bytes()
    assert b"\r\n" not in raw
    assert b"\n" in raw


def test_config_flag_explicit(tmp_path: Path) -> None:
    config_file = tmp_path / "toml-fmt-common.toml"
    config_file.write_text("extra = 'FROM_CONFIG'")
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")

    exit_code = run(Dumb(), ["E", str(dumb), "--config", str(config_file)])
    assert exit_code == 1
    assert dumb.read_text() == "\nextras = 'FROM_CONFIG'"


def test_config_flag_nonexistent(capsys: pytest.CaptureFixture[str], tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")

    with pytest.raises(SystemExit):
        run(Dumb(), ["E", str(dumb), "--config", str(tmp_path / "missing.toml")])

    out, err = capsys.readouterr()
    assert "config file does not exist" in err
    assert not out


def test_config_auto_discovery(tmp_path: Path) -> None:
    config_file = tmp_path / "toml-fmt-common.toml"
    config_file.write_text("extra = 'DISCOVERED'")
    sub = tmp_path / "sub"
    sub.mkdir()
    dumb = sub / "dumb.toml"
    dumb.write_text("")

    exit_code = run(Dumb(), ["E", str(dumb)])
    assert exit_code == 1
    assert dumb.read_text() == "\nextras = 'DISCOVERED'"


def test_config_auto_discovery_not_found(tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")

    exit_code = run(Dumb(), ["E", str(dumb)])
    assert exit_code == 1
    assert dumb.read_text() == "\nextras = 'E'"


def test_config_per_file_overrides_shared(tmp_path: Path) -> None:
    config_file = tmp_path / "toml-fmt-common.toml"
    config_file.write_text("extra = 'SHARED'")
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("[start.sub]\nextra = 'IN_FILE'")

    exit_code = run(Dumb(), ["E", str(dumb)])
    assert exit_code == 1
    assert dumb.read_text() == "[start.sub]\nextra = 'IN_FILE'\nextras = 'IN_FILE'"


def test_config_stdin_uses_cwd(mocker: MockerFixture, tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.chdir(tmp_path)
    config_file = tmp_path / "toml-fmt-common.toml"
    config_file.write_text("extra = 'CWD_CONFIG'")
    mocker.patch("sys.stdin", StringIO("ok = 1"))

    exit_code = run(Dumb(), ["E", "-"])
    assert exit_code == 1


def test_config_shared_custom_type(tmp_path: Path) -> None:
    config_file = tmp_path / "toml-fmt-common.toml"
    config_file.write_text("tuple_magic = '1.2.3'")
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")

    exit_code = run(Dumb(), ["E", str(dumb), "--config", str(config_file)])
    assert exit_code == 1
    assert dumb.read_text() == "\nextras = 'E'\nmagic = '1,2,3'"


def test_shared_args_in_help(capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit):
        run(Dumb(), ["--help"])
    out = capsys.readouterr().out
    for arg in (
        "--table-format",
        "--sub-table-spacing",
        "--separate-root-table",
        "--expand-tables",
        "--collapse-tables",
        "--skip-wrap-for-keys",
    ):
        assert arg in out


def test_shared_args_defaults(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("NO_FMT", "1")
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")
    fmt = Dumb()
    run(fmt, ["E", str(dumb)])
    assert fmt.opt.table_format == "short"
    assert not fmt.opt.sub_table_spacing
    assert fmt.opt.separate_root_table == "\n"
    assert fmt.opt.expand_tables == []
    assert fmt.opt.collapse_tables == []
    assert fmt.opt.skip_wrap_for_keys == []


def test_shared_args_cli_override(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("NO_FMT", "1")
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")
    fmt = Dumb()
    run(
        fmt,
        [
            "E",
            str(dumb),
            "--table-format",
            "long",
            "--sub-table-spacing",
            r"\n",
            "--separate-root-table",
            r"\n\n",
            "--expand-tables",
            "a,b",
            "--collapse-tables",
            "c",
            "--skip-wrap-for-keys",
            "*.parse",
        ],
    )
    assert fmt.opt.table_format == "long"
    assert fmt.opt.sub_table_spacing == "\n"
    assert fmt.opt.separate_root_table == "\n\n"
    assert fmt.opt.expand_tables == ["a", "b"]
    assert fmt.opt.collapse_tables == ["c"]
    assert fmt.opt.skip_wrap_for_keys == ["*.parse"]


def test_shared_args_config_file(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("NO_FMT", "1")
    config = tmp_path / "toml-fmt-common.toml"
    config.write_text('table_format = "long"\nsub_table_spacing = "\\n"\nexpand_tables = ["x", "y"]')
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")
    fmt = Dumb()
    run(fmt, ["E", str(dumb), "--config", str(config)])
    assert fmt.last_format_opt is not None
    assert fmt.last_format_opt.table_format == "long"
    assert fmt.last_format_opt.sub_table_spacing == "\n"
    assert fmt.last_format_opt.expand_tables == ["x", "y"]


def test_build_cli_underscore_alias_preserved() -> None:
    # _build_cli is the pre-1.3.3 name every released pyproject-fmt/tox-toml-fmt imports;
    # dropping it breaks those wheels on a fresh resolve (tox-dev/toml-fmt#355).
    assert _build_cli is build_cli


class LegacyDumb(Dumb):
    # Mirrors pyproject-fmt <=2.21.2, which re-registers the shared format flags that
    # build_cli now also defines (tox-dev/toml-fmt#355).
    def add_format_flags(self, parser: ArgumentGroup) -> None:
        super().add_format_flags(parser)
        parser.add_argument("--table-format", choices=["short", "long"], default="short")
        parser.add_argument("--expand-tables", default=[])


def test_legacy_consumer_reregistering_flags_does_not_crash(tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("")
    # Must not raise argparse.ArgumentError on the duplicate --table-format/--expand-tables.
    assert run(LegacyDumb(), ["E", str(dumb), "--table-format", "long"]) == 1
