from __future__ import annotations

import asyncio
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
    async def read_help() -> str:
        process = await asyncio.create_subprocess_exec(*command, "--help", stdout=asyncio.subprocess.PIPE)
        stdout, _ = await process.communicate()
        assert process.returncode == 0
        return stdout.decode()

    got = asyncio.run(read_help())

    assert got.startswith("usage: pyproject-fmt ")
