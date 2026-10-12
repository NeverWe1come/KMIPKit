"""Compiler-backed trait-bound proofs for Rust export auditing."""

from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path
from typing import Any


TEST_ROOT = Path(__file__).parent
FIXTURE = TEST_ROOT / "fixtures" / "rustdoc-v57" / "rustdoc_v57_fixture.json"
AUDITOR = TEST_ROOT.parent / "audit_rust_exports.py"
SPEC = importlib.util.spec_from_file_location("rust_exports", AUDITOR)
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)
CRATE = "rustdoc_v57_fixture"


class RustExportTraitPredicateTests(unittest.TestCase):
    @staticmethod
    def fixture():
        document = json.loads(FIXTURE.read_text(encoding="utf-8"))
        if document.get("format_version") != 57:
            raise AssertionError("fixture must retain authentic rustdoc JSON format 57")
        return {CRATE: document}

    @staticmethod
    def item_named(document: dict[str, Any], name: str) -> dict[str, Any]:
        return next(item for item in document["index"].values() if item.get("name") == name)

    @classmethod
    def alias_target(cls, document: dict[str, Any], name: str) -> dict[str, Any]:
        return cls.item_named(document, name)["inner"]["type_alias"]["type"]

    @classmethod
    def implementation(cls, document: dict[str, Any], owner_name: str,
                       trait_path: str) -> tuple[str, dict[str, Any]]:
        owner = cls.item_named(document, owner_name)
        kind = next(kind for kind in ("struct", "enum", "union") if kind in owner["inner"])
        index = document["index"]
        return next(
            (str(identifier), index[str(identifier)]["inner"]["impl"])
            for identifier in owner["inner"][kind].get("impls", [])
            if (index[str(identifier)].get("inner", {}).get("impl", {}).get("trait") or {})
            .get("path", "").split("::")[-1] == trait_path
        )

    @staticmethod
    def first_trait_bound(implementation: dict[str, Any]) -> dict[str, Any]:
        generics = implementation["generics"]
        for predicate in generics.get("where_predicates", []):
            bound_predicate = predicate.get("bound_predicate")
            if bound_predicate is not None:
                for bound in bound_predicate.get("bounds", []):
                    if bound.get("trait_bound") is not None:
                        return bound["trait_bound"]
        for parameter in generics.get("params", []):
            type_kind = parameter.get("kind", {}).get("type", {})
            for bound in type_kind.get("bounds", []):
                if bound.get("trait_bound") is not None:
                    return bound["trait_bound"]
        raise AssertionError("implementation has no trait bound")

    @staticmethod
    def type_argument(expression: dict[str, Any], index: int = 0) -> dict[str, Any]:
        return expression["resolved_path"]["args"]["angle_bracketed"]["args"][index]["type"]

    def test_clone_bound_uses_the_exact_external_trait_implementation(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        clone_debug = self.alias_target(document, "CloneDebugAlias")
        _, implementation = self.implementation(document, "CloneDebug", "RequiresClone")
        bound = self.first_trait_bound(implementation)
        self.assertEqual(bound["trait"]["path"], "Clone")
        self.assertIs(inventory.check_trait_bound(CRATE, clone_debug, bound, {}), True)

    def test_debug_bound_uses_the_exact_external_trait_implementation(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        clone_debug = self.alias_target(document, "CloneDebugAlias")
        _, implementation = self.implementation(document, "CloneDebug", "RequiresDebug")
        bound = self.first_trait_bound(implementation)
        self.assertEqual(bound["trait"]["path"], "Debug")
        self.assertIs(inventory.check_trait_bound(CRATE, clone_debug, bound, {}), True)

    def test_display_bound_uses_the_exact_external_trait_implementation(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        clone_debug = self.alias_target(document, "CloneDebugAlias")
        _, implementation = self.implementation(document, "CloneDebug", "RequiresDisplay")
        bound = self.first_trait_bound(implementation)
        self.assertEqual(bound["trait"]["path"], "Display")
        self.assertIs(inventory.check_trait_bound(CRATE, clone_debug, bound, {}), True)

    def test_printable_requires_both_copy_and_debug_bounds(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        implementation_id, printable_impl = self.implementation(
            document, "PrintableValue", "Printable"
        )
        bounds = [
            bound["trait_bound"]
            for predicate in printable_impl["generics"]["where_predicates"]
            for bound in predicate["bound_predicate"]["bounds"]
            if bound.get("trait_bound") is not None
        ]
        self.assertEqual(
            {bound["trait"]["path"].split("::")[-1] for bound in bounds},
            {"Copy", "Debug"},
        )

        printable_value = self.alias_target(document, "PrintableValueAlias")
        copy_only = self.alias_target(document, "CopyOnlyAlias")
        debug_only = self.alias_target(document, "DebugOnlyAlias")
        self.assertIs(
            inventory.check_implementation_predicates(
                CRATE, printable_impl, {"T": printable_value}
            ),
            True,
            f"Printable implementation {implementation_id} requires both exact bounds",
        )
        self.assertIs(
            inventory.check_implementation_predicates(CRATE, printable_impl, {"T": copy_only}),
            False,
        )
        self.assertIs(
            inventory.check_implementation_predicates(CRATE, printable_impl, {"T": debug_only}),
            False,
        )

    def test_core_trait_predicate_cycle_resolves_concrete_argument_exactly(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        _, clone_impl = self.implementation(document, "CoreTraitCycle", "Clone")
        _, debug_impl = self.implementation(document, "CoreTraitCycle", "Debug")

        clone_bound = self.first_trait_bound(clone_impl)
        debug_bound = self.first_trait_bound(debug_impl)
        self.assertEqual(clone_bound["trait"]["path"].split("::")[-1], "Debug")
        self.assertEqual(debug_bound["trait"]["path"].split("::")[-1], "Clone")

        both = self.alias_target(document, "CloneDebugAlias")
        clone_only = self.alias_target(document, "CloneOnlyAlias")
        debug_only = self.alias_target(document, "DebugOnlyAlias")
        self.assertIs(
            inventory.check_implementation_predicates(CRATE, clone_impl, {"T": both}),
            True,
        )
        self.assertIs(
            inventory.check_implementation_predicates(CRATE, debug_impl, {"T": both}),
            True,
        )
        self.assertIs(
            inventory.check_implementation_predicates(CRATE, clone_impl, {"T": clone_only}),
            False,
        )
        self.assertIs(
            inventory.check_implementation_predicates(CRATE, debug_impl, {"T": debug_only}),
            False,
        )

    def test_core_trait_impl_cycle_is_traced_and_fails_closed_for_alias_a(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        clone_impl_id, clone_impl = self.implementation(document, "A", "Clone")
        alias_a = self.alias_target(document, "AliasA")
        alias_b = self.alias_target(document, "AliasB")
        alias_a_name = inventory.format_type(CRATE, alias_a)
        alias_b_name = inventory.format_type(CRATE, alias_b)
        trace = [("Clone", alias_a_name)]
        original_check_trait_bound = inventory.check_trait_bound

        def trace_trait_bound(crate, subject, trait_bound, bindings):
            trait_name = trait_bound["trait"]["path"].split("::")[-1]
            trace.append((trait_name, inventory.format_type(crate, subject)))
            return original_check_trait_bound(crate, subject, trait_bound, bindings)

        inventory.check_trait_bound = trace_trait_bound
        alias_a_argument = self.type_argument(alias_a)
        self.assertIs(
            inventory.check_implementation_predicates(
                CRATE, clone_impl, {"T": alias_a_argument}
            ),
            None,
        )
        report = AUDIT.inventory(documents)
        alias_name = "rustdoc_v57_fixture::AliasA"
        self.assertFalse(any(
            entry["exportName"] == f"<{alias_name} as core::clone::Clone>::clone"
            for entry in report["declarations"]
        ))
        self.assertTrue(any(
            issue.get("reason") == "unresolved-rustdoc-implementation-predicate"
            and issue.get("implementationId") == clone_impl_id
            for issue in report["issues"]
            if issue.get("exportName") == alias_name
        ))
        self.assertEqual(
            trace,
            [
                ("Clone", alias_a_name),
                ("Debug", alias_b_name),
                ("Clone", alias_a_name),
            ],
        )

    def test_clone_debug_and_display_proofs_project_exact_alias_paths(self):
        report = AUDIT.inventory(self.fixture())
        names = {entry["exportName"] for entry in report["declarations"]}
        expected = {
            "<rustdoc_v57_fixture::CloneDebugAlias as "
            "rustdoc_v57_fixture::RequiresClone>::clone_bound",
            "<rustdoc_v57_fixture::CloneDebugAlias as "
            "rustdoc_v57_fixture::RequiresDebug>::debug_bound",
            "<rustdoc_v57_fixture::CloneDebugAlias as "
            "rustdoc_v57_fixture::RequiresDisplay>::display_bound",
            "<rustdoc_v57_fixture::CloneDebugAlias as core::clone::CloneToUninit>::clone_to_uninit",
            "<rustdoc_v57_fixture::CloneDebugAlias as alloc::borrow::ToOwned>::to_owned",
            "<rustdoc_v57_fixture::CloneDebugAlias as alloc::string::ToString>::to_string",
        }
        self.assertTrue(expected <= names, sorted(expected - names))

    def test_missing_trait_impl_is_proven_absent_instead_of_unresolved(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        _, clone_impl = self.implementation(document, "CloneDebug", "RequiresClone")
        clone_bound = self.first_trait_bound(clone_impl)
        no_traits = self.alias_target(document, "NoTraitsAlias")

        self.assertIs(
            inventory.check_trait_bound(CRATE, no_traits, clone_bound, {}),
            False,
        )

    def test_conditional_clone_impl_is_proven_after_positive_substitution(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        _, conditional_clone = self.implementation(document, "Conditional", "Clone")
        positive = self.type_argument(self.alias_target(document, "ConditionalCloneDebugAlias"))
        self.assertIs(
            inventory.check_implementation_predicates(CRATE, conditional_clone, {"T": positive}),
            True,
        )

    def test_conditional_clone_impl_is_rejected_after_negative_substitution(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        _, conditional_clone = self.implementation(document, "Conditional", "Clone")
        negative = self.type_argument(self.alias_target(document, "ConditionalNoTraitsAlias"))
        self.assertIs(
            inventory.check_implementation_predicates(CRATE, conditional_clone, {"T": negative}),
            False,
        )

    def test_open_generic_clone_bound_remains_unresolved(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        _, conditional_clone = self.implementation(document, "Conditional", "Clone")
        open_generic = self.type_argument(self.alias_target(document, "ConditionalOpenAlias"))
        self.assertIs(
            inventory.check_implementation_predicates(CRATE, conditional_clone, {"T": open_generic}),
            None,
        )

    def test_recursive_trait_proof_fails_closed_with_exact_impl_issue(self):
        documents = self.fixture()
        document = documents[CRATE]
        inventory = AUDIT._Inventory(documents)
        clone_impl_id, clone_impl = self.implementation(document, "Recursive", "Clone")
        recursive_bound = self.first_trait_bound(clone_impl)
        self.assertEqual(recursive_bound["trait"]["path"], "RecursiveBound")
        self.assertIs(
            inventory.check_implementation_predicates(
                CRATE, clone_impl, {"T": {"primitive": "u8"}}
            ),
            None,
        )

        report = AUDIT.inventory(documents)
        alias = "rustdoc_v57_fixture::RecursiveU8Alias"
        self.assertFalse(any(
            entry["exportName"].startswith("<" + alias + " as ")
            and entry["exportName"].endswith("RequiresClone>::clone_bound")
            for entry in report["declarations"]
        ))
        self.assertTrue(any(
            issue.get("reason") == "unresolved-rustdoc-implementation-predicate"
            and issue.get("implementationId") == clone_impl_id
            for issue in report["issues"]
            if issue.get("exportName") == alias
        ))

    def test_unknown_trait_definition_fails_closed_even_with_familiar_name(self):
        documents = self.fixture()
        document = documents[CRATE]
        implementation_id, implementation = self.implementation(document, "CloneDebug", "RequiresDebug")
        bound = self.first_trait_bound(implementation)
        bound["trait"]["id"] = 999999
        bound["trait"]["path"] = "Clone"
        document["paths"].pop("999999", None)

        inventory = AUDIT._Inventory(documents)
        clone_debug = self.alias_target(document, "CloneDebugAlias")
        self.assertIsNone(inventory.check_trait_bound(CRATE, clone_debug, bound, {}))

        report = AUDIT.inventory(documents)
        self.assertTrue(any(
            issue.get("reason") == "unresolved-rustdoc-implementation-predicate"
            and issue.get("implementationId") == implementation_id
            for issue in report["issues"]
            if issue.get("exportName") == "rustdoc_v57_fixture::CloneDebugAlias"
        ))


if __name__ == "__main__":
    unittest.main()
