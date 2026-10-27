"""Keep the cargo-edit pins aligned with the pinned Rust toolchain."""

from __future__ import annotations

import runpy
from io import StringIO
from pathlib import Path

import pytest

from tasks.update_cargo_edit import candidate, current_version, main, update, version_parts


@pytest.fixture
def workflow_tree(tmp_path: Path) -> Path:
    workflows = tmp_path / ".github/workflows"
    workflows.mkdir(parents=True)
    for name in ("prepare-release.yaml", "_build.yaml"):
        (workflows / name).write_text("          crate: cargo-edit\n          version: 0.13.13\n", encoding="utf-8")
    (workflows / "update_dependencies.yaml").write_text(
        "cargo install cargo-edit --version 0.13.13 --locked\n", encoding="utf-8"
    )
    return tmp_path


def test_candidate_skips_newer_crate_requiring_newer_rust(workflow_tree: Path) -> None:
    assert candidate(workflow_tree, [{"num": "0.13.14", "rust_version": "1.97", "yanked": False}], "1.93.1") is None


def test_candidate_selects_highest_compatible_crate(workflow_tree: Path) -> None:
    versions = [
        {"num": "0.13.16", "rust_version": "1.97", "yanked": False},
        {"num": "0.13.15", "rust_version": "1.93", "yanked": False},
        {"num": "0.13.14", "rust_version": "1.91", "yanked": False},
    ]

    assert candidate(workflow_tree, versions, "1.93.1") == "0.13.15"


@pytest.mark.parametrize(
    "entry",
    [
        pytest.param({"num": "0.13.15", "rust_version": None, "yanked": False}, id="unknown-rust"),
        pytest.param({"num": "0.13.15", "rust_version": "1.90", "yanked": True}, id="yanked"),
        pytest.param({"num": "0.13.15-rc.1", "rust_version": "1.90", "yanked": False}, id="prerelease"),
        pytest.param({"num": "0.13.13", "rust_version": "1.90", "yanked": False}, id="current"),
        pytest.param({"num": "0.13.15", "rust_version": "preview", "yanked": False}, id="invalid-rust"),
    ],
)
def test_candidate_skips_unusable_crate(workflow_tree: Path, entry: dict[str, object]) -> None:
    assert candidate(workflow_tree, [entry], "1.93.1") is None


def test_update_changes_all_three_pins(workflow_tree: Path) -> None:
    update(workflow_tree, "0.13.14")

    assert current_version(workflow_tree) == "0.13.14"
    assert all(
        "0.13.13" not in path.read_text(encoding="utf-8") for path in (workflow_tree / ".github/workflows").iterdir()
    )


def test_update_rejects_older_version(workflow_tree: Path) -> None:
    with pytest.raises(ValueError, match="is not newer"):
        update(workflow_tree, "0.13.12")

    assert current_version(workflow_tree) == "0.13.13"


def test_update_rejects_mismatched_pin_without_changes(workflow_tree: Path) -> None:
    path = workflow_tree / ".github/workflows/_build.yaml"
    path.write_text("          crate: cargo-edit\n          version: 0.13.12\n", encoding="utf-8")

    with pytest.raises(ValueError, match="pins disagree"):
        update(workflow_tree, "0.13.14")

    assert "0.13.13" in (workflow_tree / ".github/workflows/prepare-release.yaml").read_text(encoding="utf-8")


def test_update_rejects_duplicate_pin_without_changes(workflow_tree: Path) -> None:
    path = workflow_tree / ".github/workflows/_build.yaml"
    path.write_text(path.read_text(encoding="utf-8") * 2, encoding="utf-8")

    with pytest.raises(ValueError, match="expected one cargo-edit pin"):
        update(workflow_tree, "0.13.14")

    assert "0.13.13" in (workflow_tree / ".github/workflows/prepare-release.yaml").read_text(encoding="utf-8")


def test_update_rejects_pin_without_final_newline(workflow_tree: Path) -> None:
    path = workflow_tree / ".github/workflows/_build.yaml"
    path.write_text("          crate: cargo-edit\n          version: 0.13.13", encoding="utf-8")

    with pytest.raises(ValueError, match="expected one cargo-edit pin"):
        update(workflow_tree, "0.13.14")

    assert "0.13.13" in (workflow_tree / ".github/workflows/prepare-release.yaml").read_text(encoding="utf-8")


def test_version_parts_rejects_invalid_version() -> None:
    with pytest.raises(ValueError, match="invalid version"):
        version_parts("0.13.preview")


def test_main_reports_compatible_version(
    workflow_tree: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setattr("sys.stdin", StringIO('[{"num":"0.13.14","rust_version":"1.93","yanked":false}]'))
    main(["candidate", "--rust-version", "1.93.1", "--root", str(workflow_tree)])

    assert capsys.readouterr().out == "0.13.14\n"


def test_main_requires_rust_versions(workflow_tree: Path) -> None:
    with pytest.raises(SystemExit, match="2"):
        main(["candidate", "--root", str(workflow_tree)])


def test_main_rejects_invalid_metadata(workflow_tree: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("sys.stdin", StringIO("{}"))

    with pytest.raises(ValueError, match=r"expected a crates\.io version list"):
        main(["candidate", "--rust-version", "1.93.1", "--root", str(workflow_tree)])


def test_main_requires_update_version(workflow_tree: Path) -> None:
    with pytest.raises(SystemExit, match="2"):
        main(["update", "--root", str(workflow_tree)])


def test_main_updates_workflows(workflow_tree: Path) -> None:
    main(["update", "0.13.14", "--root", str(workflow_tree)])

    assert current_version(workflow_tree) == "0.13.14"


def test_command_reports_compatible_version(
    workflow_tree: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setattr("sys.stdin", StringIO('[{"num":"0.13.14","rust_version":"1.93","yanked":false}]'))
    monkeypatch.setattr(
        "sys.argv",
        [
            "tasks/update_cargo_edit.py",
            "candidate",
            "--rust-version",
            "1.93.1",
            "--root",
            str(workflow_tree),
        ],
    )

    runpy.run_path(str(Path(__file__).parents[2] / "tasks" / "update_cargo_edit.py"), run_name="__main__")

    assert capsys.readouterr().out == "0.13.14\n"
