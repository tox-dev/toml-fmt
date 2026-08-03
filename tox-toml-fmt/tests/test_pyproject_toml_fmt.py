from __future__ import annotations

import subprocess  # ruff: ignore[suspicious-subprocess-import]
import sys
from pathlib import Path


def test_help_invocation_as_module() -> None:
    subprocess.check_call([sys.executable, "-m", "tox_toml_fmt", "--help"])


def test_help_invocation_as_script() -> None:
    subprocess.check_call(
        [str(Path(sys.executable).parent / "tox-toml-fmt"), "--help"],
    )
