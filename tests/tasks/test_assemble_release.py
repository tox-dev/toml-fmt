"""Release assembly in the shared changelog."""

from __future__ import annotations

import runpy
from pathlib import Path

import pytest

from tasks.assemble_release import assemble, main


@pytest.fixture
def release_tree(tmp_path: Path) -> Path:
    (tmp_path / "CHANGELOG.md").write_text(
        "# Changelog\n\n## pyproject-fmt\n\n### 1.0.0 (2026-01-01)\n\n## tox-toml-fmt\n\n"
    )
    (tmp_path / "changelog.d" / "pyproject-fmt").mkdir(parents=True)
    return tmp_path


def test_assemble_keeps_package_sections_and_category_order(release_tree: Path) -> None:
    fragments = release_tree / "changelog.d" / "pyproject-fmt"
    (fragments / "+fix.bugfix.md").write_text("Fix an error.\n")
    (fragments / "+feature.feature.md").write_text("Add a setting.\n")

    assemble(release_tree, "pyproject-fmt", "1.1.0", "2026-10-21")

    assert (release_tree / "CHANGELOG.md").read_text() == (
        "# Changelog\n\n## pyproject-fmt\n\n"
        '### 1.1.0 (2026-10-21)\n\n<a name="release-pyproject-fmt-1-1-0"></a>\n\n'
        '<a name="release-pyproject-fmt-1-1-0-added"></a>\n\n#### Added\n\n- Add a setting.\n\n'
        '<a name="release-pyproject-fmt-1-1-0-fixed"></a>\n\n#### Fixed\n\n- Fix an error.\n\n'
        "### 1.0.0 (2026-01-01)\n\n## tox-toml-fmt\n\n"
    )


def test_assemble_consumes_only_selected_fragments(release_tree: Path) -> None:
    selected = release_tree / "changelog.d" / "pyproject-fmt" / "123.feature.md"
    selected.write_text("Add a setting.\n")
    other = release_tree / "changelog.d" / "tox-toml-fmt" / "+other.feature.md"
    other.parent.mkdir()
    other.write_text("Add another setting.\n")

    assemble(release_tree, "pyproject-fmt", "1.1.0", "2026-10-21")

    assert (selected.exists(), other.read_text()) == (False, "Add another setting.\n")


@pytest.mark.parametrize(
    ("filename", "content", "message"),
    [
        pytest.param("oops.md", "Add a setting.", "invalid changelog fragment", id="no-category"),
        pytest.param("+name.other.md", "Add a setting.", "invalid changelog fragment", id="unknown-category"),
        pytest.param("+name.feature.md", "", "expected one nonempty paragraph", id="empty"),
        pytest.param("+name.feature.md", "One.\n\nTwo.", "expected one nonempty paragraph", id="two-paragraphs"),
        pytest.param("+name.feature.md", "- Existing bullet.", "expected one nonempty paragraph", id="bullet"),
    ],
)
def test_assemble_rejects_invalid_fragments(release_tree: Path, filename: str, content: str, message: str) -> None:
    (release_tree / "changelog.d" / "pyproject-fmt" / filename).write_text(content)

    with pytest.raises(ValueError, match=message):
        assemble(release_tree, "pyproject-fmt", "1.1.0", "2026-10-21")


def test_assemble_requires_unique_package_heading(release_tree: Path) -> None:
    (release_tree / "changelog.d" / "pyproject-fmt" / "+feature.feature.md").write_text("Add a setting.\n")
    (release_tree / "CHANGELOG.md").write_text("## tox-toml-fmt\n\n")

    with pytest.raises(ValueError, match="expected one changelog section"):
        assemble(release_tree, "pyproject-fmt", "1.1.0", "2026-10-21")


def test_main_uses_the_selected_package(release_tree: Path) -> None:
    (release_tree / "changelog.d" / "pyproject-fmt" / "+feature.feature.md").write_text("Add a setting.\n")

    main(["pyproject-fmt", "1.1.0", "--root", str(release_tree)])

    assert "### 1.1.0 (" in (release_tree / "CHANGELOG.md").read_text()


def test_command_assembles_release(release_tree: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    (release_tree / "changelog.d" / "pyproject-fmt" / "+feature.feature.md").write_text("Add a setting.\n")
    monkeypatch.setattr(
        "sys.argv", ["tasks/assemble_release.py", "pyproject-fmt", "1.1.0", "--root", str(release_tree)]
    )

    runpy.run_path(str(Path(__file__).parents[2] / "tasks" / "assemble_release.py"), run_name="__main__")

    assert "### 1.1.0 (" in (release_tree / "CHANGELOG.md").read_text()
