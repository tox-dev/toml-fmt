"""Release file set and digest checks."""

from __future__ import annotations

import runpy
from hashlib import sha256
from pathlib import Path
from typing import Final

import pytest

from tasks.verify_release_artifacts import main, verify_artifacts

_RUST_TAGS: Final[tuple[str, ...]] = (
    "cp311-abi3-macosx_10_12_x86_64",
    "cp311-abi3-macosx_11_0_arm64",
    "cp311-abi3-manylinux_2_28_aarch64",
    "cp311-abi3-manylinux_2_28_x86_64",
    "cp311-abi3-manylinux_2_31_riscv64",
    "cp311-abi3-musllinux_1_2_aarch64",
    "cp311-abi3-musllinux_1_2_x86_64",
    "cp311-abi3-win_amd64",
    "cp311-abi3-win_arm64",
    "cp314-cp314t-macosx_10_12_x86_64",
    "cp314-cp314t-macosx_11_0_arm64",
    "cp314-cp314t-manylinux_2_28_aarch64",
    "cp314-cp314t-manylinux_2_28_x86_64",
    "cp314-cp314t-musllinux_1_2_aarch64",
    "cp314-cp314t-musllinux_1_2_x86_64",
    "cp314-cp314t-win_amd64",
    "cp314-cp314t-win_arm64",
    "cp315-cp315t-macosx_10_12_x86_64",
    "cp315-cp315t-macosx_11_0_arm64",
    "cp315-cp315t-manylinux_2_28_aarch64",
    "cp315-cp315t-manylinux_2_28_x86_64",
    "cp315-cp315t-musllinux_1_2_aarch64",
    "cp315-cp315t-musllinux_1_2_x86_64",
    "cp315-cp315t-win_amd64",
    "cp315-cp315t-win_arm64",
    "pp311-pypy311_pp73-macosx_10_12_x86_64",
    "pp311-pypy311_pp73-macosx_11_0_arm64",
    "pp311-pypy311_pp73-manylinux_2_28_x86_64",
)


@pytest.fixture(params=["pyproject-fmt", "tox-toml-fmt", "toml-fmt-common"])
def release_dist(tmp_path: Path, request: pytest.FixtureRequest) -> tuple[Path, str, str]:
    package = str(request.param)
    version = {"pyproject-fmt": "2.30.3", "tox-toml-fmt": "1.11.1", "toml-fmt-common": "1.3.5"}[package]
    prefix = f"{package.replace('-', '_')}-{version}"
    filenames = (
        {f"{prefix}-{tag}.whl" for tag in _RUST_TAGS} | {f"{prefix}.tar.gz"}
        if package != "toml-fmt-common"
        else {f"{prefix}-py3-none-any.whl", f"{prefix}.tar.gz"}
    )
    directory = tmp_path / "dist"
    directory.mkdir()
    for name in filenames:
        (directory / name).write_bytes(f"{package}:{name}".encode())
    return directory, package, version


def test_verify_artifacts_hashes_complete_release(release_dist: tuple[Path, str, str]) -> None:
    directory, package, version = release_dist

    manifest = verify_artifacts(package, version, directory)

    lines = manifest.splitlines()
    assert len(lines) == (2 if package == "toml-fmt-common" else 29)
    assert [line.split("  ", 1)[1] for line in lines] == sorted(path.name for path in directory.iterdir())
    first = lines[0].split("  ", 1)
    assert first[0] == sha256((directory / first[1]).read_bytes()).hexdigest()


def test_verify_artifacts_rejects_missing_file(release_dist: tuple[Path, str, str]) -> None:
    directory, package, version = release_dist
    next(directory.iterdir()).unlink()

    with pytest.raises(ValueError, match="missing"):
        verify_artifacts(package, version, directory)


def test_verify_artifacts_rejects_extra_file(release_dist: tuple[Path, str, str]) -> None:
    directory, package, version = release_dist
    (directory / "extra.whl").write_bytes(b"unexpected")

    with pytest.raises(ValueError, match="unexpected"):
        verify_artifacts(package, version, directory)


def test_verify_artifacts_rejects_non_file(release_dist: tuple[Path, str, str]) -> None:
    directory, package, version = release_dist
    file = next(directory.iterdir())
    file.unlink()
    file.mkdir()

    with pytest.raises(ValueError, match="regular files"):
        verify_artifacts(package, version, directory)


def test_verify_artifacts_rejects_symlink(release_dist: tuple[Path, str, str]) -> None:
    directory, package, version = release_dist
    file = next(directory.iterdir())
    file.unlink()
    file.symlink_to(next(directory.iterdir()).name)

    with pytest.raises(ValueError, match="regular files"):
        verify_artifacts(package, version, directory)


def test_verify_artifacts_rejects_missing_directory(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="artifact directory missing"):
        verify_artifacts("toml-fmt-common", "1.3.5", tmp_path / "missing")


def test_verify_artifacts_rejects_unknown_package(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="unknown release package"):
        verify_artifacts("unrelated", "1.0", tmp_path)


def test_main_writes_manifest(release_dist: tuple[Path, str, str]) -> None:
    directory, package, version = release_dist
    manifest = directory.parent / "dist.sha256"

    main([package, version, str(directory), str(manifest)])

    assert manifest.read_text() == verify_artifacts(package, version, directory)


def test_command_writes_manifest(release_dist: tuple[Path, str, str], monkeypatch: pytest.MonkeyPatch) -> None:
    directory, package, version = release_dist
    manifest = directory.parent / "dist.sha256"
    monkeypatch.setattr(
        "sys.argv", ["tasks/verify_release_artifacts.py", package, version, str(directory), str(manifest)]
    )

    runpy.run_path(str(Path(__file__).parents[2] / "tasks" / "verify_release_artifacts.py"), run_name="__main__")

    assert manifest.read_text() == verify_artifacts(package, version, directory)
