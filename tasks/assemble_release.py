"""Put one package's news into the shared changelog."""

from __future__ import annotations

import argparse
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence


def main(argv: Sequence[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description="Assemble one package's release history")
    parser.add_argument("package", choices=("pyproject-fmt", "tox-toml-fmt", "toml-fmt-common"))
    parser.add_argument("version")
    parser.add_argument("--root", type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    assemble(args.root, args.package, args.version, datetime.now(UTC).date().isoformat())


def assemble(root: Path, package: str, version: str, date: str) -> None:
    """Write a package release from its merged fragments."""
    names = {
        "feature": "Added",
        "change": "Changes",
        "bugfix": "Fixed",
        "doc": "Documentation",
        "breaking": "Breaking changes",
        "security": "Security",
        "performance": "Performance",
        "deprecation": "Deprecated",
        "removal": "Removed",
        "packaging": "Packaging",
    }
    fragments = sorted((root / "changelog.d" / package).glob("*.md"))
    if not fragments:
        msg = f"no changelog fragments for {package}"
        raise ValueError(msg)
    sections: dict[str, list[str]] = {}
    for fragment in fragments:
        stem, separator, category = fragment.stem.rpartition(".")
        if (
            not separator
            or not stem
            or category not in names
            or not (stem.isdecimal() or (stem.startswith("+") and stem[1:].replace("-", "").isalnum()))
        ):
            msg = f"invalid changelog fragment: {fragment}"
            raise ValueError(msg)
        content = fragment.read_text(encoding="utf-8").strip()
        if not content or "\n\n" in content or content.startswith(("- ", "# ")):
            msg = f"expected one nonempty paragraph in {fragment}"
            raise ValueError(msg)
        sections.setdefault(names[category], []).append("- " + " ".join(line.strip() for line in content.splitlines()))
    changelog = root / "CHANGELOG.md"
    history = changelog.read_text(encoding="utf-8")
    heading = f"## {package}\n\n"
    if history.count(heading) != 1:
        msg = f"expected one changelog section for {package}"
        raise ValueError(msg)
    anchor = f"release-{package}-{version.replace('.', '-')}"
    release = [f"### {version} ({date})", "", f'<a name="{anchor}"></a>', ""]
    for category in dict.fromkeys(names.values()):
        if items := sections.get(category):
            release.extend((
                f'<a name="{anchor}-{category.lower().replace(" ", "-")}"></a>',
                "",
                f"#### {category}",
                "",
                *items,
                "",
            ))
    changelog.write_text(history.replace(heading, heading + "\n".join(release) + "\n", 1), encoding="utf-8")
    for fragment in fragments:
        fragment.unlink()


if __name__ == "__main__":
    main()


__all__ = [
    "assemble",
    "main",
]
