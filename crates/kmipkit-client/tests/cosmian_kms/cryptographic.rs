//! Matrix-driven cryptographic operation tests for KMIP 2.1.
//!
//! The operation citations and parameter values are read from each matrix row.
//! Hash and HMAC request payloads use known-answer vectors so response semantics
//! are checked independently of the server.

use std::collections::BTreeSet;
use std::error::Error;

use kmipkit_client::ClientRequest;
use kmipkit_protocol::{
    ActivateRequest, AttributeSet, HashRequest, ImportRequest, MacRequest, MacVerifyRequest,
    MacVerifyResponse, ObjectType, OperationData, SecretBytes, UniqueIdentifier,
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
const KEY_USAGE_MAC_GENERATE_VERIFY: i32 = 384;
const RAW_KEY_FORMAT_TYPE: u32 = 1;
const FIXTURE_KEY: [u8; 32] = [0x5A; 32];
const EXPECTED_TARGET_ROWS: usize = 1_320;
const EXPECTED_EXECUTABLE_ROWS: usize = 1_320;

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

const ENCRYPTION_PARAMETERS: [&str; 27] = [
    "aad",
    "alphabet",
    "curve",
    "data_shape",
    "documented_scheme",
    "hash",
    "iv_length_bits",
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
    "tag_length_bytes",
    "tweak",
    "vendor_algorithm_value",
    "wire_container",
];

const HASH_PARAMETERS: [&str; 3] = ["data_length_bytes", "data_mode", "hash"];
const MAC_PARAMETERS: [&str; 4] = ["hash", "hash_selection", "key_bits", "verification"];
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
    ExecutableHash,
    ExecutableMac,
    ExecutableMacVerify,
    DeliberatelyUnmapped(&'static str),
}

#[test]
fn cosmian_cryptographic_case_ids_cover_typed_success_rows_and_dispatch_registry() {
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
        "the reviewed cryptographic typed_success inventory changed"
    );
    assert_eq!(
        listed_ids, manifest_ids,
        "every target typed_success row needs exactly one checked-in case ID"
    );

    let mut executable_ids = BTreeSet::new();
    let mut unmapped_ids = BTreeSet::new();
    for row in &rows {
        validate_operation_contract(row).unwrap_or_else(|error| panic!("{error}"));
        match disposition(row).expect("every target row has a registry disposition") {
            CaseDisposition::ExecutableHash
            | CaseDisposition::ExecutableMac
            | CaseDisposition::ExecutableMacVerify => {
                assert!(
                    executable_ids.insert(case_id(row).expect("case ID")),
                    "duplicate executable mapping for {}",
                    case_id(row).expect("case ID")
                );
            }
            CaseDisposition::DeliberatelyUnmapped(_reason) => {
                assert!(
                    unmapped_ids.insert(case_id(row).expect("case ID")),
                    "duplicate unmapped registry entry for {}",
                    case_id(row).expect("case ID")
                );
            }
        }
    }

    assert!(executable_ids.is_disjoint(&unmapped_ids));
    assert_eq!(
        executable_ids
            .union(&unmapped_ids)
            .copied()
            .collect::<BTreeSet<_>>(),
        listed_ids,
        "the live dispatcher must classify every listed row exactly once"
    );
    assert_eq!(
        executable_ids.len(),
        EXPECTED_EXECUTABLE_ROWS,
        "update the explicit parity expectation when adding an executor"
    );
    assert_eq!(
        unmapped_ids.len(),
        EXPECTED_TARGET_ROWS - EXPECTED_EXECUTABLE_ROWS
    );
    eprintln!(
        "cryptographic registry: executable={} deliberately_unmapped={}",
        executable_ids.len(),
        unmapped_ids.len()
    );
}

#[test]
fn cosmian_cryptographic_sign_and_verify_rows_require_executable_dispatch() {
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
    let mut verify_rows = 0;
    let mut independent_signer_ids = BTreeSet::new();

    for row in &rows {
        let id = case_id(row).expect("case ID");
        match row["operation"].as_str() {
            Some("Sign") => {
                sign_rows += 1;
                assert!(
                    !matches!(disposition(row), Ok(CaseDisposition::DeliberatelyUnmapped(_))),
                    "Sign typed_success case {id} needs an executable request builder"
                );
            }
            Some("Signature Verify") => {
                verify_rows += 1;
                if row["parameters"]["setup_signature_source"].as_str()
                    == Some(INDEPENDENT_SIGNER_SOURCE)
                {
                    assert!(
                        matches!(disposition(row), Ok(CaseDisposition::DeliberatelyUnmapped(_))),
                        "case {id} requires the independent signer named by its matrix setup"
                    );
                    independent_signer_ids.insert(id);
                } else {
                    assert!(
                        !matches!(disposition(row), Ok(CaseDisposition::DeliberatelyUnmapped(_))),
                        "Signature Verify typed_success case {id} needs executable dispatch"
                    );
                }
            }
            _ => {}
        }
    }

    assert_eq!(sign_rows, 110, "reviewed Sign typed_success inventory changed");
    assert_eq!(
        verify_rows, 224,
        "reviewed Signature Verify typed_success inventory changed"
    );
    assert_eq!(independent_signer_ids.len(), INDEPENDENT_SIGNER_CASE_IDS.len());
    assert_eq!(
        independent_signer_ids,
        INDEPENDENT_SIGNER_CASE_IDS.into_iter().collect(),
        "only matrix rows that explicitly require an independent signer remain unmapped"
    );
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// The exact OASIS operation citation for every row is validated against that
// row's `oasis_source` in the offline contract test.
fn cosmian_528_cryptographic_matrix_serial_live_runner() {
    let rows = target_rows().unwrap_or_else(|error| panic!("{error}"));
    let selected = selected_rows(&rows).unwrap_or_else(|error| panic!("{error}"));
    let executable_count = selected
        .iter()
        .filter(|row| {
            matches!(
                disposition(row),
                Ok(CaseDisposition::ExecutableHash
                    | CaseDisposition::ExecutableMac
                    | CaseDisposition::ExecutableMacVerify)
            )
        })
        .count();
    let selected_count = selected.len();
    let mut client = if executable_count == 0 {
        None
    } else {
        Some(
            super::client()
                .unwrap_or_else(|error| panic!("Cosmian test client setup failed: {error}")),
        )
    };
    let mut failures = Vec::new();

    for row in selected {
        let id = case_id(row).unwrap_or_else(|error| panic!("{error}"));
        match dispatch_case(row, client.as_mut()) {
            Ok(()) => println!("case_id={id} classification=typed_success"),
            Err(error) => {
                println!("case_id={id} classification=uncovered detail={error}");
                failures.push(format!("{id}: {error}"));
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

fn target_rows() -> Result<Vec<JsonValue>, Box<dyn Error>> {
    let rows = super::common::manifest_rows(FAMILY)?;
    Ok(rows
        .into_iter()
        .filter(|row| {
            matches!(
                row["expected_outcome"].as_str(),
                Some("typed_success" | "source_documentation_discrepancy" | "documented_but_source_gap")
            )
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
    let algorithm = row["algorithm"]
        .as_str()
        .ok_or_else(|| format!("case {id} has no algorithm"))?;
    let parameters = row["parameters"]
        .as_object()
        .ok_or_else(|| format!("case {id} has no parameter object"))?;
    let allowed_parameters = match operation {
        "Encrypt" | "Decrypt" => &ENCRYPTION_PARAMETERS[..],
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
    const HASHES: [&str; 7] = [
        "SHA-256", "SHA-384", "SHA-512", "SHA3-224", "SHA3-256", "SHA3-384", "SHA3-512",
    ];
    if !HASHES.contains(&algorithm)
        || parameters.get("hash").and_then(JsonValue::as_str) != Some(algorithm)
        || parameters.get("data_mode").and_then(JsonValue::as_str) != Some("one-shot")
    {
        return Err(format!(
            "case {id} Hash violates the one-shot hash contract"
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
    let selection = parameters.get("hash_selection").and_then(JsonValue::as_str);
    if !selection.is_some_and(|selection| {
        [
            "attribute-algorithm",
            "explicit-hash",
            "key-block-algorithm",
        ]
        .contains(&selection)
    }) {
        return Err(format!("case {id} {operation} has unknown hash_selection"));
    }
    let expected_verification = if operation == "MAC" {
        "compute"
    } else {
        match parameters.get("verification").and_then(JsonValue::as_str) {
            Some(value @ ("valid" | "invalid")) => value,
            _ => return Err(format!("case {id} MAC Verify has unknown verification")),
        }
    };
    if parameters.get("verification").and_then(JsonValue::as_str) != Some(expected_verification)
        || parameters.get("key_bits").and_then(JsonValue::as_u64) != Some(256)
    {
        return Err(format!(
            "case {id} {operation} violates its HMAC parameters"
        ));
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
    if !["RSA", "ECDSA", "Ed25519", "Ed448"].contains(&algorithm)
        || !row["object_type"]
            .as_str()
            .is_some_and(|object_type| expected_objects.contains(&object_type))
        || !parameters
            .get("data_mode")
            .and_then(JsonValue::as_str)
            .is_some_and(|mode| ["raw", "prehashed"].contains(&mode))
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
            "key_bits" | "iv_length_bits" | "tag_length_bytes" | "data_length_bytes" => {
                value.is_null() || value.as_u64().is_some()
            }
            "salt_length_bytes" => {
                value.is_null() || value.as_u64().is_some() || value.as_str() == Some("digest")
            }
            "non_fips" | "deterministic" | "external_digest" => {
                value.is_null() || value.is_boolean()
            }
            "post_quantum" => value.is_null(),
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
        Some("Hash") => Ok(CaseDisposition::ExecutableHash),
        Some("MAC") => Ok(CaseDisposition::ExecutableMac),
        Some("MAC Verify") => Ok(CaseDisposition::ExecutableMacVerify),
        Some("Encrypt" | "Decrypt") => Ok(CaseDisposition::DeliberatelyUnmapped(
            "row-specific key provisioning and matching cipher parameters are not implemented",
        )),
        Some("Sign" | "Signature Verify") => Ok(CaseDisposition::DeliberatelyUnmapped(
            "row-specific key material and scheme-specific signature verification are not implemented",
        )),
        Some(operation) => Err(format!("case {id} has unmapped operation {operation}")),
        None => Err(format!("case {id} has no operation for dispatch")),
    }
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

fn dispatch_case(
    row: &JsonValue,
    client: Option<&mut kmipkit_client::Client>,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
    match disposition(row).map_err(|error| -> Box<dyn Error> { error.into() })? {
        CaseDisposition::ExecutableHash => {
            let client = client
                .ok_or_else(|| format!("case {id} Hash has no initialized Cosmian client"))?;
            run_hash_case(client, row)?;
            Ok(())
        }
        CaseDisposition::ExecutableMac | CaseDisposition::ExecutableMacVerify => {
            let client =
                client.ok_or_else(|| format!("case {id} has no initialized Cosmian client"))?;
            run_mac_family_case(client, row)
        }
        CaseDisposition::DeliberatelyUnmapped(reason) => {
            Err(format!("case {id} is deliberately unmapped: {reason}").into())
        }
    }
}

fn run_hash_case(
    client: &mut kmipkit_client::Client,
    row: &JsonValue,
) -> Result<(), Box<dyn Error>> {
    let id = case_id(row)?;
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

fn import_hmac_key(
    client: &mut kmipkit_client::Client,
    case_id: &str,
    row: &JsonValue,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let selection = required_parameter_text(row, "hash_selection")?;
    let algorithm_value = wire_enum_value(row, "cryptographic_algorithm")?;
    let attribute_algorithm = selection != "key-block-algorithm";
    let block_algorithm = selection != "attribute-algorithm";
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
    if required_parameter_text(row, "hash_selection")? != "explicit-hash" {
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
