"""T092 contract for KMIPKIT-0008 credential declaration parity."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "KMIPKIT-0008"
LANGUAGES = {"rust", "c", "java", "python"}
EXPECTED_CREDENTIAL_TYPE_EXPORTS = {
    "kmipkit_protocol::CredentialType",
    "kmipkit_protocol::CredentialType::UsernameAndPassword",
    "kmipkit_protocol::CredentialType::Device",
    "kmipkit_protocol::CredentialType::Attestation",
    "kmipkit_protocol::CredentialType::OneTimePassword",
    "kmipkit_protocol::CredentialType::HashedPassword",
    "kmipkit_protocol::CredentialType::Ticket",
    "kmipkit_protocol::CredentialType::Extensions",
    "kmipkit_protocol::CredentialType::Unknown",
    "kmipkit_protocol::CredentialType::from_raw",
    "kmipkit_protocol::CredentialType::raw",
    "kmipkit_protocol::CredentialType::Extensions::0",
    "kmipkit_protocol::CredentialType::Unknown::0",
}
EXPECTED_NONCE_EXPORTS = {
    "kmipkit_protocol::Nonce",
    "kmipkit_protocol::Nonce::try_from_ttlv",
    "kmipkit_protocol::Nonce::with_ttlv",
    "kmipkit_protocol::Nonce::into_ttlv",
}
ATTESTATION_CREDENTIAL_EXPORTS = {
    "kmipkit_protocol::AttestationCredential",
    "kmipkit_protocol::AttestationCredential::try_from_ttlv",
    "kmipkit_protocol::AttestationCredential::with_ttlv",
    "kmipkit_protocol::AttestationCredential::into_ttlv",
}
CREDENTIAL_EXPORTS = {
    "kmipkit_protocol::Credential",
    "kmipkit_protocol::Credential::try_from_ttlv",
    "kmipkit_protocol::Credential::credential_type",
    "kmipkit_protocol::Credential::credential_type_raw",
    "kmipkit_protocol::Credential::with_ttlv",
    "kmipkit_protocol::Credential::into_ttlv",
}
AUTHENTICATION_EXPORTS = {
    "kmipkit_protocol::Authentication",
    "kmipkit_protocol::Authentication::new",
    "kmipkit_protocol::Authentication::try_from_ttlv",
    "kmipkit_protocol::Authentication::credentials",
    "kmipkit_protocol::Authentication::with_ttlv",
    "kmipkit_protocol::Authentication::into_ttlv",
    "kmipkit_protocol::CredentialView",
    "kmipkit_protocol::CredentialView::credential_type",
    "kmipkit_protocol::CredentialView::credential_type_raw",
    "kmipkit_protocol::CredentialView::with_ttlv",
}
DEVICE_CREDENTIAL_EXPORTS = {
    "kmipkit_protocol::DeviceCredential",
    "kmipkit_protocol::DeviceCredential::try_from_ttlv",
    "kmipkit_protocol::DeviceCredential::with_ttlv",
    "kmipkit_protocol::DeviceCredential::into_ttlv",
}
EXPECTED_MAPPED_CREDENTIAL_EXPORTS = (
    EXPECTED_CREDENTIAL_TYPE_EXPORTS
    | EXPECTED_NONCE_EXPORTS
    | CREDENTIAL_EXPORTS
    | ATTESTATION_CREDENTIAL_EXPORTS
    | DEVICE_CREDENTIAL_EXPORTS
    | AUTHENTICATION_EXPORTS
)
NONCE_REQUIREMENTS = {
    "KMIPKIT-0008-FR-006",
    "KMIPKIT-REQ-SPEC-9.11-006",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
RUST_EXPORT_INVENTORY = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-10-31.json"
)


def _is_nonblank_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _has_approved_source(record: dict[str, object]) -> bool:
    source = record.get("approvalSource")
    return (
        isinstance(source, dict)
        and source.get("status") == "Approved"
        and _is_nonblank_string(source.get("path"))
        and _is_nonblank_string(source.get("evidence"))
    )


class T092CredentialsParityManifestTests(unittest.TestCase):
    def test_exact_trait_implementation_artifacts_have_owner_backed_exclusions(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        inventory = json.loads(RUST_EXPORT_INVENTORY.read_text(encoding="utf-8"))
        declarations = manifest["publicDeclarations"]
        owner_paths = {
            row["exportName"]: tuple(json.loads(inventory_row["identity"])[3])
            for row in declarations
            if row.get("kind") == "type"
            and (
                row.get("candidateSpecId") == OWNER_SPEC
                or (
                    row.get("exportName")
                    in {
                        "kmipkit_protocol::CredentialType",
                        "kmipkit_protocol::Nonce",
                        "kmipkit_protocol::Credential",
                        "kmipkit_protocol::AttestationCredential",
                        "kmipkit_protocol::DeviceCredential",
                        "kmipkit_protocol::Authentication",
                        "kmipkit_protocol::CredentialView",
                    }
                    and row.get("approvalSource", {}).get("path")
                    == "specs/008-credentials-attestation/spec.md"
                )
            )
            for inventory_row in inventory["declarations"]
            if inventory_row.get("exportName") == row["exportName"]
            and isinstance(inventory_row.get("identity"), str)
        }
        self.assertIn("kmipkit_protocol::Authentication", owner_paths)
        self.assertIn("kmipkit_protocol::CredentialView", owner_paths)
        self.assertEqual(17, len(owner_paths), "the credential owner type inventory changed")

        inventory_rows = inventory["declarations"] + inventory["conditionalDeclarations"]
        trait_exports: set[str] = set()
        trait_path_rows = [
            row
            for row in declarations + manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("candidateSpecId") == OWNER_SPEC
            and isinstance(row.get("exportName"), str)
            and row["exportName"].startswith("<kmipkit_protocol::")
            and " as " in row["exportName"]
            and ">::" in row["exportName"]
        ]
        trait_exports = {row["exportName"] for row in trait_path_rows}
        self.assertEqual(222, len(trait_exports), "the exact T092 trait declarations changed")

        for export_name in sorted(trait_exports):
            owner_expression, trait_member = export_name[1:].split(" as ", 1)
            owner_export = owner_expression.split("<", 1)[0]
            self.assertIn(owner_export, owner_paths)
            owner_path = owner_paths[owner_export]
            matching_rows = [
                row
                for row in inventory_rows
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName") == export_name
            ]
            self.assertEqual(1, len(matching_rows), f"Rust inventory identity for {export_name}")
            observed = matching_rows[0]
            self.assertEqual("associated-item", observed.get("kind"))

            identity = (
                json.loads(observed["identity"])
                if isinstance(observed.get("identity"), str)
                else None
            )
            direct_trait = (
                isinstance(identity, list)
                and len(identity) >= 9
                and identity[0] == "rust-trait-implementation"
                and identity[1] == "rust-kmipkit-protocol"
                and identity[2] == list(owner_path)
                and isinstance(identity[3], list)
                and bool(identity[3])
            )
            conditional_traits = [
                predicate.get("traitPath")
                for predicate in observed.get("condition", {}).get("predicates", [])
                if isinstance(predicate, dict)
                and predicate.get("kind") == "traitBound"
                and isinstance(predicate.get("traitPath"), list)
                and predicate.get("traitPath")
            ]
            conditional_trait = (
                observed in inventory["conditionalDeclarations"]
                and observed.get("ownerPath") == list(owner_path)
                and bool(conditional_traits)
                and _is_nonblank_string(observed.get("implementationId"))
            )
            self.assertTrue(
                direct_trait or conditional_trait,
                f"Rust inventory does not identify {export_name} as a trait member",
            )

            exported_trait, exported_member = trait_member.rsplit(">::", 1)
            self.assertTrue(exported_member)
            if direct_trait:
                self.assertEqual("::".join(identity[3]), exported_trait.split("<", 1)[0])
                self.assertEqual(identity[7], exported_member)
            trait_exports.add(export_name)
        self.assertEqual(222, len(trait_exports), "the reviewed T092 trait slice changed")

        owner_exclusions = [
            row
            for row in manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("candidateSpecId") == OWNER_SPEC
        ]
        exclusion_counts: dict[str, int] = {}
        for row in owner_exclusions:
            name = row.get("exportName")
            if isinstance(name, str):
                exclusion_counts[name] = exclusion_counts.get(name, 0) + 1
        duplicates = sorted(name for name, count in exclusion_counts.items() if count != 1)
        self.assertEqual([], duplicates, "duplicate exact KMIPKIT-0008 exclusions")
        actual_exclusions = {
            row["exportName"]: row
            for row in owner_exclusions
            if isinstance(row.get("exportName"), str)
        }
        unaccounted = sorted(trait_exports - actual_exclusions.keys())
        extra_exclusions = sorted(actual_exclusions.keys() - trait_exports)
        self.assertEqual(
            0,
            len(unaccounted) + len(extra_exclusions),
            "unaccounted exact KMIPKIT-0008 Rust trait exports "
            f"({len(unaccounted)}); extra exclusions ({len(extra_exclusions)}): "
            f"{unaccounted[:5]} / {extra_exclusions[:5]}",
        )

        for export_name in sorted(trait_exports):
            exclusion = actual_exclusions[export_name]
            self.assertTrue(export_name.startswith("<kmipkit_protocol::"))
            self.assertIn(" as ", export_name)
            self.assertTrue(_is_nonblank_string(exclusion.get("reason")))
            self.assertTrue(_is_nonblank_string(exclusion.get("evidence")))
            self.assertTrue(_has_approved_source(exclusion))
            self.assertEqual(
                "specs/008-credentials-attestation/spec.md",
                exclusion["approvalSource"]["path"],
            )

        retained_trait_declarations = {
            row.get("exportName")
            for row in declarations
            if row.get("candidateSpecId") == OWNER_SPEC
            and row.get("exportName") in trait_exports
        }
        self.assertEqual(set(), retained_trait_declarations)

        meaningful = [
            row for row in declarations
            if (
                row.get("candidateSpecId") == OWNER_SPEC
                and row.get("exportName") not in trait_exports
            )
            or (
                row.get("exportName") in EXPECTED_MAPPED_CREDENTIAL_EXPORTS
                and row.get("approvalSource", {}).get("path")
                == "specs/008-credentials-attestation/spec.md"
            )
        ]
        self.assertEqual(96, len(meaningful))
        credential_type_rows = {
            row["exportName"]: row
            for row in meaningful
            if row.get("exportName") in EXPECTED_MAPPED_CREDENTIAL_EXPORTS
        }
        self.assertEqual(EXPECTED_MAPPED_CREDENTIAL_EXPORTS, credential_type_rows.keys())
        self.assertTrue(
            all(row.get("declarationState") == "declared" for row in credential_type_rows.values())
        )
        self.assertTrue(
            all("candidateSpecId" not in row for row in credential_type_rows.values())
        )
        remaining = [
            row for row in meaningful
            if row.get("exportName") not in EXPECTED_MAPPED_CREDENTIAL_EXPORTS
        ]
        self.assertEqual(55, len(remaining))
        self.assertTrue(all(row.get("declarationState") == "blocked" for row in remaining))

    def test_credential_type_is_mapped_as_an_open_raw_enum(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = {
            row["exportName"]: row
            for row in manifest["publicDeclarations"]
            if row.get("exportName") in EXPECTED_CREDENTIAL_TYPE_EXPORTS
        }
        self.assertEqual(EXPECTED_CREDENTIAL_TYPE_EXPORTS, declarations.keys())
        for export_name, declaration in declarations.items():
            with self.subTest(export_name=export_name):
                self.assertEqual("declared", declaration.get("declarationState"))
                self.assertEqual("four-language", declaration.get("parityScope"))
                self.assertTrue(_has_approved_source(declaration))
                self.assertEqual(
                    "specs/008-credentials-attestation/spec.md",
                    declaration["approvalSource"]["path"],
                )
                mappings = declaration.get("languageMappings")
                self.assertIsInstance(mappings, dict)
                self.assertEqual(LANGUAGES, set(mappings))
                self.assertTrue(
                    all(
                        _is_nonblank_string(mappings[language].get("name"))
                        and _is_nonblank_string(mappings[language].get("packageId"))
                        for language in LANGUAGES
                    )
                )
                self.assertTrue(
                    {"KMIPKIT-0008-FR-003", "KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"}
                    <= set(declaration.get("requirementIds", []))
                )

        credential_type = next(
            row for row in manifest["types"] if row["id"] == "CredentialType"
        )
        self.assertEqual("open-numeric", credential_type["enumRepresentation"]["shape"])
        self.assertEqual(
            "preserve-raw", credential_type["enumRepresentation"]["unknownValuePolicy"]
        )
        self.assertEqual("preserve-raw", credential_type["c"]["unknownValuePolicy"])
        function = next(
            row for row in manifest["functions"] if row["id"] == "credential_type_from_raw"
        )
        self.assertEqual("kmipkit_credential_type_from_raw", function["c"]["symbol"])
        self.assertEqual("from_raw", function["rust"]["name"])

    def test_partial_owner_parity_maps_credential_type_and_keeps_exact_blockers(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = [
            row
            for row in manifest["publicDeclarations"]
            if row.get("candidateSpecId") == OWNER_SPEC
        ]
        self.assertTrue(declarations, f"no declarations found for {OWNER_SPEC}")

        blocked = [
            declaration
            for declaration in declarations
            if declaration.get("declarationState") == "blocked"
        ]
        self.assertEqual(55, len(blocked))

        exclusions = {
            (row.get("packageId"), row.get("exportName")): row
            for row in manifest.get("approvedExclusions", [])
        }
        unresolved: list[str] = []

        for declaration in declarations:
            if declaration.get("declarationState") == "blocked":
                continue

            key = (declaration.get("packageId"), declaration.get("exportName"))
            mappings = declaration.get("languageMappings")
            complete_mapping = (
                isinstance(mappings, dict)
                and set(mappings) == LANGUAGES
                and all(
                    isinstance(mappings[language], dict)
                    and _is_nonblank_string(mappings[language].get("packageId"))
                    and _is_nonblank_string(mappings[language].get("name"))
                    for language in LANGUAGES
                )
                and _has_approved_source(declaration)
            )

            exclusion = exclusions.get(key)
            reviewed_exclusion = (
                isinstance(exclusion, dict)
                and exclusion.get("candidateSpecId") == OWNER_SPEC
                and _is_nonblank_string(exclusion.get("reason"))
                and _is_nonblank_string(exclusion.get("evidence"))
                and _has_approved_source(exclusion)
            )

            if not (complete_mapping or reviewed_exclusion):
                unresolved.append(
                    f"{declaration.get('packageId')}:{declaration.get('exportName')}"
                )

        self.assertEqual(
            0,
            len(unresolved),
            f"unresolved {OWNER_SPEC} declarations ({len(unresolved)} rows): "
            f"{unresolved[:5]}",
        )

    def test_nonce_type_and_lifecycle_map_through_all_four_languages(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = {
            row["exportName"]: row
            for row in manifest["publicDeclarations"]
            if row.get("exportName") in EXPECTED_NONCE_EXPORTS
        }
        self.assertEqual(EXPECTED_NONCE_EXPORTS, declarations.keys())

        expected_names = {
            "kmipkit_protocol::Nonce": {
                "rust": "kmipkit_protocol::Nonce",
                "c": "kmipkit_nonce_t",
                "java": "org.kmipkit.credentials.Nonce",
                "python": "kmipkit.credentials.Nonce",
            },
            "kmipkit_protocol::Nonce::try_from_ttlv": {
                "rust": "kmipkit_protocol::Nonce::try_from_ttlv",
                "c": "kmipkit_nonce_from_ttlv",
                "java": "org.kmipkit.credentials.Nonce.fromTtlv",
                "python": "kmipkit.credentials.Nonce.from_ttlv",
            },
            "kmipkit_protocol::Nonce::with_ttlv": {
                "rust": "kmipkit_protocol::Nonce::with_ttlv",
                "c": "kmipkit_nonce_ttlv_view",
                "java": "org.kmipkit.credentials.Nonce.withTtlv",
                "python": "kmipkit.credentials.Nonce.with_ttlv",
            },
            "kmipkit_protocol::Nonce::into_ttlv": {
                "rust": "kmipkit_protocol::Nonce::into_ttlv",
                "c": "kmipkit_nonce_into_ttlv",
                "java": "org.kmipkit.credentials.Nonce.intoTtlv",
                "python": "kmipkit.credentials.Nonce.into_ttlv",
            },
        }
        for export_name, declaration in declarations.items():
            with self.subTest(export_name=export_name):
                self.assertEqual("declared", declaration.get("declarationState"))
                self.assertEqual("four-language", declaration.get("parityScope"))
                self.assertNotIn("candidateSpecId", declaration)
                self.assertTrue(_has_approved_source(declaration))
                self.assertEqual(
                    "specs/008-credentials-attestation/spec.md",
                    declaration["approvalSource"]["path"],
                )
                self.assertTrue(
                    NONCE_REQUIREMENTS <= set(declaration.get("requirementIds", []))
                )
                mappings = declaration.get("languageMappings")
                self.assertIsInstance(mappings, dict)
                if not isinstance(mappings, dict):
                    continue
                self.assertEqual(LANGUAGES, set(mappings))
                for language, expected_name in expected_names[export_name].items():
                    self.assertEqual(expected_name, mappings[language]["name"])

        nonce_type = next(
            (row for row in manifest["types"] if row["id"] == "Nonce"), None
        )
        self.assertIsNotNone(nonce_type, "Nonce opaque handle record is missing")
        if nonce_type is None:
            return
        self.assertEqual("kmipkit_nonce_t", nonce_type["c"]["name"])
        self.assertEqual("kmipkit_nonce_release", nonce_type["c"]["releaseFunction"])
        self.assertEqual("org.kmipkit.credentials", nonce_type["java"]["package"])
        self.assertEqual("Nonce", nonce_type["java"]["name"])
        self.assertEqual("kmipkit.credentials", nonce_type["python"]["module"])
        self.assertEqual("Nonce", nonce_type["python"]["name"])

        for function_id, symbol in (
            ("nonce_from_ttlv", "kmipkit_nonce_from_ttlv"),
            ("nonce_ttlv_view", "kmipkit_nonce_ttlv_view"),
            ("nonce_into_ttlv", "kmipkit_nonce_into_ttlv"),
        ):
            function = next(row for row in manifest["functions"] if row["id"] == function_id)
            self.assertEqual(symbol, function["c"]["symbol"])

    def test_credential_and_nonce_jni_converters_declare_status_and_output_handle(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        expected_java = {
            "nonce_from_ttlv": {
                "package": "org.kmipkit",
                "class": "NativeExtensionRegistry",
                "method": "nonceFromTtlv",
                "parameters": [
                    {"name": "structure", "type": "long"},
                    {"name": "outNonce", "type": "long[]"},
                ],
                "jniSymbol": "Java_org_kmipkit_NativeExtensionRegistry_nonceFromTtlv",
                "returnType": "int",
                "errorCategories": ["invalid_input", "shared_owner"],
            },
            "credential_from_ttlv": {
                "package": "org.kmipkit",
                "class": "NativeExtensionRegistry",
                "method": "credentialFromTtlv",
                "parameters": [
                    {"name": "structure", "type": "long"},
                    {"name": "outCredential", "type": "long[]"},
                ],
                "jniSymbol": "Java_org_kmipkit_NativeExtensionRegistry_credentialFromTtlv",
                "returnType": "int",
                "errorCategories": ["invalid_input", "shared_owner"],
            },
        }
        functions = {row["id"]: row for row in manifest["functions"]}
        for function_id, expected_descriptor in expected_java.items():
            with self.subTest(function_id=function_id):
                self.assertEqual(expected_descriptor, functions[function_id]["java"])

    def test_credential_envelope_maps_through_all_four_languages(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = {
            row["exportName"]: row
            for row in manifest["publicDeclarations"]
            if row.get("exportName") in CREDENTIAL_EXPORTS
        }
        self.assertEqual(CREDENTIAL_EXPORTS, declarations.keys())

        expected_names = {
            "kmipkit_protocol::Credential": {
                "rust": "kmipkit_protocol::Credential",
                "c": "kmipkit_credential_t",
                "java": "org.kmipkit.credentials.Credential",
                "python": "kmipkit.credentials.Credential",
            },
            "kmipkit_protocol::Credential::try_from_ttlv": {
                "rust": "kmipkit_protocol::Credential::try_from_ttlv",
                "c": "kmipkit_credential_from_ttlv",
                "java": "org.kmipkit.credentials.Credential.fromTtlv",
                "python": "kmipkit.credentials.Credential.from_ttlv",
            },
            "kmipkit_protocol::Credential::credential_type": {
                "rust": "kmipkit_protocol::Credential::credential_type",
                "c": "kmipkit_credential_type",
                "java": "org.kmipkit.credentials.Credential.credentialType",
                "python": "kmipkit.credentials.Credential.credential_type",
            },
            "kmipkit_protocol::Credential::credential_type_raw": {
                "rust": "kmipkit_protocol::Credential::credential_type_raw",
                "c": "kmipkit_credential_type_raw",
                "java": "org.kmipkit.credentials.Credential.credentialTypeRaw",
                "python": "kmipkit.credentials.Credential.credential_type_raw",
            },
            "kmipkit_protocol::Credential::with_ttlv": {
                "rust": "kmipkit_protocol::Credential::with_ttlv",
                "c": "kmipkit_credential_ttlv_view",
                "java": "org.kmipkit.credentials.Credential.withTtlv",
                "python": "kmipkit.credentials.Credential.with_ttlv",
            },
            "kmipkit_protocol::Credential::into_ttlv": {
                "rust": "kmipkit_protocol::Credential::into_ttlv",
                "c": "kmipkit_credential_into_ttlv",
                "java": "org.kmipkit.credentials.Credential.intoTtlv",
                "python": "kmipkit.credentials.Credential.into_ttlv",
            },
        }
        for export_name, declaration in declarations.items():
            with self.subTest(export_name=export_name):
                self.assertEqual("declared", declaration.get("declarationState"))
                self.assertEqual("four-language", declaration.get("parityScope"))
                self.assertTrue(_has_approved_source(declaration))
                self.assertEqual(
                    "specs/008-credentials-attestation/spec.md",
                    declaration["approvalSource"]["path"],
                )
                requirements = set(declaration.get("requirementIds", []))
                self.assertTrue(
                    {
                        "KMIPKIT-0008-FR-003",
                        "KMIPKIT-REQ-SPEC-9.11-001",
                        "KMIPKIT-0043-FR-004",
                        "KMIPKIT-0043-FR-005",
                        "KMIPKIT-0043-FR-010",
                        "KMIPKIT-0043-FR-017",
                        "KMIPKIT-0043-FR-021",
                        "KMIPKIT-0043-FR-022",
                    }
                    <= requirements
                )
                mappings = declaration.get("languageMappings")
                self.assertIsInstance(mappings, dict)
                if isinstance(mappings, dict):
                    self.assertEqual(LANGUAGES, set(mappings))
                    for language, expected_name in expected_names[export_name].items():
                        self.assertEqual(expected_name, mappings[language]["name"])

        credential_type = next(
            row for row in manifest["types"] if row["id"] == "Credential"
        )
        self.assertEqual("kmipkit_credential_t", credential_type["c"]["name"])
        self.assertEqual("kmipkit_credential_release", credential_type["c"]["releaseFunction"])
        for function_id, symbol in (
            ("credential_from_ttlv", "kmipkit_credential_from_ttlv"),
            ("credential_type", "kmipkit_credential_type"),
            ("credential_type_raw", "kmipkit_credential_type_raw"),
            ("credential_ttlv_view", "kmipkit_credential_ttlv_view"),
            ("credential_into_ttlv", "kmipkit_credential_into_ttlv"),
        ):
            function = next(row for row in manifest["functions"] if row["id"] == function_id)
            self.assertEqual(symbol, function["c"]["symbol"])

    def test_attestation_credential_maps_through_all_four_languages(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = {
            row["exportName"]: row
            for row in manifest["publicDeclarations"]
            if row.get("exportName") in ATTESTATION_CREDENTIAL_EXPORTS
        }
        self.assertEqual(ATTESTATION_CREDENTIAL_EXPORTS, declarations.keys())

        expected_names = {
            "kmipkit_protocol::AttestationCredential": {
                "rust": "kmipkit_protocol::AttestationCredential",
                "c": "kmipkit_attestation_credential_t",
                "java": "org.kmipkit.credentials.AttestationCredential",
                "python": "kmipkit.credentials.AttestationCredential",
            },
            "kmipkit_protocol::AttestationCredential::try_from_ttlv": {
                "rust": "kmipkit_protocol::AttestationCredential::try_from_ttlv",
                "c": "kmipkit_attestation_credential_from_ttlv",
                "java": "org.kmipkit.credentials.AttestationCredential.fromTtlv",
                "python": "kmipkit.credentials.AttestationCredential.from_ttlv",
            },
            "kmipkit_protocol::AttestationCredential::with_ttlv": {
                "rust": "kmipkit_protocol::AttestationCredential::with_ttlv",
                "c": "kmipkit_attestation_credential_ttlv_view",
                "java": "org.kmipkit.credentials.AttestationCredential.withTtlv",
                "python": "kmipkit.credentials.AttestationCredential.with_ttlv",
            },
            "kmipkit_protocol::AttestationCredential::into_ttlv": {
                "rust": "kmipkit_protocol::AttestationCredential::into_ttlv",
                "c": "kmipkit_attestation_credential_into_ttlv",
                "java": "org.kmipkit.credentials.AttestationCredential.intoTtlv",
                "python": "kmipkit.credentials.AttestationCredential.into_ttlv",
            },
        }
        required_ids = {
            "KMIPKIT-0008-FR-006",
            "KMIPKIT-REQ-SPEC-9.11-006",
            "KMIPKIT-0043-FR-004",
            "KMIPKIT-0043-FR-005",
            "KMIPKIT-0043-FR-010",
            "KMIPKIT-0043-FR-017",
            "KMIPKIT-0043-FR-021",
            "KMIPKIT-0043-FR-022",
        }
        for export_name, declaration in declarations.items():
            with self.subTest(export_name=export_name):
                self.assertEqual("declared", declaration.get("declarationState"))
                self.assertEqual("four-language", declaration.get("parityScope"))
                self.assertTrue(_has_approved_source(declaration))
                self.assertEqual(
                    "specs/008-credentials-attestation/spec.md",
                    declaration["approvalSource"]["path"],
                )
                self.assertTrue(required_ids <= set(declaration.get("requirementIds", [])))
                mappings = declaration.get("languageMappings")
                self.assertIsInstance(mappings, dict)
                if isinstance(mappings, dict):
                    self.assertEqual(LANGUAGES, set(mappings))
                    for language, expected_name in expected_names[export_name].items():
                        self.assertEqual(expected_name, mappings[language]["name"])

        expected_c_declarations = {
            "kmipkit_attestation_credential_t": ("AttestationCredential:c", "type"),
            "kmipkit_attestation_credential_from_ttlv": (
                "attestation_credential_from_ttlv:c",
                "c-symbol",
            ),
            "kmipkit_attestation_credential_ttlv_view": (
                "attestation_credential_with_ttlv:c",
                "c-symbol",
            ),
            "kmipkit_attestation_credential_into_ttlv": (
                "attestation_credential_into_ttlv:c",
                "c-symbol",
            ),
            "kmipkit_attestation_credential_release": (
                "AttestationCredential:release:c",
                "c-symbol",
            ),
        }
        c_declarations = {
            row["exportName"]: row
            for row in manifest["publicDeclarations"]
            if row.get("exportName") in expected_c_declarations
        }
        self.assertEqual(expected_c_declarations.keys(), c_declarations.keys())
        for export_name, (declaration_id, kind) in expected_c_declarations.items():
            with self.subTest(c_export_name=export_name):
                declaration = c_declarations[export_name]
                self.assertEqual(declaration_id, declaration["declarationId"])
                self.assertEqual("c-kmipkit", declaration["packageId"])
                self.assertEqual(kind, declaration["kind"])
                self.assertEqual("declared", declaration["declarationState"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertTrue(_has_approved_source(declaration))

        credential_type = next(
            (
                row
                for row in manifest["types"]
                if row["id"] == "AttestationCredential"
            ),
            None,
        )
        self.assertIsNotNone(credential_type)
        if credential_type is None:
            return
        self.assertEqual(
            "kmipkit_attestation_credential_t", credential_type["c"]["name"]
        )
        self.assertEqual(
            "kmipkit_attestation_credential_release",
            credential_type["c"]["releaseFunction"],
        )
        for function_id, symbol in (
            (
                "attestation_credential_from_ttlv",
                "kmipkit_attestation_credential_from_ttlv",
            ),
            (
                "attestation_credential_ttlv_view",
                "kmipkit_attestation_credential_ttlv_view",
            ),
            (
                "attestation_credential_into_ttlv",
                "kmipkit_attestation_credential_into_ttlv",
            ),
        ):
            function = next(
                row for row in manifest["functions"] if row["id"] == function_id
            )
            self.assertEqual(symbol, function["c"]["symbol"])

    def test_authentication_and_credential_views_map_through_all_four_languages(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = {
            row["exportName"]: row
            for row in manifest["publicDeclarations"]
            if row.get("exportName") in AUTHENTICATION_EXPORTS
        }
        self.assertEqual(AUTHENTICATION_EXPORTS, declarations.keys())

        expected_names = {
            "kmipkit_protocol::Authentication": {
                "rust": "kmipkit_protocol::Authentication",
                "c": "kmipkit_authentication_t",
                "java": "org.kmipkit.credentials.Authentication",
                "python": "kmipkit.credentials.Authentication",
            },
            "kmipkit_protocol::Authentication::new": {
                "rust": "kmipkit_protocol::Authentication::new",
                "c": "kmipkit_authentication_create",
                "java": "org.kmipkit.credentials.Authentication.create",
                "python": "kmipkit.credentials.Authentication.create",
            },
            "kmipkit_protocol::Authentication::try_from_ttlv": {
                "rust": "kmipkit_protocol::Authentication::try_from_ttlv",
                "c": "kmipkit_authentication_from_ttlv",
                "java": "org.kmipkit.credentials.Authentication.fromTtlv",
                "python": "kmipkit.credentials.Authentication.from_ttlv",
            },
            "kmipkit_protocol::Authentication::credentials": {
                "rust": "kmipkit_protocol::Authentication::credentials",
                "c": "kmipkit_authentication_credential_at",
                "java": "org.kmipkit.credentials.Authentication.credentials",
                "python": "kmipkit.credentials.Authentication.credentials",
            },
            "kmipkit_protocol::Authentication::with_ttlv": {
                "rust": "kmipkit_protocol::Authentication::with_ttlv",
                "c": "kmipkit_authentication_ttlv_view",
                "java": "org.kmipkit.credentials.Authentication.withTtlv",
                "python": "kmipkit.credentials.Authentication.with_ttlv",
            },
            "kmipkit_protocol::Authentication::into_ttlv": {
                "rust": "kmipkit_protocol::Authentication::into_ttlv",
                "c": "kmipkit_authentication_into_ttlv",
                "java": "org.kmipkit.credentials.Authentication.intoTtlv",
                "python": "kmipkit.credentials.Authentication.into_ttlv",
            },
            "kmipkit_protocol::CredentialView": {
                "rust": "kmipkit_protocol::CredentialView",
                "c": "kmipkit_credential_view_t",
                "java": "org.kmipkit.credentials.CredentialView",
                "python": "kmipkit.credentials.CredentialView",
            },
            "kmipkit_protocol::CredentialView::credential_type": {
                "rust": "kmipkit_protocol::CredentialView::credential_type",
                "c": "kmipkit_credential_view_type",
                "java": "org.kmipkit.credentials.CredentialView.credentialType",
                "python": "kmipkit.credentials.CredentialView.credential_type",
            },
            "kmipkit_protocol::CredentialView::credential_type_raw": {
                "rust": "kmipkit_protocol::CredentialView::credential_type_raw",
                "c": "kmipkit_credential_view_type_raw",
                "java": "org.kmipkit.credentials.CredentialView.credentialTypeRaw",
                "python": "kmipkit.credentials.CredentialView.credential_type_raw",
            },
            "kmipkit_protocol::CredentialView::with_ttlv": {
                "rust": "kmipkit_protocol::CredentialView::with_ttlv",
                "c": "kmipkit_credential_view_ttlv_view",
                "java": "org.kmipkit.credentials.CredentialView.withTtlv",
                "python": "kmipkit.credentials.CredentialView.with_ttlv",
            },
        }
        required_ids = {
            "KMIPKIT-0008-FR-001",
            "KMIPKIT-0008-FR-002",
            "KMIPKIT-REQ-SPEC-9.4-001-001",
            "KMIPKIT-REQ-SPEC-9.4-001-002",
            "KMIPKIT-REQ-SPEC-9.4-002",
            "KMIPKIT-0043-FR-004",
            "KMIPKIT-0043-FR-005",
            "KMIPKIT-0043-FR-010",
            "KMIPKIT-0043-FR-017",
            "KMIPKIT-0043-FR-021",
            "KMIPKIT-0043-FR-022",
        }
        for export_name, declaration in declarations.items():
            with self.subTest(export_name=export_name):
                self.assertEqual("declared", declaration.get("declarationState"))
                self.assertEqual("four-language", declaration.get("parityScope"))
                self.assertTrue(_has_approved_source(declaration))
                self.assertEqual(
                    "specs/008-credentials-attestation/spec.md",
                    declaration["approvalSource"]["path"],
                )
                self.assertTrue(required_ids <= set(declaration.get("requirementIds", [])))
                mappings = declaration.get("languageMappings")
                self.assertIsInstance(mappings, dict)
                if isinstance(mappings, dict):
                    self.assertEqual(LANGUAGES, set(mappings))
                    for language, expected_name in expected_names[export_name].items():
                        self.assertEqual(expected_name, mappings[language]["name"])

    def test_device_credential_maps_the_approved_table_412_view_through_all_languages(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = {
            row["exportName"]: row
            for row in manifest["publicDeclarations"]
            if row.get("exportName") in DEVICE_CREDENTIAL_EXPORTS
        }
        self.assertEqual(DEVICE_CREDENTIAL_EXPORTS, declarations.keys())
        expected_names = {
            "kmipkit_protocol::DeviceCredential": {
                "rust": "kmipkit_protocol::DeviceCredential",
                "c": "kmipkit_device_credential_t",
                "java": "org.kmipkit.credentials.DeviceCredential",
                "python": "kmipkit.credentials.DeviceCredential",
            },
            "kmipkit_protocol::DeviceCredential::try_from_ttlv": {
                "rust": "kmipkit_protocol::DeviceCredential::try_from_ttlv",
                "c": "kmipkit_device_credential_from_ttlv",
                "java": "org.kmipkit.credentials.DeviceCredential.fromTtlv",
                "python": "kmipkit.credentials.DeviceCredential.from_ttlv",
            },
            "kmipkit_protocol::DeviceCredential::with_ttlv": {
                "rust": "kmipkit_protocol::DeviceCredential::with_ttlv",
                "c": "kmipkit_device_credential_ttlv_view",
                "java": "org.kmipkit.credentials.DeviceCredential.withTtlv",
                "python": "kmipkit.credentials.DeviceCredential.with_ttlv",
            },
            "kmipkit_protocol::DeviceCredential::into_ttlv": {
                "rust": "kmipkit_protocol::DeviceCredential::into_ttlv",
                "c": "kmipkit_device_credential_into_ttlv",
                "java": "org.kmipkit.credentials.DeviceCredential.intoTtlv",
                "python": "kmipkit.credentials.DeviceCredential.into_ttlv",
            },
        }
        requirements = {
            "KMIPKIT-0008-FR-005",
            "KMIPKIT-REQ-SPEC-9.11-004-001",
            "KMIPKIT-REQ-SPEC-9.11-004-002",
            "KMIPKIT-REQ-SPEC-9.11-004-003",
            "KMIPKIT-0043-FR-004",
            "KMIPKIT-0043-FR-005",
            "KMIPKIT-0043-FR-010",
            "KMIPKIT-0043-FR-017",
            "KMIPKIT-0043-FR-021",
            "KMIPKIT-0043-FR-022",
        }
        for export_name, declaration in declarations.items():
            with self.subTest(export_name=export_name):
                self.assertEqual("declared", declaration.get("declarationState"))
                self.assertEqual("four-language", declaration.get("parityScope"))
                self.assertTrue(_has_approved_source(declaration))
                self.assertEqual("specs/008-credentials-attestation/spec.md", declaration["approvalSource"]["path"])
                self.assertTrue(requirements <= set(declaration.get("requirementIds", [])))
                mappings = declaration.get("languageMappings")
                self.assertIsInstance(mappings, dict)
                self.assertEqual(LANGUAGES, set(mappings))
                for language, expected_name in expected_names[export_name].items():
                    self.assertEqual(expected_name, mappings[language]["name"])

        device_type = next(row for row in manifest["types"] if row["id"] == "DeviceCredential")
        self.assertEqual("kmipkit_device_credential_t", device_type["c"]["name"])
        self.assertEqual("KMIP 2.1 §9.11 Table 412", device_type["normativeSource"])
    def test_attestation_credential_binding_glue_has_exact_owner_exclusions(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        expected = {
            (
                "java-kmipkit",
                "org.kmipkit.NativeExtensionRegistry.ATTESTATION_CREDENTIAL",
            ): "specs/043-public-api-bindings/spec.md",
            (
                "java-kmipkit",
                "org.kmipkit.NativeExtensionRegistry.attestationCredentialFromTtlv",
            ): "specs/043-public-api-bindings/spec.md",
            (
                "java-kmipkit",
                "org.kmipkit.NativeExtensionRegistry.attestationCredentialTtlvView",
            ): "specs/043-public-api-bindings/spec.md",
            (
                "java-kmipkit",
                "org.kmipkit.NativeExtensionRegistry.attestationCredentialIntoTtlv",
            ): "specs/043-public-api-bindings/spec.md",
            (
                "python-kmipkit",
                "kmipkit.credentials.AttestationCredential.__init__",
            ): "specs/008-credentials-attestation/spec.md",
            (
                "python-kmipkit",
                "kmipkit.credentials.AttestationCredential.__slots__",
            ): "specs/008-credentials-attestation/spec.md",
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["approvedExclusions"]
            if (row.get("packageId"), row.get("exportName")) in expected
        }
        self.assertEqual(expected.keys(), exclusions.keys())
        expected_python_attestation_exports = {
            export_name
            for (package_id, export_name) in expected
            if package_id == "python-kmipkit"
        }
        observed_python_attestation_exports = {
            row["exportName"]
            for row in manifest["approvedExclusions"]
            if row.get("packageId") == "python-kmipkit"
            and row.get("exportName", "").startswith("kmipkit.credentials.Attestation")
        }
        self.assertEqual(
            expected_python_attestation_exports,
            observed_python_attestation_exports,
        )
        for identity, owner_path in expected.items():
            with self.subTest(export=identity[1]):
                exclusion = exclusions[identity]
                self.assertEqual(owner_path, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertTrue(exclusion.get("reason"))
                self.assertTrue(exclusion.get("evidence"))
                if identity[0] == "python-kmipkit":
                    self.assertEqual("KMIPKIT-0008", exclusion["candidateSpecId"])

    def test_authentication_binding_glue_has_exact_owner_exclusions(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        java_bridge_exports = {
            "org.kmipkit.NativeExtensionRegistry.AUTHENTICATION",
            "org.kmipkit.NativeExtensionRegistry.CREDENTIAL_VIEW",
            "org.kmipkit.NativeExtensionRegistry.authenticationCreate",
            "org.kmipkit.NativeExtensionRegistry.authenticationFromTtlv",
            "org.kmipkit.NativeExtensionRegistry.authenticationCredentialCount",
            "org.kmipkit.NativeExtensionRegistry.authenticationCredentialAt",
            "org.kmipkit.NativeExtensionRegistry.authenticationTtlvView",
            "org.kmipkit.NativeExtensionRegistry.authenticationIntoTtlv",
            "org.kmipkit.NativeExtensionRegistry.credentialViewType",
            "org.kmipkit.NativeExtensionRegistry.credentialViewTypeRaw",
            "org.kmipkit.NativeExtensionRegistry.credentialViewTtlvView",
        }
        python_glue_exports = {
            "kmipkit.credentials.Authentication.__init__",
            "kmipkit.credentials.Authentication.__slots__",
            "kmipkit.credentials.CredentialView.__init__",
            "kmipkit.credentials.CredentialView.__slots__",
        }
        expected = {
            **{("java-kmipkit", name): "specs/043-public-api-bindings/spec.md"
               for name in java_bridge_exports},
            **{("python-kmipkit", name): "specs/008-credentials-attestation/spec.md"
               for name in python_glue_exports},
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["approvedExclusions"]
            if (row.get("packageId"), row.get("exportName")) in expected
        }
        self.assertEqual(expected.keys(), exclusions.keys())
        for identity, owner_path in expected.items():
            with self.subTest(export=identity[1]):
                exclusion = exclusions[identity]
                self.assertEqual(owner_path, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertTrue(exclusion.get("reason"))
                self.assertTrue(exclusion.get("evidence"))
                if identity[0] == "python-kmipkit":
                    self.assertEqual("KMIPKIT-0008", exclusion["candidateSpecId"])


if __name__ == "__main__":
    unittest.main()
