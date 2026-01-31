"""Generate README.rst from docs/index.rst and CHANGELOG.md for PyPI."""

from __future__ import annotations

import re
import sys
from pathlib import Path


def main(package: str) -> None:
    pkg = Path(package)
    if not (docs_path := pkg / "docs" / "index.rst").exists():
        return
    processed = process_rst_for_pypi(docs_path.read_text())
    changelog_rst = ""
    if (changelog_path := pkg / "CHANGELOG.md").exists() and (
        extracted := extract_latest_changelog_as_rst(changelog_path.read_text())
    ):
        changelog_rst = extracted
    if changelog_rst:
        if (pos := processed.find("\nPhilosophy")) != -1:
            result = processed[:pos] + f"\n{changelog_rst}\n" + processed[pos:]
        else:
            result = processed + "\n\n" + changelog_rst
    else:
        result = processed
    (pkg / "README.rst").write_text(result)


def process_rst_for_pypi(content: str) -> str:
    content = re.sub(r":pypi:`([^`]+)`", r"`\1 <https://pypi.org/project/\1>`_", content)
    content = re.sub(r":gh:`([^`]+)`", r"`\1 <https://github.com/\1>`_", content)
    result: list[str] = []
    skip_section = False
    skip_tab = False
    lines = content.splitlines()
    for i, line in enumerate(lines):
        if (
            any(s in line for s in ("Command line interface", "Configuration via file"))
            and i + 1 < len(lines)
            and (next_line := lines[i + 1])
            and all(c in "-~=" for c in next_line)
        ):
            skip_section = True
            continue
        if skip_section:
            if line and all(c in "-~=" for c in line):
                skip_section = False
            continue
        if line.startswith(".. tab:: uv"):
            continue
        if line.startswith((".. tab::", ".. automodule::", ".. toctree::", ".. sphinx_argparse_cli::")):
            skip_tab = True
            continue
        if skip_tab:
            if line and not line.startswith(" ") and not line.startswith("\t"):
                skip_tab = False
            else:
                continue
        result.append(line)
    return "\n".join(result).rstrip()


def extract_latest_changelog_as_rst(content: str) -> str | None:
    if (match := re.search(r"^## .+$", content, re.MULTILINE)) is None:
        return None
    rest = content[match.start() :]
    version_end = rest.find("\n") if "\n" in rest else len(rest)
    content_start = version_end + 1
    content_end = (
        content_start + next_match.start() if (next_match := re.search(r"\n## ", rest[content_start:])) else len(rest)
    )
    rst_result = "Recent Changes\n~~~~~~~~~~~~~~~~\n\n"
    for line in rest[content_start:content_end].strip().splitlines():
        if line.startswith("<a id="):
            continue
        if line.startswith("- "):
            rst_result += f"- {convert_md_to_rst_inline(line[2:])}\n"
        elif line:
            rst_result += f"{convert_md_to_rst_inline(line)}\n"
        else:
            rst_result += "\n"
    return rst_result.rstrip()


def convert_md_to_rst_inline(line: str) -> str:
    result = ""
    in_backtick = False
    for ch in line:
        if ch == "`":
            result += "``"
            in_backtick = not in_backtick
        else:
            result += ch
    if in_backtick:
        result += "``"
    while (start := result.find("[")) != -1:
        if (bracket_end := result.find("]", start)) == -1:
            break
        if (
            bracket_end + 1 < len(result)
            and result[bracket_end + 1] == "("
            and (paren_end := result.find(")", bracket_end + 2)) != -1
        ):
            link = f"`{result[start + 1 : bracket_end]} <{result[bracket_end + 2 : paren_end]}>`_"
            result = f"{result[:start]}{link}{result[paren_end + 1 :]}"
            continue
        break
    return result


if __name__ == "__main__":
    if len(sys.argv) != 2:  # noqa: PLR2004
        print("Usage: generate_readme.py <package>")
        sys.exit(1)
    main(sys.argv[1])
