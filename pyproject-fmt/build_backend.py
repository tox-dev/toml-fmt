"""
Vendor toml-fmt-common into the wheel, as a PEP 517 backend and as a CLI patcher.

A new toml-fmt-common release must never break an already published consumer
(tox-dev/toml-fmt#355), so the wheel is made self-contained instead of depending on it.

The CLI entry point exists because CI builds wheels with ``maturin build``, which never
invokes a PEP 517 backend; the same patch then runs on maturin-action's output.
"""

# /// script
# requires-python = ">=3.10"
# dependencies = ["maturin>=1.13.3"]
# ///

from __future__ import annotations

from base64 import urlsafe_b64encode
from hashlib import sha256
from os import environ
from pathlib import Path
from re import findall, search, sub
from shutil import copy2
from sys import argv
from tempfile import mkdtemp
from typing import TYPE_CHECKING
from zipfile import ZIP_DEFLATED, ZipFile

import maturin

if TYPE_CHECKING:
    from collections.abc import Mapping

    ConfigSettings = Mapping[str, str | list[str]]

# our wrapper backend is intentional; silence maturin's missing-backend warning
environ.setdefault("MATURIN_NO_MISSING_BUILD_BACKEND_WARNING", "1")

_HERE = Path(__file__).resolve().parent
_MODULE = _HERE.name.replace("-", "_")
_COMMON = _HERE.parent / "toml-fmt-common"
_VENDOR = "toml_fmt_common"


def build_wheel(
    wheel_directory: str,
    config_settings: ConfigSettings | None = None,
    metadata_directory: str | None = None,
) -> str:
    if not (_COMMON / "src" / _VENDOR).is_dir():  # no workspace (e.g. building from sdist)
        return maturin.build_wheel(wheel_directory, config_settings, metadata_directory)
    tmp = Path(mkdtemp())
    name = maturin.build_wheel(str(tmp), config_settings, metadata_directory)
    vendor_into_wheel(tmp / name)
    copy2(tmp / name, Path(wheel_directory) / name)
    return name


def build_sdist(sdist_directory: str, config_settings: ConfigSettings | None = None) -> str:
    return maturin.build_sdist(sdist_directory, config_settings)


def build_editable(
    wheel_directory: str,
    config_settings: ConfigSettings | None = None,
    metadata_directory: str | None = None,
) -> str:
    return maturin.build_editable(wheel_directory, config_settings, metadata_directory)


def get_requires_for_build_wheel(config_settings: ConfigSettings | None = None) -> list[str]:
    return maturin.get_requires_for_build_wheel(config_settings)


def get_requires_for_build_sdist(config_settings: ConfigSettings | None = None) -> list[str]:
    return maturin.get_requires_for_build_sdist(config_settings)


def get_requires_for_build_editable(config_settings: ConfigSettings | None = None) -> list[str]:
    return maturin.get_requires_for_build_editable(config_settings)


def main() -> None:
    target = Path(argv[1])
    if not (wheels := sorted(target.glob("*.whl")) if target.is_dir() else [target]):
        print(f"no wheels found in {target}")
        raise SystemExit(1)
    for wheel in wheels:
        vendor_into_wheel(wheel)
        print(f"vendored toml-fmt-common into {wheel.name}")


def vendor_into_wheel(wheel: Path) -> None:
    common_src = _COMMON / "src" / _VENDOR

    with ZipFile(wheel) as src:
        names = src.namelist()
        dist_info = next(n for n in names if n.endswith(".dist-info/METADATA")).split("/")[0]
        out = {n: src.read(n) for n in names if not n.endswith("/RECORD")}

    if (entry := f"{_MODULE}/__main__.py") in out:
        out[entry] = sub(rf"\b{_VENDOR}\b", f"{_MODULE}._vendor.{_VENDOR}", out[entry].decode()).encode()
    meta = f"{dist_info}/METADATA"
    deps_block = search(r"(?ms)^dependencies = \[(.*?)\]", (_COMMON / "pyproject.toml").read_text())
    requires = (
        "".join(f"Requires-Dist: {d}\n" for d in findall(r'"([^"]*)"', deps_block.group(1))) if deps_block else ""
    )
    stripped = sub(r"(?m)^Requires-Dist: toml-fmt-common.*\n", "", out[meta].decode())
    out[meta] = (stripped + requires).encode()

    out[f"{_MODULE}/_vendor/__init__.py"] = b""
    for file in common_src.rglob("*"):
        if not file.is_file():
            continue
        if "__pycache__" in file.parts or file.suffix in {".pyc", ".pyo"}:
            continue
        out[f"{_MODULE}/_vendor/{file.relative_to(common_src.parent).as_posix()}"] = file.read_bytes()

    record = []
    for name, data in out.items():
        digest = urlsafe_b64encode(sha256(data).digest()).rstrip(b"=").decode()
        record.append(f"{name},sha256={digest},{len(data)}")
    record.append(f"{dist_info}/RECORD,,")
    out[f"{dist_info}/RECORD"] = ("\n".join(record) + "\n").encode()

    with ZipFile(wheel, "w", ZIP_DEFLATED) as zf:
        for name, data in out.items():
            zf.writestr(name, data)


if __name__ == "__main__":
    main()
