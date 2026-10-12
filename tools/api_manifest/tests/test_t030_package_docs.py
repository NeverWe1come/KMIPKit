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
            self.assertRegex(guide.lower(), r"no runtime (code )?download|no descarga")
            self.assertRegex(guide.lower(), r"no fallback|sin fallback")

    def test_bilingual_capability_summary_does_not_advertise_blocked_surface(self) -> None:
        capabilities = self.manifest["capabilities"]
        operations = self.manifest["operations"]
        blocked_operations = sum(
            operation["apiState"] == "blocked" for operation in operations
        )
        self.assertEqual(len(operations), blocked_operations)

        for guide in (self.english, self.spanish):
            for capability in capabilities:
                self.assertIn(capability["capabilityId"], guide)
                status = capability["capabilityState"]
                expected_status = {
                    "declared": "declared" if guide is self.english else "declarada",
                    "blocked": "blocked" if guide is self.english else "bloqueada",
                }[status]
                self.assertRegex(
                    guide.lower(),
                    rf"{re.escape(capability['capabilityId'].lower())}[\s\S]{{0,100}}{expected_status}",
                )
            self.assertRegex(guide, rf"{blocked_operations}[/ de]{len(operations)}")
            self.assertIn("full operation parity", guide.lower())
            self.assertIn("paridad completa de operaciones", guide.lower())


if __name__ == "__main__":
    unittest.main()
