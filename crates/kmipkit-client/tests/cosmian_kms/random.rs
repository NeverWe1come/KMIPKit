//! Typed RNG integration cases for KMIPKIT-0015-FR-011 and FR-016.
//!
//! RNG Parameters is defined by OASIS KMIP Specification v2.1 §7.36,
//! Table 388. Operation payloads are defined by §§6.1.49–6.1.50, Tables
//! 316–321 (RNG Retrieve and RNG Seed). Data and Data Length use §§7.9–7.10.
//! The six parameter-present cases are normative exclusions: §6.1.49/Table
//! 316 and §6.1.50/Table 319 do not include RNG Parameters in their payloads.

use std::collections::BTreeSet;
use std::error::Error;

use kmipkit_client::ClientRequest;
use kmipkit_protocol::{RngRetrieveRequest, RngSeedRequest, SecretBytes};

const FAMILY: &str = "cosmian_kms::random";
const EXCLUSION_TEST_FAMILY: &str =
    "cosmian_kms::random::cosmian_random_case_ids_cover_manifest_and_exclusions";
const CASE_IDS: &str = include_str!("random-case-ids.txt");

#[derive(Clone, Copy, Debug)]
enum RandomOperation {
    Retrieve,
    Seed,
}

#[derive(Clone, Copy, Debug)]
struct MatrixCase {
    case_id: &'static str,
    operation: RandomOperation,
    length: usize,
    rng_algorithm: Option<&'static str>,
}

const LIVE_CASES: [MatrixCase; 6] = [
    MatrixCase {
        case_id: "cosmian-528-rng-retrieve-0-parameters-false",
        operation: RandomOperation::Retrieve,
        length: 0,
        rng_algorithm: None,
    },
    MatrixCase {
        case_id: "cosmian-528-rng-retrieve-1-parameters-false",
        operation: RandomOperation::Retrieve,
        length: 1,
        rng_algorithm: None,
    },
    MatrixCase {
        case_id: "cosmian-528-rng-retrieve-32-parameters-false",
        operation: RandomOperation::Retrieve,
        length: 32,
        rng_algorithm: None,
    },
    MatrixCase {
        case_id: "cosmian-528-rng-retrieve-65536-parameters-false",
        operation: RandomOperation::Retrieve,
        length: 65_536,
        rng_algorithm: None,
    },
    MatrixCase {
        case_id: "cosmian-528-rng-seed-0-parameters-false",
        operation: RandomOperation::Seed,
        length: 0,
        rng_algorithm: None,
    },
    MatrixCase {
        case_id: "cosmian-528-rng-seed-32-parameters-false",
        operation: RandomOperation::Seed,
        length: 32,
        rng_algorithm: None,
    },
];

const NORMATIVE_EXCLUSION_CASES: [MatrixCase; 6] = [
    MatrixCase {
        case_id: "cosmian-528-rng-retrieve-0-parameters-true",
        operation: RandomOperation::Retrieve,
        length: 0,
        rng_algorithm: Some("Unspecified"),
    },
    MatrixCase {
        case_id: "cosmian-528-rng-retrieve-1-parameters-true",
        operation: RandomOperation::Retrieve,
        length: 1,
        rng_algorithm: Some("Unspecified"),
    },
    MatrixCase {
        case_id: "cosmian-528-rng-retrieve-32-parameters-true",
        operation: RandomOperation::Retrieve,
        length: 32,
        rng_algorithm: Some("Unspecified"),
    },
    MatrixCase {
        case_id: "cosmian-528-rng-retrieve-65536-parameters-true",
        operation: RandomOperation::Retrieve,
        length: 65_536,
        rng_algorithm: Some("Unspecified"),
    },
    MatrixCase {
        case_id: "cosmian-528-rng-seed-0-parameters-true",
        operation: RandomOperation::Seed,
        length: 0,
        rng_algorithm: Some("Unspecified"),
    },
    MatrixCase {
        case_id: "cosmian-528-rng-seed-32-parameters-true",
        operation: RandomOperation::Seed,
        length: 32,
        rng_algorithm: Some("Unspecified"),
    },
];

#[test]
fn cosmian_random_case_ids_cover_manifest_and_exclusions() {
    let mut rows = super::common::manifest_rows(FAMILY).expect("random manifest rows");
    rows.extend(
        super::common::manifest_rows(EXCLUSION_TEST_FAMILY).expect("RNG normative exclusion rows"),
    );
    let manifest_ids = rows
        .iter()
        .filter_map(|row| row["case_id"].as_str())
        .collect::<BTreeSet<_>>();
    let test_ids = case_ids().collect::<BTreeSet<_>>();
    assert_eq!(
        rows.len(),
        manifest_ids.len(),
        "duplicate RNG matrix case ID"
    );
    assert_eq!(case_ids().count(), test_ids.len(), "duplicate RNG case ID");
    assert_eq!(
        test_ids, manifest_ids,
        "RNG matrix rows need explicit case coverage"
    );
    assert_eq!(
        test_ids.len(),
        12,
        "the approved RNG matrix has twelve cases"
    );

    let live_ids = LIVE_CASES
        .iter()
        .map(|case| case.case_id)
        .collect::<BTreeSet<_>>();
    let excluded_ids = NORMATIVE_EXCLUSION_CASES
        .iter()
        .map(|case| case.case_id)
        .collect::<BTreeSet<_>>();
    assert!(live_ids.is_disjoint(&excluded_ids));
    assert_eq!(
        live_ids.len(),
        LIVE_CASES.len(),
        "duplicate live RNG case ID"
    );
    assert_eq!(
        live_ids
            .union(&excluded_ids)
            .copied()
            .collect::<BTreeSet<_>>(),
        test_ids
    );

    for case in LIVE_CASES {
        let row = manifest_row(&rows, case.case_id);
        assert_eq!(row["test_family"], FAMILY, "{}", case.case_id);
        assert_matrix_case(row, case, "typed_success");
    }

    for case in NORMATIVE_EXCLUSION_CASES {
        assert_normative_exclusion(&rows, case);
    }
}

#[test]
fn valid_rng_rows_construct_and_encode_the_requested_typed_payloads_offline() {
    const DATA_LENGTH: u32 = 0x0042_00D5;
    const DATA: u32 = 0x0042_00C2;

    for case in LIVE_CASES {
        let payload = match case.operation {
            RandomOperation::Retrieve => RngRetrieveRequest::new(
                i32::try_from(case.length).expect("approved RNG length fits KMIP Integer"),
            )
            .to_ttlv_payload()
            .expect("RNG Retrieve request payload"),
            RandomOperation::Seed => RngSeedRequest::new(SecretBytes::new(vec![0x5A; case.length]))
                .to_ttlv_payload()
                .expect("RNG Seed request payload"),
        };
        match case.operation {
            RandomOperation::Retrieve => {
                let length = payload.view().children().iter().find_map(|item| {
                    (item.tag().raw() == DATA_LENGTH)
                        .then(|| {
                            item.with_value(|value| match value {
                                kmipkit_ttlv::ValueView::Integer(length) => Some(*length),
                                _ => None,
                            })
                        })
                        .flatten()
                });
                assert_eq!(
                    length,
                    Some(i32::try_from(case.length).expect("RNG length fits KMIP Integer")),
                    "{}",
                    case.case_id
                );
                assert_eq!(payload.view().children().len(), 1, "{}", case.case_id);
            }
            RandomOperation::Seed => {
                let length = payload.view().children().iter().find_map(|item| {
                    (item.tag().raw() == DATA)
                        .then(|| {
                            item.with_value(|value| match value {
                                kmipkit_ttlv::ValueView::ByteString(bytes) => Some(bytes.len()),
                                _ => None,
                            })
                        })
                        .flatten()
                });
                assert_eq!(length, Some(case.length), "{}", case.case_id);
                assert_eq!(payload.view().children().len(), 1, "{}", case.case_id);
            }
        }
    }
}

#[test]
fn valid_rng_observations_record_response_validation_failures_without_interop() {
    let rows = super::common::manifest_rows(FAMILY).expect("random manifest rows");

    for case in LIVE_CASES {
        let row = manifest_row(&rows, case.case_id);
        let observed = &row["observed_result"];
        assert_eq!(row["expected_outcome"], "typed_success", "{}", case.case_id);
        assert_eq!(
            observed["classification"], "response_validation_failed",
            "{}",
            case.case_id
        );
        for (field, expected) in [
            ("request_sent", true),
            ("target_request_sent", true),
            ("response_received", true),
            ("interoperability", false),
            ("response_operation_present", false),
        ] {
            assert_eq!(
                observed[field].as_bool(),
                Some(expected),
                "{}: {field}",
                case.case_id
            );
        }
        for (field, expected) in [("result_status", 1), ("result_reason", 4)] {
            assert_eq!(
                observed[field].as_i64(),
                Some(expected),
                "{}: {field}",
                case.case_id
            );
        }
    }
}

fn assert_normative_exclusion(rows: &[serde_json::Value], case: MatrixCase) {
    let row = manifest_row(rows, case.case_id);
    assert_eq!(
        row["test_family"], EXCLUSION_TEST_FAMILY,
        "{}",
        case.case_id
    );
    assert_matrix_case(row, case, "standards_invalid_exclusion");
    assert_eq!(
        row["capability_basis"], "normative_exclusion",
        "{}",
        case.case_id
    );
    assert_eq!(
        row["execution_status"], "non_executable_normative_exclusion",
        "{}",
        case.case_id
    );
    assert_eq!(
        row["observed_result"]["classification"], "not_run",
        "{}",
        case.case_id
    );
    assert_eq!(
        row["exclusion"]["request_may_be_sent"], false,
        "{}",
        case.case_id
    );
    assert_eq!(row["setup"].as_array().expect("exclusion setup").len(), 0);
    let (operation_source, section, table, request_item) = match case.operation {
        RandomOperation::Retrieve => ("oasis-6.1.49-t316-317-318", "6.1.49", 316, "Data Length"),
        RandomOperation::Seed => ("oasis-6.1.50-t319-320-321", "6.1.50", 319, "Data"),
    };
    assert!(
        row["oasis_source"]
            .as_array()
            .is_some_and(|sources| sources.iter().any(|source| source == operation_source))
    );
    assert_eq!(
        row["exclusion"]["normative_sources"],
        serde_json::json!([{
            "document": "OASIS KMIP Specification v2.1",
            "sections": [section],
            "tables": [table],
            "referenced_from": format!("Operation request payload schema; only {request_item} is listed.")
        }]),
        "{}",
        case.case_id
    );
}

fn assert_matrix_case(row: &serde_json::Value, case: MatrixCase, expected_outcome: &str) {
    assert_eq!(
        row["expected_outcome"], expected_outcome,
        "{}",
        case.case_id
    );
    let operation = match case.operation {
        RandomOperation::Retrieve => "RNG Retrieve",
        RandomOperation::Seed => "RNG Seed",
    };
    assert_eq!(row["operation"], operation, "{}", case.case_id);
    let length_field = match case.operation {
        RandomOperation::Retrieve => "data_length_bytes",
        RandomOperation::Seed => "seed_length_bytes",
    };
    assert_eq!(
        row["parameters"][length_field].as_u64(),
        Some(u64::try_from(case.length).expect("matrix RNG lengths fit in u64")),
        "{}",
        case.case_id
    );
    match case.rng_algorithm {
        None => assert!(
            row["parameters"]["rng_parameters"].is_null(),
            "{}",
            case.case_id
        ),
        Some(algorithm) => assert_eq!(
            row["parameters"]["rng_parameters"]["rng_algorithm"], algorithm,
            "{}",
            case.case_id
        ),
    }
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_528_random_matrix_no_parameter_cases() {
    let selected_cases = selected_live_cases().unwrap_or_else(|error| panic!("{error}"));
    let mut failures = Vec::new();
    for case in selected_cases.iter().copied() {
        match run_live_case(case) {
            Ok(()) => println!("case_id={} classification=typed_success", case.case_id),
            Err(error) => {
                println!(
                    "{}",
                    rng_failure_observation_line(case.case_id, error.as_ref())
                );
                failures.push(case.case_id);
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} live RNG cases failed: {}",
        failures.len(),
        selected_cases.len(),
        failures.join(", ")
    );
}

fn rng_failure_observation_line(case_id: &str, error: &dyn Error) -> String {
    let detail = error.to_string();
    match rng_error_classification(&detail) {
        Some(classification) => format!(
            "case_id={case_id} classification={classification} expected_outcome=typed_success observed={classification} request_sent=true response_received=true interoperability=false detail={detail}"
        ),
        None => format!("case_id={case_id} classification=failed detail={detail}"),
    }
}

fn rng_error_classification(detail: &str) -> Option<&'static str> {
    if detail.contains("typed response rejected by KMIPKit") {
        Some("response_validation_failed")
    } else if detail.contains("KMIP Result Status") || detail.contains("Result Status") {
        Some("rejected")
    } else {
        None
    }
}

fn selected_live_cases() -> Result<Vec<MatrixCase>, String> {
    let filter = std::env::var("KMIPKIT_COSMIAN_CASE_FILTER").ok();
    select_live_cases(filter.as_deref())
}

fn select_live_cases(filter: Option<&str>) -> Result<Vec<MatrixCase>, String> {
    let Some(filter) = filter else {
        return Ok(LIVE_CASES.to_vec());
    };
    let selected = filter.split(',').collect::<BTreeSet<_>>();
    for case_id in &selected {
        if LIVE_CASES.iter().any(|case| case.case_id == *case_id) {
            continue;
        }
        if NORMATIVE_EXCLUSION_CASES
            .iter()
            .any(|case| case.case_id == *case_id)
        {
            return Err(format!(
                "KMIPKIT_COSMIAN_CASE_FILTER selected non-executable normative RNG exclusion {case_id}; the KMIP 2.1 request payload does not include RNG Parameters"
            ));
        }
        return Err(format!(
            "KMIPKIT_COSMIAN_CASE_FILTER contains an unknown RNG case ID: {case_id}"
        ));
    }
    Ok(selected
        .into_iter()
        .map(|case_id| {
            LIVE_CASES
                .iter()
                .copied()
                .find(|case| case.case_id == case_id)
                .expect("validated live RNG case ID")
        })
        .collect())
}

fn run_live_case(case: MatrixCase) -> Result<(), Box<dyn Error>> {
    let mut client = super::client()?;
    match case.operation {
        RandomOperation::Retrieve => {
            let requested_length = i32::try_from(case.length)
                .map_err(|_| "RNG Retrieve length does not fit the KMIP Integer")?;
            super::common::execute_case(
                &mut client,
                case.case_id,
                ClientRequest::rng_retrieve(RngRetrieveRequest::new(requested_length)),
                "RNG Retrieve",
                |response| {
                    let result = response
                        .rng_retrieve()
                        .ok_or_else(|| "typed RNG Retrieve response is missing".to_owned())?;
                    let data = result
                        .data()
                        .ok_or_else(|| "successful RNG Retrieve has no Data".to_owned())?;
                    let actual_length = data.with_bytes(<[u8]>::len);
                    if actual_length != case.length {
                        return Err(format!(
                            "RNG Retrieve returned {actual_length} bytes for requested length {}",
                            case.length
                        ));
                    }
                    Ok(())
                },
            )?;
        }
        RandomOperation::Seed => {
            let seed = vec![0x5A; case.length];
            super::common::execute_case(
                &mut client,
                case.case_id,
                ClientRequest::rng_seed(RngSeedRequest::new(SecretBytes::new(seed))),
                "RNG Seed",
                |response| {
                    let result = response
                        .rng_seed()
                        .ok_or_else(|| "typed RNG Seed response is missing".to_owned())?;
                    let used_length = result
                        .data_length()
                        .ok_or_else(|| "successful RNG Seed has no Data Length".to_owned())?;
                    let used_length = usize::try_from(used_length)
                        .map_err(|_| "RNG Seed returned a negative Data Length".to_owned())?;
                    if used_length > case.length {
                        return Err(format!(
                            "RNG Seed used {used_length} bytes from a {}-byte seed",
                            case.length
                        ));
                    }
                    Ok(())
                },
            )?;
        }
    }
    Ok(())
}

fn case_ids() -> impl Iterator<Item = &'static str> {
    CASE_IDS
        .lines()
        .map(str::trim)
        .filter(|case_id| !case_id.is_empty())
}

fn manifest_row<'a>(rows: &'a [serde_json::Value], case_id: &str) -> &'a serde_json::Value {
    rows.iter()
        .find(|row| row["case_id"] == case_id)
        .unwrap_or_else(|| panic!("random manifest case is missing: {case_id}"))
}

#[cfg(test)]
mod filter_tests {
    use super::*;

    #[test]
    fn malformed_typed_rng_response_is_reported_as_response_validation_failure() {
        let case_id = "cosmian-528-rng-retrieve-32-parameters-false";
        let error = std::io::Error::other(format!(
            "case {case_id} RNG Retrieve: typed response rejected by KMIPKit; wire operation None, Result Status Some(1), Result Reason Some(4)"
        ));

        assert_eq!(
            rng_failure_observation_line(case_id, &error),
            format!(
                "case_id={case_id} classification=response_validation_failed expected_outcome=typed_success observed=response_validation_failed request_sent=true response_received=true interoperability=false detail={error}"
            )
        );
    }

    #[test]
    fn valid_peer_rng_rejection_is_not_reported_as_response_validation_failure() {
        let case_id = "cosmian-528-rng-retrieve-32-parameters-false";
        let error = std::io::Error::other(format!(
            "case {case_id} RNG Retrieve: KMIP Result Status 1 (Operation Failed) Reason Some((10, \"Cryptographic Failure\"))"
        ));

        assert_eq!(
            rng_failure_observation_line(case_id, &error),
            format!(
                "case_id={case_id} classification=rejected expected_outcome=typed_success observed=rejected request_sent=true response_received=true interoperability=false detail={error}"
            )
        );
    }

    #[test]
    fn case_filter_selects_a_known_executable_case() {
        let selected = select_live_cases(Some("cosmian-528-rng-retrieve-32-parameters-false"))
            .expect("known live RNG case should be selectable");
        assert_eq!(
            selected.iter().map(|case| case.case_id).collect::<Vec<_>>(),
            ["cosmian-528-rng-retrieve-32-parameters-false"]
        );
    }

    #[test]
    fn case_filter_rejects_an_unknown_case() {
        let error = select_live_cases(Some("cosmian-528-rng-retrieve-999-parameters-false"))
            .expect_err("unknown RNG case must be rejected");
        assert!(error.contains("unknown RNG case ID"));
    }

    #[test]
    fn case_filter_rejects_a_normative_exclusion() {
        let error = select_live_cases(Some("cosmian-528-rng-retrieve-32-parameters-true"))
            .expect_err("normative exclusion must not be selectable for live execution");
        assert!(error.contains("non-executable normative RNG exclusion"));
    }
}
