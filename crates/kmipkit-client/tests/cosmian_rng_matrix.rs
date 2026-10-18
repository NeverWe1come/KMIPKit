//! Offline regression checks for the Cosmian RNG matrix classifications.
//!
//! RNG Parameters is defined as a structure by OASIS KMIP v2.1 §7.36,
//! Table 388, but it is not included in the operation payloads: §6.1.49,
//! Table 316 lists only Data Length for RNG Retrieve, and §6.1.50, Table 319
//! lists only Data for RNG Seed. Parameter-present variants are invalid and
//! must remain non-executable.

use serde_json::Value;

const MATRIX: &str = include_str!("../../../tests/integration/cosmian/capability-matrix.json");
const PARAMETERIZED_RNG_CASES: [&str; 6] = [
    "cosmian-528-rng-retrieve-0-parameters-true",
    "cosmian-528-rng-retrieve-1-parameters-true",
    "cosmian-528-rng-retrieve-32-parameters-true",
    "cosmian-528-rng-retrieve-65536-parameters-true",
    "cosmian-528-rng-seed-0-parameters-true",
    "cosmian-528-rng-seed-32-parameters-true",
];

#[test]
fn parameterized_rng_rows_are_normative_exclusions_without_sending_a_request() {
    let manifest: Value = serde_json::from_str(MATRIX).expect("valid capability matrix");
    let rows = manifest["cases"].as_array().expect("matrix cases array");

    for case_id in PARAMETERIZED_RNG_CASES {
        let matching_rows = rows
            .iter()
            .filter(|row| row["case_id"] == case_id)
            .collect::<Vec<_>>();
        assert_eq!(matching_rows.len(), 1, "matrix row {case_id}");
        let row = matching_rows[0];

        assert_eq!(
            row["test_family"],
            "cosmian_kms::random::cosmian_random_case_ids_cover_manifest_and_exclusions",
            "{case_id}"
        );
        assert_eq!(
            row["expected_outcome"], "standards_invalid_exclusion",
            "{case_id}"
        );
        assert_eq!(
            row["execution_status"], "non_executable_normative_exclusion",
            "{case_id}"
        );
        assert_eq!(row["exclusion"]["request_may_be_sent"], false, "{case_id}");
        assert_eq!(
            row["observed_result"]["classification"], "not_run",
            "{case_id}"
        );
        assert_eq!(row["setup"].as_array().expect("setup array").len(), 0);

        let (section, table, reference) = if case_id.contains("rng-retrieve") {
            (
                "6.1.49",
                316,
                "Operation request payload schema; only Data Length is listed.",
            )
        } else {
            (
                "6.1.50",
                319,
                "Operation request payload schema; only Data is listed.",
            )
        };
        assert_eq!(
            row["exclusion"]["normative_sources"],
            serde_json::json!([{
                "document": "OASIS KMIP Specification v2.1",
                "sections": [section],
                "tables": [table],
                "referenced_from": reference
            }]),
            "{case_id}"
        );
        assert!(
            row["oasis_source"]
                .as_array()
                .is_some_and(|sources| sources.iter().any(|source| source == "oasis-7.36-t388")),
            "{case_id}: RNG Parameters definition citation missing"
        );
        let operation_citation = format!("§{section}, Table {table}");
        let exclusion_reason = row["exclusion"]["reason"]
            .as_str()
            .expect("normative exclusion reason");
        assert!(
            exclusion_reason.contains(&operation_citation),
            "{case_id}: {exclusion_reason}"
        );
        let notes = row["notes"].as_str().expect("normative source notes");
        assert!(notes.contains(&operation_citation), "{case_id}: {notes}");
        assert!(
            notes.contains("§7.36, Table 388")
                && notes.contains("does not add it to this operation payload"),
            "{case_id}: {notes}"
        );
    }
}
