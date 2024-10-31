from __future__ import annotations

from textwrap import dedent

import pytest
from tox_toml_fmt._lib import Settings, format_toml


@pytest.mark.parametrize(
    ("start", "expected"),
    [
        pytest.param(
            """\
            requires = ["tox>=4.22"]
            env_list = ["3.13", "3.12"]
            skip_missing_interpreters = true

            [env_run_base]
            description = "run the tests with pytest under {env_name}"
            commands = [ ["pytest"] ]

            [env.type]
            description = "run type check on code base"
            commands = [["mypy", "src{/}tox_toml_fmt"], ["mypy", "tests"]]
            """,
            """\
            requires = [ "tox>=4.22" ]
            env_list = [ "3.13", "3.12" ]
            skip_missing_interpreters = true

            [env_run_base]
            description = "run the tests with pytest under {env_name}"
            commands = [ [ "pytest" ] ]

            [env.type]
            description = "run type check on code base"
            commands = [ [ "mypy", "src{/}tox_toml_fmt" ], [ "mypy", "tests" ] ]
            """,
            id="example",
        ),
    ],
)
def test_format_toml(start: str, expected: str) -> None:
    settings = Settings(column_width=120, indent=4)
    res = format_toml(dedent(start), settings)
    assert res == dedent(expected)
