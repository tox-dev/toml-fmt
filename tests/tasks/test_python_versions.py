"""Python release names and checked-in settings."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from tasks import python_versions

if TYPE_CHECKING:
    from pathlib import Path

    from pytest_mock import MockerFixture


def test_with_a_candidate_accepts_only_release_candidates(mocker: MockerFixture) -> None:
    mocker.patch(
        "tasks.python_versions.fetch",
        return_value=[
            {"name": "Python 3.15.0rc1"},
            {"name": "Python 3.16.0b2"},
            {"name": "Python 2.7.18rc1"},
            {"name": "Python 3.15.0"},
        ],
    )

    assert python_versions.with_a_candidate() == {15}


@pytest.mark.parametrize(
    "index",
    range(7),
    ids=("cli-default", "examples", "pyproject-config", "tox-config", "configuration", "formatting", "cli-minimum"),
)
def test_written_places_match_expected_spelling(tmp_path: Path, index: int) -> None:
    place = python_versions.written_places(15, 11)[index]
    previous = python_versions.written_places(14, 10)[index].spelling
    source = tmp_path / "source"
    source.write_text(f"before\n{previous}\nafter\n")

    assert python_versions.Place(source, place.pattern, place.minor, place.spelling).said() == previous
