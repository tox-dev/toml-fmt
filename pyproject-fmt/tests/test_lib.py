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
        pytest.param(
            """
            [project]
            name = "test"
            version = "0.0.1"
            classifiers = ["Programming Language :: Python :: 3", "a :: string"]
            """,
            """\
            [project]
            name = "test"
            version = "0.0.1"
            classifiers = [
                "a :: string",
                "Programming Language :: Python :: 3 :: Only",
                "Programming Language :: Python :: 3.7",
                "Programming Language :: Python :: 3.8",
            ]
            """,
            id="invalid_classifier",
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
        sub_table_spacing="",
        separate_root_table="\n",
        expand_tables=[],
        collapse_tables=[],
        skip_wrap_for_keys=[],
    )
    res = format_toml(dedent(start), settings)
    assert res == dedent(expected)


@pytest.mark.parametrize(
    ("sub_table_spacing", "has_blank_line"),
    [
        pytest.param("\n", True, id="blank_line"),
        pytest.param("", False, id="compact"),
    ],
)
def test_sub_table_spacing(sub_table_spacing: str, *, has_blank_line: bool) -> None:
    start = dedent("""\
        [tool.ruff]
        line-length = 120

        [tool.ruff.lint]
        select = ["E"]
    """)
    settings = Settings(
        column_width=120,
        indent=2,
        keep_full_version=False,
        min_supported_python=(3, 9),
        max_supported_python=(3, 9),
        generate_python_version_classifiers=False,
        table_format="long",
        sub_table_spacing=sub_table_spacing,
        separate_root_table="\n",
        expand_tables=[],
        collapse_tables=[],
        skip_wrap_for_keys=[],
    )
    res = format_toml(start, settings)
    assert ("\n\n[tool.ruff.lint]" in res) == has_blank_line
