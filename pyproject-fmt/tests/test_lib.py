from __future__ import annotations

from textwrap import dedent

import pytest

from pyproject_fmt._lib import Settings, format_toml, parse_ident


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
        do_not_collapse=[],
    )
    res = format_toml(dedent(start), settings)
    assert res == dedent(expected)


@pytest.mark.parametrize(
    ("arg", "expected"),
    [
        ("a.b", ("a", "b")),
        ("a.'b.c'", ("a", "b.c")),
    ],
)
def test_parse_idents(arg: str, expected: tuple[str, ...]) -> None:
    assert parse_ident(arg) == expected


@pytest.mark.parametrize(
    ("arg", "exc_cls", "msg_pat"),
    [
        (None, TypeError, r"None"),
        ("1 b", ValueError, r"syntax error"),
        ("[]", ValueError, r"syntax error"),
        ("x.", ValueError, r"syntax error"),
    ],
)
def test_parse_idents_errors(arg: object, exc_cls: type[Exception], msg_pat: str) -> None:
    with pytest.raises(exc_cls, match=msg_pat):
        parse_ident(arg)
