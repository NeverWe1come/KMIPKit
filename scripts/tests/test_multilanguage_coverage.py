"""Contract tests for independent Rust, Java, Python, and JNI coverage scopes."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import tempfile
import unittest
import xml.etree.ElementTree as ET

from test_coverage_gate import GATE, llvm_document


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


class MultiLanguageCoverageTests(unittest.TestCase):
    def require_gate(self) -> None:
        self.assertIsNotNone(GATE, "coverage_gate.py must provide the multi-language coverage contract")

    def test_rust_protocol_and_ttlv_are_enforced_as_separate_scopes(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            ttlv = root / "crates/kmipkit-ttlv/src/lib.rs"
            protocol = root / "crates/kmipkit-protocol/src/lib.rs"
            ttlv.parent.mkdir(parents=True)
            protocol.parent.mkdir(parents=True)
            ttlv.write_text("\n".join("pub fn item() {}" for _ in range(19)), encoding="utf-8")
            protocol.write_text("pub fn item() {}\n", encoding="utf-8")
            report = {
                "crates/kmipkit-ttlv/src/lib.rs": {line: 1 for line in range(1, 20)},
                "crates/kmipkit-protocol/src/lib.rs": {1: 0},
            }
            reports = {platform: report for platform in ("ubuntu", "windows", "macos")}

            with self.assertRaisesRegex(GATE.CoverageDataError, "kmipkit-protocol coverage"):
                GATE._evaluate_coverage(root, reports, "")

    def test_jacoco_parser_maps_production_source_lines(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "bindings/java/src/main/java/org/kmipkit/extensions/Registry.java"
            source.parent.mkdir(parents=True)
            source.write_text("class Registry {}\n", encoding="utf-8")
            report = """<?xml version="1.0"?>
            <report name="KMIPKit"><package name="org/kmipkit/extensions">
              <sourcefile name="Registry.java"><line nr="1" mi="1" ci="0" mb="0" cb="0"/></sourcefile>
            </package></report>"""

            parsed = GATE.parse_jacoco_report(report, root)

        self.assertEqual({"bindings/java/src/main/java/org/kmipkit/extensions/Registry.java": {1: 0}}, parsed)

    def test_cobertura_parser_maps_python_sources_and_rejects_outside_sources(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "bindings/python/src/kmipkit/extensions.py"
            source.parent.mkdir(parents=True)
            source.write_text("def validate():\n    return True\n", encoding="utf-8")
            report = """<?xml version="1.0"?>
            <coverage line-rate="0.5"><sources><source>bindings/python/src</source></sources>
              <packages><package name="kmipkit"><classes><class filename="kmipkit/extensions.py">
                <lines><line number="1" hits="1"/><line number="2" hits="0"/></lines>
              </class></classes></package></packages></coverage>"""

            parsed = GATE.parse_cobertura_report(report, root)

        self.assertEqual({"bindings/python/src/kmipkit/extensions.py": {1: 1, 2: 0}}, parsed)

    def test_adapter_source_scan_includes_generated_package_code_but_not_consumers(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected_paths = (
                "bindings/java/src/main/java/org/kmipkit/NativeExtensionRegistry.java",
                "bindings/java/src/main/java/org/kmipkit/generated/ExtensionRegistryApi.java",
                "bindings/python/src/kmipkit/extensions.py",
                "bindings/python/src/kmipkit/_generated/extension_registry.py",
                "bindings/java/native/kmipkit_jni.cpp",
            )
            excluded_paths = (
                "bindings/java/src/test/java/org/kmipkit/RegistryTest.java",
                "bindings/java/examples/RegistryExample.java",
                "bindings/python/tests/test_registry.py",
                "bindings/python/examples/registry_example.py",
                "bindings/java/native/test/RegistryTest.cpp",
            )
            for relative in (*expected_paths, *excluded_paths):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("source\n", encoding="utf-8")

            sources = GATE.scan_adapter_sources(root)

        self.assertEqual(set(expected_paths), set().union(*sources.values()))

    def test_jni_llvm_export_parser_accepts_the_native_bridge_file(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "bindings/java/native/kmipkit_jni.cpp"
            source.parent.mkdir(parents=True)
            source.write_text("int bridge() { return 1; }\n", encoding="utf-8")
            document = llvm_document(
                [[1, 1, 1, 31, 1, 0, 0, 0]],
                filename="bindings/java/native/kmipkit_jni.cpp",
            )

            parsed = GATE.parse_llvm_export(document, root)

        self.assertEqual({"bindings/java/native/kmipkit_jni.cpp": {1: 1}}, parsed)

    def test_changed_production_diff_includes_java_python_and_jni_lines(self) -> None:
        self.require_gate()
        diff = """diff --git a/bindings/java/src/main/java/org/kmipkit/Registry.java b/bindings/java/src/main/java/org/kmipkit/Registry.java
--- a/bindings/java/src/main/java/org/kmipkit/Registry.java
+++ b/bindings/java/src/main/java/org/kmipkit/Registry.java
@@ -1 +1,2 @@
 class Registry {}
+    void newMethod() {}
diff --git a/bindings/python/src/kmipkit/extensions.py b/bindings/python/src/kmipkit/extensions.py
--- a/bindings/python/src/kmipkit/extensions.py
+++ b/bindings/python/src/kmipkit/extensions.py
@@ -1 +1,2 @@
 def validate(): pass
+    return True
diff --git a/bindings/java/native/kmipkit_jni.cpp b/bindings/java/native/kmipkit_jni.cpp
--- a/bindings/java/native/kmipkit_jni.cpp
+++ b/bindings/java/native/kmipkit_jni.cpp
@@ -1 +1,2 @@
 int bridge() { return 1; }
+    return 2;
"""

        changed = GATE.parse_added_production_lines(diff)

        self.assertEqual(
            {
                "bindings/java/src/main/java/org/kmipkit/Registry.java": {2},
                "bindings/python/src/kmipkit/extensions.py": {2},
                "bindings/java/native/kmipkit_jni.cpp": {2},
            },
            changed,
        )

    def test_coverage_collectors_are_pinned_and_exclude_consumers(self) -> None:
        pom = ET.parse(REPOSITORY_ROOT / "bindings/java/pom.xml").getroot()
        namespace = {"m": "http://maven.apache.org/POM/4.0.0"}
        jacoco = pom.find(".//m:plugin[m:artifactId='jacoco-maven-plugin']", namespace)
        self.assertIsNotNone(jacoco, "JaCoCo must be configured for handwritten Java package code")
        jacoco_version = jacoco.findtext("m:version", namespaces=namespace)
        self.assertRegex(jacoco_version or "", r"^\d+\.\d+\.\d+$", "JaCoCo version must be pinned")
        java_configuration = ET.tostring(jacoco, encoding="unicode")
        self.assertIn("0.85", java_configuration, "Java line coverage must enforce the 85% package gate")

        pyproject_path = REPOSITORY_ROOT / "bindings/python/pyproject.toml"
        tomllib_spec = importlib.util.find_spec("tomllib")
        self.assertIsNotNone(tomllib_spec, "the pinned Python toolchain must provide tomllib")
        import tomllib

        with pyproject_path.open("rb") as pyproject_file:
            pyproject = tomllib.load(pyproject_file)
        dependencies = pyproject["project"]["optional-dependencies"]["coverage"]
        self.assertTrue(any(dependency.startswith("coverage==") for dependency in dependencies))
        self.assertTrue(any(dependency.startswith("pytest-cov==") for dependency in dependencies))
        self.assertEqual("kmipkit", pyproject["tool"]["coverage"]["run"]["source"][0])
        self.assertEqual(85, pyproject["tool"]["coverage"]["report"]["fail_under"])
        self.assertFalse(pyproject["tool"]["coverage"]["run"].get("omit"))

        native_collector = REPOSITORY_ROOT / "scripts/collect_jni_coverage.sh"
        self.assertTrue(native_collector.is_file(), "the JNI bridge needs a Linux LLVM coverage collector")
        script = native_collector.read_text(encoding="utf-8")
        self.assertRegex(script, r"clang\+\+-[0-9]+")
        self.assertRegex(script, r"llvm-profdata-[0-9]+")
        self.assertRegex(script, r"llvm-cov-[0-9]+")
        self.assertIn("-fcoverage-mapping", script)
        self.assertIn("kmipkit_jni.cpp", script)


if __name__ == "__main__":
    unittest.main()
