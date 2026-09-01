"""Keep CLI parsing in Python and TOML rewriting in Rust."""

from __future__ import annotations

from typing import TYPE_CHECKING

from toml_fmt_common import (
    ArgumentGroup,
    FmtNamespace,
    TOMLFormatter,
    TomlValue,
    build_cli,
    name_list_argument,
    run,
)

from ._lib import Settings, format_toml, settings_in

if TYPE_CHECKING:
    from argparse import ArgumentParser
    from collections.abc import Sequence


class _ToxTOMLNamespace(FmtNamespace):
    """Give tox-specific argparse fields concrete types."""

    pin_envs: list[str]


class _ToxTOMLFormatter(TOMLFormatter[_ToxTOMLNamespace]):
    """Bind the shared CLI to the tox-toml-fmt Rust extension."""

    def __init__(self) -> None:
        """Start with the namespace argparse fills for each input."""
        super().__init__(_ToxTOMLNamespace())

    @property
    def prog(self) -> str:
        """Match the distribution name used for version lookup."""
        return "tox-toml-fmt"

    @property
    def filename(self) -> str:
        """Restrict positional directories to ``tox.toml``."""
        return "tox.toml"

    @staticmethod
    def add_format_flags(parser: ArgumentGroup) -> None:
        """Add tox environment options outside the shared formatter settings."""
        parser.add_argument(
            "--pin-env",
            type=name_list_argument,
            default=[],
            dest="pin_envs",
            help="environments whose tables are written first (comma separated)",
        )

    @property
    def override_cli_from_section(self) -> tuple[str, ...]:
        """Keep per-file overrides under ``tox-toml-fmt``."""
        return ("tox-toml-fmt",)

    @staticmethod
    def settings_in(text: str, path: Sequence[str]) -> dict[str, TomlValue] | None:
        """Use the Rust parser so TOML 1.1 settings remain visible."""
        return settings_in(text, list(path))

    @staticmethod
    def format(text: str, opt: _ToxTOMLNamespace) -> str:
        """Keep Python responsible for CLI state and Rust responsible for TOML edits."""
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
    Use a supplied argument list for embedding; the console entry reads ``sys.argv``.

    Return 1 after a change or rejection, and 0 when inputs match.
    """
    return run(_ToxTOMLFormatter(), args)


def build_parser() -> ArgumentParser:
    """Build the parser without reading arguments, for documentation tooling."""
    return build_cli(_ToxTOMLFormatter())[0]


__all__ = [
    "build_parser",
    "runner",
]

if __name__ == "__main__":
    raise SystemExit(runner())
