from __future__ import annotations

from io import BytesIO
from pathlib import Path
from runpy import run_path
from shutil import copy2
from sys import modules
from tarfile import DIRTYPE, TarInfo
from tarfile import open as tar_open
from typing import TYPE_CHECKING, Final, NamedTuple, Protocol, TypeAlias, cast
from zipfile import ZipFile

import pytest

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping

    from pytest_mock import MockerFixture

    ConfigSettings: TypeAlias = Mapping[str, str | list[str]]
    BuildWheel: TypeAlias = Callable[[str, ConfigSettings | None, str | None], str]
    BuildSdist: TypeAlias = Callable[[str, ConfigSettings | None], str]

_BACKEND: Final[Path] = Path(__file__).parents[1] / "build_backend.py"
_DISTRIBUTION: Final[str] = "tox-toml-fmt"
_MODULE: Final[str] = "tox_toml_fmt"
_WHEEL: Final[str] = f"{_MODULE}-0-py3-none-any.whl"
_SDIST: Final[str] = f"{_MODULE}-0.tar.gz"
_METADATA: Final[str] = """\
Metadata-Version: 2.4
Name: tox-toml-fmt
Version: 0
Description-Content-Type: text/markdown

# tox-toml-fmt

Format your TOML.
"""
_SDIST_HELD: Final[bytes] = b'[project]\nname = "tox-toml-fmt"\n'


class Backend(NamedTuple):
    build_wheel: BuildWheel
    build_editable: BuildWheel
    build_sdist: BuildSdist


class Maturin(Protocol):
    @staticmethod
    def build_wheel(
        wheel_directory: str,
        config_settings: ConfigSettings | None = None,
        metadata_directory: str | None = None,
    ) -> str: ...

    @staticmethod
    def build_editable(
        wheel_directory: str,
        config_settings: ConfigSettings | None = None,
        metadata_directory: str | None = None,
    ) -> str: ...

    @staticmethod
    def build_sdist(sdist_directory: str, config_settings: ConfigSettings | None = None) -> str: ...


@pytest.fixture
def backend(tmp_path: Path, mocker: MockerFixture) -> Backend:
    package = tmp_path / "package"
    package.mkdir()
    copy2(_BACKEND, package / "build_backend.py")
    (package / "pyproject.toml").write_text(f'[project]\nname = "{_DISTRIBUTION}"\n', encoding="utf-8", newline="")
    _write_common(package / "toml-fmt-common")

    maturin = mocker.MagicMock(
        spec=Maturin,
        build_wheel=mocker.create_autospec(Maturin.build_wheel, side_effect=_write_wheel),
        build_editable=mocker.create_autospec(Maturin.build_editable, side_effect=_write_wheel),
        build_sdist=mocker.create_autospec(Maturin.build_sdist, side_effect=_write_sdist),
    )
    mocker.patch.dict(modules, {"maturin": maturin})
    loaded = run_path(str(package / "build_backend.py"))
    return Backend(
        cast("BuildWheel", loaded["build_wheel"]),
        cast("BuildWheel", loaded["build_editable"]),
        cast("BuildSdist", loaded["build_sdist"]),
    )


@pytest.fixture
def wheel(backend: Backend, tmp_path: Path) -> Path:
    return tmp_path / backend.build_wheel(str(tmp_path), None, None)


@pytest.fixture
def editable(backend: Backend, tmp_path: Path) -> Path:
    return tmp_path / backend.build_editable(str(tmp_path), None, None)


@pytest.fixture
def sdist(backend: Backend, tmp_path: Path) -> Path:
    return tmp_path / backend.build_sdist(str(tmp_path), None)


def test_vendor_into_wheel_ships_only_python_sources(wheel: Path) -> None:
    with ZipFile(wheel) as archive:
        vendored = {name for name in archive.namelist() if name.startswith("tox_toml_fmt/_vendor/")}
    assert vendored == {
        "tox_toml_fmt/_vendor/__init__.py",
        "tox_toml_fmt/_vendor/toml_fmt_common/__init__.py",
        "tox_toml_fmt/_vendor/toml_fmt_common/_lib.pyi",
        "tox_toml_fmt/_vendor/toml_fmt_common/py.typed",
    }


def test_vendor_into_wheel_requires_dist_in_header(wheel: Path) -> None:
    with ZipFile(wheel) as archive:
        metadata = archive.read("tox_toml_fmt-0.dist-info/METADATA").decode()

    headers, _, description = metadata.partition("\n\n")
    assert (headers.splitlines(), description) == (
        [
            "Metadata-Version: 2.4",
            "Name: tox-toml-fmt",
            "Version: 0",
            "Description-Content-Type: text/markdown",
            "Requires-Dist: tomlkit>=0.13",
        ],
        "# tox-toml-fmt\n\nFormat your TOML.\n",
    )


def test_vendor_into_wheel_spells_the_entry_point_import_vendored(wheel: Path) -> None:
    with ZipFile(wheel) as archive:
        assert archive.read("tox_toml_fmt/__main__.py") == b"import tox_toml_fmt._vendor.toml_fmt_common\n"


def test_link_common_into_wheel_points_at_the_source_tree(editable: Path, tmp_path: Path) -> None:
    with ZipFile(editable) as archive:
        assert (
            archive.read("tox_toml_fmt_toml_fmt_common.pth").decode() == f"{tmp_path / 'package/toml-fmt-common/src'}\n"
        )


def test_link_common_into_wheel_copies_nothing(editable: Path) -> None:
    with ZipFile(editable) as archive:
        assert [name for name in archive.namelist() if name.startswith("toml_fmt_common/")] == []


def test_vendor_into_sdist_carries_common(sdist: Path) -> None:
    with tar_open(sdist) as tar:
        assert set(tar.getnames()) == {
            "tox_toml_fmt-0",
            "tox_toml_fmt-0/pyproject.toml",
            "tox_toml_fmt-0/toml-fmt-common/pyproject.toml",
            "tox_toml_fmt-0/toml-fmt-common/src/toml_fmt_common/__init__.py",
            "tox_toml_fmt-0/toml-fmt-common/src/toml_fmt_common/_lib.pyi",
            "tox_toml_fmt-0/toml-fmt-common/src/toml_fmt_common/py.typed",
        }


@pytest.mark.parametrize(
    ("member", "content"),
    [
        pytest.param("pyproject.toml", _SDIST_HELD, id="held"),
        pytest.param("toml-fmt-common/src/toml_fmt_common/__init__.py", b"VALUE = 1\n", id="added"),
    ],
)
def test_vendor_into_sdist_content(sdist: Path, member: str, content: bytes) -> None:
    with tar_open(sdist) as tar:
        read = tar.extractfile(f"tox_toml_fmt-0/{member}")
        assert read is not None
        assert read.read() == content


def _write_common(common: Path) -> None:
    source = common / "src" / "toml_fmt_common"
    source.mkdir(parents=True)
    (source / "__init__.py").write_text("VALUE = 1\n", encoding="utf-8", newline="")
    (source / "_lib.pyi").write_text("VALUE: int\n", encoding="utf-8", newline="")
    (source / "py.typed").write_text("", encoding="utf-8")
    (common / "pyproject.toml").write_text('dependencies = [\n  "tomlkit>=0.13",\n]\n', encoding="utf-8", newline="")

    cache = source / "__pycache__"
    cache.mkdir()
    (cache / "__init__.cpython-312.pyc").write_bytes(b"bytecode")
    (source / "module.pyc").write_bytes(b"bytecode")
    (source / "module.pyo").write_bytes(b"optimized")
    (source / "_speedups.so").write_bytes(b"\x7fELF")
    (source / ".DS_Store").write_bytes(b"junk")


def _write_wheel(
    wheel_directory: str,
    _config_settings: ConfigSettings | None = None,
    _metadata_directory: str | None = None,
) -> str:
    with ZipFile(Path(wheel_directory) / _WHEEL, "w") as archive:
        archive.writestr(f"{_MODULE}/__main__.py", "import toml_fmt_common\n")
        archive.writestr(f"{_MODULE}-0.dist-info/METADATA", _METADATA)
        archive.writestr(f"{_MODULE}-0.dist-info/RECORD", "")
    return _WHEEL


def _write_sdist(sdist_directory: str, _config_settings: ConfigSettings | None = None) -> str:
    with tar_open(Path(sdist_directory) / _SDIST, "w:gz") as archive:
        root = TarInfo(f"{_MODULE}-0")
        root.type = DIRTYPE
        archive.addfile(root)
        project = TarInfo(f"{_MODULE}-0/pyproject.toml")
        project.size = len(_SDIST_HELD)
        archive.addfile(project, BytesIO(_SDIST_HELD))
    return _SDIST
