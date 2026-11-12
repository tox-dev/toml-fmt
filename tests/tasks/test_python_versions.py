from __future__ import annotations

import re
from typing import TYPE_CHECKING

import pytest

from tasks.python_versions import (
    Place,
    newest_candidate,
    oldest_supported,
    with_a_candidate,
    with_a_classifier,
    written_places,
)

if TYPE_CHECKING:
    from pathlib import Path

    from pytest_mock import MockerFixture


def test_with_a_candidate_accepts_only_release_candidates(mocker: MockerFixture) -> None:
    mocker.patch(
        "tasks.python_versions.fetch",
        autospec=True,
        return_value=[
            {"name": "Python 3.15.0rc1"},
            {"name": "Python 3.16.0b2"},
            {"name": "Python 2.7.18rc1"},
            {"name": "Python 3.15.0"},
        ],
    )

    assert with_a_candidate() == {15}


def test_with_a_classifier_accepts_only_python_minor_versions(mocker: MockerFixture) -> None:
    mocker.patch(
        "tasks.python_versions.classifiers",
        {"Programming Language :: Python :: 3.15", "Programming Language :: Python :: 3", "Other :: 3.16"},
    )

    assert with_a_classifier() == {15}


def test_newest_candidate_requires_release_and_classifier(mocker: MockerFixture) -> None:
    mocker.patch(
        "tasks.python_versions.fetch",
        autospec=True,
        return_value=[{"name": "Python 3.15.0rc1"}, {"name": "Python 3.16.0rc1"}],
    )
    mocker.patch("tasks.python_versions.classifiers", {"Programming Language :: Python :: 3.15"})

    assert newest_candidate() == 15


def test_oldest_supported_uses_live_release_cycle(mocker: MockerFixture) -> None:
    mocker.patch(
        "tasks.python_versions.fetch",
        autospec=True,
        return_value=[
            {"cycle": "3.10", "eol": "1000-01-01"},
            {"cycle": "3.11", "eol": "9999-12-31"},
            {"cycle": "3.12", "eol": "9999-12-31"},
        ],
    )

    assert oldest_supported() == 11


def test_place_keeps_instance_dict(tmp_path: Path) -> None:
    pattern = re.compile(r"source")
    place = Place(tmp_path / "source", pattern, 15, "target")

    assert place.__dict__ == {
        "path": tmp_path / "source",
        "pattern": pattern,
        "minor": 15,
        "spelling": "target",
    }


@pytest.mark.parametrize(
    "index",
    range(7),
    ids=("cli-default", "examples", "pyproject-config", "tox-config", "configuration", "formatting", "cli-minimum"),
)
def test_written_places_match_expected_spelling(tmp_path: Path, index: int) -> None:
    place = written_places(15, 11)[index]
    previous = written_places(14, 10)[index].spelling
    source = tmp_path / "source"
    source.write_text(f"before\n{previous}\nafter\n")

    assert Place(source, place.pattern, place.minor, place.spelling).said() == previous
