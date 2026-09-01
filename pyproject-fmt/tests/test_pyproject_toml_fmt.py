from __future__ import annotations

import subprocess  # ruff: ignore[suspicious-subprocess-import]  # the test runs the installed CLI
import sys
from pathlib import Path

import pytest


@pytest.mark.parametrize(
    "command",
    [
        pytest.param([sys.executable, "-m", "pyproject_fmt"], id="as-a-module"),
        pytest.param([str(Path(sys.executable).parent / "pyproject-fmt")], id="as-a-script"),
    ],
)
def test_help_names_the_program(command: list[str]) -> None:
    got = subprocess.check_output([*command, "--help"], text=True)

    assert got.startswith("usage: pyproject-fmt ")
