from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from tasks.python_versions import Place, with_a_candidate, written_places

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


@pytest.mark.parametrize(
    "index",
    [
        pytest.param(index, id=label)
        for index, label in enumerate((
            "cli-default",
            "examples",
            "pyproject-config",
            "tox-config",
            "configuration",
            "formatting",
            "cli-minimum",
        ))
    ],
)
def test_written_places_match_expected_spelling(tmp_path: Path, index: int) -> None:
    place = written_places(15, 11)[index]
    previous = written_places(14, 10)[index].spelling
    source = tmp_path / "source"
    source.write_text(f"before\n{previous}\nafter\n")

    assert Place(path=source, pattern=place.pattern, minor=place.minor, spelling=place.spelling).said() == previous
