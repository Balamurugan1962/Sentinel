from pydantic import ValidationError

from src.models import Config

try:
    config = Config.load()

except FileNotFoundError:
    print(
        """
## config.toml not found

Create a config.toml file in the project root.

Example:

root_server = "http://localhost:8080"

node_servers = [
  "http://student1.example.com",
  "http://student2.example.com",
  "http://student3.example.com"
]

mode = "Dev"
"""
    )
    exit(1)

except ValidationError as e:
    print(
        """
## Invalid config.toml format

Your config.toml is missing required fields or has invalid values.

Required format:

root_server = "http://localhost:8080"

node_servers = [
  "http://student1.example.com",
  "http://student2.example.com",
  "http://student3.example.com"
]

mode = "Dev"
"""
    )

    print("Details:")
    for err in e.errors():
        field = ".".join(map(str, err["loc"]))
        print(f"  - {field}: {err['msg']}")

    exit(1)
