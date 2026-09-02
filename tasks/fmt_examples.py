"""Render docs examples with the local extension so HTML and PyPI show the same output."""

from __future__ import annotations

import importlib
from typing import TYPE_CHECKING, Final, TypeAlias

if TYPE_CHECKING:
    from collections.abc import Mapping

_Setting: TypeAlias = bool | int | str | tuple[int, int] | tuple[str, ...]

# Match the CLI defaults so examples need no duplicate configuration.
_DEFAULTS: Final[Mapping[str, Mapping[str, _Setting]]] = {
    "pyproject_fmt": {
        "column_width": 120,
        "indent": 2,
        "keep_full_version": False,
        "max_supported_python": (3, 14),
        "min_supported_python": (3, 10),
        "generate_python_version_classifiers": True,
        "table_format": "short",
        "sub_table_spacing": "",
        "separate_root_table": "\n",
        "expand_tables": (),
        "collapse_tables": (),
        "skip_wrap_for_keys": (),
    },
    "tox_toml_fmt": {
        "column_width": 120,
        "indent": 2,
        "table_format": "short",
        "sub_table_spacing": "",
        "separate_root_table": "\n",
        "expand_tables": (),
        "collapse_tables": (),
        "skip_wrap_for_keys": (),
        "pin_envs": (),
    },
}


def render_example(module: str, before: str, config: str = "") -> str:
    """Use live formatter output in docs; omit labels when the input needs no change."""
    before = before.strip("\n")
    after = _format_example(module, before, config).strip("\n")
    if after == before:
        return after
    return f"# Before\n{before}\n\n# After\n{after}"


def _format_example(module: str, before: str, config: str = "") -> str:
    lib = importlib.import_module(f"{module}._lib")
    defaults = dict(_DEFAULTS[module])
    defaults.update(_parse_config(config, defaults))
    return lib.format_toml(before, lib.Settings(**defaults))


def _parse_config(config: str, defaults: Mapping[str, _Setting]) -> dict[str, _Setting]:
    overrides: dict[str, _Setting] = {}
    for token in config.split():
        key, sep, raw = token.partition("=")
        if not sep or key not in defaults:
            msg = f"invalid fmt-example config token: {token!r}"
            raise ValueError(msg)
        overrides[key] = _coerce(raw, defaults[key])
    return overrides


def _coerce(raw: str, default: _Setting) -> _Setting:
    if isinstance(default, bool):
        return raw == "true"
    if isinstance(default, int):
        return int(raw)
    if isinstance(default, tuple) and default and isinstance(default[0], int):
        major, minor = raw.split(".")
        return int(major), int(minor)
    if isinstance(default, (tuple, list)):
        return tuple(raw.split(","))
    return raw.replace("\\n", "\n")


__all__ = [
    "render_example",
]
