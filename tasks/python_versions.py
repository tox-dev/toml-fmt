# /// script
# requires-python = ">=3.12"
# dependencies = ["trove-classifiers>=2026.6.1.19"]
# ///
"""
The Python releases the formatter's defaults stand for.

The two ends answer different questions and are read from different places. The newest release a
project may be given a classifier for has to satisfy two things at once: PyPI publishes a classifier
naming it, and it has reached its release candidate. Either alone says too little, since a
classifier appears while a release is still changing and a candidate PyPI has no classifier for
cannot be written down. The oldest is the one still carrying fixes, which a classifier outlives by
years and so cannot say.

Nothing here is predicted. Each run reads what is true that day and writes down the numbers, so a
release that slips arrives when it arrives. Someone naming `max_supported_python` themselves is
saying what their own project supports, which no rule here overrides.
"""

from __future__ import annotations

import json
import re
import sys
import urllib.request
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Final

from trove_classifiers import classifiers

_ROOT: Final[Path] = Path(__file__).resolve().parents[1]
_MAJOR: Final[int] = 3


def main(*, check: bool) -> None:
    places = written_places(newest_candidate(), oldest_supported())
    drifted = [(place, said) for place in places if (said := place.said()) != place.spelling]
    if not drifted:
        print(f"the defaults name {_MAJOR}.{places[-1].minor} through {_MAJOR}.{places[0].minor}")
        return
    for place, said in drifted:
        print(f"{place.path.relative_to(_ROOT)}: {said} -> {place.spelling}")
    if check:
        sys.exit(1)
    for place, said in drifted:
        place.write(said)


def newest_candidate() -> int:
    """
    The newest release both PyPI and Python itself have got to: a classifier naming it, and a
    release candidate published.

    A release still in alpha or beta is one whose own behavior is still moving, so a project saying
    it runs there says more than it knows. A candidate PyPI publishes no classifier for is one no
    project can name, since an upload carrying an unknown classifier is turned away.
    """
    return max(with_a_candidate() & with_a_classifier())


def with_a_candidate() -> set[int]:
    """The releases Python has published a candidate for."""
    released = fetch("https://www.python.org/api/v2/downloads/release/?is_published=true")
    named = re.compile(rf"^Python {_MAJOR}\.(\d+)\.\d+rc\d+$")
    return {int(found[1]) for release in released if (found := named.match(release["name"]))}


def with_a_classifier() -> set[int]:
    """The releases PyPI publishes a classifier for."""
    prefix = f"Programming Language :: Python :: {_MAJOR}."
    return {
        int(minor) for name in classifiers if name.startswith(prefix) and (minor := name.removeprefix(prefix)).isdigit()
    }


def oldest_supported() -> int:
    """The oldest release still carrying fixes, which is the oldest the formatter writes for."""
    today = datetime.now(tz=UTC).date().isoformat()
    cycles = fetch("https://endoflife.date/api/python.json")
    live = (row for row in cycles if isinstance(row["eol"], str) and row["eol"] > today)
    return min(int(row["cycle"].split(".")[1]) for row in live)


def fetch(url: str) -> list[dict[str, str]]:
    with urllib.request.urlopen(url, timeout=30) as response:  # ruff: ignore[suspicious-url-open-usage]  # a named host
        return json.load(response)


def written_places(newest: int, oldest: int) -> list[Place]:
    """Every file that spells either release, the newest ones first."""
    spelled = [
        ("pyproject-fmt/src/pyproject_fmt/__main__.py", r"default=\(\d+, \d+\)", f"default=({_MAJOR}, {newest})"),
        (
            "tasks/fmt_examples.py",
            r'"max_supported_python": \(\d+, \d+\)',
            f'"max_supported_python": ({_MAJOR}, {newest})',
        ),
        (
            "pyproject-fmt/pyproject.toml",
            r'max_supported_python = "\d+\.\d+"',
            f'max_supported_python = "{_MAJOR}.{newest}"',
        ),
        (
            "tox-toml-fmt/pyproject.toml",
            r'max_supported_python = "\d+\.\d+"',
            f'max_supported_python = "{_MAJOR}.{newest}"',
        ),
        (
            "pyproject-fmt/docs/configuration.rst",
            r'max_supported_python = "\d+\.\d+"',
            f'max_supported_python = "{_MAJOR}.{newest}"',
        ),
        (
            "pyproject-fmt/docs/formatting.rst",
            r"``max_supported_python`` \(here ``\d+\.\d+``\)",
            f"``max_supported_python`` (here ``{_MAJOR}.{newest}``)",
        ),
    ]
    places = [Place(_ROOT / name, pattern, newest, spelling) for name, pattern, spelling in spelled]
    places.append(
        Place(
            _ROOT / "pyproject-fmt/src/pyproject_fmt/__main__.py",
            r"_MIN_SUPPORTED_PYTHON: Final\[tuple\[int, int\]\] = \(_PYTHON_MAJOR, \d+\)",
            oldest,
            f"_MIN_SUPPORTED_PYTHON: Final[tuple[int, int]] = (_PYTHON_MAJOR, {oldest})",
        )
    )
    return places


@dataclass(frozen=True)
class Place:
    """One spelling of a release, in the file that writes it."""

    path: Path
    pattern: str
    minor: int
    spelling: str

    def said(self) -> str:
        """What the file writes there, which a run holds against the spelling it should write."""
        found = re.search(self.pattern, self.path.read_text(encoding="utf-8"))
        if found is None:
            print(f"{self.path.relative_to(_ROOT)}: nothing matches {self.pattern}")
            sys.exit(1)
        return found.group(0)

    def write(self, said: str) -> None:
        self.path.write_text(self.path.read_text(encoding="utf-8").replace(said, self.spelling), encoding="utf-8")


if __name__ == "__main__":
    main(check="--check" in sys.argv[1:])
