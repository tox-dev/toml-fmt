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

    table_format: str
    expand_tables: list[str]
    collapse_tables: list[str]
    skip_wrap_for_keys: list[str]


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
        return "tox.toml"

    def add_format_flags(self, parser: ArgumentGroup) -> None:  # noqa: PLR6301
        """
        Additional formatter  config.

        :param parser: parser to operate on.
        """

        def _list_argument(value: str | list[str]) -> list[str]:
            if isinstance(value, list):
                return value
            return [x.strip() for x in value.split(",") if x.strip()]

        parser.add_argument(
            "--table-format",
            choices=["short", "long"],
            default="short",
            help="table format: 'short' collapses sub-tables, 'long' expands to [table.subtable]",
        )
        parser.add_argument(
            "--expand-tables",
            type=_list_argument,
            default=[],
            help="comma-separated list of tables to force expand (e.g. 'env.test')",
        )
        parser.add_argument(
            "--collapse-tables",
            type=_list_argument,
            default=[],
            help="comma-separated list of tables to force collapse (e.g. 'env.lint')",
        )
        parser.add_argument(
            "--skip-wrap-for-keys",
            type=_list_argument,
            default=[],
            help="comma-separated list of key patterns to skip string wrapping (e.g. '*.commands')",
        )

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
            table_format=opt.table_format,
            expand_tables=opt.expand_tables,
            collapse_tables=opt.collapse_tables,
            skip_wrap_for_keys=opt.skip_wrap_for_keys,
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
