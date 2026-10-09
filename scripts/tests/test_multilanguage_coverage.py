"""Contract tests for independent Rust, Java, Python, and JNI coverage scopes."""

from __future__ import annotations

from pathlib import Path
import re
import tempfile
import unittest
import xml.etree.ElementTree as ET

from test_coverage_gate import GATE, llvm_document


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


class MultiLanguageCoverageTests(unittest.TestCase):
    def require_gate(self) -> None:
        self.assertIsNotNone(GATE, "coverage_gate.py must provide the multi-language coverage contract")

    def test_jni_collector_ignores_hosted_toolcache_jdk_header(self) -> None:
        self.require_gate()
        script = (REPOSITORY_ROOT / "scripts/collect_jni_coverage.sh").read_text(encoding="utf-8")
        match = re.search(r"^readonly ignored_source_regex='([^']+)'$", script, re.MULTILINE)
        self.assertIsNotNone(match, "the JNI collector must declare its narrow source exclusions")
        assert match is not None
        ignored_source_regex = match.group(1)

        hosted_jdk_header = "/opt/hostedtoolcache/Java_Temurin-Hotspot_jdk/17.0.20-1/x64/include/jni.h"
        hosted_jdk_platform_header = "/opt/hostedtoolcache/Java_Temurin-Hotspot_jdk/17.0.20-1/x64/include/linux/jni_md.h"
        system_jdk_header = "/usr/lib/jvm/java-17-openjdk-amd64/include/jni.h"
        system_jdk_platform_header = "/usr/lib/jvm/java-17-openjdk-amd64/include/linux/jni_md.h"
        project_header = (REPOSITORY_ROOT / "bindings/java/native/include/jni.h").as_posix()
        project_platform_header = (REPOSITORY_ROOT / "bindings/java/native/include/linux/jni_md.h").as_posix()
        self.assertRegex(hosted_jdk_header, ignored_source_regex)
        self.assertRegex(hosted_jdk_platform_header, ignored_source_regex)
        self.assertRegex(system_jdk_header, ignored_source_regex)
        self.assertRegex(system_jdk_platform_header, ignored_source_regex)
        self.assertNotRegex(project_header, ignored_source_regex)
        self.assertNotRegex(project_platform_header, ignored_source_regex)

    def _write_gate_workspace(self, root: Path, line_count: int = 10):
        rust_source = root / "crates/kmipkit-ttlv/src/lib.rs"
        rust_source.parent.mkdir(parents=True)
        rust_source.write_text("pub fn item() {}\n", encoding="utf-8")
        source_paths = {
            "Java adapters": "bindings/java/src/main/java/org/kmipkit/Registry.java",
            "Python adapters": "bindings/python/src/kmipkit/extensions.py",
            "JNI bridge": "bindings/java/native/kmipkit_jni.cpp",
        }
        adapter_reports = {}
        for scope, relative in source_paths.items():
            source = root / relative
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text("\n".join(f"source line {number}" for number in range(1, line_count + 1)), encoding="utf-8")
            adapter_reports[scope] = {relative: {line: 1 for line in range(1, line_count + 1)}}
        rust_reports = {
            platform: {"crates/kmipkit-ttlv/src/lib.rs": {1: 1}}
            for platform in ("ubuntu", "windows", "macos")
        }
        return rust_reports, adapter_reports, source_paths

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

    def test_rust_ffi_and_transport_are_enforced_as_separate_scopes(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = {
                "kmipkit-ttlv": 1,
                "kmipkit-protocol": 1,
                "kmipkit-transport": 19,
                "kmipkit-ffi": 1,
            }
            line_maps = {}
            for crate, line_count in paths.items():
                relative = f"crates/{crate}/src/lib.rs"
                source = root / relative
                source.parent.mkdir(parents=True)
                source.write_text("\n".join("pub fn item() {}" for _ in range(line_count)), encoding="utf-8")
                line_maps[relative] = {
                    line: int(crate != "kmipkit-ffi") for line in range(1, line_count + 1)
                }
            reports = {platform: line_maps for platform in ("ubuntu", "windows", "macos")}
            ffi_c_consumer_report = {
                "crates/kmipkit-ffi/src/lib.rs": {1: 0},
            }

            with self.assertRaisesRegex(GATE.CoverageDataError, "kmipkit-ffi coverage 0/1.*85%"):
                GATE._evaluate_coverage(root, reports, "", ffi_c_consumer_report=ffi_c_consumer_report)

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

    def test_adapter_production_code_prevents_rust_only_unavailable_preflight(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rust_source = root / "crates/sample/src/lib.rs"
            rust_source.parent.mkdir(parents=True)
            rust_source.write_text("trait Declaration { fn method(); }\n", encoding="utf-8")
            java_source = root / "bindings/java/src/main/java/org/kmipkit/Registry.java"
            java_source.parent.mkdir(parents=True)
            java_source.write_text("class Registry { void register() {} }\n", encoding="utf-8")

            scan = GATE.scan_production_sources(root)

        self.assertTrue(scan.eligible)
        self.assertEqual("adapter production source requires coverage", scan.reason)

    def test_each_adapter_scope_fails_independently_at_85_percent(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rust_reports, adapter_reports, source_paths = self._write_gate_workspace(root)
            for scope, relative in source_paths.items():
                with self.subTest(scope=scope):
                    undercovered = dict(adapter_reports)
                    undercovered[scope] = {relative: {line: int(line <= 8) for line in range(1, 11)}}
                    with self.assertRaisesRegex(GATE.CoverageDataError, rf"{scope} coverage 8/10.*85%"):
                        GATE._evaluate_coverage(root, rust_reports, "", undercovered)

    def test_python_only_scope_does_not_require_rust_java_or_jni_artifacts(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            python_source = root / "bindings/python/src/kmipkit/extensions.py"
            python_source.parent.mkdir(parents=True)
            python_source.write_text("def operation():\n    return True\n", encoding="utf-8")
            java_source = root / "bindings/java/src/main/java/org/kmipkit/Registry.java"
            java_source.parent.mkdir(parents=True)
            java_source.write_text("class Registry {}\n", encoding="utf-8")
            jni_source = root / "bindings/java/native/kmipkit_jni.cpp"
            jni_source.parent.mkdir(parents=True)
            jni_source.write_text("int operation() { return 1; }\n", encoding="utf-8")
            report = {"bindings/python/src/kmipkit/extensions.py": {1: 1, 2: 1}}

            results = GATE._evaluate_coverage(
                root,
                {},
                "",
                adapter_reports={"Python adapters": report},
                required_scopes={"python"},
            )

        self.assertTrue(any(result.startswith("Python adapters coverage:") for result in results))
        self.assertFalse(any(result.startswith("Java adapters coverage:") for result in results))
        self.assertFalse(any(result.startswith("JNI bridge coverage:") for result in results))
        self.assertFalse(any(result.startswith("Workspace coverage:") for result in results))

    def test_missing_selected_scope_report_fails_and_unknown_scope_is_rejected(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "bindings/python/src/kmipkit/extensions.py"
            source.parent.mkdir(parents=True)
            source.write_text("def operation():\n    return True\n", encoding="utf-8")

            with self.assertRaisesRegex(GATE.CoverageDataError, "Required Python adapters coverage report is missing"):
                GATE._evaluate_coverage(root, {}, "", adapter_reports={}, required_scopes={"python"})

            with self.assertRaisesRegex(GATE.CoverageDataError, "Unknown coverage scope"):
                GATE._evaluate_coverage(
                    root,
                    {},
                    "",
                    adapter_reports={"Python adapters": {"bindings/python/src/kmipkit/extensions.py": {1: 1, 2: 1}}},
                    required_scopes={"not-a-scope"},
                )

    def test_ffi_c_only_scope_requires_its_own_report_and_ffi_threshold(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            ffi_source = root / "crates/kmipkit-ffi/src/lib.rs"
            ffi_source.parent.mkdir(parents=True)
            ffi_source.write_text("pub fn operation() {}\n", encoding="utf-8")
            report = {"crates/kmipkit-ffi/src/lib.rs": {1: 1}}

            results = GATE._evaluate_coverage(
                root,
                {},
                "",
                ffi_c_consumer_report=report,
                required_scopes={"ffi-c"},
            )

        self.assertTrue(any(result.startswith("kmipkit-ffi coverage:") for result in results))
        self.assertFalse(any(result.startswith("Workspace coverage:") for result in results))

    def test_changed_ffi_lines_use_the_c_abi_report_when_rust_and_ffi_are_selected(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            relative = "crates/kmipkit-ffi/src/lib.rs"
            source = root / relative
            source.parent.mkdir(parents=True)
            source.write_text(
                "".join(f"pub fn operation_{line}() {{}}\n" for line in range(1, 21)),
                encoding="utf-8",
            )
            rust_report = {relative: {line: 1 for line in range(1, 21)}}
            platform_reports = {platform: rust_report for platform in ("ubuntu", "windows", "macos")}
            ffi_report = {relative: {line: int(line != 2) for line in range(1, 21)}}
            diff = "\n".join(
                (
                    f"diff --git a/{relative} b/{relative}",
                    f"--- a/{relative}",
                    f"+++ b/{relative}",
                    "@@ -1 +1,2 @@",
                    " pub fn operation_1() {}",
                    "+pub fn operation_2() {}",
                )
            )

            with self.assertRaisesRegex(
                GATE.CoverageDataError, "Changed production code coverage 0/1.*95%"
            ):
                GATE._evaluate_coverage(
                    root,
                    platform_reports,
                    diff,
                    ffi_c_consumer_report=ffi_report,
                    required_scopes={"rust", "ffi-c"},
                )

    def test_changed_java_python_and_jni_production_lines_share_the_95_percent_gate(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rust_reports, adapter_reports, source_paths = self._write_gate_workspace(root, line_count=20)
            changed_lines = {}
            for scope, relative in source_paths.items():
                adapter_reports[scope] = {
                    relative: {line: int(line < 20) for line in range(1, 21)}
                }
                changed_lines[relative] = "\n".join(
                    (
                        f"diff --git a/{relative} b/{relative}",
                        f"--- a/{relative}",
                        f"+++ b/{relative}",
                        "@@ -19 +19,2 @@",
                        " source line 19",
                        "+new production line",
                    )
                )

            with self.assertRaisesRegex(GATE.CoverageDataError, "Changed production code coverage 0/3.*95%"):
                GATE._evaluate_coverage(root, rust_reports, "\n".join(changed_lines.values()), adapter_reports)

    def test_jni_llvm_export_parser_accepts_the_native_bridge_file(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "bindings/java/native/kmipkit_jni.cpp"
            source.parent.mkdir(parents=True)
            source.write_text("int bridge() { return 1; }\n", encoding="utf-8")
            document = llvm_document(
                [[1, 1, 1, len("int bridge() { return 1; }") + 1, 1, 0, 0, 0]],
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
        if jacoco_version and jacoco_version.startswith("${") and jacoco_version.endswith("}"):
            jacoco_version = pom.findtext(f"m:properties/m:{jacoco_version[2:-1]}", namespaces=namespace)
        self.assertRegex(jacoco_version or "", r"^\d+\.\d+\.\d+$", "JaCoCo version must be pinned")
        java_configuration = ET.tostring(jacoco, encoding="unicode")
        self.assertIn("0.85", java_configuration, "Java line coverage must enforce the 85% package gate")

        pyproject_path = REPOSITORY_ROOT / "bindings/python/pyproject.toml"
        import re
        import tomllib

        with pyproject_path.open("rb") as pyproject_file:
            pyproject = tomllib.load(pyproject_file)
        requirements = REPOSITORY_ROOT / "bindings/python/requirements-coverage.txt"
        pinned = {
            name: version
            for line in requirements.read_text(encoding="utf-8").splitlines()
            if line and not line.startswith("#")
            for name, version in [line.split("==", maxsplit=1)]
        }
        self.assertEqual(
            {"cffi", "coverage", "maturin", "pytest", "pytest-cov"},
            set(pinned),
            "the Python collector must expose one complete pinned test/build tool set",
        )
        self.assertTrue(all(re.fullmatch(r"\d+\.\d+\.\d+", version) for version in pinned.values()))
        self.assertEqual("kmipkit", pyproject["tool"]["coverage"]["run"]["source"][0])
        self.assertEqual(85, pyproject["tool"]["coverage"]["report"]["fail_under"])
        self.assertEqual(
            pyproject["tool"]["coverage"]["run"].get("omit"),
            ["*/kmipkit/_ffi/__init__.py", "*/kmipkit/_ffi/ffi.py"],
            "only build-generated CFFI wrapper modules may be excluded from Python source coverage",
        )

        native_collector = REPOSITORY_ROOT / "scripts/collect_jni_coverage.sh"
        self.assertTrue(native_collector.is_file(), "the JNI bridge needs a Linux LLVM coverage collector")
        script = native_collector.read_text(encoding="utf-8")
        self.assertIn("20.1.8", script)
        self.assertIn('readonly llvm_major="20"', script)
        self.assertIn('"clang++-${llvm_major}"', script)
        self.assertIn('"llvm-profdata-${llvm_major}"', script)
        self.assertIn('"llvm-cov-${llvm_major}"', script)
        self.assertIn('--ignore-filename-regex="${ignored_source_regex}"', script)
        self.assertIn('^/usr/lib/jvm/[^/]+/include/', script)
        self.assertIn('^/opt/hostedtoolcache/Java_[^/]+', script)
        self.assertNotIn('(^|/)include/', script)
        native_build = (REPOSITORY_ROOT / "bindings/java/native/build.sh").read_text(encoding="utf-8")
        self.assertIn("kmipkit_jni.cpp", native_build)
        self.assertIn("readlink -f", native_build, "Linux must resolve the javac symlink before locating jni.h")
        self.assertIn("-fprofile-instr-generate", native_build)
        self.assertIn("-fcoverage-mapping", native_build)


if __name__ == "__main__":
    unittest.main()
