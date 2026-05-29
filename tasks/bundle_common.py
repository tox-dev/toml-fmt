# /// script
# requires-python = ">=3.11"
# ///
"""Bundle ``toml-fmt-common`` into a consumer wheel at build time.

The consumers (``pyproject-fmt``, ``tox-toml-fmt``) import ``toml_fmt_common`` from a
sibling workspace package during development, so editable installs and tox keep resolving
it from the worktree. At release the published wheel must not carry an external
``toml-fmt-common`` dependency — a new common release would otherwise retroactively break
every published consumer (tox-dev/toml-fmt#355).

This script runs only in the release build, on the throwaway checkout: it copies common
into the consumer's private ``_vendor`` namespace, repoints the single import there, and
replaces the ``toml-fmt-common`` dependency with common's own runtime dependencies. The
private namespace avoids a top-level ``toml_fmt_common`` collision when both tools share
an environment. Nothing here is committed; local development never sees it.
"""

from __future__ import annotations

import re
import shutil
from argparse import ArgumentParser
from pathlib import Path

import tomllib

VENDOR_NAME = "toml_fmt_common"


def main(package: str) -> None:
    module = package.replace("-", "_")
    root = Path(__file__).resolve().parent.parent
    common_pkg = root / "toml-fmt-common" / "src" / VENDOR_NAME
    consumer_src = root / package / "src" / module
    vendor_root = consumer_src / "_vendor"

    bundle_package(common_pkg, vendor_root)
    repoint_imports(consumer_src, vendor_root, module)
    swap_dependency(root / "toml-fmt-common" / "pyproject.toml", root / package / "pyproject.toml")


def bundle_package(common_pkg: Path, vendor_root: Path) -> None:
    target = vendor_root / VENDOR_NAME
    shutil.rmtree(vendor_root, ignore_errors=True)
    shutil.copytree(common_pkg, target)
    (vendor_root / "__init__.py").write_text("", encoding="utf-8")


def repoint_imports(consumer_src: Path, vendor_root: Path, module: str) -> None:
    pattern = re.compile(rf"\b{re.escape(VENDOR_NAME)}\b")
    replacement = f"{module}._vendor.{VENDOR_NAME}"
    for file in consumer_src.rglob("*.py"):
        if file.is_relative_to(vendor_root):
            continue
        text = file.read_text(encoding="utf-8")
        if (updated := pattern.sub(replacement, text)) != text:
            file.write_text(updated, encoding="utf-8")


def swap_dependency(common_pyproject: Path, consumer_pyproject: Path) -> None:
    with common_pyproject.open("rb") as handle:
        common_deps = tomllib.load(handle)["project"].get("dependencies", [])

    indent = "  "
    replacement = "".join(f'{indent}"{dep}",\n' for dep in common_deps)
    pattern = re.compile(rf'^{indent}"toml-fmt-common[^"]*",\n', re.MULTILINE)
    text = consumer_pyproject.read_text(encoding="utf-8")
    if not pattern.search(text):
        msg = f"toml-fmt-common dependency entry not found in {consumer_pyproject}"
        raise SystemExit(msg)
    consumer_pyproject.write_text(pattern.sub(replacement, text), encoding="utf-8")


if __name__ == "__main__":
    parser = ArgumentParser(description="Bundle toml-fmt-common into a consumer wheel at build time.")
    parser.add_argument("package", help="consumer package name, e.g. pyproject-fmt")
    main(parser.parse_args().package)
