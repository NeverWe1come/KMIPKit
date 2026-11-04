"""Pin Revoke's Table 481 values to raw-preserving facade constructors.

Rust exports named RevocationReason constants. C, Java, and Python expose the
same numeric values through their generic raw-code constructors; they do not
export corresponding named-constant symbols. The KMIP 2.1 source is §11.48,
Table 481, used by the Revoke request in §6.1.44, Table 295.
"""

from __future__ import annotations

import json
import re
import unittest
from html import unescape
from pathlib import Path

from tools.api_manifest.tests.test_revoke_parity_manifest import (
    API_REQUIREMENTS,
    EXPECTED_MAPPINGS,
    OWNER_REQUIREMENTS,
    PARITY_EVIDENCE,
)

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
REVOKE_OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-REVOKE"

EXPECTED_REASONS = (
    ("rust-observed-1e1ac0cb1af003e02031", "UNSPECIFIED", 1),
    ("rust-observed-4fc6839fcfaf6b620fce", "KEY_COMPROMISE", 2),
    ("rust-observed-e39b91b528939a846927", "CA_COMPROMISE", 3),
    ("rust-observed-5317149641363798a749", "AFFILIATION_CHANGED", 4),
    ("rust-observed-0b8042942eef85b8e885", "SUPERSEDED", 5),
    ("rust-observed-d7de5b7ed68e562e4989", "CESSATION_OF_OPERATION", 6),
    ("rust-observed-acb02c73ead8cb7d3a4f", "PRIVILEGE_WITHDRAWN", 7),
)


class T102RevocationReasonValueMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }

    def test_exact_table_481_constants_map_to_raw_code_factories(self) -> None:
        oasis_names = (
            "Unspecified",
            "Key Compromise",
            "CA Compromise",
            "Affiliation Changed",
            "Superseded",
            "Cessation of Operation",
            "Privilege Withdrawn",
        )
        oasis_source = (
            ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
        ).read_text(encoding="cp1252")
        table = oasis_source.split(
            'name="_Toc57115828">11.48 Revocation Reason Code', 1
        )[1].split("Table </a>481: Revocation Reason Code Enumeration", 1)[0]
        table_text = unescape(re.sub(r"<[^>]+>", " ", table))
        table_text = re.sub(r"\s+", " ", table_text)
        table_values = re.findall(
            r"Unspecified|Key Compromise|CA Compromise|Affiliation Changed|"
            r"Superseded|Cessation of Operation|Privilege Withdrawn|[0-9]{8}",
            table_text,
        )
        expected_table_values = [
            value
            for code in range(1, 8)
            for value in (oasis_names[code - 1], f"{code:08X}")
        ]
        self.assertEqual(expected_table_values, table_values)

        rust_source = (ROOT / "crates/kmipkit-protocol/src/revoke.rs").read_text(
            encoding="utf-8"
        )
        associated_values = dict(
            re.findall(
                r"pub const ([A-Z_]+): Self = Self\(([^)]+)\);",
                rust_source,
            )
        )
        helper_values = {
            name: int(value)
            for name, value in re.findall(
                r"const ([A-Z_]+): u32 = (\d+);", rust_source
            )
        }

        reason_type = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-protocol"
            and row["exportName"] == "kmipkit_protocol::RevocationReason"
        )
        self.assertEqual("verified", reason_type["declarationState"])
        self.assertEqual(REVOKE_OPERATION_ID, reason_type["capabilityId"])
        self.assertEqual(OWNER_SPEC, reason_type["approvalSource"]["path"])
        self.assertEqual("Approved", reason_type["approvalSource"]["status"])

        operation = next(
            row
            for row in self.manifest["operations"]
            if row["operationId"] == REVOKE_OPERATION_ID
        )
        self.assertEqual("6.1.44", operation["section"])
        self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])

        expected_requirements = OWNER_REQUIREMENTS | API_REQUIREMENTS
        factory_mappings = {
            "c": ("c-kmipkit", "kmipkit_revocation_reason_create"),
            "java": (
                "java-kmipkit",
                "org.kmipkit.extensions.RevocationReason.create",
            ),
            "python": (
                "python-kmipkit",
                "kmipkit.extensions.RevocationReason.create",
            ),
        }
        for language, (_, name) in factory_mappings.items():
            self.assertIn(name, EXPECTED_MAPPINGS[language])

        self.assertEqual(7, len(EXPECTED_REASONS))
        for (declaration_id, constant_name, raw_code), oasis_name in zip(
            EXPECTED_REASONS, oasis_names
        ):
            rust_expression = associated_values[constant_name]
            rust_code = (
                helper_values[rust_expression]
                if rust_expression in helper_values
                else int(rust_expression)
            )
            self.assertEqual(raw_code, rust_code, constant_name)
            self.assertEqual(oasis_name, oasis_names[raw_code - 1])
            expected_export = f"kmipkit_protocol::RevocationReason::{constant_name}"
            expected_mappings = {
                "rust": {
                    "packageId": "rust-kmipkit-protocol",
                    "name": expected_export,
                },
                **{
                    language: {"packageId": package_id, "name": name}
                    for language, (package_id, name) in factory_mappings.items()
                },
            }
            row = self.declarations[declaration_id]

            with self.subTest(constant=constant_name):
                self.assertEqual("rust-kmipkit-protocol", row["packageId"])
                self.assertEqual("associated-item", row["kind"])
                self.assertEqual(expected_export, row["exportName"])
                self.assertEqual("verified", row["declarationState"])
                self.assertNotIn("candidateSpecId", row)
                self.assertNotIn("blockedReason", row)
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(REVOKE_OPERATION_ID, row["capabilityId"])
                self.assertEqual(expected_requirements, set(row["requirementIds"]))
                self.assertEqual(reason_type["approvalSource"], row["approvalSource"])
                self.assertEqual(
                    set(PARITY_EVIDENCE), set(row["parityEvidenceIds"])
                )
                self.assertEqual(expected_mappings, row["languageMappings"])

        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(648, blocked)

    def test_open_reason_constructors_keep_unknown_raw_values(self) -> None:
        checks = {
            "crates/kmipkit-protocol/tests/unit/revoke_operation_tests.rs": (
                "request_emits_table_295_fields_in_order_and_preserves_open_reason",
            ),
            "bindings/c/tests/client_abi_consumer.c": (
                "kmipkit_client_abi_c_consumer_build_revoke_request",
                "kmipkit_revocation_reason_create(",
                "UINT32_MAX",
            ),
            "bindings/java/src/test/java/org/kmipkit/extensions/RevokeMappingTest.java": (
                "requestRetainsUnsignedUnknownReasonAndNestedAuditMessage",
                "0xffff_ffffL",
            ),
            "bindings/python/tests/test_revoke_operation.py": (
                "test_revoke_request_retains_unsigned_unknown_reason_and_nested_message",
                "0xFFFF_FFFF",
            ),
        }
        self.assertEqual(set(checks), set(PARITY_EVIDENCE) & set(checks))
        for relative_path, markers in checks.items():
            content = (ROOT / relative_path).read_text(encoding="utf-8")
            for marker in markers:
                with self.subTest(path=relative_path, marker=marker):
                    self.assertIn(marker, content)

    def test_traceability_records_distinct_java_and_python_factory_paths(self) -> None:
        traceability = (ROOT / "specs/043-public-api-bindings/traceability.md").read_text(
            encoding="utf-8"
        )
        section = traceability.split(
            "### T102 - Revoke reason-value/factory mappings", 1
        )[1].split("\n### ", 1)[0]

        self.assertIn("Java uses `org.kmipkit.extensions.RevocationReason.create`", section)
        self.assertIn("Python uses `kmipkit.extensions.RevocationReason.create`", section)


if __name__ == "__main__":
    unittest.main()
