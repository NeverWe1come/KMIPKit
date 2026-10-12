"""Regression cases for Rust lifetime impl identity and predicate applicability."""

from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path


TEST_ROOT = Path(__file__).parent
FIXTURE = TEST_ROOT / "fixtures" / "rustdoc-v57" / "rustdoc_v57_fixture.json"
AUDITOR = TEST_ROOT.parent / "audit_rust_exports.py"


class RustExportLifetimeGapTests(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location("rust_exports_lifetime", AUDITOR)
        self.audit = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.audit)

    @staticmethod
    def fixture_without_static_contract():
        document = json.loads(FIXTURE.read_text(encoding="utf-8"))
        if document.get("format_version") != 57:
            raise AssertionError("fixture must retain authentic rustdoc JSON format 57")
        index = document["index"]
        generic = next(item for item in index.values() if item.get("name") == "Generic")
        retained_impls = []
        for implementation_id in generic["inner"]["struct"]["impls"]:
            implementation = index[str(implementation_id)]["inner"]["impl"]
            if (implementation.get("trait") or {}).get("path") != "StaticContract":
                retained_impls.append(implementation_id)
        generic["inner"]["struct"]["impls"] = retained_impls
        for identifier, item in list(index.items()):
            if "impl" in item.get("inner", {}) \
                    and (item["inner"]["impl"].get("trait") or {}).get("path") == "StaticContract":
                del index[identifier]
        return {"rustdoc_v57_fixture": document}

    def test_alpha_renamed_impl_lifetime_uses_only_public_reexport_path(self):
        documents = self.fixture_without_static_contract()
        document = documents["rustdoc_v57_fixture"]
        index = document["index"]
        use_item = next(
            item for item in index.values()
            if item.get("inner", {}).get("use", {}).get("name") == "AlphaAlias"
        )
        view = index[str(use_item["inner"]["use"]["id"])]
        implementation = index[str(view["inner"]["struct"]["impls"][0])]["inner"]["impl"]
        self.assertEqual(implementation["generics"]["params"][0]["name"], "'b")
        self.assertEqual(
            implementation["for"]["resolved_path"]["args"]["angle_bracketed"]["args"],
            [{"lifetime": "'b"}],
        )

        report = self.audit.inventory(documents)
        names = {declaration["exportName"] for declaration in report["declarations"]}
        view_methods = {
            name for name in names
            if name.endswith("::get")
            and name.rsplit("::", 2)[-2].split("<", 1)[0] in {"AlphaAlias", "AlphaView"}
        }

        self.assertEqual(
            {self.audit._canonical_export_name(name) for name in view_methods},
            {"rustdoc_v57_fixture::AlphaAlias::get"},
        )

    def test_unproven_lifetime_predicate_fails_closed_for_generic_alias(self):
        documents = self.fixture_without_static_contract()
        document = documents["rustdoc_v57_fixture"]
        index = document["index"]
        owner = next(item for item in index.values() if item.get("name") == "LifetimeBound")

        def has_lifetime_predicate(implementation_id):
            implementation = index[str(implementation_id)]["inner"]["impl"]
            return any(
                "lifetime_predicate" in predicate
                for predicate in implementation.get("generics", {}).get("where_predicates", [])
            )

        implementation_id = next(
            str(implementation_id)
            for implementation_id in owner["inner"]["struct"]["impls"]
            if has_lifetime_predicate(implementation_id)
        )
        predicate = index[implementation_id]["inner"]["impl"]["generics"]["where_predicates"][0]
        self.assertEqual(
            predicate["lifetime_predicate"],
            {"lifetime": "'a", "outlives": ["'static"]},
        )

        report = self.audit.inventory(documents)
        names = {declaration["exportName"] for declaration in report["declarations"]}

        self.assertFalse(any(
            "LifetimeBoundAlias" in name and name.endswith("::only_when_static")
            for name in names
        ))
        self.assertTrue(any(
            issue["reason"] == "unresolved-rustdoc-implementation-predicate"
            and issue.get("implementationId") == implementation_id
            and issue.get("exportName") == "rustdoc_v57_fixture::LifetimeBoundAlias"
            for issue in report["issues"]
        ))


if __name__ == "__main__":
    unittest.main()
