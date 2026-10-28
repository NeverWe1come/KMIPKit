"""Contracts for synchronized Java/Python packages and user-facing scope docs."""

from __future__ import annotations

import json
import re
import tomllib
import unittest
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
ENGLISH_GUIDE = ROOT / "docs" / "user-guide" / "en" / "language-packages.md"
SPANISH_GUIDE = ROOT / "docs" / "user-guide" / "es" / "paquetes-de-idiomas.md"


class PackageDocumentationContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.english = (
            ENGLISH_GUIDE.read_text(encoding="utf-8") if ENGLISH_GUIDE.exists() else ""
        )
        cls.spanish = (
            SPANISH_GUIDE.read_text(encoding="utf-8") if SPANISH_GUIDE.exists() else ""
        )

    def test_java_and_python_package_versions_match_the_manifest(self) -> None:
        release_version = self.manifest["releaseLine"]
        gradle = (ROOT / "bindings" / "java" / "build.gradle.kts").read_text(
            encoding="utf-8"
        )
        gradle_version = re.search(r'^version = "([^\"]+)"$', gradle, re.MULTILINE)
        self.assertIsNotNone(gradle_version)
        self.assertEqual(release_version, gradle_version.group(1))

        namespace = {"m": "http://maven.apache.org/POM/4.0.0"}
        pom = ET.parse(ROOT / "bindings" / "java" / "pom.xml").getroot()
        self.assertEqual(
            release_version,
            pom.findtext("m:version", namespaces=namespace),
        )
        self.assertEqual(
            "org.kmipkit:kmipkit-java",
            ":".join(
                (
                    pom.findtext("m:groupId", namespaces=namespace),
                    pom.findtext("m:artifactId", namespaces=namespace),
                )
            ),
        )

        pyproject = tomllib.loads(
            (ROOT / "bindings" / "python" / "pyproject.toml").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(release_version, pyproject["project"]["version"])
        self.assertEqual("kmipkit", pyproject["project"]["name"])
        self.assertIn(release_version, pyproject["project"]["description"])
        self.assertIn(release_version, pom.findtext("m:description", namespaces=namespace))

    def test_jspecify_is_a_public_compile_dependency_in_gradle_and_maven(self) -> None:
        gradle = (ROOT / "bindings" / "java" / "build.gradle.kts").read_text(
            encoding="utf-8"
        )
        self.assertIn('api("org.jspecify:jspecify:1.0.0")', gradle)

        namespace = {"m": "http://maven.apache.org/POM/4.0.0"}
        pom = ET.parse(ROOT / "bindings" / "java" / "pom.xml").getroot()
        dependencies = pom.findall("m:dependencies/m:dependency", namespaces=namespace)
        jspecify = [
            dependency
            for dependency in dependencies
            if dependency.findtext("m:groupId", namespaces=namespace) == "org.jspecify"
            and dependency.findtext("m:artifactId", namespaces=namespace) == "jspecify"
        ]
        self.assertEqual(1, len(jspecify))
        self.assertEqual("1.0.0", jspecify[0].findtext("m:version", namespaces=namespace))
        self.assertNotEqual("test", jspecify[0].findtext("m:scope", namespaces=namespace))

    def test_metadata_and_both_guides_name_the_supported_language_runtimes(self) -> None:
        java_build = (ROOT / "bindings" / "java" / "build.gradle.kts").read_text(
            encoding="utf-8"
        )
        pom = ET.parse(ROOT / "bindings" / "java" / "pom.xml").getroot()
        namespace = {"m": "http://maven.apache.org/POM/4.0.0"}
        pyproject = tomllib.loads(
            (ROOT / "bindings" / "python" / "pyproject.toml").read_text(
                encoding="utf-8"
            )
        )

        self.assertIn("options.release.set(17)", java_build)
        self.assertEqual("17", pom.findtext("m:properties/m:maven.compiler.release", namespaces=namespace))
        self.assertEqual(">=3.12,<3.13", pyproject["project"]["requires-python"])
        for guide in (self.english, self.spanish):
            self.assertIn("Java 17", guide)
            self.assertIn("Python 3.12", guide)
        for relative_path in (
            "docs/design/project-definition.md",
            "docs/development/coding-standards.md",
        ):
            source = (ROOT / relative_path).read_text(encoding="utf-8")
            self.assertIn("Python 3.12 only for the 1.0 package", source)

    def test_bilingual_guides_match_native_loading_and_platform_contract(self) -> None:
        packages = {
            package["id"]: package for package in self.manifest["publicPackages"]
        }
        targets = set(packages["java-kmipkit"]["supportedTargets"])
        self.assertEqual(targets, set(packages["python-kmipkit"]["supportedTargets"]))

        for guide in (self.english, self.spanish):
            for target in sorted(targets):
                self.assertIn(target, guide)
            for contract_term in (
                "KMIPKIT_NATIVE_LIBRARY",
                "kmipkit.nativeLibrary",
                "kmipkit_version_get",
                "SHA-256",
            ):
                self.assertIn(contract_term, guide)
        english = re.sub(r"\s+", " ", self.english.lower())
        spanish = re.sub(r"\s+", " ", self.spanish.lower())
        self.assertIn("do not perform a runtime executable-code download", english)
        self.assertIn("no descargan código ejecutable durante la ejecución", spanish)
        self.assertIn("no fallback", english)
        self.assertIn("sin fallback", spanish)

    def test_bilingual_capability_summary_does_not_advertise_blocked_surface(self) -> None:
        capabilities = self.manifest["capabilities"]
        operations = self.manifest["operations"]
        blocked_operations = sum(
            operation["apiState"] == "blocked" for operation in operations
        )
        verified_operation_count = sum(
            operation["apiState"] == "verified" for operation in operations
        )
        declared_operations = [
            operation for operation in operations if operation["apiState"] == "declared"
        ]
        verified_operations = [
            operation for operation in operations if operation["apiState"] == "verified"
        ]
        self.assertEqual(
            len(operations),
            blocked_operations + len(declared_operations) + verified_operation_count,
        )

        register = next(
            operation for operation in operations if operation["name"] == "Register"
        )
        self.assertEqual("verified", register["apiState"])

        for guide, operation_phrase, declared_marker, verified_marker in (
            (
                self.english,
                "full operation parity",
                f"**{len(declared_operations)}** rows are `declared`:",
                f"**{verified_operation_count}** operation mappings are `verified`:",
            ),
            (
                self.spanish,
                "paridad completa de operaciones",
                f"**{len(declared_operations)}** filas están en estado `declared`:",
                f"**{verified_operation_count}** mapeos de operación están en estado `verified`:",
            ),
        ):
            for capability in capabilities:
                self.assertIn(capability["capabilityId"], guide)
                status = capability["capabilityState"]
                self.assertRegex(
                    guide.lower(),
                    rf"{re.escape(capability['capabilityId'].lower())}[\s\S]{{0,80}}{status}",
                )
            self.assertIn(f"{blocked_operations} / {len(operations)}", guide)
            normalized_guide = re.sub(r"\s+", " ", guide.lower())
            self.assertIn(operation_phrase, normalized_guide)
            declared_start = normalized_guide.index(declared_marker.lower())
            verified_start = normalized_guide.index(
                verified_marker.lower(), declared_start + len(declared_marker)
            )
            declared_section = normalized_guide[declared_start:verified_start]
            verified_section = normalized_guide[verified_start:]
            for operation in (*declared_operations, *verified_operations):
                self.assertIn(operation["name"].lower(), normalized_guide)
            for operation in declared_operations:
                self.assertIn(operation["name"].lower(), declared_section)
            for operation in verified_operations:
                self.assertIn(operation["name"].lower(), verified_section)
        self.assertIn(
            f"**{len(declared_operations)}** rows are `declared`", self.english
        )
        self.assertIn(
            f"**{verified_operation_count}** operation mappings are `verified`",
            self.english,
        )
        self.assertIn(
            f"**{len(declared_operations)}** filas están en estado `declared`",
            self.spanish,
        )
        self.assertIn(
            f"**{verified_operation_count}** mapeos de operación están en estado `verified`",
            self.spanish,
        )
        self.assertNotIn("full Register operation parity remains blocked", self.english)
        self.assertNotIn(
            "la paridad completa de la operación Register sigue bloqueada",
            self.spanish.lower(),
        )
        self.assertIn("The Register operation row is verified.", self.english)
        self.assertIn("La fila de la operación Register está `verified`.", self.spanish)

    def test_bilingual_client_execution_guides_scope_operation_rows_to_rust(self) -> None:
        guides = {
            "en": (ROOT / "docs/user-guide/en/client-execution.md").read_text(
                encoding="utf-8"
            ),
            "es": (ROOT / "docs/user-guide/es/ejecucion-cliente.md").read_text(
                encoding="utf-8"
            ),
        }
        required_scope = {
            "en": (
                "This guide describes the Rust `kmipkit-client` API surface.",
                "A Rust model or accessor does not establish four-language 1.0 operation availability.",
                "`blocked` means the complete four-language 1.0 operation mapping is unavailable, even if Rust types exist.",
                "[language and package status](language-packages.md)",
            ),
            "es": (
                "Esta guía describe la superficie de la API Rust de `kmipkit-client`.",
                "Un modelo o accesor de Rust no establece la disponibilidad de una operación en los cuatro lenguajes de la versión 1.0.",
                "`blocked` significa que el mapeo completo de operaciones 1.0 para los cuatro lenguajes no está disponible, aunque existan tipos de Rust.",
                "[estado de paquetes e idiomas](paquetes-de-idiomas.md)",
            ),
        }
        for language, guide in guides.items():
            normalized_guide = re.sub(r"\s+", " ", guide)
            with self.subTest(language=language):
                for statement in required_scope[language]:
                    self.assertIn(statement, normalized_guide)

    def test_public_api_reference_marks_async_followups_blocked(self) -> None:
        reference = (ROOT / "docs/architecture/public-api.md").read_text(
            encoding="utf-8"
        )
        operations = {
            operation["name"]: operation for operation in self.manifest["operations"]
        }
        documented_states = {
            name.strip(): state.strip()
            for name, state in re.findall(
                r"(?m)^\|\s*([^|]+?)\s*\|\s*(blocked|declared|verified)\s*\|$",
                reference,
            )
        }
        asynchronous_followups = (
            "Cancel",
            "Poll",
            "Process",
            "Query Asynchronous Requests",
        )

        for operation_name in asynchronous_followups:
            with self.subTest(operation=operation_name):
                self.assertEqual("blocked", operations[operation_name]["apiState"])
                self.assertEqual("blocked", documented_states.get(operation_name))

    def test_public_api_reference_lists_every_verified_rust_operation(self) -> None:
        reference = (ROOT / "docs/architecture/public-api.md").read_text(
            encoding="utf-8"
        )
        typed_protocol = reference.split("### Typed protocol", maxsplit=1)[1].split(
            "### ", maxsplit=1
        )[0]
        documented_operation_list = re.sub(
            r"\s+",
            " ",
            typed_protocol.split("The API does not accept", maxsplit=1)[0],
        )

        for operation in self.manifest["operations"]:
            if operation["apiState"] != "verified":
                continue
            with self.subTest(operation=operation["name"]):
                self.assertIn(operation["name"], documented_operation_list)

    def test_manifest_maintenance_doc_counts_all_v2_output_destinations(self) -> None:
        documentation = (ROOT / "docs/development/api-manifest.md").read_text(
            encoding="utf-8"
        )
        normalized_documentation = re.sub(r"\s+", " ", documentation)
        outputs = self.manifest["generatedOutputs"]
        registry_outputs = outputs[:6]

        self.assertEqual(6, len(registry_outputs))
        self.assertEqual(6, len(outputs) - len(registry_outputs))
        self.assertIn("six additional destinations", normalized_documentation)

    def test_bilingual_package_guides_match_current_operation_readiness(self) -> None:
        operations = [
            operation
            for operation in self.manifest["operations"]
            if operation["scope"] == "client_1_0"
        ]
        counts = {
            state: sum(operation["apiState"] == state for operation in operations)
            for state in ("verified", "declared", "blocked")
        }

        for language, guide in (("en", self.english), ("es", self.spanish)):
            normalized = re.sub(r"\s+", " ", guide).lower()
            with self.subTest(language=language):
                self.assertIn(f"**{counts['blocked']} / {len(operations)}**", normalized)
                self.assertIn(f"**{counts['declared']}**", normalized)
                self.assertIn(f"**{counts['verified']}**", normalized)
                for operation in operations:
                    with self.subTest(operation=operation["name"]):
                        self.assertIn(operation["name"].lower(), normalized)

    def test_t071_checkpoint_records_active_manifest_snapshot_and_remains_open(self) -> None:
        tasks = (ROOT / "specs/043-public-api-bindings/tasks.md").read_text(
            encoding="utf-8"
        )
        operations = self.manifest["operations"]
        operation_counts = {
            state: sum(operation["apiState"] == state for operation in operations)
            for state in ("verified", "declared", "blocked")
        }
        capabilities = self.manifest["capabilities"]
        capability_counts = {
            state: sum(
                capability["capabilityState"] == state for capability in capabilities
            )
            for state in ("verified", "declared", "blocked")
        }
        declarations = self.manifest["publicDeclarations"]
        declaration_counts = {
            state: sum(
                declaration["declarationState"] == state
                for declaration in declarations
            )
            for state in ("verified", "declared", "blocked")
        }
        checkpoint = re.search(
            r"(?s)\*\*Checkpoint \(manifest snapshot, [^)]+\)\*\*: (.*?)(?=\n\n)",
            tasks,
        )
        self.assertIsNotNone(checkpoint)
        assert checkpoint is not None
        for audited_status in (
            f"{operation_counts['verified']}/{len(operations)} `client_1_0` operations as verified, "
            f"{operation_counts['declared']} as declared, and {operation_counts['blocked']} as blocked",
            f"{capability_counts['verified']}/{len(capabilities)} verified "
            f"({capability_counts['declared']} declared, {capability_counts['blocked']} blocked)",
            f"{declaration_counts['verified']:,}/{len(declarations):,} verified, "
            f"{declaration_counts['declared']:,} declared, and "
            f"{declaration_counts['blocked']:,} blocked",
            f"{len(self.manifest['approvedExclusions'])} approved exclusions",
            "T071 remains open",
        ):
            self.assertIn(audited_status, checkpoint.group(1))
        self.assertRegex(tasks, r"(?m)^- \[ \] T071 \[US2\]")

    def test_recertify_summary_names_every_mapped_request_field(self) -> None:
        request_fields = (
            "Certificate Request Unique Identifier",
            "Certificate Request Type",
            "Certificate Request Value",
            "Protection Storage Masks",
        )
        for language, guide in (("en", self.english), ("es", self.spanish)):
            with self.subTest(language=language):
                normalized_guide = " ".join(guide.split())
                for field in request_fields:
                    self.assertIn(field, normalized_guide)

    def test_vendor_extension_examples_document_native_handle_release_parity(self) -> None:
        english = (ROOT / "docs/user-guide/en/vendor-extensions.md").read_text(
            encoding="utf-8"
        )
        spanish = (ROOT / "docs/user-guide/es/extensiones-de-fabricante.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("vendor_extension_registry", english)
        self.assertIn("declared", english)
        self.assertIn("Rust drops the value", english)
        self.assertIn("C calls its matching `_release` function", english)
        self.assertIn("Java and Python expose `close()`", english)
        self.assertIn("No extension-handle release remains blocked", english)
        self.assertIn(
            "verified four-language 1.0 capability",
            " ".join(english.split()),
        )
        self.assertIn("vendor_extension_registry", spanish)
        self.assertIn("declared", spanish)
        self.assertIn("Rust descarta el valor", spanish)
        self.assertIn("C llama a su función `_release` correspondiente", spanish)
        self.assertIn("Java y Python exponen", spanish)
        self.assertIn("El manifiesto ya no bloquea la liberación", spanish)
        self.assertIn("capacidad 1.0 verificada", " ".join(spanish.split()))


if __name__ == "__main__":
    unittest.main()
