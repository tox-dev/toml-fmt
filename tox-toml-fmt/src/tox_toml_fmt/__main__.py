"""Main entry point for the formatter."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from toml_fmt_common import ArgumentGroup, FmtNamespace, TOMLFormatter, build_cli, name_list_argument, run

from ._lib import Settings, format_toml, settings_in

if TYPE_CHECKING:
    from argparse import ArgumentParser
    from collections.abc import Sequence


class PyProjectFmtNamespace(FmtNamespace):
    """Formatting arguments."""

    pin_envs: list[str]


class ToxTOMLFormatter(TOMLFormatter[PyProjectFmtNamespace]):
    """Format pyproject.toml."""

    def __init__(self) -> None:
        """Create a formatter."""
        super().__init__(PyProjectFmtNamespace())

    @property
    def prog(self) -> str:
        """Program name."""
        return "tox-toml-fmt"

    @property
    def filename(self) -> str:
        """Filename operating on."""
        return "tox.toml"

    def add_format_flags(self, parser: ArgumentGroup) -> None:  # ruff: ignore[no-self-use]
        """
        Additional formatter  config.

        :param parser: parser to operate on.
        """
        parser.add_argument(
            "--pin-env",
            type=name_list_argument,
            default=[],
            dest="pin_envs",
            help="environments whose tables are written first (comma separated)",
        )

    @property
    def override_cli_from_section(self) -> tuple[str, ...]:
        """Path where config overrides live."""
        return ("tox-toml-fmt",)

    def settings_in(self, text: str, path: Sequence[str]) -> dict[str, Any] | None:  # ruff: ignore[no-self-use]
        """
        Read the settings the text writes under a table, with the parser that reads the file itself.

        :param text: the TOML source to read
        :param path: the table the settings are written under
        :return: the settings, or ``None`` where the text writes no such table
        """
        return settings_in(text, list(path))

    def format(self, text: str, opt: PyProjectFmtNamespace) -> str:  # ruff: ignore[no-self-use]
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
            sub_table_spacing=opt.sub_table_spacing,
            separate_root_table=opt.separate_root_table,
            expand_tables=opt.expand_tables,
            collapse_tables=opt.collapse_tables,
            skip_wrap_for_keys=opt.skip_wrap_for_keys,
            pin_envs=opt.pin_envs,
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
    return build_cli(ToxTOMLFormatter())[0]  # pragma: no cover


__all__ = [
    "runner",
]

if __name__ == "__main__":
    raise SystemExit(runner())
