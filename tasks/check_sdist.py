"""Build a package from its own sdist and check the wheel that comes out carries toml-fmt-common."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory
from zipfile import ZipFile

_ROOT = Path(__file__).resolve().parents[1]


def main(package: str) -> None:
    with TemporaryDirectory() as folder:
        out = Path(folder)
        subprocess.check_call(["uv", "build", "--sdist", "--out-dir", str(out), str(_ROOT / package)])
        sdist = next(out.glob("*.tar.gz"))
        subprocess.check_call(["uv", "build", "--wheel", "--out-dir", str(out), str(sdist)])
        check(next(out.glob("*.whl")), package.replace("-", "_"))


def check(wheel: Path, module: str) -> None:
    with ZipFile(wheel) as zf:
        names = zf.namelist()
        metadata = zf.read(next(n for n in names if n.endswith(".dist-info/METADATA"))).decode()

    if (vendored := f"{module}/_vendor/toml_fmt_common/__init__.py") not in names:
        print(f"{wheel.name} built from the sdist holds no {vendored}")
        sys.exit(1)
    if "Requires-Dist: toml-fmt-common" in metadata:
        print(f"{wheel.name} built from the sdist still requires toml-fmt-common")
        sys.exit(1)
    print(f"{wheel.name} built from the sdist carries toml-fmt-common")


if __name__ == "__main__":
    if len(sys.argv) != 2:  # ruff: ignore[magic-value-comparison]
        print("Usage: check_sdist.py <package>")
        sys.exit(1)
    main(sys.argv[1])
