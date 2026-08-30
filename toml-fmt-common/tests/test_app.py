from __future__ import annotations

import os
import sys
from argparse import ArgumentTypeError
from io import StringIO
from typing import TYPE_CHECKING, Any

if sys.version_info >= (3, 11):  # pragma: >=3.11 cover
    import tomllib
else:  # pragma: <3.11 cover
    import tomli as tomllib

import pytest

from toml_fmt_common import (
    GREEN,
    RED,
    RESET,
    ArgumentGroup,
    FmtNamespace,
    TOMLFormatter,
    _build_cli,
    _color_diff,
    build_cli,
    count_argument,
    list_argument,
    name_list_argument,
    run,
    spacing_argument,
)

if TYPE_CHECKING:
    from collections.abc import Callable, Iterator, Sequence
    from pathlib import Path

    from pytest_mock import MockerFixture


class DumpNamespace(FmtNamespace):
    extra: str
    tuple_magic: tuple[str, ...]
    loud: bool


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

    def add_format_flags(self, parser: ArgumentGroup) -> None:  # ruff: ignore[no-self-use]
        parser.add_argument("extra", help="this is something extra")
        parser.add_argument("-t", "--tuple-magic", default=(), type=lambda t: tuple(t.split(".")))
        parser.add_argument("--loud", action="store_true", help="say more about what was done")

    def settings_in(self, text: str, path: Sequence[str]) -> dict[str, Any] | None:  # ruff: ignore[no-self-use]
        try:
            held: Any = tomllib.loads(text)
        except tomllib.TOMLDecodeError as exc:
            raise SyntaxError(str(exc)) from exc
        for part in path:
            if not isinstance(held, dict) or part not in held:
                return None
            held = held[part]
        if not isinstance(held, dict):
            return None
        for name, value in held.items():
            if not isinstance(value, str | int | bool | list):
                # the loader reads a malformed setting as a value error, whatever went wrong in it
                msg = f"{name}: {value} is not a setting"
                raise ValueError(msg)  # ruff: ignore[type-check-without-type-error]
        return held

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


@pytest.mark.parametrize(
    ("start", "added", "hunk"),
    [
        pytest.param("[start.sub]\nextra = 'B'", ["extras = 'B'"], "@@ -1,2 +1,3 @@", id="override-reaches-the-key"),
        pytest.param(
            "[start.sub]\ntuple_magic = '1.2.3'",
            ["extras = 'E'", "magic = '1,2,3'"],
            "@@ -1,2 +1,4 @@",
            id="a-setting-of-its-own-type",
        ),
        pytest.param("[start]\nsub = 'B'", ["extras = 'E'"], "@@ -1,2 +1,3 @@", id="the-table-holds-a-value"),
        pytest.param("start = 'B'", ["extras = 'E'"], "@@ -1 +1,2 @@", id="the-root-holds-a-value"),
    ],
)
def test_a_formatted_file_is_written_back_and_the_change_printed(
    capsys: pytest.CaptureFixture[str], tmp_path: Path, start: str, added: list[str], hunk: str
) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text(start)

    exit_code = run(Dumb(), ["E", str(dumb)])

    assert exit_code == 1
    assert dumb.read_text() == start + "\n" + "\n".join(added)
    out, err = capsys.readouterr()
    assert not err
    assert out.splitlines() == [
        f"{RED}--- {dumb}",
        f"{RESET}",
        f"{GREEN}+++ {dumb}",
        f"{RESET}",
        hunk,
        "",
        *(f" {line}" for line in start.splitlines()),
        *(f"{GREEN}+{line}{RESET}" for line in added),
    ]


def test_color_diff_disabled_by_no_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("NO_COLOR", "1")
    assert list(_color_diff(["+added", "-removed", " context"])) == ["+added", "-removed", " context"]


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


def test_dumb_stdin(capsys: pytest.CaptureFixture[str], mocker: MockerFixture) -> None:
    mocker.patch("sys.stdin", StringIO("ok = 1"))

    exit_code = run(Dumb(), ["E", "-"])
    assert exit_code == 1

    out, err = capsys.readouterr()
    assert not err
    assert out.splitlines() == ["ok = 1", "extras = 'E'"]


def _leave_missing(_path: Path) -> None:
    """The path the argument names was never written."""


def _write_unreadable(path: Path) -> None:
    path.write_text("", encoding="utf-8")
    path.chmod(0o000)


def _write_read_only(path: Path) -> None:
    path.write_text("", encoding="utf-8")
    path.chmod(0o400)


@pytest.fixture
def target(tmp_path: Path) -> Iterator[Path]:
    """A path the run is pointed at, restored to a mode the fixture can clean up."""
    path = tmp_path / "dumb.toml"
    yield path
    if path.exists():
        path.chmod(0o600)


@pytest.mark.parametrize(
    ("prepare", "message"),
    [
        pytest.param(_leave_missing, "argument inputs: path does not exist\n", id="missing"),
        pytest.param(os.mkfifo, "argument inputs: path is not a file\n", id="not-a-file"),
        pytest.param(_write_unreadable, "argument inputs: cannot read path\n", id="unreadable"),
        pytest.param(_write_read_only, "cannot write path", id="read-only"),
    ],
)
def test_a_path_the_run_cannot_use_is_named_in_the_error(
    capsys: pytest.CaptureFixture[str],
    target: Path,
    prepare: Callable[[Path], None],
    message: str,
) -> None:
    prepare(target)

    with pytest.raises(SystemExit):
        run(Dumb(), ["E", str(target)])

    out, err = capsys.readouterr()
    assert message in err
    assert not out


@pytest.mark.parametrize("mode", ["--check", "--stdout"])
def test_a_read_only_path_is_fine_for_a_run_that_writes_nothing(
    capsys: pytest.CaptureFixture[str], target: Path, monkeypatch: pytest.MonkeyPatch, mode: str
) -> None:
    monkeypatch.setenv("NO_FMT", "1")
    _write_read_only(target)

    exit_code = run(Dumb(), ["E", mode, str(target)])

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


def test_format_rejection_reported(
    capsys: pytest.CaptureFixture[str],
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    mocker: MockerFixture,
) -> None:
    monkeypatch.chdir(tmp_path)
    (tmp_path / "dumb.toml").write_text("ok = 1")
    mocker.patch.object(Dumb, "format", side_effect=ValueError("bad version"))

    assert run(Dumb(), ["E", "dumb.toml"]) == 1

    out, err = capsys.readouterr()
    assert not out
    assert err == "dumb.toml: bad version\n"


def test_format_rejection_reported_for_stdin(
    capsys: pytest.CaptureFixture[str],
    mocker: MockerFixture,
) -> None:
    mocker.patch("sys.stdin", StringIO("ok = 1"))
    mocker.patch.object(Dumb, "format", side_effect=ValueError("bad version"))

    assert run(Dumb(), ["E", "-"]) == 1

    out, err = capsys.readouterr()
    assert not out
    assert err == "<stdin>: bad version\n"


@pytest.mark.parametrize("eol", [pytest.param("\r\n", id="crlf"), pytest.param("\n", id="lf")])
def test_dumb_format_keeps_line_ending(tmp_path: Path, eol: str) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_bytes(f"[start.sub]{eol}extra = 'B'".encode())

    assert run(Dumb(), ["E", str(dumb), "--no-print-diff"]) == 1

    assert dumb.read_bytes() == f"[start.sub]{eol}extra = 'B'{eol}extras = 'B'".encode()


def test_dumb_format_mixed_line_endings_take_the_majority(tmp_path: Path) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_bytes(b"other = 1\r\nmore = 2\r\nkeep = 3\n[start.sub]\r\nextra = 'B'")

    assert run(Dumb(), ["E", str(dumb), "--no-print-diff"]) == 1

    assert dumb.read_bytes() == b"other = 1\r\nmore = 2\r\nkeep = 3\r\n[start.sub]\r\nextra = 'B'\r\nextras = 'B'"


def test_dumb_format_crlf_alone_is_not_a_change(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("NO_FMT", "1")
    dumb = tmp_path / "dumb.toml"
    dumb.write_bytes(b"[start.sub]\r\nextra = 'B'")

    assert run(Dumb(), ["E", str(dumb), "--no-print-diff"]) == 0

    assert dumb.read_bytes() == b"[start.sub]\r\nextra = 'B'"


@pytest.mark.parametrize(
    "written",
    [
        pytest.param("-1", id="negative"),
        pytest.param("two", id="not-a-number"),
        # `True` is an integer to Python, and a file that writes one there names no count
        pytest.param(True, id="boolean"),
    ],
)
def test_a_count_the_formatter_cannot_hold(written: str | int) -> None:
    with pytest.raises(ArgumentTypeError):
        count_argument(written)


def test_a_count_reads_what_it_is_given() -> None:
    assert count_argument("4") == 4


@pytest.mark.parametrize(
    "written",
    [
        # no reader accepts this, so the formatter is the one that says so
        pytest.param("key =\n", id="not-a-document"),
        # TOML 1.1, which the formatter reads and this reader may not
        pytest.param("value = 12:30\n", id="a-time-without-seconds"),
    ],
)
def test_a_target_the_settings_reader_cannot_read(tmp_path: Path, written: str) -> None:
    """The formatter is the one that reports on a file this reader cannot get the settings out of."""
    dumb = tmp_path / "dumb.toml"
    dumb.write_text(written)

    assert run(Dumb(), ["E", str(dumb)]) == 1
    assert dumb.read_text() == f"{written}\nextras = 'E'"


def test_a_setting_written_in_a_form_no_setting_takes(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    dumb = tmp_path / "dumb.toml"
    start = "[start.sub]\nextra = 1979-05-27\n"
    dumb.write_text(start)

    with pytest.raises(SystemExit):
        run(Dumb(), ["E", str(dumb)])

    assert "extra: 1979-05-27 is not a setting" in capsys.readouterr().err
    assert dumb.read_text() == start


@pytest.mark.parametrize(
    ("setting", "message"),
    [
        pytest.param("loud = true", None, id="a-flag-the-file-turns-on"),
        pytest.param('loud = "yes"', "is not written as bool", id="a-flag-written-as-text"),
        pytest.param("indent = true", "is not written as int", id="a-count-written-as-a-flag"),
        pytest.param("louder = true", "unknown setting", id="a-setting-of-no-name"),
        pytest.param("check = true", "unknown setting", id="a-run-mode-key"),
    ],
)
def test_a_setting_read_against_the_type_its_flag_takes(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    setting: str,
    message: str | None,
) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text(f"[start.sub]\nextra = 'B'\n{setting}\n")

    if message is None:
        assert run(Dumb(), ["E", str(dumb), "--no-print-diff"]) == 1
        return
    with pytest.raises(SystemExit):
        run(Dumb(), ["E", str(dumb)])
    assert message in capsys.readouterr().err


def test_a_count_wider_than_any_line() -> None:
    with pytest.raises(ArgumentTypeError, match="must be at most"):
        count_argument(10_001)


def test_spacing_reads_only_what_a_spacing_is_written_as() -> None:
    assert spacing_argument(r"\n") == "\n"
    with pytest.raises(ArgumentTypeError, match="invalid spacing"):
        spacing_argument(1)  # type: ignore[arg-type]  # a file may write a number where a spacing belongs


def test_a_list_reads_only_the_names_in_it() -> None:
    assert list_argument("a, b") == ["a", "b"]
    assert list_argument(["a"]) == ["a"]
    # a name TOML quotes may hold a comma of its own, which separates nothing
    assert list_argument('tool."a,b"') == ['tool."a,b"']
    assert list_argument("tool.'a,b', other") == ["tool.'a,b'", "other"]
    with pytest.raises(ArgumentTypeError, match="every entry names one thing"):
        list_argument([1])  # type: ignore[list-item]  # a file may write a number where a name belongs


@pytest.mark.parametrize(
    ("written", "expected"),
    [
        pytest.param("fix, type", ["fix", "type"], id="plain-names"),
        pytest.param('"a,b"', ["a,b"], id="a-comma-inside-a-name"),
        pytest.param("'a,b', c", ["a,b", "c"], id="a-literal-name"),
        pytest.param(r'"a\",b"', ['a",b'], id="an-escaped-quote"),
        pytest.param(r'"a\\", b', ["a\\", "b"], id="an-escaped-backslash"),
        pytest.param(r'"\t\n\r\f\b\e"', ["\t\n\r\f\b\x1b"], id="the-named-escapes"),
        pytest.param(r'"\u0041\U00000042\x43"', ["ABC"], id="the-numbered-escapes"),
        pytest.param(["already", "read"], ["already", "read"], id="a-list-the-file-wrote"),
    ],
)
def test_a_name_list_reads_one_name_per_entry(written: str | list[str], expected: list[str]) -> None:
    assert name_list_argument(written) == expected


@pytest.mark.parametrize(
    "written",
    [
        pytest.param(b"indent =\n", id="not-a-document"),
        pytest.param(b"indent = '\xff'\n", id="not-utf-8"),
    ],
)
def test_a_shared_config_the_reader_cannot_read_is_named(
    tmp_path: Path, capsys: pytest.CaptureFixture[str], written: bytes
) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text("[start.sub]\nextra = 'B'\n")
    shared = tmp_path / "toml-fmt-common.toml"
    shared.write_bytes(written)

    with pytest.raises(SystemExit):
        run(Dumb(), ["E", "--config", str(shared), str(dumb)])

    assert str(shared) in capsys.readouterr().err


@pytest.mark.parametrize(
    ("setting", "message"),
    [
        pytest.param("indent = -1", "must not be negative", id="a-count-it-cannot-hold"),
        pytest.param('table_format = "wide"', "invalid choice", id="a-format-it-does-not-know"),
    ],
)
def test_a_configured_setting_the_formatter_cannot_hold(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    setting: str,
    message: str,
) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_text(f"[start.sub]\nextra = 'B'\n{setting}\n")

    with pytest.raises(SystemExit):
        run(Dumb(), ["E", str(dumb)])

    assert message in capsys.readouterr().err


def test_a_target_that_is_not_utf_8(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    dumb = tmp_path / "dumb.toml"
    dumb.write_bytes(b"extra = '\xff'\n")

    with pytest.raises(SystemExit):
        run(Dumb(), ["E", str(dumb)])

    assert str(dumb) in capsys.readouterr().err
    assert dumb.read_bytes() == b"extra = '\xff'\n"


def test_a_carriage_return_toml_does_not_read(tmp_path: Path) -> None:
    """A `\r` not followed by `\n` is a syntax error, which normalizing it away would hide."""
    dumb = tmp_path / "dumb.toml"
    dumb.write_bytes(b"[start.sub]\rextra = 'B'\n")

    assert run(Dumb(), ["E", str(dumb)]) == 1

    assert dumb.read_bytes() == b"[start.sub]\rextra = 'B'\n\nextras = 'E'"
