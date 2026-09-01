from __future__ import annotations

import os
import subprocess
from pathlib import Path
from typing import TYPE_CHECKING, Final

from tox.plugin import impl

if TYPE_CHECKING:
    from tox.config.sets import EnvConfigSet
    from tox.session.state import State

_ENV_VAR: Final[str] = "_TOX_README_GENERATED"


@impl
def tox_add_env_config(env_conf: EnvConfigSet, state: State) -> None:
    if (
        os.environ.get(_ENV_VAR)
        or env_conf.name == "readme"
        or (readme := Path(state.conf.core["tox_root"]) / "README.rst").exists()
    ):
        return
    subprocess.check_call(["tox", "run", "-e", "readme"], cwd=readme.parent, env=os.environ | {_ENV_VAR: "1"})


__all__ = [
    "tox_add_env_config",
]
