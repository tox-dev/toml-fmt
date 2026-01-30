from __future__ import annotations

from textwrap import dedent

import pytest

from pyproject_fmt._lib import Settings, format_toml


@pytest.mark.parametrize(
    ("start", "expected"),
    [
        pytest.param(
            """
            [project]
            keywords = [
              "A",
            ]
            classifiers = [
              "Programming Language :: Python :: 3 :: Only",
            ]
            dynamic = [
              "B",
            ]
            dependencies = [
              "requests>=2.0",
            ]
            """,
            """\
            [project]
            keywords = [
                "A",
            ]
            classifiers = [
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.7",
                "Programming Language :: Python :: 3.8",
            ]
            dynamic = [
                "B",
            ]
            dependencies = [
                "requests>=2.0",
            ]
            """,
            id="expanded",
        ),
        pytest.param(
            """
            [project]
            keywords = ["A"]
            classifiers = ["Programming Language :: Python :: 3 :: Only"]
            dynamic = ["B"]
            dependencies = ["requests>=2.0"]
            """,
            """\
            [project]
            keywords = [ "A" ]
            classifiers = [
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.7",
                "Programming Language :: Python :: 3.8",
            ]
            dynamic = [ "B" ]
            dependencies = [ "requests>=2.0" ]
            """,
            id="collapsed",
        ),
    ],
)
def test_format_toml(start: str, expected: str) -> None:
    settings = Settings(
        column_width=120,
        indent=4,
        keep_full_version=True,
        min_supported_python=(3, 7),
        max_supported_python=(3, 8),
        generate_python_version_classifiers=True,
        table_format="short",
        expand_tables=[],
        collapse_tables=[],
    )
    res = format_toml(dedent(start), settings)
    assert res == dedent(expected)
