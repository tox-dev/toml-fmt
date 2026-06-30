"""Configuration for documentation build."""  # noqa: INP001

from __future__ import annotations

import sys
from datetime import datetime, timezone
from importlib.metadata import version as metadata_version
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parents[2] / "tasks"))

company, name = "tox-dev", "pyproject-fmt"
ver = metadata_version("pyproject-fmt")
release, version = ver, ".".join(ver.split(".")[:2])
now = datetime.now(tz=timezone.utc)
project_copyright = f"2022-{now.year}, {company}"
master_doc, source_suffix = "index", ".rst"

html_theme = "furo"
html_title, html_last_updated_fmt = name, now.isoformat()
pygments_style, pygments_dark_style = "sphinx", "monokai"

extensions = [
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
fmt_example_module = "pyproject_fmt"

exclude_patterns = ["_build", "changelog/*", "_draft.rst"]
autoclass_content, autodoc_member_order, autodoc_typehints = "class", "bysource", "none"
autodoc_default_options = {
    "member-order": "bysource",
    "undoc-members": True,
    "show-inheritance": True,
}

extlinks = {
    "pypi": ("https://pypi.org/project/%s", "%s"),
    "gh": ("https://github.com/%s", "%s"),
}
intersphinx_mapping = {"python": ("https://docs.python.org/3", None)}
nitpicky = True
nitpick_ignore = []
# anchors that resolve in a browser but linkcheck cannot verify: GitHub renders README heading ids with a
# user-content- prefix added client-side, and pyright's docs are a single-page app with hash-routed sections
linkcheck_anchors_ignore_for_url = [
    r"https://github\.com/.*",
    r"https://microsoft\.github\.io/pyright/.*",
]
