//! Opt-in integration tests against a local Cosmian KMS 5.28.0 deployment.
//!
//! The tests use `KMIPKit`'s typed Rust client and raw-TLS/mTLS transport for
//! lifecycle, attribute, query, and cryptographic operations. They exercise
//! selected KMIP 2.1 requests; they are not a full conformance or profile test.

#[path = "cosmian_kms/common.rs"]
mod common;
#[path = "cosmian_kms/cryptographic.rs"]
mod cryptographic;
#[path = "cosmian_kms/cryptographic_cipher.rs"]
mod cryptographic_cipher;
#[path = "cosmian_kms/cryptographic_covercrypt.rs"]
mod cryptographic_covercrypt;
#[path = "cosmian_kms/cryptographic_kem.rs"]
mod cryptographic_kem;
#[path = "cosmian_kms/key_management.rs"]
mod key_management;
#[path = "cosmian_kms/lifecycle.rs"]
mod lifecycle;
#[path = "cosmian_kms/random.rs"]
mod random;
#[path = "cosmian_kms/transfer.rs"]
mod transfer;

use std::collections::BTreeSet;
use std::error::Error;
use std::path::{Path, PathBuf};

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientError, ClientRequest, ClientResponseView,
};
use kmipkit_protocol::extension;
use kmipkit_protocol::{
    ActivateRequest, AddAttributeRequest, AttributeReference, AttributeSet, CreateKeyPairRequest,
    CreateRequest, CreateSplitKeyRequest, CurrentAttribute, DecryptRequest, DeleteAttributeRequest,
    EncryptRequest, GetAttributeListRequest, GetAttributesRequest, HashRequest, MacRequest,
    MacVerifyRequest, ModifyAttributeRequest, NewAttribute, ObjectType, OperationData,
    QueryFunction, QueryRequest, SecretBytes, SetAttributeRequest, SignRequest,
    SignatureVerifyRequest, SplitKeyMethod, UniqueIdentifier,
};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

const COMMENT: u32 = 0x0042_00FD;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const QUERY_PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const AES: u32 = 3;
const RSA: u32 = 4;
const SYMMETRIC_KEY: u32 = 2;
const SUCCESS: u32 = 0;

// Cosmian 5.28.0 support.md has 34 supported rows: Set Attribute (Modify)
// names two KMIP paths. Query remains advertised despite its response defect.
const COSMIAN_SUPPORTED_PATHS: [&str; 35] = [
    "Activate",
    "Add Attribute",
    "Certify",
    "Check",
    "Create",
    "Create Key Pair",
    "Create Split Key",
    "Decrypt",
    "Delete Attribute",
    "Derive Key",
    "Destroy",
    "Discover Versions",
    "Encrypt",
    "Export",
    "Get",
    "Get Attribute List",
    "Get Attributes",
    "Hash",
    "Import",
    "Join Split Key",
    "Locate",
    "MAC",
    "MAC Verify",
    "Modify Attribute",
    "Query",
    "RNG Retrieve",
    "RNG Seed",
    "Re-key",
    "Re-key Key Pair",
    "Register",
    "Revoke",
    "Set Attribute",
    "Sign",
    "Signature Verify",
    "Validate",
];

#[test]
#[should_panic(expected = "executable case cannot remain not_run")]
fn executable_matrix_case_must_have_a_measured_outcome() {
    let row = serde_json::json!({
        "case_id": "cosmian-528-regression-not-run",
        "expected_outcome": "typed_success",
        "execution_status": "planned_for_tasks_4_6",
        "observed_result": {"classification": "not_run"}
    });

    validate_capability_execution_status(&row);
}

#[test]
// Inventory contract: KMIPKIT-0015-FR-011/016, not a server conformance test.
// Normative operation clauses: OASIS KMIP v2.1 §6.1; individual exact
// operation/data sections and tables are mandatory in every manifest row.
fn cosmian_capability_manifest_is_complete() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/integration/cosmian/capability-matrix.json"
    ))
    .expect("capability manifest must be valid JSON");
    let resolved = resolve_capability_sources(&manifest);
    let rows = resolved.as_slice();
    let mut case_ids = BTreeSet::new();
    let mut supported_paths = BTreeSet::new();
    for row in rows {
        validate_capability_row(row);
        let case_id = manifest_text(row, "case_id");
        assert!(
            case_ids.insert(case_id),
            "duplicate matrix case_id: {case_id}"
        );
        let operation = manifest_text(row, "operation");
        let expected = manifest_text(row, "expected_outcome");
        if COSMIAN_SUPPORTED_PATHS.contains(&operation) {
            supported_paths.insert(operation);
            if operation == "Query" {
                assert_eq!(
                    expected, "known_response_shape_failure",
                    "Query must retain its known response-shape failure classification: {case_id}"
                );
            }
        } else {
            assert!(
                matches!(operation, "Ping" | "Adjust Attribute" | "Re-certify"),
                "unclassified operation outside the supported inventory: {case_id}"
            );
            assert!(
                matches!(
                    expected,
                    "unsupported_probe"
                        | "source_documentation_discrepancy"
                        | "client_blocked"
                        | "setup_blocked"
                ),
                "separate probe must not be classified as supported success: {case_id}"
            );
        }
    }
    let missing: Vec<_> = COSMIAN_SUPPORTED_PATHS
        .iter()
        .filter(|operation| !supported_paths.contains(**operation))
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "missing supported operation paths/cases (34 support-table rows, 35 paths; Set Attribute and Modify Attribute are separate): {}",
        missing.join(", ")
    );
    validate_capability_dimensions(rows);
    let issues = reviewed_inventory_issues(rows);
    let summary = [
        "finite-case:",
        "material-citation:",
        "MGF-citation:",
        "ECDSA-digest:",
        "CSR-exclusion:",
    ]
    .into_iter()
    .filter_map(|kind| {
        let count = issues
            .iter()
            .filter(|issue| issue.starts_with(kind))
            .count();
        issues
            .iter()
            .find(|issue| issue.starts_with(kind))
            .map(|example| format!("{kind} {count}; example: {example}"))
    })
    .collect::<Vec<_>>()
    .join("\n");
    assert!(
        issues.is_empty(),
        "reviewed finite inventory contract: {} issues; first findings:\n{}",
        issues.len(),
        summary
    );
    review_contract_detects_removed_dimensions(rows);
}

#[test]
fn reviewed_generation_inventory_matches_covercrypt_create_pair_format() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/integration/cosmian/capability-matrix.json"
    ))
    .expect("capability manifest must be valid JSON");
    let rows = resolve_capability_sources(&manifest);
    let covercrypt_pair = rows
        .iter()
        .find(|row| {
            row["case_id"]
                == "cosmian-528-create-key-pair-privatekey-covercrypt-0x88800004-disposable-master-access-structure"
        })
        .expect("CoverCrypt Create Key Pair case");
    assert_eq!(covercrypt_pair["key_format"], "CoverCryptSecretKey");

    let mut issues = Vec::new();
    review_generation_cases(&rows, &mut issues);
    assert!(
        !issues.iter().any(|issue| {
            issue.starts_with(
                "finite-case: Create Key Pair/PrivateKey/Some(\"CoverCrypt\")/None/None",
            )
        }),
        "reviewed inventory must match the CoverCrypt Create Key Pair vendor key format"
    );
}

#[test]
fn configurable_kem_nested_pq_values_remain_discrepancy_probes() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/integration/cosmian/capability-matrix.json"
    ))
    .expect("capability manifest must be valid JSON");
    let rows = resolve_capability_sources(&manifest);
    let nested_probe_rows = rows
        .iter()
        .filter(|row| {
            row["algorithm"] == "ConfigurableKEM"
                && row["wire_enum_values"]["nested_post_quantum_algorithm"]["classification"]
                    == "source_only_numeric_probe"
        })
        .collect::<Vec<_>>();

    assert_eq!(nested_probe_rows.len(), 6);
    for row in nested_probe_rows {
        assert_eq!(row["expected_outcome"], "source_documentation_discrepancy");
        assert_eq!(
            row["wire_enum_values"]["cryptographic_algorithm"]["value"],
            "0x88800003"
        );
        validate_wire_enum_values(row);
    }
}

fn review_contract_detects_removed_dimensions(rows: &[serde_json::Value]) {
    let without_rsa_3072_sign: Vec<_> = rows
        .iter()
        .filter(|row| {
            !(row["operation"] == "Sign"
                && row["algorithm"] == "RSA"
                && row["parameters"]["key_bits"] == 3072)
        })
        .cloned()
        .collect();
    let mut issues = Vec::new();
    review_signature_cases(&without_rsa_3072_sign, &mut issues);
    assert_eq!(
        issues.len(),
        28,
        "removing RSA-3072 Sign must expose all 28 missing hash/scheme/data cases"
    );
    let without_raw_chacha_export: Vec<_> = rows
        .iter()
        .filter(|row| {
            !(row["operation"] == "Export"
                && row["object_type"] == "SymmetricKey"
                && row["algorithm"] == "ChaCha20"
                && row["key_format"] == "Raw"
                && row["parameters"]["requested_key_format"] == "Raw"
                && row["fixture"]["encoding"] == "native")
        })
        .cloned()
        .collect();
    issues.clear();
    review_transfer_cases(&without_raw_chacha_export, &mut issues);
    assert_eq!(
        issues.len(),
        1,
        "a default/KEK row must not mask a missing explicit subject-format case"
    );
}

const REVIEW_RSA_BITS: [u64; 3] = [2048, 3072, 4096];
const REVIEW_EC_CURVES: [&str; 9] = [
    "P192",
    "P224",
    "P256",
    "P384",
    "P521",
    "SECP224K1",
    "SECP256K1",
    "CURVE25519",
    "CURVE448",
];
const REVIEW_HASHES: [&str; 9] = [
    "SHA-1", "SHA-224", "SHA-256", "SHA-384", "SHA-512", "SHA3-224", "SHA3-256", "SHA3-384",
    "SHA3-512",
];

// Closed source branches are enumerated independently of case IDs/counts.
// Missing a single size/curve/hash/format/encoding intersection is an error.
fn reviewed_inventory_issues(rows: &[serde_json::Value]) -> Vec<String> {
    let mut issues = Vec::new();
    review_generation_cases(rows, &mut issues);
    review_signature_cases(rows, &mut issues);
    review_certify_cases(rows, &mut issues);
    review_pqc_subject_containers(rows, &mut issues);
    review_transfer_cases(rows, &mut issues);
    review_wrapping_cases(rows, &mut issues);
    review_additional_wrapping_paths(rows, &mut issues);
    for row in rows {
        review_material_citations(row, &mut issues);
        if csr_subject_cannot_sign(row) && row["expected_outcome"] != "standards_invalid_exclusion"
        {
            issues.push(format!(
                "CSR-exclusion: {} needs a non-executable PKCS10 proof-of-possession exclusion",
                manifest_text(row, "case_id")
            ));
        }
        let p = &row["parameters"];
        if row["operation"] == "Sign"
            && row["algorithm"] == "ECDSA"
            && p["data_mode"] == "raw"
            && matches!(p["curve"].as_str(), Some("P256" | "SECP256K1"))
            && matches!(p["hash"].as_str(), Some("SHA-384" | "SHA-512"))
            && (row["expected_outcome"] != "source_documentation_discrepancy"
                || p["source_condition"] != "non-fips"
                || p["verify_requested_digest_independently"] != true
                || p["accept_sha256_substitution"] != false)
        {
            issues.push(format!(
                "ECDSA-digest: {} must retain conditional non-fips SHA256 substitution finding",
                manifest_text(row, "case_id")
            ));
        }
    }
    issues
}

fn csr_subject_cannot_sign(row: &serde_json::Value) -> bool {
    row["operation"] == "Certify"
        && row["object_type"] == "CertificateRequest"
        && (matches!(
            row["algorithm"].as_str(),
            Some("MLKEM_512" | "MLKEM_768" | "MLKEM_1024")
        ) || matches!(
            row["parameters"]["curve"].as_str(),
            Some("CURVE25519" | "CURVE448")
        ))
}

// Keep each source-backed capability axis explicit at callsites; bundling them
// would make finite-case assertions harder to audit against the manifest.
#[allow(clippy::too_many_arguments)]
fn demand_review_case(
    rows: &[serde_json::Value],
    operation: &str,
    object: &str,
    algorithm: Option<&str>,
    format: Option<&str>,
    parameters: &serde_json::Value,
    fixture: Option<&str>,
    issues: &mut Vec<String>,
) {
    let found = rows.iter().any(|row| {
        row["operation"] == operation
            && row["object_type"] == object
            && row["algorithm"].as_str() == algorithm
            && row["key_format"].as_str() == format
            && row["fixture"]["encoding"].as_str() == fixture
            && parameters
                .as_object()
                .expect("finite case parameters")
                .iter()
                .all(|(key, value)| row["parameters"][key] == *value)
    });
    if !found {
        issues.push(format!(
            "finite-case: {operation}/{object}/{algorithm:?}/{format:?}/{fixture:?} {parameters}"
        ));
    }
}

fn reviewed_symmetric_variants() -> Vec<(&'static str, u64)> {
    let mut variants = vec![
        ("AES", 128),
        ("AES", 192),
        ("AES", 256),
        ("AES", 512),
        ("ChaCha20", 256),
        ("ChaCha20Poly1305", 256),
        ("FPE_FF1", 256),
        ("3DES", 112),
        ("3DES", 168),
    ];
    for algorithm in [
        "HMAC-SHA-1",
        "HMAC-SHA-224",
        "HMAC-SHA-256",
        "HMAC-SHA-384",
        "HMAC-SHA-512",
        "SHA3-224",
        "SHA3-256",
        "SHA3-384",
        "SHA3-512",
        "SHAKE-128",
        "SHAKE-256",
    ] {
        variants.push((algorithm, 256));
    }
    variants
}

fn reviewed_pair_variants() -> Vec<(&'static str, serde_json::Value)> {
    let mut variants = Vec::new();
    for bits in REVIEW_RSA_BITS {
        variants.push(("RSA", serde_json::json!({"key_bits":bits})));
    }
    for algorithm in ["EC", "ECDH", "ECDSA"] {
        for curve in REVIEW_EC_CURVES {
            if algorithm == "ECDSA" && matches!(curve, "CURVE25519" | "CURVE448") {
                continue;
            }
            variants.push((algorithm, serde_json::json!({"curve":curve})));
        }
    }
    for (algorithm, curve) in [("Ed25519", "CURVEED25519"), ("Ed448", "CURVEED448")] {
        variants.push((algorithm, serde_json::json!({"curve":curve})));
    }
    variants
}

fn reviewed_pair_key_format(operation: &str, algorithm: &str) -> Option<&'static str> {
    match (operation, algorithm) {
        ("Create Key Pair", "CoverCrypt") => Some("CoverCryptSecretKey"),
        _ => None,
    }
}

// This finite table is deliberately exhaustive over reviewed generation variants.
#[allow(clippy::too_many_lines)]
fn review_generation_cases(rows: &[serde_json::Value], issues: &mut Vec<String>) {
    for (algorithm, bits) in reviewed_symmetric_variants() {
        for selection in [None, Some("TransparentSymmetricKey")] {
            for operation in ["Create", "Re-key"] {
                demand_review_case(
                    rows,
                    operation,
                    "SymmetricKey",
                    Some(algorithm),
                    Some("TransparentSymmetricKey"),
                    &serde_json::json!({"key_bits":bits,"requested_key_format":selection}),
                    None,
                    issues,
                );
            }
        }
    }
    for bits in [8, 8192] {
        demand_review_case(
            rows,
            "Create",
            "SymmetricKey",
            Some("HMAC-SHA-256"),
            Some("TransparentSymmetricKey"),
            &serde_json::json!({"key_bits":bits}),
            None,
            issues,
        );
    }
    demand_review_case(
        rows,
        "Create",
        "SecretData",
        None,
        Some("Raw"),
        &serde_json::json!({"output_secret_data_type":"Seed","output_bytes":32}),
        None,
        issues,
    );
    for bits in [2048, 3072] {
        demand_review_case(
            rows,
            "Create",
            "PrivateKey",
            Some("DSA"),
            Some("TransparentDSAPrivateKey"),
            &serde_json::json!({"key_bits":bits}),
            None,
            issues,
        );
    }
    demand_review_case(
        rows,
        "Create",
        "PrivateKey",
        Some("CoverCrypt"),
        Some("CoverCryptSecretKey"),
        &serde_json::json!({"vendor_algorithm_value":"0x88800004"}),
        None,
        issues,
    );
    for operation in ["Create Key Pair", "Re-key Key Pair"] {
        for (algorithm, parameters) in reviewed_pair_variants() {
            demand_review_case(
                rows,
                operation,
                "PrivateKey",
                Some(algorithm),
                None,
                &parameters,
                None,
                issues,
            );
        }
        for (algorithm, value) in [
            ("CoverCrypt", "0x88800004"),
            ("X25519MLKEM768", "0x88800006"),
            ("X448MLKEM1024", "0x88800007"),
        ] {
            demand_review_case(
                rows,
                operation,
                "PrivateKey",
                Some(algorithm),
                reviewed_pair_key_format(operation, algorithm),
                &serde_json::json!({"vendor_algorithm_value":value}),
                None,
                issues,
            );
        }
        for curve in ["P256", "CURVE25519"] {
            demand_review_case(
                rows,
                operation,
                "PrivateKey",
                Some("ConfigurableKEM"),
                None,
                &serde_json::json!({"pre_quantum":curve,"post_quantum":null}),
                None,
                issues,
            );
        }
    }
}

fn review_signature_cases(rows: &[serde_json::Value], issues: &mut Vec<String>) {
    for (operation, object) in [
        ("Sign", "PrivateKey"),
        ("Signature Verify", "PublicKey"),
        ("Signature Verify", "PrivateKey"),
    ] {
        for bits in REVIEW_RSA_BITS {
            for hash in [
                "SHA-1", "SHA-256", "SHA-384", "SHA-512", "SHA3-256", "SHA3-384", "SHA3-512",
            ] {
                for scheme in ["RSASSA-PSS", "PKCS1v15"] {
                    for mode in ["raw", "prehashed"] {
                        demand_review_case(
                            rows,
                            operation,
                            object,
                            Some("RSA"),
                            None,
                            &serde_json::json!({"key_bits":bits,"hash":hash,"signature_scheme":scheme,"data_mode":mode,
                                "mgf_hash":if scheme=="RSASSA-PSS" {Some(hash)} else {None},
                                "salt_length_bytes":if scheme=="RSASSA-PSS" {Some("digest")} else {None}}),
                            None,
                            issues,
                        );
                    }
                }
            }
        }
        for curve in REVIEW_EC_CURVES.into_iter().take(7) {
            for hash in ["SHA-256", "SHA-384", "SHA-512"] {
                for mode in ["raw", "prehashed"] {
                    demand_review_case(
                        rows,
                        operation,
                        object,
                        Some("ECDSA"),
                        None,
                        &serde_json::json!({"curve":curve,"hash":hash,"data_mode":mode}),
                        None,
                        issues,
                    );
                }
            }
        }
        for (algorithm, curve) in [("Ed25519", "CURVEED25519"), ("Ed448", "CURVEED448")] {
            demand_review_case(
                rows,
                operation,
                object,
                Some(algorithm),
                None,
                &serde_json::json!({"curve":curve,"data_mode":"raw","external_digest":null}),
                None,
                issues,
            );
        }
    }
    for (operation, object) in [("Sign", "PrivateKey"), ("Signature Verify", "PublicKey")] {
        for salt in [0, 32] {
            for mgf in ["SHA-256", "SHA-384"] {
                demand_review_case(
                    rows,
                    operation,
                    object,
                    Some("RSA"),
                    None,
                    &serde_json::json!({"key_bits":2048,"hash":"SHA-256","signature_scheme":"RSASSA-PSS",
                        "data_mode":"raw","salt_length_bytes":salt,"mgf_hash":mgf}),
                    None,
                    issues,
                );
            }
        }
        demand_review_case(
            rows,
            operation,
            object,
            Some("RSA"),
            None,
            &serde_json::json!({"key_bits":2048,"hash":"SHA-256","signature_scheme":"RSASSA-PSS","data_mode":"init-update-final"}),
            None,
            issues,
        );
    }
}

// This finite table records the reviewed issuer/subject cross-product explicitly.
#[allow(clippy::too_many_lines)]
fn review_certify_cases(rows: &[serde_json::Value], issues: &mut Vec<String>) {
    for subject in [
        "csr-pem",
        "csr-pkcs10-der",
        "existing-public",
        "renew-certificate",
        "generated-subject",
    ] {
        let object = if subject.starts_with("csr") {
            "CertificateRequest"
        } else if subject == "renew-certificate" {
            "Certificate"
        } else {
            "PublicKey"
        };
        let fixture = if subject == "csr-pem" {
            "PEM"
        } else if subject == "csr-pkcs10-der" {
            "DER"
        } else {
            "native"
        };
        for issuer in ["self", "RSA", "EC", "Ed25519", "Ed448"] {
            demand_review_case(
                rows,
                "Certify",
                object,
                Some("RSA"),
                Some("X.509"),
                &serde_json::json!({"subject_input":subject,"subject_key_bits":2048,"issuer":issuer}),
                Some(fixture),
                issues,
            );
        }
    }
    // CSR proof-of-possession uses signature-capable EC/Ed keys; exchange-only
    // X25519/X448 have existing-public subject cases, not invented CSR signers.
    let mut subjects = vec![
        ("RSA", serde_json::json!({"subject_key_bits":3072})),
        ("RSA", serde_json::json!({"subject_key_bits":4096})),
    ];
    for curve in REVIEW_EC_CURVES.into_iter().take(7) {
        subjects.push(("EC", serde_json::json!({"curve":curve})));
    }
    for (algorithm, curve) in [("Ed25519", "CURVEED25519"), ("Ed448", "CURVEED448")] {
        subjects.push((algorithm, serde_json::json!({"curve":curve})));
    }
    for (algorithm, parameters) in subjects {
        for (subject, encoding) in [("csr-pem", "PEM"), ("csr-pkcs10-der", "DER")] {
            for issuer in ["self", "RSA", "EC", "Ed25519", "Ed448"] {
                let mut p = parameters.clone();
                p["subject_input"] = subject.into();
                p["issuer"] = issuer.into();
                demand_review_case(
                    rows,
                    "Certify",
                    "CertificateRequest",
                    Some(algorithm),
                    Some("X.509"),
                    &p,
                    Some(encoding),
                    issues,
                );
            }
        }
    }
    for curve in [
        "P224",
        "P256",
        "P384",
        "P521",
        "SECP256K1",
        "CURVE25519",
        "CURVE448",
        "CURVEED25519",
        "CURVEED448",
    ] {
        demand_review_case(
            rows,
            "Certify",
            "PublicKey",
            Some("EC"),
            Some("X.509"),
            &serde_json::json!({"subject_input":"existing-public","curve":curve,"issuer":"RSA"}),
            None,
            issues,
        );
    }
    for (algorithm, parameters) in reviewed_pair_variants() {
        if algorithm == "RSA" && parameters["key_bits"] == 2048 {
            continue;
        }
        let mut p = parameters;
        p["subject_input"] = "generated-subject".into();
        p["issuer"] = "RSA".into();
        demand_review_case(
            rows,
            "Certify",
            "PublicKey",
            Some(algorithm),
            Some("X.509"),
            &p,
            Some("native"),
            issues,
        );
    }
}

fn demand_transfer_formats(
    rows: &[serde_json::Value],
    object: &str,
    algorithm: Option<&str>,
    parameters: &serde_json::Value,
    formats: &[&str],
    default: &str,
    issues: &mut Vec<String>,
) {
    for operation in ["Import", "Register", "Get", "Export"] {
        for selection in std::iter::once(None).chain(formats.iter().copied().map(Some)) {
            if selection.is_none() && matches!(operation, "Import" | "Register") {
                continue;
            }
            let format = selection.unwrap_or(default);
            let encodings: &[&str] =
                if matches!(format, "X.509" | "PKCS#1" | "PKCS#8" | "ECPrivateKey") {
                    &["DER", "PEM"]
                } else {
                    &["native"]
                };
            for encoding in encodings {
                let mut p = parameters.clone();
                p["requested_key_format"] = serde_json::json!(selection);
                demand_review_case(
                    rows,
                    operation,
                    object,
                    algorithm,
                    Some(format),
                    &p,
                    Some(encoding),
                    issues,
                );
            }
        }
    }
}

// This finite table is the source-backed transfer-format intersection checklist.
#[allow(clippy::too_many_lines)]
fn review_transfer_cases(rows: &[serde_json::Value], issues: &mut Vec<String>) {
    for (algorithm, parameters) in reviewed_pair_variants() {
        demand_transfer_formats(
            rows,
            "Certificate",
            Some(algorithm),
            &parameters,
            &["X.509"],
            "X.509",
            issues,
        );
        for object in ["PrivateKey", "PublicKey"] {
            let private = object == "PrivateKey";
            if algorithm == "RSA" {
                let transparent = if private {
                    "TransparentRSAPrivateKey"
                } else {
                    "TransparentRSAPublicKey"
                };
                demand_transfer_formats(
                    rows,
                    object,
                    Some(algorithm),
                    &parameters,
                    &["PKCS#1", "PKCS#8", transparent],
                    "PKCS#1",
                    issues,
                );
            } else {
                let transparent = if private {
                    "TransparentECPrivateKey"
                } else {
                    "TransparentECPublicKey"
                };
                let mut formats = vec![transparent, "PKCS#8"];
                if private
                    && REVIEW_EC_CURVES[..7]
                        .iter()
                        .any(|curve| parameters["curve"] == *curve)
                {
                    formats.push("ECPrivateKey");
                }
                demand_transfer_formats(
                    rows,
                    object,
                    Some(algorithm),
                    &parameters,
                    &formats,
                    transparent,
                    issues,
                );
            }
        }
    }
    let mut symmetric = reviewed_symmetric_variants();
    symmetric.extend([
        ("HMAC-SHA3-224", 256),
        ("HMAC-SHA3-256", 256),
        ("HMAC-SHA3-384", 256),
        ("HMAC-SHA3-512", 256),
        ("HMAC-SHA-256", 8),
        ("HMAC-SHA-256", 8192),
    ]);
    for (algorithm, bits) in symmetric {
        demand_transfer_formats(
            rows,
            "SymmetricKey",
            Some(algorithm),
            &serde_json::json!({"key_bits":bits}),
            &["Raw", "TransparentSymmetricKey"],
            "Raw",
            issues,
        );
    }
    for object in ["PrivateKey", "PublicKey"] {
        for bits in [2048, 3072] {
            let transparent = if object == "PrivateKey" {
                "TransparentDSAPrivateKey"
            } else {
                "TransparentDSAPublicKey"
            };
            demand_transfer_formats(
                rows,
                object,
                Some("DSA"),
                &serde_json::json!({"key_bits":bits}),
                &[transparent, "PKCS#8"],
                transparent,
                issues,
            );
        }
    }
    demand_transfer_formats(
        rows,
        "OpaqueObject",
        None,
        &serde_json::json!({}),
        &["Opaque"],
        "Opaque",
        issues,
    );
    demand_transfer_formats(
        rows,
        "SecretData",
        None,
        &serde_json::json!({}),
        &["Raw", "Opaque"],
        "Raw",
        issues,
    );
    for operation in ["Get", "Export"] {
        for selection in [None, Some("Raw")] {
            demand_review_case(
                rows,
                operation,
                "SplitKey",
                None,
                Some("Raw"),
                &serde_json::json!({"requested_key_format":selection}),
                Some("native"),
                issues,
            );
        }
        demand_review_case(
            rows,
            operation,
            "SecretData",
            None,
            Some("Opaque"),
            &serde_json::json!({"requested_key_format":null,"stored_format":"Opaque"}),
            None,
            issues,
        );
    }
    review_container_transfers(rows, issues);
    review_extension_transfers(rows, issues);
}

fn review_extension_transfers(rows: &[serde_json::Value], issues: &mut Vec<String>) {
    let mut variants = vec![
        (
            "CoverCrypt",
            serde_json::json!({"vendor_algorithm_value":"0x88800004"}),
        ),
        (
            "X25519MLKEM768",
            serde_json::json!({"vendor_algorithm_value":"0x88800006"}),
        ),
        (
            "X448MLKEM1024",
            serde_json::json!({"vendor_algorithm_value":"0x88800007"}),
        ),
    ];
    for curve in ["P256", "CURVE25519"] {
        variants.push((
            "ConfigurableKEM",
            serde_json::json!({"pre_quantum":curve,"post_quantum":null}),
        ));
    }
    for (algorithm, parameters) in variants {
        for object in ["PrivateKey", "PublicKey"] {
            for operation in ["Import", "Register", "Get", "Export"] {
                let format = match (algorithm, object) {
                    ("CoverCrypt", "PrivateKey") => "CoverCryptSecretKey",
                    ("CoverCrypt", _) => "CoverCryptPublicKey",
                    ("ConfigurableKEM", "PrivateKey") => "ConfigurableKEMSecretKey",
                    ("ConfigurableKEM", _) => "ConfigurableKEMPublicKey",
                    _ => "Raw",
                };
                demand_review_case(
                    rows,
                    operation,
                    object,
                    Some(algorithm),
                    Some(format),
                    &parameters,
                    None,
                    issues,
                );
            }
        }
    }
    for pre in [None, Some("P256"), Some("CURVE25519")] {
        for post in ["MLKEM512", "MLKEM768"] {
            for object in ["PrivateKey", "PublicKey"] {
                let format = if object == "PrivateKey" {
                    "ConfigurableKEMSecretKey"
                } else {
                    "ConfigurableKEMPublicKey"
                };
                for operation in ["Import", "Register", "Get", "Export"] {
                    let p = serde_json::json!({"opaque_fixture_configuration":{"pre_quantum":pre,"post_quantum":post}});
                    demand_review_case(
                        rows,
                        operation,
                        object,
                        Some("ConfigurableKEM"),
                        Some(format),
                        &p,
                        None,
                        issues,
                    );
                }
                for operation in ["Import", "Register"] {
                    let p = serde_json::json!({"opaque_fixture_configuration":{"pre_quantum":pre,"post_quantum":post},"wrapping_encoding":"TTLVEncoding","key_wrap_type":"AsRegistered"});
                    demand_review_case(
                        rows,
                        operation,
                        object,
                        Some("ConfigurableKEM"),
                        Some(format),
                        &p,
                        None,
                        issues,
                    );
                }
            }
        }
    }
}

fn review_container_transfers(rows: &[serde_json::Value], issues: &mut Vec<String>) {
    for operation in ["Import", "Register"] {
        for password in [false, true] {
            for chain in [false, true] {
                demand_review_case(
                    rows,
                    operation,
                    "PrivateKey",
                    Some("RSA"),
                    Some("PKCS#12"),
                    &serde_json::json!({"key_bits":2048,"password_link":password,"certificate_chain":chain}),
                    Some("DER/PFX"),
                    issues,
                );
            }
        }
    }
    for operation in ["Get", "Export"] {
        for (object, format) in [
            ("Certificate", "Cosmian-PKCS7"),
            ("Certificate", "PKCS#12"),
            ("PrivateKey", "PKCS#12"),
            ("Certificate", "Cosmian-Pkcs12Legacy"),
            ("PrivateKey", "Cosmian-Pkcs12Legacy"),
        ] {
            for password in [false, true] {
                for chain in [false, true] {
                    if format == "Cosmian-PKCS7" && password {
                        continue;
                    }
                    demand_review_case(
                        rows,
                        operation,
                        object,
                        Some("RSA"),
                        Some(format),
                        &serde_json::json!({"key_bits":2048,"password_link":password,"certificate_chain":chain}),
                        Some("DER"),
                        issues,
                    );
                }
            }
        }
    }
    for (algorithm, parameters) in reviewed_pair_variants() {
        if algorithm == "RSA" && parameters["key_bits"] == 2048 {
            continue;
        }
        for (operation, object, format) in [
            ("Import", "PrivateKey", "PKCS#12"),
            ("Register", "PrivateKey", "PKCS#12"),
            ("Get", "PrivateKey", "PKCS#12"),
            ("Export", "PrivateKey", "PKCS#12"),
            ("Get", "Certificate", "PKCS#12"),
            ("Export", "Certificate", "PKCS#12"),
            ("Get", "Certificate", "Cosmian-PKCS7"),
            ("Export", "Certificate", "Cosmian-PKCS7"),
            ("Get", "PrivateKey", "Cosmian-Pkcs12Legacy"),
            ("Export", "PrivateKey", "Cosmian-Pkcs12Legacy"),
        ] {
            demand_review_case(
                rows,
                operation,
                object,
                Some(algorithm),
                Some(format),
                &parameters,
                Some("DER/PFX"),
                issues,
            );
        }
    }
}

fn review_wrapping_cases(rows: &[serde_json::Value], issues: &mut Vec<String>) {
    let mut wrappers = Vec::new();
    for mode in ["NISTKeyWrap", "AESKeyWrapPadding", "GCM"] {
        for bits in [128, 192, 256] {
            wrappers.push(("AES", serde_json::json!({"mode":mode,"key_bits":bits})));
        }
    }
    for padding in ["OAEP", "None", "PKCS1v15"] {
        for bits in REVIEW_RSA_BITS {
            for hash in REVIEW_HASHES
                .into_iter()
                .map(Some)
                .filter(|hash| padding != "PKCS1v15" || *hash == Some("SHA-256"))
            {
                wrappers.push(("RSA",serde_json::json!({"padding":padding,"key_bits":bits,"hash":if padding=="PKCS1v15" {None} else {hash}})));
            }
        }
    }
    for curve in [
        "P192",
        "P224",
        "P256",
        "P384",
        "P521",
        "CURVE25519",
        "CURVEED25519",
    ] {
        wrappers.push(("EC", serde_json::json!({"curve":curve,"scheme":"ECIES"})));
    }
    for (algorithm, parameters) in wrappers {
        for encoding in ["NoEncoding", "TTLVEncoding"] {
            for object in ["SymmetricKey", "SecretData", "PrivateKey", "PublicKey"] {
                if object == "PublicKey"
                    && encoding == "NoEncoding"
                    && parameters["mode"] == "NISTKeyWrap"
                {
                    continue;
                }
                for operation in ["Import", "Register", "Get", "Export"] {
                    let asymmetric = matches!(object, "PrivateKey" | "PublicKey");
                    let format = if !asymmetric {
                        "Raw"
                    } else if encoding == "NoEncoding" {
                        "PKCS#8"
                    } else if object == "PrivateKey" {
                        "TransparentECPrivateKey"
                    } else {
                        "TransparentECPublicKey"
                    };
                    let mut p = parameters.clone();
                    p["wrapping_method"] = "Encrypt".into();
                    p["wrapping_encoding"] = encoding.into();
                    p["target_algorithm"] = if object == "SymmetricKey" {
                        serde_json::json!("AES")
                    } else if asymmetric {
                        serde_json::json!("Ed25519")
                    } else {
                        serde_json::Value::Null
                    };
                    p["wrapping_key_type"] = if algorithm == "AES" {
                        "SymmetricKey"
                    } else if matches!(operation, "Get" | "Export") {
                        "PublicKey"
                    } else {
                        "PrivateKey"
                    }
                    .into();
                    demand_review_case(
                        rows,
                        operation,
                        object,
                        Some(algorithm),
                        Some(format),
                        &p,
                        None,
                        issues,
                    );
                }
            }
        }
    }
}

fn review_material_citations(row: &serde_json::Value, issues: &mut Vec<String>) {
    let object = row["object_type"].as_str().unwrap_or("");
    let p = &row["parameters"];
    let algorithm = p["target_algorithm"]
        .as_str()
        .or_else(|| row["algorithm"].as_str())
        .unwrap_or("");
    let mut citations = Vec::new();
    if object == "SymmetricKey" {
        citations.push(("3.4", 18));
    }
    match (object, algorithm) {
        ("PrivateKey", "DSA") => citations.push(("3.5", 19)),
        ("PublicKey", "DSA") => citations.push(("3.6", 20)),
        ("PrivateKey", "RSA") => citations.push(("3.7", 21)),
        ("PublicKey", "RSA") => citations.push(("3.8", 22)),
        ("PrivateKey", "EC" | "ECDH" | "ECDSA" | "Ed25519" | "Ed448") => {
            citations.push(("3.11", 25));
        }
        ("PublicKey", "EC" | "ECDH" | "ECDSA" | "Ed25519" | "Ed448") => {
            citations.push(("3.12", 26));
        }
        _ => {}
    }
    match row["key_format"].as_str() {
        Some("TransparentSymmetricKey") => citations.push(("3.4", 18)),
        Some("TransparentDSAPrivateKey") => citations.push(("3.5", 19)),
        Some("TransparentDSAPublicKey") => citations.push(("3.6", 20)),
        Some("TransparentRSAPrivateKey") => citations.push(("3.7", 21)),
        Some("TransparentRSAPublicKey") => citations.push(("3.8", 22)),
        Some("TransparentECPrivateKey") => citations.push(("3.11", 25)),
        Some("TransparentECPublicKey") => citations.push(("3.12", 26)),
        _ => {}
    }
    if !p["wrapping_method"].is_null() {
        let unwrap = matches!(row["operation"].as_str(), Some("Import" | "Register"));
        match row["algorithm"].as_str() {
            Some("AES" | "ChaCha20Poly1305" | "FPE_FF1") => citations.push(("3.4", 18)),
            Some("RSA") => citations.push(if unwrap { ("3.7", 21) } else { ("3.8", 22) }),
            Some("EC") => citations.push(if unwrap { ("3.11", 25) } else { ("3.12", 26) }),
            _ => {}
        }
    }
    for (section, table) in citations {
        if !row["oasis_source"]
            .as_array()
            .expect("resolved citations")
            .iter()
            .any(|source| {
                source["section"] == section
                    && source["tables"]
                        .as_array()
                        .is_some_and(|tables| tables.contains(&serde_json::json!(table)))
            })
        {
            issues.push(format!(
                "material-citation: {} needs {section}/Table{table}",
                manifest_text(row, "case_id")
            ));
        }
    }
    if (p["mgf"] == "MGF1"
        || !p["mgf_hash"].is_null()
        || p["signature_scheme"] == "RSASSA-PSS"
        || (row["algorithm"] == "RSA" && matches!(p["padding"].as_str(), Some("OAEP" | "None"))))
        && !row["oasis_source"]
            .as_array()
            .expect("resolved citations")
            .iter()
            .any(|source| {
                source["section"] == "11.30"
                    && source["tables"].as_array().is_some_and(Vec::is_empty)
                    && source["table_title"] == "Mask Generator"
                    && source["numbering"] == "unnumbered"
            })
    {
        issues.push(format!(
            "MGF-citation: {} needs unnumbered 11.30 Mask Generator",
            manifest_text(row, "case_id")
        ));
    }
}

fn review_pqc_subject_containers(rows: &[serde_json::Value], issues: &mut Vec<String>) {
    let mut names = [
        "MLKEM_512",
        "MLKEM_768",
        "MLKEM_1024",
        "MLDSA_44",
        "MLDSA_65",
        "MLDSA_87",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    for hash in ["SHA2", "SHAKE"] {
        for size in [128, 192, 256] {
            for speed in ["s", "f"] {
                names.push(format!("SLHDSA_{hash}_{size}{speed}"));
            }
        }
    }
    for name in names {
        for (subject, encoding) in [("csr-pem", "PEM"), ("csr-pkcs10-der", "DER")] {
            demand_review_case(
                rows,
                "Certify",
                "CertificateRequest",
                Some(&name),
                Some("X.509"),
                &serde_json::json!({"subject_input":subject,"issuer":"RSA"}),
                Some(encoding),
                issues,
            );
        }
    }
}

fn review_additional_wrapping_paths(rows: &[serde_json::Value], issues: &mut Vec<String>) {
    let mut wrappers = Vec::new();
    for padding in ["OAEP", "None", "PKCS1v15"] {
        wrappers.push((
            "RSA",
            serde_json::json!({"key_bits":2048,"padding":padding,"hash":"SHA-256"}),
        ));
    }
    for curve in [
        "P192",
        "P224",
        "P256",
        "P384",
        "P521",
        "CURVE25519",
        "CURVEED25519",
    ] {
        wrappers.push(("EC", serde_json::json!({"curve":curve,"scheme":"ECIES"})));
    }
    for (algorithm, parameters) in wrappers {
        for operation in ["Import", "Register", "Get", "Export"] {
            for encoding in ["NoEncoding", "TTLVEncoding"] {
                let mut p = parameters.clone();
                p["wrapping_key_type"] = "Certificate".into();
                p["wrapping_encoding"] = encoding.into();
                demand_review_case(
                    rows,
                    operation,
                    "SymmetricKey",
                    Some(algorithm),
                    Some("Raw"),
                    &p,
                    None,
                    issues,
                );
            }
        }
    }
    let mut gaps = vec![
        ("AES", serde_json::json!({"mode":"XTS","key_bits":256})),
        ("AES", serde_json::json!({"mode":"XTS","key_bits":512})),
        ("AES", serde_json::json!({"mode":"GCMSIV","key_bits":128})),
        ("AES", serde_json::json!({"mode":"GCMSIV","key_bits":256})),
        ("ChaCha20Poly1305", serde_json::json!({"key_bits":256})),
        ("FPE_FF1", serde_json::json!({"key_bits":256})),
    ];
    for (algorithm, parameters) in &mut gaps {
        for operation in ["Import", "Export"] {
            for encoding in ["NoEncoding", "TTLVEncoding"] {
                let mut p = parameters.clone();
                p["wrapping_encoding"] = encoding.into();
                p["wrapping_method"] = "Encrypt".into();
                demand_review_case(
                    rows,
                    operation,
                    "SymmetricKey",
                    Some(*algorithm),
                    Some("Raw"),
                    &p,
                    None,
                    issues,
                );
            }
        }
    }
}

// The catalog preserves exact per-case citations while storing long source
// paths only once. Resolve typed references before applying the row contract.
fn resolve_capability_sources(manifest: &serde_json::Value) -> Vec<serde_json::Value> {
    assert_eq!(manifest["schema_version"], 2);
    let target = &manifest["target"];
    assert_eq!(manifest_text(target, "id"), "cosmian-5.28.0");
    let catalog = manifest["source_catalog"]
        .as_object()
        .expect("source catalog must be an object");
    let rows = manifest["cases"]
        .as_array()
        .expect("manifest cases must be an array");
    let mut used = BTreeSet::new();
    let mut resolved = Vec::with_capacity(rows.len());
    for row in rows {
        let mut row = row.clone();
        assert_eq!(
            manifest_text(&row["cosmian_source"], "target"),
            "cosmian-5.28.0"
        );
        for (field, kind) in [
            ("oasis_source", "oasis"),
            ("documentation", "documentation"),
            ("code", "code"),
        ] {
            let references = if field == "oasis_source" {
                &mut row[field]
            } else {
                &mut row["cosmian_source"][field]
            };
            let ids = references
                .as_array()
                .expect("source references must be an array");
            let citations: Vec<_> = ids
                .iter()
                .map(|id| {
                    let id = id.as_str().expect("source IDs must be strings");
                    let citation = catalog.get(id).expect("unresolved source reference");
                    assert_eq!(
                        manifest_text(citation, "kind"),
                        kind,
                        "wrong source reference kind: {id}"
                    );
                    used.insert(id.to_owned());
                    citation.clone()
                })
                .collect();
            *references = serde_json::Value::Array(citations);
        }
        for field in ["tag", "commit", "image_digest"] {
            row["cosmian_source"][field] = target[field].clone();
        }
        resolved.push(row);
    }
    assert_eq!(used.len(), catalog.len(), "unreferenced catalog entries");
    resolved
}

// This checks the reviewed finite source inventory, not executable-case parity
// (Task 7) or observed server support. Integer-valued domains use the explicit
// branch/boundary representatives described in research.md.
// This fixed inventory checklist keeps every required finite dimension visible.
#[allow(clippy::too_many_lines)]
fn validate_capability_dimensions(rows: &[serde_json::Value]) {
    let objects = [
        "Certificate",
        "SymmetricKey",
        "PublicKey",
        "PrivateKey",
        "SplitKey",
        "SecretData",
        "OpaqueObject",
    ];
    for operation in [
        "Add Attribute",
        "Delete Attribute",
        "Get Attributes",
        "Get Attribute List",
        "Modify Attribute",
        "Set Attribute",
        "Check",
        "Locate",
        "Revoke",
        "Destroy",
    ] {
        for object in objects {
            require_inventory_case(rows, operation, Some(object), None, None);
        }
    }
    for object in objects
        .into_iter()
        .filter(|object| *object != "OpaqueObject")
    {
        require_inventory_case(rows, "Activate", Some(object), None, None);
    }
    for operation in ["Import", "Register", "Get", "Export"] {
        for (object, algorithm, format) in [
            ("Certificate", "RSA", "X.509"),
            ("SymmetricKey", "AES", "Raw"),
            ("SymmetricKey", "AES", "TransparentSymmetricKey"),
            ("PrivateKey", "RSA", "PKCS#1"),
            ("PrivateKey", "RSA", "PKCS#8"),
            ("PrivateKey", "RSA", "TransparentRSAPrivateKey"),
            ("PublicKey", "RSA", "PKCS#1"),
            ("PublicKey", "RSA", "PKCS#8"),
            ("PublicKey", "RSA", "TransparentRSAPublicKey"),
            ("PrivateKey", "EC", "TransparentECPrivateKey"),
            ("PrivateKey", "EC", "PKCS#8"),
            ("PrivateKey", "EC", "ECPrivateKey"),
            ("PublicKey", "EC", "TransparentECPublicKey"),
            ("PublicKey", "EC", "PKCS#8"),
            ("PrivateKey", "DSA", "TransparentDSAPrivateKey"),
            ("PublicKey", "DSA", "TransparentDSAPublicKey"),
        ] {
            require_inventory_case(rows, operation, Some(object), Some(algorithm), Some(format));
        }
        for object in ["OpaqueObject", "SecretData"] {
            require_inventory_case(rows, operation, Some(object), None, None);
        }
    }
    let hashes = [
        "SHA-1", "SHA-224", "SHA-256", "SHA-384", "SHA-512", "SHA3-224", "SHA3-256", "SHA3-384",
        "SHA3-512",
    ];
    for hash in hashes {
        for operation in ["MAC", "MAC Verify"] {
            let algorithm = format!("HMAC-{hash}");
            require_inventory_case(
                rows,
                operation,
                Some("SymmetricKey"),
                Some(&algorithm),
                None,
            );
        }
        if hash != "SHA-1" {
            require_inventory_case(rows, "Hash", None, Some(hash), None);
        }
    }
    for method in ["PBKDF2", "HKDF"] {
        for hash in hashes.into_iter().take(5) {
            for input in ["SymmetricKey", "SecretData"] {
                for output in ["SymmetricKey", "SecretData"] {
                    assert!(
                        rows.iter().any(|row| {
                            row["operation"] == "Derive Key"
                                && row["object_type"] == input
                                && row["parameters"]["method"] == method
                                && row["parameters"]["hash"] == hash
                                && row["parameters"]["output_object_type"] == output
                        }),
                        "missing derivation intersection: {method}/{hash}/{input}/{output}"
                    );
                }
            }
        }
    }
    for password in [false, true] {
        for chain in [false, true] {
            assert!(
                rows.iter().any(|row| {
                    row["operation"] == "Import"
                        && row["object_type"] == "PrivateKey"
                        && row["key_format"] == "PKCS#12"
                        && row["parameters"]["password_link"] == password
                        && row["parameters"]["certificate_chain"] == chain
                        && row["notes"]
                            .as_str()
                            .is_some_and(|notes| notes.contains("formats.md omits"))
                }),
                "missing PFX import/omission branch: password={password}, chain={chain}"
            );
        }
    }
    for operation in ["Encrypt", "Decrypt"] {
        for mode in [
            "ECB",
            "CBC",
            "GCM",
            "AEAD",
            "XTS",
            "AESKeyWrapPadding",
            "NISTKeyWrap",
            "GCMSIV",
        ] {
            assert!(
                rows.iter().any(|row| row["operation"] == operation
                    && row["algorithm"] == "AES"
                    && row["parameters"]["mode"] == mode),
                "missing AES branch: {operation}/{mode}"
            );
        }
        for algorithm in [
            "ChaCha20",
            "ChaCha20Poly1305",
            "FPE_FF1",
            "CoverCrypt",
            "ConfigurableKEM",
            "X25519MLKEM768",
            "X448MLKEM1024",
        ] {
            require_inventory_case(rows, operation, None, Some(algorithm), None);
        }
    }
    for operation in ["Ping", "Adjust Attribute", "Re-certify"] {
        require_inventory_case(rows, operation, None, None, None);
    }
}

fn require_inventory_case(
    rows: &[serde_json::Value],
    operation: &str,
    object: Option<&str>,
    algorithm: Option<&str>,
    format: Option<&str>,
) {
    assert!(
        rows.iter().any(|row| {
            row["operation"] == operation
                && object.is_none_or(|value| row["object_type"] == value)
                && algorithm.is_none_or(|value| row["algorithm"] == value)
                && format.is_none_or(|value| row["key_format"] == value)
        }),
        "missing source inventory intersection: {operation}/{object:?}/{algorithm:?}/{format:?}"
    );
}

fn validate_capability_row(row: &serde_json::Value) {
    validate_required_capability_fields(row);
    validate_capability_execution_status(row);
    validate_capability_dimensions_for_row(row);
    validate_observed_result(row);
    validate_oasis_citations(row);
    validate_cosmian_citations(row);
    validate_wire_enum_values(row);
    validate_special_capability_cases(row);
}

fn validate_required_capability_fields(row: &serde_json::Value) {
    let fields = [
        "case_id",
        "operation",
        "object_type",
        "algorithm",
        "parameters",
        "key_format",
        "encoding",
        "setup",
        "oasis_source",
        "cosmian_source",
        "expected_outcome",
        "test_family",
        "observed_result",
    ];
    let object = row.as_object().expect("each matrix row must be an object");
    for field in fields {
        assert!(
            object.contains_key(field),
            "matrix row missing required field: {field}"
        );
    }
    for field in ["case_id", "operation", "expected_outcome", "test_family"] {
        manifest_text(row, field);
    }
    let requirements = row["requirements"]
        .as_array()
        .expect("every case must link to functional requirements");
    assert!(
        requirements
            .iter()
            .any(|value| value == "KMIPKIT-0015-FR-016")
    );
    assert_eq!(manifest_text(row, "encoding"), "binary-TTLV");
}

fn validate_capability_execution_status(row: &serde_json::Value) {
    // Normative exclusions are the only rows permitted to remain not_run;
    // source existence alone never counts as a live observation.
    if row["expected_outcome"] == "standards_invalid_exclusion" {
        validate_non_executable_exclusion(row);
    } else {
        assert_eq!(
            manifest_text(row, "execution_status"),
            "planned_for_tasks_4_6"
        );
        assert_ne!(
            manifest_text(&row["observed_result"], "classification"),
            "not_run",
            "executable case cannot remain not_run: {}",
            manifest_text(row, "case_id")
        );
    }
}

fn validate_non_executable_exclusion(row: &serde_json::Value) {
    assert_eq!(
        manifest_text(row, "expected_outcome"),
        "standards_invalid_exclusion"
    );
    assert_eq!(
        manifest_text(row, "capability_basis"),
        "normative_exclusion"
    );
    assert_eq!(
        manifest_text(row, "execution_status"),
        "non_executable_normative_exclusion"
    );
    assert_eq!(row["exclusion"]["request_may_be_sent"], false);
    manifest_text(&row["exclusion"], "reason");
    assert_eq!(row["setup"].as_array().expect("exclusion setup").len(), 0);
    assert_eq!(row["observed_result"]["classification"], "not_run");
    let test_family = manifest_text(row, "test_family");
    match manifest_text(row, "operation") {
        "Certify" => {
            assert!(csr_subject_cannot_sign(row));
            assert_eq!(
                test_family,
                "cosmian_kms::cosmian_capability_manifest_is_complete"
            );
            assert_eq!(
                row["exclusion"]["normative_sources"],
                serde_json::json!([{
                    "document": "RFC 2986",
                    "sections": ["3", "4.2"],
                    "url": "https://www.rfc-editor.org/rfc/rfc2986.html",
                    "referenced_from": "OASIS KMIP Specification v2.1 section 1.3 [PKCS#10]/[RFC2986]; CSR structure section 2.2/Table 3; request types section 11.8/Table 439"
                }])
            );
            assert!(
                row["oasis_source"]
                    .as_array()
                    .expect("exclusion citations")
                    .iter()
                    .any(|citation| citation["section"] == "2.2"
                        && citation["tables"] == serde_json::json!([3]))
            );
        }
        "RNG Retrieve" | "RNG Seed" => {
            assert_eq!(
                test_family,
                "cosmian_kms::random::cosmian_random_case_ids_cover_manifest_and_exclusions"
            );
            assert_eq!(
                row["parameters"]["rng_parameters"]["rng_algorithm"],
                "Unspecified"
            );
            let (section, table, referenced_from, length_field) =
                if row["operation"] == "RNG Retrieve" {
                    (
                        "6.1.49",
                        316,
                        "Operation request payload schema; only Data Length is listed.",
                        "data_length_bytes",
                    )
                } else {
                    (
                        "6.1.50",
                        319,
                        "Operation request payload schema; only Data is listed.",
                        "seed_length_bytes",
                    )
                };
            assert!(row["parameters"][length_field].as_u64().is_some());
            assert!(
                row["oasis_source"]
                    .as_array()
                    .expect("RNG exclusion citations")
                    .iter()
                    .any(|citation| citation["section"] == section
                        && citation["tables"]
                            .as_array()
                            .is_some_and(|tables| tables.contains(&serde_json::json!(table))))
            );
            assert!(
                row["oasis_source"]
                    .as_array()
                    .expect("RNG structure citations")
                    .iter()
                    .any(|citation| citation["section"] == "7.36"
                        && citation["tables"] == serde_json::json!([388]))
            );
            assert_eq!(
                row["exclusion"]["normative_sources"],
                serde_json::json!([{
                    "document": "OASIS KMIP Specification v2.1",
                    "sections": [section],
                    "tables": [table],
                    "referenced_from": referenced_from
                }])
            );
        }
        operation => panic!("unsupported normative exclusion operation: {operation}"),
    }
}

#[test]
fn rng_normative_exclusions_are_validated_without_sending() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/integration/cosmian/capability-matrix.json"
    ))
    .expect("capability manifest must be valid JSON");
    let rows = resolve_capability_sources(&manifest);
    let rng_exclusions = rows
        .iter()
        .filter(|row| {
            row["test_family"]
                == "cosmian_kms::random::cosmian_random_case_ids_cover_manifest_and_exclusions"
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rng_exclusions.len(),
        6,
        "all invalid RNG parameter rows are explicit"
    );
    for row in rng_exclusions {
        validate_non_executable_exclusion(row);
    }
}

fn validate_capability_dimensions_for_row(row: &serde_json::Value) {
    manifest_text(row, "expected_semantics");
    for field in ["object_type", "algorithm", "key_format", "encoding"] {
        assert!(
            row[field].is_null()
                || row[field]
                    .as_str()
                    .is_some_and(|value| !value.trim().is_empty()),
            "{field} must be a nonempty string or null for an inapplicable dimension"
        );
    }
    assert!(
        row["parameters"].is_null() || row["parameters"].is_object(),
        "parameters must be an object or null when inapplicable"
    );
    let setup = row["setup"]
        .as_array()
        .expect("setup must be an array (empty for no dependencies)");
    assert!(
        setup
            .iter()
            .all(|value| value.as_str().is_some_and(|text| !text.trim().is_empty())),
        "setup dependencies must be nonempty strings"
    );
    assert!(
        matches!(
            manifest_text(row, "expected_outcome"),
            "typed_success"
                | "known_response_shape_failure"
                | "documented_but_source_gap"
                | "source_documentation_discrepancy"
                | "unsupported_probe"
                | "client_blocked"
                | "setup_blocked"
                | "standards_invalid_exclusion"
        ),
        "invalid expected_outcome"
    );
}

#[test]
fn observed_result_accepts_setup_rejection_before_target_send() {
    let row = serde_json::json!({
        "observed_result": {
            "classification": "setup_rejected",
            "target_request_sent": false,
            "setup_request_sent": true,
            "setup_response_received": true,
            "reason": "A prerequisite request was rejected; target operation was not sent."
        }
    });
    validate_observed_result(&row);
}

#[test]
#[should_panic(expected = "setup_rejected observations must show target_request_sent=false")]
fn observed_result_rejects_setup_rejection_after_target_send() {
    let row = serde_json::json!({
        "observed_result": {
            "classification": "setup_rejected",
            "target_request_sent": true,
            "setup_request_sent": true,
            "setup_response_received": true,
            "reason": "A prerequisite request was rejected."
        }
    });
    validate_observed_result(&row);
}

fn validate_observed_result(row: &serde_json::Value) {
    // Observations remain independent of the expected outcome and never use
    // null to conflate not-run with a successful or rejected exchange.
    let observed = &row["observed_result"];
    assert!(observed.is_object(), "observed_result must be an object");
    let classification = manifest_text(observed, "classification");
    assert!(
        matches!(
            classification,
            "not_run"
                | "typed_success"
                | "rejected"
                | "setup_rejected"
                | "fixture_unavailable"
                | "response_validation_failed"
                | "semantic_mismatch"
                | "malformed_response"
                | "unsupported"
                | "client_blocked"
                | "setup_blocked"
        ),
        "invalid observed_result classification"
    );
    if classification == "setup_rejected" {
        assert_eq!(
            observed["target_request_sent"], false,
            "setup_rejected observations must show target_request_sent=false"
        );
        assert_eq!(
            observed["setup_request_sent"], true,
            "setup_rejected observations must show setup_request_sent=true"
        );
        assert_eq!(
            observed["setup_response_received"], true,
            "setup_rejected observations must show setup_response_received=true"
        );
    }
}

fn validate_oasis_citations(row: &serde_json::Value) {
    let oasis = row["oasis_source"]
        .as_array()
        .expect("oasis_source must be an array of exact citations");
    assert!(
        !oasis.is_empty(),
        "oasis_source requires operation/data citations"
    );
    for citation in oasis {
        assert_eq!(
            manifest_text(citation, "document"),
            "OASIS KMIP Specification v2.1"
        );
        manifest_text(citation, "section");
        let tables = citation["tables"]
            .as_array()
            .expect("OASIS citation tables must be an array");
        let unnumbered_mask_generator = citation["section"] == "11.30"
            && tables.is_empty()
            && citation["table_title"] == "Mask Generator"
            && citation["numbering"] == "unnumbered";
        assert!(
            unnumbered_mask_generator
                || (!tables.is_empty()
                    && tables
                        .iter()
                        .all(|table| table.as_u64().is_some_and(|number| number > 0))),
            "OASIS citations require exact table numbers, or the explicitly unnumbered 11.30 Mask Generator table"
        );
    }
}

fn validate_cosmian_citations(row: &serde_json::Value) {
    let cosmian = &row["cosmian_source"];
    assert_eq!(manifest_text(cosmian, "tag"), "5.28.0");
    assert_eq!(
        manifest_text(cosmian, "commit"),
        "4ab3ee65829a903abb71f1298f400657fcee7ec9"
    );
    assert_eq!(
        manifest_text(cosmian, "image_digest"),
        "sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e"
    );
    for field in ["documentation", "code"] {
        let citations = cosmian[field]
            .as_array()
            .expect("Cosmian documentation/code citations must be arrays");
        assert!(
            !citations.is_empty(),
            "Cosmian {field} citations cannot be empty"
        );
        for citation in citations {
            manifest_text(citation, "path");
            manifest_text(citation, if field == "code" { "symbol" } else { "section" });
            if field == "code" {
                assert!(citation["line"].as_u64().is_some_and(|line| line > 0));
            }
        }
    }
}

fn validate_wire_enum_values(row: &serde_json::Value) {
    let enums = row["wire_enum_values"]
        .as_object()
        .expect("numeric wire identities are required");
    for (dimension, enumeration) in enums {
        let classification = manifest_text(enumeration, "classification");
        if classification == "not_sent_asn1_oid_only" {
            assert!(enumeration["value"].is_null());
            assert_eq!(manifest_text(row, "operation"), "Certify");
            assert_eq!(manifest_text(row, "object_type"), "CertificateRequest");
            continue;
        }
        assert!(matches!(
            classification,
            "standard" | "vendor_extension" | "source_only_numeric_probe"
        ));
        let text = manifest_text(enumeration, "value");
        let value = u32::from_str_radix(text.trim_start_matches("0x"), 16)
            .expect("enumeration must record a numeric hexadecimal identity");
        if classification == "vendor_extension" {
            assert!((0x8000_0000..=0x8FFF_FFFF).contains(&value));
        } else if classification == "source_only_numeric_probe" {
            assert_eq!(
                manifest_text(row, "expected_outcome"),
                "source_documentation_discrepancy"
            );
            match dimension.as_str() {
                "cryptographic_algorithm" => assert!((0x39..=0x4A).contains(&value)),
                "nested_post_quantum_algorithm" => {
                    assert_eq!(manifest_text(row, "operation"), "Create Key Pair");
                    assert_eq!(manifest_text(row, "algorithm"), "ConfigurableKEM");
                    assert!((0x39..=0x3A).contains(&value));
                }
                _ => panic!("unexpected source-only numeric enum dimension {dimension}"),
            }
        } else if dimension == "cryptographic_algorithm" {
            assert!(
                (1..=0x38).contains(&value),
                "reserved KMIP 2.1 algorithm in a wire row"
            );
        }
    }
}

fn validate_special_capability_cases(row: &serde_json::Value) {
    if row["operation"] == "Hash" && row["algorithm"] == "SHA-224" {
        assert_eq!(
            manifest_text(row, "expected_outcome"),
            "documented_but_source_gap"
        );
    }
    if row["fixture"]["encoding"] == "PEM" && row["operation"] != "Certify" {
        assert!(manifest_text(&row["fixture"], "normalization").contains("PEM-to-DER"));
        assert_eq!(
            manifest_text(&row["fixture"], "wire_representation"),
            "DER byte string"
        );
    }
    assert_ne!(
        row["key_format"], "PEM",
        "PEM is not a KMIP Key Format Type"
    );
    if row["operation"] == "Derive Key" && row["parameters"]["method"] == "PBKDF2" {
        assert!(
            row["parameters"]["iterations"]
                .as_u64()
                .is_some_and(|count| count > 0)
        );
        assert_eq!(row["parameters"]["salt"], "present");
        if row["object_type"] == "SecretData" {
            assert!(row["parameters"]["derivation_data"].is_null());
        } else {
            manifest_text(&row["parameters"], "derivation_data");
        }
    }
}

fn manifest_text<'a>(value: &'a serde_json::Value, field: &str) -> &'a str {
    value[field]
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| panic!("manifest field {field} must be a nonempty string"))
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.16, Tables 211–213; Protocol Version: §9.16, Table 421.
fn cosmian_kms_accepts_kmip_2_1_discover_versions_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let result = execute_one(
        &mut client,
        ClientRequest::discover_versions(),
        "Discover Versions",
        |response| {
            let versions = response
                .discover_versions()
                .and_then(|result| result.supported_versions())
                .ok_or("successful Discover Versions response has no version list")?;
            assert!(
                versions.contains(&kmipkit_protocol::ProtocolVersion::from_raw(2, 1)),
                "Cosmian reports the KMIP 2.1 version offered by KMIPKit"
            );
            Ok(())
        },
    );
    if result.is_ok() {
        println!(
            "case_id=cosmian-528-discover-versions classification=typed_success expected_outcome=typed_success observed=typed_success request_sent=true response_received=true interoperability=true"
        );
    }
    result
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.8, Tables 186–188.
fn cosmian_kms_accepts_kmip_2_1_create_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    assert!(!identifier.is_empty(), "Create returns a Unique Identifier");
    Ok(())
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.1, Tables 163–166.
fn cosmian_kms_accepts_kmip_2_1_activate_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    activate_key(&mut client, &identifier)
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.9, Tables 189–192.
fn cosmian_kms_accepts_kmip_2_1_create_key_pair_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    execute_one(
        &mut client,
        ClientRequest::CreateKeyPair(rsa_key_pair_request()?),
        "Create Key Pair",
        |response| {
            let key_pair = response
                .create_key_pair()
                .ok_or("KMIPKit did not return a typed Create Key Pair result")?;
            let private_identifier = key_pair
                .private_key_unique_identifier()
                .ok_or("successful Create Key Pair response has no private-key identifier")?;
            let public_identifier = key_pair
                .public_key_unique_identifier()
                .ok_or("successful Create Key Pair response has no public-key identifier")?;
            assert!(
                !matches!(private_identifier, UniqueIdentifier::TextString(value) if value.is_empty()),
                "private-key identifier is nonempty"
            );
            assert!(
                !matches!(public_identifier, UniqueIdentifier::TextString(value) if value.is_empty()),
                "public-key identifier is nonempty"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.10, Tables 193–195; Split Key Method: §11.36, Table 470.
fn cosmian_kms_accepts_kmip_2_1_create_split_key_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let source_identifier = create_symmetric_key(&mut client)?;
    execute_one(
        &mut client,
        ClientRequest::CreateSplitKey(
            CreateSplitKeyRequest::new(
                ObjectType::from_raw(SYMMETRIC_KEY),
                2,
                2,
                SplitKeyMethod::XOR,
                AttributeSet::new(),
            )
            .with_unique_identifier(UniqueIdentifier::TextString(source_identifier)),
        ),
        "Create Split Key",
        |response| {
            let identifiers = response
                .create_split_key()
                .ok_or("KMIPKit did not return a typed Create Split Key result")?
                .unique_identifiers();
            assert_eq!(
                identifiers.len(),
                2,
                "two-of-two split returns two identifiers"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.2, Tables 167–169.
fn cosmian_kms_accepts_kmip_2_1_add_attribute_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    add_comment(
        &mut client,
        &identifier,
        "KMIPKit integration Add Attribute",
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.13, Tables 202–204.
fn cosmian_kms_accepts_kmip_2_1_delete_attribute_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    add_comment(
        &mut client,
        &identifier,
        "KMIPKit integration Delete Attribute",
    )?;
    execute_one(
        &mut client,
        ClientRequest::delete_attribute(DeleteAttributeRequest::new(
            Some(identifier),
            None,
            Some(AttributeReference::tag(COMMENT)),
        )),
        "Delete Attribute",
        |response| {
            let returned_identifier = response
                .delete_attribute()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Delete Attribute response has no Unique Identifier")?;
            assert_ne!(returned_identifier, "");
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.20, Tables 223–225.
fn cosmian_kms_accepts_kmip_2_1_get_attributes_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    add_comment(
        &mut client,
        &identifier,
        "KMIPKit integration Get Attributes",
    )?;
    execute_one(
        &mut client,
        ClientRequest::get_attributes(GetAttributesRequest::try_new(
            Some(identifier),
            [AttributeReference::tag(COMMENT)],
        )?),
        "Get Attributes",
        |response| {
            let attributes = response
                .get_attributes()
                .and_then(|result| result.attributes())
                .ok_or("successful Get Attributes response has no Attributes set")?;
            assert!(
                attributes
                    .as_items()
                    .iter()
                    .any(|attribute| attribute.tag().raw() == COMMENT),
                "Get Attributes returns the requested Comment"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.21, Tables 226–228.
fn cosmian_kms_accepts_kmip_2_1_get_attribute_list_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    execute_one(
        &mut client,
        ClientRequest::get_attribute_list(GetAttributeListRequest::new(Some(identifier))),
        "Get Attribute List",
        |response| {
            let attribute_references = response
                .get_attribute_list()
                .and_then(|result| result.attribute_references())
                .ok_or("successful Get Attribute List response has no Attribute References")?;
            assert!(
                !attribute_references.is_empty(),
                "Get Attribute List returns at least one Attribute Reference"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.34, Tables 265–267.
fn cosmian_kms_accepts_kmip_2_1_modify_attribute_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    add_comment(
        &mut client,
        &identifier,
        "KMIPKit integration Modify Attribute",
    )?;
    execute_one(
        &mut client,
        ClientRequest::modify_attribute(ModifyAttributeRequest::new(
            Some(identifier),
            Some(CurrentAttribute::new(comment_item(
                "KMIPKit integration Modify Attribute",
            )?)),
            NewAttribute::new(comment_item("KMIPKit integration Modified Attribute")?),
        )),
        "Modify Attribute",
        |response| {
            let returned_identifier = response
                .modify_attribute()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Modify Attribute response has no Unique Identifier")?;
            assert_ne!(returned_identifier, "");
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.51, Tables 322–324.
fn cosmian_kms_accepts_kmip_2_1_set_attribute_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    add_comment(
        &mut client,
        &identifier,
        "KMIPKit integration Set Attribute",
    )?;
    execute_one(
        &mut client,
        ClientRequest::set_attribute(SetAttributeRequest::new(
            Some(identifier),
            NewAttribute::new(comment_item(
                "KMIPKit integration Set Attribute replacement",
            )?),
        )),
        "Set Attribute",
        |response| {
            let returned_identifier = response
                .set_attribute()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Set Attribute response has no Unique Identifier")?;
            assert_ne!(returned_identifier, "");
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.40, Tables 282–284; Protection Storage Masks is required by Table 283.
fn cosmian_kms_query_response_without_storage_masks_is_reported_over_mutual_tls()
-> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let result = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::query(
            QueryRequest::new([QueryFunction::OPERATIONS]),
        ))),
        &CodecLimits::defaults(),
    );
    match result {
        Ok(response) => {
            let item = response
                .get(0)
                .ok_or("Cosmian response has no Query result")?;
            let outcome = item.outcome();
            let response_view = outcome.response();
            let query = response_view
                .query()
                .ok_or("KMIPKit did not return a typed Query result")?;
            assert!(query.operations().next().is_some());
            Ok(())
        }
        Err(error) => {
            assert_eq!(
                error.category(),
                kmipkit_client::ClientErrorCategory::Protocol
            );
            let response = error
                .response_ttlv()
                .ok_or("Query parse failure retains the complete response TTLV")?;
            let observation = response.with_ttlv(|root| {
                let batch = root
                    .children()
                    .iter()
                    .find(|item| item.tag().raw() == BATCH_ITEM)?;
                batch.with_value(|value| {
                    let ValueView::Structure(batch_item) = value else {
                        return None;
                    };
                    let status = batch_item.children().iter().find_map(|field| {
                        (field.tag().raw() == RESULT_STATUS)
                            .then(|| {
                                field.with_value(|value| match value {
                                    ValueView::Enumeration(value) => Some(*value),
                                    _ => None,
                                })
                            })
                            .flatten()
                    })?;
                    let payload = batch_item
                        .children()
                        .iter()
                        .find(|field| field.tag().raw() == RESPONSE_PAYLOAD)?;
                    payload.with_value(|value| {
                        let ValueView::Structure(payload) = value else {
                            return None;
                        };
                        let operation_count = payload
                            .children()
                            .iter()
                            .filter(|field| field.tag().raw() == OPERATION)
                            .count();
                        let has_protection_storage_masks = payload
                            .children()
                            .iter()
                            .any(|field| field.tag().raw() == QUERY_PROTECTION_STORAGE_MASKS);
                        Some((status, operation_count, has_protection_storage_masks))
                    })
                })
            });
            let (status, operation_count, has_masks) =
                observation.ok_or("Cosmian Query response has no recognizable result payload")?;
            assert_eq!(status, SUCCESS, "Cosmian reports Query as successful");
            assert!(operation_count > 0, "Cosmian returns supported Operations");
            assert!(
                !has_masks,
                "the observed structured response omits Protection Storage Masks"
            );
            println!(
                "Cosmian Query interoperability limitation: KMIP result is Success with {operation_count} Operations, but the response omits required Protection Storage Masks and KMIPKit rejects the structured payload."
            );
            println!(
                "case_id=cosmian-528-query classification=response_validation_failed expected_outcome=known_response_shape_failure observed=response_validation_failed request_sent=true response_received=true interoperability=false result_status=0 missing_field=ProtectionStorageMasks"
            );
            Ok(())
        }
    }
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.36, Tables 271–272.
fn cosmian_kms_rejects_unsupported_kmip_2_1_ping_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::ping()));
    match client.execute(batch, &CodecLimits::defaults()) {
        Ok(response) => {
            let item = response
                .get(0)
                .ok_or("Cosmian response has no Ping result")?;
            let outcome = item.outcome();
            let view = outcome.response();
            let ping = view
                .ping()
                .ok_or("KMIPKit did not return a typed Ping result")?;
            assert_eq!(ping.result().status().raw(), SUCCESS);
            Ok(())
        }
        Err(error) => {
            assert_eq!(
                error.category(),
                kmipkit_client::ClientErrorCategory::Protocol
            );
            let (operation, status, reason) = response_result_fields(&error)
                .ok_or("Cosmian Ping rejection retains a result in its error response")?;
            assert_eq!(operation, None, "the error response omits Operation");
            assert_eq!(status, Some(1), "Cosmian reports Operation Failed");
            assert_eq!(reason, Some(4), "Cosmian reports Invalid Message for Ping");
            println!(
                "Cosmian rejects KMIP Ping with Operation Failed / Invalid Message; the error response omits the Batch Item Operation field."
            );
            println!(
                "case_id=cosmian-528-ping classification=malformed_response expected_outcome=unsupported_probe observed=malformed_response request_sent=true response_received=true interoperability=false result_status=1 result_reason=4 missing_field=Operation"
            );
            Ok(())
        }
    }
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.17, Tables 214–216.
fn cosmian_kms_accepts_kmip_2_1_encrypt_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    activate_key(&mut client, &identifier)?;
    let request = EncryptRequest::new(
        Some(UniqueIdentifier::TextString(identifier)),
        Some(OperationData::ByteString(
            kmipkit_protocol::SecretBytes::new(b"KMIPKit Cosmian integration plaintext".to_vec()),
        )),
    )
    .with_cryptographic_parameters(cipher_parameters()?)
    .with_iv_counter_nonce(kmipkit_protocol::SecretBytes::new(vec![0x31; 16]));
    execute_one(
        &mut client,
        ClientRequest::encrypt(request),
        "Encrypt",
        |response| {
            let encrypted = response
                .encrypt()
                .ok_or("KMIPKit did not return a typed Encrypt result")?;
            assert!(
                encrypted.data().is_some(),
                "successful Encrypt response returns ciphertext Data"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.11, Tables 196–198.
fn cosmian_kms_accepts_kmip_2_1_decrypt_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    activate_key(&mut client, &identifier)?;
    let plaintext = b"KMIPKit Cosmian decrypt integration".to_vec();
    let encrypted = execute_one(
        &mut client,
        ClientRequest::encrypt(
            EncryptRequest::new(
                Some(UniqueIdentifier::TextString(identifier.clone())),
                Some(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(plaintext.clone()),
                )),
            )
            .with_cryptographic_parameters(cipher_parameters()?)
            .with_iv_counter_nonce(kmipkit_protocol::SecretBytes::new(vec![0x32; 16])),
        ),
        "Encrypt setup for Decrypt",
        |response| {
            let ciphertext = response
                .encrypt()
                .and_then(|result| result.data())
                .ok_or("successful Encrypt setup has no ciphertext Data")?;
            Ok(ciphertext.with_bytes(|value| SecretBytes::new(value.to_vec())))
        },
    )?;
    let decrypted = execute_one(
        &mut client,
        ClientRequest::decrypt(
            DecryptRequest::new(
                Some(UniqueIdentifier::TextString(identifier)),
                Some(OperationData::ByteString(encrypted)),
            )
            .with_cryptographic_parameters(cipher_parameters()?)
            .with_iv_counter_nonce(kmipkit_protocol::SecretBytes::new(vec![0x32; 16])),
        ),
        "Decrypt",
        |response| {
            let decrypted = response
                .decrypt()
                .and_then(|result| result.data())
                .ok_or("successful Decrypt response has no plaintext Data")?;
            Ok(decrypted.with_bytes(|value| SecretBytes::new(value.to_vec())))
        },
    )?;
    assert!(
        decrypted.with_bytes(|bytes| bytes == plaintext.as_slice()),
        "Decrypt restores the test plaintext"
    );
    Ok(())
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.24, Tables 235–237.
fn cosmian_kms_accepts_kmip_2_1_hash_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let parameters = structure([kmip_item(0x0042_0038, Value::enumeration(6))?])?;
    execute_one(
        &mut client,
        ClientRequest::hash(
            HashRequest::new(parameters).with_data(OperationData::ByteString(
                kmipkit_protocol::SecretBytes::new(b"KMIPKit Cosmian hash integration".to_vec()),
            )),
        ),
        "Hash",
        |response| {
            let digest = response
                .hash()
                .and_then(|result| result.data())
                .ok_or("successful Hash response has no digest Data")?;
            digest.with_bytes(|bytes| {
                assert!(!bytes.is_empty(), "the returned digest is nonempty");
            });
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.32, Tables 259–261; MAC Data tag: §11.56, Table 487.
fn cosmian_kms_accepts_kmip_2_1_mac_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_mac_key(&mut client)?;
    execute_one(
        &mut client,
        ClientRequest::mac(
            MacRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier))
                .with_data(OperationData::ByteString(SecretBytes::new(
                    b"KMIPKit MAC integration".to_vec(),
                ))),
        ),
        "MAC",
        |response| {
            let mac = response
                .mac()
                .and_then(|result| result.mac_data())
                .ok_or("successful MAC response has no MAC Data")?;
            mac.with_bytes(|bytes| assert!(!bytes.is_empty(), "the returned MAC is nonempty"));
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.33, Tables 262–264; MAC Data and Validity Indicator tags: §11.56, Table 487.
fn cosmian_kms_accepts_kmip_2_1_mac_verify_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_mac_key(&mut client)?;
    let data = b"KMIPKit MAC Verify integration".to_vec();
    let mac = execute_one(
        &mut client,
        ClientRequest::mac(
            MacRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier.clone()))
                .with_data(OperationData::ByteString(SecretBytes::new(data.clone()))),
        ),
        "MAC setup for MAC Verify",
        |response| {
            let mac = response
                .mac()
                .and_then(|result| result.mac_data())
                .ok_or("successful MAC setup has no MAC Data")?;
            Ok(mac.with_bytes(|bytes| SecretBytes::new(bytes.to_vec())))
        },
    )?;
    execute_one(
        &mut client,
        ClientRequest::mac_verify(
            MacVerifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier))
                .with_data(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(data),
                ))
                .with_mac_data(mac),
        ),
        "MAC Verify",
        |response| {
            let result = response
                .mac_verify()
                .ok_or("KMIPKit did not return a typed MAC Verify result")?;
            assert!(
                result.validity_indicator().is_some(),
                "successful single-part MAC Verify returns a Validity Indicator"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.55, Tables 334–336; Signature Data tag: §11.56, Table 487.
fn cosmian_kms_accepts_kmip_2_1_sign_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let (private_identifier, _) = create_rsa_key_pair(&mut client)?;
    activate_key(&mut client, &private_identifier)?;
    execute_one(
        &mut client,
        ClientRequest::sign(
            SignRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(private_identifier))
                .with_data(OperationData::ByteString(SecretBytes::new(
                    b"KMIPKit signing integration".to_vec(),
                ))),
        ),
        "Sign",
        |response| {
            let signature = response
                .sign()
                .and_then(|result| result.signature_data())
                .ok_or("successful Sign response has no Signature Data")?;
            signature.with_bytes(|bytes| {
                assert!(!bytes.is_empty(), "the returned signature is nonempty");
            });
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.56, Tables 337–339; Signature Data and Validity Indicator tags: §11.56, Table 487.
fn cosmian_kms_accepts_kmip_2_1_signature_verify_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let (private_identifier, public_identifier) = create_rsa_key_pair(&mut client)?;
    activate_key(&mut client, &private_identifier)?;
    activate_key(&mut client, &public_identifier)?;
    let data = b"KMIPKit signature verification integration".to_vec();
    let signature = execute_one(
        &mut client,
        ClientRequest::sign(
            SignRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(private_identifier))
                .with_data(OperationData::ByteString(SecretBytes::new(data.clone()))),
        ),
        "Sign setup for Signature Verify",
        |response| {
            let signature = response
                .sign()
                .and_then(|result| result.signature_data())
                .ok_or("successful Sign setup has no Signature Data")?;
            Ok(signature.with_bytes(|bytes| SecretBytes::new(bytes.to_vec())))
        },
    )?;
    execute_one(
        &mut client,
        ClientRequest::signature_verify(
            SignatureVerifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(public_identifier))
                .with_data(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(data),
                ))
                .with_signature_data(signature),
        ),
        "Signature Verify",
        |response| {
            let result = response
                .signature_verify()
                .ok_or("KMIPKit did not return a typed Signature Verify result")?;
            assert!(
                result.validity_indicator().is_some(),
                "successful single-part Signature Verify returns a Validity Indicator"
            );
            Ok(())
        },
    )
}

fn client() -> Result<Client, Box<dyn Error>> {
    let certificate_directory = certificate_directory();
    let transport_configuration = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", 5696))
        .tls_server_name("localhost")
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file(certificate_directory.join("client/client.crt"))?,
            PrivateKeyInput::from_pem_file(certificate_directory.join("client/client.key"))?,
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file(certificate_directory.join("client/ca.crt"))?,
        ]))
        .build()?;
    let registry = client_extension_registry(Vec::new(), extension::defaults())?;
    Ok(Client::new(
        ClientConfiguration::new(registry),
        transport_configuration,
    )?)
}

fn execute_one<T>(
    client: &mut Client,
    request: ClientRequest,
    operation: &str,
    validate_response: impl FnOnce(ClientResponseView<'_>) -> Result<T, Box<dyn Error>>,
) -> Result<T, Box<dyn Error>> {
    let response = client.execute(
        ClientBatch::new(ClientBatchItem::new(request)),
        &CodecLimits::defaults(),
    )?;
    assert_eq!(response.len(), 1, "Cosmian returns one {operation} result");
    let item = response
        .get(0)
        .ok_or_else(|| format!("Cosmian response has no {operation} result"))?;
    let outcome = item.outcome();
    let status = outcome.result().status();
    let reason = outcome
        .result()
        .reason()
        .map(kmipkit_protocol::ResultReason::raw);
    assert_eq!(
        status.raw(),
        SUCCESS,
        "Cosmian {operation} failed: Result Status {} ({}) and Result Reason {:?}",
        status.raw(),
        status.known_name().unwrap_or("unknown"),
        reason,
    );
    validate_response(outcome.response())
}

fn response_result_fields(error: &ClientError) -> Option<(Option<u32>, Option<u32>, Option<u32>)> {
    error.response_ttlv()?.with_ttlv(|root| {
        let batch_item = root
            .children()
            .iter()
            .find(|item| item.tag().raw() == BATCH_ITEM)?;
        batch_item.with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                return None;
            };
            let read_enumeration = |tag| {
                batch_item.children().iter().find_map(|field| {
                    (field.tag().raw() == tag)
                        .then(|| {
                            field.with_value(|value| match value {
                                ValueView::Enumeration(value) => Some(*value),
                                _ => None,
                            })
                        })
                        .flatten()
                })
            };
            Some((
                read_enumeration(OPERATION),
                read_enumeration(RESULT_STATUS),
                read_enumeration(RESULT_REASON),
            ))
        })
    })
}

fn create_symmetric_key(client: &mut Client) -> Result<String, Box<dyn Error>> {
    let attributes = AttributeSet::try_new([
        kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(AES))?,
        kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))?,
        kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(12))?,
    ])?;
    execute_one(
        client,
        ClientRequest::Create(CreateRequest::new(
            ObjectType::from_raw(SYMMETRIC_KEY),
            attributes,
        )),
        "Create",
        |response| {
            let identifier = response
                .create()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Create response has no Unique Identifier")?;
            text_identifier(identifier)
        },
    )
}

fn create_mac_key(client: &mut Client) -> Result<String, Box<dyn Error>> {
    let attributes = AttributeSet::try_new([
        kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(9))?,
        kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))?,
        kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(384))?, // MAC Generate + MAC Verify, §12.1 Table 495.
    ])?;
    let identifier = execute_one(
        client,
        ClientRequest::Create(CreateRequest::new(
            ObjectType::from_raw(SYMMETRIC_KEY),
            attributes,
        )),
        "Create HMAC key",
        |response| {
            let identifier = response
                .create()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Create HMAC key response has no Unique Identifier")?;
            text_identifier(identifier)
        },
    )?;
    activate_key(client, &identifier)?;
    Ok(identifier)
}

fn create_rsa_key_pair(client: &mut Client) -> Result<(String, String), Box<dyn Error>> {
    execute_one(
        client,
        ClientRequest::CreateKeyPair(rsa_key_pair_request()?),
        "Create Key Pair test setup",
        |response| {
            let key_pair = response
                .create_key_pair()
                .ok_or("successful Create Key Pair setup has no typed result")?;
            let private_identifier = key_pair
                .private_key_unique_identifier()
                .ok_or("successful Create Key Pair setup has no private-key identifier")?;
            let public_identifier = key_pair
                .public_key_unique_identifier()
                .ok_or("successful Create Key Pair setup has no public-key identifier")?;
            Ok((
                text_identifier(private_identifier)?,
                text_identifier(public_identifier)?,
            ))
        },
    )
}

fn activate_key(client: &mut Client, identifier: &str) -> Result<(), Box<dyn Error>> {
    execute_one(
        client,
        ClientRequest::activate(ActivateRequest::new(Some(UniqueIdentifier::TextString(
            identifier.to_owned(),
        )))),
        "Activate test key",
        |response| {
            let returned_identifier = response
                .activate()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Activate response has no Unique Identifier")?;
            text_identifier(returned_identifier)?;
            Ok(())
        },
    )
}

fn rsa_key_pair_request() -> Result<CreateKeyPairRequest, Box<dyn Error>> {
    let common_attributes = AttributeSet::try_new([
        kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(RSA))?,
        kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(2048))?,
    ])?;
    // OASIS KMIP v2.1 §12.1, Table 495: Sign is 0x01; Verify is 0x02.
    let private_attributes =
        AttributeSet::try_new([kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(1))?])?;
    let public_attributes =
        AttributeSet::try_new([kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(2))?])?;
    Ok(CreateKeyPairRequest::new()
        .with_common_attributes(common_attributes)
        .with_private_key_attributes(private_attributes)
        .with_public_key_attributes(public_attributes))
}

fn create_rsa_public_key(client: &mut Client) -> Result<String, Box<dyn Error>> {
    execute_one(
        client,
        ClientRequest::CreateKeyPair(rsa_key_pair_request()?),
        "Create Key Pair test setup",
        |response| {
            let identifier = response
                .create_key_pair()
                .and_then(|result| result.public_key_unique_identifier())
                .ok_or("successful Create Key Pair setup has no public-key identifier")?;
            text_identifier(identifier)
        },
    )
}

fn add_comment(client: &mut Client, identifier: &str, value: &str) -> Result<(), Box<dyn Error>> {
    execute_one(
        client,
        ClientRequest::add_attribute(AddAttributeRequest::new(
            Some(identifier.to_owned()),
            NewAttribute::new(comment_item(value)?),
        )),
        "Add Attribute",
        |response| {
            let returned_identifier = response
                .add_attribute()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Add Attribute response has no Unique Identifier")?;
            assert_ne!(returned_identifier, "");
            Ok(())
        },
    )
}

fn text_identifier(identifier: &UniqueIdentifier) -> Result<String, Box<dyn Error>> {
    match identifier {
        UniqueIdentifier::TextString(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err("server-generated Unique Identifier is not a nonempty Text String".into()),
    }
}

fn comment_item(value: &str) -> Result<Item, Box<dyn Error>> {
    kmip_item(COMMENT, Value::text_string(value.to_owned()))
}

fn kmip_item(raw_tag: u32, value: Value) -> Result<Item, Box<dyn Error>> {
    let tag = RawTag::new(raw_tag)?.try_checked()?;
    Ok(Item::new(tag, value)?)
}

fn structure(items: impl IntoIterator<Item = Item>) -> Result<Structure, Box<dyn Error>> {
    let mut structure = Structure::new();
    for item in items {
        structure.try_push(item)?;
    }
    Ok(structure)
}

fn cipher_parameters() -> Result<Structure, Box<dyn Error>> {
    structure([
        kmip_item(0x0042_0011, Value::enumeration(1))?, // CBC
        kmip_item(0x0042_005F, Value::enumeration(3))?, // PKCS5 padding
    ])
}

fn certificate_directory() -> PathBuf {
    std::env::var_os("KMIPKIT_COSMIAN_CERT_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/cosmian-kms/certs"),
        PathBuf::from,
    )
}
