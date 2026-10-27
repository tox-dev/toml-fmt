"""Render one package's Markdown release history as an RST documentation page."""

from __future__ import annotations

import sys
from pathlib import Path

import pypandoc
from release_creator.changelog.preview import render_preview


def main(package: str) -> None:
    source = render_preview(Path("../CHANGELOG.md"), package, Path("../changelog.d") / package)
    lines = source.splitlines()
    anchors: dict[int, str] = {}
    for index, line in enumerate(lines):
        if not line.startswith('<a name="'):
            continue
        anchor = line.removeprefix('<a name="').removesuffix('"></a>')
        previous = next((position for position in range(index - 1, -1, -1) if lines[position].startswith("#")), None)
        if anchor.startswith("release-") and previous is not None and lines[previous].startswith("### "):
            heading = previous if previous not in anchors else None
        else:
            heading = None
        if heading is None:
            heading = next(
                (position for position in range(index + 1, len(lines)) if lines[position].startswith("#")), None
            )
        if heading is None or heading in anchors:
            msg = f"cannot attach changelog anchor {anchor} to a heading"
            raise ValueError(msg)
        anchors[heading] = anchor
    marked = "\n".join(
        f"{line} {{#{anchors[index]}}}" if index in anchors else line
        for index, line in enumerate(lines)
        if not line.startswith('<a name="')
    )
    converted = pypandoc.convert_text(marked, "rst", format="markdown+header_attributes", extra_args=["--columns=120"])
    Path("docs/changelog.rst").write_text(converted, encoding="utf-8")


if __name__ == "__main__":
    main(sys.argv[1])


__all__ = [
    "main",
]
