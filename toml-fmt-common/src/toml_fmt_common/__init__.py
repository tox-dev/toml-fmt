"""Common logic for a TOML formatter."""

from __future__ import annotations

import difflib
import os
import sys
from abc import ABC, abstractmethod
from argparse import (
    ArgumentDefaultsHelpFormatter,
    ArgumentParser,
    ArgumentTypeError,
    Namespace,
    _ArgumentGroup,  # ruff: ignore[import-private-name]  # argparse names the group type nowhere else
)
from collections.abc import Mapping, Sequence
from copy import deepcopy
from dataclasses import dataclass
from functools import partial
from importlib.metadata import version
from pathlib import Path
from typing import TYPE_CHECKING, Final, Generic, TypeAlias, TypeVar, cast

if TYPE_CHECKING:
    from collections.abc import Callable, Iterable, Iterator

ArgumentGroup: TypeAlias = _ArgumentGroup

#: What a TOML document holds, and so what a setting read from one arrives as.
TomlValue: TypeAlias = "bool | int | float | str | Sequence[TomlValue] | Mapping[str, TomlValue]"


class FmtNamespace(Namespace):
    """Options for pyproject-fmt tool."""

    inputs: list[Path]
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


T = TypeVar("T", bound=FmtNamespace)


class TOMLFormatter(ABC, Generic[T]):
    """API for a TOML formatter."""

    def __init__(self, opt: T) -> None:
        """
        Create a new TOML formatter.

        :param opt: configuration options
        """
        self.opt: T = opt

    @property
    @abstractmethod
    def prog(self) -> str:
        """Name of the application (must be same as the package name)."""
        raise NotImplementedError

    @property
    @abstractmethod
    def filename(self) -> str:
        """Name of the file type it formats."""
        raise NotImplementedError

    @abstractmethod
    def add_format_flags(self, parser: ArgumentGroup) -> None:
        """
         Add any additional flags to configure the formatter.

        :param parser: the parser to operate on
        """
        raise NotImplementedError

    @property
    @abstractmethod
    def override_cli_from_section(self) -> tuple[str, ...]:
        """
         Allow overriding CLI defaults from within the TOML files this section.

        :returns: the section path
        """
        raise NotImplementedError

    @abstractmethod
    def settings_in(self, text: str, path: Sequence[str]) -> dict[str, TomlValue] | None:
        """
        Read the settings the text writes under a table, with the parser that reads the file itself.

        A second reader of an older TOML would drop a file's own configuration on a value it cannot
        read, and format the file as though none had been written.

        :param text: the TOML source to read
        :param path: the table the settings are written under
        :return: the settings, or ``None`` where the text writes no such table
        :raises SyntaxError: if the text is not a TOML document
        :raises ValueError: if a setting is written in a form no setting takes
        """
        raise NotImplementedError

    @abstractmethod
    def format(self, text: str, opt: T) -> str:
        """
        Run the formatter.

        :param text: the TOML text to format
        :param opt: the flags to format with
        :returns: the formatted TOML text
        """
        raise NotImplementedError


def run(info: TOMLFormatter[T], args: Sequence[str] | None = None) -> int:
    """
    Run the formatter.

    :param info: information specific to the current formatter
    :param args: command line arguments, by default use sys.argv[1:]
    :return: exit code - 0 means already formatted correctly, otherwise 1
    """
    configs = _cli_args(info, sys.argv[1:] if args is None else args)
    results = [_handle_one(info, config) for config in configs]
    return 1 if any(results) else 0  # exit with non success on change or rejection


@dataclass(frozen=True)
class _Config(Generic[T]):
    """Configuration flags for the formatting."""

    toml_filename: Path | None  # path to the toml file or None if stdin
    toml: str  # the toml file content
    stdout: bool  # push to standard out, implied if reading from stdin
    check: bool  # check only
    no_print_diff: bool  # don't print diff
    opt: T
    eol: str  # line ending to write the file back with


def _check_write_permission(parser: ArgumentParser, opt: FmtNamespace) -> None:
    if opt.stdout or opt.check:
        return
    for toml_path in opt.inputs:
        if toml_path is not None and not os.access(toml_path, os.W_OK):
            parser.error(f"argument inputs: cannot write path {toml_path}")


def _cli_args(info: TOMLFormatter[T], args: Sequence[str]) -> list[_Config[T]]:
    """
    Load the tools options.

    :param info: information
    :param args: CLI arguments
    :return: the parsed options
    """
    parser, type_conversion = build_cli(info)
    parser.parse_args(namespace=info.opt, args=args)
    if (explicit_config := info.opt.config) is not None and not explicit_config.is_file():
        parser.error(f"config file does not exist: {explicit_config}")
    _check_write_permission(parser, info.opt)
    held = _Constraints(
        conversion=type_conversion,
        # a value read from a file has to be one the command line would have accepted
        allowed={
            action.dest: action.choices
            for action in parser._actions  # ruff: ignore[private-member-access]  # argparse lists them nowhere else
            if action.choices
        },
        accepts=_accepted_types(parser, info.opt),
    )
    configs = []
    for toml_path in info.opt.inputs:
        raw, eol = _read_input(parser, toml_path)
        source = _display_name(toml_path)
        try:
            config = info.settings_in(raw, info.override_cli_from_section)
        except SyntaxError:
            # the formatter reads the same source next and reports on it in its own words, against
            # the file rather than against one setting
            config = None
        except ValueError as exc:
            parser.error(f"{source}: {exc}")
        override_opt = deepcopy(info.opt)
        if explicit_config is not None:
            shared = _load_shared_config(parser, info, explicit_config)
            _apply_config(parser, override_opt, shared, str(explicit_config), held)
        elif found := _find_config_file(info.prog, toml_path.parent if toml_path is not None else Path.cwd()):
            _apply_config(parser, override_opt, _load_shared_config(parser, info, found), str(found), held)
        if config is not None:
            _apply_config(parser, override_opt, config, source, held)

        configs.append(
            _Config(
                toml_filename=toml_path,
                toml=raw,
                stdout=info.opt.stdout,
                check=info.opt.check,
                no_print_diff=info.opt.no_print_diff,
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


_NON_FORMAT_KEYS = frozenset({"inputs", "stdout", "check", "no_print_diff", "config"})


@dataclass(frozen=True)
class _Constraints:
    """What a setting has to satisfy, whether it was given on the command line or read from a file."""

    conversion: Mapping[str, Callable[[TomlValue], TomlValue]]
    allowed: Mapping[str, Iterable[TomlValue]]
    accepts: Mapping[str, type]


def _accepted_types(parser: ArgumentParser, opt: T) -> Mapping[str, type]:
    """
    Read the TOML type of each setting: what the schema says it is, or what it defaults to.

    A formatter may add a flag of its own without naming it here; what it defaults to says whether a
    file writes it as a flag or a list, and anything else is written as text.
    """
    named = {setting.name(): setting.takes for setting in SHARED_SETTINGS}
    return {key: named.get(key) or _written_as(parser.get_default(key)) for key in vars(opt).keys() - _NON_FORMAT_KEYS}


def _written_as(default: TomlValue) -> type:
    if isinstance(default, bool):
        return bool
    return list if isinstance(default, list) else str


def _apply_config(
    parser: ArgumentParser, opt: T, config: dict[str, TomlValue], source: str, held: _Constraints
) -> None:
    """Read the settings a TOML table holds, under the same constraints the command line applies."""
    known = set(vars(opt).keys()) - _NON_FORMAT_KEYS
    for key, raw in config.items():
        if key not in known:
            parser.error(f"{source}: {key}: unknown setting")
        wants = held.accepts[key]
        # `True` is an integer to Python, so a flag's value is only ever the one it was written as
        if not isinstance(raw, wants) or (wants is not bool and isinstance(raw, bool)):
            parser.error(f"{source}: {key}: {raw!r} is not written as {wants.__name__}")
        try:
            value = held.conversion[key](raw) if key in held.conversion else raw
        except (ArgumentTypeError, TypeError, ValueError) as exc:
            parser.error(f"{source}: {key}: {exc}")
        if key in held.allowed and value not in held.allowed[key]:
            choices = ", ".join(repr(choice) for choice in held.allowed[key])
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


def _load_shared_config(parser: ArgumentParser, info: TOMLFormatter[T], path: Path) -> dict[str, TomlValue]:
    try:
        return info.settings_in(path.read_text(encoding="utf-8"), ()) or {}
    except (SyntaxError, ValueError, UnicodeDecodeError) as exc:
        parser.error(f"{path}: {exc}")


def build_cli(formatter: TOMLFormatter[T]) -> tuple[ArgumentParser, Mapping[str, Callable[[TomlValue], TomlValue]]]:
    """
    Build the command line the formatter is driven by.

    :param formatter: the formatter to build the CLI for
    :return: the parser, and what reads each setting the parser holds
    """
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
    msg = "print the formatted TOML to the stdout, implied if reading from stdin"
    mode.add_argument("-s", "--stdout", action="store_true", help=msg)
    msg = "check and fail if any input would be formatted, printing any diffs"
    mode.add_argument("--check", action="store_true", help=msg)
    mode_group.add_argument(
        "-n",
        "--no-print-diff",
        action="store_true",
        help="Flag indicating to print diff for the check mode",
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
    format_group = parser.add_argument_group("formatting behavior", conflict_handler="resolve")
    for setting in SHARED_SETTINGS:
        setting.add_to(format_group)
    formatter.add_format_flags(format_group)
    type_conversion: Mapping[str, Callable[[TomlValue], TomlValue]] = {
        action.dest: cast("Callable[[TomlValue], TomlValue]", action.type)
        for action in format_group._actions  # ruff: ignore[private-member-access]  # argparse lists them nowhere else
        if action.type and action.dest
    }
    msg = f"{formatter.filename} file(s) to format, use '-' to read from stdin"
    parser.add_argument(
        "inputs",
        nargs="+",
        type=partial(_toml_path_creator, formatter.filename),
        help=msg,
    )
    return parser, type_conversion


_COUNT_LIMIT: Final[int] = 10_000
"""No line or indent a formatter writes runs this wide, and a count beyond it only asks the
formatter to build a string nothing can hold."""


def count_argument(value: TomlValue) -> int:
    """Read a count of columns or spaces, which the formatter holds as an unsigned number."""
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
    r"""Convert literal ``\n`` sequences to actual newlines."""
    if not isinstance(value, str):
        msg = f"invalid spacing: {value!r}"
        raise ArgumentTypeError(msg)
    return value.replace("\\n", "\n")


def list_argument(value: TomlValue) -> list[str]:
    """Convert a comma-separated string or list to a list of stripped strings."""
    if isinstance(value, str):
        return [held for part in _split_outside_quotes(value) if (held := part.strip())]
    read = [held for held in value if isinstance(held, str)] if isinstance(value, Sequence) else []
    if not isinstance(value, Sequence) or len(read) != len(value):
        msg = f"invalid list: {value!r}, every entry names one thing"
        raise ArgumentTypeError(msg)
    return read


def _split_outside_quotes(value: str) -> Iterator[str]:
    """Split on the commas between names, since a name TOML quotes may hold one of its own."""
    quote: str | None = None
    escaped = False
    start = 0
    for at, held in enumerate(value):
        if quote is not None:
            # only a basic string reads a backslash as opening an escape, and one before another
            # escapes it rather than what follows
            if escaped:
                escaped = False
            elif quote == '"' and held == "\\":
                escaped = True
            elif held == quote:
                quote = None
        elif held in {'"', "'"}:
            quote = held
        elif held == ",":
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
    """Read a list of literal names, where a name TOML quotes may hold a comma or a space of its own."""
    if not isinstance(value, str):
        return list_argument(value)
    return [held for part in _split_outside_quotes(value) if (held := _read_name(part.strip()))]


def _read_name(name: str) -> str:
    """Read the one name the text writes, in whichever of TOML's forms it wrote it."""
    quoted = len(name) >= 2 and name[0] == name[-1]  # ruff: ignore[magic-value-comparison]  # an open and a close
    if not quoted or name[0] not in {'"', "'"}:
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
        held = body[at + 1]
        if held in {"u", "U"}:
            width = 4 if held == "u" else 8
            read.append(chr(int(body[at + 2 : at + 2 + width], 16)))
            for _ in range(width + 1):
                next(rest, None)
            continue
        if held == "x":
            read.append(chr(int(body[at + 2 : at + 4], 16)))
            for _ in range(3):
                next(rest, None)
            continue
        read.append(_ESCAPES[held])
        next(rest, None)
    return "".join(read)


@dataclass(frozen=True)
class Setting:
    """One formatting setting, named once for the command line and for the file that may hold it."""

    flag: str
    help: str
    #: The TOML type a file writes the setting as.
    takes: type
    default: TomlValue
    #: What reads the written value, where the setting takes more than the type it is written as.
    convert: Callable[[TomlValue], TomlValue] | None = None
    choices: tuple[str, ...] | None = None
    metavar: str | None = None

    def name(self) -> str:
        """Give the name the file and the namespace hold it under."""
        return self.flag.removeprefix("--").replace("-", "_")

    def add_to(self, parser: ArgumentGroup) -> None:
        """Register the flag that reads this setting from the command line."""
        action = parser.add_argument(self.flag, default=self.default, help=self.help)
        if self.convert is not None:
            action.type = self.convert
        if self.choices is not None:
            action.choices = list(self.choices)
        if self.metavar is not None:
            action.metavar = self.metavar


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
"""The settings every formatter reads, so the command line and the file agree on each one."""


def _toml_path_creator(filename: str, argument: str) -> Path | None:
    """
    Validate that toml can be formatted.

    :param filename: name of the toml file
    :param argument: the string argument passed in
    :return: the file path, or None for stdin
    :raises ArgumentTypeError: invalid argument
    """
    if argument == "-":
        return None  # stdin, no further validation needed
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


def _handle_one(info: TOMLFormatter[T], config: _Config[T]) -> bool:
    try:
        formatted = info.format(config.toml, config.opt)
    except ValueError as exc:  # the formatter rejected the content, e.g. an invalid project.version
        print(f"{_display_name(config.toml_filename)}: {exc}", file=sys.stderr)
        return True
    before = config.toml
    changed = before != formatted
    if config.toml_filename is None or config.stdout:  # when reading from stdin or writing to stdout, print new format
        print(formatted, end="")
        return changed

    if changed and not config.check:
        config.toml_filename.write_text(formatted, encoding="utf-8", newline=config.eol)
    if config.no_print_diff:
        return changed
    name = _display_name(config.toml_filename)
    diff: Iterable[str] = []
    if changed:
        diff = difflib.unified_diff(before.splitlines(), formatted.splitlines(), fromfile=name, tofile=name)

    if diff:
        diff = _color_diff(diff)
        print("\n".join(diff))
    else:
        print(f"no change for {name}")
    return changed


_GREEN: Final[str] = "\u001b[32m"
_RED: Final[str] = "\u001b[31m"
_RESET: Final[str] = "\u001b[0m"


def _color_diff(diff: Iterable[str]) -> Iterable[str]:
    """
    Visualize difference with colors.

    :param diff: the diff lines
    """
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


# Backwards-compatibility alias: build_cli was named _build_cli through 1.3.2 and every
# released pyproject-fmt/tox-toml-fmt imports that name. Keep it so a fresh resolve of
# this package does not break already-published consumers (tox-dev/toml-fmt#355).
_build_cli = build_cli

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
