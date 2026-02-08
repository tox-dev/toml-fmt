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
    _ArgumentGroup,  # noqa: PLC2701
)
from collections import deque
from copy import deepcopy
from dataclasses import dataclass
from functools import partial
from importlib.metadata import version
from pathlib import Path
from typing import TYPE_CHECKING, Any, Generic, TypeVar, cast

if TYPE_CHECKING:
    from collections.abc import Callable, Iterable, Mapping, Sequence

if sys.version_info >= (3, 11):  # pragma: >=3.11 cover
    import tomllib
else:  # pragma: <3.11 cover
    import tomli as tomllib

ArgumentGroup = _ArgumentGroup


class FmtNamespace(Namespace):
    """Options for pyproject-fmt tool."""

    inputs: list[Path]
    stdout: bool
    check: bool
    no_print_diff: bool

    column_width: int
    indent: int


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
        """:returns: name of the application (must be same as the package name)"""
        raise NotImplementedError

    @property
    @abstractmethod
    def filename(self) -> str:
        """:returns: name of the file type it formats"""
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
    return 1 if any(results) else 0  # exit with non success on change


@dataclass(frozen=True)
class _Config(Generic[T]):
    """Configuration flags for the formatting."""

    toml_filename: Path | None  # path to the toml file or None if stdin
    toml: str  # the toml file content
    stdout: bool  # push to standard out, implied if reading from stdin
    check: bool  # check only
    no_print_diff: bool  # don't print diff
    opt: T


def _cli_args(info: TOMLFormatter[T], args: Sequence[str]) -> list[_Config[T]]:
    """
    Load the tools options.

    :param info: information
    :param args: CLI arguments
    :return: the parsed options
    """
    parser, type_conversion = _build_cli(info)
    parser.parse_args(namespace=info.opt, args=args)
    res = []
    for pyproject_toml in info.opt.inputs:
        raw_pyproject_toml = sys.stdin.read() if pyproject_toml is None else pyproject_toml.read_text(encoding="utf-8")
        config: dict[str, Any] | None = tomllib.loads(raw_pyproject_toml)

        parts = deque(info.override_cli_from_section)
        while parts:  # pragma: no branch
            part = parts.popleft()
            if not isinstance(config, dict) or part not in config:
                config = None
                break
            config = config[part]
        override_opt = deepcopy(info.opt)
        if isinstance(config, dict):
            for key in set(vars(override_opt).keys()) - {"inputs", "stdout", "check", "no_print_diff"}:
                if key in config:
                    raw = config[key]
                    converted = type_conversion[key](raw) if key in type_conversion else raw
                    setattr(override_opt, key, converted)
        res.append(
            _Config(
                toml_filename=pyproject_toml,
                toml=raw_pyproject_toml,
                stdout=info.opt.stdout,
                check=info.opt.check,
                no_print_diff=info.opt.no_print_diff,
                opt=override_opt,
            )
        )

    return res


def _build_cli(of: TOMLFormatter[T]) -> tuple[ArgumentParser, Mapping[str, Callable[[Any], Any]]]:
    parser = ArgumentParser(
        formatter_class=ArgumentDefaultsHelpFormatter,
        prog=of.prog,
    )
    parser.add_argument(
        "-V",
        "--version",
        action="version",
        help="print package version of pyproject_fmt",
        version=f"%(prog)s ({version(of.prog)})",
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

    format_group = parser.add_argument_group("formatting behavior")
    format_group.add_argument(
        "--column-width",
        type=int,
        default=120,
        help="max column width in the TOML file",
        metavar="count",
    )
    format_group.add_argument(
        "--indent",
        type=int,
        default=2,
        help="number of spaces to use for indentation",
        metavar="count",
    )
    of.add_format_flags(format_group)
    type_conversion: Mapping[str, Callable[[Any], Any]] = {
        a.dest: cast("Callable[[Any], Any]", a.type)
        for a in format_group._actions  # noqa: SLF001
        if a.type and a.dest
    }
    msg = "pyproject.toml file(s) to format, use '-' to read from stdin"
    parser.add_argument(
        "inputs",
        nargs="+",
        type=partial(_toml_path_creator, of.filename),
        help=msg,
    )
    return parser, type_conversion


def _toml_path_creator(filename: str, argument: str) -> Path | None:
    """
    Validate that toml can be formatted.

    :param filename: name of the toml file
    :param argument: the string argument passed in
    :return: the pyproject.toml path or None if stdin
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
    if not os.access(path, os.W_OK):
        msg = "cannot write path"
        raise ArgumentTypeError(msg)
    return path


def _handle_one(info: TOMLFormatter[T], config: _Config[T]) -> bool:
    formatted = info.format(config.toml, config.opt)
    before = config.toml
    changed = before != formatted
    if config.toml_filename is None or config.stdout:  # when reading from stdin or writing to stdout, print new format
        print(formatted, end="")  # noqa: T201
        return changed

    if before != formatted and not config.check:
        config.toml_filename.write_text(formatted, encoding="utf-8", newline="\n")
    if config.no_print_diff:
        return changed
    try:
        name = str(config.toml_filename.relative_to(Path.cwd()))
    except ValueError:
        name = str(config.toml_filename)
    diff: Iterable[str] = []
    if changed:
        diff = difflib.unified_diff(before.splitlines(), formatted.splitlines(), fromfile=name, tofile=name)

    if diff:
        diff = _color_diff(diff)
        print("\n".join(diff))  # print diff on change  # noqa: T201
    else:
        print(f"no change for {name}")  # noqa: T201
    return changed


GREEN = "\u001b[32m"
RED = "\u001b[31m"
RESET = "\u001b[0m"


def _color_diff(diff: Iterable[str]) -> Iterable[str]:
    """
    Visualize difference with colors.

    :param diff: the diff lines
    """
    for line in diff:
        if line.startswith("+"):
            yield f"{GREEN}{line}{RESET}"
        elif line.startswith("-"):
            yield f"{RED}{line}{RESET}"
        else:
            yield line


__all__ = [
    "ArgumentGroup",
    "FmtNamespace",
    "TOMLFormatter",
    "run",
]
