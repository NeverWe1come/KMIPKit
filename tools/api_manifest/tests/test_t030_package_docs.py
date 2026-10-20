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
        declared_operations = [
            operation for operation in operations if operation["apiState"] == "declared"
        ]
        verified_operations = [
            operation for operation in operations if operation["apiState"] == "verified"
        ]
        self.assertEqual(
            [operation["name"] for operation in declared_operations],
            [
                "Create",
                "Create Key Pair",
                "Create Split Key",
                "Discover Versions",
                "Hash",
                "MAC",
                "MAC Verify",
                "Sign",
                "Signature Verify",
            ],
        )
        self.assertEqual(
            [operation["name"] for operation in verified_operations],
            [
                "Certify",
                "Check",
                "Decrypt",
                "Delegated Login",
                "Derive Key",
                "Encrypt",
                "Export",
                "Get",
                "Get Constraints",
                "Get Usage Allocation",
                "Import",
                "Interop",
                "Join Split Key",
                "Locate",
                "Log",
                "Login",
                "Logout",
                "Obtain Lease",
                "Ping",
                "PKCS#11",
                "Query",
                "Re-certify",
                "Re-key",
                "Re-key Key Pair",
            ],
        )
        self.assertEqual(
            len(operations),
            blocked_operations + len(declared_operations) + len(verified_operations),
        )

        for guide, operation_phrase in (
            (self.english, "full operation parity"),
            (self.spanish, "paridad completa de operaciones"),
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
            for operation in (*declared_operations, *verified_operations):
                self.assertIn(operation["name"].lower(), normalized_guide)

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
