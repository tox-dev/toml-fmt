"""
Vendor toml-fmt-common into what we build, as a PEP 517 backend and as a CLI patcher.

A new toml-fmt-common release must never break an already published consumer
(tox-dev/toml-fmt#355), so every artifact is made self-contained instead of depending on it.

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
from io import BytesIO
from os import environ
from pathlib import Path
from re import findall, search, sub
from shutil import copy2
from sys import argv
from tarfile import TarInfo
from tarfile import open as tar_open
from tempfile import mkdtemp
from typing import TYPE_CHECKING
from zipfile import ZIP_DEFLATED, ZipFile

import maturin

if TYPE_CHECKING:
    from collections.abc import Callable, Iterator, Mapping

    ConfigSettings = Mapping[str, str | list[str]]

# our wrapper backend is intentional; silence maturin's missing-backend warning
environ.setdefault("MATURIN_NO_MISSING_BUILD_BACKEND_WARNING", "1")

_HERE = Path(__file__).resolve().parent
_MODULE = findall(r'(?m)^name = "(.*)"', (_HERE / "pyproject.toml").read_text())[0].replace("-", "_")
_VENDOR = "toml_fmt_common"
# a checkout keeps toml-fmt-common beside the package; the sdist carries it within
_COMMON = _HERE / "toml-fmt-common" if (_HERE / "toml-fmt-common").is_dir() else _HERE.parent / "toml-fmt-common"


def build_wheel(
    wheel_directory: str,
    config_settings: ConfigSettings | None = None,
    metadata_directory: str | None = None,
) -> str:
    return built(maturin.build_wheel, wheel_directory, config_settings, metadata_directory, vendor_into_wheel)


def build_sdist(sdist_directory: str, config_settings: ConfigSettings | None = None) -> str:
    name = maturin.build_sdist(sdist_directory, config_settings)
    if common_is_present():
        vendor_into_sdist(Path(sdist_directory) / name)
    return name


def build_editable(
    wheel_directory: str,
    config_settings: ConfigSettings | None = None,
    metadata_directory: str | None = None,
) -> str:
    return built(maturin.build_editable, wheel_directory, config_settings, metadata_directory, link_common_into_wheel)


def get_requires_for_build_wheel(config_settings: ConfigSettings | None = None) -> list[str]:
    return maturin.get_requires_for_build_wheel(config_settings)


def get_requires_for_build_sdist(config_settings: ConfigSettings | None = None) -> list[str]:
    return maturin.get_requires_for_build_sdist(config_settings)


def get_requires_for_build_editable(config_settings: ConfigSettings | None = None) -> list[str]:
    return maturin.get_requires_for_build_editable(config_settings)


def built(
    build: Callable[[str, ConfigSettings | None, str | None], str],
    wheel_directory: str,
    config_settings: ConfigSettings | None,
    metadata_directory: str | None,
    carry: Callable[[Path], None],
) -> str:
    if not common_is_present():
        return build(wheel_directory, config_settings, metadata_directory)
    tmp = Path(mkdtemp())
    name = build(str(tmp), config_settings, metadata_directory)
    carry(tmp / name)
    copy2(tmp / name, Path(wheel_directory) / name)
    return name


def common_is_present() -> bool:
    return (_COMMON / "src" / _VENDOR).is_dir()


def main() -> None:
    target = Path(argv[1])
    if not (wheels := sorted(target.glob("*.whl")) if target.is_dir() else [target]):
        print(f"no wheels found in {target}")
        raise SystemExit(1)
    for wheel in wheels:
        vendor_into_wheel(wheel)
        print(f"vendored toml-fmt-common into {wheel.name}")


def vendor_into_wheel(wheel: Path) -> None:
    at = f"{_MODULE}/_vendor/"
    changed = {f"{at}__init__.py": b""} | {f"{at}{name}": data for name, data in common_sources()}
    with ZipFile(wheel) as src:
        if (entry := f"{_MODULE}/__main__.py") in src.namelist():
            spelled = sub(rf"\b{_VENDOR}\b", f"{_MODULE}._vendor.{_VENDOR}", src.read(entry).decode())
            changed[entry] = spelled.encode()
    rewrite(wheel, changed)


def link_common_into_wheel(wheel: Path) -> None:
    # an editable install reads the package from the source tree, where the import stays unvendored
    rewrite(wheel, {f"{_MODULE}_{_VENDOR}.pth": f"{_COMMON / 'src'}\n".encode()})


def rewrite(wheel: Path, changed: dict[str, bytes]) -> None:
    with ZipFile(wheel) as src:
        names = src.namelist()
        dist_info = next(n for n in names if n.endswith(".dist-info/METADATA")).split("/")[0]
        out = {n: src.read(n) for n in names if not n.endswith("/RECORD")}
    out.update(changed)
    out[f"{dist_info}/METADATA"] = own_metadata(out[f"{dist_info}/METADATA"])

    record = []
    for name, data in out.items():
        digest = urlsafe_b64encode(sha256(data).digest()).rstrip(b"=").decode()
        record.append(f"{name},sha256={digest},{len(data)}")
    record.append(f"{dist_info}/RECORD,,")
    out[f"{dist_info}/RECORD"] = ("\n".join(record) + "\n").encode()

    with ZipFile(wheel, "w", ZIP_DEFLATED) as zf:
        for name, data in out.items():
            zf.writestr(name, data)


def vendor_into_sdist(sdist: Path) -> None:
    held = []
    with tar_open(sdist) as src:
        for member in src.getmembers():
            content = src.extractfile(member)
            held.append((member, content.read() if content else b""))
    root = held[0][0].name.split("/")[0]

    added = [(f"{root}/{_COMMON.name}/pyproject.toml", (_COMMON / "pyproject.toml").read_bytes())]
    added += [(f"{root}/{_COMMON.name}/src/{name}", data) for name, data in common_sources()]

    with tar_open(sdist, "w:gz") as out:
        for member, data in held:
            out.addfile(member, BytesIO(data) if member.isfile() else None)
        for name, data in added:
            info = TarInfo(name)
            info.size = len(data)
            info.mode = 0o644
            out.addfile(info, BytesIO(data))


def own_metadata(metadata: bytes) -> bytes:
    deps_block = search(r"(?ms)^dependencies = \[(.*?)\]", (_COMMON / "pyproject.toml").read_text())
    if not (deps := findall(r'"([^"]*)"', deps_block.group(1)) if deps_block else []):
        return metadata
    # Requires-Dist belongs in the header block; everything past the first blank line is the description
    headers, sep, description = metadata.decode().partition("\n\n")
    return "".join([headers.rstrip("\n"), *(f"\nRequires-Dist: {d}" for d in deps), sep, description]).encode()


def common_sources() -> Iterator[tuple[str, bytes]]:
    src = _COMMON / "src" / _VENDOR
    # vendor only the package's Python sources; local build artifacts (bytecode, caches, ext modules) never leak
    for file in sorted(src.rglob("*")):
        if file.is_file() and (file.suffix in {".py", ".pyi"} or file.name == "py.typed"):
            yield file.relative_to(src.parent).as_posix(), file.read_bytes()


if __name__ == "__main__":
    main()
