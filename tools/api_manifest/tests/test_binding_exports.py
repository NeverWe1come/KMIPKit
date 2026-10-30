"""Contract tests for deterministic non-Rust public declaration audits."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import struct
import tempfile
import textwrap
import unittest
from pathlib import Path
from unittest.mock import patch


MODULE_PATH = Path(__file__).resolve().parents[1] / "audit_binding_exports.py"
SPEC = importlib.util.spec_from_file_location("audit_binding_exports", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)


class BindingExportAuditTests(unittest.TestCase):
    def test_java_compile_uses_gradle_configured_jspecify_classpath(self):
        repo_root = Path(__file__).resolve().parents[3]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_root = root / "src"
            package_info = source_root / "org" / "example" / "package-info.java"
            sample = source_root / "org" / "example" / "Sample.java"
            package_info.parent.mkdir(parents=True)
            package_info.write_text(
                "@org.jspecify.annotations.NullMarked\npackage org.example;\n",
                encoding="utf-8",
            )
            sample.write_text(
                "package org.example;\npublic final class Sample { String value; }\n",
                encoding="utf-8",
            )
            sources = [package_info, sample]
            output = root / "classes"
            output.mkdir()

            bare_javac = subprocess.run(
                ["javac", "--release", "17", "-d", str(output), *(str(path) for path in sources)],
                check=False,
                capture_output=True,
                text=True,
                encoding="utf-8",
            )
            self.assertNotEqual(bare_javac.returncode, 0)
            self.assertIn("org.jspecify.annotations", bare_javac.stderr)

            classpath = AUDIT.resolve_java_compile_classpath(repo_root)
            compile_result = subprocess.run(
                ["javac", "--release", "17", "-classpath", classpath, "-d", str(output),
                 *(str(path) for path in sources)],
                check=False,
                capture_output=True,
                text=True,
                encoding="utf-8",
            )

        self.assertEqual(compile_result.returncode, 0, compile_result.stderr)

    def test_c_only_audit_reuses_the_existing_library_and_skips_other_bindings(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest_path = root / "specification" / "api" / "public-api.json"
            manifest_path.parent.mkdir(parents=True)
            manifest_path.write_text(
                '{"publicDeclarations":[{"packageId":"c-kmipkit",'
                '"kind":"c-symbol","exportName":"kmipkit_probe",'
                '"declarationState":"declared"},{"packageId":"c-kmipkit",'
                '"kind":"c-symbol","exportName":"kmipkit_blocked_probe",'
                '"declarationState":"blocked"},{"packageId":"c-kmipkit",'
                '"kind":"field","exportName":"kmipkit_options_t.secret",'
                '"declarationState":"blocked"}],'
                '"approvedExclusions":[]}',
                encoding="utf-8",
            )
            header_path = root / "bindings" / "c" / "include" / "kmipkit.h"
            header_path.parent.mkdir(parents=True)
            header_path.write_text("int32_t kmipkit_probe(void);\n", encoding="utf-8")
            binary = root / "kmipkit_ffi.dll"
            binary.write_bytes(b"synthetic native library")

            with patch.object(
                AUDIT,
                "read_native_exports",
                return_value=["kmipkit_probe", "kmipkit_blocked_probe"],
            ) as read_exports, patch.object(
                AUDIT, "build_c_binary_exports", side_effect=AssertionError("must reuse the existing binary")
            ):
                result = AUDIT.run(root, c_only=True, c_binary=binary)

        read_exports.assert_called_once_with(binary)
        self.assertEqual(["c-kmipkit"], [item["packageId"] for item in result["packages"]])
        self.assertTrue(result["packages"][0]["matchesManifest"])

    def test_published_c_abi_exports_only_kmipkit_symbols(self):
        exports = AUDIT.build_c_binary_exports(Path(__file__).resolve().parents[3])
        non_kmipkit_exports = [name for name in exports if not name.startswith("kmipkit_")]
        self.assertEqual(non_kmipkit_exports, [])
        self.assertTrue(any(name.startswith("kmipkit_") for name in exports))

    def test_c_build_export_reader_reads_library_before_temporary_cleanup(self):
        build_exports = getattr(AUDIT, "build_c_binary_exports", None)
        self.assertIsNotNone(build_exports)
        with tempfile.TemporaryDirectory() as directory:
            def fake_run(arguments, **_kwargs):
                target_root = Path(arguments[arguments.index("--target-dir") + 1])
                suffix = ".dll" if AUDIT.os.name == "nt" else ".dylib" if AUDIT.sys.platform == "darwin" else ".so"
                binary = target_root / "debug" / f"kmipkit_ffi{suffix}"
                binary.parent.mkdir(parents=True)
                binary.write_bytes(b"synthetic PE library")
                return subprocess.CompletedProcess(arguments, 0, "", "")

            def read_existing_binary(binary):
                self.assertTrue(binary.is_file())
                return ["kmipkit_probe"]

            with patch.object(AUDIT.subprocess, "run", side_effect=fake_run), patch.object(
                AUDIT, "read_native_exports", side_effect=read_existing_binary
            ):
                exports = build_exports(Path(directory))

        self.assertEqual(exports, ["kmipkit_probe"])

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

    def test_c_header_inventory_omits_the_preprocessor_include_guard(self):
        header = """
            #ifndef KMIPKIT_H
            #define KMIPKIT_H
            #define KMIPKIT_ERROR_INVALID_INPUT ((int32_t)4)
            int32_t kmipkit_client_create(void);
            #endif
        """
        result = AUDIT.inventory_c_header(header)
        names = {row["exportName"] for row in result["declarations"]}
        self.assertNotIn("KMIPKIT_H", names)
        self.assertIn("KMIPKIT_ERROR_INVALID_INPUT", names)
        self.assertTrue(result["complete"])

    def test_c_header_inventory_keeps_a_conditional_public_version_macro(self):
        result = AUDIT.inventory_c_header(
            "#ifndef KMIPKIT_ABI_VERSION\n#define KMIPKIT_ABI_VERSION 1\n#endif\n"
            "#define KMIPKIT_FEATURE_LEVEL 1\n"
        )
        names = {row["exportName"] for row in result["declarations"]}
        self.assertIn("KMIPKIT_ABI_VERSION", names)
        self.assertIn("KMIPKIT_FEATURE_LEVEL", names)

    def test_c_header_inventory_classifies_ttlv_decode_error_macros_as_constants(self):
        result = AUDIT.inventory_c_header(
            "#define KMIPKIT_TTLV_DECODE_ERROR_KIND_TRUNCATED_HEADER UINT32_C(2)\n"
        )
        declarations = {
            row["exportName"]: row["kind"] for row in result["declarations"]
        }
        self.assertEqual(
            declarations["KMIPKIT_TTLV_DECODE_ERROR_KIND_TRUNCATED_HEADER"],
            "constant",
        )
        self.assertTrue(result["complete"])

    def test_c_header_inventory_accounts_for_enum_types_and_values(self):
        result = AUDIT.inventory_c_header("typedef enum { KMIPKIT_A = 0, KMIPKIT_B = 1 } kmipkit_mode_t;\n")
        actual = {(row["kind"], row["exportName"]) for row in result["declarations"]}
        self.assertEqual(actual, {
            ("type", "kmipkit_mode_t"),
            ("constant", "KMIPKIT_A"),
            ("constant", "KMIPKIT_B"),
        })
        self.assertTrue(result["complete"])

    def test_c_header_inventory_accounts_for_tagged_structs_and_union_fields(self):
        header = "typedef struct Public { int field; } kmipkit_options_t;\ntypedef union { int integer; } kmipkit_value_t;\n"
        result = AUDIT.inventory_c_header(header)
        actual = {(row["kind"], row["exportName"]) for row in result["declarations"]}
        self.assertEqual(actual, {
            ("type", "kmipkit_options_t"),
            ("field", "kmipkit_options_t.field"),
            ("type", "kmipkit_value_t"),
            ("field", "kmipkit_value_t.integer"),
        })
        self.assertTrue(result["complete"])

    def test_c_header_inventory_accounts_for_forward_declared_tagged_struct_fields(self):
        header = (
            "typedef struct kmipkit_transport_config_v1_t "
            "kmipkit_transport_config_v1_t;\n"
            "struct kmipkit_transport_config_v1_t {\n"
            "    uint32_t struct_size;\n"
            "    uint16_t endpoint_port;\n"
            "};\n"
        )
        result = AUDIT.inventory_c_header(header)
        actual = {(row["kind"], row["exportName"]) for row in result["declarations"]}
        self.assertEqual(actual, {
            ("type", "kmipkit_transport_config_v1_t"),
            ("field", "kmipkit_transport_config_v1_t.struct_size"),
            ("field", "kmipkit_transport_config_v1_t.endpoint_port"),
        })
        self.assertTrue(result["complete"])

    def test_c_header_inventory_accounts_for_multiple_fields_in_one_declaration(self):
        result = AUDIT.inventory_c_header("typedef struct { int left, right; } kmipkit_pair_t;\n")
        actual = {(row["kind"], row["exportName"]) for row in result["declarations"]}
        self.assertEqual(actual, {
            ("type", "kmipkit_pair_t"),
            ("field", "kmipkit_pair_t.left"),
            ("field", "kmipkit_pair_t.right"),
        })
        self.assertTrue(result["complete"])

    def test_c_binary_export_audit_reconciles_header_and_dll_symbols(self):
        header = "int32_t kmipkit_client_create(void);\n"
        result = AUDIT.audit_c_surface(header, ["kmipkit_client_create", "kmipkit_client_destroy"])
        names = {row["exportName"] for row in result["declarations"] if row["kind"] == "c-symbol"}
        self.assertEqual(names, {"kmipkit_client_create", "kmipkit_client_destroy"})
        self.assertEqual({issue["reason"] for issue in result["issues"]},
                         {"exported-symbol-missing-from-header"})

    def test_blocked_c_export_can_remain_binary_only_without_an_issue(self):
        result = AUDIT.audit_c_surface(
            "int32_t kmipkit_client_create(void);\n",
            ["kmipkit_client_create", "kmipkit_internal_helper"],
            blocked_exports={"kmipkit_internal_helper"},
        )

        self.assertTrue(result["complete"])
        self.assertEqual(result["issues"], [])
        symbols = {
            row["exportName"]: row
            for row in result["declarations"]
            if row["kind"] == "c-symbol"
        }
        self.assertEqual(set(symbols), {"kmipkit_client_create", "kmipkit_internal_helper"})
        self.assertEqual(symbols["kmipkit_internal_helper"]["location"], "binary")

    def test_c_binary_audit_honors_only_exact_approved_dependency_exclusions(self):
        exclusions = [{
            "packageId": "c-kmipkit",
            "exportName": "aws_lc_0_45_0_jent_version",
        }]
        result = AUDIT.audit_c_surface(
            "int32_t kmipkit_client_create(void);\n",
            ["kmipkit_client_create", "aws_lc_0_45_0_jent_version"],
            approved_exclusions=exclusions,
        )
        self.assertTrue(result["complete"])
        self.assertEqual(result["issues"], [])
        names = {row["exportName"] for row in result["declarations"]}
        self.assertEqual(names, {"kmipkit_client_create", "aws_lc_0_45_0_jent_version"})

    def test_c_binary_audit_still_reports_unapproved_transitive_exports(self):
        exclusions = [{
            "packageId": "c-kmipkit",
            "exportName": "aws_lc_0_45_0_jent_version",
        }]
        result = AUDIT.audit_c_surface(
            "int32_t kmipkit_client_create(void);\n",
            ["kmipkit_client_create", "unreviewed_vendor_symbol"],
            approved_exclusions=exclusions,
        )
        self.assertEqual({issue["exportName"] for issue in result["issues"]},
                         {"unreviewed_vendor_symbol"})
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
        ordinal_only = bytearray(image)
        struct.pack_into("<I", ordinal_only, export + 20, 2)
        with self.assertRaisesRegex(ValueError, "ordinal-only"):
            AUDIT.read_pe_exports(bytes(ordinal_only))

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

    def test_python_inventory_includes_package_constructor_inherited_from_private_base(self):
        sources = {
            "kmipkit/_handles.py": "class NativeHandle:\n    def __init__(self, handle): ...\n",
            "kmipkit/client.py": textwrap.dedent("""
                from ._handles import NativeHandle
                __all__ = ["Client"]
                class Client(NativeHandle):
                    pass
            """),
        }
        result = AUDIT.inventory_python_sources(sources)
        declarations = {row["exportName"]: row for row in result["declarations"]}
        constructor = declarations["kmipkit.client.Client.__init__"]
        self.assertEqual(constructor["kind"], "python-export")
        self.assertEqual(constructor["location"], "inherited:kmipkit._handles.NativeHandle:2")
        self.assertTrue(result["complete"])

    def test_python_inventory_excludes_internal_lifecycle_hooks_but_keeps_public_protocol(self):
        sources = {
            "kmipkit/_handles.py": textwrap.dedent("""
                class NativeHandle:
                    def __init__(self, handle): ...
                    def __del__(self): ...
                    def close(self): ...
                    def __enter__(self): ...
                    def __exit__(self, *args): ...
            """),
            "kmipkit/client.py": textwrap.dedent("""
                from dataclasses import dataclass
                from ._handles import NativeHandle
                __all__ = ["ClientConfig"]

                @dataclass
                class ClientConfig(NativeHandle):
                    endpoint: str

                    def __post_init__(self): ...
            """),
        }
        result = AUDIT.inventory_python_sources(sources)
        names = {row["exportName"] for row in result["declarations"]}
        public_type = "kmipkit.client.ClientConfig"
        self.assertTrue({
            f"{public_type}.__init__",
            f"{public_type}.close",
            f"{public_type}.__enter__",
            f"{public_type}.__exit__",
        }.issubset(names))
        self.assertNotIn(f"{public_type}.__post_init__", names)
        self.assertNotIn(f"{public_type}.__del__", names)
        self.assertTrue(result["complete"])

    def test_python_inventory_includes_public_instance_fields_assigned_in_init(self):
        source = textwrap.dedent("""
            class BatchResult:
                def __init__(self, items):
                    self.items = items

            class ClientExecutionError(Exception):
                def __init__(self, status):
                    self.status = status
        """)
        result = AUDIT.inventory_python_sources({"kmipkit/client.py": source})
        observed = {row["exportName"]: row["kind"] for row in result["declarations"]}
        self.assertEqual(observed["kmipkit.client.BatchResult.items"], "field")
        self.assertEqual(observed["kmipkit.client.ClientExecutionError.status"], "field")
        self.assertTrue(result["complete"])

    def test_python_inventory_deduplicates_annotated_fields_assigned_in_init(self):
        source = textwrap.dedent("""
            class Pkcs11Response:
                response_ttlv: object
                interface_ttlv: object | None

                def __init__(self, response_ttlv, interface_ttlv=None):
                    self.response_ttlv = response_ttlv
                    self.interface_ttlv = interface_ttlv
        """)
        result = AUDIT.inventory_python_sources({"kmipkit/client.py": source})
        fields = [
            row for row in result["declarations"]
            if row["exportName"].startswith("kmipkit.client.Pkcs11Response.")
            and row["kind"] == "field"
        ]

        self.assertEqual(
            {"kmipkit.client.Pkcs11Response.response_ttlv", "kmipkit.client.Pkcs11Response.interface_ttlv"},
            {row["exportName"] for row in fields},
        )
        self.assertEqual(2, len(fields))
        self.assertTrue(result["complete"], result["issues"])

    def test_python_inventory_includes_public_dataclass_generated_members(self):
        sources = {
            "kmipkit/client.py": textwrap.dedent("""
                from dataclasses import dataclass
                __all__ = ["BatchItemResult", "CustomValue"]

                @dataclass(frozen=True)
                class BatchItemResult:
                    item_id: bytes

                @dataclass(init=False, repr=False, eq=False)
                class CustomValue:
                    value: int
            """),
        }
        result = AUDIT.inventory_python_sources(sources)
        observed = {(row["kind"], row["exportName"]) for row in result["declarations"]}
        self.assertTrue({
            ("python-export", "kmipkit.client.BatchItemResult.__init__"),
            ("python-export", "kmipkit.client.BatchItemResult.__repr__"),
            ("python-export", "kmipkit.client.BatchItemResult.__eq__"),
            ("python-export", "kmipkit.client.BatchItemResult.__hash__"),
        }.issubset(observed))
        custom = "kmipkit.client.CustomValue"
        self.assertTrue({
            ("field", f"{custom}.__dataclass_fields__"),
            ("field", f"{custom}.__dataclass_params__"),
            ("field", f"{custom}.__match_args__"),
        }.issubset(observed))
        self.assertFalse(any(
            name in {f"{custom}.__init__", f"{custom}.__repr__", f"{custom}.__eq__",
                     f"{custom}.__hash__", f"{custom}.__slots__", f"{custom}.__setattr__",
                     f"{custom}.__delattr__"}
            for _, name in observed
        ))
        self.assertTrue(result["complete"])

    def test_python_inventory_audits_dataclass_generated_metadata_and_constructor_signature(self):
        sources = {
            "kmipkit/models.py": textwrap.dedent("""
                import dataclasses as dc
                __all__ = ["FrozenRecord", "ManualRecord"]

                @dc.dataclass(frozen=True, slots=True)
                class FrozenRecord:
                    required: int
                    label: str = "x"

                @dc.dataclass(init=False, match_args=False, slots=False,
                              frozen=False, repr=False, eq=False)
                class ManualRecord:
                    value: int

                @dc.dataclass
                class MutableRecord:
                    value: int

                @dc.dataclass
                class ExplicitHashRecord:
                    value: int

                    def __hash__(self):
                        return 1

                @dc.dataclass
                class SuppressedHashRecord:
                    value: int
                    __hash__ = None
            """),
        }
        result = AUDIT.inventory_python_sources(sources)
        declarations = {row["exportName"]: row for row in result["declarations"]}
        frozen = "kmipkit.models.FrozenRecord"
        self.assertTrue({
            f"{frozen}.__init__",
            f"{frozen}.__repr__",
            f"{frozen}.__eq__",
            f"{frozen}.__hash__",
            f"{frozen}.__setattr__",
            f"{frozen}.__delattr__",
            f"{frozen}.__match_args__",
            f"{frozen}.__slots__",
            f"{frozen}.__dataclass_fields__",
            f"{frozen}.__dataclass_params__",
        }.issubset(declarations))
        self.assertEqual(
            declarations[f"{frozen}.__init__"].get("signature"),
            "(required: int, label: str = 'x') -> None",
        )
        manual = "kmipkit.models.ManualRecord"
        self.assertIn(f"{manual}.__dataclass_fields__", declarations)
        self.assertIn(f"{manual}.__dataclass_params__", declarations)
        self.assertNotIn(f"{manual}.__init__", declarations)
        self.assertNotIn(f"{manual}.__match_args__", declarations)
        self.assertNotIn(f"{manual}.__slots__", declarations)
        self.assertEqual(declarations["kmipkit.models.MutableRecord.__hash__"]["kind"], "field")
        self.assertEqual(declarations["kmipkit.models.ExplicitHashRecord.__hash__"]["kind"], "python-export")
        self.assertEqual(declarations["kmipkit.models.SuppressedHashRecord.__hash__"]["kind"], "field")
        self.assertEqual(
            sum(name == "kmipkit.models.SuppressedHashRecord.__hash__" for name in declarations),
            1,
        )
        self.assertTrue(result["complete"])

    def test_python_inventory_fails_closed_for_dynamic_dataclass_options(self):
        source = textwrap.dedent("""
            from dataclasses import dataclass
            __all__ = ["DynamicRecord"]

            def slots_enabled():
                return True

            @dataclass(slots=slots_enabled())
            class DynamicRecord:
                value: int
        """)
        result = AUDIT.inventory_python_sources({"kmipkit/models.py": source})
        self.assertFalse(result["complete"])
        self.assertIn(
            {"packageId": "python-kmipkit", "exportName": "kmipkit.models.DynamicRecord",
             "reason": "dynamic-dataclass-options"},
            result["issues"],
        )

    def test_python_inventory_resolves_private_module_values_and_property_exports(self):
        sources = {
            "kmipkit/_ffi/__init__.py": "ffi = object()\nlib = object()\n",
            "kmipkit/api.py": "from ._ffi import lib\n__all__ = []\n",
            "kmipkit/_handles.py": "class Base:\n    @property\n    def closed(self) -> bool: ...\n",
            "kmipkit/handles.py": "from ._handles import Base\nclass Handle(Base):\n    pass\n",
        }
        result = AUDIT.inventory_python_sources(sources)
        observed = {row["exportName"]: row["kind"] for row in result["declarations"]}
        self.assertEqual(observed["kmipkit.handles.Handle.closed"], "python-export")
        self.assertFalse(any(issue["reason"] == "unresolved-import-target" for issue in result["issues"]))
        self.assertTrue(result["complete"])

    def test_python_inventory_marks_explicit_external_imports_as_public_exports(self):
        source = "from datetime import datetime\n__all__ = ['datetime']\n"
        result = AUDIT.inventory_python_sources({"kmipkit/api.py": source})
        names = {row["exportName"] for row in result["declarations"]}
        self.assertIn("kmipkit.api.datetime", names)
        self.assertFalse(result["complete"])
        self.assertIn("unresolved-import-target", {issue["reason"] for issue in result["issues"]})

    def test_python_inventory_fails_closed_for_augmented_all_exports(self):
        source = "from datetime import datetime\n__all__ = []\n__all__ += ['datetime']\n"
        result = AUDIT.inventory_python_sources({"kmipkit/api.py": source})
        names = {row["exportName"] for row in result["declarations"]}
        self.assertIn("kmipkit.api.datetime", names)
        self.assertFalse(result["complete"])
        self.assertIn("unresolved-import-target", {issue["reason"] for issue in result["issues"]})

    def test_python_inventory_resolves_reexports_through_multiple_modules(self):
        sources = {
            "kmipkit/core.py": "class Client:\n    pass\n",
            "kmipkit/reexports.py": "from .core import Client\n__all__ = ['Client']\n",
            "kmipkit/__init__.py": "from .reexports import Client\n__all__ = ['Client']\n",
        }
        result = AUDIT.inventory_python_sources(sources)
        names = {row["exportName"] for row in result["declarations"]}
        self.assertIn("kmipkit.Client", names)
        self.assertTrue(result["complete"])

    def test_python_inventory_ignores_future_imports_and_same_path_module_aliases(self):
        sources = {
            "kmipkit/__init__.py": "from . import errors\n__all__ = ['errors']\n",
            "kmipkit/errors.py": "from __future__ import annotations\nclass Error: pass\n",
        }
        result = AUDIT.inventory_python_sources(sources)
        names = [row["exportName"] for row in result["declarations"]]
        self.assertEqual(names.count("kmipkit.errors"), 1)
        self.assertNotIn("kmipkit.errors.annotations", names)
        self.assertTrue(result["complete"])

    def test_python_inventory_omits_private_external_import_aliases(self):
        source = "from enum import IntEnum as _IntEnum\nclass Choice(_IntEnum): pass\n"
        result = AUDIT.inventory_python_sources({"kmipkit/errors.py": source})
        names = {row["exportName"] for row in result["declarations"]}
        self.assertIn("kmipkit.errors.Choice", names)
        self.assertNotIn("kmipkit.errors._IntEnum", names)
        self.assertTrue(result["complete"])

    def test_internal_bridge_and_dataclass_artifacts_use_exact_exclusions(self):
        repo_root = Path(__file__).resolve().parents[3]
        manifest_path = repo_root / "specification/api/public-api.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["approvedExclusions"]
        }

        native_members = {
            "CLIENT", "CLIENT_BATCH", "CLIENT_BATCH_ITEM_RESPONSE",
            "CLIENT_BATCH_RESPONSE", "CODEC_LIMITS", "TRANSPORT_CONFIGURATION",
            "SPLIT_KEY_REFERENCE", "USAGE_ALLOCATION_CONTEXT",
            "clientBatchAddErrorContinuationValue", "clientBatchAddItem",
            "clientExecuteWithOptions",
            "clientBatchCreate", "clientBatchItemResponseMetadata",
            "clientBatchItemResponsePendingCorrelationValue",
            "clientBatchItemResponseDiscoverVersionsSupportedVersions",
            "clientBatchItemResponseUniqueBatchItemId",
            "REVOCATION_REASON", "REVOKE_REQUEST",
            "clientBatchItemReprovision", "clientBatchItemRevoke",
            "clientBatchItemSetConstraints", "clientBatchItemSetDefaults",
            "clientBatchItemValidate", "revocationReasonCreate", "revokeRequestCreate",
            "clientBatchItemRekey", "clientBatchItemRekeyKeyPair",
            "clientBatchItemRngRetrieve", "clientBatchItemRngSeed",
            "clientBatchItemCreate", "clientBatchItemCreateKeyPair",
            "clientBatchItemCreateSplitKey", "clientBatchItemCertify",
            "splitKeyReferenceCreate", "clientBatchItemJoinSplitKey",
            "clientBatchItemDelegatedLogin", "clientBatchItemDeriveKey",
            "clientBatchItemGetConstraints", "clientBatchItemImport",
            "clientBatchItemCheck",
            "clientBatchItemResponseCreate",
            "clientBatchItemResponseCreateKeyPair",
            "clientBatchItemResponseCreateSplitKey",
            "clientBatchItemResponseJoinSplitKey",
            "clientBatchItemResponseDelegatedLogin",
            "clientBatchItemResponseDeriveKey",
            "clientBatchItemResponseGetConstraints",
            "clientBatchItemResponseImport",
            "clientBatchItemResponseCertify",
            "clientBatchItemResponseCheck",
            "clientBatchItemResponseMetadataV2",
            "clientBatchItemResponseRekey", "clientBatchItemResponseRekeyKeyPair",
            "clientBatchItemResponseRngRetrieve", "clientBatchItemResponseRngSeed",
            "clientBatchItemResponseReprovision", "clientBatchItemResponseReprovisionTtlv",
            "clientBatchItemResponseRegister", "clientBatchItemResponseRevoke",
            "clientBatchItemResponseSetConstraints", "clientBatchItemResponseSetDefaults",
            "clientBatchItemResponseValidate",
            "clientBatchItemPkcs11", "clientBatchItemRecertify",
            "clientBatchItemHash", "clientBatchItemMac",
            "clientBatchItemMacVerify", "clientBatchItemSign",
            "clientBatchItemSignatureVerify", "clientBatchItemResponseHash",
            "clientBatchItemResponseMac", "clientBatchItemResponseMacVerify",
            "clientBatchItemResponseSign",
            "clientBatchItemResponseSignatureVerify",
            "clientBatchItemGet", "clientBatchItemLocate",
            "clientBatchItemQueryWithLimits",
            "clientBatchItemPing", "clientBatchItemQuery",
            "clientBatchItemEncrypt", "clientBatchItemDecrypt",
            "clientBatchItemLogin", "clientBatchItemObtainLease",
            "clientBatchItemLogout", "clientBatchItemResponseLogout",
            "clientExecuteGuardedProtectionItem",
            "clientExecuteGuardedProtectionItemWithAsynchronousIndicator",
            "usageAllocationContextCreate", "usageAllocationContextUsageLimitsUnit",
            "usageAllocationContextAllocatedUnits", "usageAllocationContextAvailableUnits",
            "usageAllocationContextReservedUnits", "usageAllocationContextUniqueIdentifier",
            "clientBatchItemResponsePing", "clientBatchItemResponseQueryTtlv",
            "clientBatchItemResponsePkcs11", "clientBatchItemResponseRecertify",
            "clientBatchItemResponseEncrypt", "clientBatchItemResponseDecrypt",
            "clientBatchItemResponseLogin", "clientBatchItemResponseObtainLease",
            "clientBatchItemResponseGet", "clientBatchItemResponseLocate",
            "clientBatchItemGetUsageAllocation",
            "clientBatchItemExport",
            "clientBatchItemResponseGetUsageAllocation",
            "clientBatchItemResponseExport",
            "clientBatchItemInterop", "clientBatchItemResponseInterop",
            "clientBatchItemLog", "clientBatchItemResponseLog",
            "clientBatchItemWithUniqueBatchItemId", "clientBatchResponseItemAt",
            "clientBatchResponseItemCount", "clientBatchSetAsynchronousIndicator",
            "clientBatchSetClientCorrelationValue", "clientBatchSetOrderOption",
            "clientBatchSetRequestTimestamp", "clientCreate", "clientExecute",
            "transportConfigCreate",
        }
        expected_native = {
            ("java-kmipkit", f"org.kmipkit.NativeExtensionRegistry.{member}")
            for member in native_members
        }
        actual_native = {
            key for key in exclusions
            if key[0] == "java-kmipkit"
            and key[1].startswith("org.kmipkit.NativeExtensionRegistry.")
        }
        self.assertEqual(expected_native, actual_native)
        self.assertNotIn(
            ("java-kmipkit", "org.kmipkit.NativeExtensionRegistry"), exclusions
        )
        for key in expected_native:
            exclusion = exclusions[key]
            self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
            self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
            self.assertIn("ffi-and-bindings.md", exclusion["evidence"])

        get_usage_request = exclusions[(
            "java-kmipkit",
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemGetUsageAllocation",
        )]["evidence"]
        self.assertIn(
            "identifier kind, optional text or numeric identifier, and usage limits count",
            get_usage_request,
        )
        self.assertNotIn("only transfers opaque handles", get_usage_request)

        get_usage_response = exclusions[(
            "java-kmipkit",
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseGetUsageAllocation",
        )]["evidence"]
        self.assertIn(
            "Object[] tuple of presence fields, typed identifier data, and an owned response-TTLV handle",
            get_usage_response,
        )
        self.assertIn("parseGetUsageAllocationResponse", get_usage_response)
        self.assertNotIn("only transfers opaque handles", get_usage_response)

        java_boilerplate = {
            "org.kmipkit.extensions.BatchOutcomeKind.equals",
            "org.kmipkit.extensions.BatchOutcomeKind.hashCode",
            "org.kmipkit.extensions.BatchOutcomeKind.toString",
            "org.kmipkit.extensions.TransportConfiguration.Timeout.equals",
            "org.kmipkit.extensions.TransportConfiguration.Timeout.hashCode",
            "org.kmipkit.extensions.TransportConfiguration.Timeout.toString",
            "org.kmipkit.extensions.TransportConfiguration.Encoding.valueOf",
            "org.kmipkit.extensions.TransportConfiguration.Encoding.values",
            "org.kmipkit.extensions.ClientBatchItemResult.HashResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.MacResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.MacVerifyResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.SignResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.SignatureVerifyResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.GetUsageAllocationResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.JoinSplitKeyResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.DelegatedLoginResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.DeriveKeyResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.GetConstraintsResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.GetResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.ImportResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.LogResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.ReProvisionResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.RegisterResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.RevokeResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.SetConstraintsResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.SetDefaultsResponse.toString",
            "org.kmipkit.extensions.ClientBatchItemResult.ValidateResponse.toString",
            "org.kmipkit.extensions.RevocationReason.toString",
            "org.kmipkit.extensions.UsageAllocationContext.toString",
        }
        for path in java_boilerplate:
            self.assertIn(("java-kmipkit", path), exclusions)

        for path in java_boilerplate:
            if path.endswith("Response.toString") or path.endswith(
                "UsageAllocationContext.toString"
            ):
                self.assertIn("redacted", exclusions[("java-kmipkit", path)]["reason"])

        dataclass_names = (
            "kmipkit.client.BatchItemResult",
            "kmipkit.client.Endpoint",
            "kmipkit.client.TimeoutLimit",
            "kmipkit.client.TransportConfig",
            "kmipkit.client.TransportInput",
            "kmipkit.client.TrustSource",
            "kmipkit.extensions.ExtensionRegistryLimits",
            "kmipkit.metadata.Version",
            "kmipkit.client.LogResponse",
        )
        dataclass_artifacts = (
            "__dataclass_fields__", "__dataclass_params__", "__delattr__",
            "__eq__", "__hash__", "__match_args__", "__repr__", "__setattr__",
            "__slots__",
        )
        expected_python = {
            ("python-kmipkit", f"{class_name}.{member}")
            for class_name in dataclass_names
            for member in dataclass_artifacts
        }
        for key in expected_python:
            self.assertIn(key, exclusions)
        approved_constructor_exclusions = {
            "kmipkit.client.BatchItemResult": (
                "read-only ClientBatchResponse traversal",
                "KMIPKIT-0043-FR-012",
            ),
            "kmipkit.metadata.Version": (
                "runtime metadata query remain mapped",
                "KMIPKIT-0043-FR-021",
            ),
            "kmipkit.client.LogResponse": (
                "typed Log response and close lifecycle are inventoried",
                "KMIPKIT-0043-FR-021",
            ),
        }
        for class_name in dataclass_names:
            constructor = ("python-kmipkit", f"{class_name}.__init__")
            if class_name in approved_constructor_exclusions:
                exclusion = exclusions[constructor]
                evidence_phrase, requirement = approved_constructor_exclusions[class_name]
                self.assertIn(evidence_phrase, exclusion["evidence"])
                self.assertIn(requirement, exclusion["evidence"])
            else:
                self.assertNotIn(constructor, exclusions)
        for class_name in dataclass_names:
            repr_exclusion = exclusions[("python-kmipkit", f"{class_name}.__repr__")]
            expected_repr_test = (
                "test_log_operation.py"
                if class_name == "kmipkit.client.LogResponse"
                else "test_secret_bearing_client_records_redact_repr"
            )
            self.assertIn(expected_repr_test, repr_exclusion["evidence"])

        self.assertIn(("python-kmipkit", "kmipkit.client"), exclusions)

        python_response_ttlv = next(
            row for row in manifest["publicDeclarations"]
            if row["exportName"] == "kmipkit.errors.ClientExecutionError.response_ttlv"
        )
        self.assertEqual(python_response_ttlv["kind"], "field")

        local_conveniences = {
            ("java-kmipkit", "org.kmipkit.extensions.ClientBatch.itemCount"),
            ("python-kmipkit", "kmipkit.client.Client.closed"),
            ("python-kmipkit", "kmipkit.client.Batch.closed"),
            ("python-kmipkit", "kmipkit.client.BatchResult.closed"),
        }
        for key in local_conveniences:
            self.assertTrue(key in exclusions, f"missing exact exclusion {key}")
            self.assertIn("language-only convenience", exclusions[key]["reason"])
            self.assertIn("KMIPKIT-0043-FR-021", exclusions[key]["evidence"])
    def test_checked_in_binding_export_report_reconciles_every_package(self):
        repo_root = Path(__file__).resolve().parents[3]
        report_dir = repo_root / "specs/043-public-api-bindings/reports"
        report_path = max(report_dir.glob("binding-export-report-*.json"))
        report = json.loads(report_path.read_text(encoding="utf-8"))
        self.assertEqual(
            {package["packageId"] for package in report["packages"]},
            {"c-kmipkit", "java-kmipkit", "python-kmipkit"},
        )
        for package in report["packages"]:
            with self.subTest(package=package["packageId"]):
                self.assertTrue(package["complete"])
                self.assertTrue(package["matchesManifest"])
                self.assertEqual(package["missingFromManifest"], [])
                self.assertEqual(package["notObserved"], [])
                self.assertEqual(package["kindMismatches"], [])
                self.assertEqual(package["issues"], [])

    def test_python_inventory_collapses_explicit_overload_group_to_one_export(self):
        sources = {
            "kmipkit/api.py": textwrap.dedent("""
                from typing import overload
                __all__ = ['convert']
                @overload
                def convert(value: int) -> int: ...
                @overload
                def convert(value: str) -> str: ...
                def convert(value):
                    return value
            """),
        }
        result = AUDIT.inventory_python_sources(sources)
        exports = [row for row in result["declarations"]
                   if row["exportName"] == "kmipkit.api.convert"]
        self.assertTrue(result["complete"])
        self.assertEqual(len(exports), 1)

    def test_python_inventory_resolves_inherited_members_through_module_attribute(self):
        sources = {
            "kmipkit/_handles.py": "class Base:\n    def close(self) -> None: ...\n",
            "kmipkit/handle.py": "from . import _handles as _base\n__all__ = ['Handle']\nclass Handle(_base.Base):\n    pass\n",
        }
        result = AUDIT.inventory_python_sources(sources)
        names = {row["exportName"] for row in result["declarations"]}
        self.assertIn("kmipkit.handle.Handle.close", names)
        self.assertTrue(result["complete"])

    def test_java_overloads_that_share_a_manifest_path_keep_exact_signatures(self):
        output = """
            public class org.example.Client {
              public void send(java.lang.String);
              public void send(byte[]);
            }
        """
        result = AUDIT.inventory_javap_outputs({"Client.class": output})
        self.assertTrue(result["complete"])
        declaration = next(
            row for row in result["declarations"]
            if row["exportName"] == "org.example.Client.send"
        )
        self.assertEqual(
            declaration["signatures"],
            ["public void send(byte[])", "public void send(java.lang.String)"]
        )

    def test_java_manifest_must_declare_all_overload_signatures(self):
        output = """
            public class org.example.Client {
              public void send(java.lang.String);
              public void send(byte[]);
            }
        """
        report = AUDIT.inventory_javap_outputs({"Client.class": output})
        declaration = {
            "packageId": "java-kmipkit",
            "exportName": "org.example.Client.send",
            "kind": "java-member",
            "javaSignatures": ["public void send(java.lang.String)", "public void send(byte[])"]
        }
        class_declaration = {
            "packageId": "java-kmipkit",
            "exportName": "org.example.Client",
            "kind": "type",
        }
        manifest = {"publicDeclarations": [class_declaration, declaration]}
        exact = AUDIT.compare(report, manifest, "java-kmipkit")
        self.assertTrue(exact["matchesManifest"])

        declaration.pop("javaSignatures")
        missing = AUDIT.compare(report, manifest, "java-kmipkit")
        self.assertFalse(missing["matchesManifest"])
        self.assertIn("java-overload-signatures-not-declared", {row["reason"] for row in missing["issues"]})

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

    def test_comparison_rejects_duplicate_approved_exclusion_paths(self):
        exclusion = {"packageId": "java-kmipkit", "exportName": "org.example.Internal"}
        manifest = {
            "publicDeclarations": [],
            "approvedExclusions": [exclusion, dict(exclusion, reason="duplicate")],
        }
        report = {
            "complete": True,
            "declarations": [],
            "issues": [],
        }

        result = AUDIT.compare(report, manifest, "java-kmipkit")

        self.assertFalse(result["matchesManifest"])
        self.assertIn(
            "duplicate-manifest-approved-exclusion",
            {issue["reason"] for issue in result["issues"]},
        )

    def test_comparison_rejects_declaration_exclusion_overlap(self):
        manifest = {
            "publicDeclarations": [{
                "packageId": "java-kmipkit",
                "exportName": "org.example.Internal",
                "kind": "type",
            }],
            "approvedExclusions": [{
                "packageId": "java-kmipkit",
                "exportName": "org.example.Internal",
            }],
        }
        report = {
            "complete": True,
            "declarations": [{
                "packageId": "java-kmipkit",
                "exportName": "org.example.Internal",
                "kind": "type",
            }],
            "issues": [],
        }

        result = AUDIT.compare(report, manifest, "java-kmipkit")

        self.assertFalse(result["matchesManifest"])
        self.assertIn(
            "manifest-export-also-excluded",
            {issue["reason"] for issue in result["issues"]},
        )

    def test_comparison_counts_exact_matches_separately_from_path_matches(self):
        manifest = {"publicDeclarations": [{
            "packageId": "java-kmipkit",
            "exportName": "org.example.Version.major",
            "kind": "java-member",
        }]}
        observed = {"complete": True, "declarations": [{
            "packageId": "java-kmipkit",
            "exportName": "org.example.Version.major",
            "kind": "java-type",
        }], "issues": []}
        result = AUDIT.compare(observed, manifest, "java-kmipkit")
        self.assertEqual(result["counts"]["pathMatches"], 1)
        self.assertEqual(result["counts"]["exactMatches"], 0)
        self.assertEqual(result["counts"]["kindMismatches"], 1)
        self.assertFalse(result["matchesManifest"])

    def test_python_function_and_reexport_kinds_are_semantically_equivalent(self):
        manifest = {"publicDeclarations": [{
            "packageId": "java-kmipkit",
            "exportName": "org.example.Version.major",
            "kind": "java-member",
        }]}
        observed = {"complete": True, "declarations": [{
            "packageId": "java-kmipkit",
            "exportName": "org.example.Version.major",
            "kind": "java-type",
        }], "issues": []}
        result = AUDIT.compare(observed, manifest, "java-kmipkit")
        self.assertEqual(result["counts"]["pathMatches"], 1)
        self.assertEqual(result["counts"]["exactMatches"], 0)
        self.assertEqual(result["counts"]["kindMismatches"], 1)
        self.assertFalse(result["matchesManifest"])


if __name__ == "__main__":
    unittest.main()
