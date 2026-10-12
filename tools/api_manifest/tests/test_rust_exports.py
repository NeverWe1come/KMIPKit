"""Reachability and exact manifest comparison for Rust declaration inventories."""

from __future__ import annotations

import copy
import importlib.util
import json
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "audit_rust_exports.py"


def item(identifier, name, inner, visibility="public", file="crates/sample/src/lib.rs"):
    return {
        "id": identifier, "crate_id": 0, "name": name,
        "span": {"filename": file, "begin": [identifier + 1, 1], "end": [identifier + 1, 8]},
        "visibility": visibility, "inner": inner, "attrs": [],
    }


def document(name, items, root_items, paths=None):
    root = item(0, name, {"module": {"is_crate": True, "items": root_items}})
    return {
        "root": 0, "index": {str(i["id"]): i for i in [root, *items]},
        "paths": paths or {}, "format_version": 57, "includes_private": True,
        "target": {"triple": "x86_64-pc-windows-msvc"},
    }


def structure(identifier, name, fields=(), impls=(), visibility="public"):
    return item(identifier, name, {"struct": {
        "kind": {"plain": {"fields": list(fields)}}, "impls": list(impls),
    }}, visibility)


class RustExportAuditTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(MODULE_PATH.is_file(), "the reproducible Rust export auditor is missing")
        spec = importlib.util.spec_from_file_location("rust_exports", MODULE_PATH)
        self.audit = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.audit)

    def inventory(self, documents):
        return self.audit.inventory(documents)

    def names(self, report):
        return {record["exportName"] for record in report["declarations"]}

    def test_private_module_items_are_visible_only_through_public_reexports(self):
        doc = document("sample", [
            item(1, "hidden", {"module": {"items": [2, 3]}}, "default"),
            structure(2, "Visible", impls=[6]), structure(3, "Unreachable"),
            item(4, None, {"use": {"id": 2, "name": "Alias", "is_glob": False}}),
            item(5, "new", {"function": {}}),
            item(6, None, {"impl": {"trait": None, "items": [5]}}),
        ], [1, 4])
        report = self.inventory({"sample": doc})
        self.assertEqual(self.names(report), {"sample::Alias", "sample::Alias::new"})
        self.assertFalse(report["issues"])

    def test_public_modules_and_aliases_keep_exact_paths_without_duplicate_records(self):
        doc = document("sample", [
            item(1, "model", {"module": {"items": [2]}}), structure(2, "Value"),
            item(3, None, {"use": {"id": 2, "name": "Value", "is_glob": False}}),
            item(4, None, {"use": {"id": 2, "name": "Value", "is_glob": False}}),
        ], [1, 3, 4])
        report = self.inventory({"sample": doc})
        self.assertEqual(self.names(report), {"sample::model", "sample::model::Value", "sample::Value"})
        self.assertEqual(len(report["declarations"]), 3)
        aliases = [r for r in report["declarations"] if r["kind"] == "type"]
        self.assertEqual(aliases[0]["identity"], aliases[1]["identity"])
        self.assertEqual(aliases[0]["aliases"], ["sample::Value", "sample::model::Value"])

    def test_glob_reexports_expand_public_children_and_stop_module_cycles(self):
        doc = document("sample", [
            item(1, "hidden", {"module": {"items": [2, 3, 5]}}, "default"),
            structure(2, "Public"), structure(3, "Private", visibility="default"),
            item(4, None, {"use": {"id": 1, "name": "hidden", "is_glob": True}}),
            item(5, None, {"use": {"id": 0, "name": "again", "is_glob": True}}),
        ], [4])
        report = self.inventory({"sample": doc})
        self.assertEqual(self.names(report), {"sample::Public"})

    def test_fields_use_owner_paths_and_keep_only_public_struct_fields(self):
        doc = document("sample", [
            structure(1, "Record", fields=[2, 3]),
            item(2, "public", {"struct_field": {}}),
            item(3, "private", {"struct_field": {}}, "default"),
            item(4, "Tuple", {"struct": {"kind": {"tuple": [5, None, 6]}, "impls": []}}),
            item(5, "0", {"struct_field": {}}), item(6, "2", {"struct_field": {}}),
        ], [1, 4])
        self.assertEqual(self.names(self.inventory({"sample": doc})), {
            "sample::Record", "sample::Record::public", "sample::Tuple", "sample::Tuple::0", "sample::Tuple::2",
        })

    def test_enum_variants_and_payload_fields_have_default_public_visibility(self):
        doc = document("sample", [
            item(1, "Choice", {"enum": {"variants": [2, 4], "impls": []}}),
            item(2, "Tuple", {"variant": {"kind": {"tuple": [3]}}}, "default"),
            item(3, "0", {"struct_field": {}}, "default"),
            item(4, "Record", {"variant": {"kind": {"struct": {"fields": [5]}}}}, "default"),
            item(5, "value", {"struct_field": {}}, "default"),
        ], [1])
        self.assertEqual(self.names(self.inventory({"sample": doc})), {
            "sample::Choice", "sample::Choice::Tuple", "sample::Choice::Tuple::0",
            "sample::Choice::Record", "sample::Choice::Record::value",
        })

    def test_trait_items_and_public_inherent_members_are_associated_declarations(self):
        doc = document("sample", [
            item(1, "Contract", {"trait": {"items": [2, 3, 4]}}),
            item(2, "run", {"function": {}}, "default"),
            item(3, "Result", {"assoc_type": {}}, "default"),
            item(4, "LIMIT", {"assoc_const": {}}, "default"),
            structure(5, "Value", impls=[6]),
            item(6, None, {"impl": {"trait": None, "items": [7, 8, 9]}}),
            item(7, "create", {"function": {}}),
            item(8, "secret", {"function": {}}, "default"),
            item(9, "MAX", {"assoc_const": {}}),
        ], [1, 5])
        report = self.inventory({"sample": doc})
        associated = {r["exportName"] for r in report["declarations"] if r["kind"] == "associated-item"}
        self.assertEqual(associated, {
            "sample::Contract::run", "sample::Contract::Result", "sample::Contract::LIMIT",
            "sample::Value::create", "sample::Value::MAX",
        })

    def test_trait_implementation_members_are_explicitly_reported_as_unaudited(self):
        doc = document("sample", [
            structure(1, "Value", impls=[2]),
            item(2, None, {"impl": {"trait": {"path": "Debug", "id": 10}, "items": [3]}}),
            item(3, "fmt", {"function": {}}, "default"),
        ], [1])
        report = self.inventory({"sample": doc})
        self.assertEqual(self.names(report), {"sample::Value"})
        self.assertFalse(report["complete"])
        self.assertEqual(report["issues"][0]["reason"], "trait-implementation-members-not-audited")

    def test_exported_macros_and_hidden_public_items_are_kept(self):
        hidden = structure(2, "HiddenDocumentation")
        hidden["attrs"] = ["doc(hidden)"]
        doc = document("sample", [item(1, "make", {"macro": "macro_rules! make"}), hidden], [1, 2])
        report = self.inventory({"sample": doc})
        self.assertEqual(self.names(report), {"sample::make", "sample::HiddenDocumentation"})
        self.assertEqual(next(r["kind"] for r in report["declarations"] if r["exportName"].endswith("make")), "macro")

    def test_external_reexports_resolve_to_origin_document_and_retain_alias_locations(self):
        origin = document("origin", [
            item(1, "hidden", {"module": {"items": [2]}}, "default"),
            structure(2, "Value", impls=[3]),
            item(3, None, {"impl": {"trait": None, "items": [4]}}),
            item(4, "new", {"function": {}}),
            item(5, None, {"use": {"id": 2, "name": "Value", "is_glob": False}}),
        ], [1, 5], {"2": {"path": ["origin", "hidden", "Value"], "kind": "struct"}})
        facade = document("facade", [
            item(1, None, {"use": {"id": 99, "name": "Alias", "is_glob": False}}, file="crates/facade/src/lib.rs"),
        ], [1], {"99": {"path": ["origin", "hidden", "Value"], "kind": "struct"}})
        report = self.inventory({"facade": facade, "origin": origin})
        self.assertTrue({"facade::Alias", "facade::Alias::new"}.issubset(self.names(report)))
        alias = next(r for r in report["declarations"] if r["exportName"] == "facade::Alias")
        self.assertEqual(alias["location"]["path"], "crates/sample/src/lib.rs")
        self.assertEqual(alias["exportLocations"][0]["path"], "crates/facade/src/lib.rs")

    def test_external_glob_module_reexports_follow_origin_public_module_contents(self):
        origin = document("origin", [
            item(1, "api", {"module": {"items": [2]}}), structure(2, "Value"),
        ], [1], {"1": {"path": ["origin", "api"], "kind": "module"}})
        facade = document("facade", [
            item(1, None, {"use": {"id": 99, "name": "api", "is_glob": True}}),
        ], [1], {"99": {"path": ["origin", "api"], "kind": "module"}})
        self.assertIn("facade::Value", self.names(self.inventory({"origin": origin, "facade": facade})))

    def test_unresolved_exports_fail_closed_instead_of_manufacturing_a_declaration(self):
        doc = document("sample", [
            item(1, None, {"use": {"id": 99, "name": "External", "is_glob": False}}),
        ], [1], {"99": {"path": ["unavailable", "Value"], "kind": "struct"}})
        report = self.inventory({"sample": doc})
        self.assertFalse(report["complete"])
        self.assertEqual(report["declarations"], [])
        self.assertEqual(report["issues"][0]["exportName"], "sample::External")
        self.assertEqual(report["issues"][0]["reason"], "unresolved-reexport")

    def test_format_and_private_documentation_must_match_the_pinned_contract(self):
        for key, value in [("format_version", 58), ("includes_private", False)]:
            doc = document("sample", [], [])
            doc[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.inventory({"sample": doc})

    def test_report_is_deterministic_when_index_documents_and_manifest_records_are_reordered(self):
        docs = {"sample": document("sample", [structure(1, "Value")], [1])}
        manifest = {"publicDeclarations": [
            {"packageId": "rust-sample", "kind": "type", "exportName": "sample::Value", "declarationState": "blocked"},
            {"packageId": "rust-sample", "kind": "function", "exportName": "sample::old", "declarationState": "blocked"},
        ], "approvedExclusions": []}
        baseline = self.audit.compare(self.inventory(docs), manifest)
        reversed_docs = copy.deepcopy(docs)
        reversed_docs["sample"]["index"] = dict(reversed(list(docs["sample"]["index"].items())))
        manifest["publicDeclarations"].reverse()
        self.assertEqual(json.dumps(baseline, sort_keys=True), json.dumps(self.audit.compare(self.inventory(reversed_docs), manifest), sort_keys=True))

    def test_comparison_reports_exact_missing_stale_and_wrong_kind_names(self):
        doc = document("sample", [structure(1, "Value"), item(2, "run", {"function": {}})], [1, 2])
        manifest = {"publicDeclarations": [
            {"packageId": "rust-sample", "kind": "function", "exportName": "sample::Value", "declarationState": "blocked"},
            {"packageId": "rust-sample", "kind": "type", "exportName": "sample::Old", "declarationState": "declared"},
            {"packageId": "c-kmipkit", "kind": "type", "exportName": "ignored", "declarationState": "declared"},
        ], "approvedExclusions": []}
        result = self.audit.compare(self.inventory({"sample": doc}), manifest)
        self.assertEqual([r["exportName"] for r in result["missingFromManifest"]], ["sample::run"])
        self.assertEqual([r["exportName"] for r in result["notObserved"]], ["sample::Old"])
        self.assertEqual([r["exportName"] for r in result["kindMismatches"]], ["sample::Value"])
        self.assertEqual(result["matched"], [])
        self.assertFalse(result["matchesManifest"])

    def test_exact_approved_exclusions_resolve_an_observed_path(self):
        manifest = {"publicDeclarations": [], "approvedExclusions": [{"packageId": "rust-sample", "exportName": "sample::Value"}]}
        result = self.audit.compare(self.inventory({"sample": document("sample", [structure(1, "Value")], [1])}), manifest)
        self.assertTrue(result["matchesManifest"])
        self.assertEqual([r["exportName"] for r in result["excluded"]], ["sample::Value"])

    def test_duplicate_manifest_names_are_rejected_even_if_declaration_ids_differ(self):
        row = {"packageId": "rust-sample", "kind": "type", "exportName": "sample::Value", "declarationState": "blocked"}
        with self.assertRaisesRegex(ValueError, "duplicate"):
            self.audit.compare(self.inventory({"sample": document("sample", [], [])}), {"publicDeclarations": [row, dict(row)]})

    def test_missing_exact_toolchain_is_rejected_before_rustup_can_download_it(self):
        require_toolchain = getattr(self.audit, "require_installed_toolchain", None)
        self.assertIsNotNone(require_toolchain, "the runner needs an offline installed-toolchain guard")
        for installed in ["", "1.94-x86_64-pc-windows-msvc\n", "stable-x86_64-unknown-linux-gnu (default)\n"]:
            with self.subTest(installed=installed), self.assertRaisesRegex(ValueError, "already installed"):
                require_toolchain(installed)
        require_toolchain("1.94.0-x86_64-pc-windows-msvc\n")
        require_toolchain("stable-x86_64-unknown-linux-gnu (default)\n1.94.0-x86_64-unknown-linux-gnu\n")

    def test_macro_generated_members_with_equal_source_spans_have_distinct_owner_identities(self):
        first = item(3, "from_raw", {"function": {}})
        second = item(6, "from_raw", {"function": {}})
        second["span"] = copy.deepcopy(first["span"])
        doc = document("sample", [
            structure(1, "First", impls=[2]),
            item(2, None, {"impl": {"trait": None, "items": [3]}}), first,
            structure(4, "Second", impls=[5]),
            item(5, None, {"impl": {"trait": None, "items": [6]}}), second,
        ], [1, 4])
        members = [r for r in self.inventory({"sample": doc})["declarations"] if r["kind"] == "associated-item"]
        self.assertNotEqual(members[0]["identity"], members[1]["identity"])
        self.assertEqual(members[0]["aliases"], ["sample::First::from_raw"])
        self.assertEqual(members[1]["aliases"], ["sample::Second::from_raw"])


if __name__ == "__main__":
    unittest.main()
