from __future__ import annotations

from pathlib import Path
from runpy import run_path
from sys import modules
from types import SimpleNamespace
from typing import Final
from zipfile import ZipFile

import pytest

_BACKEND: Final[Path] = Path(__file__).parents[1] / "build_backend.py"
_METADATA: Final[str] = """\
Metadata-Version: 2.4
Name: pyproject-fmt
Version: 0
Requires-Dist: toml-fmt-common
Description-Content-Type: text/markdown

# pyproject-fmt

Format your TOML.
"""


@pytest.fixture
def wheel(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    monkeypatch.setitem(modules, "maturin", SimpleNamespace())
    vendor = run_path(str(_BACKEND))["vendor_into_wheel"]

    common = tmp_path / "toml-fmt-common"
    src = common / "src" / "toml_fmt_common"
    src.mkdir(parents=True)
    (src / "__init__.py").write_text("VALUE = 1\n", encoding="utf-8")
    (src / "_lib.pyi").write_text("VALUE: int\n", encoding="utf-8")
    (src / "py.typed").write_text("", encoding="utf-8")
    (common / "pyproject.toml").write_text('dependencies = [\n  "tomlkit>=0.13",\n]\n', encoding="utf-8")

    cache = src / "__pycache__"
    cache.mkdir()
    (cache / "__init__.cpython-312.pyc").write_bytes(b"bytecode")
    (src / "module.pyc").write_bytes(b"bytecode")
    (src / "module.pyo").write_bytes(b"optimized")
    (src / "_speedups.so").write_bytes(b"\x7fELF")
    (src / ".DS_Store").write_bytes(b"junk")

    path = tmp_path / "pyproject_fmt-0-py3-none-any.whl"
    with ZipFile(path, "w") as zf:
        zf.writestr("pyproject_fmt/__main__.py", "import toml_fmt_common\n")
        zf.writestr("pyproject_fmt-0.dist-info/METADATA", _METADATA)
        zf.writestr("pyproject_fmt-0.dist-info/RECORD", "")

    monkeypatch.setitem(vendor.__globals__, "_COMMON", common)
    vendor(path)
    return path


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
