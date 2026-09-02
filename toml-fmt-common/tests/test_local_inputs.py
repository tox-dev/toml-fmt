"""The ownership data the changelog and the build workflows share."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from typing import TYPE_CHECKING, Final

import pytest

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping

_ROOT: Final[Path] = Path(__file__).parents[2]


def _load() -> tuple[Mapping[str, tuple[str, ...]], Callable[[str, list[str]], bool]]:
    """Read the ownership data where the repository keeps it, without joining it to this package."""
    spec = importlib.util.spec_from_file_location("local_inputs", _ROOT / "tasks" / "local_inputs.py")
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.LOCAL_INPUTS, module.affects


LOCAL_INPUTS, affects = _load()


@pytest.mark.parametrize("project", ["pyproject-fmt", "tox-toml-fmt"])
@pytest.mark.parametrize(
    ("path", "reaches"),
    [
        pytest.param("toml-doc/src/lib.rs", True, id="parser"),
        pytest.param("common/src/layout.rs", True, id="shared-rust"),
        pytest.param("toml-fmt-common/src/toml_fmt_common/__init__.py", True, id="shared-python"),
        pytest.param("Cargo.lock", True, id="lockfile"),
        pytest.param("Cargo.toml", True, id="workspace-manifest"),
        pytest.param("README.md", False, id="unrelated"),
        pytest.param("docs/index.rst", False, id="docs"),
    ],
)
def test_local_inputs_shared_by_both_wheels(project: str, path: str, reaches: bool) -> None:
    assert affects(project, [path]) is reaches


@pytest.mark.parametrize(
    ("project", "path", "reaches"),
    [
        pytest.param("pyproject-fmt", "pyproject-fmt/rust/src/main.rs", True, id="own-files"),
        pytest.param("pyproject-fmt", "tox-rules/src/lib.rs", True, id="linked-tox-library"),
        pytest.param("pyproject-fmt", "tox-toml-fmt/rust/src/main.rs", False, id="not-a-dependency"),
        pytest.param("tox-toml-fmt", "tox-rules/src/lib.rs", True, id="shared-tox-rules"),
        pytest.param("tox-toml-fmt", "pyproject-fmt/rust/src/main.rs", False, id="not-a-dependency"),
    ],
)
def test_local_inputs_per_package(project: str, path: str, reaches: bool) -> None:
    assert affects(project, [path]) is reaches


@pytest.mark.parametrize(
    ("path", "reaches"),
    [
        pytest.param("toml-fmt-common/src/toml_fmt_common/__init__.py", True, id="own-files"),
        pytest.param("toml-doc/src/lib.rs", False, id="ships-no-rust"),
        pytest.param("common/src/layout.rs", False, id="ships-no-shared-rust"),
        pytest.param("README.md", False, id="unrelated"),
    ],
)
def test_local_inputs_of_the_python_library(path: str, reaches: bool) -> None:
    assert affects("toml-fmt-common", [path]) is reaches


def test_every_project_the_changelog_accepts_has_ownership() -> None:
    """The changelog reads its choices from the same mapping, so neither can name what the other does not."""
    assert set(LOCAL_INPUTS) == {"pyproject-fmt", "tox-toml-fmt", "toml-fmt-common"}


@pytest.mark.parametrize(
    ("changed", "reaches"),
    [
        pytest.param(["README.md", "toml-doc/src/lib.rs"], True, id="one-of-them-reaches"),
        pytest.param(["README.md", "docs/index.rst"], False, id="none-of-them-reaches"),
    ],
)
def test_affects_reads_every_changed_file(changed: list[str], reaches: bool) -> None:
    assert affects("tox-toml-fmt", changed) is reaches


@pytest.mark.parametrize("kind", ["build", "test"])
@pytest.mark.parametrize("project", ["pyproject-fmt", "tox-toml-fmt"])
def test_every_workflow_watches_exactly_the_local_inputs(project: str, kind: str) -> None:
    """A wheel rebuilds when what it ships changes, and watches nothing it does not ship."""
    workflow = (_ROOT / ".github" / "workflows" / f"{project.replace('-', '_')}_{kind}.yaml").read_text()

    assert _watched(workflow) == set(LOCAL_INPUTS[project])


def _watched(workflow: str) -> set[str]:
    """The path prefixes a workflow filters on, however its two filter lists quote them."""
    held = set()
    for line in workflow.splitlines():
        text = line.strip().lstrip("- ").strip("\"'")
        if text.endswith("/**"):
            held.add(text[:-2])
        elif text in {"Cargo.toml", "Cargo.lock"}:
            held.add("Cargo.")
        elif text == "rust-toolchain.toml":
            held.add(text)
    return held
