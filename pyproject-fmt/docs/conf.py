"""Configure the documentation build."""

from __future__ import annotations

import os
import sys
from datetime import UTC, datetime
from importlib.metadata import version as metadata_version
from pathlib import Path
from typing import Final

sys.path.insert(0, str(Path(__file__).parents[2] / "tasks"))

company, name = "tox-dev", "pyproject-fmt"
ver: Final[str] = metadata_version("pyproject-fmt")
release, version = ver, ".".join(ver.split(".")[:2])  # simplint: ignore[SMP530]  # one Sphinx config per package
now: Final[datetime] = datetime.now(tz=UTC)
project_copyright: Final[str] = f"2022-{now.year}, {company}"
master_doc, source_suffix = "index", ".rst"

html_theme: Final[str] = "furo"
html_title, html_last_updated_fmt = name, now.isoformat()
pygments_style, pygments_dark_style = "sphinx", "monokai"

extensions: Final[list[str]] = [
    "sphinx_llm.txt",
    "sphinx.ext.autodoc",
    "sphinx.ext.extlinks",
    "sphinx.ext.intersphinx",
    "sphinx_argparse_cli",
    "sphinx_autodoc_typehints",
    "sphinx_inline_tabs",
    "sphinx_copybutton",
    "sphinx_design",
    "sphinx_fmt_example",
]
fmt_example_module: Final[str] = "pyproject_fmt"

exclude_patterns: Final[list[str]] = [  # simplint: ignore[SMP530]  # one Sphinx config per package
    "_build",
    "changelog/*",
    "_draft.rst",
]
autoclass_content, autodoc_member_order, autodoc_typehints = "class", "bysource", "none"
autodoc_default_options: Final[dict[str, bool | str]] = {
    "member-order": "bysource",
    "undoc-members": True,
    "show-inheritance": True,
}

extlinks: Final[dict[str, tuple[str, str]]] = {
    "pypi": ("https://pypi.org/project/%s", "%s"),
    "gh": ("https://github.com/%s", "%s"),
}
intersphinx_mapping: Final[dict[str, tuple[str, None]]] = {"python": ("https://docs.python.org/3", None)}
nitpicky: Final[bool] = True
nitpick_ignore: Final[list[str]] = []
# anchors that resolve in a browser but linkcheck cannot verify: GitHub renders README heading ids with a
# user-content- prefix added client-side, and pyright's docs are a single-page app with hash-routed sections
linkcheck_anchors_ignore_for_url: Final[list[str]] = [
    r"https://github\.com/.*",
    r"https://microsoft\.github\.io/pyright/.*",
]

markdown_http_base: Final[str] = os.environ.get("READTHEDOCS_CANONICAL_URL", "")
# sphinx-markdown-builder cannot translate the label node of sphinx-inline-tabs, so llms.txt lacks tab titles
llms_txt_suppress_unknown_node_warnings: Final[list[str]] = ["label"]
