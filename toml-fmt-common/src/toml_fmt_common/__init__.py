"""Keep CLI behavior identical across the TOML formatters."""

from __future__ import annotations

from ._app import (
    SHARED_SETTINGS,
    ArgumentGroup,
    FmtNamespace,
    Setting,
    TOMLFormatter,
    TomlValue,
    _build_cli,
    build_cli,
    count_argument,
    list_argument,
    name_list_argument,
    run,
    spacing_argument,
)

__all__ = [
    "SHARED_SETTINGS",
    "ArgumentGroup",
    "FmtNamespace",
    "Setting",
    "TOMLFormatter",
    "TomlValue",
    "_build_cli",
    "build_cli",
    "count_argument",
    "list_argument",
    "name_list_argument",
    "run",
    "spacing_argument",
]
