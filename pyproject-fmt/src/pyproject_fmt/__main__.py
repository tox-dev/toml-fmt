"""Main entry point for the formatter."""

from __future__ import annotations

from argparse import ArgumentParser, ArgumentTypeError
from typing import Sequence

from toml_fmt_common import ArgumentGroup, FmtNamespace, TOMLFormatter, _build_cli, run  # noqa: PLC2701

from ._lib import Settings, format_toml


class PyProjectFmtNamespace(FmtNamespace):
    """Formatting arguments."""

    keep_full_version: bool
    max_supported_python: tuple[int, int]


class PyProjectFormatter(TOMLFormatter[PyProjectFmtNamespace]):
    """Format pyproject.toml."""

    def __init__(self) -> None:
        """Create a formatter."""
        super().__init__(PyProjectFmtNamespace())

    @property
    def prog(self) -> str:
        """:return: program name"""
        return "pyproject-fmt"

    @property
    def filename(self) -> str:
        """:return: filename operating on"""
        return "pyproject.toml"

    def add_format_flags(self, parser: ArgumentGroup) -> None:  # noqa: PLR6301
        """
        Additional formatter  config.

        :param parser: parser to operate on.
        """
        msg = "keep full dependency versions - do not remove redundant .0 from versions"
        parser.add_argument("--keep-full-version", action="store_true", help=msg)

        def _version_argument(got: str) -> tuple[int, int]:
            parts = got.split(".")
            if len(parts) != 2:  # noqa: PLR2004
                err = f"invalid version: {got}, must be e.g. 3.13"
                raise ArgumentTypeError(err)
            try:
                return int(parts[0]), int(parts[1])
            except ValueError as exc:
                err = f"invalid version: {got} due {exc!r}, must be e.g. 3.13"
                raise ArgumentTypeError(err) from exc

        parser.add_argument(
            "--max-supported-python",
            metavar="minor.major",
            type=_version_argument,
            default=(3, 13),
            help="latest Python version the project supports (e.g. 3.13)",
        )

    @property
    def override_cli_from_section(self) -> tuple[str, ...]:
        """:return: path where config overrides live"""
        return "tool", "pyproject-fmt"

    def format(self, text: str, opt: PyProjectFmtNamespace) -> str:  # noqa: PLR6301
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
            min_supported_python=(3, 9),  # default for when the user didn't specify via requires-python
        )
        return format_toml(text, settings)


def runner(args: Sequence[str] | None = None) -> int:
    """
    Run the formatter.

    :param args: CLI arguments
    :return: exit code
    """
    return run(PyProjectFormatter(), args)


def _build_our_cli() -> ArgumentParser:
    return _build_cli(PyProjectFormatter())[0]  # pragma: no cover


__all__ = [
    "runner",
]

if __name__ == "__main__":
    raise SystemExit(runner())
