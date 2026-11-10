from __future__ import annotations

import hashlib
import html
import json
import re
import tempfile
import unittest
from html.parser import HTMLParser
from pathlib import Path

from interop.shared.tools.aggregate_matrix import aggregate_results, generate_html, generate_markdown


ROOT = Path(__file__).resolve().parents[3]
CATALOG = ROOT / "interop/shared/catalog/kmipkit-1.0-test-catalog.json"
NORMATIVE = ROOT / "specification/catalog/kmip-2.1.json"
OASIS_PATHS = {
    "2.0": ROOT / "specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html",
    "2.1": ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html",
}


class SectionReader(HTMLParser):
    """Independent test reader for section headings in pinned OASIS HTML."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.heading_tag: str | None = None
        self.heading_text: list[str] = []
        self.by_section: dict[str, str] = {}

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if re.fullmatch(r"h[1-6]", tag):
            self.heading_tag = tag
            self.heading_text = []

    def handle_data(self, data: str) -> None:
        if self.heading_tag is not None:
            self.heading_text.append(data)

    def handle_endtag(self, tag: str) -> None:
        if self.heading_tag != tag:
            return
        text = " ".join("".join(self.heading_text).split())
        match = re.match(r"(\d+(?:\.\d+)*)(?:\s+|$)(.*)", text)
        if match:
            self.by_section[match.group(1)] = match.group(2).strip()
        self.heading_tag = None
        self.heading_text = []


def oasis_sections() -> dict[str, dict[str, str]]:
    sections = {}
    for version, path in OASIS_PATHS.items():
        reader = SectionReader()
        reader.feed(path.read_bytes().decode("cp1252"))
        sections[version] = reader.by_section
    return sections


def write_result(directory: Path, name: str, payload: dict) -> Path:
    path = directory / f"{name}.json"
    path.write_text(json.dumps(payload), encoding="utf-8")
    return path


def result_payload(
    *,
    run_id: str,
    language: str,
    protocol_version: str,
    transport: str,
    encoding: str,
    api_level: str,
    status: str,
    configuration_fingerprint: str = "a" * 64,
    library_commit: str = "abcdef0",
    catalog_sha256: str = "c" * 64,
    server_version: str = "kms-1",
) -> dict:
    return {
        "schema_version": "2.0",
        "run_id": run_id,
        "language": language,
        "server": {
            "id": "test-kms",
            "product": "KMIP test service",
            "version": server_version,
            "configuration_fingerprint": configuration_fingerprint,
        },
        "protocol_version": protocol_version,
        "catalog_sha256": catalog_sha256,
        "execution": {
            "transport": transport,
            "encoding": encoding,
            "api_level": api_level,
            "library_commit": library_commit,
            "artifact_sha256": ("b" if language == "rust" else "d") * 64,
        },
        "summary": {
            "total": 1,
            "passed": int(status == "passed"),
            "failed": int(status == "failed"),
            "unsupported": int(status == "server_unsupported"),
            "policy_restricted": int(status == "policy_restricted"),
            "not_run": int(status == "not_run"),
        },
        "cases": [{
            "case_id": "test-get",
            "status": status,
            "library_correctness": "passed" if status == "passed" else "not_checked",
            "server_compatibility": "supported" if status == "passed" else "unknown",
            "parity": "not_compared",
        }],
    }


class VersionedContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        cls.normative = json.loads(NORMATIVE.read_text(encoding="utf-8"))
        cls.sections = oasis_sections()

    def test_pinned_oasis_sources_match_the_recorded_digests(self) -> None:
        expected = {
            "2.0": "41346355fc966e555dbcbae68b4d73421b0da0f32fb1a86fc8e3cb5fd35bb4a2",
            "2.1": "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf",
        }
        for version, path in OASIS_PATHS.items():
            with self.subTest(version=version):
                self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), expected[version])

    def test_every_operation_case_has_oasis_backed_version_and_section(self) -> None:
        elements = self.normative["elements"]
        cases = [c for c in self.catalog["test_cases"] if c["category"] not in {"version_boundaries", "normative_inventory"}]
        version_by_case = {}
        for case in cases:
            operation = case["operation"]
            if case["category"] in {"managed_objects", "attributes", "enums_and_bitmasks"}:
                continue
            section_prefix = "6.2." if case.get("requires", {}).get("workflow_type") == "server_to_client" else "6.1."
            for version in ("2.0", "2.1"):
                matches = [
                    section for section, title in self.sections[version].items()
                    if section.startswith(section_prefix) and title.casefold() == operation.casefold()
                ]
                if matches:
                    version_by_case.setdefault(case["case_id"], []).append(version)
                    expected_ref = {
                        "document": f"OASIS KMIP {version} Specification",
                        "section": matches[0],
                    }
                    self.assertIn(expected_ref, case["normative_refs"], case["case_id"])
            self.assertEqual(
                sorted(version_by_case.get(case["case_id"], [])),
                sorted(case["execution"]["protocol_versions"]),
                case["case_id"],
            )
            for ref in case["normative_refs"]:
                version = "2.0" if "2.0" in ref["document"] else "2.1"
                self.assertIn(ref["section"], self.sections[version], case["case_id"])

        for operation in ("Get Constraints", "Ping", "Process", "Query Asynchronous Requests", "Set Constraints", "Set Defaults"):
            case = next(c for c in cases if c["operation"] == operation)
            with self.subTest(operation=operation):
                self.assertEqual(["2.1"], case["execution"]["protocol_versions"])

    def test_attribute_and_local_value_cases_use_versioned_source_sections(self) -> None:
        elements_by_kind_name = {
            (item["kind"], item["name"]): item for item in self.normative["elements"]
            if item.get("kind") in {"attribute", "enumeration", "bitmask", "object_type"}
        }
        for case in self.catalog["test_cases"]:
            requirement = case.get("requires", {})
            if case["category"] == "attributes":
                key = ("attribute", requirement["attribute"])
            elif case["category"] == "enums_and_bitmasks":
                key = ("enumeration", requirement["enum_type"]) if "enum_type" in requirement else ("bitmask", requirement["bitmask_type"])
            elif case["category"] == "managed_objects":
                key = ("object_type", requirement["object_type"])
            else:
                continue
            element = elements_by_kind_name[key]
            section21 = next(ref["section"] for ref in element["source_refs"] if ref["source_id"] == "KMIPKIT-SRC-spec")
            title = self.sections["2.1"][section21]
            family = section21.rsplit(".", 1)[0] + "."
            sections_by_version = {
                version: [section for section, candidate in self.sections[version].items() if section.startswith(family) and candidate == title]
                for version in ("2.0", "2.1")
            }
            expected_versions = [version for version, matches in sections_by_version.items() if matches]
            with self.subTest(case=case["case_id"]):
                self.assertEqual(expected_versions, case["execution"]["protocol_versions"])
                for version in expected_versions:
                    self.assertIn({
                        "document": f"OASIS KMIP {version} Specification",
                        "section": sections_by_version[version][0],
                    }, case["normative_refs"])

        vendor_attribute = next(c for c in self.catalog["test_cases"] if c["case_id"] == "test-attr-vendor-attribute")
        self.assertEqual(["2.0", "2.1"], vendor_attribute["execution"]["protocol_versions"])
        self.assertIn({"document": "OASIS KMIP 2.0 Specification", "section": "4.53"}, vendor_attribute["normative_refs"])
        for enum_name in ("Validation Authority Type", "Validation Type"):
            case = next(c for c in self.catalog["test_cases"] if c["requires"].get("enum_type") == enum_name)
            self.assertEqual(["2.0", "2.1"], case["execution"]["protocol_versions"])

    def test_bounded_negative_cases_publish_allowed_result_reasons(self) -> None:
        reasons = {
            case["case_id"]: case["requires"].get("expected_result_reasons")
            for case in self.catalog["test_cases"] if case["category"] == "negative_and_boundary"
        }
        self.assertEqual([7, 10], reasons["test-neg-encrypt-preactive-key"])
        self.assertEqual([67], reasons["test-neg-destroy-active-without-revoke"])
        try:
            import jsonschema
        except ImportError:
            self.skipTest("jsonschema dependency is not installed")
        schema = json.loads((ROOT / "interop/shared/schemas/test-catalog.schema.json").read_text(encoding="utf-8"))
        for value in ([], [7, 7], [4294967296]):
            invalid = json.loads(json.dumps(self.catalog))
            negative = next(case for case in invalid["test_cases"] if case["case_id"] == "test-neg-encrypt-preactive-key")
            negative["requires"]["expected_result_reasons"] = value
            with self.subTest(value=value), self.assertRaises(jsonschema.ValidationError):
                jsonschema.validate(invalid, schema)

    def test_every_catalog_reference_resolves_in_its_pinned_document(self) -> None:
        for case in self.catalog["test_cases"]:
            for ref in case["normative_refs"]:
                version = "2.0" if "2.0" in ref["document"] else "2.1"
                with self.subTest(case=case["case_id"], version=version, section=ref["section"]):
                    self.assertIn(ref["section"], self.sections[version])
        for case in self.catalog["test_cases"]:
            if case["category"] == "version_boundaries":
                self.assertEqual(["2.0"], case["execution"]["protocol_versions"])
                self.assertIn({"document": "OASIS KMIP 2.0 Specification", "section": "6.1"}, case["normative_refs"])

    def test_connection_contract_requires_stable_server_configuration_fingerprint(self) -> None:
        try:
            import jsonschema
        except ImportError:
            self.skipTest("jsonschema dependency is not installed")
        schema = json.loads((ROOT / "interop/shared/schemas/server-connection.schema.json").read_text(encoding="utf-8"))
        config = json.loads((ROOT / "interop/shared/profiles/connection.example.json").read_text(encoding="utf-8"))
        jsonschema.validate(config, schema)
        for value in (None, "not-a-fingerprint"):
            invalid = json.loads(json.dumps(config))
            if value is None:
                invalid.pop("server_configuration_fingerprint")
            else:
                invalid["server_configuration_fingerprint"] = value
            with self.subTest(value=value), self.assertRaises(jsonschema.ValidationError):
                jsonschema.validate(invalid, schema)

    def test_workflow_runs_the_contract_checks_for_this_feature_branch(self) -> None:
        workflow = (ROOT / ".github/workflows/interop-common.yml").read_text(encoding="utf-8")
        self.assertIn("feature/KMIPKIT-0088-interop-contract-matrix", workflow)
        self.assertIn("specs/088-interop-contract-version-matrix/**", workflow)
        self.assertIn("generate_test_catalog.py --check", workflow)
        self.assertIn("unittest discover -s interop/shared/tests", workflow)

    def test_aggregator_preserves_same_language_runs_and_execution_axes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            payloads = [
                result_payload(run_id="rust-ttlv", language="rust", protocol_version="2.1", transport="raw_tls", encoding="TTLV", api_level="typed-protocol", status="passed"),
                result_payload(run_id="rust-json", language="rust", protocol_version="2.1", transport="https", encoding="JSON", api_level="high-level", status="passed"),
                result_payload(run_id="rust-json", language="rust", protocol_version="2.1", transport="https", encoding="JSON", api_level="high-level", status="failed"),
                result_payload(run_id="python-xml", language="python", protocol_version="2.1", transport="https", encoding="XML", api_level="typed-protocol", status="server_unsupported"),
                result_payload(run_id="rust-20", language="rust", protocol_version="2.0", transport="raw_tls", encoding="TTLV", api_level="typed-protocol", status="passed"),
            ]
            payloads[0]["cases"].append({
                "case_id": "test-ping", "status": "not_run", "library_correctness": "not_checked",
                "server_compatibility": "unknown", "parity": "not_compared",
                "request_body": "secret request bytes", "diagnostic": "private key material",
            })
            payloads[1]["cases"][0]["request_body"] = "secret response bytes"
            payloads[1]["cases"].append({
                "case_id": "test-ping", "status": "not_run", "library_correctness": "not_checked",
                "server_compatibility": "unknown", "parity": "not_compared",
            })
            payloads[0]["summary"]["raw_response_body"] = "secret body in malformed summary"
            paths = [write_result(root, value["run_id"], value) for value in payloads]
            matrix = aggregate_results(paths)

        self.assertEqual(2, len(matrix["matrices"]))
        matrix21 = next(item for item in matrix["matrices"] if item["protocol_version"] == "2.1")
        self.assertEqual("COMPARABLE", matrix21["comparability"])
        self.assertEqual({"rust-ttlv", "rust-json", "rust-json#2", "python-xml"}, {run["result_key"] for run in matrix21["runs"]})
        row = next(row for row in matrix21["cases"] if row["case_id"] == "test-get")
        self.assertEqual({"rust-ttlv", "rust-json", "rust-json#2", "python-xml"}, set(row["results"]))
        self.assertEqual("PARITY_MISMATCH", row["status"])
        pending = next(row for row in matrix21["cases"] if row["case_id"] == "test-ping")
        self.assertEqual({"NOT_RUN"}, {item["status"] for item in pending["results"].values()})
        matrix20 = next(item for item in matrix["matrices"] if item["protocol_version"] == "2.0")
        self.assertEqual("NOT_COMPARED", matrix20["cases"][0]["status"])

        markdown = generate_markdown(matrix)
        html_output = generate_html(matrix)
        json_output = json.dumps(matrix)
        for label in ("rust-ttlv", "rust-json", "rust-json#2", "python-xml", "raw_tls", "JSON", "XML", "Library correctness", "Server compatibility", "not_run"):
            self.assertIn(label, markdown)
            self.assertIn(label, html.unescape(html_output))
        self.assertIn("rust-ttlv", json_output)
        self.assertIn('"api_level": "high-level"', json_output)
        self.assertNotIn("secret request bytes", json_output)
        self.assertNotIn("secret response bytes", json_output)
        self.assertNotIn("private key material", json_output)
        self.assertNotIn("secret body in malformed summary", json_output)

    def test_aggregator_requires_matching_base_evidence_but_ignores_route_differences(self) -> None:
        baseline = result_payload(run_id="rust", language="rust", protocol_version="2.1", transport="raw_tls", encoding="TTLV", api_level="typed-protocol", status="passed")
        variant = result_payload(run_id="python", language="python", protocol_version="2.1", transport="https", encoding="JSON", api_level="high-level", status="passed")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = [write_result(root, "a", baseline), write_result(root, "b", variant)]
            matrix = aggregate_results(paths)
            self.assertEqual("COMPARABLE", matrix["matrices"][0]["comparability"])

            for field, replacement in (("library_commit", "1234567"), ("catalog_sha256", "e" * 64), ("configuration_fingerprint", "f" * 64), ("version", "kms-2")):
                incompatible = json.loads(json.dumps(variant))
                if field == "library_commit":
                    incompatible["execution"][field] = replacement
                elif field == "catalog_sha256":
                    incompatible[field] = replacement
                else:
                    incompatible["server"][field] = replacement
                write_result(root, "b", incompatible)
                matrix = aggregate_results(paths)
                with self.subTest(field=field):
                    self.assertEqual("INCOMPARABLE", matrix["matrices"][0]["comparability"])

            for field, replacement in (("id", "other-kms"), ("protocol_version", "2.0")):
                separated = json.loads(json.dumps(variant))
                if field == "protocol_version":
                    separated[field] = replacement
                else:
                    separated["server"][field] = replacement
                write_result(root, "b", separated)
                matrix = aggregate_results(paths)
                with self.subTest(separate=field):
                    self.assertEqual(2, len(matrix["matrices"]))

    def test_aggregator_does_not_claim_parity_for_incompatible_or_unverified_evidence(self) -> None:
        first = result_payload(run_id="rust", language="rust", protocol_version="2.1", transport="raw_tls", encoding="TTLV", api_level="typed-protocol", status="passed")
        second = result_payload(run_id="python", language="python", protocol_version="2.1", transport="https", encoding="JSON", api_level="high-level", status="failed", configuration_fingerprint="f" * 64)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = [write_result(root, "first", first), write_result(root, "second", second)]
            matrix = aggregate_results(paths)
            row = matrix["matrices"][0]["cases"][0]
            self.assertEqual("INCOMPARABLE", matrix["matrices"][0]["comparability"])
            self.assertEqual("NOT_COMPARABLE", row["status"])
            self.assertEqual([], row["difference_dimensions"])

            second["server"]["configuration_fingerprint"] = first["server"]["configuration_fingerprint"]
            second["server"].pop("version")
            write_result(root, "second", second)
            matrix = aggregate_results(paths)
            row = matrix["matrices"][0]["cases"][0]
            self.assertEqual("UNVERIFIED", matrix["matrices"][0]["comparability"])
            self.assertEqual("UNVERIFIED", row["status"])
            self.assertEqual([], row["difference_dimensions"])

    def test_aggregator_compares_parity_reasons_and_library_correctness(self) -> None:
        first = result_payload(run_id="rust", language="rust", protocol_version="2.1", transport="raw_tls", encoding="TTLV", api_level="typed-protocol", status="failed")
        second = result_payload(run_id="python", language="python", protocol_version="2.1", transport="https", encoding="JSON", api_level="high-level", status="failed")
        first["cases"][0].update({
            "result_status": "OperationFailed", "result_reason": "Invalid Field",
            "library_correctness": "passed", "parity": "matched",
        })
        second["cases"][0].update({
            "result_status": "operation_failed", "result_reason": "CryptographicFailure",
            "library_correctness": "failed", "parity": "mismatched",
        })
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            matrix = aggregate_results([
                write_result(root, "first", first), write_result(root, "second", second),
            ])
        row = matrix["matrices"][0]["cases"][0]
        self.assertEqual("COMPARABLE", matrix["matrices"][0]["comparability"])
        self.assertEqual("PARITY_MISMATCH", row["status"])
        self.assertEqual({"result_reason", "library_correctness", "parity"}, set(row["difference_dimensions"]))
        markdown = generate_markdown(matrix)
        html_output = html.unescape(generate_html(matrix))
        self.assertEqual({7, 10}, {result["result_reason_code"] for result in row["results"].values()})
        for label in ("KMIP:1:7", "KMIP:1:10", "library:passed", "library:failed", "parity:mismatched"):
            self.assertIn(label, markdown + html_output)

    def test_enum_name_aliases_normalize_to_the_same_numeric_code(self) -> None:
        first = result_payload(run_id="rust", language="rust", protocol_version="2.1", transport="raw_tls", encoding="TTLV", api_level="typed-protocol", status="passed")
        second = result_payload(run_id="python", language="python", protocol_version="2.1", transport="https", encoding="JSON", api_level="high-level", status="passed")
        first["cases"][0].update({"result_status": "OperationSucceeded", "result_reason": "InvalidField"})
        second["cases"][0].update({"result_status": "operation_succeeded", "result_reason": "INVALID_FIELD"})
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            matrix = aggregate_results([
                write_result(root, "first", first), write_result(root, "second", second),
            ])
        row = matrix["matrices"][0]["cases"][0]
        self.assertEqual("PARITY_OK", row["status"])
        self.assertEqual({0}, {result["result_status_code"] for result in row["results"].values()})
        self.assertEqual({7}, {result["result_reason_code"] for result in row["results"].values()})

    def test_boolean_is_not_accepted_as_a_numeric_kmip_result_code(self) -> None:
        first = result_payload(run_id="rust", language="rust", protocol_version="2.1", transport="raw_tls", encoding="TTLV", api_level="typed-protocol", status="passed")
        second = result_payload(run_id="python", language="python", protocol_version="2.1", transport="https", encoding="JSON", api_level="high-level", status="passed")
        first["cases"][0]["result_reason"] = True
        second["cases"][0]["result_reason_code"] = 1
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            matrix = aggregate_results([
                write_result(root, "first", first), write_result(root, "second", second),
            ])
        row = matrix["matrices"][0]["cases"][0]
        self.assertEqual("PARITY_MISMATCH", row["status"])
        self.assertIn("result_reason", row["difference_dimensions"])
        self.assertNotEqual(1, row["results"]["rust"]["result_reason_code"])


if __name__ == "__main__":
    unittest.main()
