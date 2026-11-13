"""Keep Python callers on the same parser as the tox-toml-fmt CLI."""

from __future__ import annotations

from .__main__ import build_parser
from .__main__ import runner as run

__all__ = [
    "build_parser",
    "run",
]
