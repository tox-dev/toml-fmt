"""
Vendor toml-fmt-common into each wheel and source distribution.

Published consumers resolve new toml-fmt-common releases, so each artifact carries matching sources
(tox-dev/toml-fmt#355). ``maturin build`` skips PEP 517 backends; the CLI patches maturin-action output.
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
from tempfile import TemporaryDirectory
from typing import TYPE_CHECKING, Final, TypeAlias
from zipfile import ZIP_DEFLATED, ZipFile

import maturin

if TYPE_CHECKING:
    from collections.abc import Callable, Iterator, Mapping

    _ConfigSettings: TypeAlias = Mapping[str, str | list[str]]

# Maturin warns when a wrapper backend omits this switch.
environ.setdefault("MATURIN_NO_MISSING_BUILD_BACKEND_WARNING", "1")

_HERE: Final[Path] = Path(__file__).resolve().parent
# the sdist keeps a copy of this file beside the tests, a directory below the pyproject.toml it belongs to
_OWN: Final[Path] = next(path for path in (_HERE / "pyproject.toml", _HERE.parent / "pyproject.toml") if path.is_file())
_MODULE: Final[str] = findall(r'(?m)^name = "(.*)"', _OWN.read_text())[0].replace("-", "_")
_VENDOR: Final[str] = "toml_fmt_common"
# a checkout keeps toml-fmt-common beside the package; the sdist carries it within
_COMMON: Final[Path] = (
    _HERE / "toml-fmt-common" if (_HERE / "toml-fmt-common").is_dir() else _HERE.parent / "toml-fmt-common"
)


def build_wheel(
    wheel_directory: str,
    config_settings: _ConfigSettings | None = None,
    metadata_directory: str | None = None,
) -> str:
    return _built(maturin.build_wheel, wheel_directory, config_settings, metadata_directory, _vendor_into_wheel)


def build_sdist(sdist_directory: str, config_settings: _ConfigSettings | None = None) -> str:
    name = maturin.build_sdist(sdist_directory, config_settings)
    if _common_is_present():
        _vendor_into_sdist(Path(sdist_directory) / name)
    return name


def build_editable(
    wheel_directory: str,
    config_settings: _ConfigSettings | None = None,
    metadata_directory: str | None = None,
) -> str:
    return _built(maturin.build_editable, wheel_directory, config_settings, metadata_directory, _link_common_into_wheel)


def get_requires_for_build_wheel(config_settings: _ConfigSettings | None = None) -> list[str]:
    return maturin.get_requires_for_build_wheel(config_settings)


def get_requires_for_build_sdist(config_settings: _ConfigSettings | None = None) -> list[str]:
    return maturin.get_requires_for_build_sdist(config_settings)


def get_requires_for_build_editable(config_settings: _ConfigSettings | None = None) -> list[str]:
    return maturin.get_requires_for_build_editable(config_settings)


def _built(
    build: Callable[[str, _ConfigSettings | None, str | None], str],
    wheel_directory: str,
    config_settings: _ConfigSettings | None,
    metadata_directory: str | None,
    carry: Callable[[Path], None],
) -> str:
    if not _common_is_present():
        return build(wheel_directory, config_settings, metadata_directory)
    with TemporaryDirectory() as directory:
        temporary = Path(directory)
        name = build(str(temporary), config_settings, metadata_directory)
        carry(temporary / name)
        copy2(temporary / name, Path(wheel_directory) / name)
        return name


def _common_is_present() -> bool:
    return (_COMMON / "src" / _VENDOR).is_dir()


def _main() -> None:
    target = Path(argv[1])
    if not _common_is_present():
        print(f"no toml-fmt-common sources under {_COMMON}")
        raise SystemExit(1)
    if not (wheels := sorted(target.glob("*.whl")) if target.is_dir() else [target]):
        print(f"no wheels found in {target}")
        raise SystemExit(1)
    for wheel in wheels:
        _vendor_into_wheel(wheel)
        print(f"vendored toml-fmt-common into {wheel.name}")


def _vendor_into_wheel(wheel: Path) -> None:
    vendor_path = f"{_MODULE}/_vendor/"
    changed = {f"{vendor_path}__init__.py": b""} | {f"{vendor_path}{name}": data for name, data in _common_sources()}
    with ZipFile(wheel) as source:
        if (entry := f"{_MODULE}/__main__.py") in source.namelist():
            spelled = sub(rf"\b{_VENDOR}\b", f"{_MODULE}._vendor.{_VENDOR}", source.read(entry).decode())
            changed[entry] = spelled.encode()
    _rewrite(wheel, changed)


def _link_common_into_wheel(wheel: Path) -> None:
    # an editable install reads the package from the source tree, where the import stays unvendored
    _rewrite(wheel, {f"{_MODULE}_{_VENDOR}.pth": f"{_COMMON / 'src'}\n".encode()})


def _rewrite(wheel: Path, changed: dict[str, bytes]) -> None:
    with ZipFile(wheel) as source:
        names = source.namelist()
        dist_info = next(n for n in names if n.endswith(".dist-info/METADATA")).split("/")[0]
        entries = {name: source.read(name) for name in names if not name.endswith("/RECORD")}
    entries.update(changed)
    entries[f"{dist_info}/METADATA"] = _own_metadata(entries[f"{dist_info}/METADATA"])

    record = []
    for name, data in entries.items():
        digest = urlsafe_b64encode(sha256(data).digest()).rstrip(b"=").decode()
        record.append(f"{name},sha256={digest},{len(data)}")
    record.append(f"{dist_info}/RECORD,,")
    entries[f"{dist_info}/RECORD"] = ("\n".join(record) + "\n").encode()

    with ZipFile(wheel, "w", ZIP_DEFLATED) as archive:
        for name, data in entries.items():
            archive.writestr(name, data)


def _vendor_into_sdist(sdist: Path) -> None:
    members = []
    with tar_open(sdist) as source:
        for member in source.getmembers():
            content = source.extractfile(member)
            members.append((member, content.read() if content else b""))
    root = members[0][0].name.split("/")[0]

    added = [(f"{root}/{_COMMON.name}/pyproject.toml", (_COMMON / "pyproject.toml").read_bytes())]
    added += [(f"{root}/{_COMMON.name}/src/{name}", data) for name, data in _common_sources()]

    with tar_open(sdist, "w:gz") as archive:
        for member, data in members:
            archive.addfile(member, BytesIO(data) if member.isfile() else None)
        for name, data in added:
            info = TarInfo(name)
            info.size = len(data)
            info.mode = 0o644
            archive.addfile(info, BytesIO(data))


def _own_metadata(metadata: bytes) -> bytes:
    dependency_block = search(r"(?ms)^dependencies = \[(.*?)\]", (_COMMON / "pyproject.toml").read_text())
    if not (dependencies := findall(r'"([^"]*)"', dependency_block.group(1)) if dependency_block else []):
        return metadata
    # Requires-Dist belongs in the header block; everything past the first blank line is the description
    headers, separator, description = metadata.decode().partition("\n\n")
    return "".join([
        headers.rstrip("\n"),
        *(f"\nRequires-Dist: {dependency}" for dependency in dependencies),
        separator,
        description,
    ]).encode()


def _common_sources() -> Iterator[tuple[str, bytes]]:
    source = _COMMON / "src" / _VENDOR
    # Build-host artifacts do not belong in a source distribution.
    for file in sorted(source.rglob("*")):
        if file.is_file() and (file.suffix in {".py", ".pyi"} or file.name == "py.typed"):
            yield file.relative_to(source.parent).as_posix(), file.read_bytes()


__all__ = [
    "build_editable",
    "build_sdist",
    "build_wheel",
    "get_requires_for_build_editable",
    "get_requires_for_build_sdist",
    "get_requires_for_build_wheel",
]


if __name__ == "__main__":
    _main()
