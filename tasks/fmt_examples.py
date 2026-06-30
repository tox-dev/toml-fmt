"""Render TOML formatting examples by running the local formatter.

Shared by the ``fmt-example`` Sphinx directive (live HTML build) and
``generate_readme.py`` (static expansion for PyPI), so both surfaces show the
exact output the installed formatter produces and can never drift from it.
"""

from __future__ import annotations

import importlib
from typing import TYPE_CHECKING, Any, Final

if TYPE_CHECKING:
    from collections.abc import Mapping

# Default Settings kwargs per formatter, mirroring each package's CLI defaults.
DEFAULTS: Final[Mapping[str, Mapping[str, Any]]] = {
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
    """
    Format ``before`` with ``module``'s formatter and render the example text.

    :param module: formatter package, ``pyproject_fmt`` or ``tox_toml_fmt``
    :param before: the input TOML the author wrote
    :param config: ``key=value`` overrides, space-separated (e.g. ``table_format=long``)
    :return: a ``# Before`` / ``# After`` block, or a single block when already formatted
    """
    before = before.strip("\n")
    after = format_example(module, before, config).strip("\n")
    if after == before:
        return after
    return f"# Before\n{before}\n\n# After\n{after}"


def format_example(module: str, before: str, config: str = "") -> str:
    """
    Format ``before`` with ``module``'s formatter.

    :param module: formatter package, ``pyproject_fmt`` or ``tox_toml_fmt``
    :param before: the input TOML the author wrote
    :param config: ``key=value`` overrides, space-separated
    :return: the formatted TOML
    """
    lib = importlib.import_module(f"{module}._lib")
    defaults = dict(DEFAULTS[module])
    defaults.update(_parse_config(config, defaults))
    return lib.format_toml(before, lib.Settings(**defaults))


def _parse_config(config: str, defaults: Mapping[str, Any]) -> dict[str, Any]:
    overrides: dict[str, Any] = {}
    for token in config.split():
        key, sep, raw = token.partition("=")
        if not sep or key not in defaults:
            msg = f"invalid fmt-example config token: {token!r}"
            raise ValueError(msg)
        overrides[key] = _coerce(raw, defaults[key])
    return overrides


def _coerce(raw: str, default: Any) -> Any:
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
