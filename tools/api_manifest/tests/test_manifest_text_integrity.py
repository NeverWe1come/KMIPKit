"""Guard reviewed API manifest text against UTF-8 decoding damage."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
MOJIBAKE_MARKERS = (
    "\u00c2\u00a7",
    "\u00e2\u20ac",
    "\u00c3\u201a",
    "\u00c3\u0192",
    "\u00c3\u00a2",
    "\u00c5\u201c",
    "\ufffd",
)


def _text_values(value: Any, path: str = "$") -> list[tuple[str, str]]:
    if isinstance(value, dict):
        return [
            item
            for key, nested in value.items()
            for item in _text_values(nested, f"{path}.{key}")
        ]
    if isinstance(value, list):
        return [
            item
            for index, nested in enumerate(value)
            for item in _text_values(nested, f"{path}[{index}]")
        ]
    if isinstance(value, str):
        return [(path, value)]
    return []


class ManifestTextIntegrityTests(unittest.TestCase):
    def test_manifest_strings_do_not_contain_utf8_mojibake_markers(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        damaged = [
            (path, text)
            for path, text in _text_values(manifest)
            if any(marker in text for marker in MOJIBAKE_MARKERS)
        ]

        preview = [f"{path}: {text[:100]!r}" for path, text in damaged[:5]]
        self.assertEqual(0, len(damaged), f"damaged text examples: {preview}")


if __name__ == "__main__":
    unittest.main()
