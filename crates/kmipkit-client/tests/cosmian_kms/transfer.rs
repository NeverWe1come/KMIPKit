//! Manifest parity for transfer operation cases against Cosmian 5.28.0.

#[test]
fn cosmian_transfer_manifest_cases_have_typed_executors() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/integration/cosmian/capability-matrix.json"
    ))
    .expect("capability manifest must be valid JSON");
    let manifest_count = manifest["cases"]
        .as_array()
        .expect("capability manifest contains cases")
        .iter()
        .filter(|row| row["test_family"] == "cosmian_kms::transfer")
        .count();
    let executable_count = EXECUTABLE_CASE_IDS.len();
    assert_eq!(
        executable_count, manifest_count,
        "each transfer manifest row must map to exactly one typed executor"
    );
}

const EXECUTABLE_CASE_IDS: &[&str] = &[];
