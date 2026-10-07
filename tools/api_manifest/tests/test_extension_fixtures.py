"""Contract checks for the shared, adapter-neutral extension fixture corpus."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[3]
FIXTURE_DIRECTORY = ROOT / "tests" / "fixtures" / "extensions"
FIXTURE_SCHEMA = FIXTURE_DIRECTORY / "fixture.schema.json"
FIXTURE_CORPUS = FIXTURE_DIRECTORY / "cases.json"


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

    def test_fixture_matrix_uses_stable_adapter_neutral_outcomes(self) -> None:
        corpus, schema = self.corpus_and_schema()
        self.assertTrue(Draft202012Validator(schema).is_valid(corpus), "fixture corpus violates its schema")

        actual_cases = {fixture["id"]: fixture for fixture in corpus["cases"]}
        expected_outcomes = {
            "valid-recognized": "recognized",
            "invalid-schema": "unrecognized.schema_invalid",
            "multiply-matching": "unrecognized.ambiguous",
            "unknown-preserved": "unrecognized.no_match",
            "synthetic-secret-bearing": "unrecognized.schema_invalid",
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
        self.assertTrue(expected["diagnostics"]["redacted"], "secret-bearing case must require redacted diagnostics")
        self.assertFalse(
            expected["diagnostics"]["containsPayloadValues"],
            "secret-bearing case diagnostics must not contain payload values",
        )


if __name__ == "__main__":
    unittest.main()
