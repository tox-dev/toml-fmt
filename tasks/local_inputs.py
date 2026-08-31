"""What each distributable package is built from."""

from __future__ import annotations

from typing import TYPE_CHECKING, Final

if TYPE_CHECKING:
    from collections.abc import Iterable

#: The local inputs of each wheel, its Rust and Python dependencies included. A parser or shared tox
#: fix ships in the wheel whether or not the wrapper changed, so a commit touching one belongs in
#: that package's changelog and has to rebuild it. The build workflows filter on the same paths.
LOCAL_INPUTS: Final[dict[str, tuple[str, ...]]] = {
    "pyproject-fmt": (
        "toml-doc/",
        "common/",
        "toml-fmt-common/",
        "tox-rules/",
        "pyproject-fmt/",
        "Cargo.",
        "rust-toolchain.toml",
    ),
    "tox-toml-fmt": (
        "toml-doc/",
        "common/",
        "toml-fmt-common/",
        "tox-rules/",
        "tox-toml-fmt/",
        "Cargo.",
        "rust-toolchain.toml",
    ),
    # a pure Python library, which ships none of the Rust crates
    "toml-fmt-common": ("toml-fmt-common/",),
}


def affects(project: str, changed_files: Iterable[str]) -> bool:
    """Whether a change to these files reaches what `project` ships."""
    return any(file_path.startswith(LOCAL_INPUTS[project]) for file_path in changed_files)


__all__ = [
    "LOCAL_INPUTS",
    "affects",
]
