class Settings:
    def __init__(
        self,
        *,
        column_width: int,
        indent: int,
    ) -> None: ...
    @property
    def column_width(self) -> int: ...
    @property
    def indent(self) -> int: ...

def format_toml(content: str, settings: Settings) -> str: ...
