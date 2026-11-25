from __future__ import annotations

import json
import shutil
import subprocess
from typing import Any

try:
    import sublime
    import sublime_plugin
    from LSP.plugin import AbstractPlugin, register_plugin, unregister_plugin
except ImportError:  # Allows unit testing outside Sublime Text runtime
    sublime = None  # type: ignore[assignment]
    sublime_plugin = None  # type: ignore[assignment]
    AbstractPlugin = object  # type: ignore[assignment,misc]

    def register_plugin(_cls: Any) -> None:
        pass

    def unregister_plugin(_cls: Any) -> None:
        pass


def resolve_kmip_lsp_command(configured_command: list[str] | None = None) -> list[str]:
    if configured_command:
        return configured_command
    found = shutil.which("kmip-lsp")
    return [found or "kmip-lsp", "--stdio"]


def format_ttlv_tree_lines(node: dict[str, Any], indent: int = 0) -> list[str]:
    prefix = "  " * indent
    line = (
        f"{prefix}{node.get('tagHex', '')} {node.get('tagName', '')} "
        f"[{node.get('itemType', '')}, {node.get('valueLength', 0)}B] = "
        f"{node.get('displayValue', '')}"
    )
    lines = [line]
    for child in node.get("children", []):
        lines.extend(format_ttlv_tree_lines(child, indent + 1))
    return lines


def decode_ttlv_via_cli(payload: str, binary: str = "kmip-lsp") -> dict[str, Any]:
    proc = subprocess.run(
        [binary, "decode", "--hex", payload],
        capture_output=True,
        text=True,
        check=True,
        timeout=10,
    )
    return json.loads(proc.stdout)


class LspKmipPlugin(AbstractPlugin):
    @classmethod
    def name(cls) -> str:
        return "LSP-kmip"

    @classmethod
    def needs_update_or_installation(cls) -> bool:
        return shutil.which("kmip-lsp") is None

    @classmethod
    def install_or_update(cls) -> None:
        pass


if sublime_plugin is not None:

    class KmipDecodeTtlvSelectionCommand(sublime_plugin.TextCommand):
        def run(self, _edit: Any) -> None:
            view = self.view
            sel = view.sel()
            if not sel:
                return
            text = view.substr(sel[0]).strip()
            if not text:
                sublime.status_message("KMIPKit: Select a hex TTLV payload first.")
                return
            try:
                decoded = decode_ttlv_via_cli(text, shutil.which("kmip-lsp") or "kmip-lsp")
                lines = [f"# KMIP TTLV Inspector ({decoded.get('byteLength', 0)} bytes)", ""]
                lines.extend(format_ttlv_tree_lines(decoded["root"]))
                out = view.window().new_file()
                out.set_name("KMIP TTLV Inspector.md")
                out.set_scratch(True)
                out.run_command("append", {"characters": "\n".join(lines)})
                out.set_read_only(True)
            except Exception as exc:
                sublime.error_message(f"KMIPKit decode error: {exc}")

    class KmipLookupSpecCommand(sublime_plugin.WindowCommand):
        def run(self) -> None:
            self.window.show_input_panel(
                "KMIP OASIS Tag or Name:",
                "0x420028",
                self._on_done,
                None,
                None,
            )

        def _on_done(self, query: str) -> None:
            try:
                proc = subprocess.run(
                    [shutil.which("kmip-lsp") or "kmip-lsp", "lookup", query],
                    capture_output=True,
                    text=True,
                    check=True,
                    timeout=10,
                )
                out = self.window.new_file()
                out.set_name(f"KMIP Spec - {query}.json")
                out.set_scratch(True)
                out.run_command("append", {"characters": proc.stdout})
                out.set_read_only(True)
            except Exception as exc:
                sublime.error_message(f"KMIPKit lookup error: {exc}")


def plugin_loaded() -> None:
    register_plugin(LspKmipPlugin)


def plugin_unloaded() -> None:
    unregister_plugin(LspKmipPlugin)
