"""Build a package the way a release builds it and run what comes out with no index behind it."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path
from tarfile import open as tar_open
from tempfile import TemporaryDirectory

_ROOT = Path(__file__).resolve().parents[1]
_CARRIED = "toml-fmt-common/src/toml_fmt_common/__init__.py"


def main(package: str) -> None:
    with TemporaryDirectory() as folder:
        at = Path(folder)
        sdist = build_sdist(package, at / "sdist")
        carries_common(sdist)
        runs_on_its_own(package, wheel_from(sdist, at / "wheel"), at / "venv")


def build_sdist(package: str, at: Path) -> Path:
    run("uv", "build", "--sdist", "--out-dir", str(at), str(_ROOT / package))
    return next(at.glob("*.tar.gz"))


def carries_common(sdist: Path) -> None:
    with tar_open(sdist) as tar:
        if not [name for name in tar.getnames() if name.endswith(_CARRIED)]:
            print(f"{sdist.name} holds no {_CARRIED}")
            sys.exit(1)


def wheel_from(sdist: Path, at: Path) -> Path:
    run("uv", "build", "--wheel", "--out-dir", str(at), str(sdist))
    return next(at.glob("*.whl"))


def runs_on_its_own(package: str, wheel: Path, venv: Path) -> None:
    run("uv", "venv", str(venv))
    scripts = venv / ("Scripts" if sys.platform == "win32" else "bin")
    # --no-index leaves nothing to fall back on, so an unvendored wheel cannot install or import
    run("uv", "pip", "install", "--no-index", "--python", str(scripts / "python"), str(wheel))
    run(str(scripts / package), "--version")
    print(f"{wheel.name} built from the sdist runs with no index behind it")


def run(*command: str) -> None:
    subprocess.check_call(command)


if __name__ == "__main__":
    if len(sys.argv) != 2:  # ruff: ignore[magic-value-comparison]
        print("Usage: check_sdist.py <package>")
        sys.exit(1)
    main(sys.argv[1])
