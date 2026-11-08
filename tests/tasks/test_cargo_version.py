from __future__ import annotations

import tomllib
from typing import TYPE_CHECKING

import pytest

from tasks.cargo_version import main

if TYPE_CHECKING:
    from pathlib import Path


def test_main_reads_package_version(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    cargo_toml = tmp_path / "Cargo.toml"
    cargo_toml.write_text('[package]\nname = "example"\nversion = "1.2.3"\n[dependencies]\nversion = "9.0"\n')

    main(cargo_toml)

    assert capsys.readouterr().out == "1.2.3\n"


def test_main_rejects_invalid_toml(tmp_path: Path) -> None:
    cargo_toml = tmp_path / "Cargo.toml"
    cargo_toml.write_text("[package\n")

    with pytest.raises(tomllib.TOMLDecodeError):
        main(cargo_toml)
