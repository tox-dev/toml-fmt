"""Keep CLI parsing in Python and TOML rewriting in Rust."""

from __future__ import annotations

from argparse import ArgumentParser, ArgumentTypeError
from typing import TYPE_CHECKING, Final

from pyproject_fmt._lib import Settings, format_toml, settings_in
from toml_fmt_common import ArgumentGroup, FmtNamespace, TOMLFormatter, TomlValue, build_cli, run

if TYPE_CHECKING:
    from collections.abc import Sequence


_MINOR_LIMIT: Final[int] = 255
_PYTHON_MAJOR: Final[int] = 3
_MIN_SUPPORTED_PYTHON: Final[tuple[int, int]] = (_PYTHON_MAJOR, 10)


class _PyProjectFmtNamespace(FmtNamespace):
    """Give project-specific argparse fields concrete types."""

    keep_full_version: bool
    max_supported_python: tuple[int, int]
    generate_python_version_classifiers: bool


class _PyProjectFormatter(TOMLFormatter[_PyProjectFmtNamespace]):
    """Bind the shared CLI to the pyproject-fmt Rust extension."""

    def __init__(self) -> None:
        """Start with the namespace argparse fills for each input."""
        super().__init__(_PyProjectFmtNamespace())

    @property
    def prog(self) -> str:
        """Match the distribution name used for version lookup."""
        return "pyproject-fmt"

    @property
    def filename(self) -> str:
        """Restrict positional directories to ``pyproject.toml``."""
        return "pyproject.toml"

    @staticmethod
    def add_format_flags(parser: ArgumentGroup) -> None:
        """Add project metadata options outside the shared formatter settings."""
        msg = "retain redundant .0 components in dependency versions"
        parser.add_argument("--keep-full-version", action="store_true", help=msg)
        msg = "retain Python version classifiers instead of deriving them from requires-python"
        parser.add_argument(
            "--no-generate-python-version-classifiers",
            action="store_false",
            dest="generate_python_version_classifiers",
            help=msg,
        )

        parser.add_argument(
            "--max-supported-python",
            metavar="major.minor",
            type=_version_argument,
            default=(3, 14),
            help="latest Python version the project supports (e.g. 3.14)",
        )

    @property
    def override_cli_from_section(self) -> tuple[str, ...]:
        """Keep per-file overrides under ``tool.pyproject-fmt``."""
        return "tool", "pyproject-fmt"

    @staticmethod
    def settings_in(text: str, path: Sequence[str]) -> dict[str, TomlValue] | None:
        """Use the Rust parser so TOML 1.1 settings remain visible."""
        return settings_in(text, list(path))

    @staticmethod
    def format(text: str, opt: _PyProjectFmtNamespace) -> str:
        """Keep Python responsible for CLI state and Rust responsible for TOML edits."""
        settings = Settings(
            column_width=opt.column_width,
            indent=opt.indent,
            keep_full_version=opt.keep_full_version,
            max_supported_python=opt.max_supported_python,
            min_supported_python=_MIN_SUPPORTED_PYTHON,
            generate_python_version_classifiers=opt.generate_python_version_classifiers,
            table_format=opt.table_format,
            sub_table_spacing=opt.sub_table_spacing,
            separate_root_table=opt.separate_root_table,
            expand_tables=opt.expand_tables,
            collapse_tables=opt.collapse_tables,
            skip_wrap_for_keys=opt.skip_wrap_for_keys,
        )
        return format_toml(text, settings)


def _version_argument(got: str) -> tuple[int, int]:
    try:
        major_text, minor_text = got.split(".")
    except ValueError as exc:
        msg = f"invalid version: {got}, must be e.g. 3.14"
        raise ArgumentTypeError(msg) from exc
    try:
        major, minor = int(major_text), int(minor_text)
    except ValueError as exc:
        msg = f"invalid version: {got} due {exc!r}, must be e.g. 3.14"
        raise ArgumentTypeError(msg) from exc
    # the classifiers this generates name Python 3, and the formatter holds a minor as a byte
    if major != _PYTHON_MAJOR or not 0 <= minor <= _MINOR_LIMIT:
        msg = (
            f"invalid version: {got}, must name a Python {_PYTHON_MAJOR} minor "
            f"from {_PYTHON_MAJOR}.0 to {_PYTHON_MAJOR}.{_MINOR_LIMIT}"
        )
        raise ArgumentTypeError(msg)
    # A window that ends before it starts names no release and would empty the classifier set.
    if (major, minor) < _MIN_SUPPORTED_PYTHON:
        msg = f"invalid version: {got}, must not precede {'.'.join(str(x) for x in _MIN_SUPPORTED_PYTHON)}"
        raise ArgumentTypeError(msg)
    return major, minor


def runner(args: Sequence[str] | None = None) -> int:
    """
    Use a supplied argument list for embedding; the console entry reads ``sys.argv``.

    Return 1 after a change or rejection, and 0 when inputs match.
    """
    return run(_PyProjectFormatter(), args)


def build_parser() -> ArgumentParser:
    """Build the parser without reading arguments, for documentation tooling."""
    return build_cli(_PyProjectFormatter())[0]


__all__ = [
    "build_parser",
    "runner",
]

if __name__ == "__main__":
    raise SystemExit(runner())
