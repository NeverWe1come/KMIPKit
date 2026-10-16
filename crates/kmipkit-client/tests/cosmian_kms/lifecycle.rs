//! Manifest-to-test contract for the per-object lifecycle family.

const EXECUTABLE_CASE_IDS: &[&str] = &[];

#[test]
fn cosmian_lifecycle_manifest_cases_have_typed_executors() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/integration/cosmian/capability-matrix.json"
    ))
    .expect("capability manifest must be valid JSON");
    let manifest_cases: std::collections::BTreeSet<_> = manifest["cases"]
        .as_array()
        .expect("capability manifest contains cases")
        .iter()
        .filter(|row| row["test_family"] == "cosmian_kms::lifecycle")
        .filter_map(|row| row["case_id"].as_str())
        .collect();
    let executable_cases: std::collections::BTreeSet<_> = EXECUTABLE_CASE_IDS.iter().copied().collect();
    assert_eq!(
        executable_cases, manifest_cases,
        "every lifecycle manifest case must map to exactly one typed executor"
    );
}
