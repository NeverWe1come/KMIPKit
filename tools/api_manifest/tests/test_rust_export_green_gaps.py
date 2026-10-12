"""Regression cases for exact Rust declaration identities and applicability."""

from __future__ import annotations

import copy
import importlib.util
import json
import unittest
from pathlib import Path


TEST_ROOT = Path(__file__).parent
FIXTURE = TEST_ROOT / "fixtures" / "rustdoc-v57" / "rustdoc_v57_fixture.json"
AUDITOR = TEST_ROOT.parent / "audit_rust_exports.py"


class RustExportGreenGapTests(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location("rust_exports", AUDITOR)
        self.audit = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.audit)

    @staticmethod
    def fixture():
        document = json.loads(FIXTURE.read_text(encoding="utf-8"))
        if document.get("format_version") != 57:
            raise AssertionError("fixture must retain authentic rustdoc JSON format 57")
        return {"rustdoc_v57_fixture": document}

    def test_empty_trait_impl_includes_default_associated_constant_and_type(self):
        report = self.audit.inventory(self.fixture())
        names = {declaration["exportName"] for declaration in report["declarations"]}

        self.assertIn(
            "<rustdoc_v57_fixture::Service as rustdoc_v57_fixture::DefaultMembers>::LIMIT",
            names,
        )
        self.assertIn(
            "<rustdoc_v57_fixture::Service as rustdoc_v57_fixture::DefaultMembers>::Output",
            names,
        )

    def test_trait_identity_resolves_rustdoc_ids_to_canonical_paths(self):
        original = self.fixture()
        changed_id = copy.deepcopy(original)
        document = changed_id["rustdoc_v57_fixture"]
        generic = next(item for item in document["index"].values() if item.get("name") == "Generic")
        contract_impl = next(
            document["index"][str(impl_id)]["inner"]["impl"]
            for impl_id in generic["inner"]["struct"]["impls"]
            if (document["index"][str(impl_id)]["inner"]["impl"].get("trait") or {}).get("path") == "Contract"
        )
        predicate_trait = (
            contract_impl["generics"]["where_predicates"][0]["bound_predicate"]["bounds"][0]
            ["trait_bound"]["trait"]
        )
        document["paths"]["999999"] = {"path": ["core", "marker", "Copy"], "kind": "trait"}
        predicate_trait["id"] = 999999

        def generic_identity(documents):
            report = self.audit.inventory(documents)
            declaration = next(
                entry for entry in report["declarations"]
                if entry["exportName"] ==
                "<rustdoc_v57_fixture::Generic<T> as rustdoc_v57_fixture::Contract<T>>::required"
            )
            return declaration["identity"]

        self.assertEqual(generic_identity(original), generic_identity(changed_id))

    def test_non_applicable_generic_alias_does_not_receive_copy_bounded_impl(self):
        report = self.audit.inventory(self.fixture())
        export_name = (
            "<rustdoc_v57_fixture::GenericDerivedAlias as "
            "rustdoc_v57_fixture::Contract<rustdoc_v57_fixture::Derived>>::required"
        )
        self.assertNotIn(export_name, {entry["exportName"] for entry in report["declarations"]})

    def test_composite_generic_arguments_are_compared_recursively(self):
        report = self.audit.inventory(self.fixture())
        names = {declaration["exportName"] for declaration in report["declarations"]}

        self.assertIn("rustdoc_v57_fixture::GenericU8TupleAlias::only_tuple_u8", names)
        self.assertNotIn("rustdoc_v57_fixture::GenericU16TupleAlias::only_tuple_u8", names)

    def test_public_trait_inside_private_module_is_not_reported_as_exported(self):
        report = self.audit.inventory(self.fixture())
        names = {declaration["exportName"] for declaration in report["declarations"]}

        self.assertNotIn(
            "<rustdoc_v57_fixture::Service as "
            "rustdoc_v57_fixture::HiddenContract>::hidden_method",
            names,
        )

    def test_comparison_normalizes_lifetime_arguments_and_field_separators(self):
        observed = {
            "crates": ["demo"],
            "complete": True,
            "declarations": [
                {"packageId": "rust-demo", "exportName": "demo::View<'a>::result", "kind": "associated-item"},
                {"packageId": "rust-demo", "exportName": "demo::Alias<'a>::result", "kind": "associated-item"},
                {"packageId": "rust-demo", "exportName": "demo::Record::field", "kind": "field"},
            ],
            "issues": [],
        }
        manifest = {
            "publicDeclarations": [
                {"packageId": "rust-demo", "exportName": "demo::View::result", "kind": "associated-item", "declarationState": "blocked"},
                {"packageId": "rust-demo", "exportName": "demo::Alias<'_>::result", "kind": "associated-item", "declarationState": "blocked"},
                {"packageId": "rust-demo", "exportName": "demo::Record.field", "kind": "field", "declarationState": "blocked"},
            ],
            "approvedExclusions": [],
        }

        report = self.audit.compare(observed, manifest)

        self.assertTrue(report["matchesManifest"])
        self.assertEqual(report["counts"]["matched"], 3)
        self.assertEqual(report["notObserved"], [])

    def test_path_normalization_keeps_type_const_and_static_lifetime_arguments(self):
        normalize = getattr(self.audit, "_canonical_export_name", lambda value: value)

        self.assertEqual(normalize("demo::Borrowed<'a>::get"), normalize("demo::Borrowed<'_>::get"))
        self.assertNotEqual(normalize("demo::Value<u8>::new"), normalize("demo::Value<u16>::new"))
        self.assertNotEqual(normalize("demo::Array<3>::new"), normalize("demo::Array<4>::new"))
        self.assertNotEqual(normalize("demo::Borrowed<'static>::get"), normalize("demo::Borrowed<'a>::get"))


if __name__ == "__main__":
    unittest.main()
