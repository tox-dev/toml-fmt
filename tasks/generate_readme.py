"""Generate README.rst from the docs for PyPI."""

from __future__ import annotations

import re
import sys
from pathlib import Path

from fmt_examples import render_example


def main(package: str) -> None:
    pkg = Path(package)
    module = package.replace("-", "_")
    docs_dir = pkg / "docs"
    if not (index_path := docs_dir / "index.rst").exists():
        return

    def read(path: Path) -> str:
        return expand_fmt_examples(path.read_text(encoding="utf-8"), module)

    processed = process_rst_for_pypi(read(index_path))

    if (config_path := docs_dir / "configuration.rst").exists():
        processed += "\n\n" + process_rst_for_pypi(strip_main_title(read(config_path)))

    if (formatting_path := docs_dir / "formatting.rst").exists():
        processed += "\n\n" + process_rst_for_pypi(strip_main_title(read(formatting_path)))

    (pkg / "README.rst").write_text(processed.rstrip() + "\n", encoding="utf-8")


def expand_fmt_examples(content: str, module: str) -> str:
    """Replace ``.. fmt-example::`` directives with the static ``code-block`` they render to."""
    lines = content.splitlines()
    result: list[str] = []
    i = 0
    while i < len(lines):
        if (match := re.match(r"^(\s*)\.\. fmt-example::\s*$", lines[i])) is None:
            result.append(lines[i])
            i += 1
            continue
        base = match.group(1)
        config = ""
        i += 1
        while i < len(lines) and (option := re.match(rf"^{base}\s+:config:\s*(.*)$", lines[i])):
            config = option.group(1).strip()
            i += 1
        while i < len(lines) and not lines[i].strip():
            i += 1
        body: list[str] = []
        while i < len(lines) and (not lines[i].strip() or lines[i].startswith(f"{base} ")):
            body.append(lines[i])
            i += 1
        before = "\n".join(line[len(base) :] for line in body).strip("\n")
        rendered = render_example(module, _dedent(before), config)
        result.extend((f"{base}.. code-block:: toml", ""))
        result.extend(f"{base}   {line}" if line else "" for line in rendered.splitlines())
        result.append("")
    return "\n".join(result)


def _dedent(text: str) -> str:
    body = [line for line in text.splitlines() if line.strip()]
    pad = min((len(line) - len(line.lstrip()) for line in body), default=0)
    return "\n".join(line[pad:] if line.strip() else "" for line in text.splitlines())


def strip_main_title(content: str) -> str:
    lines = content.splitlines()
    if len(lines) >= 2 and lines[1] and all(c == "=" for c in lines[1]):  # ruff: ignore[magic-value-comparison]
        return "\n".join(lines[2:]).lstrip()
    return content


def unwrap_dropdowns(content: str) -> str:
    """Drop ``.. dropdown::`` directives and dedent their bodies; PyPI's renderer has no such directive."""
    result: list[str] = []
    in_dropdown = False
    for line in content.splitlines():
        if not in_dropdown:
            if line.startswith(".. dropdown::"):
                in_dropdown = True
            else:
                result.append(line)
        elif not line:
            result.append("")
        elif line.startswith("    "):
            result.append(line[4:])
        else:
            in_dropdown = False
            result.append(line)
    return "\n".join(result)


def process_rst_for_pypi(content: str) -> str:
    content = unwrap_dropdowns(content)
    content = re.sub(r":pypi:`([^`]+)`", r"`\1 <https://pypi.org/project/\1>`_", content)
    content = re.sub(r":gh:`([^`]+)`", r"`\1 <https://github.com/\1>`_", content)
    content = re.sub(r"^See :doc:`[^`]+`.*$\n?", "", content, flags=re.MULTILINE)
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
    pos = 0
    while (start := result.find("[", pos)) != -1:
        if (bracket_end := result.find("]", start)) == -1:
            break
        if (
            bracket_end + 1 < len(result)
            and result[bracket_end + 1] == "("
            and (paren_end := result.find(")", bracket_end + 2)) != -1
        ):
            link = f"`{result[start + 1 : bracket_end]} <{result[bracket_end + 2 : paren_end]}>`_"
            result = f"{result[:start]}{link}{result[paren_end + 1 :]}"
            pos = start + len(link)
        else:
            pos = bracket_end + 1
    return result


if __name__ == "__main__":
    if len(sys.argv) != 2:  # ruff: ignore[magic-value-comparison]
        print("Usage: generate_readme.py <package>")
        sys.exit(1)
    main(sys.argv[1])
