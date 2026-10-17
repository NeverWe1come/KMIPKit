//! Matrix-driven cryptographic operation tests for KMIP 2.1.
//!
//! The operation citations and parameter values are read from each matrix row.
//! Hash and HMAC request payloads use known-answer vectors so response semantics
//! are checked independently of the server.

use std::collections::BTreeSet;
use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

use kmipkit_client::ClientRequest;
use kmipkit_protocol::{
    ActivateRequest, AttributeSet, CertifyRequest, CreateKeyPairRequest, CreateRequest,
    DecryptRequest, EncryptRequest, HashRequest, ImportRequest, MacRequest, MacVerifyRequest,
    MacVerifyResponse, ObjectType, OperationData, RegisterRequest, SecretBytes, SignRequest,
    SignatureVerifyRequest, UniqueIdentifier,
};
use kmipkit_ttlv::{Structure, Value};
use serde_json::Value as JsonValue;

const FAMILY: &str = "cosmian_kms::cryptographic";
const CASE_IDS: &str = include_str!("cryptographic-case-ids.txt");
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;
const SYMMETRIC_KEY: u32 = 0x0042_007F;
const CRYPTOGRAPHIC_DOMAIN_PARAMETERS: u32 = 0x0042_0029;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const PADDING_METHOD: u32 = 0x0042_005F;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_00AE;
const SALT_LENGTH: u32 = 0x0042_0100;
const MASK_GENERATOR: u32 = 0x0042_0101;
const MASK_GENERATOR_HASHING_ALGORITHM: u32 = 0x0042_0102;
const P_SOURCE: u32 = 0x0042_0103;
const RECOMMENDED_CURVE: u32 = 0x0042_0060;
const KEY_FORMAT_TYPE_ATTRIBUTE: u32 = 0x0042_0042;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const SECRET_DATA_TYPE: u32 = 0x0042_0086;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_VALUE: u32 = 0x0042_001E;
const CERTIFICATE_LENGTH: u32 = 0x0042_00AD;
const PUBLIC_KEY: u32 = 0x0042_006F;
const PRIVATE_KEY: u32 = 0x0042_006E;
const LINK: u32 = 0x0042_004A;
const LINK_TYPE: u32 = 0x0042_004B;
const LINKED_OBJECT_IDENTIFIER: u32 = 0x0042_004C;
const KEY_USAGE_SIGN: i32 = 0x0000_0010;
const KEY_USAGE_VERIFY: i32 = 0x0000_0020;
const KEY_USAGE_ENCRYPT: i32 = 0x0000_0004;
const KEY_USAGE_DECRYPT: i32 = 0x0000_0008;
const OAEP_TEST_LABEL: &[u8] = b"KMIPKit OAEP label";
const KEY_USAGE_MAC_GENERATE_VERIFY: i32 = 384;
const RAW_KEY_FORMAT_TYPE: u32 = 1;
const FIXTURE_KEY: [u8; 32] = [0x5A; 32];
const EXPECTED_TARGET_ROWS: usize = 1_466;

const OPERATIONS: [&str; 10] = [
    "Certify",
    "Create",
    "Create Key Pair",
    "Decrypt",
    "Encrypt",
    "Hash",
    "MAC",
    "MAC Verify",
    "Sign",
    "Signature Verify",
];

const ENCRYPTION_PARAMETERS: [&str; 30] = [
    "aad",
    "alphabet",
    "curve",
    "data_shape",
    "documented_scheme",
    "hash",
    "iv_length_bits",
    "kem_action",
    "opaque_fixture_configuration",
    "key_bits",
    "key_algorithm",
    "metadata",
    "mgf",
    "mgf_hash",
    "mode",
    "non_fips",
    "nested_kmip_algorithm_field",
    "nonce_source",
    "p_source",
    "padding",
    "policy",
    "provisioning",
    "post_quantum",
    "pre_quantum",
    "scheme",
    "source_only_numeric_probe",
    "tag_length_bytes",
    "tweak",
    "vendor_algorithm_value",
    "wire_container",
];

const CREATE_PARAMETERS: [&str; 8] = [
    "key_bits",
    "requested_key_format",
    "intended_mode",
    "boundary",
    "output_secret_data_type",
    "output_bytes",
    "access_policy",
    "vendor_algorithm_value",
];
const CREATE_KEY_PAIR_PARAMETERS: [&str; 8] = [
    "curve",
    "key_bits",
    "nested_post_quantum_numeric_probe",
    "source_only_numeric_probe",
    "vendor_algorithm_value",
    "pre_quantum",
    "post_quantum",
    "policy",
];
const CERTIFY_PARAMETERS: [&str; 11] = [
    "subject_input",
    "subject_key_bits",
    "issuer",
    "issuer_digest",
    "validity",
    "extensions",
    "curve",
    "issuer_key_bits",
    "algorithm_dimension",
    "key_bits",
    "non_fips",
];

const HASH_PARAMETERS: [&str; 3] = ["data_length_bytes", "data_mode", "hash"];
const MAC_PARAMETERS: [&str; 5] = [
    "data_mode",
    "hash",
    "hash_selection",
    "key_bits",
    "verification",
];
const SIGN_PARAMETERS: [&str; 13] = [
    "accept_sha256_substitution",
    "curve",
    "data_mode",
    "deterministic",
    "digital_signature_algorithm",
    "external_digest",
    "hash",
    "key_bits",
    "mgf_hash",
    "salt_length_bytes",
    "signature_scheme",
    "source_condition",
    "verify_requested_digest_independently",
];
const VERIFY_PARAMETERS: [&str; 12] = [
    "curve",
    "data_mode",
    "deterministic",
    "digital_signature_algorithm",
    "external_digest",
    "hash",
    "key_bits",
    "mgf_hash",
    "salt_length_bytes",
    "setup_signature_source",
    "signature_scheme",
    "source_condition",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaseDisposition {
    Executable(&'static str),
    ExpectedFindingProbe {
        operation: &'static str,
        expected_outcome: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FindingProbePreflight {
    SendExactTarget,
    NotSent(&'static str),
}

#[test]
fn cosmian_crypto_finding_rows_are_not_success_dispositions() {
    let rows = target_rows().expect("cryptographic target rows");
    let findings = rows
        .iter()
        .filter(|row| row["expected_outcome"] != "typed_success")
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 246, "reviewed finding inventory changed");
    for row in findings {
        assert!(
            matches!(
                disposition(row),
                Ok(CaseDisposition::ExpectedFindingProbe { .. })
            ),
            "expected finding {} must stay distinct from interoperability success",
            case_id(row).expect("case ID")
        );
    }
}

#[test]
fn post_quantum_certify_finding_builds_exact_pkcs10_request_without_algorithm_enum() {
    const CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
    const CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
    const ATTRIBUTES: u32 = 0x0042_0125;
    const CERTIFICATE_TYPE: u32 = 0x0042_001D;

    let id = "cosmian-528-certify-certificaterequest-mldsa-44-subject-oid-only-der";
    let row = target_rows()
        .expect("cryptographic target rows")
        .into_iter()
        .find(|row| row["case_id"] == id)
        .expect("reviewed ML-DSA Certify discrepancy row");
    assert_eq!(row["expected_outcome"], "source_documentation_discrepancy");
    assert_eq!(
        row["parameters"]["algorithm_dimension"],
        "subject ASN.1 OID only; no CryptographicAlgorithm enumeration is sent"
    );
    assert_eq!(
        row["wire_enum_values"]["cryptographic_algorithm"]["value"],
        JsonValue::Null
    );
    assert_eq!(
        row["wire_enum_values"]["cryptographic_algorithm"]["classification"],
        "not_sent_asn1_oid_only"
    );

    validate_exact_finding_probe_dimensions(&row)
        .expect("Certify finding probe dimensions must match the exact request");
    let payload = build_offline_certify_request(
        &row,
        UniqueIdentifier::TextString("offline-certify-subject".to_owned()),
    )
    .expect("offline Certify request builder")
    .into_ttlv_payload()
    .expect("offline Certify TTLV payload");

    assert_eq!(enum_field(&payload, CERTIFICATE_REQUEST_TYPE), Some(2));
    assert!(
        byte_string_field_length(&payload, CERTIFICATE_REQUEST_VALUE).is_some_and(|len| len > 0)
    );
    let attribute_contract = payload
        .view()
        .children()
        .iter()
        .find(|item| item.tag().raw() == ATTRIBUTES)
        .and_then(|item| {
            item.with_value(|value| {
                let kmipkit_ttlv::ValueView::Structure(attributes) = value else {
                    return None;
                };
                let fields = attributes.children();
                let certificate_type = fields
                    .iter()
                    .find(|field| field.tag().raw() == CERTIFICATE_TYPE)
                    .and_then(|field| {
                        field.with_value(|value| match value {
                            kmipkit_ttlv::ValueView::Enumeration(value) => Some(*value),
                            _ => None,
                        })
                    })?;
                Some((
                    certificate_type,
                    fields
                        .iter()
                        .any(|field| field.tag().raw() == CRYPTOGRAPHIC_ALGORITHM),
                ))
            })
        })
        .expect("Certify Attributes structure");
    assert_eq!(attribute_contract, (5, false));
}

#[test]
fn pqc_certify_pem_and_der_use_their_kmip_request_types() {
    const CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
    // OASIS KMIP v2.1 §11.8, Table 439 assigns PKCS#10=2 and PEM=3.
    for case_id in [
        "cosmian-528-certify-certificaterequest-mldsa-44-subject-oid-only-pem",
        "cosmian-528-certify-certificaterequest-mldsa-44-subject-oid-only-der",
    ] {
        let row = target_rows()
            .expect("cryptographic target rows")
            .into_iter()
            .find(|row| row["case_id"] == case_id)
            .unwrap_or_else(|| panic!("missing Certify case {case_id}"));
        let (expected_name, oasis_value) = match row["parameters"]["subject_input"].as_str() {
            Some("csr-pem") => ("PEM", 3),
            Some("csr-pkcs10-der") => ("PKCS10", 2),
            other => panic!("case {case_id} has unexpected CSR input {other:?}"),
        };
        assert_eq!(
            row["wire_enum_values"]["certificate_request_type"]["name"],
            expected_name
        );
        let row_request_type = wire_enum_value(&row, "certificate_request_type")
            .unwrap_or_else(|error| panic!("case {case_id}: {error}"));
        assert_eq!(
            row_request_type, oasis_value,
            "case {case_id} manifest value must match OASIS KMIP v2.1 §11.8, Table 439"
        );

        validate_exact_finding_probe_dimensions(&row)
            .unwrap_or_else(|error| panic!("case {case_id}: {error}"));
        let payload = build_offline_certify_request(
            &row,
            UniqueIdentifier::TextString(format!("offline-{case_id}")),
        )
        .and_then(|request| request.into_ttlv_payload().map_err(Into::into))
        .unwrap_or_else(|error| panic!("case {case_id}: {error}"));
        assert_eq!(
            enum_field(&payload, CERTIFICATE_REQUEST_TYPE),
            Some(row_request_type)
        );
        assert_eq!(enum_field(&payload, CRYPTOGRAPHIC_ALGORITHM), None);
    }
}

#[test]
fn cosmian_crypto_finding_preflight_separates_exact_probes_from_not_sent_rows() {
    let rows = target_rows().expect("cryptographic target rows");
    let findings = rows
        .iter()
        .filter(|row| row["expected_outcome"] != "typed_success")
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 246, "reviewed finding inventory changed");

    let mut runnable = BTreeSet::new();
    let mut not_sent = BTreeSet::new();
    let mut blocker_counts = BTreeSet::new();
    let mut serialized_probe_payloads = std::collections::BTreeMap::<&str, usize>::new();
    for row in findings {
        let id = case_id(row).expect("case ID");
        let expected_outcome = row["expected_outcome"]
            .as_str()
            .expect("finding expected outcome");
        assert_ne!(expected_outcome, "typed_success", "{id}");
        assert!(matches!(
            disposition(row),
            Ok(CaseDisposition::ExpectedFindingProbe { .. })
        ));
        let CaseDisposition::ExpectedFindingProbe { operation, .. } =
            disposition(row).unwrap_or_else(|error| panic!("case {id}: {error}"))
        else {
            panic!("finding {id} is not classified as a target probe");
        };
        assert_eq!(row["operation"], operation, "finding {id}");
        match finding_probe_preflight(row).unwrap_or_else(|error| panic!("case {id}: {error}")) {
            FindingProbePreflight::SendExactTarget => {
                validate_exact_finding_probe_dimensions(row)
                    .unwrap_or_else(|error| panic!("case {id}: {error}"));
                assert!(
                    has_typed_executor(operation),
                    "finding {id} has no executor"
                );
                let payloads = build_offline_target_payloads(row)
                    .unwrap_or_else(|error| panic!("finding probe {id} request: {error}"));
                assert!(
                    !payloads.is_empty(),
                    "finding probe {id} has no exact serialized request"
                );
                for payload in &payloads {
                    assert!(
                        !payload.view().children().is_empty(),
                        "finding probe {id} has an empty {operation} TTLV payload"
                    );
                }
                *serialized_probe_payloads.entry(operation).or_default() += payloads.len();
                assert!(runnable.insert(id), "duplicate runnable probe {id}");
            }
            FindingProbePreflight::NotSent(reason) => {
                assert!(not_sent.insert(id), "duplicate blocked probe {id}");
                blocker_counts.insert((operation, reason));
            }
        }
    }
    eprintln!(
        "crypto probe inventory: runnable={} not_sent={} blockers={blocker_counts:?}",
        runnable.len(),
        not_sent.len()
    );
    assert_eq!(
        runnable.len(),
        231,
        "only probes with exact request and fixture dimensions may reach the target"
    );
    assert_eq!(
        not_sent.len(),
        15,
        "every unrepresentable probe stays NotSent"
    );
    assert!(runnable.is_disjoint(&not_sent));
    assert_eq!(
        runnable.len(),
        231,
        "all exact findings need typed payloads"
    );
    assert_eq!(
        not_sent.len(),
        15,
        "all 15 fixture gaps must remain NotSent"
    );
    assert_eq!(
        blocker_counts,
        [
            (
                "Decrypt",
                "exact AES/RSA hybrid ciphertext fixture is unavailable"
            ),
            (
                "Encrypt",
                "exact AES/RSA hybrid request fixture is unavailable"
            ),
            (
                "Decrypt",
                "opaque ConfigurableKEM key fixture is unavailable"
            ),
            (
                "Encrypt",
                "opaque ConfigurableKEM key fixture is unavailable"
            ),
        ]
        .into_iter()
        .collect(),
        "preflight blockers must remain operation-specific and explicit"
    );
    eprintln!(
        "offline finding probes: exact_rows={} serialized={serialized_probe_payloads:?} not_sent_rows={}",
        runnable.len(),
        not_sent.len()
    );
}

#[test]
fn every_typed_success_matrix_row_builds_a_typed_ttlv_payload_offline() {
    let rows = target_rows().expect("cryptographic target rows");
    let expected_rows = [
        ("Certify", 173),
        ("Create", 70),
        ("Create Key Pair", 35),
        ("Decrypt", 224),
        ("Encrypt", 289),
        ("Hash", 14),
        ("MAC", 27),
        ("MAC Verify", 54),
        ("Sign", 110),
        ("Signature Verify", 224),
    ]
    .into_iter()
    .collect::<std::collections::BTreeMap<_, _>>();
    let mut observed_rows = std::collections::BTreeMap::<&str, usize>::new();
    let mut observed_payloads = std::collections::BTreeMap::<&str, usize>::new();

    for row in rows
        .iter()
        .filter(|row| row["expected_outcome"] == "typed_success")
    {
        let id = case_id(row).expect("case ID");
        let operation = row["operation"].as_str().expect("operation");
        *observed_rows.entry(operation).or_default() += 1;
        let payloads = build_offline_target_payloads(row)
            .unwrap_or_else(|error| panic!("case {id} {operation}: {error}"));
        assert!(!payloads.is_empty(), "case {id} built no target payload");
        for payload in &payloads {
            assert!(
                !payload.view().children().is_empty(),
                "case {id} built an empty {operation} TTLV payload"
            );
        }
        *observed_payloads.entry(operation).or_default() += payloads.len();
    }

    assert_eq!(
        observed_rows, expected_rows,
        "typed-success row counts changed"
    );
    assert_eq!(observed_rows.values().sum::<usize>(), 1_220);
    assert_eq!(
        observed_payloads.values().sum::<usize>(),
        1_220,
        "typed-success payloads count only the rows classified as interoperable success"
    );
    eprintln!(
        "offline typed-success payloads: rows={observed_rows:?} serialized={observed_payloads:?}"
    );
}

#[test]
fn cosmian_cryptographic_case_ids_cover_manifest_rows_and_dispatch_registry() {
    let rows = target_rows().expect("cryptographic target rows");
    let manifest_ids = rows
        .iter()
        .map(|row| case_id(row).expect("case ID"))
        .collect::<BTreeSet<_>>();
    let listed_ids = case_ids().collect::<BTreeSet<_>>();

    assert_eq!(
        rows.len(),
        manifest_ids.len(),
        "duplicate cryptographic matrix case ID"
    );
    assert_eq!(
        case_ids().count(),
        listed_ids.len(),
        "duplicate cryptographic case ID"
    );
    assert_eq!(
        rows.len(),
        EXPECTED_TARGET_ROWS,
        "the reviewed cryptographic matrix inventory changed"
    );
    assert_eq!(
        listed_ids, manifest_ids,
        "every applicable matrix row needs exactly one checked-in case ID"
    );

    let mut typed_success_ids = BTreeSet::new();
    let mut executable_ids = BTreeSet::new();
    let mut finding_probe_ids = BTreeSet::new();
    for row in &rows {
        validate_operation_contract(row).unwrap_or_else(|error| panic!("{error}"));
        let id = case_id(row).expect("case ID");
        match disposition(row).expect("every target row has a registry disposition") {
            CaseDisposition::Executable(operation) => {
                assert_eq!(row["expected_outcome"], "typed_success", "{id}");
                assert_eq!(row["operation"], operation, "{id}");
                assert!(
                    typed_success_ids.insert(id),
                    "duplicate success mapping for {id}"
                );
                assert!(
                    executable_ids.insert(id),
                    "duplicate executable mapping for {id}"
                );
            }
            CaseDisposition::ExpectedFindingProbe {
                operation,
                expected_outcome,
            } => {
                assert_ne!(expected_outcome, "typed_success", "{id}");
                assert_eq!(row["expected_outcome"], expected_outcome, "{id}");
                assert_eq!(row["operation"], operation, "{id}");
                assert!(
                    finding_probe_ids.insert(id),
                    "duplicate finding mapping for {id}"
                );
            }
        }
    }

    assert!(typed_success_ids.is_disjoint(&finding_probe_ids));
    let mut classified_ids = executable_ids.clone();
    classified_ids.extend(finding_probe_ids.iter().copied());
    assert_eq!(
        classified_ids, listed_ids,
        "the live dispatcher must classify every listed row exactly once"
    );
    assert_eq!(
        typed_success_ids.len(),
        1_220,
        "the reviewed typed-success inventory changed"
    );
    assert_eq!(
        finding_probe_ids.len(),
        246,
        "the reviewed finding inventory changed"
    );
    assert_eq!(
        executable_ids.len(),
        1_220,
        "every typed-success row must have an executor"
    );
    assert_eq!(typed_success_ids, executable_ids);
    eprintln!(
        "cryptographic registry: typed_success={} executable={} dispatch_unavailable={} finding_probes={}",
        typed_success_ids.len(),
        executable_ids.len(),
        0,
        finding_probe_ids.len()
    );
}

#[test]
fn finding_probe_delivery_requires_typed_target_response_evidence() {
    let row = serde_json::json!({"case_id": "case-1"});
    assert_eq!(
        target_operation_delivery(
            &row,
            "Sign",
            "case case-1 Sign: KMIP Result Status 1 (Operation Failed) Reason Some(1)",
        ),
        TargetOperationDelivery::ResponseReceived
    );
    assert_eq!(
        target_operation_delivery(
            &row,
            "Sign",
            "case case-1 Sign: successful response failed a test postcondition",
        ),
        TargetOperationDelivery::ResponseReceived
    );
    assert_eq!(
        target_operation_delivery(
            &row,
            "Sign",
            "case case-1 Sign: client_blocked before response (delivery=NotSent)",
        ),
        TargetOperationDelivery::NotSent
    );
    assert_eq!(
        target_operation_delivery(
            &row,
            "Sign",
            "case case-1 Sign: client_blocked before response (delivery=PossiblySent)",
        ),
        TargetOperationDelivery::Uncertain
    );
    assert_eq!(
        target_operation_delivery(
            &row,
            "Sign",
            "case case-1 Sign: client_blocked before response (delivery=ResponseStarted)",
        ),
        TargetOperationDelivery::Uncertain
    );
    assert_eq!(
        target_operation_delivery(
            &row,
            "Sign",
            "case case-1 Signature Verify: client_blocked before response (delivery=NotSent)",
        ),
        TargetOperationDelivery::Uncertain
    );
    assert_eq!(
        target_operation_delivery(
            &row,
            "Signature Verify",
            "case case-1 Sign: KMIP Result Status 1 (Operation Failed)",
        ),
        TargetOperationDelivery::SetupResponseReceived
    );
}

#[test]
fn crypto_error_classification_separates_setup_from_target_rejections() {
    let row = serde_json::json!({"case_id": "case-1", "operation": "Encrypt"});
    assert_eq!(
        classify_case_error(
            &row,
            "case case-1 Create Key Pair: KMIP Result Status 1 (Operation Failed)"
        ),
        "setup_rejected"
    );
    assert_eq!(
        classify_case_error(
            &row,
            "case case-1 Encrypt: KMIP Result Status 1 (Operation Failed)"
        ),
        "target_rejected"
    );
}

#[test]
fn malformed_target_response_is_not_a_target_rejection_or_interoperability_success() {
    let row = serde_json::json!({"case_id": "case-1", "operation": "Encrypt"});
    let response_error = "case case-1 Encrypt: typed response rejected by KMIPKit; wire operation None, Result Status Some(1), Result Reason Some(4)";

    let observed = classify_case_error(&row, response_error);

    assert_eq!(observed, "response_validation_failed");
    assert_ne!(observed, "target_rejected");
    assert_ne!(observed, "typed_success");
}

#[test]
fn sha224_known_answer_vectors_cover_signature_prehash() {
    assert_eq!(
        known_answer_digest("SHA-224", false).expect("SHA-224 abc digest"),
        "23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7"
    );
    assert_eq!(
        known_answer_digest("SHA-224", true).expect("SHA-224 empty digest"),
        "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f"
    );
}

#[test]
fn multipart_matrix_rows_construct_all_three_typed_parts() {
    let mut streaming_rows = 0;
    for row in target_rows().expect("cryptographic target rows") {
        if row["parameters"]["data_mode"].as_str() != Some("init-update-final") {
            continue;
        }
        streaming_rows += 1;
        let id = case_id(&row).expect("case ID");
        match row["operation"].as_str().expect("operation") {
            "Hash" => {
                let initial = HashRequest::new(hash_parameters(&row).expect("Hash parameters"))
                    .with_init_indicator(true);
                assert!(initial.validate_multipart_shape().is_ok(), "{id}");
                assert!(initial.to_ttlv_payload().is_ok(), "{id}");
                let middle = HashRequest::new(hash_parameters(&row).expect("Hash parameters"))
                    .with_correlation_value(SecretBytes::new(vec![1]))
                    .with_init_indicator(false);
                assert!(middle.validate_multipart_shape().is_ok(), "{id}");
                assert!(middle.to_ttlv_payload().is_ok(), "{id}");
                let final_part = HashRequest::new(hash_parameters(&row).expect("Hash parameters"))
                    .with_correlation_value(SecretBytes::new(vec![1]))
                    .with_final_indicator(true);
                assert!(final_part.validate_multipart_shape().is_ok(), "{id}");
                assert!(final_part.to_ttlv_payload().is_ok(), "{id}");
            }
            "MAC" => {
                let initial = MacRequest::new().with_init_indicator(true);
                assert!(initial.validate_multipart_shape().is_ok(), "{id}");
                assert!(initial.to_ttlv_payload().is_ok(), "{id}");
                let middle = MacRequest::new()
                    .with_correlation_value(SecretBytes::new(vec![1]))
                    .with_init_indicator(false);
                assert!(middle.validate_multipart_shape().is_ok(), "{id}");
                assert!(middle.to_ttlv_payload().is_ok(), "{id}");
                let final_part = MacRequest::new()
                    .with_correlation_value(SecretBytes::new(vec![1]))
                    .with_final_indicator(true);
                assert!(final_part.validate_multipart_shape().is_ok(), "{id}");
                assert!(final_part.to_ttlv_payload().is_ok(), "{id}");
            }
            "Sign" => {
                let initial = SignRequest::new()
                    .with_cryptographic_parameters(
                        signature_parameters(&row).expect("Sign parameters"),
                    )
                    .with_init_indicator(true);
                assert!(initial.validate_multipart_shape().is_ok(), "{id}");
                assert!(initial.to_ttlv_payload().is_ok(), "{id}");
                let middle = SignRequest::new()
                    .with_cryptographic_parameters(
                        signature_parameters(&row).expect("Sign parameters"),
                    )
                    .with_correlation_value(SecretBytes::new(vec![1]))
                    .with_init_indicator(false);
                assert!(middle.validate_multipart_shape().is_ok(), "{id}");
                assert!(middle.to_ttlv_payload().is_ok(), "{id}");
                let final_part = SignRequest::new()
                    .with_cryptographic_parameters(
                        signature_parameters(&row).expect("Sign parameters"),
                    )
                    .with_correlation_value(SecretBytes::new(vec![1]))
                    .with_final_indicator(true);
                assert!(final_part.validate_multipart_shape().is_ok(), "{id}");
                assert!(final_part.to_ttlv_payload().is_ok(), "{id}");
            }
            "Signature Verify" => {
                let initial = SignatureVerifyRequest::new()
                    .with_cryptographic_parameters(
                        signature_parameters(&row).expect("Verify parameters"),
                    )
                    .with_init_indicator(true);
                assert!(initial.validate_multipart_shape().is_ok(), "{id}");
                assert!(initial.to_ttlv_payload().is_ok(), "{id}");
                let middle = SignatureVerifyRequest::new()
                    .with_cryptographic_parameters(
                        signature_parameters(&row).expect("Verify parameters"),
                    )
                    .with_correlation_value(SecretBytes::new(vec![1]))
                    .with_init_indicator(false);
                assert!(middle.validate_multipart_shape().is_ok(), "{id}");
                assert!(middle.to_ttlv_payload().is_ok(), "{id}");
                let final_part = SignatureVerifyRequest::new()
                    .with_cryptographic_parameters(
                        signature_parameters(&row).expect("Verify parameters"),
                    )
                    .with_correlation_value(SecretBytes::new(vec![1]))
                    .with_final_indicator(true);
                assert!(final_part.validate_multipart_shape().is_ok(), "{id}");
                assert!(final_part.to_ttlv_payload().is_ok(), "{id}");
            }
            operation => panic!("{id} has unexpected streaming operation {operation}"),
        }
    }
    assert_eq!(
        streaming_rows, 19,
        "the reviewed multipart inventory changed"
    );
}

#[test]
fn cosmian_sign_and_verify_typed_success_rows_have_executable_dispatch() {
    const INDEPENDENT_SIGNER_SOURCE: &str =
        "independent signer using requested digest; never reuse affected server raw Sign as proof";
    const INDEPENDENT_SIGNER_CASE_IDS: [&str; 8] = [
        "cosmian-528-signature-verify-publickey-ecdsa-p256-sha-384-raw",
        "cosmian-528-signature-verify-privatekey-ecdsa-p256-sha-384-raw",
        "cosmian-528-signature-verify-publickey-ecdsa-p256-sha-512-raw",
        "cosmian-528-signature-verify-privatekey-ecdsa-p256-sha-512-raw",
        "cosmian-528-signature-verify-publickey-ecdsa-secp256k1-sha-384-raw",
        "cosmian-528-signature-verify-privatekey-ecdsa-secp256k1-sha-384-raw",
        "cosmian-528-signature-verify-publickey-ecdsa-secp256k1-sha-512-raw",
        "cosmian-528-signature-verify-privatekey-ecdsa-secp256k1-sha-512-raw",
    ];

    let rows = target_rows().expect("cryptographic target rows");
    let mut sign_rows = 0;
    let mut sign_success_rows = 0;
    let mut verify_rows = 0;
    let mut verify_success_rows = 0;
    let mut independent_signer_ids = BTreeSet::new();

    for row in &rows {
        let id = case_id(row).expect("case ID");
        let expected_outcome = row["expected_outcome"].as_str().expect("expected outcome");
        match row["operation"].as_str() {
            Some("Sign") => {
                sign_rows += 1;
                if expected_outcome == "typed_success" {
                    sign_success_rows += 1;
                    assert!(
                        matches!(disposition(row), Ok(CaseDisposition::Executable("Sign"))),
                        "Sign typed_success case {id} must have executable typed dispatch"
                    );
                } else {
                    assert!(
                        matches!(
                            disposition(row),
                            Ok(CaseDisposition::ExpectedFindingProbe {
                                operation: "Sign",
                                expected_outcome: actual,
                            }) if actual == expected_outcome
                        ),
                        "Sign finding case {id} must retain its expected outcome"
                    );
                }
            }
            Some("Signature Verify") => {
                verify_rows += 1;
                if expected_outcome == "typed_success" {
                    verify_success_rows += 1;
                    assert!(
                        matches!(
                            disposition(row),
                            Ok(CaseDisposition::Executable("Signature Verify"))
                        ),
                        "Signature Verify typed_success case {id} must have executable typed dispatch"
                    );
                    if row["parameters"]["setup_signature_source"].as_str()
                        == Some(INDEPENDENT_SIGNER_SOURCE)
                    {
                        independent_signer_ids.insert(id);
                    }
                } else {
                    assert!(
                        matches!(
                            disposition(row),
                            Ok(CaseDisposition::ExpectedFindingProbe {
                                operation: "Signature Verify",
                                expected_outcome: actual,
                            }) if actual == expected_outcome
                        ),
                        "Signature Verify finding case {id} must retain its expected outcome"
                    );
                }
            }
            _ => {}
        }
    }

    assert_eq!(sign_rows, 163, "reviewed Sign matrix inventory changed");
    assert_eq!(
        sign_success_rows, 110,
        "reviewed Sign typed-success count changed"
    );
    assert_eq!(
        verify_rows, 321,
        "reviewed Signature Verify matrix inventory changed"
    );
    assert_eq!(
        verify_success_rows, 224,
        "reviewed Signature Verify typed-success count changed"
    );
    assert_eq!(
        independent_signer_ids.len(),
        INDEPENDENT_SIGNER_CASE_IDS.len()
    );
    assert_eq!(
        independent_signer_ids,
        INDEPENDENT_SIGNER_CASE_IDS.into_iter().collect(),
        "all eight independent signer rows stay explicitly identified"
    );
}

#[test]
fn asymmetric_cipher_parameter_builder_preserves_all_manifest_rsa_axes() {
    let rows = target_rows().expect("cryptographic target rows");
    let rsa_rows = rows
        .iter()
        .filter(|row| {
            matches!(row["operation"].as_str(), Some("Encrypt" | "Decrypt"))
                && row["algorithm"] == "RSA"
        })
        .collect::<Vec<_>>();
    assert_eq!(rsa_rows.len(), 174, "reviewed RSA cipher row count changed");

    for row in rsa_rows {
        let id = case_id(row).expect("case ID");
        let parameters =
            asymmetric_cipher_parameters(row).unwrap_or_else(|error| panic!("case {id}: {error}"));
        let expected_algorithm = wire_enum_value(row, "cryptographic_algorithm")
            .expect("manifest cryptographic algorithm");
        assert_eq!(
            enum_field(&parameters, CRYPTOGRAPHIC_ALGORITHM),
            Some(expected_algorithm),
            "case {id} must send its requested cryptographic algorithm"
        );
        match row["parameters"]["padding"].as_str() {
            Some("OAEP") => {
                assert_eq!(enum_field(&parameters, PADDING_METHOD), Some(2), "{id}");
                assert_eq!(
                    enum_field(&parameters, HASHING_ALGORITHM),
                    Some(
                        hashing_algorithm_value(
                            row["parameters"]["hash"].as_str().expect("OAEP hash"),
                        )
                        .expect("known OAEP hash")
                    ),
                    "case {id} must send its own OAEP hash"
                );
                assert_eq!(enum_field(&parameters, MASK_GENERATOR), Some(1), "{id}");
                assert_eq!(
                    enum_field(&parameters, MASK_GENERATOR_HASHING_ALGORITHM),
                    Some(
                        hashing_algorithm_value(
                            row["parameters"]["mgf_hash"]
                                .as_str()
                                .expect("OAEP mask generator hash"),
                        )
                        .expect("known OAEP MGF hash")
                    ),
                    "case {id} must send its own MGF hash"
                );
                assert!(
                    parameters
                        .view()
                        .children()
                        .iter()
                        .any(|item| item.tag().raw() == P_SOURCE),
                    "case {id} must explicitly encode its empty PSource"
                );
                let expected_label_length = if row["parameters"]["p_source"] == "nonempty" {
                    OAEP_TEST_LABEL.len()
                } else {
                    0
                };
                assert_eq!(
                    byte_string_field_length(&parameters, P_SOURCE),
                    Some(expected_label_length),
                    "case {id} must carry the selected PSource value"
                );
            }
            Some("PKCS1v15") => {
                assert_eq!(enum_field(&parameters, PADDING_METHOD), Some(8), "{id}");
            }
            Some("None") => {
                assert_eq!(enum_field(&parameters, PADDING_METHOD), Some(1), "{id}");
                assert_eq!(
                    enum_field(&parameters, HASHING_ALGORITHM),
                    Some(
                        hashing_algorithm_value(
                            row["parameters"]["hash"].as_str().expect("hybrid hash"),
                        )
                        .expect("known hybrid hash")
                    ),
                    "case {id} must preserve the hybrid KDF hash"
                );
            }
            other => panic!("case {id} has unexpected padding {other:?}"),
        }
    }
}

#[test]
fn signature_parameter_builder_preserves_manifest_algorithm_axes() {
    let mut typed_signature_rows = 0;
    for row in target_rows().expect("cryptographic target rows") {
        if !matches!(row["operation"].as_str(), Some("Sign" | "Signature Verify")) {
            continue;
        }
        if row["expected_outcome"] != "typed_success" {
            continue;
        }
        typed_signature_rows += 1;
        let id = case_id(&row).expect("case ID");
        let parameters =
            signature_parameters(&row).unwrap_or_else(|error| panic!("case {id}: {error}"));
        if row["parameters"]["data_mode"].as_str() != Some("init-update-final") {
            let _message =
                signature_message(&row).unwrap_or_else(|error| panic!("case {id}: {error}"));
        }
        assert_eq!(
            enum_field(&parameters, CRYPTOGRAPHIC_ALGORITHM),
            Some(wire_enum_value(&row, "cryptographic_algorithm").expect("algorithm enum")),
            "case {id} must preserve the row's Cryptographic Algorithm"
        );
        if let Some(hash) = row["parameters"]["hash"].as_str() {
            assert_eq!(
                enum_field(&parameters, HASHING_ALGORITHM),
                Some(hashing_algorithm_value(hash).expect("known hash")),
                "case {id} must preserve its requested hash"
            );
        }
        if let Some(scheme) = row["parameters"]["signature_scheme"].as_str() {
            let expected_padding = if scheme == "RSASSA-PSS" { 10 } else { 8 };
            assert_eq!(
                enum_field(&parameters, PADDING_METHOD),
                Some(expected_padding),
                "case {id} must preserve its RSA padding scheme"
            );
            assert_eq!(
                enum_field(&parameters, DIGITAL_SIGNATURE_ALGORITHM),
                Some(
                    digital_signature_algorithm_value(scheme, row["parameters"]["hash"].as_str())
                        .expect("signature enum")
                ),
                "case {id} must preserve its signature scheme"
            );
            if scheme == "RSASSA-PSS" {
                let hash = row["parameters"]["hash"].as_str().expect("PSS hash");
                let salt = match row["parameters"]["salt_length_bytes"].as_str() {
                    Some("digest") => i32::try_from(digest_length(hash).expect("digest length"))
                        .expect("digest length fits KMIP Integer"),
                    _ => i32::try_from(
                        row["parameters"]["salt_length_bytes"]
                            .as_u64()
                            .expect("explicit salt length"),
                    )
                    .expect("salt length fits KMIP Integer"),
                };
                assert_eq!(integer_field(&parameters, SALT_LENGTH), Some(salt), "{id}");
                assert_eq!(enum_field(&parameters, MASK_GENERATOR), Some(1), "{id}");
                assert_eq!(
                    enum_field(&parameters, MASK_GENERATOR_HASHING_ALGORITHM),
                    Some(
                        hashing_algorithm_value(
                            row["parameters"]["mgf_hash"]
                                .as_str()
                                .expect("PSS MGF hash"),
                        )
                        .expect("valid PSS MGF hash")
                    ),
                    "case {id} must preserve its PSS MGF hash"
                );
            }
        }
        if let Some(algorithm) = row["parameters"]["digital_signature_algorithm"].as_str() {
            assert_eq!(
                enum_field(&parameters, DIGITAL_SIGNATURE_ALGORITHM),
                Some(digital_signature_algorithm_name(algorithm).expect("signature algorithm")),
                "case {id} must preserve its Digital Signature Algorithm"
            );
        }
    }
    assert_eq!(
        typed_signature_rows, 334,
        "reviewed signature typed-success count changed"
    );
}

fn enum_field(structure: &Structure, tag: u32) -> Option<u32> {
    structure
        .view()
        .children()
        .iter()
        .find(|item| item.tag().raw() == tag)
        .and_then(|item| {
            item.with_value(|value| match value {
                kmipkit_ttlv::ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            })
        })
}

fn integer_field(structure: &Structure, tag: u32) -> Option<i32> {
    structure
        .view()
        .children()
        .iter()
        .find(|item| item.tag().raw() == tag)
        .and_then(|item| {
            item.with_value(|value| match value {
                kmipkit_ttlv::ValueView::Integer(raw) => Some(*raw),
                _ => None,
            })
        })
}

#[test]
fn certificate_encrypt_rows_have_matching_local_round_trip_key_fixtures() {
    let rows = target_rows().expect("cryptographic target rows");
    for row in rows
        .iter()
        .filter(|row| row["operation"] == "Encrypt" && row["object_type"] == "Certificate")
    {
        let id = case_id(row).expect("case ID");
        let mut fixture_row = row.clone();
        if let Some(key_algorithm) = row["parameters"]["key_algorithm"].as_str() {
            fixture_row["algorithm"] = JsonValue::String(key_algorithm.to_owned());
            if key_algorithm == "RSA" {
                fixture_row["wire_enum_values"]["cryptographic_algorithm"] = serde_json::json!({"name":"RSA","value":"0x00000004","classification":"standard"});
            }
        }
        let certificate =
            validation_fixture_path(&fixture_row, "certificate").expect("certificate path");
        let stem = validation_fixture_stem(&fixture_row).expect("fixture stem");
        let directory = certificate.parent().expect("fixture directory");
        assert!(certificate.is_file(), "{id} certificate fixture is missing");
        assert!(
            directory.join(format!("{stem}-public-spki.der")).is_file(),
            "{id} matching SPKI fixture is missing"
        );
        assert!(
            directory
                .join(format!("{stem}-private-pkcs8.der"))
                .is_file(),
            "{id} matching PKCS#8 oracle fixture is missing"
        );
    }
}

#[test]
fn aes_rsa_gap_probe_is_not_rewritten_as_an_rsa_target_request() {
    let rows = target_rows().expect("cryptographic target rows");
    let row = rows
        .iter()
        .find(|row| {
            row["case_id"] == "cosmian-528-decrypt-privatekey-aes-rsa-aes-selection-discrepancy"
        })
        .expect("AES/RSA Decrypt discrepancy row");
    assert!(
        asymmetric_ciphertext_fixture_row(row).is_err(),
        "the Decrypt/AES-vs-RSA gap must not synthesize a target ciphertext fixture"
    );
    assert_eq!(
        row["algorithm"], "AES",
        "target Decrypt axes must be preserved"
    );
}

#[test]
fn direct_kem_decrypt_uses_its_algorithm_as_the_key_algorithm() {
    let row = target_rows()
        .expect("cryptographic target rows")
        .into_iter()
        .find(|row| row["case_id"] == "cosmian-528-decrypt-privatekey-mlkem-512-source-only")
        .expect("direct ML-KEM Decrypt row");

    assert_eq!(
        asymmetric_ciphertext_fixture_row(&row).expect("matching ML-KEM ciphertext fixture row"),
        row,
        "a direct KEM operation uses the same algorithm for its generated key and target request"
    );
}

#[test]
fn covercrypt_create_builders_keep_policy_and_parent_link_dimensions() {
    const MASTER_ID: &str = "offline-covercrypt-master-private-id";
    const ACCESS_POLICY: &str =
        "Security Level::Confidential && (Department::FIN || Department::HR)";
    let rows = target_rows().expect("cryptographic target rows");
    let user_row = rows
        .iter()
        .find(|row| row["case_id"] == "cosmian-528-create-privatekey-covercrypt")
        .expect("CoverCrypt user-key Create row");
    let master_row = rows
        .iter()
        .find(|row| {
            row["case_id"]
                == "cosmian-528-create-key-pair-privatekey-covercrypt-0x88800004-disposable-master-access-structure"
        })
        .expect("CoverCrypt master Create Key Pair row");

    let user_payload = build_create_request(user_row, Some(MASTER_ID))
        .expect("CoverCrypt Create request")
        .into_ttlv_payload()
        .expect("CoverCrypt Create TTLV payload");
    assert_eq!(
        vendor_attribute_bytes(user_payload.view(), "cover_crypt_access_policy"),
        Some(ACCESS_POLICY.as_bytes().to_vec()),
        "CoverCrypt Create must carry the exact case policy bytes"
    );
    assert!(
        structure_contains_text(user_payload.view(), LINKED_OBJECT_IDENTIFIER, MASTER_ID),
        "CoverCrypt user key must link to the matching master private key"
    );

    let master_payload = row_key_pair_request(
        master_row,
        case_id(master_row).unwrap(),
        KEY_USAGE_DECRYPT,
        KEY_USAGE_ENCRYPT,
    )
    .expect("CoverCrypt Create Key Pair request")
    .into_ttlv_payload()
    .expect("CoverCrypt Create Key Pair TTLV payload");
    let policy = vendor_attribute_bytes(master_payload.view(), "cover_crypt_policy")
        .expect("CoverCrypt master policy vendor attribute");
    let parsed_policy: JsonValue =
        serde_json::from_slice(&policy).expect("CoverCrypt master policy JSON bytes");
    assert_eq!(parsed_policy["version"], "V2");
    assert_eq!(parsed_policy["last_attribute_value"], 7);
    assert_eq!(
        parsed_policy["dimensions"]["Security Level"]["order"],
        serde_json::json!(["Protected", "Confidential", "Top Secret"])
    );
    assert_eq!(
        parsed_policy["dimensions"]["Department"]["attributes"]
            .as_object()
            .map(serde_json::Map::len),
        Some(4)
    );
    assert!(
        parsed_policy["dimensions"]["Security Level"]["attributes"]["Top Secret"]["encryption_hint"]
            == "Hybridized",
        "CoverCrypt master policy must preserve the hybridized Top Secret dimension"
    );
    assert!(
        master_payload
            .view()
            .children()
            .iter()
            .all(|item| item.tag().raw() != KEY_FORMAT_TYPE_ATTRIBUTE),
        "Create Key Pair row does not request a Key Format Type"
    );
}

fn byte_string_field_length(structure: &Structure, tag: u32) -> Option<usize> {
    structure
        .view()
        .children()
        .iter()
        .find(|item| item.tag().raw() == tag)
        .and_then(|item| {
            item.with_value(|value| match value {
                kmipkit_ttlv::ValueView::ByteString(bytes) => Some(bytes.len()),
                _ => None,
            })
        })
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// The exact OASIS operation citation for every row is validated against that
// row's `oasis_source` in the offline contract test.
fn cosmian_528_cryptographic_matrix_serial_live_runner() {
    let rows = target_rows().unwrap_or_else(|error| panic!("{error}"));
    let selected = selected_rows(&rows).unwrap_or_else(|error| panic!("{error}"));
    let selected_count = selected.len();
    let mut failures = Vec::new();

    for row in selected {
        let id = case_id(row).unwrap_or_else(|error| panic!("{error}"));
        if matches!(
            disposition(row),
            Ok(CaseDisposition::ExpectedFindingProbe { .. })
        ) {
            match finding_probe_preflight(row) {
                Ok(FindingProbePreflight::SendExactTarget) => {}
                Ok(FindingProbePreflight::NotSent(reason)) => {
                    let expected_outcome = row["expected_outcome"]
                        .as_str()
                        .unwrap_or("unknown_expected_outcome");
                    println!(
                        "case_id={id} classification=expected_finding_probe expected_outcome={expected_outcome} observed=preflight_not_sent request_sent=false interoperability=false preflight_reason={reason}"
                    );
                    failures.push(format!("{id} (preflight NotSent: {reason})"));
                    continue;
                }
                Err(error) => {
                    println!(
                        "case_id={id} classification=preflight_error request_sent=false interoperability=false detail={error}"
                    );
                    failures.push(format!("{id} (invalid finding probe: {error})"));
                    continue;
                }
            }
        }
        let mut client = match super::client() {
            Ok(client) => client,
            Err(_) => {
                match disposition(row) {
                    Ok(CaseDisposition::Executable(_)) => {
                        println!("case_id={id} classification=setup_blocked request_sent=false");
                    }
                    Ok(CaseDisposition::ExpectedFindingProbe {
                        operation,
                        expected_outcome,
                    }) => println!(
                        "case_id={id} classification=expected_finding_probe expected_outcome={expected_outcome} observed=setup_blocked request_sent=false interoperability=false operation={operation}"
                    ),
                    Err(_) => println!("case_id={id} classification=registry_error"),
                }
                failures.push(format!("{id} (fresh client setup failed before request)"));
                continue;
            }
        };
        match disposition(row) {
            Ok(CaseDisposition::Executable(_)) => match dispatch_case(row, Some(&mut client)) {
                Ok(()) => println!("case_id={id} classification=typed_success"),
                Err(error) => {
                    let detail = error.to_string();
                    let classification = classify_case_error(row, &detail);
                    println!("case_id={id} classification={classification} detail={error}");
                    failures.push(format!("{id} ({classification}): {error}"));
                }
            },
            Ok(CaseDisposition::ExpectedFindingProbe {
                operation,
                expected_outcome,
            }) => match dispatch_operation(row, Some(&mut client), operation) {
                Ok(()) => println!(
                    "case_id={id} classification=expected_finding_probe expected_outcome={expected_outcome} observed=typed_success request_sent=true interoperability=false"
                ),
                Err(error) => {
                    let detail = error.to_string();
                    let delivery = target_operation_delivery(row, operation, &detail);
                    let observed = match delivery {
                        TargetOperationDelivery::ResponseReceived => {
                            classify_case_error(row, &detail)
                        }
                        TargetOperationDelivery::SetupResponseReceived => "setup_rejected",
                        TargetOperationDelivery::NotSent => "setup_or_client_blocked",
                        TargetOperationDelivery::Uncertain => "delivery_uncertain",
                    };
                    let request_sent = match delivery {
                        TargetOperationDelivery::ResponseReceived => "true",
                        TargetOperationDelivery::SetupResponseReceived
                        | TargetOperationDelivery::NotSent => "false",
                        TargetOperationDelivery::Uncertain => "unknown",
                    };
                    println!(
                        "case_id={id} classification=expected_finding_probe expected_outcome={expected_outcome} observed={observed} request_sent={request_sent} setup_response_received={} interoperability=false detail={detail}",
                        delivery == TargetOperationDelivery::SetupResponseReceived
                    );
                    if delivery != TargetOperationDelivery::ResponseReceived {
                        failures.push(format!(
                            "{id} (finding probe did not reach target operation): {detail}"
                        ));
                    }
                }
            },
            Err(error) => {
                println!("case_id={id} classification=registry_error detail={error}");
                failures.push(format!("{id} (registry_error): {error}"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} selected cryptographic matrix cases are not verified: {}",
        failures.len(),
        selected_count,
        failures.join("; ")
    );
}

fn classify_case_error(row: &JsonValue, message: &str) -> &'static str {
    let id = row["case_id"].as_str().unwrap_or_default();
    let operation = row["operation"].as_str().unwrap_or_default();
    let target_response = message.contains(&format!("case {id} {operation}:"));
    if message.contains("KMIP Result Status") {
        if target_response {
            "target_rejected"
        } else {
            "setup_rejected"
        }
    } else if message.contains("typed response rejected by KMIPKit") {
        if target_response {
            "response_rejected"
        } else {
            "setup_response_rejected"
        }
    } else if message.contains("client_blocked before a usable response") {
        if target_response {
            "client_blocked"
        } else {
            "setup_blocked"
        }
    } else if message.contains("fixture is unavailable")
        || message.contains("has no local fixture")
        || message.contains("required CSR fixture")
        || message.contains("required certificate fixture")
    {
        "setup_blocked"
    } else if message.contains("did not match")
        || message.contains("returned")
        || message.contains("has no ")
        || message.contains("unsupported")
    {
        "semantic_mismatch"
    } else {
        "uncovered"
    }
}

fn target_rows() -> Result<Vec<JsonValue>, Box<dyn Error>> {
    let rows = super::common::manifest_rows(FAMILY)?;
    Ok(rows
        .into_iter()
        .filter(|row| {
            [
                "typed_success",
                "source_documentation_discrepancy",
                "documented_but_source_gap",
            ]
            .contains(&row["expected_outcome"].as_str().unwrap_or_default())
                && row["operation"]
                    .as_str()
                    .is_some_and(|operation| OPERATIONS.contains(&operation))
        })
        .collect())
}

fn case_id(row: &JsonValue) -> Result<&str, Box<dyn Error>> {
    row["case_id"]
        .as_str()
        .ok_or_else(|| "cryptographic manifest row has no textual case_id".into())
}

fn case_ids() -> impl Iterator<Item = &'static str> {
    CASE_IDS
        .lines()
        .map(str::trim)
        .filter(|case_id| !case_id.is_empty())
}

fn validate_operation_contract(row: &JsonValue) -> Result<(), String> {
    let id = row["case_id"].as_str().unwrap_or("<missing case_id>");
    let operation = row["operation"]
        .as_str()
        .ok_or_else(|| format!("case {id} has no operation"))?;
    let algorithm = row["algorithm"].as_str().unwrap_or_default();
    let parameters = row["parameters"]
        .as_object()
        .ok_or_else(|| format!("case {id} has no parameter object"))?;
    let allowed_parameters = match operation {
        "Encrypt" | "Decrypt" => &ENCRYPTION_PARAMETERS[..],
        "Create" => &CREATE_PARAMETERS[..],
        "Create Key Pair" => &CREATE_KEY_PAIR_PARAMETERS[..],
        "Certify" => &CERTIFY_PARAMETERS[..],
        "Hash" => &HASH_PARAMETERS[..],
        "MAC" | "MAC Verify" => &MAC_PARAMETERS[..],
        "Sign" => &SIGN_PARAMETERS[..],
        "Signature Verify" => &VERIFY_PARAMETERS[..],
        other => return Err(format!("case {id} has unsupported operation {other}")),
    };
    for parameter in parameters.keys() {
        if !allowed_parameters.contains(&parameter.as_str()) {
            return Err(format!(
                "case {id} {operation} has unrecognized parameter axis {parameter}"
            ));
        }
    }
    validate_parameter_value_shapes(id, operation, parameters)?;

    let citation = operation_citation(operation)?;
    let sources = row["oasis_source"]
        .as_array()
        .ok_or_else(|| format!("case {id} has no oasis_source array"))?;
    if sources.first().and_then(JsonValue::as_str) != Some(citation) {
        return Err(format!(
            "case {id} {operation} must retain its matrix operation citation {citation}"
        ));
    }
    if sources
        .iter()
        .any(|source| source.as_str().is_none_or(str::is_empty))
    {
        return Err(format!("case {id} has an empty or non-text OASIS citation"));
    }

    validate_algorithm_contract(row, operation, algorithm, parameters)
}

fn operation_citation(operation: &str) -> Result<&'static str, String> {
    match operation {
        "Certify" => Ok("oasis-6.1.6-t179-180-181"),
        "Create" => Ok("oasis-6.1.8-t186-187-188"),
        "Create Key Pair" => Ok("oasis-6.1.9-t189-190-191-192"),
        "Encrypt" => Ok("oasis-6.1.17-t214-215-216"),
        "Decrypt" => Ok("oasis-6.1.11-t196-197-198"),
        "Hash" => Ok("oasis-6.1.24-t235-236-237"),
        "MAC" => Ok("oasis-6.1.32-t259-260-261"),
        "MAC Verify" => Ok("oasis-6.1.33-t262-263-264"),
        "Sign" => Ok("oasis-6.1.55-t334-335-336"),
        "Signature Verify" => Ok("oasis-6.1.56-t337-338-339"),
        other => Err(format!("no operation citation registered for {other}")),
    }
}

fn validate_algorithm_contract(
    row: &JsonValue,
    operation: &str,
    algorithm: &str,
    parameters: &serde_json::Map<String, JsonValue>,
) -> Result<(), String> {
    let id = row["case_id"].as_str().unwrap_or("<missing case_id>");
    match operation {
        "Hash" => validate_hash_contract(row, id, algorithm, parameters)?,
        "MAC" | "MAC Verify" => {
            validate_mac_contract(row, id, operation, algorithm, parameters)?;
        }
        "Encrypt" | "Decrypt" => {
            validate_cipher_contract(row, id, operation, algorithm)?;
        }
        "Sign" | "Signature Verify" => {
            validate_signature_contract(row, id, operation, algorithm, parameters)?;
        }
        "Create" => {
            if let Some(algorithm_name) =
                row["wire_enum_values"]["cryptographic_algorithm"]["name"].as_str()
            {
                require_wire_enum(row, "cryptographic_algorithm", algorithm_name)?;
            }
            if parameters.get("requested_key_format").is_some_and(|value| {
                value
                    .as_str()
                    .is_some_and(|format| format != "TransparentSymmetricKey")
            }) {
                return Err(format!(
                    "case {id} Create requests an unsupported key format axis"
                ));
            }
        }
        "Create Key Pair" => {
            require_wire_enum(
                row,
                "cryptographic_algorithm",
                row["wire_enum_values"]["cryptographic_algorithm"]["name"]
                    .as_str()
                    .ok_or_else(|| format!("case {id} Create Key Pair has no algorithm enum"))?,
            )?;
        }
        "Certify" => {
            let subject_input = parameters
                .get("subject_input")
                .and_then(JsonValue::as_str)
                .ok_or_else(|| format!("case {id} Certify has no subject input"))?;
            match subject_input {
                "csr-pem" => require_wire_enum(row, "certificate_request_type", "PEM")?,
                "csr-pkcs10-der" => {
                    require_wire_enum(row, "certificate_request_type", "PKCS10")?;
                }
                "existing-public" | "generated-subject" | "renew-certificate" => {}
                other => {
                    return Err(format!(
                        "case {id} has unknown Certify subject input {other}"
                    ));
                }
            }
            require_wire_enum(row, "key_format_type", "X509")?;
        }
        _ => return Err(format!("case {id} has unsupported operation {operation}")),
    }
    Ok(())
}

fn validate_hash_contract(
    row: &JsonValue,
    id: &str,
    algorithm: &str,
    parameters: &serde_json::Map<String, JsonValue>,
) -> Result<(), String> {
    const HASHES: [&str; 8] = [
        "SHA-224", "SHA-256", "SHA-384", "SHA-512", "SHA3-224", "SHA3-256", "SHA3-384", "SHA3-512",
    ];
    if !HASHES.contains(&algorithm)
        || parameters.get("hash").and_then(JsonValue::as_str) != Some(algorithm)
        || !parameters
            .get("data_mode")
            .and_then(JsonValue::as_str)
            .is_some_and(|mode| ["one-shot", "init-update-final"].contains(&mode))
    {
        return Err(format!(
            "case {id} Hash violates its declared algorithm/data-mode contract"
        ));
    }
    if parameters
        .get("data_length_bytes")
        .is_some_and(|length| length.as_u64() != Some(0))
    {
        return Err(format!(
            "case {id} Hash data_length_bytes must be absent or zero"
        ));
    }
    require_wire_enum(row, "hashing_algorithm", algorithm)
}

fn validate_mac_contract(
    row: &JsonValue,
    id: &str,
    operation: &str,
    algorithm: &str,
    parameters: &serde_json::Map<String, JsonValue>,
) -> Result<(), String> {
    let hash = parameters.get("hash").and_then(JsonValue::as_str);
    if algorithm.strip_prefix("HMAC-") != hash {
        return Err(format!(
            "case {id} {operation} HMAC algorithm/hash mismatch"
        ));
    }
    let streaming =
        parameters.get("data_mode").and_then(JsonValue::as_str) == Some("init-update-final");
    let selection = parameters.get("hash_selection").and_then(JsonValue::as_str);
    let selection_valid = if streaming {
        selection.is_none()
    } else {
        selection.is_some_and(|selection| {
            [
                "attribute-algorithm",
                "explicit-hash",
                "key-block-algorithm",
            ]
            .contains(&selection)
        })
    };
    if !selection_valid {
        return Err(format!(
            "case {id} {operation} has an invalid hash_selection"
        ));
    }
    let expected_verification: Option<&str> = if streaming {
        None
    } else if operation == "MAC" {
        Some("compute")
    } else {
        match parameters.get("verification").and_then(JsonValue::as_str) {
            Some(value @ ("valid" | "invalid")) => Some(value),
            _ => return Err(format!("case {id} MAC Verify has unknown verification")),
        }
    };
    if parameters.get("verification").and_then(JsonValue::as_str) != expected_verification
        || parameters.get("key_bits").and_then(JsonValue::as_u64) != Some(256)
    {
        return Err(format!(
            "case {id} {operation} violates its HMAC parameters"
        ));
    }
    if parameters
        .get("data_mode")
        .and_then(JsonValue::as_str)
        .is_some_and(|mode| !["one-shot", "init-update-final"].contains(&mode))
    {
        return Err(format!("case {id} {operation} has an unknown data_mode"));
    }
    let wire_name = match algorithm {
        "HMAC-SHA-1" => "HMACSHA1",
        "HMAC-SHA-224" => "HMACSHA224",
        "HMAC-SHA-256" => "HMACSHA256",
        "HMAC-SHA-384" => "HMACSHA384",
        "HMAC-SHA-512" => "HMACSHA512",
        "HMAC-SHA3-224" => "HMACSHA3224",
        "HMAC-SHA3-256" => "HMACSHA3256",
        "HMAC-SHA3-384" => "HMACSHA3384",
        "HMAC-SHA3-512" => "HMACSHA3512",
        _ => return Err(format!("case {id} has unknown HMAC algorithm {algorithm}")),
    };
    require_wire_enum(row, "cryptographic_algorithm", wire_name)
}

fn validate_cipher_contract(
    row: &JsonValue,
    id: &str,
    operation: &str,
    algorithm: &str,
) -> Result<(), String> {
    let expected_objects = if operation == "Encrypt" {
        ["Certificate", "PublicKey", "SymmetricKey"].as_slice()
    } else {
        ["PrivateKey", "SymmetricKey"].as_slice()
    };
    if !row["object_type"]
        .as_str()
        .is_some_and(|object_type| expected_objects.contains(&object_type))
    {
        return Err(format!("case {id} {operation} has an invalid object type"));
    }
    require_wire_enum(row, "cryptographic_algorithm", algorithm)
}

fn validate_signature_contract(
    row: &JsonValue,
    id: &str,
    operation: &str,
    algorithm: &str,
    parameters: &serde_json::Map<String, JsonValue>,
) -> Result<(), String> {
    let expected_objects = if operation == "Sign" {
        ["PrivateKey"].as_slice()
    } else {
        ["PrivateKey", "PublicKey"].as_slice()
    };
    let source_only_numeric_probe = row["expected_outcome"] == "source_documentation_discrepancy"
        && row["wire_enum_values"]["cryptographic_algorithm"]["classification"]
            == "source_only_numeric_probe";
    if (!source_only_numeric_probe && !["RSA", "ECDSA", "Ed25519", "Ed448"].contains(&algorithm))
        || !row["object_type"]
            .as_str()
            .is_some_and(|object_type| expected_objects.contains(&object_type))
        || !parameters
            .get("data_mode")
            .and_then(JsonValue::as_str)
            .is_some_and(|mode| ["raw", "prehashed", "init-update-final"].contains(&mode))
    {
        return Err(format!(
            "case {id} {operation} violates its key/data contract"
        ));
    }
    require_wire_enum(row, "cryptographic_algorithm", algorithm)
}

fn validate_parameter_value_shapes(
    id: &str,
    operation: &str,
    parameters: &serde_json::Map<String, JsonValue>,
) -> Result<(), String> {
    for (name, value) in parameters {
        let valid = match name.as_str() {
            "key_bits" | "subject_key_bits" | "issuer_key_bits" | "iv_length_bits"
            | "tag_length_bytes" | "data_length_bytes" => {
                value.is_null() || value.as_u64().is_some()
            }
            "salt_length_bytes" => {
                value.is_null() || value.as_u64().is_some() || value.as_str() == Some("digest")
            }
            "non_fips"
            | "deterministic"
            | "external_digest"
            | "accept_sha256_substitution"
            | "verify_requested_digest_independently" => value.is_null() || value.is_boolean(),
            "post_quantum" => value.is_null() || value.as_str().is_some(),
            "opaque_fixture_configuration" | "wire_container" => {
                value.is_null() || value.is_object()
            }
            "output_bytes" => value.is_null() || value.as_u64().is_some(),
            _ => value.is_null() || value.as_str().is_some(),
        };
        if !valid {
            return Err(format!(
                "case {id} {operation} parameter {name} has an invalid JSON value type"
            ));
        }
    }
    Ok(())
}

fn require_wire_enum(row: &JsonValue, member: &str, expected_name: &str) -> Result<(), String> {
    let id = row["case_id"].as_str().unwrap_or("<missing case_id>");
    let enum_name = row["wire_enum_values"][member]["name"].as_str();
    let enum_value = row["wire_enum_values"][member]["value"].as_str();
    let parsed_value =
        enum_value.and_then(|value| u32::from_str_radix(value.trim_start_matches("0x"), 16).ok());
    if enum_name != Some(expected_name) || parsed_value.is_none() {
        return Err(format!(
            "case {id} has no usable {member} wire enum for {expected_name}"
        ));
    }
    Ok(())
}

fn disposition(row: &JsonValue) -> Result<CaseDisposition, String> {
    let id = row["case_id"].as_str().unwrap_or("<missing case_id>");
    match row["operation"].as_str() {
        Some(operation) if OPERATIONS.contains(&operation) => {
            let operation = OPERATIONS
                .iter()
                .copied()
                .find(|candidate| *candidate == operation)
                .ok_or_else(|| format!("case {id} has unmapped operation {operation}"))?;
            match row["expected_outcome"].as_str() {
                Some("typed_success") => Ok(CaseDisposition::Executable(operation)),
                Some("documented_but_source_gap") => Ok(CaseDisposition::ExpectedFindingProbe {
                    operation,
                    expected_outcome: "documented_but_source_gap",
                }),
                Some("source_documentation_discrepancy") => {
                    Ok(CaseDisposition::ExpectedFindingProbe {
                        operation,
                        expected_outcome: "source_documentation_discrepancy",
                    })
                }
                Some(expected_outcome) => Err(format!(
                    "case {id} has unsupported expected outcome {expected_outcome}"
                )),
                None => Err(format!("case {id} has no expected outcome")),
            }
        }
        Some(operation) => Err(format!("case {id} has unmapped operation {operation}")),
        None => Err(format!("case {id} has no operation for dispatch")),
    }
}

fn finding_probe_preflight(row: &JsonValue) -> Result<FindingProbePreflight, String> {
    let id = row["case_id"].as_str().unwrap_or("<missing case_id>");
    let operation = row["operation"].as_str().unwrap_or_default();
    if row["expected_outcome"] == "typed_success" {
        return Err(format!("case {id} is not an expected-finding probe"));
    }

    if row["algorithm"] == "ConfigurableKEM" && matches!(operation, "Encrypt" | "Decrypt") {
        return Ok(FindingProbePreflight::NotSent(
            "opaque ConfigurableKEM key fixture is unavailable",
        ));
    }
    if row["algorithm"] == "AES" && row["parameters"]["key_algorithm"] == "RSA" {
        let reason = if operation == "Decrypt" {
            "exact AES/RSA hybrid ciphertext fixture is unavailable"
        } else {
            "exact AES/RSA hybrid request fixture is unavailable"
        };
        return Ok(FindingProbePreflight::NotSent(reason));
    }
    if operation == "Certify"
        && row["algorithm"].as_str().is_some_and(|algorithm| {
            algorithm.starts_with("MLDSA_") || algorithm.starts_with("SLHDSA_")
        })
    {
        let fixture_path =
            validation_fixture_path(row, "csr").map_err(|error| format!("case {id}: {error}"))?;
        if !fixture_path.is_file() {
            return Ok(FindingProbePreflight::NotSent(
                "matching post-quantum CSR fixture is unavailable",
            ));
        }
    }

    validate_exact_finding_probe_dimensions(row)?;
    Ok(FindingProbePreflight::SendExactTarget)
}

fn validate_exact_finding_probe_dimensions(row: &JsonValue) -> Result<(), String> {
    let id = row["case_id"].as_str().unwrap_or("<missing case_id>");
    let operation = row["operation"].as_str().unwrap_or_default();
    match operation {
        "Create" => {
            let algorithm = wire_enum_value(row, "cryptographic_algorithm")
                .map_err(|error| format!("case {id}: {error}"))?;
            if row["algorithm"] == "DSA" {
                let bits = row["parameters"]["key_bits"]
                    .as_i64()
                    .ok_or_else(|| format!("case {id} DSA Create has no key size"))?;
                if !matches!(bits, 2048 | 3072)
                    || algorithm
                        != wire_enum_value(row, "cryptographic_algorithm")
                            .map_err(|error| error.to_string())?
                {
                    return Err(format!(
                        "case {id} DSA Create dimensions do not match its wire request"
                    ));
                }
            }
        }
        "Hash" => {
            let algorithm = required_parameter_text(row, "hash")
                .map_err(|error| format!("case {id}: {error}"))?;
            if algorithm != row["algorithm"].as_str().unwrap_or_default() {
                return Err(format!("case {id} Hash algorithm and parameter disagree"));
            }
            hashing_algorithm_value(algorithm).map_err(|error| format!("case {id}: {error}"))?;
            hash_parameters(row).map_err(|error| format!("case {id}: {error}"))?;
            let empty_input = row["parameters"]["data_length_bytes"].as_u64() == Some(0);
            known_answer_digest(algorithm, empty_input)
                .map_err(|error| format!("case {id}: {error}"))?;
        }
        "Create Key Pair" => {
            build_create_key_pair_request(row, id)
                .and_then(|request| request.into_ttlv_payload().map_err(Into::into))
                .map_err(|error| format!("case {id}: {error}"))?;
        }
        "Encrypt" | "Decrypt" => {
            asymmetric_cipher_parameters(row).map_err(|error| format!("case {id}: {error}"))?;
        }
        "MAC" => {
            let hash = required_parameter_text(row, "hash")
                .map_err(|error| format!("case {id}: {error}"))?;
            known_answer_hmac(hash).map_err(|error| format!("case {id}: {error}"))?;
            mac_parameters(row).map_err(|error| format!("case {id}: {error}"))?;
            if !matches!(
                row["parameters"]["data_mode"].as_str(),
                Some("one-shot" | "init-update-final")
            ) {
                return Err(format!("case {id} MAC has no explicit data mode"));
            }
        }
        "Certify" => {
            let algorithm = required_case_text(row, "algorithm")
                .map_err(|error| format!("case {id}: {error}"))?;
            if !algorithm.starts_with("MLDSA_") && !algorithm.starts_with("SLHDSA_") {
                return Err(format!(
                    "case {id} Certify finding has no reviewed post-quantum CSR builder"
                ));
            }
            if row["parameters"]["algorithm_dimension"]
                != "subject ASN.1 OID only; no CryptographicAlgorithm enumeration is sent"
                || !row["wire_enum_values"]["cryptographic_algorithm"]["value"].is_null()
                || row["wire_enum_values"]["cryptographic_algorithm"]["classification"]
                    != "not_sent_asn1_oid_only"
            {
                return Err(format!(
                    "case {id} Certify must preserve the CSR algorithm OID without a KMIP Cryptographic Algorithm enum"
                ));
            }
            let subject_input = required_parameter_text(row, "subject_input")
                .map_err(|error| format!("case {id}: {error}"))?;
            let (expected_name, expected_value) = match subject_input {
                // OASIS KMIP v2.1 §11.8, Table 439 assigns distinct types:
                // PKCS#10 is 2; PEM is 3.
                "csr-pem" => ("PEM", 3),
                "csr-pkcs10-der" => ("PKCS10", 2),
                other => {
                    return Err(format!(
                        "case {id} Certify finding has unsupported CSR input {other}"
                    ));
                }
            };
            if row["wire_enum_values"]["certificate_request_type"]["name"] != expected_name
                || wire_enum_value(row, "certificate_request_type")
                    .map_err(|error| format!("case {id}: {error}"))?
                    != expected_value
            {
                return Err(format!(
                    "case {id} Certify request type does not match its KMIP CSR type"
                ));
            }
            if row["wire_enum_values"]["key_format_type"]["name"] != "X509"
                || wire_enum_value(row, "key_format_type")
                    .map_err(|error| format!("case {id}: {error}"))?
                    != 5
            {
                return Err(format!("case {id} Certify finding is not an X.509 request"));
            }
            build_offline_certify_request(
                row,
                UniqueIdentifier::TextString(format!("offline-{id}")),
            )
            .and_then(|request| request.into_ttlv_payload().map_err(Into::into))
            .map_err(|error| format!("case {id} Certify request: {error}"))?;
        }
        "Sign" | "Signature Verify" => {
            signature_parameters(row).map_err(|error| format!("case {id}: {error}"))?;
            if row["parameters"]["data_mode"] != "init-update-final" {
                signature_message(row).map_err(|error| format!("case {id}: {error}"))?;
            }
        }
        other => {
            return Err(format!(
                "case {id} has no exact finding-probe builder for {other}"
            ));
        }
    }
    Ok(())
}

fn selected_rows(rows: &[JsonValue]) -> Result<Vec<&JsonValue>, String> {
    let Some(filter) = std::env::var("KMIPKIT_COSMIAN_CASE_FILTER").ok() else {
        return Ok(rows.iter().collect());
    };
    let selected = filter.split(',').map(str::trim).collect::<BTreeSet<_>>();
    for id in &selected {
        if !case_ids().any(|listed| listed == *id) {
            return Err(format!(
                "KMIPKIT_COSMIAN_CASE_FILTER contains unknown cryptographic case ID: {id}"
            ));
        }
    }
    Ok(selected
        .into_iter()
        .filter_map(|id| rows.iter().find(|row| row["case_id"] == id))
        .collect())
}

fn build_offline_target_payloads(row: &JsonValue) -> Result<Vec<Structure>, Box<dyn Error>> {
    let id = case_id(row)?;
    let operation = required_case_text(row, "operation")?;
    let identifier = UniqueIdentifier::TextString(format!("offline-{id}"));
    let payloads = match operation {
        "Create" => vec![
            build_create_request(
                row,
                (row["algorithm"] == "CoverCrypt").then_some("offline-covercrypt-master-id"),
            )?
            .into_ttlv_payload()?,
        ],
        "Create Key Pair" => vec![build_create_key_pair_request(row, id)?.into_ttlv_payload()?],
        "Certify" => vec![build_offline_certify_request(row, identifier)?.into_ttlv_payload()?],
        "Encrypt" | "Decrypt" if row["object_type"] == "SymmetricKey" => {
            vec![super::cryptographic_cipher::offline_target_payload(row)?]
        }
        "Encrypt" | "Decrypt" => {
            let data = if row["algorithm"] == "CoverCrypt" && operation == "Encrypt" {
                super::cryptographic_covercrypt::encode_data_to_encrypt(
                    covercrypt_access_policy(),
                    &[],
                    b"offline KMIPKit cryptographic matrix payload",
                )?
            } else {
                SecretBytes::new(b"offline KMIPKit cryptographic matrix payload".to_vec())
            };
            let parameters = asymmetric_cipher_parameters(row)?;
            let payload = if operation == "Encrypt" {
                EncryptRequest::new(Some(identifier), Some(OperationData::ByteString(data)))
                    .with_cryptographic_parameters(parameters)
                    .to_ttlv_payload()?
            } else {
                DecryptRequest::new(Some(identifier), Some(OperationData::ByteString(data)))
                    .with_cryptographic_parameters(parameters)
                    .to_ttlv_payload()?
            };
            vec![payload]
        }
        "Hash" if row["parameters"]["data_mode"] == "init-update-final" => {
            let initial = HashRequest::new(hash_parameters(row)?).with_init_indicator(true);
            let middle = HashRequest::new(hash_parameters(row)?)
                .with_correlation_value(SecretBytes::new(vec![0x5A]))
                .with_init_indicator(false);
            let final_part = HashRequest::new(hash_parameters(row)?)
                .with_correlation_value(SecretBytes::new(vec![0x5A]))
                .with_final_indicator(true);
            vec![
                initial.to_ttlv_payload()?,
                middle.to_ttlv_payload()?,
                final_part.to_ttlv_payload()?,
            ]
        }
        "Hash" => {
            let data = if row["parameters"]["data_length_bytes"].as_u64() == Some(0) {
                Vec::new()
            } else {
                b"abc".to_vec()
            };
            vec![
                HashRequest::new(hash_parameters(row)?)
                    .with_data(OperationData::ByteString(SecretBytes::new(data)))
                    .to_ttlv_payload()?,
            ]
        }
        "MAC" | "MAC Verify" => build_offline_mac_payloads(row, identifier)?,
        "Sign" | "Signature Verify" => build_offline_signature_payloads(row, identifier)?,
        other => {
            return Err(format!("case {id} has no offline payload builder for {other}").into());
        }
    };
    Ok(payloads)
}

fn build_offline_certify_request(
    row: &JsonValue,
    identifier: UniqueIdentifier,
) -> Result<CertifyRequest, Box<dyn Error>> {
    let subject_input = required_parameter_text(row, "subject_input")?;
    let mut request = CertifyRequest::new().with_attributes(certify_attributes(row)?);
    match subject_input {
        "csr-pem" | "csr-pkcs10-der" => {
            let request_type = wire_enum_value(row, "certificate_request_type")?;
            request = request
                .with_certificate_request_type(request_type)
                .with_certificate_request_value(SecretBytes::new(
                    b"offline-only CSR shape fixture".to_vec(),
                ));
        }
        "existing-public" | "generated-subject" | "renew-certificate" => {
            request = request.with_unique_identifier(identifier);
        }
        other => {
            return Err(format!(
                "case {} has unsupported Certify input {other}",
                case_id(row)?
            )
            .into());
        }
    }
    Ok(request)
}

fn build_offline_mac_payloads(
    row: &JsonValue,
    identifier: UniqueIdentifier,
) -> Result<Vec<Structure>, Box<dyn Error>> {
    let operation = required_case_text(row, "operation")?;
    if row["parameters"]["data_mode"] == "init-update-final" {
        let mut initial = MacRequest::new()
            .with_unique_identifier(identifier.clone())
            .with_init_indicator(true);
        let mut middle = MacRequest::new()
            .with_unique_identifier(identifier.clone())
            .with_correlation_value(SecretBytes::new(vec![0x5A]))
            .with_init_indicator(false);
        let mut final_part = MacRequest::new()
            .with_unique_identifier(identifier)
            .with_correlation_value(SecretBytes::new(vec![0x5A]))
            .with_final_indicator(true);
        if let Some(parameters) = mac_parameters(row)? {
            initial = initial.with_cryptographic_parameters(parameters);
        }
        if let Some(parameters) = mac_parameters(row)? {
            middle = middle.with_cryptographic_parameters(parameters);
        }
        if let Some(parameters) = mac_parameters(row)? {
            final_part = final_part.with_cryptographic_parameters(parameters);
        }
        return Ok(vec![
            initial.to_ttlv_payload()?,
            middle.to_ttlv_payload()?,
            final_part.to_ttlv_payload()?,
        ]);
    }

    let data = SecretBytes::new(b"abc".to_vec());
    let parameters = mac_parameters(row)?;
    let payload = if operation == "MAC" {
        let mut request = MacRequest::new()
            .with_unique_identifier(identifier)
            .with_data(OperationData::ByteString(data));
        if let Some(parameters) = parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        request.to_ttlv_payload()?
    } else {
        let hash = required_parameter_text(row, "hash")?;
        let mut request = MacVerifyRequest::new()
            .with_unique_identifier(identifier)
            .with_data(OperationData::ByteString(data))
            .with_mac_data(SecretBytes::new(decode_hex(known_answer_hmac(hash)?)?));
        if let Some(parameters) = parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        request.to_ttlv_payload()?
    };
    Ok(vec![payload])
}

fn build_offline_signature_payloads(
    row: &JsonValue,
    identifier: UniqueIdentifier,
) -> Result<Vec<Structure>, Box<dyn Error>> {
    let operation = required_case_text(row, "operation")?;
    if row["parameters"]["data_mode"] == "init-update-final" {
        if operation == "Sign" {
            return Ok(vec![
                SignRequest::new()
                    .with_unique_identifier(identifier.clone())
                    .with_cryptographic_parameters(signature_parameters(row)?)
                    .with_init_indicator(true)
                    .to_ttlv_payload()?,
                SignRequest::new()
                    .with_unique_identifier(identifier.clone())
                    .with_cryptographic_parameters(signature_parameters(row)?)
                    .with_correlation_value(SecretBytes::new(vec![0x5A]))
                    .with_init_indicator(false)
                    .to_ttlv_payload()?,
                SignRequest::new()
                    .with_unique_identifier(identifier)
                    .with_cryptographic_parameters(signature_parameters(row)?)
                    .with_correlation_value(SecretBytes::new(vec![0x5A]))
                    .with_final_indicator(true)
                    .to_ttlv_payload()?,
            ]);
        }
        return Ok(vec![
            SignatureVerifyRequest::new()
                .with_unique_identifier(identifier.clone())
                .with_cryptographic_parameters(signature_parameters(row)?)
                .with_init_indicator(true)
                .to_ttlv_payload()?,
            SignatureVerifyRequest::new()
                .with_unique_identifier(identifier.clone())
                .with_cryptographic_parameters(signature_parameters(row)?)
                .with_correlation_value(SecretBytes::new(vec![0x5A]))
                .with_init_indicator(false)
                .to_ttlv_payload()?,
            SignatureVerifyRequest::new()
                .with_unique_identifier(identifier)
                .with_cryptographic_parameters(signature_parameters(row)?)
                .with_correlation_value(SecretBytes::new(vec![0x5A]))
                .with_final_indicator(true)
                .to_ttlv_payload()?,
        ]);
    }

    let message = signature_message(row)?;
    let parameters = signature_parameters(row)?;
    let payload = if operation == "Sign" {
        let request = SignRequest::new()
            .with_unique_identifier(identifier)
            .with_cryptographic_parameters(parameters);
        match message {
            SignatureMessage::Raw(data) => request
                .with_data(OperationData::ByteString(SecretBytes::new(data)))
                .to_ttlv_payload()?,
            SignatureMessage::Digested(data) => request
                .with_digested_data(SecretBytes::new(data))
                .to_ttlv_payload()?,
        }
    } else {
        let request = SignatureVerifyRequest::new()
            .with_unique_identifier(identifier)
            .with_cryptographic_parameters(parameters)
            .with_signature_data(SecretBytes::new(vec![0x5A; 64]));
        match message {
            SignatureMessage::Raw(data) => request
                .with_data(OperationData::ByteString(SecretBytes::new(data)))
                .to_ttlv_payload()?,
            SignatureMessage::Digested(data) => request
                .with_digested_data(SecretBytes::new(data))
                .to_ttlv_payload()?,
        }
    };
    Ok(vec![payload])
}

fn dispatch_case(
    row: &JsonValue,
    client: Option<&mut kmipkit_client::Client>,
) -> Result<(), Box<dyn Error>> {
    match disposition(row).map_err(|error| -> Box<dyn Error> { error.into() })? {
        CaseDisposition::Executable(operation) => dispatch_operation(row, client, operation),
        CaseDisposition::ExpectedFindingProbe { operation, .. } => {
            dispatch_operation(row, client, operation)
        }
    }
}

fn dispatch_operation(
    row: &JsonValue,
    client: Option<&mut kmipkit_client::Client>,
    operation: &str,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    let client =
        client.ok_or_else(|| format!("case {id} {operation} has no initialized Cosmian client"))?;
    if !has_typed_executor(operation) {
        return Err(format!("case {id} has no executor for {operation}").into());
    }
    match operation {
        "Hash" => run_hash_case(client, row),
        "MAC" | "MAC Verify" => run_mac_family_case(client, row),
        "Create" => run_create_case(client, row),
        "Create Key Pair" => run_create_key_pair_case(client, row),
        "Certify" => run_certify_case(client, row),
        "Encrypt" | "Decrypt" => run_cipher_case(client, row),
        "Sign" | "Signature Verify" => run_signature_case(client, row),
        other => Err(format!("case {id} has no executor for {other}").into()),
    }
}

fn has_typed_executor(operation: &str) -> bool {
    matches!(
        operation,
        "Hash"
            | "MAC"
            | "MAC Verify"
            | "Create"
            | "Create Key Pair"
            | "Certify"
            | "Encrypt"
            | "Decrypt"
            | "Sign"
            | "Signature Verify"
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TargetOperationDelivery {
    ResponseReceived,
    SetupResponseReceived,
    NotSent,
    Uncertain,
}

#[test]
fn finding_probe_setup_response_is_not_misreported_as_target_response() {
    let row = serde_json::json!({"case_id": "case-1"});
    for detail in [
        "case case-1 Create Key Pair: KMIP Result Status 1 (Operation Failed)",
        "case case-1 Sign: KMIP Result Status 1 (Operation Failed)",
        "case case-1-covercrypt-master Create Key Pair: KMIP Result Status 1 (Operation Failed)",
    ] {
        assert_eq!(
            target_operation_delivery(&row, "Signature Verify", detail),
            TargetOperationDelivery::SetupResponseReceived,
            "a prerequisite response must not count as reaching Signature Verify"
        );
    }
}

fn target_operation_delivery(
    row: &JsonValue,
    operation: &str,
    detail: &str,
) -> TargetOperationDelivery {
    let id = row["case_id"].as_str().unwrap_or_default();
    let target_prefix = format!("case {id} {operation}:");
    if detail.contains(&target_prefix)
        && !detail.contains("client_blocked")
        && (detail.contains("KMIP Result Status")
            || detail.contains("typed response rejected by KMIPKit")
            || detail.contains("target response received")
            || detail.contains("expected one response item")
            || detail.contains("response item missing")
            || detail.contains("test postcondition"))
    {
        return TargetOperationDelivery::ResponseReceived;
    }
    if detail.contains(&target_prefix) && detail.contains("client_blocked") {
        return if detail.contains("delivery=NotSent") {
            TargetOperationDelivery::NotSent
        } else {
            TargetOperationDelivery::Uncertain
        };
    }
    if detail.contains("KMIP Result Status")
        || detail.contains("typed response rejected by KMIPKit")
    {
        return TargetOperationDelivery::SetupResponseReceived;
    }
    if detail.contains("has no exact")
        || detail.contains("has no local fixture")
        || detail.contains("fixture is unavailable")
        || detail.contains("manifest has no exact")
    {
        return TargetOperationDelivery::NotSent;
    }
    TargetOperationDelivery::Uncertain
}

fn build_create_request(
    row: &JsonValue,
    covercrypt_parent_id: Option<&str>,
) -> Result<CreateRequest, Box<dyn Error>> {
    let id = case_id(row)?;
    let mut attributes = vec![super::common::name_attribute(id)?];
    if let Some(value) = row["wire_enum_values"]["cryptographic_algorithm"]["value"].as_str() {
        attributes.push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(parse_raw_enum(value)?),
        )?);
    }
    if let Some(bits) = row["parameters"]["key_bits"].as_i64() {
        attributes.push(super::kmip_item(
            CRYPTOGRAPHIC_LENGTH,
            Value::integer(i32::try_from(bits)?),
        )?);
    }
    if let Some(format) = row["parameters"]["requested_key_format"].as_str() {
        let value = wire_enum_value(row, "key_format_type")?;
        if format != "TransparentSymmetricKey" || value != 7 {
            return Err(format!("case {id} has an unsupported requested key format").into());
        }
        attributes.push(super::kmip_item(
            KEY_FORMAT_TYPE_ATTRIBUTE,
            Value::enumeration(value),
        )?);
    }
    if let Some(secret_type) = row["parameters"]["output_secret_data_type"].as_str() {
        let value = match secret_type {
            "Password" => 2,
            "Seed" => 3,
            "PrivateKeyMaterial" => 1,
            other => return Err(format!("case {id} has unknown Secret Data Type {other}").into()),
        };
        attributes.push(super::kmip_item(
            SECRET_DATA_TYPE,
            Value::enumeration(value),
        )?);
    }
    if row["object_type"] == "SymmetricKey" {
        attributes.push(super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(KEY_USAGE_ENCRYPT | KEY_USAGE_DECRYPT),
        )?);
    }
    if row["algorithm"] == "CoverCrypt" {
        let parent_id = covercrypt_parent_id
            .ok_or_else(|| format!("case {id} CoverCrypt Create has no master private key"))?;
        let key_format = wire_enum_value(row, "key_format_type")?;
        let link = super::structure([
            super::kmip_item(LINK_TYPE, Value::enumeration(0x08))?,
            super::kmip_item(
                LINKED_OBJECT_IDENTIFIER,
                Value::text_string(parent_id.to_owned()),
            )?,
        ])?;
        attributes.push(super::kmip_item(
            KEY_FORMAT_TYPE_ATTRIBUTE,
            Value::enumeration(key_format),
        )?);
        attributes.push(super::kmip_item(LINK, Value::structure(link))?);
        attributes.push(super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(KEY_USAGE_DECRYPT),
        )?);
        attributes.push(covercrypt_vendor_attribute(
            "cover_crypt_access_policy",
            Value::byte_string(covercrypt_access_policy().as_bytes().to_vec()),
        )?);
    }
    Ok(CreateRequest::new(
        ObjectType::from_raw(object_type_raw(row)?),
        AttributeSet::try_new(attributes)?,
    ))
}

pub(super) fn build_create_key_pair_request(
    row: &JsonValue,
    id: &str,
) -> Result<CreateKeyPairRequest, Box<dyn Error>> {
    let algorithm = required_case_text(row, "algorithm")?;
    let (private_usage, public_usage) = if is_kem_algorithm(algorithm) || algorithm == "CoverCrypt"
    {
        (KEY_USAGE_DECRYPT, KEY_USAGE_ENCRYPT)
    } else {
        (KEY_USAGE_SIGN, KEY_USAGE_VERIFY)
    };
    row_key_pair_request(row, id, private_usage, public_usage)
}

fn covercrypt_vendor_attribute(
    attribute_name: &str,
    value: Value,
) -> Result<kmipkit_ttlv::Item, Box<dyn Error>> {
    const VENDOR_ATTRIBUTE: u32 = 0x0042_0008;
    const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
    const ATTRIBUTE_NAME: u32 = 0x0042_000A;
    const ATTRIBUTE_VALUE: u32 = 0x0042_000B;

    super::kmip_item(
        VENDOR_ATTRIBUTE,
        Value::structure(super::structure([
            super::kmip_item(
                VENDOR_IDENTIFICATION,
                Value::text_string("cosmian".to_owned()),
            )?,
            super::kmip_item(
                ATTRIBUTE_NAME,
                Value::text_string(attribute_name.to_owned()),
            )?,
            super::kmip_item(ATTRIBUTE_VALUE, value)?,
        ])?),
    )
}

fn covercrypt_access_policy() -> &'static str {
    "Security Level::Confidential && (Department::FIN || Department::HR)"
}

fn covercrypt_master_policy() -> Result<Vec<u8>, Box<dyn Error>> {
    let policy = serde_json::json!({
        "version": "V2",
        "last_attribute_value": 7,
        "dimensions": {
            "Security Level": {
                "order": ["Protected", "Confidential", "Top Secret"],
                "attributes": {
                    "Protected": {
                        "rotation_values": [5],
                        "encryption_hint": "Classic",
                        "write_status": "EncryptDecrypt"
                    },
                    "Confidential": {
                        "rotation_values": [6],
                        "encryption_hint": "Classic",
                        "write_status": "EncryptDecrypt"
                    },
                    "Top Secret": {
                        "rotation_values": [7],
                        "encryption_hint": "Hybridized",
                        "write_status": "EncryptDecrypt"
                    }
                }
            },
            "Department": {
                "order": null,
                "attributes": {
                    "RnD": {
                        "rotation_values": [1],
                        "encryption_hint": "Classic",
                        "write_status": "EncryptDecrypt"
                    },
                    "HR": {
                        "rotation_values": [2],
                        "encryption_hint": "Classic",
                        "write_status": "EncryptDecrypt"
                    },
                    "MKG": {
                        "rotation_values": [3],
                        "encryption_hint": "Classic",
                        "write_status": "EncryptDecrypt"
                    },
                    "FIN": {
                        "rotation_values": [4],
                        "encryption_hint": "Classic",
                        "write_status": "EncryptDecrypt"
                    }
                }
            }
        }
    });
    Ok(serde_json::to_vec(&policy)?)
}

fn vendor_attribute_bytes(
    structure: kmipkit_ttlv::StructureView<'_>,
    expected_name: &str,
) -> Option<Vec<u8>> {
    const VENDOR_ATTRIBUTE: u32 = 0x0042_0008;
    const ATTRIBUTE_NAME: u32 = 0x0042_000A;
    const ATTRIBUTE_VALUE: u32 = 0x0042_000B;

    for item in structure.children() {
        if item.tag().raw() == VENDOR_ATTRIBUTE {
            let matched = item.with_value(|value| {
                let kmipkit_ttlv::ValueView::Structure(fields) = value else {
                    return None;
                };
                let name = fields
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == ATTRIBUTE_NAME)
                    .and_then(|field| {
                        field.with_value(|value| match value {
                            kmipkit_ttlv::ValueView::TextString(name) => Some(name.to_owned()),
                            _ => None,
                        })
                    });
                if name.as_deref() != Some(expected_name) {
                    return None;
                }
                fields
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == ATTRIBUTE_VALUE)
                    .and_then(|field| {
                        field.with_value(|value| match value {
                            kmipkit_ttlv::ValueView::ByteString(bytes) => Some(bytes.to_vec()),
                            _ => None,
                        })
                    })
            });
            if matched.is_some() {
                return matched;
            }
        }
        let nested = item.with_value(|value| match value {
            kmipkit_ttlv::ValueView::Structure(child) => {
                vendor_attribute_bytes(child, expected_name)
            }
            _ => None,
        });
        if nested.is_some() {
            return nested;
        }
    }
    None
}

fn structure_contains_text(
    structure: kmipkit_ttlv::StructureView<'_>,
    expected_tag: u32,
    expected_text: &str,
) -> bool {
    structure.children().iter().any(|item| {
        if item.tag().raw() == expected_tag
            && item.with_value(|value| {
                matches!(value, kmipkit_ttlv::ValueView::TextString(actual) if actual == expected_text)
            })
        {
            return true;
        }
        item.with_value(|value| match value {
            kmipkit_ttlv::ValueView::Structure(child) => {
                structure_contains_text(child, expected_tag, expected_text)
            }
            _ => false,
        })
    })
}

fn run_create_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    let mut ownership = super::common::CaseOwnership::default();
    let operation_result = (|| {
        let parent_id = if row["algorithm"] == "CoverCrypt" {
            let master_id = format!("{id}-covercrypt-master");
            let master_request =
                row_key_pair_request(row, &master_id, KEY_USAGE_DECRYPT, KEY_USAGE_ENCRYPT)?;
            let (private_id, _) = super::common::execute_created_key_pair(
                client,
                id,
                master_request,
                &mut ownership,
            )?;
            Some(private_id)
        } else {
            None
        };
        let request = ClientRequest::Create(build_create_request(row, parent_id.as_deref())?);
        super::common::create_result_id(client, id, request, "Create", &mut ownership)?;
        Ok::<(), Box<dyn Error>>(())
    })()
    .map_err(|error| error.to_string());
    super::common::finish_operation_with_cleanup(operation_result, || {
        super::common::cleanup_owned_objects(client, id, &mut ownership)
    })
    .map_err(|message| -> Box<dyn Error> { message.into() })
}

fn run_create_key_pair_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    let mut ownership = super::common::CaseOwnership::default();
    let operation_result = (|| {
        let algorithm = required_case_text(row, "algorithm")?;
        let (private_usage, public_usage) =
            if is_kem_algorithm(algorithm) || algorithm == "CoverCrypt" {
                (KEY_USAGE_DECRYPT, KEY_USAGE_ENCRYPT)
            } else {
                (KEY_USAGE_SIGN, KEY_USAGE_VERIFY)
            };
        create_row_key_pair(client, row, id, &mut ownership, private_usage, public_usage)?;
        Ok::<(), Box<dyn Error>>(())
    })()
    .map_err(|error| error.to_string());
    super::common::finish_operation_with_cleanup(operation_result, || {
        super::common::cleanup_owned_objects(client, id, &mut ownership)
    })
    .map_err(|message| -> Box<dyn Error> { message.into() })
}

fn object_type_raw(row: &JsonValue) -> Result<u32, Box<dyn Error>> {
    let id = case_id(row)?;
    match row["object_type"].as_str() {
        Some("SymmetricKey") => Ok(2),
        Some("PublicKey") => Ok(3),
        Some("PrivateKey") => Ok(4),
        Some("SecretData") => Ok(7),
        Some(other) => Err(format!("case {id} has unsupported object type {other}").into()),
        None => Err(format!("case {id} has no object type").into()),
    }
}

fn run_certify_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    let mut ownership = super::common::CaseOwnership::default();
    let operation_result = (|| {
        let subject_input = required_parameter_text(row, "subject_input")?;
        let mut request = CertifyRequest::new().with_attributes(certify_attributes(row)?);
        match subject_input {
            "csr-pem" | "csr-pkcs10-der" => {
                let path = validation_fixture_path(row, "csr")?;
                let bytes = std::fs::read(path)
                    .map_err(|_| format!("case {id} required CSR fixture is unavailable"))?;
                let request_type = wire_enum_value(row, "certificate_request_type")?;
                let expected_type = if subject_input == "csr-pem" { 3 } else { 2 };
                if request_type != expected_type {
                    return Err(format!("case {id} CSR encoding and request type disagree").into());
                }
                request = request
                    .with_certificate_request_type(request_type)
                    .with_certificate_request_value(SecretBytes::new(bytes));
            }
            "existing-public" => {
                let (_, public_id) = create_row_key_pair(
                    client,
                    row,
                    id,
                    &mut ownership,
                    KEY_USAGE_SIGN,
                    KEY_USAGE_VERIFY,
                )?;
                request = request.with_unique_identifier(UniqueIdentifier::TextString(public_id));
            }
            "generated-subject" => {
                let (_, public_id) = create_row_key_pair(
                    client,
                    row,
                    id,
                    &mut ownership,
                    KEY_USAGE_SIGN,
                    KEY_USAGE_VERIFY,
                )?;
                super::common::run_activate(client, id, "PublicKey", &public_id)?;
                request = request.with_unique_identifier(UniqueIdentifier::TextString(public_id));
            }
            "renew-certificate" => {
                let certificate_id = register_row_certificate(client, row, id, &mut ownership)?;
                request =
                    request.with_unique_identifier(UniqueIdentifier::TextString(certificate_id));
            }
            other => return Err(format!("case {id} has unsupported Certify input {other}").into()),
        }

        super::common::execute_case(
            client,
            id,
            ClientRequest::Certify(request),
            "Certify",
            |response| {
                let identifier = response
                    .certify()
                    .and_then(|result| result.unique_identifier())
                    .ok_or_else(|| {
                        "successful Certify response has no Unique Identifier".to_owned()
                    })?;
                let identifier = super::text_identifier(identifier)
                    .map_err(|_| "Certify response identifier is not text".to_owned())?;
                ownership.track(identifier);
                Ok(())
            },
        )?;
        Ok::<(), Box<dyn Error>>(())
    })()
    .map_err(|error| error.to_string());
    super::common::finish_operation_with_cleanup(operation_result, || {
        super::common::cleanup_owned_objects(client, id, &mut ownership)
    })
    .map_err(|message| -> Box<dyn Error> { message.into() })
}

fn certify_attributes(row: &JsonValue) -> Result<Structure, Box<dyn Error>> {
    super::structure([super::kmip_item(
        CERTIFICATE_TYPE,
        Value::enumeration(wire_enum_value(row, "key_format_type")?),
    )?])
}

fn create_row_key_pair(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    id: &str,
    ownership: &mut super::common::CaseOwnership,
    private_usage: i32,
    public_usage: i32,
) -> Result<(String, String), Box<dyn Error>> {
    let request = row_key_pair_request(row, id, private_usage, public_usage)?;
    super::common::execute_created_key_pair(client, id, request, ownership)
}

fn row_key_pair_request(
    row: &JsonValue,
    id: &str,
    private_usage: i32,
    public_usage: i32,
) -> Result<CreateKeyPairRequest, Box<dyn Error>> {
    let algorithm = wire_enum_value(row, "cryptographic_algorithm")?;
    let configurable_kem = row["algorithm"].as_str() == Some("ConfigurableKEM");
    let configuration = if row["parameters"]["opaque_fixture_configuration"].is_object() {
        &row["parameters"]["opaque_fixture_configuration"]
    } else {
        &row["parameters"]
    };
    let mut common_items = vec![
        super::common::name_attribute(id)?,
        super::kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(algorithm))?,
    ];
    let bits = row["parameters"]["key_bits"]
        .as_i64()
        .or_else(|| row["parameters"]["subject_key_bits"].as_i64());
    if let Some(bits) = bits {
        common_items.push(super::kmip_item(
            CRYPTOGRAPHIC_LENGTH,
            Value::integer(i32::try_from(bits)?),
        )?);
    }
    let curve = row["wire_enum_values"]["recommended_curve"]["value"]
        .as_str()
        .map(parse_raw_enum)
        .transpose()?
        .or_else(|| {
            row["parameters"]["curve"]
                .as_str()
                .or_else(|| configuration["pre_quantum"].as_str())
                .and_then(curve_enum_value)
        });
    if let Some(curve) = curve {
        common_items.push(super::kmip_item(
            CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
            Value::structure(super::structure([super::kmip_item(
                RECOMMENDED_CURVE,
                Value::enumeration(curve),
            )?])?),
        )?);
    }
    if configurable_kem {
        if let Some(post_quantum) = configuration["post_quantum"].as_str() {
            let post_quantum_algorithm = match post_quantum {
                "MLKEM512" | "MLKEM_512" => 0x39,
                "MLKEM768" | "MLKEM_768" => 0x3A,
                "MLKEM1024" | "MLKEM_1024" => 0x3B,
                other => {
                    return Err(format!(
                        "case {id} has unsupported ConfigurableKEM post-quantum algorithm {other}"
                    )
                    .into());
                }
            };
            let cryptographic_parameters = super::structure([super::kmip_item(
                CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(post_quantum_algorithm),
            )?])?;
            common_items.push(super::kmip_item(
                CRYPTOGRAPHIC_PARAMETERS,
                Value::structure(cryptographic_parameters),
            )?);
        }
    }
    if row["algorithm"] == "CoverCrypt" {
        common_items.push(covercrypt_vendor_attribute(
            "cover_crypt_policy",
            Value::byte_string(covercrypt_master_policy()?),
        )?);
    }
    let request = CreateKeyPairRequest::new()
        .with_common_attributes(AttributeSet::try_new(common_items)?)
        .with_private_key_attributes(AttributeSet::try_new([super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(if configurable_kem {
                KEY_USAGE_DECRYPT
            } else {
                private_usage
            }),
        )?])?)
        .with_public_key_attributes(AttributeSet::try_new([super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(if configurable_kem {
                KEY_USAGE_ENCRYPT
            } else {
                public_usage
            }),
        )?])?);
    Ok(request)
}

fn register_row_certificate(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    id: &str,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let path = validation_fixture_path(row, "certificate")?;
    let bytes = std::fs::read(path)
        .map_err(|_| format!("case {id} required certificate fixture is unavailable"))?;
    let identifier = format!("{id}-certificate");
    ownership.track(identifier.clone());
    let attributes = AttributeSet::try_new([
        super::common::name_attribute(id)?,
        super::kmip_item(
            CERTIFICATE_LENGTH,
            Value::integer(i32::try_from(bytes.len())?),
        )?,
    ])?;
    let object = super::kmip_item(
        CERTIFICATE,
        Value::structure(super::structure([
            super::kmip_item(CERTIFICATE_TYPE, Value::enumeration(1))?,
            super::kmip_item(CERTIFICATE_VALUE, Value::byte_string(bytes))?,
        ])?),
    )?;
    super::common::execute_case(
        client,
        id,
        ClientRequest::register(RegisterRequest::new(
            ObjectType::from_raw(1),
            attributes,
            object,
        )),
        "Register Certify fixture",
        |response| {
            let registered = response
                .register()
                .and_then(|result| result.unique_identifier())
                .ok_or_else(|| {
                    "successful Register response has no Unique Identifier".to_owned()
                })?;
            let registered = super::text_identifier(registered)
                .map_err(|_| "Register response identifier is not text".to_owned())?;
            if registered != identifier {
                return Err("Register returned a different case-owned identifier".to_owned());
            }
            Ok(())
        },
    )?;
    Ok(identifier)
}

#[cfg(test)]
mod lifecycle_fixture_regressions {
    use super::*;

    #[test]
    fn lifecycle_key_rows_without_an_algorithm_can_build_a_typed_pair() {
        for case_id in [
            "cosmian-528-activate-privatekey",
            "cosmian-528-activate-publickey",
        ] {
            let row = super::super::common::manifest_row(case_id)
                .expect("lifecycle key case exists in the manifest");
            let request = build_create_key_pair_request(&row, case_id)
                .expect("the lifecycle fixture supplies a deterministic RSA key pair");
            let payload = request
                .into_ttlv_payload()
                .expect("the lifecycle key pair is valid typed TTLV");
            let view = payload.view();
            assert!(
                structure_contains_integer(&view, CRYPTOGRAPHIC_ALGORITHM, 4),
                "{case_id} must use the explicit RSA fixture algorithm"
            );
            assert!(
                structure_contains_integer(&view, CRYPTOGRAPHIC_LENGTH, 2048),
                "{case_id} must use the explicit 2,048-bit RSA fixture"
            );
        }
    }

    fn structure_contains_integer(
        structure: &kmipkit_ttlv::StructureView<'_>,
        tag: u32,
        expected: i64,
    ) -> bool {
        structure.children().iter().any(|item| {
            if item.tag().raw() == tag
                && item.with_value(|value| {
                    matches!(value, kmipkit_ttlv::ValueView::Enumeration(actual) if i64::from(*actual) == expected)
                        || matches!(value, kmipkit_ttlv::ValueView::Integer(actual) if i64::from(*actual) == expected)
                })
            {
                return true;
            }
            item.with_value(|value| match value {
                kmipkit_ttlv::ValueView::Structure(child) => {
                    structure_contains_integer(&child, tag, expected)
                }
                _ => false,
            })
        })
    }
}

fn register_certificate_private_key(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    id: &str,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let certificate_path = validation_fixture_path(row, "certificate")?;
    let file_name = certificate_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("case {id} has an invalid local certificate fixture path"))?;
    let stem = file_name
        .strip_suffix("-certificate.der")
        .ok_or_else(|| format!("case {id} has an invalid local certificate fixture name"))?;
    let directory = certificate_path
        .parent()
        .ok_or_else(|| format!("case {id} has no local fixture directory"))?;
    let public_bytes = std::fs::read(directory.join(format!("{stem}-public-spki.der")))
        .map_err(|_| format!("case {id} certificate oracle public-key fixture is unavailable"))?;
    let private_bytes = std::fs::read(directory.join(format!("{stem}-private-pkcs8.der")))
        .map_err(|_| format!("case {id} certificate oracle private-key fixture is unavailable"))?;
    let algorithm = wire_enum_value(row, "cryptographic_algorithm")?;
    let curve_name = row["parameters"]["curve"].as_str();
    let curve = row["wire_enum_values"]["recommended_curve"]["value"]
        .as_str()
        .map(parse_raw_enum)
        .transpose()?
        .or_else(|| curve_name.and_then(curve_enum_value));
    let bits = row["parameters"]["subject_key_bits"]
        .as_i64()
        .or_else(|| row["parameters"]["key_bits"].as_i64())
        .or_else(|| curve_name.and_then(curve_bit_length))
        .ok_or_else(|| format!("case {id} certificate oracle key omits its length"))?;

    let public_attributes =
        cipher_fixture_key_attributes(id, algorithm, bits, KEY_USAGE_ENCRYPT, curve, None)?;
    let public_object = signature_key_object(public_bytes, 5, algorithm, false)?;
    let public_identifier = register_independent_key(
        client,
        id,
        RegisterRequest::new(ObjectType::from_raw(3), public_attributes, public_object),
        ownership,
    )?;
    super::common::run_activate(
        client,
        id,
        "certificate oracle PublicKey",
        &public_identifier,
    )?;

    let link = super::kmip_item(
        LINK,
        Value::structure(super::structure([
            super::kmip_item(LINK_TYPE, Value::enumeration(0x0000_0102))?,
            super::kmip_item(
                LINKED_OBJECT_IDENTIFIER,
                Value::text_string(public_identifier),
            )?,
        ])?),
    )?;
    let private_attributes =
        cipher_fixture_key_attributes(id, algorithm, bits, KEY_USAGE_DECRYPT, curve, Some(link))?;
    let private_object = signature_key_object(private_bytes, 4, algorithm, true)?;
    let private_identifier = register_independent_key(
        client,
        id,
        RegisterRequest::new(ObjectType::from_raw(4), private_attributes, private_object),
        ownership,
    )?;
    super::common::run_activate(
        client,
        id,
        "certificate oracle PrivateKey",
        &private_identifier,
    )?;
    Ok(private_identifier)
}

fn cipher_fixture_key_attributes(
    id: &str,
    algorithm: u32,
    bits: i64,
    usage: i32,
    curve: Option<u32>,
    link: Option<kmipkit_ttlv::Item>,
) -> Result<AttributeSet, Box<dyn Error>> {
    let mut items = vec![
        super::common::name_attribute(id)?,
        super::kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(algorithm))?,
        super::kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(i32::try_from(bits)?))?,
        super::kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(usage))?,
    ];
    if let Some(curve) = curve {
        items.push(super::kmip_item(
            CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
            Value::structure(super::structure([super::kmip_item(
                RECOMMENDED_CURVE,
                Value::enumeration(curve),
            )?])?),
        )?);
    }
    if let Some(link) = link {
        items.push(link);
    }
    Ok(AttributeSet::try_new(items)?)
}

fn validation_fixture_path(
    row: &JsonValue,
    material: &str,
) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let id = case_id(row)?;
    let stem = validation_fixture_stem(row)?;
    let parameters = &row["parameters"];
    let subject_input = parameters["subject_input"].as_str().unwrap_or_default();
    let file_name = match (material, subject_input) {
        ("csr", "csr-pem") => format!("{stem}-csr-pem.pem"),
        ("csr", "csr-pkcs10-der") => format!("{stem}-csr-pkcs10.der"),
        ("certificate", _) => format!("{stem}-certificate.der"),
        _ => return Err(format!("case {id} requests an unsupported certificate fixture").into()),
    };
    Ok(super::certificate_directory()
        .join("validation")
        .join(file_name))
}

fn validation_fixture_stem(row: &JsonValue) -> Result<String, Box<dyn Error>> {
    let id = case_id(row)?;
    let algorithm = row["algorithm"].as_str().unwrap_or_default();
    let parameters = &row["parameters"];
    let stem = match algorithm {
        "RSA" => format!(
            "rsa-{}",
            parameters["subject_key_bits"].as_u64().unwrap_or(2048)
        ),
        "EC" | "ECDSA" | "ECDH" => match parameters["curve"].as_str().unwrap_or("P256") {
            "CURVE25519" => "x25519".to_owned(),
            "CURVEED25519" => "ed25519".to_owned(),
            "CURVE448" => "x448".to_owned(),
            "CURVEED448" => "ed448".to_owned(),
            curve => format!("ec-{curve}"),
        },
        "Ed25519" => "ed25519".to_owned(),
        "Ed448" => "ed448".to_owned(),
        "MLDSA_44" => "ML-DSA-44".to_owned(),
        "MLDSA_65" => "ML-DSA-65".to_owned(),
        "MLDSA_87" => "ML-DSA-87".to_owned(),
        other if other.starts_with("SLHDSA_") => {
            other.replace("SLHDSA_", "SLH-DSA-").replace("_", "-")
        }
        other => return Err(format!("case {id} has no local fixture mapping for {other}").into()),
    };
    Ok(stem)
}

fn curve_enum_value(name: &str) -> Option<u32> {
    match name {
        "P192" => Some(1),
        "P224" => Some(4),
        "P256" => Some(7),
        "P384" => Some(10),
        "P521" => Some(13),
        "SECP224K1" => Some(0x18),
        "SECP256K1" => Some(0x19),
        "CURVE25519" | "CURVEED25519" => Some(0x45),
        "CURVE448" | "CURVEED448" => Some(0x46),
        _ => None,
    }
}

fn curve_bit_length(name: &str) -> Option<i64> {
    match name {
        "P192" => Some(192),
        "P224" | "SECP224K1" => Some(224),
        "P256" | "SECP256K1" | "CURVE25519" | "CURVEED25519" => Some(256),
        "P384" => Some(384),
        "P521" => Some(521),
        "CURVE448" | "CURVEED448" => Some(448),
        _ => None,
    }
}

fn parse_raw_enum(value: &str) -> Result<u32, Box<dyn Error>> {
    u32::from_str_radix(value.trim_start_matches("0x"), 16)
        .map_err(|_| "manifest enum value is not valid hexadecimal".into())
}

fn run_cipher_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    if row["algorithm"].as_str() == Some("CoverCrypt") {
        return run_covercrypt_cipher_case(client, row);
    }
    if super::cryptographic_cipher::run_symmetric_case(row, client)?.is_some() {
        return Ok(());
    }

    let id = case_id(row)?;
    let operation = required_case_text(row, "operation")?;
    let target_type = required_case_text(row, "object_type")?;
    if !matches!(operation, "Encrypt" | "Decrypt") {
        return Err(format!("case {id} has unsupported cipher operation {operation}").into());
    }
    let mut ownership = super::common::CaseOwnership::default();
    let operation_result = (|| {
        let plaintext = b"KMIPKit Cosmian typed cipher matrix";
        let ciphertext_fixture_row = asymmetric_ciphertext_fixture_row(row)?;
        let setup_row = if operation == "Decrypt" {
            &ciphertext_fixture_row
        } else {
            row
        };
        let provisioning_algorithm = match setup_row["algorithm"].as_str() {
            // These finding probes deliberately apply a symmetric algorithm
            // to an asymmetric object. Provision a valid RSA object so the
            // requested target operation, rather than setup, is what fails.
            Some("AES") if target_type != "SymmetricKey" => "RSA",
            Some(value) => value,
            None => return Err(format!("case {id} has no algorithm").into()),
        };
        let (public_id, private_id) = if target_type == "Certificate" {
            if operation != "Encrypt" {
                return Err(format!("case {id} Decrypt cannot target a Certificate").into());
            }
            let mut fixture_row = setup_row.clone();
            fixture_row["algorithm"] = JsonValue::String(provisioning_algorithm.to_owned());
            if provisioning_algorithm != setup_row["algorithm"].as_str().unwrap_or_default()
                && provisioning_algorithm == "RSA"
            {
                fixture_row["wire_enum_values"]["cryptographic_algorithm"] =
                    serde_json::json!({"name":"RSA","value":"0x00000004","classification":"standard"});
            }
            if fixture_row["parameters"]["subject_key_bits"].is_null() {
                let bits = fixture_row["parameters"]["key_bits"]
                    .as_u64()
                    .unwrap_or(2048);
                fixture_row["parameters"]["subject_key_bits"] = JsonValue::from(bits);
            }
            let certificate_id = register_row_certificate(
                client,
                &fixture_row,
                id,
                &mut ownership,
            )?;
            super::common::run_activate(client, id, "Certificate", &certificate_id)?;
            let private_id = register_certificate_private_key(
                client,
                &fixture_row,
                id,
                &mut ownership,
            )?;
            super::common::run_activate(client, id, "Certificate oracle PrivateKey", &private_id)?;
            (certificate_id, private_id)
        } else {
            let mut key_row = setup_row.clone();
            key_row["algorithm"] = JsonValue::String(provisioning_algorithm.to_owned());
            if key_row["wire_enum_values"]["cryptographic_algorithm"].is_null() {
                return Err(format!("case {id} has no key-pair algorithm enum").into());
            }
            if provisioning_algorithm != setup_row["algorithm"].as_str().unwrap_or_default() {
                key_row["wire_enum_values"]["cryptographic_algorithm"] =
                    serde_json::json!({"name":"RSA","value":"0x00000004","classification":"standard"});
                key_row["parameters"]["key_bits"] = JsonValue::from(2048);
            }
            let (private_id, public_id) = create_row_key_pair(
                client,
                &key_row,
                id,
                &mut ownership,
                KEY_USAGE_DECRYPT,
                KEY_USAGE_ENCRYPT,
            )?;
            super::common::run_activate(client, id, "PrivateKey", &private_id)?;
            super::common::run_activate(client, id, "PublicKey", &public_id)?;
            (public_id, private_id)
        };

        let encryption_parameters_row = if operation == "Decrypt" {
            &ciphertext_fixture_row
        } else {
            row
        };
        let encrypted = execute_asymmetric_encrypt(
            client,
            encryption_parameters_row,
            &public_id,
            plaintext,
        )?;
        if operation == "Encrypt" {
            let decrypted = execute_asymmetric_decrypt(client, row, &private_id, &encrypted)
                .map_err(|_| {
                    format!("case {id} Encrypt: target response received; round-trip oracle failed")
                })?;
            let expected = if is_kem_algorithm(required_case_text(row, "algorithm")?) {
                &encrypted.data
            } else {
                // The ciphertext's Data is opaque for classical encryption;
                // its oracle must recover the private fixed input.
                if !decrypted.with_bytes(|value| value == plaintext) {
                    return Err(format!(
                        "case {id} Encrypt: target response received; round-trip did not reproduce the known plaintext"
                    )
                    .into());
                }
                return Ok::<(), Box<dyn Error>>(());
            };
            if !decrypted.with_bytes(|value| expected.with_bytes(|secret| value == secret)) {
                return Err(format!(
                    "case {id} Encrypt: target response received; KEM shared secret did not round-trip"
                )
                .into());
            }
            return Ok::<(), Box<dyn Error>>(());
        }
        let decrypted = execute_asymmetric_decrypt(client, row, &private_id, &encrypted)?;
        let expected_matches = if is_kem_algorithm(required_case_text(row, "algorithm")?) {
            decrypted.with_bytes(|value| encrypted.data.with_bytes(|secret| value == secret))
        } else {
            decrypted.with_bytes(|value| value == plaintext)
        };
        if !expected_matches {
            return Err(format!(
                "case {id} Decrypt: target response received; decrypted material did not match the fixture"
            )
            .into());
        }
        Ok(())
    })()
    .map_err(|error| error.to_string());

    super::common::finish_operation_with_cleanup(operation_result, || {
        super::common::cleanup_owned_objects(client, id, &mut ownership)
    })
    .map_err(|message| -> Box<dyn Error> { message.into() })
}

fn run_covercrypt_cipher_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    let operation = required_case_text(row, "operation")?;
    if !matches!(operation, "Encrypt" | "Decrypt") {
        return Err(format!("case {id} has unsupported CoverCrypt operation {operation}").into());
    }
    let rows = target_rows()?;
    let master_row = rows
        .iter()
        .find(|candidate| {
            candidate["operation"] == "Create Key Pair"
                && candidate["algorithm"] == "CoverCrypt"
                && candidate["expected_outcome"] == "typed_success"
        })
        .ok_or("CoverCrypt master key Create Key Pair row is missing")?;
    let user_key_row = rows
        .iter()
        .find(|candidate| {
            candidate["operation"] == "Create"
                && candidate["algorithm"] == "CoverCrypt"
                && candidate["expected_outcome"] == "typed_success"
        })
        .ok_or("CoverCrypt user key Create row is missing")?;

    let mut ownership = super::common::CaseOwnership::default();
    let operation_result = (|| {
        let (master_private_id, master_public_id) = create_row_key_pair(
            client,
            master_row,
            &format!("{id}-covercrypt-master"),
            &mut ownership,
            KEY_USAGE_DECRYPT,
            KEY_USAGE_ENCRYPT,
        )?;
        super::common::run_activate(client, id, "CoverCrypt master PrivateKey", &master_private_id)?;
        super::common::run_activate(client, id, "CoverCrypt master PublicKey", &master_public_id)?;

        let user_key_id = super::common::create_result_id(
            client,
            id,
            ClientRequest::Create(build_create_request(
                user_key_row,
                Some(&master_private_id),
            )?),
            "CoverCrypt user-key Create setup",
            &mut ownership,
        )?;
        super::common::run_activate(client, id, "CoverCrypt user PrivateKey", &user_key_id)?;

        const PLAINTEXT: &[u8] = b"KMIPKit CoverCrypt integration payload";
        if operation == "Encrypt" {
            let ciphertext = execute_covercrypt_encrypt(client, row, &master_public_id, "Encrypt")?;
            let decrypted = execute_covercrypt_decrypt(
                client,
                row,
                &user_key_id,
                ciphertext,
                "Decrypt oracle",
            )?;
            if !decrypted.with_bytes(|bytes| bytes == PLAINTEXT) {
                return Err(format!(
                    "case {id} Encrypt: target response received; CoverCrypt round-trip did not reproduce the known plaintext"
                )
                .into());
            }
        } else {
            let ciphertext = execute_covercrypt_encrypt(
                client,
                row,
                &master_public_id,
                "Encrypt oracle setup",
            )?;
            let decrypted = execute_covercrypt_decrypt(
                client,
                row,
                &user_key_id,
                ciphertext,
                "Decrypt",
            )?;
            if !decrypted.with_bytes(|bytes| bytes == PLAINTEXT) {
                return Err(format!(
                    "case {id} Decrypt: target response received; CoverCrypt plaintext did not match the setup fixture"
                )
                .into());
            }
        }
        Ok::<(), Box<dyn Error>>(())
    })()
    .map_err(|error| error.to_string());

    super::common::finish_operation_with_cleanup(operation_result, || {
        super::common::cleanup_owned_objects(client, id, &mut ownership)
    })
    .map_err(|message| -> Box<dyn Error> { message.into() })
}

fn execute_covercrypt_encrypt(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    public_identifier: &str,
    operation_label: &str,
) -> Result<SecretBytes, Box<dyn Error>> {
    let id = case_id(row)?;
    let plaintext = super::cryptographic_covercrypt::encode_data_to_encrypt(
        covercrypt_access_policy(),
        &[],
        b"KMIPKit CoverCrypt integration payload",
    )?;
    let parameters = asymmetric_cipher_parameters(row)?;
    super::common::execute_case(
        client,
        id,
        ClientRequest::encrypt(
            EncryptRequest::new(
                Some(UniqueIdentifier::TextString(public_identifier.to_owned())),
                Some(OperationData::ByteString(plaintext)),
            )
            .with_cryptographic_parameters(parameters),
        ),
        operation_label,
        |response| {
            let result = response.encrypt().ok_or_else(|| {
                "successful CoverCrypt Encrypt response has no typed result".to_owned()
            })?;
            let response_identifier = result
                .unique_identifier()
                .and_then(|identifier| super::text_identifier(identifier).ok());
            if response_identifier.as_deref() != Some(public_identifier) {
                return Err("CoverCrypt Encrypt returned a different Unique Identifier".to_owned());
            }
            let data = result
                .data()
                .ok_or_else(|| "successful CoverCrypt Encrypt response has no Data".to_owned())?
                .with_bytes(|bytes| SecretBytes::new(bytes.to_vec()));
            if data.with_bytes(<[u8]>::is_empty) {
                return Err("successful CoverCrypt Encrypt response has empty Data".to_owned());
            }
            Ok(data)
        },
    )
}

fn execute_covercrypt_decrypt(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    private_identifier: &str,
    ciphertext: SecretBytes,
    operation_label: &str,
) -> Result<SecretBytes, Box<dyn Error>> {
    let id = case_id(row)?;
    let parameters = asymmetric_cipher_parameters(row)?;
    super::common::execute_case(
        client,
        id,
        ClientRequest::decrypt(
            DecryptRequest::new(
                Some(UniqueIdentifier::TextString(private_identifier.to_owned())),
                Some(OperationData::ByteString(ciphertext)),
            )
            .with_cryptographic_parameters(parameters),
        ),
        operation_label,
        |response| {
            let result = response.decrypt().ok_or_else(|| {
                "successful CoverCrypt Decrypt response has no typed result".to_owned()
            })?;
            let response_identifier = result
                .unique_identifier()
                .and_then(|identifier| super::text_identifier(identifier).ok());
            if response_identifier.as_deref() != Some(private_identifier) {
                return Err("CoverCrypt Decrypt returned a different Unique Identifier".to_owned());
            }
            result
                .data()
                .map(|data| data.with_bytes(|bytes| SecretBytes::new(bytes.to_vec())))
                .ok_or_else(|| "successful CoverCrypt Decrypt response has no Data".to_owned())
        },
    )
}

fn asymmetric_ciphertext_fixture_row(row: &JsonValue) -> Result<JsonValue, Box<dyn Error>> {
    let operation = required_case_text(row, "operation")?;
    if operation != "Decrypt" {
        return Ok(row.clone());
    }
    let target_algorithm = required_case_text(row, "algorithm")?;
    let key_algorithm = row["parameters"]["key_algorithm"]
        .as_str()
        .unwrap_or(target_algorithm);
    if target_algorithm == key_algorithm {
        return Ok(row.clone());
    }
    let id = case_id(row)?;
    Err(format!(
        "case {id} Decrypt target algorithm {target_algorithm} does not match the {key_algorithm} key; the manifest has no exact hybrid ciphertext fixture"
    )
    .into())
}

struct AsymmetricEncrypted {
    data: SecretBytes,
    encapsulation: Option<SecretBytes>,
}

fn is_kem_algorithm(algorithm: &str) -> bool {
    matches!(
        algorithm,
        "ConfigurableKEM"
            | "MLKEM_512"
            | "MLKEM_768"
            | "MLKEM_1024"
            | "X25519MLKEM768"
            | "X448MLKEM1024"
    )
}

fn execute_asymmetric_encrypt(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    identifier: &str,
    plaintext: &[u8],
) -> Result<AsymmetricEncrypted, Box<dyn Error>> {
    let id = case_id(row)?;
    let algorithm = required_case_text(row, "algorithm")?;
    let parameters = asymmetric_cipher_parameters(row)?;
    super::common::execute_case(
        client,
        id,
        ClientRequest::encrypt(
            EncryptRequest::new(
                Some(UniqueIdentifier::TextString(identifier.to_owned())),
                Some(OperationData::ByteString(SecretBytes::new(
                    plaintext.to_vec(),
                ))),
            )
            .with_cryptographic_parameters(parameters),
        ),
        "Encrypt",
        |response| {
            let result = response
                .encrypt()
                .ok_or_else(|| "successful Encrypt response has no typed result".to_owned())?;
            let data = result
                .data()
                .ok_or_else(|| "successful Encrypt response has no Data".to_owned())?
                .with_bytes(|bytes| SecretBytes::new(bytes.to_vec()));
            if data.with_bytes(<[u8]>::is_empty) {
                return Err("successful Encrypt response has empty Data".to_owned());
            }
            if algorithm == "ConfigurableKEM" {
                let (shared_secret, encapsulation) =
                    super::cryptographic_kem::decode_configurable_kem_encrypt_data(data)
                        .map_err(str::to_owned)?;
                return Ok(AsymmetricEncrypted {
                    data: shared_secret,
                    encapsulation: Some(encapsulation),
                });
            }
            let encapsulation = if is_kem_algorithm(algorithm) {
                Some(
                    result
                        .iv_counter_nonce()
                        .ok_or_else(|| {
                            "successful KEM Encrypt response has no IVCounterNonce encapsulation"
                                .to_owned()
                        })?
                        .with_bytes(|bytes| SecretBytes::new(bytes.to_vec())),
                )
            } else {
                None
            };
            Ok(AsymmetricEncrypted {
                data,
                encapsulation,
            })
        },
    )
}

fn execute_asymmetric_decrypt(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    identifier: &str,
    encrypted: &AsymmetricEncrypted,
) -> Result<SecretBytes, Box<dyn Error>> {
    let id = case_id(row)?;
    let algorithm = required_case_text(row, "algorithm")?;
    let decryption_data = if is_kem_algorithm(algorithm) {
        encrypted
            .encapsulation
            .as_ref()
            .ok_or_else(|| format!("case {id} KEM encryption has no encapsulation"))?
    } else {
        &encrypted.data
    };
    let parameters = asymmetric_cipher_parameters(row)?;
    super::common::execute_case(
        client,
        id,
        ClientRequest::decrypt(
            DecryptRequest::new(
                Some(UniqueIdentifier::TextString(identifier.to_owned())),
                Some(OperationData::ByteString(SecretBytes::new(
                    decryption_data.with_bytes(<[u8]>::to_vec),
                ))),
            )
            .with_cryptographic_parameters(parameters),
        ),
        "Decrypt",
        |response| {
            response
                .decrypt()
                .and_then(|result| result.data())
                .map(|data| SecretBytes::new(data.with_bytes(<[u8]>::to_vec)))
                .ok_or_else(|| "successful Decrypt response has no Data".to_owned())
        },
    )
}

fn asymmetric_cipher_parameters(row: &JsonValue) -> Result<Structure, Box<dyn Error>> {
    let id = case_id(row)?;
    let algorithm = wire_enum_value(row, "cryptographic_algorithm")?;
    let algorithm_name = required_case_text(row, "algorithm")?;
    let parameters = &row["parameters"];
    let mut items = Vec::new();
    if algorithm_name == "RSA" {
        let padding = match parameters["padding"].as_str() {
            Some("OAEP") => 2,
            Some("PKCS1v15") => 8,
            Some("None") => 1,
            Some(other) => {
                return Err(format!("case {id} has unsupported RSA padding {other}").into());
            }
            None => return Err(format!("case {id} RSA parameters omit padding axis").into()),
        };
        items.push(super::kmip_item(
            PADDING_METHOD,
            Value::enumeration(padding),
        )?);
        if padding == 1 {
            // Cosmian's hybrid RSA/AES-KWP mode consumes the manifest hash
            // even though KMIP Padding Method is explicitly None.
            let hash = parameters["hash"]
                .as_str()
                .ok_or_else(|| format!("case {id} hybrid RSA parameters omit Hash"))?;
            items.push(super::kmip_item(
                HASHING_ALGORITHM,
                Value::enumeration(hashing_algorithm_value(hash)?),
            )?);
        } else if padding == 2 {
            let hash = parameters["hash"]
                .as_str()
                .ok_or_else(|| format!("case {id} OAEP parameters omit Hash"))?;
            items.push(super::kmip_item(
                HASHING_ALGORITHM,
                Value::enumeration(hashing_algorithm_value(hash)?),
            )?);
        }
    }
    // OASIS KMIP v2.1 §4.16, Table 59 places Cryptographic Algorithm after
    // Padding Method and Hashing Algorithm, before the OAEP-specific fields.
    items.push(super::kmip_item(
        CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(algorithm),
    )?);
    if algorithm_name == "RSA" && parameters["padding"].as_str() == Some("OAEP") {
        if parameters["mgf"].as_str() != Some("MGF1") {
            return Err(format!("case {id} has unsupported OAEP Mask Generator").into());
        }
        items.push(super::kmip_item(MASK_GENERATOR, Value::enumeration(1))?);
        let mgf_hash = parameters["mgf_hash"]
            .as_str()
            .ok_or_else(|| format!("case {id} OAEP parameters omit MGF Hash"))?;
        items.push(super::kmip_item(
            MASK_GENERATOR_HASHING_ALGORITHM,
            Value::enumeration(hashing_algorithm_value(mgf_hash)?),
        )?);
        match parameters["p_source"].as_str() {
            Some("empty") => {
                items.push(super::kmip_item(P_SOURCE, Value::byte_string(Vec::new()))?)
            }
            Some("nonempty") => items.push(super::kmip_item(
                P_SOURCE,
                Value::byte_string(OAEP_TEST_LABEL.to_vec()),
            )?),
            Some(_) => return Err(format!("case {id} uses an unsupported OAEP PSource").into()),
            None => return Err(format!("case {id} OAEP parameters omit PSource").into()),
        }
    }
    Ok(super::structure(items)?)
}

fn signature_parameters(row: &JsonValue) -> Result<Structure, Box<dyn Error>> {
    let id = case_id(row)?;
    let algorithm = required_case_text(row, "algorithm")?;
    let mut items = Vec::new();
    if row["expected_outcome"] == "source_documentation_discrepancy"
        && row["wire_enum_values"]["cryptographic_algorithm"]["classification"]
            == "source_only_numeric_probe"
    {
        items.push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(wire_enum_value(row, "cryptographic_algorithm")?),
        )?);
        return Ok(super::structure(items)?);
    }
    match algorithm {
        "RSA" => {
            let hash = required_parameter_text(row, "hash")?;
            let scheme = required_parameter_text(row, "signature_scheme")?;
            match scheme {
                "RSASSA-PSS" => {
                    items.push(super::kmip_item(PADDING_METHOD, Value::enumeration(10))?)
                }
                "PKCS1v15" => items.push(super::kmip_item(PADDING_METHOD, Value::enumeration(8))?),
                other => {
                    return Err(
                        format!("case {id} has unsupported RSA signature scheme {other}").into(),
                    );
                }
            }
            items.push(super::kmip_item(
                HASHING_ALGORITHM,
                Value::enumeration(hashing_algorithm_value(hash)?),
            )?);
            items.push(super::kmip_item(
                DIGITAL_SIGNATURE_ALGORITHM,
                Value::enumeration(digital_signature_algorithm_value(scheme, Some(hash))?),
            )?);
            items.push(super::kmip_item(
                CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(wire_enum_value(row, "cryptographic_algorithm")?),
            )?);
            if scheme == "RSASSA-PSS" {
                let salt = &row["parameters"]["salt_length_bytes"];
                if !salt.is_null() {
                    let salt_length = match salt.as_str() {
                        Some("digest") => i32::try_from(digest_length(hash)?)?,
                        Some(other) => {
                            return Err(
                                format!("case {id} has invalid PSS Salt Length {other}").into()
                            );
                        }
                        None => i32::try_from(
                            salt.as_u64()
                                .ok_or_else(|| format!("case {id} has invalid PSS Salt Length"))?,
                        )?,
                    };
                    items.push(super::kmip_item(SALT_LENGTH, Value::integer(salt_length))?);
                }
                if let Some(mgf_hash) = row["parameters"]["mgf_hash"].as_str() {
                    if mgf_hash.is_empty() {
                        return Err(format!("case {id} has an empty PSS MGF Hash").into());
                    }
                    items.push(super::kmip_item(MASK_GENERATOR, Value::enumeration(1))?);
                    items.push(super::kmip_item(
                        MASK_GENERATOR_HASHING_ALGORITHM,
                        Value::enumeration(hashing_algorithm_value(mgf_hash)?),
                    )?);
                } else if !row["parameters"]["mgf_hash"].is_null() {
                    return Err(format!("case {id} has invalid PSS MGF Hash").into());
                }
            }
        }
        "ECDSA" => {
            let hash = required_parameter_text(row, "hash")?;
            items.push(super::kmip_item(
                HASHING_ALGORITHM,
                Value::enumeration(hashing_algorithm_value(hash)?),
            )?);
            items.push(super::kmip_item(
                DIGITAL_SIGNATURE_ALGORITHM,
                Value::enumeration(digital_signature_algorithm_name(required_parameter_text(
                    row,
                    "digital_signature_algorithm",
                )?)?),
            )?);
            items.push(super::kmip_item(
                CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(wire_enum_value(row, "cryptographic_algorithm")?),
            )?);
        }
        "Ed25519" | "Ed448" => items.push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(wire_enum_value(row, "cryptographic_algorithm")?),
        )?),
        other => {
            return Err(format!("case {id} has unsupported signature algorithm {other}").into());
        }
    }
    Ok(super::structure(items)?)
}

fn digital_signature_algorithm_value(
    scheme: &str,
    hash: Option<&str>,
) -> Result<u32, Box<dyn Error>> {
    let value = match (scheme, hash) {
        ("RSASSA-PSS", _) => 8,
        ("PKCS1v15", Some("SHA-1")) => 3,
        ("PKCS1v15", Some("SHA-224")) => 4,
        ("PKCS1v15", Some("SHA-256")) => 5,
        ("PKCS1v15", Some("SHA-384")) => 6,
        ("PKCS1v15", Some("SHA-512")) => 7,
        ("PKCS1v15", Some("SHA3-256")) => 17,
        ("PKCS1v15", Some("SHA3-384")) => 18,
        ("PKCS1v15", Some("SHA3-512")) => 19,
        (other, _) => return Err(format!("unsupported Digital Signature Algorithm {other}").into()),
    };
    Ok(value)
}

fn digital_signature_algorithm_name(name: &str) -> Result<u32, Box<dyn Error>> {
    let value = match name {
        "ECDSAWithSHA1" => 12,
        "ECDSAWithSHA224" => 13,
        "ECDSAWithSHA256" => 14,
        "ECDSAWithSHA384" => 15,
        "ECDSAWithSHA512" => 16,
        other => return Err(format!("unsupported Digital Signature Algorithm {other}").into()),
    };
    Ok(value)
}

fn digest_length(hash: &str) -> Result<usize, Box<dyn Error>> {
    match hash {
        "SHA-1" => Ok(20),
        "SHA-224" | "SHA3-224" => Ok(28),
        "SHA-256" | "SHA3-256" => Ok(32),
        "SHA-384" | "SHA3-384" => Ok(48),
        "SHA-512" | "SHA3-512" => Ok(64),
        other => Err(format!("unknown digest length for {other}").into()),
    }
}

enum SignatureMessage {
    Raw(Vec<u8>),
    Digested(Vec<u8>),
}

fn signature_message(row: &JsonValue) -> Result<SignatureMessage, Box<dyn Error>> {
    match required_parameter_text(row, "data_mode")? {
        "raw" => Ok(SignatureMessage::Raw(b"abc".to_vec())),
        "prehashed" => {
            if let Some(hash) = row["parameters"]["hash"].as_str() {
                let digest = match hash {
                    "SHA-1" => "a9993e364706816aba3e25717850c26c9cd0d89d",
                    other => known_answer_digest(other, false)?,
                };
                Ok(SignatureMessage::Digested(decode_hex(digest)?))
            } else if row["wire_enum_values"]["cryptographic_algorithm"]["classification"]
                == "source_only_numeric_probe"
            {
                // The source-only PQ signature probe records a DigestedData
                // field shape, without asserting a KMIP 2.1 prehash profile.
                Ok(SignatureMessage::Digested(vec![0xA5; 32]))
            } else {
                Err(format!(
                    "case {} prehashed signature omits its hash algorithm",
                    case_id(row)?
                )
                .into())
            }
        }
        mode => Err(format!(
            "case {} has unsupported signature data mode {mode}",
            case_id(row)?
        )
        .into()),
    }
}

fn run_signature_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    let mut ownership = super::common::CaseOwnership::default();
    let operation_result = (|| {
        if required_case_text(row, "operation")? == "Signature Verify"
            && row["parameters"]["setup_signature_source"].as_str()
                == Some("independent signer using requested digest; never reuse affected server raw Sign as proof")
        {
            let message = signature_message(row)?;
            let signature = independent_signature(row, &message)?;
            let verification_id = register_independent_verification_key(
                client,
                row,
                id,
                &mut ownership,
            )?;
            return execute_signature_verify(
                client,
                row,
                &verification_id,
                &message,
                signature,
            );
        }
        let (private_id, public_id) = create_row_key_pair(
            client,
            row,
            id,
            &mut ownership,
            KEY_USAGE_SIGN,
            KEY_USAGE_VERIFY,
        )?;
        super::common::run_activate(client, id, "PrivateKey", &private_id)?;
        super::common::run_activate(client, id, "PublicKey", &public_id)?;
        if required_parameter_text(row, "data_mode")? == "init-update-final" {
            return run_signature_multipart_case(client, row, &private_id, &public_id);
        }
        let message = signature_message(row)?;
        let signature = execute_signature_sign(client, row, &private_id, &message)?;
        let verification_id = match required_case_text(row, "operation")? {
            "Sign" => public_id,
            "Signature Verify" => match required_case_text(row, "object_type")? {
                "PublicKey" => public_id,
                "PrivateKey" => private_id,
                other => {
                    return Err(
                        format!("case {id} has invalid Signature Verify object {other}").into(),
                    );
                }
            },
            other => {
                return Err(
                    format!("case {id} has unsupported signature operation {other}").into(),
                );
            }
        };
        execute_signature_verify(client, row, &verification_id, &message, signature)
    })()
    .map_err(|error: Box<dyn Error>| error.to_string());

    super::common::finish_operation_with_cleanup(operation_result, || {
        super::common::cleanup_owned_objects(client, id, &mut ownership)
    })
    .map_err(|message| -> Box<dyn Error> { message.into() })
}

fn independent_signature(
    row: &JsonValue,
    message: &SignatureMessage,
) -> Result<SecretBytes, Box<dyn Error>> {
    let id = case_id(row)?;
    let SignatureMessage::Raw(data) = message else {
        return Err(format!("case {id} independent fixture signer requires raw data").into());
    };
    let stem = signature_fixture_stem(row)?;
    let key_path = signature_fixture_directory().join(format!("{stem}.key"));
    if !key_path.is_file() {
        return Err(
            format!("case {id} independent signer private-key fixture is unavailable").into(),
        );
    }
    let digest = required_parameter_text(row, "hash")?
        .replace('-', "")
        .to_ascii_lowercase();
    let mut child = independent_openssl_command()?
        .arg("dgst")
        .arg(format!("-{digest}"))
        .arg("-sign")
        .arg(key_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| format!("case {id} independent signer could not start OpenSSL"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| format!("case {id} independent signer input is unavailable"))?
        .write_all(data)
        .map_err(|_| format!("case {id} independent signer input failed"))?;
    let output = child
        .wait_with_output()
        .map_err(|_| format!("case {id} independent signer did not complete"))?;
    if !output.status.success() || output.stdout.is_empty() {
        return Err(format!("case {id} independent signer failed for the selected digest").into());
    }
    Ok(SecretBytes::new(output.stdout))
}

fn register_independent_verification_key(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    id: &str,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let stem = signature_fixture_stem(row)?;
    let directory = signature_fixture_directory();
    let public_bytes = std::fs::read(directory.join(format!("{stem}-public-spki.der")))
        .map_err(|_| format!("case {id} independent signer public-key fixture is unavailable"))?;
    let algorithm = wire_enum_value(row, "cryptographic_algorithm")?;
    let curve = wire_enum_value(row, "recommended_curve")?;
    let bits = row["parameters"]["key_bits"]
        .as_i64()
        .or_else(|| match row["parameters"]["curve"].as_str() {
            Some("P256" | "SECP256K1") => Some(256),
            Some("P384") => Some(384),
            Some("P521") => Some(521),
            _ => None,
        })
        .ok_or_else(|| format!("case {id} independent signer omits key size"))?;
    let public_attributes =
        signature_key_attributes(id, algorithm, curve, bits, KEY_USAGE_VERIFY, None)?;
    let public_object = signature_key_object(public_bytes, 5, algorithm, false)?;
    let public_identifier = register_independent_key(
        client,
        id,
        RegisterRequest::new(ObjectType::from_raw(3), public_attributes, public_object),
        ownership,
    )?;
    super::common::run_activate(client, id, "independent PublicKey", &public_identifier)?;

    if required_case_text(row, "object_type")? == "PublicKey" {
        return Ok(public_identifier);
    }

    let private_bytes = std::fs::read(directory.join(format!("{stem}-private-pkcs8.der")))
        .map_err(|_| format!("case {id} independent signer private-key fixture is unavailable"))?;
    let link = super::kmip_item(
        LINK,
        Value::structure(super::structure([
            super::kmip_item(LINK_TYPE, Value::enumeration(0x0000_0102))?,
            super::kmip_item(
                LINKED_OBJECT_IDENTIFIER,
                Value::text_string(public_identifier.clone()),
            )?,
        ])?),
    )?;
    let private_attributes =
        signature_key_attributes(id, algorithm, curve, bits, KEY_USAGE_VERIFY, Some(link))?;
    let private_object = signature_key_object(private_bytes, 4, algorithm, true)?;
    let private_id = register_independent_key(
        client,
        id,
        RegisterRequest::new(ObjectType::from_raw(4), private_attributes, private_object),
        ownership,
    )?;
    super::common::run_activate(client, id, "independent PrivateKey", &private_id)?;
    Ok(private_id)
}

fn register_independent_key(
    client: &mut kmipkit_client::Client,
    id: &str,
    request: RegisterRequest,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    super::common::execute_case(
        client,
        id,
        ClientRequest::register(request),
        "Register independent signature fixture",
        |response| {
            let identifier = response
                .register()
                .and_then(|result| result.unique_identifier())
                .ok_or_else(|| {
                    "successful Register response has no Unique Identifier".to_owned()
                })?;
            let UniqueIdentifier::TextString(identifier) = identifier else {
                return Err("Register returned a non-text fixture identifier".to_owned());
            };
            ownership.track(identifier.clone());
            Ok(identifier.clone())
        },
    )
}

fn signature_key_attributes(
    id: &str,
    algorithm: u32,
    curve: u32,
    bits: i64,
    usage: i32,
    link: Option<kmipkit_ttlv::Item>,
) -> Result<AttributeSet, Box<dyn Error>> {
    let mut items = vec![
        super::common::name_attribute(id)?,
        super::kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(algorithm))?,
        super::kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(i32::try_from(bits)?))?,
        super::kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(usage))?,
        super::kmip_item(
            CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
            Value::structure(super::structure([super::kmip_item(
                RECOMMENDED_CURVE,
                Value::enumeration(curve),
            )?])?),
        )?,
    ];
    if let Some(link) = link {
        items.push(link);
    }
    Ok(AttributeSet::try_new(items)?)
}

fn signature_key_object(
    material: Vec<u8>,
    key_format: u32,
    algorithm: u32,
    sensitive: bool,
) -> Result<kmipkit_ttlv::Item, Box<dyn Error>> {
    let mut key_value = Structure::new();
    let key_material = if sensitive {
        SecretBytes::new(material).into_ttlv_value()
    } else {
        Value::byte_string(material)
    };
    key_value.try_push(super::kmip_item(KEY_MATERIAL, key_material)?)?;
    let mut key_block = Structure::new();
    key_block.try_push(super::kmip_item(
        KEY_FORMAT_TYPE,
        Value::enumeration(key_format),
    )?)?;
    key_block.try_push(super::kmip_item(KEY_VALUE, Value::structure(key_value))?)?;
    key_block.try_push(super::kmip_item(
        CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(algorithm),
    )?)?;
    let object_tag = if sensitive { PRIVATE_KEY } else { PUBLIC_KEY };
    super::kmip_item(
        object_tag,
        Value::structure(super::structure([super::kmip_item(
            KEY_BLOCK,
            Value::structure(key_block),
        )?])?),
    )
}

fn signature_fixture_stem(row: &JsonValue) -> Result<String, Box<dyn Error>> {
    let id = case_id(row)?;
    if required_case_text(row, "algorithm")? != "ECDSA" {
        return Err(format!("case {id} independent signer fixture is not ECDSA").into());
    }
    let curve = required_parameter_text(row, "curve")?;
    Ok(format!("ec-{curve}"))
}

fn signature_fixture_directory() -> std::path::PathBuf {
    super::certificate_directory().join("validation")
}

fn independent_openssl_command() -> Result<Command, Box<dyn Error>> {
    let configured = std::env::var_os("KMIPKIT_OPENSSL");
    let mut candidates = Vec::new();
    if let Some(path) = configured {
        candidates.push(std::path::PathBuf::from(path));
    }
    candidates.push(std::path::PathBuf::from("openssl"));
    #[cfg(windows)]
    {
        candidates.push(std::path::PathBuf::from(
            r"C:\Program Files\Git\usr\bin\openssl.exe",
        ));
        candidates.push(std::path::PathBuf::from(
            r"C:\Program Files\Git\mingw64\bin\openssl.exe",
        ));
    }
    for candidate in candidates {
        let available = Command::new(&candidate)
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if available {
            return Ok(Command::new(candidate));
        }
    }
    Err("independent signature fixture requires OpenSSL".into())
}

fn run_signature_multipart_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    private_id: &str,
    public_id: &str,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    let operation = required_case_text(row, "operation")?;
    let object_type = required_case_text(row, "object_type")?;
    let identifier = match (operation, object_type) {
        ("Sign", "PrivateKey") => private_id,
        ("Signature Verify", "PublicKey") => public_id,
        ("Signature Verify", "PrivateKey") => private_id,
        _ => return Err(format!("case {id} has invalid multipart signature target").into()),
    };
    match operation {
        "Sign" => {
            let initial = SignRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
                .with_cryptographic_parameters(signature_parameters(row)?)
                .with_init_indicator(true);
            let mut correlation = execute_sign_multipart_part(client, id, initial, "Sign initial")?;
            let middle = SignRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
                .with_cryptographic_parameters(signature_parameters(row)?)
                .with_correlation_value(copy_secret(&correlation))
                .with_init_indicator(false);
            correlation = execute_sign_multipart_part(client, id, middle, "Sign middle")?;
            let final_request = SignRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
                .with_cryptographic_parameters(signature_parameters(row)?)
                .with_correlation_value(correlation)
                .with_final_indicator(true);
            super::common::execute_case(
                client,
                id,
                ClientRequest::sign(final_request),
                "Sign",
                |response| {
                    let result = response.sign().ok_or_else(|| {
                        "target response received; final Sign result is absent".to_owned()
                    })?;
                    if result.signature_data().is_some() {
                        return Err(
                            "target response received; multipart Sign returned Signature Data"
                                .to_owned(),
                        );
                    }
                    Ok(())
                },
            )?;
        }
        "Signature Verify" => {
            let initial = SignatureVerifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
                .with_cryptographic_parameters(signature_parameters(row)?)
                .with_init_indicator(true);
            let mut correlation = execute_signature_verify_multipart_part(
                client,
                id,
                initial,
                "Signature Verify initial",
            )?;
            let middle = SignatureVerifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
                .with_cryptographic_parameters(signature_parameters(row)?)
                .with_correlation_value(copy_secret(&correlation))
                .with_init_indicator(false);
            correlation = execute_signature_verify_multipart_part(
                client,
                id,
                middle,
                "Signature Verify middle",
            )?;
            let final_request = SignatureVerifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
                .with_cryptographic_parameters(signature_parameters(row)?)
                .with_correlation_value(correlation)
                .with_final_indicator(true);
            super::common::execute_case(
                client,
                id,
                ClientRequest::signature_verify(final_request),
                "Signature Verify",
                |response| {
                    response.signature_verify().ok_or_else(|| {
                        "target response received; final Signature Verify result is absent"
                            .to_owned()
                    })?;
                    Ok(())
                },
            )?;
        }
        other => {
            return Err(format!("case {id} has unsupported multipart operation {other}").into());
        }
    }
    Ok(())
}

fn execute_sign_multipart_part(
    client: &mut kmipkit_client::Client,
    id: &str,
    request: SignRequest,
    label: &str,
) -> Result<SecretBytes, Box<dyn Error>> {
    super::common::execute_case(
        client,
        id,
        ClientRequest::sign(request),
        "Sign",
        |response| {
            let result = response
                .sign()
                .ok_or_else(|| format!("target response received; {label} result is absent"))?;
            let correlation = result.correlation_value().ok_or_else(|| {
                format!("target response received; {label} omitted Correlation Value")
            })?;
            Ok(copy_secret(correlation))
        },
    )
}

fn execute_signature_verify_multipart_part(
    client: &mut kmipkit_client::Client,
    id: &str,
    request: SignatureVerifyRequest,
    label: &str,
) -> Result<SecretBytes, Box<dyn Error>> {
    super::common::execute_case(
        client,
        id,
        ClientRequest::signature_verify(request),
        "Signature Verify",
        |response| {
            let result = response
                .signature_verify()
                .ok_or_else(|| format!("target response received; {label} result is absent"))?;
            let correlation = result.correlation_value().ok_or_else(|| {
                format!("target response received; {label} omitted Correlation Value")
            })?;
            Ok(copy_secret(correlation))
        },
    )
}

fn execute_signature_sign(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    identifier: &str,
    message: &SignatureMessage,
) -> Result<SecretBytes, Box<dyn Error>> {
    let id = case_id(row)?;
    let request = SignRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
        .with_cryptographic_parameters(signature_parameters(row)?);
    let request = match message {
        SignatureMessage::Raw(data) => {
            request.with_data(OperationData::ByteString(SecretBytes::new(data.clone())))
        }
        SignatureMessage::Digested(data) => {
            request.with_digested_data(SecretBytes::new(data.clone()))
        }
    };
    super::common::execute_case(
        client,
        id,
        ClientRequest::sign(request),
        "Sign",
        |response| {
            let result = response
                .sign()
                .ok_or_else(|| "successful Sign response has no typed result".to_owned())?;
            let response_id = result
                .unique_identifier()
                .and_then(|value| super::text_identifier(value).ok());
            if response_id.as_deref() != Some(identifier) {
                return Err("Sign returned a different Unique Identifier".to_owned());
            }
            let signature = result
                .signature_data()
                .ok_or_else(|| "successful Sign response has no Signature Data".to_owned())?;
            let signature = signature.with_bytes(|bytes| SecretBytes::new(bytes.to_vec()));
            if signature.with_bytes(<[u8]>::is_empty) {
                return Err("successful Sign response has empty Signature Data".to_owned());
            }
            Ok(signature)
        },
    )
}

fn execute_signature_verify(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
    identifier: &str,
    message: &SignatureMessage,
    signature: SecretBytes,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    let request = SignatureVerifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
        .with_cryptographic_parameters(signature_parameters(row)?)
        .with_signature_data(signature);
    let request = match message {
        SignatureMessage::Raw(data) => {
            request.with_data(OperationData::ByteString(SecretBytes::new(data.clone())))
        }
        SignatureMessage::Digested(data) => {
            request.with_digested_data(SecretBytes::new(data.clone()))
        }
    };
    super::common::execute_case(
        client,
        id,
        ClientRequest::signature_verify(request),
        "Signature Verify",
        |response| {
            let actual = response
                .signature_verify()
                .and_then(|result| result.validity_indicator())
                .map(kmipkit_protocol::ValidityIndicator::raw);
            if actual != Some(1) {
                return Err(format!(
                    "Signature Verify returned Validity Indicator {actual:?}; expected 1"
                ));
            }
            Ok(())
        },
    )?;
    Ok(())
}

fn run_hash_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    if required_parameter_text(row, "data_mode")? == "init-update-final" {
        return run_hash_multipart_case(client, row);
    }
    let algorithm = row["algorithm"]
        .as_str()
        .ok_or_else(|| format!("case {id} has no hash algorithm"))?;
    let enum_value = row["wire_enum_values"]["hashing_algorithm"]["value"]
        .as_str()
        .and_then(|value| u32::from_str_radix(value.trim_start_matches("0x"), 16).ok())
        .ok_or_else(|| format!("case {id} has invalid Hashing Algorithm enum value"))?;
    let empty_input = row["parameters"]["data_length_bytes"].as_u64() == Some(0);
    let input = if empty_input {
        Vec::new()
    } else {
        b"abc".to_vec()
    };
    let expected = decode_hex(known_answer_digest(algorithm, empty_input)?)?;
    let mut parameters = Structure::new();
    parameters.try_push(super::kmip_item(
        HASHING_ALGORITHM,
        Value::enumeration(enum_value),
    )?)?;
    let request = ClientRequest::hash(
        HashRequest::new(parameters).with_data(OperationData::ByteString(SecretBytes::new(input))),
    );

    super::common::execute_case(client, id, request, "Hash", |response| {
        let digest = response
            .hash()
            .and_then(|result| result.data())
            .ok_or_else(|| "successful Hash response has no digest Data".to_owned())?;
        let matches = digest.with_bytes(|bytes| bytes == expected.as_slice());
        if !matches {
            return Err(format!(
                "Hash digest did not match the independent {algorithm} known-answer vector"
            ));
        }
        Ok(())
    })?;
    Ok(())
}

fn run_hash_multipart_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    let initial = HashRequest::new(hash_parameters(row)?).with_init_indicator(true);
    let mut correlation = execute_hash_multipart_part(client, id, initial, "Hash initial", true)?;
    correlation = execute_hash_multipart_part(
        client,
        id,
        HashRequest::new(hash_parameters(row)?)
            .with_correlation_value(copy_secret(&correlation))
            .with_init_indicator(false),
        "Hash middle",
        true,
    )?;
    let final_result = super::common::execute_case(
        client,
        id,
        ClientRequest::hash(
            HashRequest::new(hash_parameters(row)?)
                .with_correlation_value(correlation)
                .with_final_indicator(true),
        ),
        "Hash",
        |response| {
            let result = response.hash().ok_or_else(|| {
                "target response received; final Hash result is absent".to_owned()
            })?;
            if result.data().is_some() {
                return Err("target response received; multipart Hash returned Data".to_owned());
            }
            Ok(())
        },
    );
    final_result.map_err(|error| -> Box<dyn Error> { error })
}

fn hash_parameters(row: &JsonValue) -> Result<Structure, Box<dyn Error>> {
    let mut parameters = Structure::new();
    parameters.try_push(super::kmip_item(
        HASHING_ALGORITHM,
        Value::enumeration(hashing_algorithm_value(required_parameter_text(
            row, "hash",
        )?)?),
    )?)?;
    Ok(parameters)
}

fn execute_hash_multipart_part(
    client: &mut kmipkit_client::Client,
    id: &str,
    request: HashRequest,
    label: &str,
    require_correlation: bool,
) -> Result<SecretBytes, Box<dyn Error>> {
    super::common::execute_case(
        client,
        id,
        ClientRequest::hash(request),
        "Hash",
        |response| {
            let result = response
                .hash()
                .ok_or_else(|| format!("target response received; {label} result is absent"))?;
            let correlation = result.correlation_value().ok_or_else(|| {
                if require_correlation {
                    format!("target response received; {label} omitted Correlation Value")
                } else {
                    format!("{label} omitted Correlation Value")
                }
            })?;
            Ok(copy_secret(correlation))
        },
    )
}

fn copy_secret(value: &SecretBytes) -> SecretBytes {
    value.with_bytes(|bytes| SecretBytes::new(bytes.to_vec()))
}

fn run_mac_family_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    let mut ownership = super::common::CaseOwnership::default();
    let operation_result = run_mac_family_operation(client, id, row, &mut ownership)
        .map_err(|error| error.to_string());
    super::common::finish_operation_with_cleanup(operation_result, || {
        super::common::cleanup_owned_objects(client, id, &mut ownership)
    })
    .map_err(|error| -> Box<dyn Error> { error.into() })
}

fn run_mac_family_operation(
    client: &mut kmipkit_client::Client,
    id: &str,
    row: &JsonValue,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let identifier = import_hmac_key(client, id, row, ownership)?;
    let parameters = mac_parameters(row)?;
    if row["parameters"]["data_mode"].as_str() == Some("init-update-final") {
        return run_mac_multipart_case(client, id, &identifier, row);
    }
    let hash = required_parameter_text(row, "hash")?;
    let expected_mac = decode_hex(known_answer_hmac(hash)?)?;
    let data = b"abc".to_vec();

    match required_case_text(row, "operation")? {
        "MAC" => {
            let mut request = MacRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier))
                .with_data(OperationData::ByteString(SecretBytes::new(data)));
            if let Some(parameters) = parameters {
                request = request.with_cryptographic_parameters(parameters);
            }
            super::common::execute_case(
                client,
                id,
                ClientRequest::mac(request),
                "MAC",
                |response| {
                    let actual = response
                        .mac()
                        .and_then(|result| result.mac_data())
                        .ok_or_else(|| "successful MAC response has no MAC Data".to_owned())?;
                    if !actual.with_bytes(|bytes| bytes == expected_mac.as_slice()) {
                        return Err(format!(
                            "MAC output did not match the independent {hash} HMAC vector"
                        ));
                    }
                    Ok(())
                },
            )?;
        }
        "MAC Verify" => {
            let verification = required_parameter_text(row, "verification")?;
            let mut supplied_mac = expected_mac;
            let expected_indicator = match verification {
                "valid" => 1,
                "invalid" => {
                    let first = supplied_mac
                        .first_mut()
                        .ok_or_else(|| format!("case {id} has an empty HMAC vector"))?;
                    *first ^= 1;
                    2
                }
                other => {
                    return Err(format!("case {id} has unknown verification value {other}").into());
                }
            };
            let mut request = MacVerifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier))
                .with_data(OperationData::ByteString(SecretBytes::new(data)))
                .with_mac_data(SecretBytes::new(supplied_mac));
            if let Some(parameters) = parameters {
                request = request.with_cryptographic_parameters(parameters);
            }
            super::common::execute_case(
                client,
                id,
                ClientRequest::mac_verify(request),
                "MAC Verify",
                |response| {
                    let actual = response
                        .mac_verify()
                        .and_then(MacVerifyResponse::validity_indicator)
                        .map(kmipkit_protocol::ValidityIndicator::raw);
                    if actual != Some(expected_indicator) {
                        return Err(format!(
                            "MAC Verify returned Validity Indicator {actual:?}; expected {expected_indicator}"
                        ));
                    }
                    Ok(())
                },
            )?;
        }
        other => {
            return Err(format!("case {id} has unsupported MAC family operation {other}").into());
        }
    }
    Ok(())
}

fn run_mac_multipart_case(
    client: &mut kmipkit_client::Client,
    id: &str,
    identifier: &str,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    let mut initial = MacRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
        .with_init_indicator(true);
    if let Some(value) = mac_parameters(row)? {
        initial = initial.with_cryptographic_parameters(value);
    }
    let mut correlation = execute_mac_multipart_part(client, id, initial, "MAC initial")?;

    let mut middle = MacRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
        .with_correlation_value(copy_secret(&correlation))
        .with_init_indicator(false);
    if let Some(value) = mac_parameters(row)? {
        middle = middle.with_cryptographic_parameters(value);
    }
    correlation = execute_mac_multipart_part(client, id, middle, "MAC middle")?;

    let mut final_request = MacRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
        .with_correlation_value(correlation)
        .with_final_indicator(true);
    if let Some(value) = mac_parameters(row)? {
        final_request = final_request.with_cryptographic_parameters(value);
    }
    super::common::execute_case(
        client,
        id,
        ClientRequest::mac(final_request),
        "MAC",
        |response| {
            let result = response
                .mac()
                .ok_or_else(|| "target response received; final MAC result is absent".to_owned())?;
            if result.mac_data().is_some() {
                return Err("target response received; multipart MAC returned MAC Data".to_owned());
            }
            Ok(())
        },
    )?;
    Ok(())
}

fn execute_mac_multipart_part(
    client: &mut kmipkit_client::Client,
    id: &str,
    request: MacRequest,
    label: &str,
) -> Result<SecretBytes, Box<dyn Error>> {
    super::common::execute_case(client, id, ClientRequest::mac(request), "MAC", |response| {
        let result = response
            .mac()
            .ok_or_else(|| format!("target response received; {label} result is absent"))?;
        let correlation = result.correlation_value().ok_or_else(|| {
            format!("target response received; {label} omitted Correlation Value")
        })?;
        Ok(copy_secret(correlation))
    })
}

fn import_hmac_key(
    client: &mut kmipkit_client::Client,
    case_id: &str,
    row: &JsonValue,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let selection = row["parameters"]["hash_selection"].as_str();
    let algorithm_value = wire_enum_value(row, "cryptographic_algorithm")?;
    let attribute_algorithm = selection != Some("key-block-algorithm");
    let block_algorithm = selection != Some("attribute-algorithm");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let identifier = format!("{case_id}-key-{now}");
    ownership.track(identifier.clone());

    let mut key_value = Structure::new();
    key_value.try_push(super::kmip_item(
        KEY_MATERIAL,
        SecretBytes::new(FIXTURE_KEY.to_vec()).into_ttlv_value(),
    )?)?;
    let mut key_block = Structure::new();
    key_block.try_push(super::kmip_item(
        KEY_FORMAT_TYPE,
        Value::enumeration(RAW_KEY_FORMAT_TYPE),
    )?)?;
    key_block.try_push(super::kmip_item(KEY_VALUE, Value::structure(key_value))?)?;
    if block_algorithm {
        key_block.try_push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(algorithm_value),
        )?)?;
    }
    let object = super::kmip_item(
        SYMMETRIC_KEY,
        Value::structure(super::structure([super::kmip_item(
            KEY_BLOCK,
            Value::structure(key_block),
        )?])?),
    )?;

    let mut attributes = vec![
        super::kmip_item(
            CRYPTOGRAPHIC_LENGTH,
            Value::integer(i32::try_from(FIXTURE_KEY.len() * 8)?),
        )?,
        super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(KEY_USAGE_MAC_GENERATE_VERIFY),
        )?,
        super::common::name_attribute(case_id)?,
    ];
    if attribute_algorithm {
        attributes.push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(algorithm_value),
        )?);
    }
    let attributes = AttributeSet::try_new(attributes)?;
    super::common::execute_case(
        client,
        case_id,
        ClientRequest::import(ImportRequest::new(
            UniqueIdentifier::TextString(identifier.clone()),
            ObjectType::from_raw(2),
            attributes,
            object,
        )),
        "Import HMAC fixture",
        |response| {
            let result = response
                .import()
                .ok_or_else(|| "successful Import response has no typed result".to_owned())?;
            if !matches!(result.unique_identifier(), Some(UniqueIdentifier::TextString(actual)) if actual == &identifier)
            {
                return Err("Import did not retain the case-owned HMAC key identifier".to_owned());
            }
            Ok(())
        },
    )?;
    super::common::execute_case(
        client,
        case_id,
        ClientRequest::activate(ActivateRequest::new(Some(UniqueIdentifier::TextString(
            identifier.clone(),
        )))),
        "Activate HMAC fixture",
        |response| {
            let returned = response
                .activate()
                .and_then(|result| result.unique_identifier());
            if !matches!(returned, Some(UniqueIdentifier::TextString(actual)) if actual == &identifier)
            {
                return Err("Activate did not retain the case-owned HMAC key identifier".to_owned());
            }
            Ok(())
        },
    )?;
    Ok(identifier)
}

fn mac_parameters(row: &JsonValue) -> Result<Option<Structure>, Box<dyn Error>> {
    if row["parameters"]["hash_selection"].as_str() != Some("explicit-hash") {
        return Ok(None);
    }
    let algorithm = hashing_algorithm_value(required_parameter_text(row, "hash")?)?;
    let mut parameters = Structure::new();
    parameters.try_push(super::kmip_item(
        HASHING_ALGORITHM,
        Value::enumeration(algorithm),
    )?)?;
    Ok(Some(parameters))
}

fn hashing_algorithm_value(name: &str) -> Result<u32, Box<dyn Error>> {
    // The Hashing Algorithm enum values are those carried by the row's
    // `oasis-11.21-t454` citation; MAC rows do not repeat them in wire_enum_values.
    let value = match name {
        "SHA-1" => 4,
        "SHA-224" => 5,
        "SHA-256" => 6,
        "SHA-384" => 7,
        "SHA-512" => 8,
        "SHA3-224" => 14,
        "SHA3-256" => 15,
        "SHA3-384" => 16,
        "SHA3-512" => 17,
        other => return Err(format!("no Hashing Algorithm enum for {other}").into()),
    };
    Ok(value)
}

fn wire_enum_value(row: &JsonValue, member: &str) -> Result<u32, Box<dyn Error>> {
    let id = case_id(row)?;
    row["wire_enum_values"][member]["value"]
        .as_str()
        .and_then(|value| u32::from_str_radix(value.trim_start_matches("0x"), 16).ok())
        .ok_or_else(|| format!("case {id} has no valid {member} enum value").into())
}

fn required_case_text<'a>(row: &'a JsonValue, member: &str) -> Result<&'a str, Box<dyn Error>> {
    let id = case_id(row)?;
    row[member]
        .as_str()
        .ok_or_else(|| format!("case {id} has no textual {member}").into())
}

fn required_parameter_text<'a>(
    row: &'a JsonValue,
    member: &str,
) -> Result<&'a str, Box<dyn Error>> {
    let id = case_id(row)?;
    row["parameters"][member]
        .as_str()
        .ok_or_else(|| format!("case {id} has no textual parameter {member}").into())
}

fn known_answer_hmac(hash: &str) -> Result<&'static str, String> {
    // Expected HMACs are for key 0x5A repeated 32 times and data `abc`.
    match hash {
        "SHA-1" => Ok("465955c5d6fc3877696fb0516385dc7c54b44b34"),
        "SHA-224" => Ok("b3a129ec4d05925f13230323c1a3bbd4fdd31c3d2e77eb233434027c"),
        "SHA-256" => Ok("4ac3ba798b0bd780cf9c24f34dab801e8540602f161a5ce9ef4006194ce5c49d"),
        "SHA-384" => Ok(
            "ea9c435530d8e352be653b9a4d1dea0b6ed050280ccdb549865e49100017fe257d13068703864888b1c3fc9b487686e7",
        ),
        "SHA-512" => Ok(
            "495c2d83300efacedba65947c56f2b06845ee85145c849ae69958f29251fea25fc3e733e03be26945c48f8dc45572606355dafdc40f80052490c36033184c3bb",
        ),
        "SHA3-224" => Ok("ad86534ca722f9b1b68c86b1be3a7f9d089cda54e9343949f9df7cbc"),
        "SHA3-256" => Ok("41b7d9fd35a69db98715c9d1bf037a32a89da861f055f5e59371de248907dc62"),
        "SHA3-384" => Ok(
            "ded37ab21d73cac46e807c18778398e99514fa0a11aaef4f4065f61c627b58603118daf0fab347cbcd1786636fa0f723",
        ),
        "SHA3-512" => Ok(
            "8c9f0ef8e907858bbfd123ea87edd4a0e8520af8022f175c908dc188e2322c88613f59c1c0bd7f498b99836edea264e59ae9d4bb54dfb1ee4ca8ee87e9b0a612",
        ),
        other => Err(format!(
            "no independent HMAC known-answer vector for {other}"
        )),
    }
}

fn known_answer_digest(algorithm: &str, empty_input: bool) -> Result<&'static str, String> {
    let digest = match (algorithm, empty_input) {
        ("SHA-224", true) => "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f",
        ("SHA-224", false) => "23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7",
        ("SHA-256", true) => "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ("SHA-256", false) => "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ("SHA-384", true) => {
            "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b"
        }
        ("SHA-384", false) => {
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7"
        }
        ("SHA-512", true) => {
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
        }
        ("SHA-512", false) => {
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        }
        ("SHA3-224", true) => "6b4e03423667dbb73b6e15454f0eb1abd4597f9a1b078e3f5b5a6bc7",
        ("SHA3-224", false) => "e642824c3f8cf24ad09234ee7d3c766fc9a3a5168d0c94ad73b46fdf",
        ("SHA3-256", true) => "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
        ("SHA3-256", false) => "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532",
        ("SHA3-384", true) => {
            "0c63a75b845e4f7d01107d852e4c2485c51a50aaaa94fc61995e71bbee983a2ac3713831264adb47fb6bd1e058d5f004"
        }
        ("SHA3-384", false) => {
            "ec01498288516fc926459f58e2c6ad8df9b473cb0fc08c2596da7cf0e49be4b298d88cea927ac7f539f1edf228376d25"
        }
        ("SHA3-512", true) => {
            "a69f73cca23a9ac5c8b567dc185a756e97c982164fe25859e0d1dcc1475c80a615b2123af1f5f94c11e3e9402c3ac558f500199d95b6d3e301758586281dcd26"
        }
        ("SHA3-512", false) => {
            "b751850b1a57168a5693cd924b6b096e08f621827444f70d884f5d0240d2712e10e116e9192af3c91a7ec57647e3934057340b4cf408d5a56592f8274eec53f0"
        }
        _ => {
            return Err(format!(
                "no independent known-answer vector for {algorithm}"
            ));
        }
    };
    Ok(digest)
}

fn decode_hex(value: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if !value.len().is_multiple_of(2) {
        return Err("known-answer digest hex has odd length".into());
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for index in (0..bytes.len()).step_by(2) {
        let high = hex_nibble(bytes[index])?;
        let low = hex_nibble(bytes[index + 1])?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

fn hex_nibble(value: u8) -> Result<u8, Box<dyn Error>> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err("known-answer vector contains a non-hexadecimal character".into()),
    }
}
