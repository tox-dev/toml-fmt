from __future__ import annotations

import os
import subprocess
from pathlib import Path
from typing import TYPE_CHECKING

from tox.plugin import impl

if TYPE_CHECKING:
    from tox.config.sets import EnvConfigSet
    from tox.session.state import State


@impl
def tox_add_env_config(env_conf: EnvConfigSet, state: State) -> None:
    env_var: str = "_TOX_README_GENERATED"
    if (
        os.environ.get(env_var)
        or env_conf.name == "readme"
        or (readme := Path(state.conf.core["tox_root"]) / "README.rst").exists()
    ):
        return
    subprocess.check_call(["tox", "run", "-e", "readme"], cwd=readme.parent, env=os.environ | {env_var: "1"})


__all__ = []
