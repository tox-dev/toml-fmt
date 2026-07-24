from __future__ import annotations

import sys
from pathlib import Path
from runpy import run_path
from types import FunctionType, SimpleNamespace
from typing import TYPE_CHECKING
from zipfile import ZipFile

if TYPE_CHECKING:
    import pytest


def test_vendor_into_wheel_skips_bytecode(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setitem(sys.modules, "maturin", SimpleNamespace())
    backend = run_path(str(Path(__file__).parents[1] / "build_backend.py"))

    common = tmp_path / "toml-fmt-common"
    source = common / "src" / "toml_fmt_common"
    cache = source / "__pycache__"
    cache.mkdir(parents=True)
    (source / "__init__.py").write_text("", encoding="utf-8")
    (source / "module.py").write_text("VALUE = 1\n", encoding="utf-8")
    (source / "module.pyc").write_bytes(b"bytecode")
    (source / "module.pyo").write_bytes(b"optimized bytecode")
    (cache / "__init__.cpython-312.pyc").write_bytes(b"cached bytecode")
    (cache / "marker.txt").write_text("not package data", encoding="utf-8")
    (common / "pyproject.toml").write_text("dependencies = []\n", encoding="utf-8")

    wheel = tmp_path / "pyproject_fmt-0-py3-none-any.whl"
    with ZipFile(wheel, "w") as archive:
        archive.writestr("pyproject_fmt/__main__.py", "import toml_fmt_common\n")
        archive.writestr("pyproject_fmt-0.dist-info/METADATA", "Requires-Dist: toml-fmt-common\n")
        archive.writestr("pyproject_fmt-0.dist-info/RECORD", "")

    vendor_into_wheel = backend["vendor_into_wheel"]
    assert isinstance(vendor_into_wheel, FunctionType)
    monkeypatch.setitem(vendor_into_wheel.__globals__, "_COMMON", common)
    vendor_into_wheel(wheel)

    with ZipFile(wheel) as archive:
        vendor_files = {name for name in archive.namelist() if name.startswith("pyproject_fmt/_vendor/")}

    assert vendor_files == {
        "pyproject_fmt/_vendor/__init__.py",
        "pyproject_fmt/_vendor/toml_fmt_common/__init__.py",
        "pyproject_fmt/_vendor/toml_fmt_common/module.py",
    }
