from __future__ import annotations

import sys
import tomllib
from pathlib import Path


def main(path: Path) -> None:
    with path.open("rb") as stream:
        print(tomllib.load(stream)["package"]["version"])


if __name__ == "__main__":
    main(Path(sys.argv[1]))


__all__ = [
    "main",
]
