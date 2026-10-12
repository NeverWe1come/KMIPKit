"""Contract tests for deterministic non-Rust public declaration audits."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "audit_binding_exports.py"
SPEC = importlib.util.spec_from_file_location("audit_binding_exports", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)


class BindingExportAuditTests(unittest.TestCase):
    def test_c_header_inventory_includes_aliases_functions_macros_and_fields(self):
        header = """
            #define KMIPKIT_MODE_DEFAULT ((uint8_t)0)
            typedef struct kmipkit_handle_t kmipkit_handle_t;
            typedef struct {
                uint32_t count;
                const char *name;
            } kmipkit_options_t;
            int32_t kmipkit_handle_create(
                const kmipkit_options_t *options, kmipkit_handle_t **output);
        """
        result = AUDIT.inventory_c_header(header)
        actual = {(row["kind"], row["exportName"]) for row in result["declarations"]}
        self.assertEqual(actual, {
            ("macro", "KMIPKIT_MODE_DEFAULT"),
            ("type", "kmipkit_handle_t"),
            ("type", "kmipkit_options_t"),
            ("field", "kmipkit_options_t::count"),
            ("field", "kmipkit_options_t::name"),
            ("c-symbol", "kmipkit_handle_create"),
        })
        self.assertTrue(result["complete"])

    def test_java_source_inventory_includes_public_types_constructors_fields_and_methods(self):
        source = """
            package org.example;
            public final class Client {
                public static final int VERSION = 1;
                private int hidden;
                public Client(String name) {}
                public int execute(String request) { return 0; }
                public static class Nested {}
            }
        """
        result = AUDIT.inventory_java_sources({"src/org/example/Client.java": source})
        actual = {(row["kind"], row["exportName"]) for row in result["declarations"]}
        self.assertEqual(actual, {
            ("type", "org.example.Client"),
            ("java-member", "org.example.Client.VERSION"),
            ("java-member", "org.example.Client.Client"),
            ("java-member", "org.example.Client.execute"),
            ("type", "org.example.Client.Nested"),
        })
        self.assertTrue(result["complete"])

    def test_python_inventory_resolves_explicit_all_and_class_fields(self):
        sources = {
            "kmipkit/__init__.py": "from .core import Client\n__all__ = ['Client']\n",
            "kmipkit/core.py": """
                __all__ = ["Client", "create_client"]
                class Client:
                    name: str
                    def close(self) -> None: ...
                def create_client() -> Client: ...
            """,
        }
        result = AUDIT.inventory_python_sources(sources)
        actual = {(row["kind"], row["exportName"]) for row in result["declarations"]}
        self.assertEqual(actual, {
            ("type", "kmipkit.core.Client"),
            ("field", "kmipkit.core.Client::name"),
            ("python-export", "kmipkit.core.Client.close"),
            ("python-export", "kmipkit.core.create_client"),
            ("module", "kmipkit"),
        })
        self.assertTrue(result["complete"])

    def test_dynamic_python_exports_fail_closed(self):
        result = AUDIT.inventory_python_sources({"kmipkit/__init__.py": "__all__ = build_exports()\n"})
        self.assertFalse(result["complete"])
        self.assertEqual(result["issues"][0]["reason"], "dynamic-all")


if __name__ == "__main__":
    unittest.main()
