import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from plugin import format_ttlv_tree_lines, resolve_kmip_lsp_command


class SublimePluginTests(unittest.TestCase):
    def test_resolve_command_defaults(self) -> None:
        cmd = resolve_kmip_lsp_command(["/custom/kmip-lsp", "--stdio"])
        self.assertEqual(cmd, ["/custom/kmip-lsp", "--stdio"])

    def test_format_ttlv_tree_preserves_redaction(self) -> None:
        root = {
            "tagHex": "0x420042",
            "tagName": "Key Block",
            "itemType": "Structure",
            "valueLength": 16,
            "displayValue": "Structure (1 items)",
            "children": [
                {
                    "tagHex": "0x420043",
                    "tagName": "Key Material",
                    "itemType": "ByteString",
                    "valueLength": 8,
                    "displayValue": "[REDACTED - Click to reveal] (8 bytes)",
                    "children": [],
                }
            ],
        }
        lines = format_ttlv_tree_lines(root)
        self.assertEqual(len(lines), 2)
        self.assertIn("[REDACTED - Click to reveal]", lines[1])


if __name__ == "__main__":
    unittest.main()
