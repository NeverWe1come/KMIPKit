"""Shared assertions for Rust fluent builder manifest contracts."""

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


def assert_fluent_builder_manifest_contract(
    test_case: unittest.TestCase,
    *,
    builder_name: str,
    operation_name: str,
    methods: set[str],
    additional_exports: set[str] | None = None,
) -> None:
    manifest = json.loads(
        (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
    )
    declarations = {
        row["exportName"]: row
        for row in manifest["publicDeclarations"]
        if row.get("packageId") == "rust-kmipkit-client"
    }
    modules = ("kmipkit_client", "kmipkit_client::fluent")
    expected = set(additional_exports or ())
    expected.update(f"{module}::{builder_name}" for module in modules)
    expected.update(
        f"{module}::RequestBuilder<'a>::{operation_name}" for module in modules
    )
    for module in modules:
        expected.update(
            f"{module}::{builder_name}<'a>::{method}" for method in methods
        )

    test_case.assertTrue(expected <= declarations.keys())
    for export_name in expected:
        with test_case.subTest(export=export_name):
            declaration = declarations[export_name]
            test_case.assertEqual("blocked", declaration["declarationState"])
            test_case.assertEqual("four-language", declaration["parityScope"])
            test_case.assertEqual("KMIPKIT-0093", declaration["candidateSpecId"])
            test_case.assertEqual(["KMIPKIT-0093-FR-011"], declaration["requirementIds"])
            test_case.assertEqual("public_binding_surface", declaration["capabilityId"])
            test_case.assertNotIn("languageMappings", declaration)
            test_case.assertNotIn("approvalSource", declaration)
