from __future__ import annotations

import subprocess
import sys
from pathlib import Path
from tarfile import open as tar_open
from tempfile import TemporaryDirectory
from typing import Final

_ROOT: Final[Path] = Path(__file__).resolve().parents[1]
_CARRIED: Final[str] = "toml-fmt-common/src/toml_fmt_common/__init__.py"


def main(package: str) -> None:
    with TemporaryDirectory() as folder:
        at = Path(folder)
        sdist = build_sdist(package, at / "sdist")
        carries_common(sdist)
        runs_on_its_own(package, wheel_from(sdist, at / "wheel"))
        ships_a_suite_that_passes(package, sdist, at / "unpacked")


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


def runs_on_its_own(package: str, wheel: Path) -> None:
    # --no-index leaves nothing to fall back on, so an unvendored wheel cannot install or import
    run("uv", "pip", "install", "--python", sys.executable, "--no-index", str(wheel))
    run(str(Path(sys.executable).parent / package), "--version")
    print(f"{wheel.name} built from the sdist runs with no index behind it")


def ships_a_suite_that_passes(package: str, sdist: Path, at: Path) -> None:
    # what a packager runs: unpack the sdist, install it, run the suite it ships
    at.mkdir()
    with tar_open(sdist) as tar:
        tar.extractall(at, filter="data")
    root = next(at.iterdir())

    run("uv", "pip", "install", "--python", sys.executable, "-e", ".", at=root)
    run(sys.executable, "-m", "pytest", package, at=root)
    print(f"{sdist.name} passes the suite it ships")


def run(*command: str, at: Path | None = None) -> None:
    subprocess.check_call(command, cwd=at)


if __name__ == "__main__":
    try:
        _, package = sys.argv
    except ValueError:
        print("Usage: check_sdist.py <package>")
        sys.exit(1)
    main(package)
