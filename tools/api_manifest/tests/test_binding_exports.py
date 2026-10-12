"""Contract tests for deterministic non-Rust public declaration audits."""

from __future__ import annotations

import importlib.util
import struct
import textwrap
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
            ("field", "kmipkit_options_t.count"),
            ("field", "kmipkit_options_t.name"),
            ("c-symbol", "kmipkit_handle_create"),
        })
        self.assertTrue(result["complete"])

    def test_c_binary_export_audit_reconciles_header_and_dll_symbols(self):
        header = "int32_t kmipkit_client_create(void);\n"
        result = AUDIT.audit_c_surface(header, ["kmipkit_client_create", "kmipkit_client_destroy"])
        names = {row["exportName"] for row in result["declarations"] if row["kind"] == "c-symbol"}
        self.assertEqual(names, {"kmipkit_client_create", "kmipkit_client_destroy"})
        self.assertEqual({issue["reason"] for issue in result["issues"]},
                         {"exported-symbol-missing-from-header"})
        self.assertFalse(result["complete"])

    def test_pe_export_reader_extracts_named_abi_exports(self):
        image = bytearray(0x600)
        image[:2] = b"MZ"
        struct.pack_into("<I", image, 0x3C, 0x80)
        image[0x80:0x84] = b"PE\0\0"
        struct.pack_into("<HHIIIHH", image, 0x84, 0x8664, 1, 0, 0, 0, 0xF0, 0)
        optional = 0x98
        struct.pack_into("<H", image, optional, 0x20B)
        struct.pack_into("<I", image, optional + 108, 16)
        struct.pack_into("<II", image, optional + 112, 0x1000, 0x80)
        section = optional + 0xF0
        image[section:section + 8] = b".edata\0\0"
        struct.pack_into("<IIII", image, section + 8, 0x400, 0x1000, 0x400, 0x200)
        export = 0x200
        struct.pack_into("<II", image, export + 20, 1, 1)
        struct.pack_into("<III", image, export + 28, 0x1040, 0x1044, 0x1048)
        struct.pack_into("<I", image, 0x240, 0x1100)
        struct.pack_into("<I", image, 0x244, 0x1050)
        struct.pack_into("<H", image, 0x248, 0)
        image[0x250:0x266] = b"kmipkit_client_create\0"
        self.assertEqual(AUDIT.read_pe_exports(bytes(image)), ["kmipkit_client_create"])

    def test_javap_inventory_includes_public_types_constructors_fields_and_methods(self):
        outputs = {
            "org/example/Client.class": """
                public final class org.example.Client {
                  public static final int VERSION;
                  public org.example.Client(java.lang.String);
                  public int execute(java.lang.String);
                  protected void closeInternal();
                }
            """,
            "org/example/Client$Nested.class": """
                public class org.example.Client$Nested {
                  public org.example.Client$Nested();
                }
            """,
        }
        result = AUDIT.inventory_javap_outputs(outputs)
        actual = {(row["kind"], row["exportName"]) for row in result["declarations"]}
        self.assertEqual(actual, {
            ("type", "org.example.Client"),
            ("java-member", "org.example.Client.VERSION"),
            ("java-member", "org.example.Client.<init>"),
            ("java-member", "org.example.Client.execute"),
            ("java-member", "org.example.Client.closeInternal"),
            ("type", "org.example.Client.Nested"),
            ("java-member", "org.example.Client.Nested.<init>"),
        })
        self.assertTrue(result["complete"])

    def test_python_inventory_resolves_explicit_all_and_class_fields(self):
        sources = {
            "kmipkit/__init__.py": "from .core import Client\n__all__ = ['Client']\n",
            "kmipkit/core.py": textwrap.dedent("""
                __all__ = ["Client", "create_client"]
                class Client:
                    name: str
                    def close(self) -> None: ...
                def create_client() -> Client: ...
            """),
        }
        result = AUDIT.inventory_python_sources(sources)
        actual = {(row["kind"], row["exportName"]) for row in result["declarations"]}
        self.assertEqual(actual, {
            ("type", "kmipkit.Client"),
            ("type", "kmipkit.core.Client"),
            ("field", "kmipkit.core.Client.name"),
            ("python-export", "kmipkit.core.Client.close"),
            ("python-export", "kmipkit.core.create_client"),
            ("module", "kmipkit"),
            ("module", "kmipkit.core"),
        })
        self.assertTrue(result["complete"])

    def test_dynamic_python_exports_fail_closed(self):
        result = AUDIT.inventory_python_sources({"kmipkit/__init__.py": "__all__ = build_exports()\n"})
        self.assertFalse(result["complete"])
        self.assertEqual(result["issues"][0]["reason"], "dynamic-all")

    def test_python_inventory_includes_public_members_inherited_from_private_handle_base(self):
        sources = {
            "kmipkit/_handles.py": """
                class NativeHandle:
                    @property
                    def closed(self) -> bool: ...
                    def close(self) -> None: ...
                    def __enter__(self): ...
                    def __exit__(self, *args): ...
            """,
            "kmipkit/handle.py": """
                from ._handles import NativeHandle
                __all__ = ["Handle"]
                class Handle(NativeHandle):
                    pass
            """,
        }
        result = AUDIT.inventory_python_sources({key: textwrap.dedent(value) for key, value in sources.items()})
        names = {row["exportName"] for row in result["declarations"]}
        self.assertTrue({
            "kmipkit.handle.Handle.close",
            "kmipkit.handle.Handle.closed",
            "kmipkit.handle.Handle.__enter__",
            "kmipkit.handle.Handle.__exit__",
        }.issubset(names))
        self.assertTrue(result["complete"])

    def test_java_overloads_that_share_a_manifest_path_remain_incomplete(self):
        output = """
            public class org.example.Client {
              public void send(java.lang.String);
              public void send(byte[]);
            }
        """
        result = AUDIT.inventory_javap_outputs({"Client.class": output})
        self.assertFalse(result["complete"])
        self.assertEqual(result["issues"][0]["reason"], "multiple-declarations-share-export-path")

    def test_javap_inventory_keeps_compiler_generated_enum_members(self):
        output = """
            public final class org.example.Kind extends java.lang.Enum<org.example.Kind> {
              public static final org.example.Kind A;
              public static org.example.Kind[] values();
              public static org.example.Kind valueOf(java.lang.String);
            }
        """
        result = AUDIT.inventory_javap_outputs({"Kind.class": output})
        actual = {row["exportName"] for row in result["declarations"]}
        self.assertEqual(actual, {
            "org.example.Kind",
            "org.example.Kind.A",
            "org.example.Kind.values",
            "org.example.Kind.valueOf",
        })

    def test_comparison_rejects_duplicate_manifest_paths(self):
        manifest = {"publicDeclarations": [
            {"packageId": "java-kmipkit", "exportName": "org.example.Client", "kind": "type"},
            {"packageId": "java-kmipkit", "exportName": "org.example.Client", "kind": "type"},
        ], "approvedExclusions": []}
        report = {"complete": True, "declarations": [
            {"packageId": "java-kmipkit", "exportName": "org.example.Client", "kind": "type"},
        ], "issues": []}
        result = AUDIT.compare(report, manifest, "java-kmipkit")
        self.assertFalse(result["matchesManifest"])
        self.assertEqual(result["issues"][0]["reason"], "duplicate-manifest-export-name")


if __name__ == "__main__":
    unittest.main()
