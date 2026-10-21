"""PyPI README rendering from Sphinx source."""

from __future__ import annotations

import runpy
from pathlib import Path
from typing import TYPE_CHECKING, cast

if TYPE_CHECKING:
    from collections.abc import Callable

    import pytest
    from pytest_mock import MockerFixture


def test_process_rst_for_pypi_rewrites_roles(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.syspath_prepend(str(Path(__file__).parents[2] / "tasks"))
    module = runpy.run_path(str(Path(__file__).parents[2] / "tasks" / "generate_readme.py"))
    process = cast("Callable[[str], str]", module["process_rst_for_pypi"])

    assert process(":pypi:`pyproject-fmt` and :gh:`tox-dev/toml-fmt`\nSee :doc:`configuration` for details.\n") == (
        "`pyproject-fmt <https://pypi.org/project/pyproject-fmt>`_ and "
        "`tox-dev/toml-fmt <https://github.com/tox-dev/toml-fmt>`_"
    )


def test_expand_fmt_examples_keeps_indent_and_config(monkeypatch: pytest.MonkeyPatch, mocker: MockerFixture) -> None:
    monkeypatch.syspath_prepend(str(Path(__file__).parents[2] / "tasks"))
    render = mocker.patch("fmt_examples.render_example", return_value="x = 2")
    module = runpy.run_path(str(Path(__file__).parents[2] / "tasks" / "generate_readme.py"))
    expand = cast("Callable[[str, str], str]", module["expand_fmt_examples"])

    assert expand("  .. fmt-example::\n    :config: column_width=30\n\n      x = 1", "pyproject_fmt") == (
        "  .. code-block:: toml\n\n     x = 2\n"
    )
    render.assert_called_once_with("pyproject_fmt", "x = 1", "column_width=30")
