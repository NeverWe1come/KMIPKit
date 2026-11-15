"""Guard reviewed API manifest text against UTF-8 decoding damage."""

from __future__ import annotations

import json
import unittest
from collections.abc import Iterator
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


def _text_values(value: Any, path: str = "$") -> Iterator[tuple[str, str]]:
    if isinstance(value, dict):
        for key, nested in value.items():
            yield from _text_values(nested, f"{path}.{key}")
    elif isinstance(value, list):
        for index, nested in enumerate(value):
            yield from _text_values(nested, f"{path}[{index}]")
    if isinstance(value, str):
        yield path, value


class ManifestTextIntegrityTests(unittest.TestCase):
    def test_manifest_strings_do_not_contain_utf8_mojibake_markers(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        damaged_count = 0
        examples = []
        for path, text in _text_values(manifest):
            if any(marker in text for marker in MOJIBAKE_MARKERS):
                damaged_count += 1
                if len(examples) < 5:
                    examples.append(f"{path}: {text[:100]!r}")

        self.assertEqual(0, damaged_count, f"damaged text examples: {examples}")


if __name__ == "__main__":
    unittest.main()
