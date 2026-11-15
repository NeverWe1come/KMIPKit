"""Source review coverage for the remaining KMIP Specification sections."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


class ProfileSectionsSevenThroughElevenCrosswalkTests(unittest.TestCase):
    """Keep every catalog requirement in §§7–11 tied to a reviewed source disposition."""

    def test_client_correlation_value_requirement_maps_only_request_field(self) -> None:
        catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8")
        )
        requirement_id = "KMIPKIT-REQ-SPEC-9.9-001"
        request_field_id = "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-CLIENT-CORRELATION-VALUE"
        response_field_id = "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-CLIENT-CORRELATION-VALUE"
        requirement = next(
            item for item in catalog["requirements"] if item["requirement_id"] == requirement_id
        )
        response_field = next(
            item for item in catalog["elements"] if item["element_id"] == response_field_id
        )

        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertIn(request_field_id, requirement["element_ids"])
        self.assertNotIn(response_field_id, requirement["element_ids"])
        self.assertNotIn(requirement_id, response_field["requirement_ids"])

    def test_sections_seven_through_eleven_requirement_ids_are_source_reviewed(self) -> None:
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json").read_text(
                encoding="utf-8"
            )
        )
        catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8")
        )

        section_requirements = {
            requirement["requirement_id"]
            for requirement in catalog["requirements"]
            if requirement["requirement_id"].startswith("KMIPKIT-REQ-SPEC-")
            and requirement["requirement_id"].removeprefix("KMIPKIT-REQ-SPEC-").split("-", 1)[0].split(".", 1)[0]
            in {"7", "8", "9", "10", "11"}
        }
        reviewed_ids = {row["requirement_id"] for row in oracle["requirement_crosswalk"]}
        for review in oracle["source_requirement_reviews"]:
            reviewed_ids.update(
                obligation["candidate_requirement_id"]
                for obligation in review["source_obligations"]
                if obligation["candidate_requirement_id"] is not None
            )
            reviewed_ids.update(candidate["requirement_id"] for candidate in review["rejected_candidate_mappings"])
        reviewed_ids.update(
            gap["candidate_requirement_id"]
            for gap in oracle.get("source_only_requirement_gaps", [])
            if gap["candidate_requirement_id"] is not None
        )

        self.assertEqual(
            sorted(section_requirements - reviewed_ids),
            [],
            "every §7–§11 catalog label needs a source-backed mapping, rejection, or explicit gap disposition",
        )


if __name__ == "__main__":
    unittest.main()
