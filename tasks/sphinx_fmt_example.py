"""Render ``fmt-example`` through the live formatter during Sphinx builds."""

from __future__ import annotations

from typing import TYPE_CHECKING, ClassVar

from docutils import nodes
from docutils.parsers.rst import Directive, directives
from fmt_examples import render_example

if TYPE_CHECKING:
    from sphinx.application import Sphinx


class _FmtExample(Directive):
    has_content = True
    option_spec: ClassVar = {"config": directives.unchanged}

    def run(self) -> list[nodes.Node]:
        module = self.state.document.settings.env.config.fmt_example_module
        text = render_example(module, "\n".join(self.content), self.options.get("config", ""))
        node = nodes.literal_block(text, text)
        node["language"] = "toml"
        return [node]


def setup(app: Sphinx) -> dict[str, bool]:
    app.add_config_value("fmt_example_module", "", "env")
    app.add_directive("fmt-example", _FmtExample)
    return {"parallel_read_safe": True, "parallel_write_safe": True}


__all__ = [
    "setup",
]
