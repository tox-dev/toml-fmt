from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from toml_fmt_common import FmtNamespace, TOMLFormatter, run

if TYPE_CHECKING:
    from pathlib import Path

    from pytest_mock import MockerFixture


@pytest.mark.parametrize(
    ("no_color", "colored"), [(None, True), ("", True), ("1", False), ("0", False), ("false", False)]
)
def test_diff_color_respects_nonempty_no_color(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
    mocker: MockerFixture,
    no_color: str | None,
    colored: bool,
) -> None:
    if no_color is None:
        monkeypatch.delenv("NO_COLOR", raising=False)
    else:
        monkeypatch.setenv("NO_COLOR", no_color)
    path = tmp_path / "demo.toml"
    path.write_text("value = 1\n")
    formatter = mocker.Mock(spec=TOMLFormatter)
    formatter.opt = FmtNamespace()
    formatter.prog = "toml-fmt-common"
    formatter.filename = "demo.toml"
    formatter.override_cli_from_section = ("tool", "demo")
    formatter.settings_in.return_value = None
    formatter.format.return_value = "value = 2\n"

    assert run(formatter, ["--check", str(path)]) == 1

    out, err = capsys.readouterr()
    assert not err
    assert path.read_text() == "value = 1\n"
    red, green, reset = ("\x1b[31m", "\x1b[32m", "\x1b[0m") if colored else ("", "", "")
    assert out == (
        f"{red}--- {path}\n{reset}\n{green}+++ {path}\n{reset}\n"
        f"@@ -1 +1 @@\n\n{red}-value = 1{reset}\n{green}+value = 2{reset}\n"
    )
