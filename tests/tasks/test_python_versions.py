"""Python release names and checked-in settings."""

from __future__ import annotations

from typing import TYPE_CHECKING

from tasks import python_versions

if TYPE_CHECKING:
    import pytest


def test_with_a_candidate_accepts_only_release_candidates(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        python_versions,
        "fetch",
        lambda _: [
            {"name": "Python 3.15.0rc1"},
            {"name": "Python 3.16.0b2"},
            {"name": "Python 2.7.18rc1"},
            {"name": "Python 3.15.0"},
        ],
    )

    assert python_versions.with_a_candidate() == {15}


def test_written_places_match_current_source() -> None:
    places = python_versions.written_places(15, 11)

    assert all(place.said() == place.spelling for place in places)
