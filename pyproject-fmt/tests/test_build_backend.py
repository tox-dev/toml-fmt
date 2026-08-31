from __future__ import annotations

from io import BytesIO
from pathlib import Path
from runpy import run_path
from sys import modules
from tarfile import DIRTYPE, TarInfo
from tarfile import open as tar_open
from types import SimpleNamespace
from typing import TYPE_CHECKING, Final, NamedTuple
from zipfile import ZipFile

import pytest

if TYPE_CHECKING:
    from collections.abc import Callable

_BACKEND: Final[Path] = Path(__file__).parents[1] / "build_backend.py"
_METADATA: Final[str] = """\
Metadata-Version: 2.4
Name: pyproject-fmt
Version: 0
Description-Content-Type: text/markdown

# pyproject-fmt

Format your TOML.
"""
_SDIST_HELD: Final[bytes] = b'[project]\nname = "pyproject-fmt"\n'


class Backend(NamedTuple):
    vendor_into_wheel: Callable[[Path], None]
    link_common_into_wheel: Callable[[Path], None]
    vendor_into_sdist: Callable[[Path], None]


@pytest.fixture
def backend(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Backend:
    monkeypatch.setitem(modules, "maturin", SimpleNamespace())
    loaded = run_path(str(_BACKEND))

    common = tmp_path / "toml-fmt-common"
    src = common / "src" / "toml_fmt_common"
    src.mkdir(parents=True)
    (src / "__init__.py").write_text("VALUE = 1\n", encoding="utf-8", newline="")
    (src / "_lib.pyi").write_text("VALUE: int\n", encoding="utf-8", newline="")
    (src / "py.typed").write_text("", encoding="utf-8")
    (common / "pyproject.toml").write_text('dependencies = [\n  "tomlkit>=0.13",\n]\n', encoding="utf-8", newline="")

    cache = src / "__pycache__"
    cache.mkdir()
    (cache / "__init__.cpython-312.pyc").write_bytes(b"bytecode")
    (src / "module.pyc").write_bytes(b"bytecode")
    (src / "module.pyo").write_bytes(b"optimized")
    (src / "_speedups.so").write_bytes(b"\x7fELF")
    (src / ".DS_Store").write_bytes(b"junk")

    monkeypatch.setitem(loaded["vendor_into_wheel"].__globals__, "_COMMON", common)
    return Backend(
        loaded["vendor_into_wheel"],
        loaded["link_common_into_wheel"],
        loaded["vendor_into_sdist"],
    )


@pytest.fixture
def unvendored(tmp_path: Path) -> Path:
    path = tmp_path / "pyproject_fmt-0-py3-none-any.whl"
    with ZipFile(path, "w") as zf:
        zf.writestr("pyproject_fmt/__main__.py", "import toml_fmt_common\n")
        zf.writestr("pyproject_fmt-0.dist-info/METADATA", _METADATA)
        zf.writestr("pyproject_fmt-0.dist-info/RECORD", "")
    return path


@pytest.fixture
def wheel(backend: Backend, unvendored: Path) -> Path:
    backend.vendor_into_wheel(unvendored)
    return unvendored


@pytest.fixture
def editable(backend: Backend, unvendored: Path) -> Path:
    backend.link_common_into_wheel(unvendored)
    return unvendored


@pytest.fixture
def plain_sdist(tmp_path: Path) -> Path:
    path = tmp_path / "pyproject_fmt-0.tar.gz"
    root = TarInfo("pyproject_fmt-0")
    root.type = DIRTYPE
    held = TarInfo("pyproject_fmt-0/pyproject.toml")
    held.size = len(_SDIST_HELD)
    with tar_open(path, "w:gz") as tar:
        tar.addfile(root)
        tar.addfile(held, BytesIO(_SDIST_HELD))
    return path


@pytest.fixture
def sdist(backend: Backend, plain_sdist: Path) -> Path:
    backend.vendor_into_sdist(plain_sdist)
    return plain_sdist


def test_vendor_into_wheel_ships_only_python_sources(wheel: Path) -> None:
    with ZipFile(wheel) as zf:
        vendored = {n for n in zf.namelist() if n.startswith("pyproject_fmt/_vendor/")}
    assert vendored == {
        "pyproject_fmt/_vendor/__init__.py",
        "pyproject_fmt/_vendor/toml_fmt_common/__init__.py",
        "pyproject_fmt/_vendor/toml_fmt_common/_lib.pyi",
        "pyproject_fmt/_vendor/toml_fmt_common/py.typed",
    }


def test_vendor_into_wheel_requires_dist_in_header(wheel: Path) -> None:
    with ZipFile(wheel) as zf:
        metadata = zf.read("pyproject_fmt-0.dist-info/METADATA").decode()

    headers, _, description = metadata.partition("\n\n")
    assert headers.splitlines() == [
        "Metadata-Version: 2.4",
        "Name: pyproject-fmt",
        "Version: 0",
        "Description-Content-Type: text/markdown",
        "Requires-Dist: tomlkit>=0.13",
    ]
    assert description == "# pyproject-fmt\n\nFormat your TOML.\n"


def test_vendor_into_wheel_spells_the_entry_point_import_vendored(wheel: Path) -> None:
    with ZipFile(wheel) as zf:
        assert zf.read("pyproject_fmt/__main__.py") == b"import pyproject_fmt._vendor.toml_fmt_common\n"


def test_link_common_into_wheel_points_at_the_source_tree(editable: Path, tmp_path: Path) -> None:
    with ZipFile(editable) as zf:
        assert zf.read("pyproject_fmt_toml_fmt_common.pth").decode() == f"{tmp_path / 'toml-fmt-common' / 'src'}\n"


def test_link_common_into_wheel_copies_nothing(editable: Path) -> None:
    with ZipFile(editable) as zf:
        assert [n for n in zf.namelist() if n.startswith("toml_fmt_common/")] == []


def test_vendor_into_sdist_carries_common(sdist: Path) -> None:
    with tar_open(sdist) as tar:
        assert set(tar.getnames()) == {
            "pyproject_fmt-0",
            "pyproject_fmt-0/pyproject.toml",
            "pyproject_fmt-0/toml-fmt-common/pyproject.toml",
            "pyproject_fmt-0/toml-fmt-common/src/toml_fmt_common/__init__.py",
            "pyproject_fmt-0/toml-fmt-common/src/toml_fmt_common/_lib.pyi",
            "pyproject_fmt-0/toml-fmt-common/src/toml_fmt_common/py.typed",
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
        read = tar.extractfile(f"pyproject_fmt-0/{member}")
        assert read is not None
        assert read.read() == content
