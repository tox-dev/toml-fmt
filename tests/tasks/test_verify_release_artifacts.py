from __future__ import annotations

import runpy
from hashlib import sha256
from pathlib import Path
from zipfile import ZipFile

import pytest

from tasks.verify_release_artifacts import main, verify_artifacts


def make_release_dist(tmp_path: Path, package: str) -> tuple[Path, str, str]:
    version = {"pyproject-fmt": "2.30.3", "tox-toml-fmt": "1.11.1", "toml-fmt-common": "1.3.5"}[package]
    prefix = f"{package.replace('-', '_')}-{version}"
    filenames = (
        {
            f"{prefix}-{tag}.whl"
            for tag in (
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
        }
        | {f"{prefix}.tar.gz"}
        if package != "toml-fmt-common"
        else {f"{prefix}-py3-none-any.whl", f"{prefix}.tar.gz"}
    )
    directory = tmp_path / "dist"
    directory.mkdir()
    for name in filenames:
        path = directory / name
        if package != "toml-fmt-common" and name.endswith(".whl"):
            suffix = ".pyd" if "-win_" in name else ".so"
            with ZipFile(path, "w") as wheel:
                wheel.writestr(f"{package.replace('-', '_')}/_lib.abi3{suffix}", b"native extension")
        else:
            path.write_bytes(f"{package}:{name}".encode())
    return directory, package, version


@pytest.fixture(params=["pyproject-fmt", "tox-toml-fmt", "toml-fmt-common"])
def release_dist(tmp_path: Path, request: pytest.FixtureRequest) -> tuple[Path, str, str]:
    return make_release_dist(tmp_path, str(request.param))


@pytest.fixture(params=["pyproject-fmt", "tox-toml-fmt"])
def rust_release_dist(tmp_path: Path, request: pytest.FixtureRequest) -> tuple[Path, str, str]:
    return make_release_dist(tmp_path, str(request.param))


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


def test_verify_artifacts_rejects_missing_native_extension(rust_release_dist: tuple[Path, str, str]) -> None:
    directory, package, version = rust_release_dist
    wheel = next(directory.glob("*-manylinux_2_28_x86_64.whl"))
    with ZipFile(wheel, "w") as archive:
        archive.writestr(f"{package.replace('-', '_')}/__init__.py", "")

    with pytest.raises(ValueError, match="expected one native extension"):
        verify_artifacts(package, version, directory)


def test_verify_artifacts_rejects_duplicate_native_extensions(rust_release_dist: tuple[Path, str, str]) -> None:
    directory, package, version = rust_release_dist
    wheel = next(directory.glob("*-manylinux_2_28_x86_64.whl"))
    with ZipFile(wheel, "a") as archive:
        archive.writestr(f"{package.replace('-', '_')}/_lib.second.so", b"another extension")

    with pytest.raises(ValueError, match="expected one native extension"):
        verify_artifacts(package, version, directory)


def test_verify_artifacts_rejects_invalid_wheel_archive(rust_release_dist: tuple[Path, str, str]) -> None:
    directory, package, version = rust_release_dist
    next(directory.glob("*-manylinux_2_28_x86_64.whl")).write_bytes(b"not a wheel")

    with pytest.raises(ValueError, match="invalid wheel archive"):
        verify_artifacts(package, version, directory)


def test_verify_artifacts_rejects_corrupt_native_extension(rust_release_dist: tuple[Path, str, str]) -> None:
    directory, package, version = rust_release_dist
    wheel = next(directory.glob("*-manylinux_2_28_x86_64.whl"))
    wheel.write_bytes(wheel.read_bytes().replace(b"native extension", b"broken extension", 1))

    with pytest.raises(ValueError, match="corrupt wheel member"):
        verify_artifacts(package, version, directory)


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
