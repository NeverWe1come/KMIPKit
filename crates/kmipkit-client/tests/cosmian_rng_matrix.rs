//! Offline regression checks for the Cosmian RNG matrix classifications.
//!
//! RNG Parameters is a valid KMIP structure under OASIS KMIP v2.1 §7.36,
//! Table 388. The RNG Retrieve and RNG Seed operation payloads are defined in
//! §§6.1.49–6.1.50, Tables 316–321. These rows remain client-blocked only
//! because KMIPKit's typed request models cannot represent RNG Parameters.

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
fn parameterized_rng_rows_are_client_blocked_without_sending_a_request() {
    let manifest: Value = serde_json::from_str(MATRIX).expect("valid capability matrix");
    let rows = manifest["cases"].as_array().expect("matrix cases array");

    for case_id in PARAMETERIZED_RNG_CASES {
        let matching_rows = rows
            .iter()
            .filter(|row| row["case_id"] == case_id)
            .collect::<Vec<_>>();
        assert_eq!(matching_rows.len(), 1, "matrix row {case_id}");
        let row = matching_rows[0];

        assert_eq!(row["test_family"], "cosmian_kms::random", "{case_id}");
        assert_eq!(row["expected_outcome"], "client_blocked", "{case_id}");
        assert_eq!(row["execution_status"], "planned_for_tasks_4_6", "{case_id}");
        assert_eq!(row["observed_result"]["classification"], "client_blocked", "{case_id}");
        assert_eq!(row["observed_result"]["target_request_sent"], false, "{case_id}");
        assert_eq!(row["observed_result"]["response_received"], false, "{case_id}");

        let reason = row["observed_result"]["reason"]
            .as_str()
            .expect("client-blocked reason");
        assert!(reason.contains("typed RNG request API"), "{case_id}: {reason}");
        assert!(reason.contains("separately scoped approved feature"), "{case_id}: {reason}");
        assert!(!reason.contains("KMIPKIT-0027"), "{case_id}: {reason}");
    }
}
