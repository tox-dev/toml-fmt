"""Keep CLI behavior identical across the TOML formatters."""

from __future__ import annotations

import difflib
import os
import sys
from abc import ABC, abstractmethod
from argparse import (
    Action,
    ArgumentDefaultsHelpFormatter,
    ArgumentParser,
    ArgumentTypeError,
    Namespace,
)
from collections.abc import Mapping, Sequence
from copy import deepcopy
from dataclasses import dataclass, field
from functools import partial
from importlib.metadata import version
from pathlib import Path
from typing import TYPE_CHECKING, Final, Generic, Protocol, TypeAlias, TypeVar, cast

if TYPE_CHECKING:
    from collections.abc import Callable, Iterable, Iterator


class ArgumentGroup(Protocol):
    """Expose the argument-group operation formatter plugins use."""

    def add_argument(self, *name_or_flags: str, **kwargs: object) -> Action:
        """Register one argument and return its parser action."""
        ...


TomlValue: TypeAlias = "bool | int | float | str | Sequence[TomlValue] | Mapping[str, TomlValue]"


class FmtNamespace(Namespace):
    """Give argparse fields the types shared by both formatters."""

    inputs: list[Path | None]
    stdout: bool
    check: bool
    no_print_diff: bool
    config: Path | None

    column_width: int
    indent: int
    table_format: str
    sub_table_spacing: str
    separate_root_table: str
    expand_tables: Sequence[str]
    collapse_tables: Sequence[str]
    skip_wrap_for_keys: Sequence[str]


_NAMESPACE_T = TypeVar("_NAMESPACE_T", bound=FmtNamespace)


class TOMLFormatter(ABC, Generic[_NAMESPACE_T]):
    """Supply tool-specific hooks to the shared CLI."""

    def __init__(self, opt: _NAMESPACE_T) -> None:
        """Start with the namespace argparse fills for each input."""
        self.opt: _NAMESPACE_T = opt

    @property
    @abstractmethod
    def prog(self) -> str:
        """Match the distribution name used for version lookup."""
        raise NotImplementedError

    @property
    @abstractmethod
    def filename(self) -> str:
        """Constrain positional paths and configuration discovery."""
        raise NotImplementedError

    @staticmethod
    @abstractmethod
    def add_format_flags(parser: ArgumentGroup) -> None:
        """Add options that the shared settings do not cover."""
        raise NotImplementedError

    @property
    @abstractmethod
    def override_cli_from_section(self) -> tuple[str, ...]:
        """Keep per-file overrides beside the tool configuration they control."""
        raise NotImplementedError

    @staticmethod
    @abstractmethod
    def settings_in(text: str, path: Sequence[str]) -> dict[str, TomlValue] | None:
        """
        Read the settings the text writes under a table, with the parser that reads the file itself.

        A second reader of an older TOML would drop a file's own configuration on a value it cannot
        read, and format the file as though none had been written.

        :return: the settings, or ``None`` where the text writes no such table
        :raises SyntaxError: if the text is not a TOML document
        :raises ValueError: if a setting is written in a form no setting takes
        """
        raise NotImplementedError

    @staticmethod
    @abstractmethod
    def format(text: str, opt: _NAMESPACE_T) -> str:
        """Keep CLI state outside the TOML rewriting layer."""
        raise NotImplementedError


def run(formatter: TOMLFormatter[_NAMESPACE_T], args: Sequence[str] | None = None) -> int:
    """
    Parse one option set per input and return 1 after a change or rejection.

    A supplied argument list supports embedding; the console script falls back to ``sys.argv``.
    """
    configs = _cli_args(formatter, sys.argv[1:] if args is None else args)
    return int(any(_handle_one(formatter, config) for config in configs))


@dataclass(frozen=True)
class _Config(Generic[_NAMESPACE_T]):
    toml_filename: Path | None
    toml: str
    stdout: bool
    check: bool
    no_print_diff: bool
    opt: _NAMESPACE_T
    eol: str


def _check_write_permission(parser: ArgumentParser, opt: FmtNamespace) -> None:
    if opt.stdout or opt.check:
        return
    for toml_path in opt.inputs:
        if toml_path is not None and not os.access(toml_path, os.W_OK):
            parser.error(f"argument inputs: cannot write path {toml_path}")


def _cli_args(formatter: TOMLFormatter[_NAMESPACE_T], args: Sequence[str]) -> list[_Config[_NAMESPACE_T]]:
    parser, type_conversion, actions = _make_cli(formatter)
    parser.parse_args(namespace=formatter.opt, args=args)
    if (explicit_config := formatter.opt.config) is not None and not explicit_config.is_file():
        parser.error(f"config file does not exist: {explicit_config}")
    _check_write_permission(parser, formatter.opt)
    constraints = _Constraints(
        conversion=type_conversion,
        # a value read from a file has to be one the command line would have accepted
        allowed={action.dest: action.choices for action in actions if action.choices},
        accepts=_accepted_types(parser, formatter.opt),
    )
    configs = []
    for toml_path in formatter.opt.inputs:
        raw, eol = _read_input(parser, toml_path)
        source = _display_name(toml_path)
        try:
            config = formatter.settings_in(raw, formatter.override_cli_from_section)
        except SyntaxError:
            # the formatter reads the same source next and reports on it in its own words, against
            # the file rather than against one setting
            config = None
        except (TypeError, ValueError) as exc:
            parser.error(f"{source}: {exc}")
        override_opt = deepcopy(formatter.opt)
        if explicit_config is not None:
            shared = _load_shared_config(parser, formatter, explicit_config)
            _apply_config(parser, override_opt, shared, str(explicit_config), constraints)
        elif found := _find_config_file(formatter.prog, toml_path.parent if toml_path is not None else Path.cwd()):
            _apply_config(
                parser,
                override_opt,
                _load_shared_config(parser, formatter, found),
                str(found),
                constraints,
            )
        if config is not None:
            _apply_config(parser, override_opt, config, source, constraints)

        configs.append(
            _Config(
                toml_filename=toml_path,
                toml=raw,
                stdout=formatter.opt.stdout,
                check=formatter.opt.check,
                no_print_diff=formatter.opt.no_print_diff,
                opt=override_opt,
                eol=eol,
            )
        )

    return configs


def _read_input(parser: ArgumentParser, path: Path | None) -> tuple[str, str]:
    if path is None:
        return sys.stdin.read(), "\n"
    try:
        with path.open(encoding="utf-8", newline="") as file_handler:
            raw = file_handler.read()
    except UnicodeDecodeError as exc:
        parser.error(f"{path}: {exc}")
    crlf = raw.count("\r\n")
    # a carriage return TOML does not read is left for the formatter to reject, and a mixed file
    # gets the ending it uses most, ties going to LF
    return raw.replace("\r\n", "\n"), "\r\n" if crlf > raw.count("\n") - crlf else "\n"


_NON_FORMAT_KEYS: Final[frozenset[str]] = frozenset({"inputs", "stdout", "check", "no_print_diff", "config"})


@dataclass(frozen=True)
class _Constraints:
    conversion: Mapping[str, Callable[[TomlValue], TomlValue]]
    allowed: Mapping[str, Iterable[TomlValue]]
    accepts: Mapping[str, type]


def _accepted_types(parser: ArgumentParser, opt: _NAMESPACE_T) -> Mapping[str, type]:
    """Infer plugin settings from defaults because formatters can add flags outside ``SHARED_SETTINGS``."""
    named = {setting.name(): setting.takes for setting in SHARED_SETTINGS}
    return {key: named.get(key) or _written_as(parser.get_default(key)) for key in vars(opt).keys() - _NON_FORMAT_KEYS}


def _written_as(default: TomlValue) -> type:
    if isinstance(default, bool):
        return bool
    return list if isinstance(default, list) else str


def _apply_config(
    parser: ArgumentParser,
    opt: _NAMESPACE_T,
    config: dict[str, TomlValue],
    source: str,
    constraints: _Constraints,
) -> None:
    known = set(vars(opt).keys()) - _NON_FORMAT_KEYS
    for key, raw in config.items():
        if key not in known:
            parser.error(f"{source}: {key}: unknown setting")
        wants = constraints.accepts[key]
        # Python treats `True` as an integer; flags reject other integers.
        if not isinstance(raw, wants) or (wants is not bool and isinstance(raw, bool)):
            parser.error(f"{source}: {key}: {raw!r} is not written as {wants.__name__}")
        try:
            value = constraints.conversion[key](raw) if key in constraints.conversion else raw
        except (ArgumentTypeError, TypeError, ValueError) as exc:
            parser.error(f"{source}: {key}: {exc}")
        if key in constraints.allowed and value not in constraints.allowed[key]:
            choices = ", ".join(repr(choice) for choice in constraints.allowed[key])
            parser.error(f"{source}: {key}: invalid choice: {value!r} (choose from {choices})")
        setattr(opt, key, value)


def _find_config_file(prog: str, start: Path) -> Path | None:
    current = start.resolve()
    while True:
        if (candidate := current / f"{prog}.toml").is_file():
            return candidate
        if (parent := current.parent) == current:
            return None
        current = parent


def _load_shared_config(
    parser: ArgumentParser, formatter: TOMLFormatter[_NAMESPACE_T], path: Path
) -> dict[str, TomlValue]:
    try:
        return formatter.settings_in(path.read_text(encoding="utf-8"), ()) or {}
    except (SyntaxError, TypeError, ValueError, UnicodeDecodeError) as exc:
        parser.error(f"{path}: {exc}")


def build_cli(
    formatter: TOMLFormatter[_NAMESPACE_T],
) -> tuple[ArgumentParser, Mapping[str, Callable[[TomlValue], TomlValue]]]:
    """
    Build the parser without reading arguments, for documentation and embedding.

    The conversion mapping applies the same readers to settings loaded from TOML.
    """
    parser, type_conversion, _ = _make_cli(formatter)
    return parser, type_conversion


def _make_cli(
    formatter: TOMLFormatter[_NAMESPACE_T],
) -> tuple[ArgumentParser, Mapping[str, Callable[[TomlValue], TomlValue]], Sequence[Action]]:
    parser = ArgumentParser(
        formatter_class=ArgumentDefaultsHelpFormatter,
        prog=formatter.prog,
    )
    parser.add_argument(
        "-V",
        "--version",
        action="version",
        help=f"print package version of {formatter.prog}",
        version=f"%(prog)s ({version(formatter.prog)})",
    )

    mode_group = parser.add_argument_group("run mode")
    mode = mode_group.add_mutually_exclusive_group()
    msg = "write formatted TOML to stdout; implied for stdin"
    mode.add_argument("-s", "--stdout", action="store_true", help=msg)
    msg = "fail when an input needs formatting and print its diff"
    mode.add_argument("--check", action="store_true", help=msg)
    mode_group.add_argument(
        "-n",
        "--no-print-diff",
        action="store_true",
        help="suppress diffs in check mode",
    )
    parser.add_argument(
        "--config",
        type=Path,
        default=None,
        help=f"path to a shared {formatter.prog}.toml config file",
        metavar="path",
    )

    # conflict_handler="resolve": released consumers (pyproject-fmt <=2.21.2) re-register
    # these same flags in their add_format_flags, since 1.3.2 didn't define them here.
    # Resolving lets the consumer's identical definition override ours instead of raising
    # ArgumentError, so a fresh resolve of toml-fmt-common doesn't break them
    # (tox-dev/toml-fmt#355).
    format_group = _ArgumentRecorder(
        cast("ArgumentGroup", parser.add_argument_group("formatting behavior", conflict_handler="resolve"))
    )
    for setting in SHARED_SETTINGS:
        setting.add_to(format_group)
    formatter.add_format_flags(format_group)
    type_conversion: Mapping[str, Callable[[TomlValue], TomlValue]] = {
        action.dest: cast("Callable[[TomlValue], TomlValue]", action.type)
        for action in format_group.actions
        if action.type and action.dest
    }
    msg = f"{formatter.filename} file(s) to format, use '-' to read from stdin"
    parser.add_argument(
        "inputs",
        nargs="+",
        type=partial(_toml_path_creator, formatter.filename),
        help=msg,
    )
    return parser, type_conversion, format_group.actions


@dataclass
class _ArgumentRecorder:
    group: ArgumentGroup
    actions: list[Action] = field(default_factory=list)

    def add_argument(self, *name_or_flags: str, **kwargs: object) -> Action:
        action = self.group.add_argument(*name_or_flags, **kwargs)
        self.actions.append(action)
        return action


_COUNT_LIMIT: Final[int] = 10_000
# A larger count asks the formatter to allocate a string no line can hold.


def count_argument(value: TomlValue) -> int:
    """Reject booleans and counts outside 0 through 10,000."""
    # `True` is an integer to Python, and a file that writes one there names no count
    if isinstance(value, bool) or not isinstance(value, (str, int)):
        msg = f"invalid count: {value!r}"
        raise ArgumentTypeError(msg)
    try:
        count = int(value)
    except ValueError as exc:
        msg = f"invalid count: {value!r} due {exc!r}"
        raise ArgumentTypeError(msg) from exc
    if count < 0:
        msg = f"invalid count: {count}, must not be negative"
        raise ArgumentTypeError(msg)
    if count > _COUNT_LIMIT:
        msg = f"invalid count: {count}, must be at most {_COUNT_LIMIT}"
        raise ArgumentTypeError(msg)
    return count


def spacing_argument(value: TomlValue) -> str:
    r"""Accept strings because TOML settings may spell newlines as ``\n``."""
    if not isinstance(value, str):
        msg = f"invalid spacing: {value!r}"
        raise ArgumentTypeError(msg)
    return value.replace("\\n", "\n")


def list_argument(value: TomlValue) -> list[str]:
    """Accept CLI comma lists and TOML string arrays through one reader."""
    if isinstance(value, str):
        return [name for part in _split_outside_quotes(value) if (name := part.strip())]
    read = [item for item in value if isinstance(item, str)] if isinstance(value, Sequence) else []
    if not isinstance(value, Sequence) or len(read) != len(value):
        msg = f"invalid list: {value!r}, every entry names one thing"
        raise ArgumentTypeError(msg)
    return read


def _split_outside_quotes(value: str) -> Iterator[str]:
    """Split on the commas between names, since a name TOML quotes may hold one of its own."""
    quote: str | None = None
    escaped = False
    start = 0
    for at, character in enumerate(value):
        if quote is not None:
            # Basic strings use backslashes for escapes; a doubled backslash consumes itself.
            if escaped:
                escaped = False
            elif quote == '"' and character == "\\":
                escaped = True
            elif character == quote:
                quote = None
        elif character in {'"', "'"}:
            quote = character
        elif character == ",":
            yield value[start:at]
            start = at + 1
    yield value[start:]


_ESCAPES: Final[Mapping[str, str]] = {
    "b": "\b",
    "e": "\x1b",
    "t": "\t",
    "n": "\n",
    "f": "\f",
    "r": "\r",
    '"': '"',
    "\\": "\\",
}


def name_list_argument(value: TomlValue) -> list[str]:
    """Preserve commas and spaces inside TOML-quoted names."""
    if not isinstance(value, str):
        return list_argument(value)
    return [name for part in _split_outside_quotes(value) if (name := _read_name(part.strip()))]


def _read_name(name: str) -> str:
    if name[:1] not in {'"', "'"} or name == name[:1] or name[-1:] != name[:1]:
        return name
    body = name[1:-1]
    if name[0] == "'":
        return body
    read: list[str] = []
    rest = iter(range(len(body)))
    for at in rest:
        if body[at] != "\\":
            read.append(body[at])
            continue
        escape = body[at + 1]
        if escape in {"u", "U"}:
            width = 4 if escape == "u" else 8
            read.append(chr(int(body[at + 2 : at + 2 + width], 16)))
            for _ in range(width + 1):
                next(rest, None)
            continue
        if escape == "x":
            read.append(chr(int(body[at + 2 : at + 4], 16)))
            for _ in range(3):
                next(rest, None)
            continue
        read.append(_ESCAPES[escape])
        next(rest, None)
    return "".join(read)


@dataclass(frozen=True)
class Setting:
    """One formatting setting, named once for the command line and for the file that may hold it."""

    flag: str
    help: str
    takes: type
    default: TomlValue
    convert: Callable[[TomlValue], TomlValue] | None = None
    choices: tuple[str, ...] | None = None
    metavar: str | None = None

    def name(self) -> str:
        """Use argparse's destination spelling for TOML settings."""
        return self.flag.removeprefix("--").replace("-", "_")

    def add_to(self, parser: ArgumentGroup) -> None:
        """Keep parser defaults and TOML conversion metadata on one declaration."""
        action = parser.add_argument(self.flag, default=self.default, help=self.help)
        if self.convert is not None:
            action.type = self.convert
        if self.choices is not None:
            action.choices = list(self.choices)
        if self.metavar is not None:
            action.metavar = self.metavar


# One declaration keeps CLI and file settings under the same constraints.
SHARED_SETTINGS: Final[tuple[Setting, ...]] = (
    Setting("--column-width", "max column width in the TOML file", int, 120, count_argument, metavar="count"),
    Setting("--indent", "number of spaces to use for indentation", int, 2, count_argument, metavar="count"),
    Setting(
        "--table-format",
        "table format: 'short' collapses sub-tables, 'long' expands to [table.subtable]",
        str,
        "short",
        choices=("short", "long"),
    ),
    Setting(
        "--sub-table-spacing",
        r"extra newlines between sub-tables in the same group (e.g. '\n' for one blank line, empty for compact)",
        str,
        "",
        spacing_argument,
    ),
    Setting(
        "--separate-root-table",
        r"extra newlines between root table groups (e.g. '\n' for one blank line, '\n\n' for two)",
        str,
        "\n",
        spacing_argument,
    ),
    Setting("--expand-tables", "comma-separated list of tables to force expand", list, [], list_argument),
    Setting("--collapse-tables", "comma-separated list of tables to force collapse", list, [], list_argument),
    Setting(
        "--skip-wrap-for-keys",
        "comma-separated list of key patterns to skip string wrapping (supports wildcards like '*.parse')",
        list,
        [],
        list_argument,
    ),
)


def _toml_path_creator(filename: str, argument: str) -> Path | None:
    if argument == "-":
        return None
    path = Path(argument).absolute()
    if path.is_dir():
        path /= filename
    if not path.exists():
        msg = "path does not exist"
        raise ArgumentTypeError(msg)
    if not path.is_file():
        msg = "path is not a file"
        raise ArgumentTypeError(msg)
    if not os.access(path, os.R_OK):
        msg = "cannot read path"
        raise ArgumentTypeError(msg)
    return path


def _display_name(path: Path | None) -> str:
    if path is None:
        return "<stdin>"
    try:
        return str(path.relative_to(Path.cwd()))
    except ValueError:
        return str(path)


def _handle_one(formatter: TOMLFormatter[_NAMESPACE_T], config: _Config[_NAMESPACE_T]) -> bool:
    try:
        formatted = formatter.format(config.toml, config.opt)
    except ValueError as exc:  # the formatter rejected the content, e.g. an invalid project.version
        print(f"{_display_name(config.toml_filename)}: {exc}", file=sys.stderr)
        return True
    before = config.toml
    changed = before != formatted
    if config.toml_filename is None or config.stdout:
        print(formatted, end="")
        return changed

    if changed and not config.check:
        config.toml_filename.write_text(formatted, encoding="utf-8", newline=config.eol)
    if config.no_print_diff:
        return changed
    name = _display_name(config.toml_filename)
    if changed:
        print(
            "\n".join(
                _color_diff(
                    difflib.unified_diff(before.splitlines(), formatted.splitlines(), fromfile=name, tofile=name)
                )
            )
        )
    else:
        print(f"no change for {name}")
    return changed


_GREEN: Final[str] = "\u001b[32m"
_RED: Final[str] = "\u001b[31m"
_RESET: Final[str] = "\u001b[0m"


def _color_diff(diff: Iterable[str]) -> Iterable[str]:
    if "NO_COLOR" in os.environ:  # https://no-color.org
        yield from diff
        return
    for line in diff:
        if line.startswith("+"):
            yield f"{_GREEN}{line}{_RESET}"
        elif line.startswith("-"):
            yield f"{_RED}{line}{_RESET}"
        else:
            yield line


# Releases through 1.3.2 import the old name; removing it breaks their next dependency resolve
# (tox-dev/toml-fmt#355).
_build_cli: Final = build_cli

__all__ = [
    "SHARED_SETTINGS",
    "ArgumentGroup",
    "FmtNamespace",
    "Setting",
    "TOMLFormatter",
    "TomlValue",
    "_build_cli",
    "build_cli",
    "list_argument",
    "name_list_argument",
    "run",
]
