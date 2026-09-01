"""Main entry point for the formatter."""

from __future__ import annotations

from argparse import ArgumentParser, ArgumentTypeError
from typing import TYPE_CHECKING, Final

from pyproject_fmt._lib import Settings, format_toml, settings_in
from toml_fmt_common import ArgumentGroup, FmtNamespace, TOMLFormatter, TomlValue, build_cli, run

if TYPE_CHECKING:
    from collections.abc import Sequence


_MINOR_LIMIT: Final[int] = 255
_MIN_SUPPORTED_PYTHON: Final[tuple[int, int]] = (3, 10)
"""What a project supports where it says nothing about it, which is also the oldest release the
formatter writes a classifier for."""


class PyProjectFmtNamespace(FmtNamespace):
    """Formatting arguments."""

    keep_full_version: bool
    max_supported_python: tuple[int, int]
    generate_python_version_classifiers: bool


class PyProjectFormatter(TOMLFormatter[PyProjectFmtNamespace]):
    """Format pyproject.toml."""

    def __init__(self) -> None:
        """Create a formatter."""
        super().__init__(PyProjectFmtNamespace())

    @property
    def prog(self) -> str:
        """Program name."""
        return "pyproject-fmt"

    @property
    def filename(self) -> str:
        """Filename operating on."""
        return "pyproject.toml"

    def add_format_flags(self, parser: ArgumentGroup) -> None:  # ruff: ignore[no-self-use]  # the formatter API declares it a method
        """
        Additional formatter  config.

        :param parser: parser to operate on.
        """
        msg = "keep full dependency versions - do not remove redundant .0 from versions"
        parser.add_argument("--keep-full-version", action="store_true", help=msg)
        msg = "do not generate Python version classifiers based on requires-python"
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
        """Path where config overrides live."""
        return "tool", "pyproject-fmt"

    def settings_in(self, text: str, path: Sequence[str]) -> dict[str, TomlValue] | None:  # ruff: ignore[no-self-use]  # the formatter API declares it a method
        """
        Read the settings the text writes under a table, with the parser that reads the file itself.

        :param text: the TOML source to read
        :param path: the table the settings are written under
        :return: the settings, or ``None`` where the text writes no such table
        """
        return settings_in(text, list(path))

    def format(self, text: str, opt: PyProjectFmtNamespace) -> str:  # ruff: ignore[no-self-use]  # the formatter API declares it a method
        """
        Perform the formatting.

        :param text: content to operate on
        :param opt: formatter config
        :return: formatted text
        """
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
    parts = got.split(".")
    if len(parts) != 2:  # ruff: ignore[magic-value-comparison]  # a major and a minor
        msg = f"invalid version: {got}, must be e.g. 3.14"
        raise ArgumentTypeError(msg)
    try:
        major, minor = int(parts[0]), int(parts[1])
    except ValueError as exc:
        msg = f"invalid version: {got} due {exc!r}, must be e.g. 3.14"
        raise ArgumentTypeError(msg) from exc
    # the classifiers this generates name Python 3, and the formatter holds a minor as a byte
    if major != 3 or not 0 <= minor <= _MINOR_LIMIT:  # ruff: ignore[magic-value-comparison]  # Python 3 only
        msg = f"invalid version: {got}, must name a Python 3 minor from 3.0 to 3.{_MINOR_LIMIT}"
        raise ArgumentTypeError(msg)
    # a window that ends before it starts names no release, and would drop every classifier
    if (major, minor) < _MIN_SUPPORTED_PYTHON:
        msg = f"invalid version: {got}, must not precede {'.'.join(str(x) for x in _MIN_SUPPORTED_PYTHON)}"
        raise ArgumentTypeError(msg)
    return major, minor


def runner(args: Sequence[str] | None = None) -> int:
    """
    Run the formatter.

    :param args: CLI arguments
    :return: exit code
    """
    return run(PyProjectFormatter(), args)


def _build_our_cli() -> ArgumentParser:
    return build_cli(PyProjectFormatter())[0]  # pragma: no cover


__all__ = [
    "runner",
]

if __name__ == "__main__":
    raise SystemExit(runner())
