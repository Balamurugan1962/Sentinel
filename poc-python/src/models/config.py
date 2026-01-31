import tomllib
from pathlib import Path
from typing import Literal

from pydantic_settings import BaseSettings


class Config(BaseSettings):
    root_server: str
    node_servers: list[str]
    mode: Literal["Dev", "Prod"] = "Dev"

    @classmethod
    def load(cls, path: Path | None = None) -> "Config":
        if path is None:
            path = Path(__file__).resolve().parents[3] / "config.toml"

        data = tomllib.loads(path.read_text(encoding="utf-8"))
        return cls(**data)
