"""Keep Python callers on the same parser as the pyproject-fmt CLI."""

from __future__ import annotations

from .__main__ import build_parser, format_example
from .__main__ import runner as run

__all__ = [
    "build_parser",
    "format_example",
    "run",
]
