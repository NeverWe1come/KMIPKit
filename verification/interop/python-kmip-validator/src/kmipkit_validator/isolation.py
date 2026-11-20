from __future__ import annotations

import uuid


class AttemptContext:
    def __init__(self, run_id: str, language: str, encoding: str, api_level: str) -> None:
        self.run_id = run_id
        self.language = language
        self.encoding = encoding
        self.api_level = api_level
        self.unique_suffix = uuid.uuid4().hex[:8]

    def generate_unique_name(self, prefix: str) -> str:
        return f"{prefix}-{self.language}-{self.encoding}-{self.api_level}-{self.unique_suffix}"
