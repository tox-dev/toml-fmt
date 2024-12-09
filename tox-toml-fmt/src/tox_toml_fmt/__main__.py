"""Main entry point for the formatter."""

from __future__ import annotations

from typing import TYPE_CHECKING

from toml_fmt_common import ArgumentGroup, FmtNamespace, TOMLFormatter, _build_cli, run  # noqa: PLC2701

from ._lib import Settings, format_toml

if TYPE_CHECKING:
    from argparse import ArgumentParser
    from collections.abc import Sequence


class PyProjectFmtNamespace(FmtNamespace):
    """Formatting arguments."""


class ToxTOMLFormatter(TOMLFormatter[PyProjectFmtNamespace]):
    """Format pyproject.toml."""

    def __init__(self) -> None:
        """Create a formatter."""
        super().__init__(PyProjectFmtNamespace())

    @property
    def prog(self) -> str:
        """:return: program name"""
        return "tox-toml-fmt"

    @property
    def filename(self) -> str:
        """:return: filename operating on"""
        return "pyproject.toml"

    def add_format_flags(self, parser: ArgumentGroup) -> None:
        """
        Additional formatter  config.

        :param parser: parser to operate on.
        """

    @property
    def override_cli_from_section(self) -> tuple[str, ...]:
        """:return: path where config overrides live"""
        return ("tox-toml-fmt",)

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
        )
        return format_toml(text, settings)


def runner(args: Sequence[str] | None = None) -> int:
    """
    Run the formatter.

    :param args: CLI arguments
    :return: exit code
    """
    return run(ToxTOMLFormatter(), args)


def _build_our_cli() -> ArgumentParser:
    return _build_cli(ToxTOMLFormatter())[0]  # pragma: no cover


__all__ = [
    "runner",
]

if __name__ == "__main__":
    raise SystemExit(runner())
