"""Contract tests for deterministic non-Rust public declaration audits."""

from __future__ import annotations

import importlib.util
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
