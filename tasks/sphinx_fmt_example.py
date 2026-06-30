"""Sphinx ``fmt-example`` directive: render TOML examples via the live formatter.

The directive body is the input TOML; the formatted output is computed at build
time, so documented behavior can never drift from the installed formatter. The
``:config:`` option passes ``key=value`` overrides to the formatter.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, ClassVar

from docutils import nodes
from docutils.parsers.rst import Directive, directives
from fmt_examples import render_example

if TYPE_CHECKING:
    from sphinx.application import Sphinx


class FmtExample(Directive):
    has_content = True
    option_spec: ClassVar = {"config": directives.unchanged}

    def run(self) -> list[nodes.Node]:
        module = self.state.document.settings.env.config.fmt_example_module
        before = "\n".join(self.content)
        text = render_example(module, before, self.options.get("config", ""))
        node = nodes.literal_block(text, text)
        node["language"] = "toml"
        return [node]


def setup(app: Sphinx) -> dict[str, bool]:
    app.add_config_value("fmt_example_module", "", "env")
    app.add_directive("fmt-example", FmtExample)
    return {"parallel_read_safe": True, "parallel_write_safe": True}
