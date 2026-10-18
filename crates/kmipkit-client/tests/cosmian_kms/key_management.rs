//! Manifest-driven KMIP Derive Key, Split Key, and re-key integration cases.
//!
//! Normative requests: OASIS KMIP v2.1 §§6.1.14, 6.1.27, 6.1.46–6.1.47,
//! Tables 205–207, 244–246, and 305–312. PBKDF2/HKDF parameters use §7.12,
//! Table 364; output cryptographic attributes use §§4.13–4.15, Tables 53–58.

use std::collections::BTreeSet;
use std::error::Error;

use kmipkit_client::{Client, ClientRequest};
use kmipkit_protocol::{
    AttributeSet, CreateKeyPairRequest, CreateSplitKeyRequest, DerivationMethod,
    DerivationParameters, DeriveKeyRequest, JoinSplitKeyRequest, ObjectType, RekeyKeyPairRequest,
    RekeyRequest, SecretBytes, SplitKeyMethod, SplitKeyReference, UniqueIdentifier,
};
use kmipkit_ttlv::{Structure, Value};

const FAMILY: &str = "cosmian_kms::key_management";
const CASE_IDS: &str = include_str!("key-management-case-ids.txt");
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_DOMAIN_PARAMETERS: u32 = 0x0042_0029;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const RECOMMENDED_CURVE: u32 = 0x0042_0060;
const SECRET_DATA_TYPE: u32 = 0x0042_0086;

#[test]
fn cosmian_key_management_manifest_cases_have_typed_executors() {
    assert_manifest_parity();
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_528_key_management_matrix() {
    assert_manifest_parity();
    let mut failures = Vec::new();
    let selected = selected_case_ids();
    for case_id in &selected {
        match run_case(case_id) {
            Ok(()) => println!("case_id={case_id} classification=typed_success"),
            Err(error) => {
                let classification = classify_error(error.as_ref());
                let target_request_sent = match classification {
                    "setup_rejected" => "false",
                    "rejected" => "true",
                    _ => "unknown",
                };
                println!(
                    "case_id={case_id} classification={classification} target_request_sent={target_request_sent} detail={error}"
                );
                failures.push(case_id.to_owned());
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} key-management cases did not complete successfully; see sanitized per-case output",
        failures.len(),
        selected.len()
    );
}

fn assert_manifest_parity() {
    let manifest = super::common::manifest_rows(FAMILY).expect("key-management manifest rows");
    let ids = case_ids().collect::<Vec<_>>();
    let manifest_ids = manifest
        .iter()
        .filter_map(|row| row["case_id"].as_str())
        .collect::<BTreeSet<_>>();
    let executable_ids = ids.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        ids.len(),
        executable_ids.len(),
        "duplicate key-management case ID"
    );
    assert_eq!(
        executable_ids, manifest_ids,
        "every key-management manifest row must map to exactly one typed executor"
    );
    for row in manifest {
        assert!(
            matches!(
                row["operation"].as_str(),
                Some(
                    "Derive Key"
                        | "Create Split Key"
                        | "Join Split Key"
                        | "Re-key"
                        | "Re-key Key Pair"
                )
            ),
            "unmapped key-management operation for {}",
            row["case_id"]
        );
    }
}

fn case_ids() -> impl Iterator<Item = &'static str> {
    CASE_IDS.lines().filter(|line| !line.is_empty())
}

fn build_case_request(
    row: &serde_json::Value,
    case_id: &str,
    source_ids: Vec<UniqueIdentifier>,
) -> Result<ClientRequest, Box<dyn Error>> {
    let operation = super::common::required_text(row, "operation")?;
    let source = || {
        source_ids
            .first()
            .cloned()
            .ok_or_else(|| format!("case {case_id}: {operation} requires a source identifier"))
    };
    match operation {
        "Re-key" => Ok(ClientRequest::rekey(
            RekeyRequest::new()
                .with_unique_identifier(source()?)
                .with_attributes(rekey_attributes(row)?),
        )),
        "Re-key Key Pair" => Ok(ClientRequest::rekey_key_pair(
            RekeyKeyPairRequest::new()
                .with_private_key_unique_identifier(source()?)
                .with_common_attributes(key_pair_attributes(row, case_id)?),
        )),
        "Create Split Key" => {
            let object_name = super::common::required_text(row, "object_type")?;
            let parts = manifest_integer(row, "parts").unwrap_or(2);
            let threshold = manifest_integer(row, "threshold").unwrap_or(parts);
            let request = CreateSplitKeyRequest::new(
                object_type(object_name)?,
                parts,
                threshold,
                SplitKeyMethod::from_raw(1),
                split_attributes(row, case_id)?,
            )
            .with_unique_identifier(source()?);
            Ok(ClientRequest::CreateSplitKey(request))
        }
        "Join Split Key" => {
            let parts = manifest_integer(row, "parts").unwrap_or(2);
            let threshold = manifest_integer(row, "threshold").unwrap_or(parts);
            if source_ids.len() != usize::try_from(parts)? {
                return Err(format!(
                    "case {case_id}: Join Split Key needs exactly {parts} case-owned split identifiers"
                )
                .into());
            }
            let references = source_ids
                .into_iter()
                .map(|id| {
                    SplitKeyReference::new(id).with_split_key_threshold(threshold.cast_unsigned())
                })
                .collect();
            let output_type = object_type(
                row["parameters"]["output_object_type"]
                    .as_str()
                    .unwrap_or("SymmetricKey"),
            )?;
            Ok(ClientRequest::join_split_key(
                JoinSplitKeyRequest::new(output_type, references)
                    .with_attributes(derive_output_attributes(row, output_type)?),
            ))
        }
        "Derive Key" => build_derive_key_request(row, case_id, source_ids),
        other => {
            Err(format!("case {case_id}: unsupported key-management operation {other}").into())
        }
    }
}

fn build_derive_key_request(
    row: &serde_json::Value,
    case_id: &str,
    source_ids: Vec<UniqueIdentifier>,
) -> Result<ClientRequest, Box<dyn Error>> {
    let output_type = object_type(
        row["parameters"]["output_object_type"]
            .as_str()
            .unwrap_or("SymmetricKey"),
    )?;
    let method_name = row["parameters"]["method"].as_str().unwrap_or("PBKDF2");
    let method = match method_name {
        "PBKDF2" => DerivationMethod::from_raw(1),
        "HKDF" => DerivationMethod::from_raw(10),
        "Asymmetric Key" => DerivationMethod::from_raw(8),
        other => return Err(format!("case {case_id}: unmapped Derivation Method {other}").into()),
    };
    let mut parameters = DerivationParameters::new();
    if let Some(hash) = row["parameters"]["hash"].as_str() {
        let hash_value = hash_value(hash)
            .ok_or_else(|| format!("case {case_id}: unmapped Hashing Algorithm {hash}"))?;
        let mut crypto_parameters = Structure::new();
        crypto_parameters.try_push(super::kmip_item(
            HASHING_ALGORITHM,
            Value::enumeration(hash_value),
        )?)?;
        parameters = parameters.with_cryptographic_parameters(crypto_parameters);
    }
    match row["parameters"]["salt"].as_str() {
        Some("present") => {
            parameters = parameters.with_salt(SecretBytes::new(b"kmipkit-salt".to_vec()));
        }
        Some("absent") | None => {}
        _ => return Err(format!("case {case_id} has an unrecognized Salt dimension").into()),
    }
    if let Some(iterations) = row["parameters"]["iterations"].as_i64() {
        parameters = parameters.with_iteration_count(i32::try_from(iterations)?);
    }
    if row["parameters"]["info"] == "present" {
        parameters =
            parameters.with_derivation_data(SecretBytes::new(b"kmipkit-hkdf-info".to_vec()));
    }
    if method_name == "PBKDF2" {
        parameters =
            parameters.with_derivation_data(SecretBytes::new(b"disposable-test-password".to_vec()));
    } else if method_name == "Asymmetric Key" {
        parameters = parameters.with_derivation_data(SecretBytes::new(Vec::new()));
    }
    Ok(ClientRequest::derive_key(DeriveKeyRequest::new(
        output_type,
        source_ids,
        method,
        parameters,
        derive_output_attributes(row, output_type)?,
    )))
}

fn encode_case_request(request: ClientRequest) -> Result<Structure, Box<dyn Error>> {
    match request {
        ClientRequest::Rekey(request) => Ok(request.into_ttlv_payload()?),
        ClientRequest::RekeyKeyPair(request) => Ok(request.into_ttlv_payload()?),
        ClientRequest::CreateSplitKey(request) => Ok(request.into_ttlv_payload()?),
        ClientRequest::JoinSplitKey(request) => Ok(request.into_ttlv_payload()?),
        ClientRequest::DeriveKey(request) => Ok(request.into_ttlv_payload()?),
        _ => Err("key-management inventory produced an unexpected request variant".into()),
    }
}

fn selected_case_ids() -> Vec<&'static str> {
    let Some(filter) = std::env::var("KMIPKIT_COSMIAN_CASE_FILTER").ok() else {
        return case_ids().collect();
    };
    let selected = filter.split(',').collect::<BTreeSet<_>>();
    let all = case_ids().collect::<BTreeSet<_>>();
    assert!(
        selected.iter().all(|case_id| all.contains(case_id)),
        "KMIPKIT_COSMIAN_CASE_FILTER contains an unknown key-management case ID"
    );
    selected
        .into_iter()
        .map(|case_id| *all.get(case_id).expect("validated key-management case ID"))
        .collect()
}

fn run_case(case_id: &str) -> Result<(), Box<dyn Error>> {
    let row = super::common::manifest_row(case_id)?;
    let operation = super::common::required_text(&row, "operation")?;
    let mut client = super::client()?;
    let mut ownership = super::common::CaseOwnership::default();
    let operation_result = match operation {
        "Re-key" => run_rekey(&mut client, case_id, &row, &mut ownership),
        "Re-key Key Pair" => run_rekey_key_pair(&mut client, case_id, &row, &mut ownership),
        "Create Split Key" => run_create_split(&mut client, case_id, &row, &mut ownership),
        "Join Split Key" => run_join_split(&mut client, case_id, &row, &mut ownership),
        "Derive Key" => run_derive_key(&mut client, case_id, &row, &mut ownership),
        other => {
            Err(format!("case {case_id}: unsupported key-management operation {other}").into())
        }
    }
    .map_err(|error| error.to_string());
    super::common::finish_operation_with_cleanup(operation_result, || {
        super::common::cleanup_owned_objects(&mut client, case_id, &mut ownership)
    })
    .map_err(|error| -> Box<dyn Error> { error.into() })
}

fn run_rekey(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let source = super::common::provision_object(client, "SymmetricKey", case_id, row, ownership)?;
    let request = build_case_request(row, case_id, vec![identifier(&source.primary)])?;
    super::common::execute_case(client, case_id, request, "Re-key", |response| {
        let replacement = response
            .rekey()
            .and_then(|result| result.unique_identifier())
            .ok_or_else(|| "Re-key omitted replacement Unique Identifier".to_owned())?;
        let replacement = super::text_identifier(replacement)
            .map_err(|_| "Re-key returned a non-text identifier".to_owned())?;
        ownership.track(replacement);
        Ok(())
    })
}

fn run_rekey_key_pair(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let source = super::common::provision_object(client, "PrivateKey", case_id, row, ownership)?;
    let request = build_case_request(row, case_id, vec![identifier(&source.primary)])?;
    super::common::execute_case(client, case_id, request, "Re-key Key Pair", |response| {
        let result = response
            .rekey_key_pair()
            .ok_or_else(|| "KMIPKit did not return a typed Re-key Key Pair result".to_owned())?;
        for identifier in [
            result.private_key_unique_identifier(),
            result.public_key_unique_identifier(),
        ] {
            let identifier = identifier
                .ok_or_else(|| "Re-key Key Pair omitted a replacement identifier".to_owned())?;
            let identifier = super::text_identifier(identifier)
                .map_err(|_| "Re-key Key Pair returned a non-text identifier".to_owned())?;
            ownership.track(identifier);
        }
        Ok(())
    })
}

fn run_create_split(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let source_type = super::common::required_text(row, "object_type")?;
    let source = super::common::provision_object(client, source_type, case_id, row, ownership)?;
    let parts = manifest_integer(row, "parts").unwrap_or(2);
    let request = build_case_request(row, case_id, vec![identifier(&source.primary)])?;
    super::common::execute_case(client, case_id, request, "Create Split Key", |response| {
        let result = response
            .create_split_key()
            .ok_or_else(|| "KMIPKit did not return a typed Create Split Key result".to_owned())?;
        if result.unique_identifiers().len() != usize::try_from(parts).unwrap_or_default() {
            return Err(
                "Create Split Key returned a count different from Split Key Parts".to_owned(),
            );
        }
        for identifier in result.unique_identifiers() {
            let identifier = super::text_identifier(identifier)
                .map_err(|_| "Create Split Key returned a non-text identifier".to_owned())?;
            ownership.track(identifier);
        }
        Ok(())
    })
}

fn run_join_split(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let parts = manifest_integer(row, "parts").unwrap_or(2);
    let threshold = manifest_integer(row, "threshold").unwrap_or(parts);
    let source_row = serde_json::json!({
        "parameters": {"key_bits": row["parameters"]["key_bits"]},
        "wire_enum_values": row["wire_enum_values"]
    });
    let source =
        super::common::provision_object(client, "SymmetricKey", case_id, &source_row, ownership)?;
    let split_request = CreateSplitKeyRequest::new(
        ObjectType::from_raw(2),
        parts,
        threshold,
        SplitKeyMethod::from_raw(1),
        split_attributes(&source_row, case_id)?,
    )
    .with_unique_identifier(identifier(&source.primary));
    let split_ids = super::common::execute_case(
        client,
        case_id,
        ClientRequest::CreateSplitKey(split_request),
        "Create Split Key setup",
        |response| {
            let result = response
                .create_split_key()
                .ok_or_else(|| "setup returned no typed Create Split Key result".to_owned())?;
            let mut identifiers = Vec::new();
            for identifier in result.unique_identifiers() {
                let identifier = super::text_identifier(identifier)
                    .map_err(|_| "setup returned a non-text Split Key identifier".to_owned())?;
                ownership.track(identifier.clone());
                identifiers.push(identifier);
            }
            if identifiers.len() != usize::try_from(parts).unwrap_or_default() {
                return Err("setup split count did not match the manifest".to_owned());
            }
            Ok(identifiers)
        },
    )?;
    let request = build_case_request(
        row,
        case_id,
        split_ids.iter().map(|id| identifier(id)).collect(),
    )?;
    super::common::execute_case(client, case_id, request, "Join Split Key", |response| {
        let joined = response
            .join_split_key()
            .and_then(|result| result.unique_identifier())
            .ok_or_else(|| "Join Split Key omitted output Unique Identifier".to_owned())?;
        let joined = super::text_identifier(joined)
            .map_err(|_| "Join Split Key returned a non-text identifier".to_owned())?;
        ownership.track(joined);
        Ok(())
    })
}

fn run_derive_key(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let source_type = super::common::required_text(row, "object_type")?;
    let mut ids = Vec::new();
    if source_type == "PrivateKey" {
        let (private_id, public_id) = create_x25519_pair(client, case_id, row, ownership)?;
        ids.push(identifier(&private_id));
        ids.push(identifier(&public_id));
    } else {
        let source = super::common::provision_object(client, source_type, case_id, row, ownership)?;
        ids.push(identifier(&source.primary));
    }
    let request = build_case_request(row, case_id, ids)?;
    super::common::execute_case(client, case_id, request, "Derive Key", |response| {
        let identifier = response
            .derive_key()
            .and_then(|result| result.unique_identifier())
            .ok_or_else(|| "Derive Key omitted output Unique Identifier".to_owned())?;
        let identifier = super::text_identifier(identifier)
            .map_err(|_| "Derive Key returned a non-text identifier".to_owned())?;
        ownership.track(identifier);
        Ok(())
    })
}

fn create_x25519_pair(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(String, String), Box<dyn Error>> {
    let algorithm = enum_value(row, "cryptographic_algorithm").unwrap_or(14);
    let curve = enum_value(row, "recommended_curve").unwrap_or(0x45);
    let mut domain_parameters = Structure::new();
    domain_parameters.try_push(super::kmip_item(
        RECOMMENDED_CURVE,
        Value::enumeration(curve),
    )?)?;
    let attributes = AttributeSet::try_new([
        super::kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(algorithm))?,
        super::kmip_item(
            CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
            Value::structure(domain_parameters),
        )?,
        super::kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))?,
    ])?;
    let request = CreateKeyPairRequest::new()
        .with_common_attributes(attributes)
        .with_private_key_attributes(AttributeSet::try_new([super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(1),
        )?])?)
        .with_public_key_attributes(AttributeSet::try_new([super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(2),
        )?])?);
    super::common::execute_created_key_pair(client, case_id, request, ownership)
}

fn key_pair_attributes(
    row: &serde_json::Value,
    case_id: &str,
) -> Result<AttributeSet, Box<dyn Error>> {
    let mut attributes = vec![super::kmip_item(
        CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(enum_value(row, "cryptographic_algorithm").unwrap_or(4)),
    )?];
    if let Some(bits) = row["parameters"]["key_bits"].as_i64() {
        attributes.push(super::kmip_item(
            CRYPTOGRAPHIC_LENGTH,
            Value::integer(i32::try_from(bits)?),
        )?);
    }
    if let Some(curve) = enum_value(row, "recommended_curve") {
        let mut parameters = Structure::new();
        parameters.try_push(super::kmip_item(
            RECOMMENDED_CURVE,
            Value::enumeration(curve),
        )?)?;
        attributes.push(super::kmip_item(
            CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
            Value::structure(parameters),
        )?);
    }
    attributes.push(super::comment_item(case_id)?);
    Ok(AttributeSet::try_new(attributes)?)
}

fn rekey_attributes(row: &serde_json::Value) -> Result<Structure, Box<dyn Error>> {
    let mut attributes = Structure::new();
    if let Some(algorithm) = enum_value(row, "cryptographic_algorithm") {
        attributes.try_push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(algorithm),
        )?)?;
    }
    if let Some(bits) = row["parameters"]["key_bits"].as_i64() {
        attributes.try_push(super::kmip_item(
            CRYPTOGRAPHIC_LENGTH,
            Value::integer(i32::try_from(bits)?),
        )?)?;
    }
    Ok(attributes)
}

fn split_attributes(
    row: &serde_json::Value,
    case_id: &str,
) -> Result<AttributeSet, Box<dyn Error>> {
    let mut attributes = vec![super::comment_item(case_id)?];
    if let Some(algorithm) = enum_value(row, "cryptographic_algorithm") {
        attributes.push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(algorithm),
        )?);
    }
    if let Some(bits) = row["parameters"]["key_bits"].as_i64() {
        attributes.push(super::kmip_item(
            CRYPTOGRAPHIC_LENGTH,
            Value::integer(i32::try_from(bits)?),
        )?);
    }
    if row["object_type"] == "SecretData" {
        attributes.push(super::kmip_item(SECRET_DATA_TYPE, Value::enumeration(1))?);
    }
    attributes.push(super::kmip_item(
        CRYPTOGRAPHIC_USAGE_MASK,
        Value::integer(12),
    )?);
    Ok(AttributeSet::try_new(attributes)?)
}

fn derive_output_attributes(
    row: &serde_json::Value,
    object_type: ObjectType,
) -> Result<AttributeSet, Box<dyn Error>> {
    let mut attributes = Vec::new();
    if matches!(object_type.raw(), 1..=3) {
        attributes.push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(enum_value(row, "cryptographic_algorithm").unwrap_or(3)),
        )?);
    }
    let bits = row["parameters"]["key_bits"].as_i64().unwrap_or(256);
    attributes.push(super::kmip_item(
        CRYPTOGRAPHIC_LENGTH,
        Value::integer(i32::try_from(bits)?),
    )?);
    if object_type.raw() == 7 {
        attributes.push(super::kmip_item(SECRET_DATA_TYPE, Value::enumeration(1))?);
    }
    Ok(AttributeSet::try_new(attributes)?)
}

fn object_type(name: &str) -> Result<ObjectType, Box<dyn Error>> {
    let raw = match name {
        "Certificate" => 1,
        "SymmetricKey" => 2,
        "PublicKey" => 3,
        "PrivateKey" => 4,
        "SplitKey" => 5,
        "SecretData" => 7,
        "OpaqueObject" => 8,
        other => return Err(format!("unmapped Object Type {other}").into()),
    };
    Ok(ObjectType::from_raw(raw))
}

fn enum_value(row: &serde_json::Value, member: &str) -> Option<u32> {
    row["wire_enum_values"][member]["value"]
        .as_str()
        .and_then(|value| u32::from_str_radix(value.trim_start_matches("0x"), 16).ok())
}

fn manifest_integer(row: &serde_json::Value, member: &str) -> Option<i32> {
    row["parameters"][member]
        .as_i64()
        .and_then(|value| i32::try_from(value).ok())
}

fn hash_value(name: &str) -> Option<u32> {
    // OASIS KMIP v2.1 §11.21: Table values are SHA-1=4, SHA-224=5,
    // SHA-256=6, SHA-384=7, and SHA-512=8.
    match name {
        "SHA-1" => Some(4),
        "SHA-224" => Some(5),
        "SHA-256" => Some(6),
        "SHA-384" => Some(7),
        "SHA-512" => Some(8),
        _ => None,
    }
}

fn identifier(value: &str) -> UniqueIdentifier {
    UniqueIdentifier::TextString(value.to_owned())
}

fn classify_error(error: &dyn Error) -> &'static str {
    let text = error.to_string();
    if text.contains("case cleanup failed") {
        "cleanup_failed"
    } else if text.contains("client_blocked") || text.contains("category=") {
        "client_blocked"
    } else if text.contains("KMIP Result Status") || text.contains("Result Status") {
        if [" Create:", " Create Key Pair:", " Create Split Key setup:"]
            .iter()
            .any(|stage| text.contains(stage))
        {
            "setup_rejected"
        } else {
            "rejected"
        }
    } else {
        "setup_or_typed_validation_failed"
    }
}

#[cfg(test)]
mod inventory_tests {
    use super::*;

    #[test]
    fn key_management_case_ids_are_unique_and_match_manifest() {
        assert_manifest_parity();
    }

    #[test]
    fn setup_server_rejections_are_not_reported_as_target_operation_rejections() {
        let create_pair_rejection = std::io::Error::other(
            "case cosmian-528-derive-key-privatekey Create Key Pair: KMIP Result Status 1 (Operation Failed)",
        );
        let split_setup_rejection = std::io::Error::other(
            "case cosmian-528-join-split-key-aes Create Split Key setup: KMIP Result Status 1 (Operation Failed)",
        );
        let derive_rejection = std::io::Error::other(
            "case cosmian-528-derive-key-secretdata-hkdf Derive Key: KMIP Result Status 1 (Operation Failed)",
        );
        let split_target_rejection = std::io::Error::other(
            "case cosmian-528-create-split-key-aes Create Split Key: KMIP Result Status 1 (Operation Failed)",
        );

        assert_eq!(classify_error(&create_pair_rejection), "setup_rejected");
        assert_eq!(classify_error(&split_setup_rejection), "setup_rejected");
        assert_eq!(classify_error(&derive_rejection), "rejected");
        assert_eq!(classify_error(&split_target_rejection), "rejected");
    }

    #[test]
    fn hashing_algorithm_ids_match_the_kmip_21_enumeration() {
        assert_eq!(hash_value("SHA-1"), Some(4));
        assert_eq!(hash_value("SHA-224"), Some(5));
        assert_eq!(hash_value("SHA-256"), Some(6));
        assert_eq!(hash_value("SHA-384"), Some(7));
        assert_eq!(hash_value("SHA-512"), Some(8));
    }

    #[test]
    fn every_key_management_case_constructs_a_serializable_typed_request() {
        let rows =
            super::super::common::manifest_rows(FAMILY).expect("key-management manifest rows");
        let mut encoded = 0;
        for row in rows {
            let case_id = super::super::common::required_text(&row, "case_id")
                .expect("key-management case ID");
            let operation = super::super::common::required_text(&row, "operation")
                .expect("key-management operation");
            let source_ids = match operation {
                "Join Split Key" => {
                    let parts = manifest_integer(&row, "parts").unwrap_or(2);
                    (0..parts)
                        .map(|index| identifier(&format!("{case_id}-part-{index}")))
                        .collect()
                }
                "Derive Key" if row["parameters"]["method"] == "Asymmetric Key" => {
                    vec![identifier("private-source"), identifier("public-source")]
                }
                _ => vec![identifier("source")],
            };
            let request = build_case_request(&row, case_id, source_ids)
                .unwrap_or_else(|error| panic!("{case_id}: typed request constructor: {error}"));
            let payload = encode_case_request(request)
                .unwrap_or_else(|error| panic!("{case_id}: typed request serialization: {error}"));
            assert!(
                !payload.view().children().is_empty(),
                "{case_id}: typed request has an empty payload"
            );
            encoded += 1;
        }
        assert_eq!(encoded, 216);
    }
}
