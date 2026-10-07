"""Contract checks for shared extension fixtures; KMIP 2.1 §11.56 Extensions range, ADR-0010."""

from __future__ import annotations

import json
import copy
import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[3]
FIXTURE_DIRECTORY = ROOT / "tests" / "fixtures" / "extensions"
FIXTURE_SCHEMA = FIXTURE_DIRECTORY / "fixture.schema.json"
FIXTURE_CORPUS = FIXTURE_DIRECTORY / "cases.json"
PUBLIC_API_MANIFEST = ROOT / "specification" / "api" / "public-api.json"
ADAPTER_FIXTURE_GENERATOR = ROOT / "tools" / "extension_fixtures" / "generate.py"
GENERATED_C_FIXTURES = ROOT / "bindings" / "c" / "tests" / "extension_fixtures.generated.h"
GENERATED_JAVA_FIXTURES = (
    ROOT
    / "bindings"
    / "java"
    / "src"
    / "test"
    / "java"
    / "org"
    / "kmipkit"
    / "SharedExtensionFixtures.java"
)
GENERATED_RUST_FIXTURES = (
    ROOT
    / "crates"
    / "kmipkit-client"
    / "tests"
    / "fixtures"
    / "extensions"
    / "extension_fixtures.generated.rs"
)
ADAPTERS = ("rust", "c", "java", "python")

_GENERATOR_SPEC = importlib.util.spec_from_file_location(
    "extension_fixture_generator", ADAPTER_FIXTURE_GENERATOR
)
if _GENERATOR_SPEC is None or _GENERATOR_SPEC.loader is None:
    raise RuntimeError("cannot load the extension fixture generator")
_GENERATOR = importlib.util.module_from_spec(_GENERATOR_SPEC)
sys.modules[_GENERATOR_SPEC.name] = _GENERATOR
_GENERATOR_SPEC.loader.exec_module(_GENERATOR)


class ExtensionFixtureCorpusTests(unittest.TestCase):
    def read_json(self, path: Path, label: str) -> Any:
        self.assertTrue(path.is_file(), f"required extension {label} is missing")
        try:
            return json.loads(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, json.JSONDecodeError):
            self.fail(f"extension {label} is not valid UTF-8 JSON")

    def corpus_and_schema(self) -> tuple[dict[str, Any], dict[str, Any]]:
        schema = self.read_json(FIXTURE_SCHEMA, "fixture schema")
        corpus = self.read_json(FIXTURE_CORPUS, "fixture corpus")
        self.assertIsInstance(schema, dict, "fixture schema must be a JSON object")
        self.assertIsInstance(corpus, dict, "fixture corpus must be a JSON object")
        return corpus, schema

    def case(self, corpus: dict[str, Any], case_id: str) -> dict[str, Any]:
        for fixture in corpus["cases"]:
            if fixture["id"] == case_id:
                return fixture
        self.fail("required extension fixture case is missing")

    @staticmethod
    def definition_matches(definition: dict[str, Any], payload: dict[str, Any]) -> bool:
        current: dict[str, Any] = payload
        for tag in definition["discriminator"]["path"]:
            children = current.get("children", [])
            matches = [item for item in children if item.get("tag") == tag]
            if len(matches) != 1:
                return False
            current = matches[0]
        discriminator = definition["discriminator"]
        return (
            current.get("type") == discriminator["itemType"]
            and current.get("value") == discriminator["value"]
        )

    def matching_definition_ids(
        self, definitions: list[dict[str, Any]], extension: dict[str, Any]
    ) -> list[str]:
        matches = [
            definition["id"]
            for definition in definitions
            if definition["vendorIdentification"] == extension["vendorIdentification"]
            and self.definition_matches(definition, extension["payload"])
        ]
        return sorted(matches)

    def test_corpus_matches_the_fixture_schema_and_embedded_schemas_are_valid(self) -> None:
        corpus, schema = self.corpus_and_schema()
        try:
            Draft202012Validator.check_schema(schema)
            corpus_is_valid = Draft202012Validator(schema).is_valid(corpus)
        except Exception:
            self.fail("extension fixture schema is not valid Draft 2020-12 JSON Schema")
        self.assertTrue(corpus_is_valid, "extension fixture corpus violates its schema")

        for definition in corpus["definitions"]:
            try:
                Draft202012Validator.check_schema(definition["payloadSchema"])
            except Exception:
                self.fail("definition payload schema is not valid Draft 2020-12 JSON Schema")

    def test_generated_adapter_descriptors_cover_the_shared_corpus(self) -> None:
        self.assertTrue(ADAPTER_FIXTURE_GENERATOR.is_file(), "shared adapter fixture generator is missing")
        result = subprocess.run(
            [sys.executable, "-B", str(ADAPTER_FIXTURE_GENERATOR), "--check"],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

        corpus, _ = self.corpus_and_schema()
        case_ids = [fixture["id"] for fixture in corpus["cases"]]
        for generated_path in (GENERATED_C_FIXTURES, GENERATED_JAVA_FIXTURES, GENERATED_RUST_FIXTURES):
            self.assertTrue(generated_path.is_file(), f"generated adapter fixtures are missing: {generated_path}")
            generated = generated_path.read_text(encoding="utf-8")
            for case_id in case_ids:
                with self.subTest(path=generated_path.name, fixture=case_id):
                    self.assertIn(case_id, generated)

    def test_rust_fixture_strings_escape_quotes_and_controls(self) -> None:
        literal = _GENERATOR._rust_string('quote" slash\\ line\n tab\t café \x01')
        self.assertEqual(literal, '"quote\\" slash\\\\ line\\n tab\\t café \\u{1}"')

    def test_generated_rust_descriptors_use_clippy_readable_fields_and_literals(self) -> None:
        api = self.read_json(PUBLIC_API_MANIFEST, "public API manifest")
        corpus, _ = self.corpus_and_schema()
        rust = _GENERATOR.render(api, corpus)[GENERATED_RUST_FIXTURES]

        self.assertIn("pub kind: ItemType", rust)
        self.assertNotIn("pub item_type:", rust)
        self.assertIn("0x0054_0010", rust)
        self.assertNotIn("0x540010", rust)
        self.assertIn("number: 8_675_309", rust)
        self.assertNotIn("number: 8675309", rust)
        self.assertIn("number: 4_294_967_295", rust)
        self.assertNotRegex(rust, r"\b0x[0-9A-Fa-f]{5,}\b")
        self.assertNotRegex(rust, r"\b\d{5,}\b")
        self.assertEqual(_GENERATOR._rust_integer_literal(-1_234_567_890), "-1_234_567_890")

    def test_generated_adapter_fixtures_keep_criticality_and_outbound_order(self) -> None:
        api = self.read_json(PUBLIC_API_MANIFEST, "public API manifest")
        corpus, _ = self.corpus_and_schema()
        outputs = _GENERATOR.render(api, corpus)

        for path, generated in outputs.items():
            with self.subTest(adapter_fixture=path.name):
                self.assertIn("outbound.validated", generated)
                self.assertIn("synthetic-secret-bearing", generated)
                self.assertIn("valid-recognized", generated)
        self.assertIn("critical: false", outputs[GENERATED_RUST_FIXTURES])
        self.assertIn("critical: true", outputs[GENERATED_RUST_FIXTURES])

    def test_fixture_generator_rejects_invalid_outbound_attachment_records(self) -> None:
        api = self.read_json(PUBLIC_API_MANIFEST, "public API manifest")
        corpus, _ = self.corpus_and_schema()

        unknown_case = copy.deepcopy(corpus)
        outbound = unknown_case["cases"][-1]["expected"]["outboundRequest"]
        outbound["attachments"][0]["fixtureId"] = "missing-fixture"
        with self.assertRaises(_GENERATOR.FixtureError):
            _GENERATOR.render(api, unknown_case)

        invalid_criticality = copy.deepcopy(corpus)
        outbound = invalid_criticality["cases"][-1]["expected"]["outboundRequest"]
        outbound["attachments"][0]["criticalityIndicator"] = "false"
        with self.assertRaises(_GENERATOR.FixtureError):
            _GENERATOR.render(api, invalid_criticality)

    def test_fixture_generator_fails_closed_and_tracks_manifest_metadata(self) -> None:
        api = self.read_json(PUBLIC_API_MANIFEST, "public API manifest")
        corpus, _ = self.corpus_and_schema()

        unsupported_item = copy.deepcopy(corpus)
        unsupported_item["cases"][0]["extension"]["payload"]["children"][0]["type"] = "Boolean"
        with self.assertRaises(_GENERATOR.FixtureError):
            _GENERATOR.render(api, unsupported_item)

        unsupported_schema = copy.deepcopy(corpus)
        unsupported_schema["definitions"][1]["payloadSchema"]["properties"]["children"]["minItems"] = 1
        with self.assertRaises(_GENERATOR.FixtureError):
            _GENERATOR.render(api, unsupported_schema)

        malformed_mapping = copy.deepcopy(api)
        next(
            function for function in malformed_mapping["functions"]
            if function["id"] == "client_extension_registry_inspect"
        )["c"].pop("symbol")
        with self.assertRaises(_GENERATOR.FixtureError):
            _GENERATOR.render(malformed_mapping, corpus)

        changed_mapping = copy.deepcopy(api)
        next(
            function for function in changed_mapping["functions"]
            if function["id"] == "client_extension_registry_inspect"
        )["python"]["function"] = "changed_inspection_mapping"
        current_outputs = _GENERATOR.render(api, corpus)
        changed_outputs = _GENERATOR.render(changed_mapping, corpus)
        self.assertNotEqual(
            current_outputs[GENERATED_C_FIXTURES],
            changed_outputs[GENERATED_C_FIXTURES],
            "generated descriptors must fingerprint the supported manifest mappings",
        )

    def test_fixture_matrix_uses_stable_adapter_neutral_outcomes(self) -> None:
        corpus, schema = self.corpus_and_schema()
        self.assertTrue(Draft202012Validator(schema).is_valid(corpus), "fixture corpus violates its schema")

        actual_cases = {fixture["id"]: fixture for fixture in corpus["cases"]}
        expected_outcomes = {
            "valid-recognized": "recognized",
            "invalid-schema": "unrecognized.schema_invalid",
            "multiply-matching": "unrecognized.ambiguous",
            "unknown-preserved": "unrecognized.no_match",
            "synthetic-secret-bearing": "recognized",
        }
        self.assertTrue(
            set(expected_outcomes).issubset(actual_cases),
            "fixture corpus is missing a required scenario",
        )
        for case_id, outcome_code in expected_outcomes.items():
            fixture = actual_cases[case_id]
            expected = fixture["expected"]
            self.assertEqual(expected["outcomeCode"], outcome_code, "fixture has an unstable outcome code")
            self.assertTrue(expected["genericPayloadPreserved"], "fixture must preserve the generic payload")
            self.assertTrue(
                expected["preservedPayload"] == fixture["extension"]["payload"],
                "preserved payload must retain its exact tree and array order",
            )

    def test_each_expected_match_set_is_independent_of_definition_order(self) -> None:
        corpus, schema = self.corpus_and_schema()
        self.assertTrue(Draft202012Validator(schema).is_valid(corpus), "fixture corpus violates its schema")

        definitions = corpus["definitions"]
        for fixture in corpus["cases"]:
            expected_ids = fixture["expected"]["matchedDefinitionIds"]
            actual_ids = self.matching_definition_ids(definitions, fixture["extension"])
            reversed_ids = self.matching_definition_ids(list(reversed(definitions)), fixture["extension"])
            self.assertEqual(actual_ids, expected_ids, "fixture match set does not match its expected outcome")
            self.assertEqual(reversed_ids, expected_ids, "fixture outcome depends on registration order")

    def test_ambiguous_case_has_two_distinct_matching_paths_and_two_valid_schemas(self) -> None:
        corpus, schema = self.corpus_and_schema()
        self.assertTrue(Draft202012Validator(schema).is_valid(corpus), "fixture corpus violates its schema")

        fixture = self.case(corpus, "multiply-matching")
        expected_ids = fixture["expected"]["matchedDefinitionIds"]
        definitions_by_id = {definition["id"]: definition for definition in corpus["definitions"]}
        candidates = [definitions_by_id[definition_id] for definition_id in expected_ids]
        self.assertEqual(len(candidates), 2, "ambiguous outcome must name exactly two definitions")
        first, second = candidates
        self.assertNotEqual(
            first["discriminator"]["path"],
            second["discriminator"]["path"],
            "ambiguous definitions must use distinct discriminator paths",
        )
        self.assertEqual(
            self.matching_definition_ids(candidates, fixture["extension"]),
            expected_ids,
            "both distinct discriminator paths must match the same payload",
        )
        payload = fixture["extension"]["payload"]
        for definition in candidates:
            self.assertTrue(
                Draft202012Validator(definition["payloadSchema"]).is_valid(payload),
                "ambiguous payload must satisfy each matching definition schema",
            )
        self.assertFalse(fixture["expected"]["typedValueAvailable"], "ambiguous outcome cannot expose a typed value")

    def test_secret_fixture_is_synthetic_and_diagnostics_exclude_payload_values(self) -> None:
        corpus, schema = self.corpus_and_schema()
        self.assertTrue(Draft202012Validator(schema).is_valid(corpus), "fixture corpus violates its schema")

        fixture = self.case(corpus, "synthetic-secret-bearing")
        secret_value = fixture["secretSentinel"]
        self.assertTrue(secret_value.startswith("SYNTHETIC-ONLY-"), "secret fixture value must be a synthetic sentinel")
        expected = fixture["expected"]
        diagnostic_metadata = {key: value for key, value in expected.items() if key != "preservedPayload"}
        serialized_metadata = json.dumps(diagnostic_metadata, sort_keys=True)
        self.assertTrue(secret_value not in serialized_metadata, "secret sentinel leaked into outcome metadata")
        self.assertEqual(expected["outcomeCode"], "recognized", "secret-bearing value must validate for outbound use")
        self.assertTrue(expected["typedValueAvailable"], "outbound request must use a validated typed value")
        definition = next(
            item for item in corpus["definitions"] if item["id"] == "known.alpha"
        )
        self.assertTrue(
            Draft202012Validator(definition["payloadSchema"]).is_valid(fixture["extension"]["payload"]),
            "secret-bearing outbound payload must satisfy its definition schema",
        )
        outbound_request = expected.get("outboundRequest")
        self.assertTrue(
            isinstance(outbound_request, dict),
            "secret-bearing case must declare a normalized outbound request expectation",
        )
        if not isinstance(outbound_request, dict):
            return
        self.assertEqual(
            outbound_request["outcomeCode"],
            "outbound.validated",
            "outbound outcome code must be stable",
        )
        attachments = outbound_request["attachments"]
        self.assertEqual(
            [attachment["fixtureId"] for attachment in attachments],
            ["synthetic-secret-bearing", "valid-recognized"],
            "outbound extension attachment order must remain explicit",
        )
        self.assertEqual(
            [attachment["criticalityIndicator"] for attachment in attachments],
            [False, True],
            "each outbound extension must carry its explicit criticality indicator",
        )
        cases_by_id = {item["id"]: item for item in corpus["cases"]}
        for attachment in attachments:
            attached_fixture = cases_by_id[attachment["fixtureId"]]
            self.assertEqual(
                attached_fixture["expected"]["outcomeCode"],
                "recognized",
                "outbound attachment must reference a recognized fixture",
            )
            self.assertTrue(
                attached_fixture["expected"]["typedValueAvailable"],
                "outbound attachment must reference a validated typed value",
            )
        self.assertTrue(expected["diagnostics"]["redacted"], "secret-bearing case must require redacted diagnostics")
        self.assertFalse(
            expected["diagnostics"]["containsPayloadValues"],
            "secret-bearing case diagnostics must not contain payload values",
        )

    def test_limit_cases_cover_parity_boundaries_and_raised_defaults(self) -> None:
        corpus, schema = self.corpus_and_schema()
        self.assertTrue(Draft202012Validator(schema).is_valid(corpus), "fixture corpus violates its schema")

        manifest = self.read_json(PUBLIC_API_MANIFEST, "public API manifest")
        manifest_limits = manifest["limits"]
        fixture_limits = corpus.get("limitCases")
        self.assertIsInstance(fixture_limits, list, "fixture corpus must include resource-limit parity cases")
        limits_by_id = {limit["id"]: limit for limit in fixture_limits}
        self.assertEqual(
            list(limits_by_id),
            [limit["id"] for limit in manifest_limits],
            "limit parity fixtures must follow manifest order",
        )

        for manifest_limit in manifest_limits:
            fixture = limits_by_id[manifest_limit["id"]]
            with self.subTest(limit=manifest_limit["id"]):
                self.assertEqual(fixture["default"], manifest_limit["default"])
                self.assertEqual(fixture["hardMaximum"], manifest_limit["hardMaximum"])
                probes = {probe["profile"]: probe for probe in fixture["probes"]}
                required_profiles = {"default", "lowered", "hard", "over-hard"}
                if manifest_limit["default"] < manifest_limit["hardMaximum"]:
                    required_profiles.add("raised-default")
                self.assertEqual(set(probes), required_profiles)

                expected_values = {
                    "default": manifest_limit["default"],
                    "lowered": max(1, manifest_limit["default"] // 2),
                    "hard": manifest_limit["hardMaximum"],
                    "over-hard": manifest_limit["hardMaximum"] + 1,
                }
                if "raised-default" in required_profiles:
                    expected_values["raised-default"] = manifest_limit["default"] + 1

                for profile, probe in probes.items():
                    expected = "resource_limit" if profile == "over-hard" else "accepted"
                    self.assertEqual(probe["value"], expected_values[profile])
                    self.assertEqual(probe["expectedOutcome"], expected)
                    self.assertEqual(
                        probe["expectedByAdapter"],
                        {adapter: expected for adapter in ADAPTERS},
                        "all adapters must agree on each normalized limit outcome",
                    )

    def test_algorithm_work_cases_declare_fixed_cross_adapter_bounds(self) -> None:
        corpus, schema = self.corpus_and_schema()
        self.assertTrue(Draft202012Validator(schema).is_valid(corpus), "fixture corpus violates its schema")

        work_cases = corpus.get("algorithmWorkCases")
        self.assertIsInstance(work_cases, list, "fixture corpus must declare fixed algorithm work cases")
        by_id = {case["id"]: case for case in work_cases}
        expected_bounds = {
            "client-definitions": {"hardMaximum": 1024},
            "discriminator-path-depth": {"hardMaximum": 64},
            "payload-items-including-root": {"hardMaximum": 100000},
            "path-step-tag-comparisons": {"hardMaximum": 35},
            "total-lookup-comparisons": {"hardMaximum": 4194304},
            "schema-tag-comparisons-per-item": {"hardMaximum": 13},
            "allowed-enum-comparisons-per-item": {"hardMaximum": 13},
            "order-constraint-position-pass": {"passesPerPayload": 1},
            "order-constraint-edge-check": {"checksPerEdge": 1},
        }
        self.assertEqual(set(by_id), set(expected_bounds))

        for case_id, bounds in expected_bounds.items():
            case = by_id[case_id]
            with self.subTest(work_case=case_id):
                for field, value in bounds.items():
                    self.assertEqual(case[field], value)
                expected = case["expectedOutcome"]
                self.assertIn(expected, {"accepted", "resource_limit"})
                self.assertEqual(
                    case["expectedByAdapter"],
                    {adapter: expected for adapter in ADAPTERS},
                    "all adapters must agree on fixed algorithm work outcomes",
                )


if __name__ == "__main__":
    unittest.main()
