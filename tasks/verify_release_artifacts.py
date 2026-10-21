from __future__ import annotations

import argparse
from hashlib import file_digest
from pathlib import Path
from typing import TYPE_CHECKING
from zipfile import BadZipFile, ZipFile

if TYPE_CHECKING:
    from collections.abc import Sequence


def main(argv: Sequence[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description="Verify one distribution's release files")
    parser.add_argument("package", choices=("pyproject-fmt", "tox-toml-fmt", "toml-fmt-common"))
    parser.add_argument("version")
    parser.add_argument("directory", type=Path)
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args(argv)
    args.manifest.write_text(verify_artifacts(args.package, args.version, args.directory), encoding="utf-8")


def verify_artifacts(package: str, version: str, directory: Path) -> str:
    if not directory.is_dir():
        msg = f"artifact directory missing: {directory}"
        raise ValueError(msg)
    prefix = f"{package.replace('-', '_')}-{version}"
    if package == "toml-fmt-common":
        expected = {f"{prefix}-py3-none-any.whl", f"{prefix}.tar.gz"}
    elif package in {"pyproject-fmt", "tox-toml-fmt"}:
        abi3_platforms = (
            "macosx_10_12_x86_64",
            "macosx_11_0_arm64",
            "manylinux_2_28_aarch64",
            "manylinux_2_28_x86_64",
            "manylinux_2_31_riscv64",
            "musllinux_1_2_aarch64",
            "musllinux_1_2_x86_64",
            "win_amd64",
            "win_arm64",
        )
        expected = {f"{prefix}-cp311-abi3-{platform}.whl" for platform in abi3_platforms}
        expected.update(
            f"{prefix}-{python}-{platform}.whl"
            for python in ("cp314-cp314t", "cp315-cp315t")
            for platform in abi3_platforms
            if "riscv64" not in platform
        )
        expected.update(
            f"{prefix}-pp311-pypy311_pp73-{platform}.whl"
            for platform in ("macosx_10_12_x86_64", "macosx_11_0_arm64", "manylinux_2_28_x86_64")
        )
        expected.add(f"{prefix}.tar.gz")
    else:
        msg = f"unknown release package: {package}"
        raise ValueError(msg)
    files = {path.name: path for path in directory.iterdir()}
    if files.keys() != expected:
        msg = (
            f"release files differ for {package}: missing {sorted(expected - files.keys())}, "
            f"unexpected {sorted(files.keys() - expected)}"
        )
        raise ValueError(msg)
    if any(path.is_symlink() or not path.is_file() for path in files.values()):
        msg = f"release files must be regular files: {directory}"
        raise ValueError(msg)
    if package != "toml-fmt-common":
        _verify_native_extensions(files, package.replace("-", "_"))
    hashes = []
    for name in sorted(files):
        with files[name].open("rb") as stream:
            hashes.append(f"{file_digest(stream, 'sha256').hexdigest()}  {name}")
    return "\n".join(hashes) + "\n"


def _verify_native_extensions(files: dict[str, Path], module: str) -> None:
    for name, path in files.items():
        if not name.endswith(".whl"):
            continue
        suffix = ".pyd" if "-win_" in name else ".so"
        try:
            with ZipFile(path) as wheel:
                if corrupt := wheel.testzip():
                    msg = f"corrupt wheel member: {name}: {corrupt}"
                    raise ValueError(msg)
                extensions = [
                    member
                    for member in wheel.namelist()
                    if member.startswith(f"{module}/_lib.") and member.endswith(suffix)
                ]
        except BadZipFile as error:
            msg = f"invalid wheel archive: {name}"
            raise ValueError(msg) from error
        if len(extensions) != 1:
            msg = f"expected one native extension in {name}: {extensions}"
            raise ValueError(msg)


if __name__ == "__main__":
    main()


__all__ = [
    "main",
    "verify_artifacts",
]
