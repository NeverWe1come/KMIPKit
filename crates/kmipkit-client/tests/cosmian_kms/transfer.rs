//! Manifest-driven KMIP Get, Export, Import, and Register integration cases.
//!
//! Normative operation payloads: OASIS KMIP v2.1 §§6.1.18–6.1.19 and
//! §§6.1.43–6.1.44, Tables 217–222 and 291–294. Key Format Type and Key
//! Material use §4.26, Tables 79–81; §11.25, Tables 460–461. The fixture
//! encoding field is never interpreted as a KMIP Key Format Type.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use kmipkit_client::{Client, ClientRequest};
use kmipkit_protocol::{
    AttributeReference, AttributeSet, ExportRequest, GetAttributesRequest, GetRequest,
    ImportRequest, KeyFormatType, KeyWrapType, ObjectType, OpaqueObjectContext, RegisterRequest,
    SecretBytes, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, Structure, Value, ValueView};

const FAMILY: &str = "cosmian_kms::transfer";
const CASE_IDS: &str = include_str!("transfer-case-ids.txt");
const CONFIGURABLE_KEM_PROBE_CASE_IDS: &str =
    include_str!("configurable-kem-transfer-probe-case-ids.txt");
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_DOMAIN_PARAMETERS: u32 = 0x0042_0029;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const RECOMMENDED_CURVE: u32 = 0x0042_0060;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
// OASIS KMIP v2.1 §4.55, Table 139.
const SENSITIVE: u32 = 0x0042_00CD;
const ACTIVATION_DATE: u32 = 0x0042_0001;
const CERTIFICATE_LENGTH: u32 = 0x0042_00AD;
const SECRET_DATA_TYPE: u32 = 0x0042_0086;
const LINK: u32 = 0x0042_004A;
const LINK_TYPE: u32 = 0x0042_004B;
const LINKED_OBJECT_IDENTIFIER: u32 = 0x0042_004C;
const KEY_BLOCK: u32 = 0x0042_0040;
// OASIS KMIP v2.1 §11.56, Table 487; §3.4, Table 18 defines its transparent-key use.
const KEY: u32 = 0x0042_003F;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;
const PRIVATE_KEY: u32 = 0x0042_0064;
const PUBLIC_KEY: u32 = 0x0042_006D;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const CERTIFICATE_VALUE: u32 = 0x0042_001E;
// OASIS KMIP v2.1 §11.56, Table 487: Symmetric Key (distinct from Result Status).
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const SECRET_DATA: u32 = 0x0042_0085;
const OPAQUE_OBJECT: u32 = 0x0042_005B;
const OPAQUE_DATA_TYPE: u32 = 0x0042_0059;
const OPAQUE_DATA_VALUE: u32 = 0x0042_005A;
const KEY_WRAPPING_DATA: u32 = 0x0042_0046;
const KEY_WRAPPING_SPECIFICATION: u32 = 0x0042_0047;
const SECRET_DATA_VALUE: u32 = 0x0042_0084;
const PKCS12_CERTIFICATE_LINK: u32 = 0x10C;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const PADDING_METHOD: u32 = 0x0042_005F;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const MASK_GENERATOR: u32 = 0x0042_0101;
const MGF_HASHING_ALGORITHM: u32 = 0x0042_0102;
const IV_LENGTH: u32 = 0x0042_00CD;
const TAG_LENGTH: u32 = 0x0042_00CE;
const WRAPPING_METHOD: u32 = 0x0042_009E;
const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ENCODING_OPTION: u32 = 0x0042_00A3;
const PKCS12_TEST_PASSWORD: &str = "kmipkit-test-password";
const USAGE_ENCRYPT: i64 = 0x04;
const USAGE_DECRYPT: i64 = 0x08;
const USAGE_WRAP_KEY: i64 = 0x10;
const USAGE_UNWRAP_KEY: i64 = 0x20;
const USAGE_MAC_GENERATE: i64 = 0x40;
const USAGE_MAC_VERIFY: i64 = 0x80;

fn provision_wrapped_target(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    if object_type.raw() == 7 {
        let length = row["parameters"]["target_bytes"]
            .as_u64()
            .ok_or("wrapped SecretData row has no target_bytes")?;
        let material = (0..length)
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>();
        let unique_id = super::common::register_secret_data_with_material(
            client,
            case_id,
            SecretBytes::new(material),
            ownership,
        )?;
        super::common::ensure_active_object(client, case_id, "SecretData", &unique_id)?;
        return Ok(unique_id);
    }
    let mut target_row = row.clone();
    let target_algorithm = row["parameters"]["target_algorithm"]
        .as_str()
        .unwrap_or("AES");
    target_row["algorithm"] = serde_json::Value::String(target_algorithm.to_owned());
    let target_algorithm_raw = algorithm_value(target_algorithm).ok_or_else(|| {
        format!("case {case_id}: no KMIP Algorithm value for target {target_algorithm}")
    })?;
    target_row["wire_enum_values"]["cryptographic_algorithm"]["value"] =
        serde_json::Value::String(format!("0x{target_algorithm_raw:08X}"));
    target_row["parameters"]["key_bits"] = serde_json::json!(if object_type.raw() == 2 {
        row["parameters"]["target_bytes"]
            .as_u64()
            .ok_or("wrapped SymmetricKey row has no target_bytes")?
            .checked_mul(8)
            .ok_or("wrapped SymmetricKey target size overflow")?
    } else {
        256
    });
    if object_type.raw() == 2 {
        // The Cosmian 5.28 wrapping e2e fixture gives both target and KEK
        // WrapKey | Encrypt usage (KMIP v2.1 §11.62, Table 493).
        target_row["parameters"]["usage_mask"] = serde_json::json!(USAGE_WRAP_KEY | USAGE_ENCRYPT);
        target_row["parameters"]["activation_date"] = serde_json::json!("now");
    }
    if object_type.raw() == 4
        && row["algorithm"] == "ConfigurableKEM"
        && row["key_format"] == "ConfigurableKEMSecretKey"
    {
        // OASIS KMIP v2.1 §4.55, Tables 139–140: explicitly request a
        // non-sensitive test key so the setup Export can exercise wrapping.
        target_row["parameters"]["sensitive"] = serde_json::json!(false);
    }
    let source = super::common::provision_object(
        client,
        super::common::required_text(row, "object_type")?,
        case_id,
        &target_row,
        ownership,
    )?;
    super::common::ensure_active_object(
        client,
        case_id,
        super::common::required_text(row, "object_type")?,
        &source.primary,
    )?;
    if target_row["parameters"]["sensitive"].as_bool() == Some(false) {
        verify_sensitive_attribute(client, case_id, &source.primary, false)?;
    }
    if object_type.raw() == 2 {
        verify_usage_mask(
            client,
            case_id,
            &source.primary,
            USAGE_WRAP_KEY | USAGE_ENCRYPT,
        )?;
    }
    Ok(source.primary)
}

fn provision_wrapping_key(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let key_type = row["parameters"]["wrapping_key_type"]
        .as_str()
        .unwrap_or("SymmetricKey");
    match key_type {
        "SymmetricKey" => provision_symmetric_kek(client, case_id, row, ownership),
        "PrivateKey" | "PublicKey" => provision_rsa_kek(client, case_id, row, ownership),
        "Certificate" => {
            let mut certificate_row = row.clone();
            certificate_row["parameters"]["key_bits"] = row["parameters"]["key_bits"].clone();
            certificate_row["fixture"]["encoding"] = serde_json::json!("DER/PFX");
            let private_id = import_pfx_source(client, case_id, &certificate_row, ownership)?;
            let certificate_id =
                pfx_certificate_identifier(client, case_id, &private_id.private_id, ownership)?;
            super::common::ensure_active_object(client, case_id, "Certificate", &certificate_id)?;
            Ok(certificate_id)
        }
        other => Err(format!("case {case_id}: unsupported wrapping key type {other}").into()),
    }
}

fn provision_symmetric_kek(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let bits = row["parameters"]["kek_bits"]
        .as_i64()
        .or_else(|| row["parameters"]["key_bits"].as_i64())
        .unwrap_or(256);
    let algorithm = row["parameters"]["kek_algorithm"]
        .as_str()
        .or_else(|| row["algorithm"].as_str())
        .unwrap_or("AES");
    let algorithm = algorithm_value(algorithm)
        .ok_or_else(|| format!("case {case_id}: unmapped wrapping-key algorithm {algorithm}"))?;
    let attributes = AttributeSet::try_new([
        super::kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(algorithm))?,
        super::kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(i32::try_from(bits)?))?,
        super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(i32::try_from(wrapping_key_usage_mask(
                "SymmetricKey",
                super::common::required_text(row, "operation")?,
            )?)?),
        )?,
        super::kmip_item(
            ACTIVATION_DATE,
            Value::date_time(super::common::now_unix_seconds()?),
        )?,
        super::common::name_attribute(&format!("{case_id}-kek"))?,
    ])?;
    let unique_id = super::common::create_result_id(
        client,
        case_id,
        ClientRequest::Create(kmipkit_protocol::CreateRequest::new(
            ObjectType::from_raw(2),
            attributes,
        )),
        "Create wrapping key",
        ownership,
    )?;
    let expected_usage = wrapping_key_usage_mask(
        "SymmetricKey",
        super::common::required_text(row, "operation")?,
    )?;
    verify_usage_mask(client, case_id, &unique_id, expected_usage)?;
    super::common::ensure_active_object(client, case_id, "SymmetricKey", &unique_id)?;
    Ok(unique_id)
}

fn provision_rsa_kek(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let bits = row["parameters"]["key_bits"].as_i64().unwrap_or(2048);
    let algorithm_name = row["algorithm"].as_str().unwrap_or("RSA");
    let algorithm = algorithm_value(algorithm_name).ok_or_else(|| {
        format!("case {case_id}: unmapped wrapping-key algorithm {algorithm_name}")
    })?;
    let mut common_items = vec![
        super::kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(algorithm))?,
        super::kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(i32::try_from(bits)?))?,
    ];
    if let Some(curve) = enum_value(row, "recommended_curve") {
        common_items.push(super::kmip_item(
            CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
            Value::structure(super::structure([super::kmip_item(
                RECOMMENDED_CURVE,
                Value::enumeration(curve),
            )?])?),
        )?);
    }
    common_items.push(super::kmip_item(
        CRYPTOGRAPHIC_PARAMETERS,
        Value::structure(wrapping_crypto_parameters(row)?),
    )?);
    common_items.push(super::common::name_attribute(&format!("{case_id}-kek"))?);
    let common = AttributeSet::try_new(common_items)?;
    let request = kmipkit_protocol::CreateKeyPairRequest::new()
        .with_common_attributes(common)
        .with_private_key_attributes(AttributeSet::try_new([super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(i32::try_from(wrapping_key_usage_mask(
                "PrivateKey",
                "Export",
            )?)?),
        )?])?)
        .with_public_key_attributes(AttributeSet::try_new([super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(i32::try_from(wrapping_key_usage_mask(
                "PublicKey",
                "Export",
            )?)?),
        )?])?);
    let (private_id, public_id) =
        super::common::execute_created_key_pair(client, case_id, request, ownership)?;
    // Key Wrapping Specification uses Wrapping Method=Encrypt, so its
    // Encryption Key Information must name the public half of an asymmetric
    // key pair. The private half is the Unwrap/Decrypt key and is not valid
    // for Export/Get wrapping. Both objects are activated because Cosmian
    // resolves and validates the linked pair during wrapping.
    super::common::ensure_active_object(client, case_id, "PrivateKey", &private_id)?;
    super::common::ensure_active_object(client, case_id, "PublicKey", &public_id)?;
    let wrapping_key_id = rsa_wrapping_key_identifier("Encrypt", &private_id, &public_id)?;
    super::common::ensure_active_object(client, case_id, "PublicKey", wrapping_key_id)?;
    Ok(wrapping_key_id.to_owned())
}

fn rsa_wrapping_key_identifier<'a>(
    wrapping_method: &str,
    private_id: &'a str,
    public_id: &'a str,
) -> Result<&'a str, &'static str> {
    match wrapping_method {
        "Encrypt" => Ok(public_id),
        "Decrypt" => Ok(private_id),
        _ => Err("unsupported asymmetric Key Wrapping Method"),
    }
}

fn wrapping_key_usage_mask(key_type: &str, operation: &str) -> Result<i64, &'static str> {
    match key_type {
        "SymmetricKey" if operation == "Import" => {
            Ok(USAGE_WRAP_KEY | USAGE_ENCRYPT | USAGE_UNWRAP_KEY | USAGE_DECRYPT)
        }
        "SymmetricKey" | "PublicKey" | "Certificate" => Ok(USAGE_WRAP_KEY | USAGE_ENCRYPT),
        "PrivateKey" => Ok(USAGE_UNWRAP_KEY | USAGE_DECRYPT),
        _ => Err("unsupported KMIP wrapping key object type"),
    }
}

fn verify_usage_mask(
    client: &mut Client,
    case_id: &str,
    unique_id: &str,
    expected: i64,
) -> Result<(), Box<dyn Error>> {
    let actual = super::common::execute_case(
        client,
        case_id,
        ClientRequest::get_attributes(GetAttributesRequest::try_new(
            Some(unique_id.to_owned()),
            [AttributeReference::tag(CRYPTOGRAPHIC_USAGE_MASK)],
        )?),
        "Get Attributes Cryptographic Usage Mask verification",
        |response| {
            let result = response
                .get_attributes()
                .ok_or_else(|| "Usage Mask verification returned no typed result".to_owned())?;
            let returned_id = result.unique_identifier().ok_or_else(|| {
                "Usage Mask verification omitted the Unique Identifier".to_owned()
            })?;
            if returned_id != unique_id {
                return Err("Usage Mask verification returned an unexpected object".to_owned());
            }
            let attributes = result
                .attributes()
                .ok_or_else(|| "Usage Mask verification returned no Attributes".to_owned())?;
            attributes
                .as_items()
                .iter()
                .find(|item| item.tag().raw() == CRYPTOGRAPHIC_USAGE_MASK)
                .and_then(|item| {
                    item.with_value(|value| match value {
                        ValueView::Integer(raw) => Some(i64::from(*raw)),
                        _ => None,
                    })
                })
                .ok_or_else(|| {
                    "Usage Mask verification omitted Cryptographic Usage Mask".to_owned()
                })
        },
    )?;
    if actual != expected {
        return Err(format!(
            "case {case_id}: server persisted Cryptographic Usage Mask {actual:#x}, expected {expected:#x}"
        )
        .into());
    }
    Ok(())
}

fn sensitive_attribute_value(attributes: &AttributeSet) -> Option<bool> {
    attributes
        .as_items()
        .iter()
        .find(|item| item.tag().raw() == SENSITIVE)
        .and_then(|item| {
            item.with_value(|value| match value {
                ValueView::Boolean(value) => Some(*value),
                _ => None,
            })
        })
}

fn verify_sensitive_attribute(
    client: &mut Client,
    case_id: &str,
    unique_id: &str,
    expected: bool,
) -> Result<(), Box<dyn Error>> {
    let actual = super::common::execute_case(
        client,
        case_id,
        ClientRequest::get_attributes(GetAttributesRequest::try_new(
            Some(unique_id.to_owned()),
            [AttributeReference::tag(SENSITIVE)],
        )?),
        "Get Attributes Sensitive verification",
        |response| {
            let result = response
                .get_attributes()
                .ok_or_else(|| "Sensitive verification returned no typed result".to_owned())?;
            let returned_id = result
                .unique_identifier()
                .ok_or_else(|| "Sensitive verification omitted the Unique Identifier".to_owned())?;
            if returned_id != unique_id {
                return Err("Sensitive verification returned an unexpected object".to_owned());
            }
            let attributes = result
                .attributes()
                .ok_or_else(|| "Sensitive verification returned no Attributes".to_owned())?;
            sensitive_attribute_value(attributes)
                .ok_or_else(|| "Sensitive verification omitted a Boolean value".to_owned())
        },
    )?;
    if actual != expected {
        return Err(format!(
            "case {case_id}: server returned Sensitive={actual} after Create Key Pair requested Sensitive={expected}"
        )
        .into());
    }
    Ok(())
}

fn wrapping_crypto_parameters(row: &serde_json::Value) -> Result<Structure, Box<dyn Error>> {
    wrapping_crypto_parameters_with_algorithm(row, None)
}

fn wrapping_crypto_parameters_with_algorithm(
    row: &serde_json::Value,
    algorithm: Option<u32>,
) -> Result<Structure, Box<dyn Error>> {
    let parameters = &row["parameters"];
    let mut fields = Structure::new();
    if let Some(mode) = parameters["kek_mode"]
        .as_str()
        .or_else(|| parameters["mode"].as_str())
    {
        let raw = match mode {
            "GCM" => 9,
            "XTS" => 11,
            "AESKeyWrapPadding" => 12,
            "NISTKeyWrap" => 13,
            "GCMSIV" => 0x8000_0002,
            other => return Err(format!("unmapped Block Cipher Mode {other}").into()),
        };
        fields.try_push(super::kmip_item(
            BLOCK_CIPHER_MODE,
            Value::enumeration(raw),
        )?)?;
    }
    let padding = parameters["padding"].as_str();
    if let Some(padding) = padding {
        let raw = match padding {
            "None" => Some(1),
            "OAEP" => Some(2),
            "PKCS1v1.5" | "PKCS1v15" => Some(8),
            other => return Err(format!("unmapped Padding Method {other}").into()),
        };
        if let Some(raw) = raw {
            fields.try_push(super::kmip_item(PADDING_METHOD, Value::enumeration(raw))?)?;
        }
    }
    let hash = parameters["hash"]
        .as_str()
        .map(|name| hash_value(name).ok_or_else(|| format!("unmapped Hashing Algorithm {name}")))
        .transpose()?;
    if let Some(hash) = hash {
        fields.try_push(super::kmip_item(
            HASHING_ALGORITHM,
            Value::enumeration(hash),
        )?)?;
    }
    if let Some(algorithm) = algorithm {
        // OASIS KMIP v2.1 §4.16, Table 59 places Cryptographic Algorithm
        // after Hashing Algorithm and before the IV/tag/salt/MGF fields.
        fields.try_push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(algorithm),
        )?)?;
    }
    if let Some(iv_bits) = parameters["iv_length_bits"].as_i64() {
        fields.try_push(super::kmip_item(
            IV_LENGTH,
            Value::integer(i32::try_from(iv_bits)?),
        )?)?;
    }
    if let Some(tag_bytes) = parameters["tag_length_bytes"].as_i64() {
        fields.try_push(super::kmip_item(
            TAG_LENGTH,
            Value::integer(i32::try_from(tag_bytes)?),
        )?)?;
    }
    if padding == Some("OAEP") {
        let hash = hash.ok_or("OAEP wrapping requires a Hashing Algorithm")?;
        fields.try_push(super::kmip_item(MASK_GENERATOR, Value::enumeration(1))?)?;
        fields.try_push(super::kmip_item(
            MGF_HASHING_ALGORITHM,
            Value::enumeration(hash),
        )?)?;
    }
    Ok(fields)
}

fn wrapping_specification(
    row: &serde_json::Value,
    wrapping_key_id: &str,
) -> Result<Structure, Box<dyn Error>> {
    let method = match row["parameters"]["wrapping_method"].as_str() {
        Some("Encrypt") => 1,
        Some(other) => return Err(format!("unmapped Wrapping Method {other}").into()),
        None => return Err("wrapping row has no Wrapping Method".into()),
    };
    let encoding = match row["parameters"]["wrapping_encoding"].as_str() {
        Some("NoEncoding") => 1,
        Some("TTLVEncoding") => 2,
        Some(other) => return Err(format!("unmapped Encoding Option {other}").into()),
        None => return Err("wrapping row has no Encoding Option".into()),
    };
    let mut key_information = Structure::new();
    key_information.try_push(super::kmip_item(
        UNIQUE_IDENTIFIER,
        Value::text_string(wrapping_key_id.to_owned()),
    )?)?;
    key_information.try_push(super::kmip_item(
        CRYPTOGRAPHIC_PARAMETERS,
        Value::structure(wrapping_specification_crypto_parameters(row)?),
    )?)?;
    super::structure([
        super::kmip_item(WRAPPING_METHOD, Value::enumeration(method))?,
        super::kmip_item(
            ENCRYPTION_KEY_INFORMATION,
            Value::structure(key_information),
        )?,
        super::kmip_item(ENCODING_OPTION, Value::enumeration(encoding))?,
    ])
}

fn wrapping_specification_crypto_parameters(
    row: &serde_json::Value,
) -> Result<Structure, Box<dyn Error>> {
    let key_type = row["parameters"]["wrapping_key_type"]
        .as_str()
        .unwrap_or("SymmetricKey");
    let algorithm_name = if key_type == "SymmetricKey" {
        row["parameters"]["kek_algorithm"]
            .as_str()
            .or_else(|| row["algorithm"].as_str())
            .unwrap_or("AES")
    } else {
        row["algorithm"].as_str().unwrap_or("RSA")
    };
    let algorithm = algorithm_value(algorithm_name)
        .ok_or_else(|| format!("unmapped wrapping-key algorithm {algorithm_name}"))?;
    wrapping_crypto_parameters_with_algorithm(row, Some(algorithm))
}

fn algorithm_value(name: &str) -> Option<u32> {
    match name {
        "AES" => Some(3),
        "RSA" => Some(4),
        "EC" | "ECDH" | "ECDSA" => Some(26),
        "Ed25519" => Some(55),
        "ChaCha20Poly1305" => Some(30),
        "FPE_FF1" => Some(0x8880_0001),
        "ConfigurableKEM" => Some(0x8880_0003),
        other if other.starts_with("HMAC-") => match other {
            "HMAC-SHA-1" => Some(7),
            "HMAC-SHA-224" => Some(8),
            "HMAC-SHA-256" => Some(9),
            "HMAC-SHA-384" => Some(10),
            "HMAC-SHA-512" => Some(11),
            _ => None,
        },
        _ => None,
    }
}

fn hash_value(name: &str) -> Option<u32> {
    match name {
        "SHA-1" => Some(4),
        "SHA-224" => Some(5),
        "SHA-256" => Some(6),
        "SHA-384" => Some(7),
        "SHA-512" => Some(8),
        "SHA3-224" => Some(14),
        "SHA3-256" => Some(15),
        "SHA3-384" => Some(16),
        "SHA3-512" => Some(17),
        _ => None,
    }
}

#[test]
fn cosmian_transfer_manifest_cases_have_typed_executors() {
    assert_manifest_parity();
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_528_transfer_matrix() {
    assert_manifest_parity();
    let mut failures = Vec::new();
    let mut target_requests_sent = 0;
    let mut interoperability_successes = 0;
    let mut expected_finding_observations = 0;
    let selected = selected_case_ids();
    let expected_outcomes = crate::common::manifest_rows(FAMILY)
        .expect("transfer manifest rows")
        .into_iter()
        .map(|row| {
            let case_id =
                crate::common::required_text(&row, "case_id").expect("transfer manifest case ID");
            let expected_outcome = crate::common::required_text(&row, "expected_outcome")
                .expect("transfer manifest expected outcome");
            (case_id.to_owned(), expected_outcome.to_owned())
        })
        .collect::<BTreeMap<_, _>>();
    for case_id in &selected {
        let expected_outcome = expected_outcomes
            .get(*case_id)
            .expect("selected transfer case has an expected outcome");
        match run_case(case_id) {
            Ok(()) => {
                target_requests_sent += 1;
                interoperability_successes += 1;
                if expected_outcome != "typed_success" {
                    expected_finding_observations += 1;
                }
                println!(
                    "{}",
                    transfer_observation_line(case_id, expected_outcome, "typed_success", "true")
                );
            }
            Err(error) => {
                let class = classify_error(error.as_ref());
                let request_sent = target_request_sent_label(class, error.as_ref());
                println!(
                    "{} detail={error}",
                    transfer_observation_line(case_id, expected_outcome, class, request_sent)
                );
                if request_sent == "true" {
                    target_requests_sent += 1;
                    if expected_outcome != "typed_success" {
                        expected_finding_observations += 1;
                    }
                }
                if expected_outcome == "typed_success" || request_sent != "true" {
                    failures.push(case_id.to_owned());
                }
            }
        }
    }
    println!(
        "transfer_matrix selected={} target_requests_sent={} interoperability_successes={} expected_finding_observations={} failures={}",
        selected.len(),
        target_requests_sent,
        interoperability_successes,
        expected_finding_observations,
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "{} of {} transfer cases did not complete successfully; see sanitized per-case output",
        failures.len(),
        selected.len()
    );
}

fn assert_manifest_parity() {
    let manifest = super::common::manifest_rows(FAMILY).expect("transfer manifest rows");
    let ids = case_ids().collect::<Vec<_>>();
    let manifest_ids = manifest
        .iter()
        .filter_map(|row| row["case_id"].as_str())
        .collect::<BTreeSet<_>>();
    let executable_ids = ids.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        ids.len(),
        executable_ids.len(),
        "duplicate transfer case ID"
    );
    assert_eq!(
        executable_ids, manifest_ids,
        "every transfer manifest row must map to exactly one typed executor"
    );
    for row in manifest {
        assert!(
            matches!(
                row["operation"].as_str(),
                Some("Get" | "Export" | "Import" | "Register")
            ),
            "unmapped transfer operation for {}",
            row["case_id"]
        );
        if matches!(row["operation"].as_str(), Some("Get" | "Export"))
            && row["key_format"] == "Cosmian-Pkcs12Legacy"
        {
            assert_eq!(
                row["parameters"]["requested_format_value"],
                row["wire_enum_values"]["key_format_type"]["value"],
                "legacy PKCS#12 Get/Export must request the vendor Key Format Type for {}",
                row["case_id"]
            );
        }
    }
}

fn case_ids() -> impl Iterator<Item = &'static str> {
    CASE_IDS.lines().filter(|line| !line.is_empty())
}

fn configurable_kem_probe_case_ids() -> impl Iterator<Item = &'static str> {
    CONFIGURABLE_KEM_PROBE_CASE_IDS
        .lines()
        .filter(|line| !line.is_empty())
}

fn selected_case_ids() -> Vec<&'static str> {
    let Some(filter) = std::env::var("KMIPKIT_COSMIAN_CASE_FILTER").ok() else {
        return case_ids().collect();
    };
    let selected = filter.split(',').collect::<BTreeSet<_>>();
    let all = case_ids().collect::<BTreeSet<_>>();
    assert!(
        selected.iter().all(|case_id| all.contains(case_id)),
        "KMIPKIT_COSMIAN_CASE_FILTER contains an unknown transfer case ID"
    );
    selected
        .into_iter()
        .map(|case_id| *all.get(case_id).expect("validated transfer case ID"))
        .collect()
}

fn source_is_pfx_fixture(row: &serde_json::Value, operation: &str) -> bool {
    row["fixture"]["encoding"] == "DER/PFX"
        || (matches!(operation, "Get" | "Export")
            && matches!(
                row["key_format"].as_str(),
                Some("PKCS#12" | "Cosmian-Pkcs12Legacy")
            ))
}

// One case owns the full fixture/provision/request/cleanup sequence so object
// ownership and error cleanup remain auditable in the live matrix.
#[allow(clippy::too_many_lines)]
fn run_case(case_id: &str) -> Result<(), Box<dyn Error>> {
    let row = super::common::manifest_row(case_id)?;
    let operation = super::common::required_text(&row, "operation")?;
    let object_name = super::common::required_text(&row, "object_type")?;
    let object_type = object_type(object_name)?;
    let mut client = super::client()?;
    let mut ownership = super::common::CaseOwnership::default();
    let pfx_fixture = source_is_pfx_fixture(&row, operation);
    let fixture_encoding = row["fixture"]["encoding"].as_str();
    let native_symmetric_fixture = object_type.raw() == 2
        && row["parameters"]["raw_material_bytes"].is_number()
        && matches!(
            row["key_format"].as_str(),
            Some("Raw" | "TransparentSymmetricKey")
        );
    let direct_fixture =
        matches!(fixture_encoding, Some("DER" | "PEM")) || native_symmetric_fixture;
    let operation_result = (|| -> Result<(), Box<dyn Error>> {
        let wrapping = row["parameters"]["wrapping_encoding"].is_string();
        let wrapping_key_id = if wrapping {
            Some(provision_wrapping_key(
                &mut client,
                case_id,
                &row,
                &mut ownership,
            )?)
        } else {
            None
        };
        match operation {
            "Get" | "Export" => {
                let mut password_id = None;
                let source_id = if wrapping {
                    provision_wrapped_target(
                        &mut client,
                        case_id,
                        &row,
                        object_type,
                        &mut ownership,
                    )
                } else if pfx_fixture {
                    let imported = import_pfx_source(&mut client, case_id, &row, &mut ownership)?;
                    password_id = imported.password_id;
                    if object_type.raw() == 1 {
                        pfx_certificate_identifier(
                            &mut client,
                            case_id,
                            &imported.private_id,
                            &mut ownership,
                        )
                    } else {
                        Ok(imported.private_id)
                    }
                } else if direct_fixture {
                    register_fixture_source(&mut client, case_id, &row, object_type, &mut ownership)
                } else {
                    super::common::provision_object(
                        &mut client,
                        object_name,
                        case_id,
                        &row,
                        &mut ownership,
                    )
                    .map(|source| source.primary)
                }?;
                match operation {
                    "Get" => get_case(
                        &mut client,
                        case_id,
                        &row,
                        object_type,
                        &source_id,
                        wrapping_key_id.as_deref(),
                        password_id.as_deref(),
                    ),
                    "Export" => export_case(
                        &mut client,
                        case_id,
                        &row,
                        object_type,
                        &source_id,
                        wrapping_key_id.as_deref(),
                        password_id.as_deref(),
                    ),
                    _ => unreachable!(),
                }
            }
            "Import" if wrapping => {
                let source = provision_wrapped_target(
                    &mut client,
                    case_id,
                    &row,
                    object_type,
                    &mut ownership,
                )?;
                import_case(
                    &mut client,
                    case_id,
                    &row,
                    object_type,
                    &source,
                    wrapping_key_id.as_deref(),
                    &mut ownership,
                )
            }
            "Register" if wrapping => {
                let source = provision_wrapped_target(
                    &mut client,
                    case_id,
                    &row,
                    object_type,
                    &mut ownership,
                )?;
                register_case(
                    &mut client,
                    case_id,
                    &row,
                    object_type,
                    &source,
                    wrapping_key_id.as_deref(),
                    &mut ownership,
                )
            }
            "Import" if pfx_fixture => {
                import_pfx_case(&mut client, case_id, &row, object_type, &mut ownership)
            }
            "Import" if direct_fixture => {
                import_fixture_case(&mut client, case_id, &row, object_type, &mut ownership)
            }
            "Register" if pfx_fixture => {
                register_pfx_case(&mut client, case_id, &row, object_type, &mut ownership)
            }
            "Register" if direct_fixture => {
                register_fixture_case(&mut client, case_id, &row, object_type, &mut ownership)
            }
            "Import" | "Register" => {
                let source = super::common::provision_object(
                    &mut client,
                    object_name,
                    case_id,
                    &row,
                    &mut ownership,
                )?;
                match operation {
                    "Import" => import_case(
                        &mut client,
                        case_id,
                        &row,
                        object_type,
                        &source.primary,
                        None,
                        &mut ownership,
                    ),
                    "Register" => register_case(
                        &mut client,
                        case_id,
                        &row,
                        object_type,
                        &source.primary,
                        None,
                        &mut ownership,
                    ),
                    _ => unreachable!(),
                }
            }
            other => Err(format!("case {case_id}: unsupported transfer operation {other}").into()),
        }
    })();
    let operation_result = operation_result.map_err(|error| error.to_string());
    super::common::finish_operation_with_cleanup(operation_result, || {
        super::common::cleanup_owned_objects(&mut client, case_id, &mut ownership)
    })
    .map_err(|error| -> Box<dyn Error> { error.into() })
}

fn get_case(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    source_id: &str,
    wrapping_key_id: Option<&str>,
    password_id: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let expected_object_type = response_object_type(object_type, row);
    let request = build_get_request(row, case_id, source_id, wrapping_key_id, password_id)?;
    super::common::execute_case(
        client,
        case_id,
        ClientRequest::get(request),
        "Get",
        |response| {
            let result = response
                .get()
                .ok_or_else(|| "KMIPKit did not return a typed Get result".to_owned())?;
            if result.object_type() != Some(expected_object_type) || result.object().is_none() {
                return Err("Get returned a missing or mismatched typed object".to_owned());
            }
            require_format(result.object(), row)
        },
    )
}

fn export_case(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    source_id: &str,
    wrapping_key_id: Option<&str>,
    password_id: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let expected_object_type = response_object_type(object_type, row);
    let request = build_export_request(row, case_id, source_id, wrapping_key_id, password_id)?;
    super::common::execute_case(
        client,
        case_id,
        ClientRequest::export(request),
        "Export",
        |response| {
            let result = response
                .export()
                .ok_or_else(|| "KMIPKit did not return a typed Export result".to_owned())?;
            if result.object_type() != Some(expected_object_type) || result.object().is_none() {
                return Err("Export returned a missing or mismatched typed object".to_owned());
            }
            require_format(result.object(), row)
        },
    )
}

fn import_case(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    source_id: &str,
    wrapping_key_id: Option<&str>,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let exported = export_object(client, case_id, row, source_id, wrapping_key_id)?;
    let target_id = format!("{case_id}-imported");
    // Import requires the caller-selected identifier; track it before sending
    // so a possibly-sent request is eligible for one state-aware cleanup pass.
    ownership.track(target_id.clone());
    let request = build_import_request(
        &target_id,
        object_type,
        attributes_for(row, object_type)?,
        exported,
        wrapping_key_id.is_some(),
    );
    let imported_id = super::common::execute_case(
        client,
        case_id,
        ClientRequest::import(request),
        "Import",
        |response| {
            let result = response
                .import()
                .ok_or_else(|| "KMIPKit did not return a typed Import result".to_owned())?;
            let imported_id = import_response_identifier(result.unique_identifier())?;
            ownership.track(imported_id.clone());
            Ok(imported_id)
        },
    )?;
    if imported_id != target_id {
        println!("case_id={case_id} event=import_identifier server_assigned=true");
    }
    Ok(())
}

fn import_pfx_case(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    import_pfx_with_suffix(
        client,
        case_id,
        row,
        object_type,
        ownership,
        ImportTarget::Imported,
    )?;
    Ok(())
}

fn import_pfx_source(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<PfxImportResult, Box<dyn Error>> {
    let mut source_row = row.clone();
    source_row["wire_enum_values"]["key_format_type"]["value"] =
        serde_json::Value::String("0x00000016".to_owned());
    import_pfx_with_suffix(
        client,
        case_id,
        &source_row,
        ObjectType::from_raw(4),
        ownership,
        ImportTarget::PfxSource,
    )
}

fn import_pfx_with_suffix(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    ownership: &mut super::common::CaseOwnership,
    target: ImportTarget,
) -> Result<PfxImportResult, Box<dyn Error>> {
    if object_type.raw() != 4 {
        return Err(format!("case {case_id}: PFX fixture must be imported as PrivateKey").into());
    }
    let (object, attributes, password_id) =
        pfx_object_and_attributes(client, case_id, row, ownership)?;
    let private_id = import_object(
        client,
        case_id,
        object_type,
        attributes,
        object,
        ownership,
        target,
    )?;
    Ok(PfxImportResult {
        private_id,
        password_id,
    })
}

fn pfx_object_and_attributes(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(Item, AttributeSet, Option<String>), Box<dyn Error>> {
    let password_link = row["parameters"]["password_link"]
        .as_bool()
        .unwrap_or(false);
    let has_chain = row["parameters"]["certificate_chain"]
        .as_bool()
        .unwrap_or(false);
    let pfx = std::fs::read(pfx_fixture_path(row, password_link, has_chain)?)?;
    let key_format = enum_value(row, "key_format_type")
        .ok_or("PKCS#12 row has no normative Key Format Type value")?;
    let mut key_value = Structure::new();
    key_value.try_push(super::kmip_item(
        KEY_MATERIAL,
        SecretBytes::new(pfx).into_ttlv_value(),
    )?)?;
    let mut key_block = Structure::new();
    key_block.try_push(super::kmip_item(
        KEY_FORMAT_TYPE,
        Value::enumeration(key_format),
    )?)?;
    key_block.try_push(super::kmip_item(KEY_VALUE, Value::structure(key_value))?)?;
    if let Some(algorithm) = enum_value(row, "cryptographic_algorithm") {
        key_block.try_push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(algorithm),
        )?)?;
    }
    let object = super::kmip_item(
        PRIVATE_KEY,
        Value::structure(super::structure([super::kmip_item(
            KEY_BLOCK,
            Value::structure(key_block),
        )?])?),
    )?;

    let mut attributes = attributes_for(row, ObjectType::from_raw(4))?.into_items();
    let password_id = if password_link {
        let password_case_id = format!("{case_id}-pkcs12-password");
        let password_id = super::common::register_secret_data_with_material(
            client,
            &password_case_id,
            SecretBytes::new(PKCS12_TEST_PASSWORD.as_bytes().to_vec()),
            ownership,
        )?;
        attributes.push(pkcs12_password_link_attribute(&password_id)?);
        Some(password_id)
    } else {
        None
    };
    Ok((object, AttributeSet::try_new(attributes)?, password_id))
}

struct PfxImportResult {
    private_id: String,
    password_id: Option<String>,
}

#[derive(Clone, Copy)]
enum ImportTarget {
    PfxSource,
    Imported,
}

impl ImportTarget {
    const fn suffix(self) -> &'static str {
        match self {
            Self::PfxSource => "-pfx-source",
            Self::Imported => "-imported",
        }
    }

    const fn operation_label(self) -> &'static str {
        match self {
            Self::PfxSource => "Import setup",
            Self::Imported => "Import",
        }
    }
}

fn import_object(
    client: &mut Client,
    case_id: &str,
    object_type: ObjectType,
    attributes: AttributeSet,
    object: Item,
    ownership: &mut super::common::CaseOwnership,
    target: ImportTarget,
) -> Result<String, Box<dyn Error>> {
    let target_id = format!("{}{suffix}", case_id, suffix = target.suffix());
    // Import requires the caller-selected identifier; track it before sending
    // so a possibly-sent request is eligible for one state-aware cleanup pass.
    ownership.track(target_id.clone());
    let request = build_import_request(&target_id, object_type, attributes, object, false);
    let imported_id = super::common::execute_case(
        client,
        case_id,
        ClientRequest::import(request),
        target.operation_label(),
        |response| {
            let result = response
                .import()
                .ok_or_else(|| "KMIPKit did not return a typed Import result".to_owned())?;
            let imported_id = import_response_identifier(result.unique_identifier())?;
            ownership.track(imported_id.clone());
            Ok(imported_id)
        },
    )?;
    if imported_id != target_id {
        println!("case_id={case_id} event=import_identifier server_assigned=true");
    }
    pfx_linked_object_ids(client, case_id, &imported_id, ownership)?;
    Ok(imported_id)
}

fn register_pfx_case(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let (object, attributes, _password_id) =
        pfx_object_and_attributes(client, case_id, row, ownership)?;
    let registered_id = register_object(
        client,
        case_id,
        object_type,
        attributes,
        object,
        ownership,
        "Register",
    )?;
    pfx_linked_object_ids(client, case_id, &registered_id, ownership)?;
    Ok(())
}

fn pfx_fixture_path(
    row: &serde_json::Value,
    password_link: bool,
    has_chain: bool,
) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let stem = fixture_stem(row)?;
    let suffix = match (password_link, has_chain) {
        (true, true) => "password-chain",
        (true, false) => "password-no-chain",
        (false, true) => "empty-password-chain",
        (false, false) => "empty-password-no-chain",
    };
    Ok(super::certificate_directory()
        .join("validation")
        .join(format!("{stem}-{suffix}.p12")))
}

fn pfx_certificate_identifier(
    client: &mut Client,
    case_id: &str,
    private_id: &str,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    pfx_linked_object_ids(client, case_id, private_id, ownership)?
        .into_iter()
        .find_map(|(link_type, linked_id)| {
            (link_type == PKCS12_CERTIFICATE_LINK).then_some(linked_id)
        })
        .ok_or_else(|| "PFX private key has no PKCS#12 Certificate Link".into())
}

fn pfx_linked_object_ids(
    client: &mut Client,
    case_id: &str,
    private_id: &str,
    ownership: &mut super::common::CaseOwnership,
) -> Result<Vec<(u32, String)>, Box<dyn Error>> {
    let request = GetAttributesRequest::try_new(
        Some(private_id.to_owned()),
        [AttributeReference::tag(LINK)],
    )?;
    super::common::execute_case(
        client,
        case_id,
        ClientRequest::get_attributes(request),
        "PFX link inspection",
        |response| {
            let attributes = response
                .get_attributes()
                .and_then(|result| result.attributes())
                .ok_or_else(|| "PFX source has no typed Link attributes".to_owned())?;
            let mut linked_objects = Vec::new();
            for link in attributes
                .as_items()
                .iter()
                .filter(|item| item.tag().raw() == LINK)
            {
                let (link_type, linked_id) = link_parts(link)?;
                ownership.track(linked_id.clone());
                linked_objects.push((link_type, linked_id));
            }
            Ok(linked_objects)
        },
    )
}

fn link_parts(link: &Item) -> Result<(u32, String), String> {
    let mut link_type = None;
    let mut linked_identifier = None;
    link.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            return;
        };
        for child in structure.children() {
            match child.tag().raw() {
                LINK_TYPE => child.with_value(|value| {
                    if let ValueView::Enumeration(raw) = value {
                        link_type = Some(*raw);
                    }
                }),
                LINKED_OBJECT_IDENTIFIER => child.with_value(|value| {
                    if let ValueView::TextString(raw) = value {
                        linked_identifier = Some((*raw).to_owned());
                    }
                }),
                _ => {}
            }
        }
    });
    Ok((
        link_type.ok_or_else(|| "PFX Link omitted its Link Type".to_owned())?,
        linked_identifier
            .ok_or_else(|| "PFX Link omitted its Linked Object Identifier".to_owned())?,
    ))
}

fn register_case(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    source_id: &str,
    wrapping_key_id: Option<&str>,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let exported = export_object(client, case_id, row, source_id, wrapping_key_id)?;
    let request = build_register_request(object_type, attributes_for(row, object_type)?, exported);
    super::common::execute_case(
        client,
        case_id,
        ClientRequest::register(request),
        "Register",
        |response| {
            let result = response
                .register()
                .ok_or_else(|| "KMIPKit did not return a typed Register result".to_owned())?;
            let identifier = result
                .unique_identifier()
                .ok_or_else(|| "Register omitted its Unique Identifier".to_owned())?;
            let owned = super::text_identifier(identifier)
                .map_err(|_| "Register returned a non-text Unique Identifier".to_owned())?;
            ownership.track(owned);
            Ok(())
        },
    )
}

fn register_fixture_source(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    ownership: &mut super::common::CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let object = fixture_object(row, object_type)?;
    register_object(
        client,
        case_id,
        object_type,
        attributes_for(row, object_type)?,
        object,
        ownership,
        "Register setup",
    )
}

fn import_fixture_case(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let object = fixture_object(row, object_type)?;
    import_object(
        client,
        case_id,
        object_type,
        attributes_for(row, object_type)?,
        object,
        ownership,
        ImportTarget::Imported,
    )
    .map(|_| ())
}

fn register_fixture_case(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    object_type: ObjectType,
    ownership: &mut super::common::CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let object = fixture_object(row, object_type)?;
    register_object(
        client,
        case_id,
        object_type,
        attributes_for(row, object_type)?,
        object,
        ownership,
        "Register",
    )
    .map(|_| ())
}

fn register_object(
    client: &mut Client,
    case_id: &str,
    object_type: ObjectType,
    attributes: AttributeSet,
    object: Item,
    ownership: &mut super::common::CaseOwnership,
    operation_label: &str,
) -> Result<String, Box<dyn Error>> {
    let request = build_register_request(object_type, attributes, object);
    super::common::execute_case(
        client,
        case_id,
        ClientRequest::register(request),
        operation_label,
        |response| {
            let result = response
                .register()
                .ok_or_else(|| "KMIPKit did not return a typed Register result".to_owned())?;
            let identifier = result
                .unique_identifier()
                .ok_or_else(|| "Register omitted its Unique Identifier".to_owned())?;
            let identifier = super::text_identifier(identifier)
                .map_err(|_| "Register returned a non-text Unique Identifier".to_owned())?;
            ownership.track(identifier.clone());
            Ok(identifier)
        },
    )
}

// Keep the three supported fixture object encodings adjacent so key material
// wrapping and secrecy decisions can be reviewed in the full construction.
#[allow(clippy::too_many_lines)]
fn fixture_object(
    row: &serde_json::Value,
    object_type: ObjectType,
) -> Result<Item, Box<dyn Error>> {
    if object_type.raw() == 1 {
        let certificate = fixture_material_path(row, FixtureMaterial::Certificate)?;
        let der = fixture_bytes(row, &certificate)?;
        return super::kmip_item(
            CERTIFICATE,
            Value::structure(super::structure([
                super::kmip_item(CERTIFICATE_TYPE, Value::enumeration(1))?,
                super::kmip_item(CERTIFICATE_VALUE, Value::byte_string(der))?,
            ])?),
        );
    }
    if object_type.raw() == 2 {
        let format = enum_value(row, "key_format_type")
            .ok_or("symmetric fixture row has no wire Key Format Type")?;
        let length = row["parameters"]["raw_material_bytes"]
            .as_u64()
            .ok_or("symmetric fixture row has no raw_material_bytes")?;
        let material = SecretBytes::new(vec![0x5A; usize::try_from(length)?]);
        let key_material = if format == 7 {
            Value::structure(super::structure([super::kmip_item(
                KEY,
                material.into_ttlv_value(),
            )?])?)
        } else {
            material.into_ttlv_value()
        };
        let mut key_block = Structure::new();
        key_block.try_push(super::kmip_item(
            KEY_FORMAT_TYPE,
            Value::enumeration(format),
        )?)?;
        key_block.try_push(super::kmip_item(
            KEY_VALUE,
            Value::structure(super::structure([super::kmip_item(
                KEY_MATERIAL,
                key_material,
            )?])?),
        )?)?;
        if let Some(algorithm) = enum_value(row, "cryptographic_algorithm") {
            key_block.try_push(super::kmip_item(
                CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(algorithm),
            )?)?;
        }
        if let Some(bits) = cryptographic_length_bits(row) {
            key_block.try_push(super::kmip_item(
                CRYPTOGRAPHIC_LENGTH,
                Value::integer(i32::try_from(bits)?),
            )?)?;
        }
        return super::kmip_item(
            SYMMETRIC_KEY,
            Value::structure(super::structure([super::kmip_item(
                KEY_BLOCK,
                Value::structure(key_block),
            )?])?),
        );
    }
    if !matches!(object_type.raw(), 3 | 4) {
        return Err(format!(
            "fixture object builder does not map Object Type {}",
            object_type.raw()
        )
        .into());
    }
    let material_path = fixture_material_path(
        row,
        if object_type.raw() == 4 {
            FixtureMaterial::PrivateKey
        } else {
            FixtureMaterial::PublicKey
        },
    )?;
    let material = fixture_bytes(row, &material_path)?;
    let format = enum_value(row, "key_format_type")
        .ok_or("fixture row has no wire Key Format Type value")?;
    let mut key_value = Structure::new();
    key_value.try_push(super::kmip_item(
        KEY_MATERIAL,
        if object_type.raw() == 4 {
            SecretBytes::new(material).into_ttlv_value()
        } else {
            Value::byte_string(material)
        },
    )?)?;
    let mut key_block = Structure::new();
    key_block.try_push(super::kmip_item(
        KEY_FORMAT_TYPE,
        Value::enumeration(format),
    )?)?;
    key_block.try_push(super::kmip_item(KEY_VALUE, Value::structure(key_value))?)?;
    if let Some(algorithm) = enum_value(row, "cryptographic_algorithm") {
        key_block.try_push(super::kmip_item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(algorithm),
        )?)?;
    }
    if let Some(bits) = row["parameters"]["key_bits"].as_i64() {
        key_block.try_push(super::kmip_item(
            CRYPTOGRAPHIC_LENGTH,
            Value::integer(i32::try_from(bits)?),
        )?)?;
    }
    let object_tag = if object_type.raw() == 4 {
        PRIVATE_KEY
    } else {
        PUBLIC_KEY
    };
    super::kmip_item(
        object_tag,
        Value::structure(super::structure([super::kmip_item(
            KEY_BLOCK,
            Value::structure(key_block),
        )?])?),
    )
}

#[derive(Clone, Copy)]
enum FixtureMaterial {
    Certificate,
    PrivateKey,
    PublicKey,
}

fn fixture_material_path(
    row: &serde_json::Value,
    material: FixtureMaterial,
) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let stem = fixture_stem(row)?;
    let encoding = row["fixture"]["encoding"].as_str().unwrap_or("DER");
    let format = row["key_format"].as_str().unwrap_or("");
    let suffix = match material {
        FixtureMaterial::Certificate => match encoding {
            "DER" | "PEM" => "certificate",
            other => return Err(format!("unsupported certificate fixture encoding {other}").into()),
        },
        FixtureMaterial::PrivateKey => match format {
            "PKCS#1" => "private-pkcs1",
            "PKCS#8" => "private-pkcs8",
            "ECPrivateKey" => "private-ec",
            other => return Err(format!("no private-key fixture for format {other}").into()),
        },
        FixtureMaterial::PublicKey => match format {
            "PKCS#1" => "public-pkcs1",
            "PKCS#8" => "public-spki",
            other => return Err(format!("no public-key fixture for format {other}").into()),
        },
    };
    let extension = if encoding == "PEM" { "pem" } else { "der" };
    let filename = match material {
        FixtureMaterial::Certificate if encoding == "PEM" => format!("{stem}.crt"),
        FixtureMaterial::PrivateKey if encoding == "PEM" && format == "PKCS#8" => {
            format!("{stem}.key")
        }
        FixtureMaterial::PrivateKey if encoding == "PEM" && format == "ECPrivateKey" => {
            format!("{stem}-private-ec.pem")
        }
        FixtureMaterial::PublicKey if encoding == "PEM" && format == "PKCS#8" => {
            format!("{stem}-public-spki.pem")
        }
        FixtureMaterial::PublicKey if encoding == "PEM" && format == "PKCS#1" => {
            format!("{stem}-public-pkcs1.pem")
        }
        _ => format!("{stem}-{suffix}.{extension}"),
    };
    Ok(super::certificate_directory()
        .join("validation")
        .join(filename))
}

fn fixture_stem(row: &serde_json::Value) -> Result<String, Box<dyn Error>> {
    let algorithm = match row["algorithm"].as_str().unwrap_or("") {
        "ECDH" | "ECDSA" => "EC",
        value => value,
    };
    let stem = match algorithm {
        "RSA" => format!(
            "rsa-{}",
            row["parameters"]["key_bits"].as_i64().unwrap_or(2048)
        ),
        "EC" => match row["parameters"]["curve"].as_str().unwrap_or("P256") {
            "CURVE25519" => "x25519".to_owned(),
            "CURVE448" => "x448".to_owned(),
            "CURVEED25519" => "ed25519".to_owned(),
            "CURVEED448" => "ed448".to_owned(),
            curve => format!("ec-{curve}"),
        },
        "DSA" => format!(
            "dsa-{}",
            row["parameters"]["key_bits"].as_i64().unwrap_or(2048)
        ),
        "Ed25519" => "ed25519".to_owned(),
        "Ed448" => "ed448".to_owned(),
        other => return Err(format!("no fixture generator for algorithm {other}").into()),
    };
    Ok(stem)
}

fn fixture_bytes(
    row: &serde_json::Value,
    path: &std::path::Path,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match row["fixture"]["encoding"].as_str() {
        Some("PEM") => super::common::read_pem_der(path),
        Some("DER") => Ok(std::fs::read(path)?),
        other => Err(format!("fixture bytes require DER or PEM, received {other:?}").into()),
    }
}

fn export_object(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    source_id: &str,
    wrapping_key_id: Option<&str>,
) -> Result<Item, Box<dyn Error>> {
    let request = build_export_request(row, case_id, source_id, wrapping_key_id, None)?;
    super::common::execute_case(
        client,
        case_id,
        ClientRequest::export(request),
        "Export setup",
        |response| {
            response
                .export()
                .and_then(|result| result.object())
                .ok_or_else(|| "Export setup returned no object".to_owned())?
                .try_clone()
                .map_err(|_| "Export setup object could not be retained".to_owned())
        },
    )
}

fn with_requested_format<T>(request: T, row: &serde_json::Value) -> T
where
    T: FormatSelector,
{
    match requested_format_raw(row) {
        Some(raw) => request.with_format(raw),
        None => request,
    }
}

fn pkcs12_password_link_attribute(password_id: &str) -> Result<Item, Box<dyn Error>> {
    super::kmip_item(
        LINK,
        Value::structure(super::structure([
            super::kmip_item(LINK_TYPE, Value::enumeration(0x10D))?,
            super::kmip_item(
                LINKED_OBJECT_IDENTIFIER,
                Value::text_string(password_id.to_owned()),
            )?,
        ])?),
    )
}

fn pkcs12_password_wrapping_specification(password_id: &str) -> Result<Structure, Box<dyn Error>> {
    let key_information = super::structure([super::kmip_item(
        UNIQUE_IDENTIFIER,
        Value::text_string(password_id.to_owned()),
    )?])?;
    super::structure([
        super::kmip_item(WRAPPING_METHOD, Value::enumeration(1))?,
        super::kmip_item(
            ENCRYPTION_KEY_INFORMATION,
            Value::structure(key_information),
        )?,
        super::kmip_item(ENCODING_OPTION, Value::enumeration(2))?,
    ])
}

fn with_pkcs12_password_specification<T: WrappingSelector>(
    request: T,
    row: &serde_json::Value,
    password_id: Option<&str>,
) -> Result<T, Box<dyn Error>> {
    if !row["parameters"]["password_link"]
        .as_bool()
        .unwrap_or(false)
    {
        return Ok(request);
    }
    if row["parameters"]["wrapping_encoding"].is_string() {
        return Err(
            "PKCS#12 password selection cannot share a KeyWrappingSpecification with key wrapping"
                .into(),
        );
    }
    let password_id = password_id
        .ok_or("password-linked PKCS#12 request has no case-owned SecretData identifier")?;
    Ok(request.with_wrapping(pkcs12_password_wrapping_specification(password_id)?))
}

fn requested_format_raw(row: &serde_json::Value) -> Option<u32> {
    if let Some(value) = row["parameters"]["requested_format_value"].as_str() {
        return value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
            .and_then(|digits| u32::from_str_radix(digits, 16).ok());
    }
    match row["parameters"]["requested_key_format"].as_str() {
        Some("Opaque") | None => None,
        Some(_) => enum_value(row, "key_format_type"),
    }
    .or_else(|| {
        (matches!(row["operation"].as_str(), Some("Get" | "Export"))
            && row["key_format"] == "Cosmian-PKCS7")
            .then(|| enum_value(row, "key_format_type"))
            .flatten()
    })
}

fn with_wrapping_specification<T>(
    request: T,
    row: &serde_json::Value,
    wrapping_key_id: Option<&str>,
) -> Result<T, Box<dyn Error>>
where
    T: WrappingSelector,
{
    if !row["parameters"]["wrapping_encoding"].is_string() {
        return Ok(request);
    }
    let wrapping_key_id = wrapping_key_id
        .ok_or_else(|| "wrapping row has no independently provisioned case-owned KEK".to_owned())?;
    Ok(request.with_wrapping(wrapping_specification(row, wrapping_key_id)?))
}

trait FormatSelector {
    fn with_format(self, raw: u32) -> Self;
}

trait WrappingSelector {
    fn with_wrapping(self, specification: Structure) -> Self;
}

impl FormatSelector for GetRequest {
    fn with_format(self, raw: u32) -> Self {
        self.with_key_format_type(KeyFormatType::from_raw(raw))
    }
}

impl FormatSelector for ExportRequest {
    fn with_format(self, raw: u32) -> Self {
        self.with_key_format_type(KeyFormatType::from_raw(raw))
    }
}

impl WrappingSelector for GetRequest {
    fn with_wrapping(self, specification: Structure) -> Self {
        self.with_key_wrapping_specification(specification)
    }
}

impl WrappingSelector for ExportRequest {
    fn with_wrapping(self, specification: Structure) -> Self {
        self.with_key_wrapping_specification(specification)
    }
}

fn build_get_request(
    row: &serde_json::Value,
    case_id: &str,
    source_id: &str,
    wrapping_key_id: Option<&str>,
    password_id: Option<&str>,
) -> Result<GetRequest, Box<dyn Error>> {
    let request = with_pkcs12_password_specification(
        with_requested_format(
            GetRequest::new().with_unique_identifier(identifier(source_id)),
            row,
        ),
        row,
        password_id,
    )?;
    let wrapping_key_id = wrapping_key_id.map(str::to_owned).or_else(|| {
        row["parameters"]["wrapping_encoding"]
            .as_str()
            .map(|_| format!("{case_id}-kek"))
    });
    with_wrapping_specification(request, row, wrapping_key_id.as_deref())
}

fn build_export_request(
    row: &serde_json::Value,
    case_id: &str,
    source_id: &str,
    wrapping_key_id: Option<&str>,
    password_id: Option<&str>,
) -> Result<ExportRequest, Box<dyn Error>> {
    let request = with_pkcs12_password_specification(
        with_requested_format(
            ExportRequest::new().with_unique_identifier(identifier(source_id)),
            row,
        ),
        row,
        password_id,
    )?;
    let wrapping_key_id = wrapping_key_id.map(str::to_owned).or_else(|| {
        row["parameters"]["wrapping_encoding"]
            .as_str()
            .map(|_| format!("{case_id}-kek"))
    });
    with_wrapping_specification(request, row, wrapping_key_id.as_deref())
}

fn build_import_request(
    target_id: &str,
    object_type: ObjectType,
    attributes: AttributeSet,
    object: Item,
    has_wrapping_key: bool,
) -> ImportRequest {
    let request = ImportRequest::new(identifier(target_id), object_type, attributes, object);
    if has_wrapping_key && matches!(object_type.raw(), 2 | 3 | 4 | 5 | 9) {
        request.with_key_wrap_type(KeyWrapType::from_raw(2))
    } else {
        request
    }
}

fn build_register_request(
    object_type: ObjectType,
    attributes: AttributeSet,
    object: Item,
) -> RegisterRequest {
    let request = RegisterRequest::new(object_type, attributes, object);
    if object_type.raw() == 8 {
        request.with_opaque_object_context(OpaqueObjectContext::Generic)
    } else {
        request
    }
}

fn build_case_request(
    row: &serde_json::Value,
    case_id: &str,
    object_type: ObjectType,
    object: Item,
    attributes: AttributeSet,
) -> Result<ClientRequest, Box<dyn Error>> {
    let operation = super::common::required_text(row, "operation")?;
    let password_id = row["parameters"]["password_link"]
        .as_bool()
        .filter(|enabled| *enabled)
        .map(|_| format!("{case_id}-password-test-id"));
    match operation {
        "Get" => Ok(ClientRequest::get(build_get_request(
            row,
            case_id,
            case_id,
            None,
            password_id.as_deref(),
        )?)),
        "Export" => Ok(ClientRequest::export(build_export_request(
            row,
            case_id,
            case_id,
            None,
            password_id.as_deref(),
        )?)),
        "Import" => Ok(ClientRequest::import(build_import_request(
            &format!("{case_id}-imported"),
            object_type,
            attributes,
            object,
            row["parameters"]["wrapping_encoding"].is_string(),
        ))),
        "Register" => Ok(ClientRequest::register(build_register_request(
            object_type,
            attributes,
            object,
        ))),
        other => Err(format!("case {case_id}: unsupported transfer operation {other}").into()),
    }
}

fn encode_case_request(request: ClientRequest) -> Result<Structure, Box<dyn Error>> {
    match request {
        ClientRequest::Get(request) => Ok(request.to_ttlv_payload()?),
        ClientRequest::Export(request) => Ok(request.into_ttlv_payload()?),
        ClientRequest::Import(request) => Ok(request.into_ttlv_payload()?),
        ClientRequest::Register(request) => Ok(request.into_ttlv_payload()?),
        _ => Err("transfer inventory produced an unexpected request variant".into()),
    }
}

fn constructor_attributes(
    row: &serde_json::Value,
    object_type: ObjectType,
) -> Result<AttributeSet, Box<dyn Error>> {
    if object_type.raw() != 1 {
        return attributes_for(row, object_type);
    }
    AttributeSet::try_new([
        super::kmip_item(CERTIFICATE_LENGTH, Value::integer(1))?,
        super::common::name_attribute(
            row["case_id"]
                .as_str()
                .unwrap_or("kmipkit-transfer-inventory"),
        )?,
    ])
    .map_err(Into::into)
}

fn constructor_object(
    row: &serde_json::Value,
    object_type: ObjectType,
) -> Result<Item, Box<dyn Error>> {
    match object_type.raw() {
        1 => super::kmip_item(
            CERTIFICATE,
            Value::structure(super::structure([
                super::kmip_item(CERTIFICATE_TYPE, Value::enumeration(1))?,
                super::kmip_item(CERTIFICATE_VALUE, Value::byte_string(vec![0]))?,
            ])?),
        ),
        7 => super::kmip_item(
            SECRET_DATA,
            Value::structure(super::structure([
                super::kmip_item(SECRET_DATA_TYPE, Value::enumeration(1))?,
                super::kmip_item(SECRET_DATA_VALUE, Value::byte_string(vec![0]))?,
            ])?),
        ),
        8 => super::kmip_item(
            OPAQUE_OBJECT,
            Value::structure(super::structure([
                super::kmip_item(OPAQUE_DATA_TYPE, Value::enumeration(0x8000_0001))?,
                super::kmip_item(OPAQUE_DATA_VALUE, Value::byte_string(vec![0]))?,
            ])?),
        ),
        raw => {
            let format = enum_value(row, "key_format_type").unwrap_or(2);
            let mut key_block = Structure::new();
            key_block.try_push(super::kmip_item(
                KEY_FORMAT_TYPE,
                Value::enumeration(format),
            )?)?;
            let mut key_value = Structure::new();
            key_value.try_push(super::kmip_item(
                KEY_MATERIAL,
                SecretBytes::new(vec![0]).into_ttlv_value(),
            )?)?;
            key_block.try_push(super::kmip_item(KEY_VALUE, Value::structure(key_value))?)?;
            if let Some(algorithm) = enum_value(row, "cryptographic_algorithm") {
                key_block.try_push(super::kmip_item(
                    CRYPTOGRAPHIC_ALGORITHM,
                    Value::enumeration(algorithm),
                )?)?;
            }
            if let Some(bits) = cryptographic_length_bits(row).or_else(|| {
                // Constructor-only placeholder: CoverCrypt rows have no
                // manifest key length, but Register's typed validator requires
                // the algorithm/length pair in the supplied Key Block. This
                // object is serialized offline and is never sent to a server.
                enum_value(row, "cryptographic_algorithm").map(|_| 1)
            }) {
                key_block.try_push(super::kmip_item(
                    CRYPTOGRAPHIC_LENGTH,
                    Value::integer(i32::try_from(bits)?),
                )?)?;
            }
            if row["parameters"]["wrapping_encoding"].is_string() {
                let case_id = row["case_id"]
                    .as_str()
                    .ok_or("wrapped constructor row has no case ID")?;
                let wrapping_data = wrapping_specification(row, &format!("{case_id}-kek"))?;
                key_block.try_push(super::kmip_item(
                    KEY_WRAPPING_DATA,
                    Value::structure(wrapping_data),
                )?)?;
            }
            let object_tag = match raw {
                2 | 5 => SYMMETRIC_KEY,
                3 => PUBLIC_KEY,
                4 => PRIVATE_KEY,
                other => {
                    return Err(format!("no constructor object tag for Object Type {other}").into());
                }
            };
            super::kmip_item(
                object_tag,
                Value::structure(super::structure([super::kmip_item(
                    KEY_BLOCK,
                    Value::structure(key_block),
                )?])?),
            )
        }
    }
}

fn require_format(object: Option<&Item>, row: &serde_json::Value) -> Result<(), String> {
    let Some(object) = object else {
        return Err("typed response omitted the requested object".to_owned());
    };
    if row["object_type"] == "OpaqueObject" {
        return if opaque_object_preserves_case_payload(object, row) {
            Ok(())
        } else {
            Err(
                "OpaqueObject response did not preserve its Opaque Data Type and payload"
                    .to_owned(),
            )
        };
    }
    if row["object_type"] == "Certificate"
        && response_object_type(ObjectType::from_raw(1), row).raw() == 1
    {
        let expected_type = enum_value(row, "response_certificate_type").unwrap_or(1);
        return if item_contains_certificate_type(object, expected_type)
            && item_contains_nonempty_certificate_value(object)
        {
            Ok(())
        } else {
            Err(format!(
                "Certificate response did not preserve expected CertificateType {expected_type} and a nonempty CertificateValue"
            ))
        };
    }
    let Some(expected_raw) = response_format_raw(row) else {
        // `key_format` describes the manifest's input/output format. It does
        // not select a wire format unless the request has an explicit
        // requested_key_format or requested_format_value parameter.
        return Ok(());
    };
    let expected = row["parameters"]["requested_key_format"]
        .as_str()
        .or_else(|| row["parameters"]["requested_format_value"].as_str())
        .unwrap_or("explicit Key Format Type");
    let found = item_contains_format(object, expected_raw);
    if found {
        Ok(())
    } else {
        Err(format!(
            "requested Key Format Type {expected} was not present in the typed object; returned Key Format Type values: {:?}",
            item_key_format_values(object)
        ))
    }
}

fn response_format_raw(row: &serde_json::Value) -> Option<u32> {
    if row["key_format"] == "Cosmian-Pkcs12Legacy" {
        // Cosmian accepts the legacy vendor selector but normalizes the
        // exported object to the KMIP PKCS#12 Key Format Type.
        Some(0x16)
    } else {
        requested_format_raw(row)
    }
}

fn response_object_type(object_type: ObjectType, row: &serde_json::Value) -> ObjectType {
    if object_type.raw() == 1 && matches!(requested_format_raw(row), Some(0x16 | 0x8880_0001)) {
        // Cosmian packages a certificate's linked private key and certificate
        // as a PKCS#12 object, represented on the KMIP wire as a PrivateKey.
        ObjectType::from_raw(4)
    } else {
        object_type
    }
}

fn opaque_object_preserves_case_payload(object: &Item, row: &serde_json::Value) -> bool {
    if object.tag().raw() != OPAQUE_OBJECT {
        return false;
    }
    let expected_payload = row["case_id"].as_str().unwrap_or_default().as_bytes();
    object.with_value(|value| {
        let ValueView::Structure(fields) = value else {
            return false;
        };
        let mut data_type_matches = false;
        let mut data_value_matches = false;
        for field in fields.children() {
            match field.tag().raw() {
                OPAQUE_DATA_TYPE => field.with_value(|value| {
                    data_type_matches = matches!(
                        value,
                        ValueView::Enumeration(raw) if *raw == 0x8000_0001
                    );
                }),
                OPAQUE_DATA_VALUE => field.with_value(|value| {
                    data_value_matches = matches!(
                        value,
                        ValueView::ByteString(bytes) if bytes == expected_payload
                    );
                }),
                _ => {}
            }
        }
        data_type_matches && data_value_matches
    })
}

fn item_contains_format(item: &Item, expected: u32) -> bool {
    item.with_value(|value| {
        let ValueView::Structure(object) = value else {
            return false;
        };
        object.children().iter().any(|field| {
            field.with_value(|value| {
                let ValueView::Structure(block) = value else {
                    return false;
                };
                block.children().iter().any(|member| {
                    member.tag().raw() == KEY_FORMAT_TYPE
                        && member.with_value(|value| {
                            matches!(value, ValueView::Enumeration(raw) if *raw == expected)
                        })
                })
            })
        })
    })
}

fn item_contains_tag(item: &Item, expected: u32) -> bool {
    fn contains(value: ValueView<'_>, expected: u32) -> bool {
        let ValueView::Structure(structure) = value else {
            return false;
        };
        structure.children().iter().any(|field| {
            field.tag().raw() == expected || field.with_value(|value| contains(value, expected))
        })
    }

    item.with_value(|value| contains(value, expected))
}

fn item_key_format_values(item: &Item) -> Vec<u32> {
    fn collect(value: ValueView<'_>, formats: &mut Vec<u32>) {
        let ValueView::Structure(structure) = value else {
            return;
        };
        for field in structure.children() {
            if field.tag().raw() == KEY_FORMAT_TYPE {
                field.with_value(|value| {
                    if let ValueView::Enumeration(raw) = value {
                        formats.push(*raw);
                    }
                });
            }
            field.with_value(|value| collect(value, formats));
        }
    }

    let mut formats = Vec::new();
    item.with_value(|value| collect(value, &mut formats));
    formats
}

fn item_contains_certificate_type(item: &Item, expected: u32) -> bool {
    item.with_value(|value| {
        let ValueView::Structure(fields) = value else {
            return false;
        };
        fields.children().iter().any(|field| {
            field.tag().raw() == CERTIFICATE_TYPE
                && field.with_value(
                    |value| matches!(value, ValueView::Enumeration(actual) if *actual == expected),
                )
        })
    })
}

fn item_contains_nonempty_certificate_value(item: &Item) -> bool {
    item.with_value(|value| {
        let ValueView::Structure(fields) = value else {
            return false;
        };
        fields.children().iter().any(|field| {
            field.tag().raw() == CERTIFICATE_VALUE
                && field.with_value(
                    |value| matches!(value, ValueView::ByteString(bytes) if !bytes.is_empty()),
                )
        })
    })
}

fn import_payload_preserves_wrapping_specification(
    payload: &Structure,
    expected: &Structure,
) -> bool {
    let mut has_wrapped_type = false;
    let mut object_matches = false;
    for field in payload.view().children() {
        if field.tag().raw() == 0x0042_00F8 {
            field.with_value(|value| {
                has_wrapped_type = matches!(value, ValueView::Enumeration(raw) if *raw == 2);
            });
        }
        if !matches!(field.tag().raw(), SYMMETRIC_KEY | PUBLIC_KEY | PRIVATE_KEY) {
            continue;
        }
        field.with_value(|value| {
            let ValueView::Structure(object) = value else {
                return;
            };
            for object_field in object.children() {
                if object_field.tag().raw() != KEY_BLOCK {
                    continue;
                }
                object_field.with_value(|value| {
                    let ValueView::Structure(key_block) = value else {
                        return;
                    };
                    for block_field in key_block.children() {
                        if block_field.tag().raw() != KEY_WRAPPING_DATA {
                            continue;
                        }
                        block_field.with_value(|value| {
                            object_matches = match value {
                                ValueView::Structure(actual) => structures_match(&actual, expected),
                                _ => false,
                            };
                        });
                    }
                });
            }
        });
    }
    has_wrapped_type && object_matches
}

fn structures_match(actual: &kmipkit_ttlv::StructureView<'_>, expected: &Structure) -> bool {
    let expected = expected.view();
    let actual_items = actual.children();
    let expected_items = expected.children();
    actual_items.len() == expected_items.len()
        && actual_items
            .iter()
            .zip(expected_items.iter())
            .all(|(actual, expected)| {
                actual.tag().raw() == expected.tag().raw()
                    && actual.with_value(|actual_value| {
                        expected
                            .with_value(|expected_value| values_match(actual_value, expected_value))
                    })
            })
}

fn values_match(actual: ValueView<'_>, expected: ValueView<'_>) -> bool {
    match (actual, expected) {
        (ValueView::Structure(actual), ValueView::Structure(expected)) => {
            let actual_items = actual.children();
            let expected_items = expected.children();
            actual_items.len() == expected_items.len()
                && actual_items
                    .iter()
                    .zip(expected_items.iter())
                    .all(|(actual, expected)| {
                        actual.tag().raw() == expected.tag().raw()
                            && actual.with_value(|actual_value| {
                                expected.with_value(|expected_value| {
                                    values_match(actual_value, expected_value)
                                })
                            })
                    })
        }
        (ValueView::Integer(actual), ValueView::Integer(expected)) => actual == expected,
        (ValueView::LongInteger(actual), ValueView::LongInteger(expected))
        | (ValueView::DateTime(actual), ValueView::DateTime(expected)) => actual == expected,
        (ValueView::BigInteger(actual), ValueView::BigInteger(expected))
        | (ValueView::ByteString(actual), ValueView::ByteString(expected)) => actual == expected,
        (ValueView::Enumeration(actual), ValueView::Enumeration(expected))
        | (ValueView::Interval(actual), ValueView::Interval(expected)) => actual == expected,
        (ValueView::Boolean(actual), ValueView::Boolean(expected)) => actual == expected,
        (ValueView::TextString(actual), ValueView::TextString(expected)) => actual == expected,
        (ValueView::DateTimeExtended(actual), ValueView::DateTimeExtended(expected)) => {
            actual == expected
        }
        _ => false,
    }
}

fn attributes_for(
    row: &serde_json::Value,
    object_type: ObjectType,
) -> Result<AttributeSet, Box<dyn Error>> {
    let mut items = Vec::new();
    if object_type.raw() != 1 {
        if let (Some(algorithm), Some(bits)) = (
            enum_value(row, "cryptographic_algorithm"),
            cryptographic_length_bits(row),
        ) {
            items.push(super::kmip_item(
                CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(algorithm),
            )?);
            items.push(super::kmip_item(
                CRYPTOGRAPHIC_LENGTH,
                Value::integer(i32::try_from(bits)?),
            )?);
        }
        if let Some(curve) = enum_value(row, "recommended_curve") {
            items.push(super::kmip_item(
                CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
                Value::structure(super::structure([super::kmip_item(
                    RECOMMENDED_CURVE,
                    Value::enumeration(curve),
                )?])?),
            )?);
        }
    }
    match object_type.raw() {
        2 => items.push(super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(i32::try_from(symmetric_key_usage_mask(row))?),
        )?),
        7 => items.push(super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(12),
        )?),
        3 => items.push(super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(2),
        )?),
        4 => items.push(super::kmip_item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(1),
        )?),
        1 => {
            let path = fixture_material_path(row, FixtureMaterial::Certificate)?;
            let bytes = fixture_bytes(row, &path)?;
            items.push(super::kmip_item(
                CERTIFICATE_LENGTH,
                Value::integer(i32::try_from(bytes.len())?),
            )?);
        }
        _ => {}
    }
    if object_type.raw() == 7 {
        items.push(super::kmip_item(SECRET_DATA_TYPE, Value::enumeration(1))?);
    }
    items.push(super::common::name_attribute(
        row["case_id"].as_str().unwrap_or("kmipkit-transfer-case"),
    )?);
    Ok(AttributeSet::try_new(items)?)
}

fn symmetric_key_usage_mask(row: &serde_json::Value) -> i64 {
    if row["algorithm"]
        .as_str()
        .is_some_and(|algorithm| algorithm.starts_with("HMAC"))
    {
        USAGE_MAC_GENERATE | USAGE_MAC_VERIFY
    } else {
        USAGE_ENCRYPT | USAGE_DECRYPT
    }
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
        other => return Err(format!("unmapped KMIP object type: {other}").into()),
    };
    Ok(ObjectType::from_raw(raw))
}

fn enum_value(row: &serde_json::Value, member: &str) -> Option<u32> {
    row["wire_enum_values"][member]["value"]
        .as_str()
        .and_then(|value| u32::from_str_radix(value.trim_start_matches("0x"), 16).ok())
}

fn cryptographic_length_bits(row: &serde_json::Value) -> Option<i64> {
    row["parameters"]["key_bits"].as_i64().or_else(|| {
        let curve = row["parameters"]["curve"]
            .as_str()
            .or_else(|| row["parameters"]["pre_quantum"].as_str())?;
        match curve {
            "P192" => Some(192),
            "P224" | "SECP224K1" => Some(224),
            "P256" | "SECP256K1" | "CURVE25519" | "CURVEED25519" => Some(256),
            "P384" => Some(384),
            "P521" => Some(521),
            "CURVE448" | "CURVEED448" => Some(448),
            _ => None,
        }
    })
}

fn identifier(value: &str) -> UniqueIdentifier {
    UniqueIdentifier::TextString(value.to_owned())
}

fn import_response_identifier(actual: Option<&UniqueIdentifier>) -> Result<String, String> {
    match actual {
        Some(UniqueIdentifier::TextString(value)) if !value.is_empty() => Ok(value.clone()),
        _ => Err("Import omitted a nonempty text Unique Identifier".to_owned()),
    }
}

fn classify_error(error: &dyn Error) -> &'static str {
    let text = error.to_string();
    if text.contains("case cleanup failed") {
        "cleanup_failed"
    } else if text.contains("client_blocked") || text.contains("category=") {
        "client_blocked"
    } else if text.contains("KMIP Result Status") || text.contains("Result Status") {
        if [
            " Create:",
            " Create Key Pair:",
            " Create wrapping key:",
            " Create split source:",
            " Create Split Key setup:",
            " Export setup:",
            " Register setup:",
            " Import setup:",
            " PFX link inspection:",
        ]
        .iter()
        .any(|stage| text.contains(stage))
        {
            "setup_rejected"
        } else {
            "rejected"
        }
    } else if is_target_transfer_stage_error(&text) {
        "response_validation_failed"
    } else {
        "setup_or_typed_validation_failed"
    }
}

fn is_target_transfer_stage_error(text: &str) -> bool {
    [" Get:", " Export:", " Import:", " Register:"]
        .iter()
        .any(|stage| text.contains(stage))
}

fn target_request_sent_label(classification: &str, error: &dyn Error) -> &'static str {
    match classification {
        "typed_success" | "rejected" | "response_validation_failed" => "true",
        "setup_rejected" | "fixture_unavailable" => "false",
        "setup_or_typed_validation_failed" => {
            if is_target_transfer_stage_error(&error.to_string()) {
                "true"
            } else {
                "false"
            }
        }
        _ => "unknown",
    }
}

fn transfer_observation_line(
    case_id: &str,
    expected_outcome: &str,
    observed: &str,
    request_sent: &str,
) -> String {
    if observed == "typed_success" && request_sent == "true" {
        format!(
            "case_id={case_id} classification=typed_success expected_outcome={expected_outcome} observed=typed_success request_sent=true interoperability=true"
        )
    } else if expected_outcome != "typed_success" {
        format!(
            "case_id={case_id} classification=expected_finding_probe expected_outcome={expected_outcome} observed={observed} request_sent={request_sent} interoperability=false"
        )
    } else {
        format!(
            "case_id={case_id} classification={observed} expected_outcome=typed_success observed={observed} request_sent={request_sent} interoperability=false"
        )
    }
}

#[cfg(test)]
mod inventory_tests {
    use super::*;

    #[test]
    fn rsa_hybrid_wrapping_explicitly_encodes_padding_method_none() {
        let row = serde_json::json!({
            "parameters": {
                "padding": "None",
                "hash": "SHA-256"
            }
        });
        let parameters = wrapping_crypto_parameters_with_algorithm(&row, Some(4))
            .expect("RSA hybrid wrapping parameters");
        let padding_method = parameters.view().children().iter().find_map(|item| {
            if item.tag().raw() != PADDING_METHOD {
                return None;
            }
            item.with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            })
        });

        assert_eq!(
            padding_method,
            Some(1),
            "KMIP Padding Method None must select Cosmian's RSA-AES hybrid wrapping"
        );
    }

    fn has_sensitive_false(structure: &kmipkit_ttlv::StructureView<'_>) -> bool {
        structure.children().iter().any(|item| {
            if item.tag().raw() == SENSITIVE
                && item.with_value(
                    |value| matches!(value, ValueView::Boolean(sensitive) if !*sensitive),
                )
            {
                return true;
            }
            item.with_value(|value| match value {
                ValueView::Structure(child) => has_sensitive_false(&child),
                _ => false,
            })
        })
    }

    #[test]
    fn configurable_kem_register_setup_can_explicitly_request_non_sensitive_key_material() {
        // OASIS KMIP v2.1 §4.55, Tables 139–140: Sensitive is client-modifiable,
        // and defaults to False when the client omits it.
        let mut row = crate::common::manifest_rows(FAMILY)
            .expect("transfer manifest rows")
            .into_iter()
            .find(|row| {
                row["case_id"]
                    == "cosmian-528-register-privatekey-configurablekem-opaque-fixture-curve25519-mlkem512"
            })
            .expect("ConfigurableKEM private-key Register case");
        row["parameters"]["sensitive"] = serde_json::json!(false);

        let request =
            super::super::cryptographic::build_create_key_pair_request(&row, "sensitive-probe")
                .expect("typed Create Key Pair request")
                .into_ttlv_payload()
                .expect("typed Create Key Pair payload");

        assert!(
            has_sensitive_false(&request.view()),
            "the private-key setup request must encode Sensitive=False"
        );
    }

    #[test]
    fn sensitive_attribute_reader_preserves_the_server_boolean_value() {
        let sensitive =
            AttributeSet::try_new([super::super::kmip_item(SENSITIVE, Value::boolean(true))
                .expect("Sensitive attribute")])
            .expect("Sensitive attributes");
        let non_sensitive =
            AttributeSet::try_new([super::super::kmip_item(SENSITIVE, Value::boolean(false))
                .expect("Sensitive attribute")])
            .expect("Sensitive attributes");

        assert_eq!(sensitive_attribute_value(&sensitive), Some(true));
        assert_eq!(sensitive_attribute_value(&non_sensitive), Some(false));
    }

    #[test]
    fn transfer_case_ids_are_unique_and_match_manifest() {
        assert_manifest_parity();
    }

    #[test]
    fn descriptive_key_format_does_not_become_a_wire_selector() {
        let mut row = serde_json::json!({
            "key_format": "Raw",
            "parameters": {"requested_key_format": null},
            "fixture": {"encoding": "PEM"},
            "wire_enum_values": {"key_format_type": {"value": "0x00000001"}}
        });
        let request = with_requested_format(ExportRequest::new(), &row)
            .into_ttlv_payload()
            .expect("default-format Export payload");
        assert!(
            !request
                .view()
                .children()
                .iter()
                .any(|field| field.tag().raw() == KEY_FORMAT_TYPE)
        );
        row["fixture"]["encoding"] = serde_json::json!("DER");
        assert_eq!(enum_value(&row, "key_format_type"), Some(1));
    }

    #[test]
    fn certificate_constructor_uses_standard_certificate_type_and_value_tags() {
        let row = crate::common::manifest_rows(FAMILY)
            .expect("transfer manifest rows")
            .into_iter()
            .find(|row| row["case_id"] == "cosmian-528-get-certificate-rsa-2048-none-der")
            .expect("certificate Get row");
        let object = constructor_object(&row, ObjectType::from_raw(1))
            .expect("typed Certificate constructor");
        assert_eq!(object.tag().raw(), CERTIFICATE);
        require_format(Some(&object), &row)
            .expect("X.509 Certificate Type satisfies the requested X.509 format");
        let (has_type, has_value) = object.with_value(|value| {
            let ValueView::Structure(fields) = value else {
                return (false, false);
            };
            let has_type = fields.children().iter().any(|field| {
                field.tag().raw() == CERTIFICATE_TYPE
                    && field.with_value(
                        |value| matches!(value, ValueView::Enumeration(actual) if *actual == 1),
                    )
            });
            let has_value = fields.children().iter().any(|field| {
                field.tag().raw() == CERTIFICATE_VALUE
                    && field.with_value(
                        |value| matches!(value, ValueView::ByteString(bytes) if !bytes.is_empty()),
                    )
            });
            (has_type, has_value)
        });
        assert!(
            has_type,
            "CertificateType must use the KMIP 2.1 tag 0x42001D"
        );
        assert!(
            has_value,
            "CertificateValue must use the KMIP 2.1 tag 0x42001E"
        );
    }

    #[test]
    fn opaque_get_and_export_postconditions_check_opaque_payload_instead_of_key_format() {
        let case_id = "cosmian-528-get-opaqueobject-opaque-native";
        let row = serde_json::json!({
            "case_id": case_id,
            "operation": "Get",
            "object_type": "OpaqueObject",
            "key_format": "Opaque",
            "parameters": {"requested_key_format": "Opaque"}
        });
        let object = super::super::kmip_item(
            OPAQUE_OBJECT,
            Value::structure(
                super::super::structure([
                    super::super::kmip_item(OPAQUE_DATA_TYPE, Value::enumeration(0x8000_0001))
                        .expect("Opaque Data Type item"),
                    super::super::kmip_item(
                        OPAQUE_DATA_VALUE,
                        Value::byte_string(case_id.as_bytes().to_vec()),
                    )
                    .expect("Opaque Data Value item"),
                ])
                .expect("Opaque Object fields"),
            ),
        )
        .expect("Opaque Object item");

        require_format(Some(&object), &row)
            .expect("Opaque Object postcondition checks its own payload fields");
    }

    #[test]
    fn explicit_container_format_value_is_serialized_as_key_format_type() {
        let row = serde_json::json!({
            "parameters": {"requested_format_value": "0x88800002"},
            "wire_enum_values": {
                "key_format_type": {"value": "0x88800002"}
            }
        });
        let payload = with_requested_format(ExportRequest::new(), &row)
            .into_ttlv_payload()
            .expect("container Export payload");
        assert!(payload.view().children().iter().any(|field| {
            field.tag().raw() == KEY_FORMAT_TYPE
                && field.with_value(
                    |value| matches!(value, ValueView::Enumeration(raw) if *raw == 0x8880_0002),
                )
        }));
    }

    #[test]
    fn pkcs7_output_rows_serialize_the_vendor_key_format_selector() {
        let row = serde_json::json!({
            "operation": "Export",
            "key_format": "Cosmian-PKCS7",
            "parameters": {"requested_key_format": null},
            "wire_enum_values": {
                "key_format_type": {"value": "0x88800002"}
            }
        });
        let payload = with_requested_format(ExportRequest::new(), &row)
            .into_ttlv_payload()
            .expect("PKCS7 Export payload");
        assert!(payload.view().children().iter().any(|field| {
            field.tag().raw() == KEY_FORMAT_TYPE
                && field.with_value(
                    |value| matches!(value, ValueView::Enumeration(raw) if *raw == 0x8880_0002),
                )
        }));
    }

    #[test]
    fn pkcs12_get_export_outputs_use_a_linked_pfx_source_fixture() {
        let row = serde_json::json!({
            "key_format": "PKCS#12",
            "fixture": {"encoding": "DER"}
        });
        assert!(source_is_pfx_fixture(&row, "Get"));
        assert!(source_is_pfx_fixture(&row, "Export"));
        assert!(!source_is_pfx_fixture(&row, "Import"));
    }

    #[test]
    fn pkcs12_password_link_uses_the_case_owned_secret_data_identifier() {
        let link = pkcs12_password_link_attribute("password-object-id-42")
            .expect("PKCS#12 Password Link item");
        let mut link_type = None;
        let mut linked_identifier = None;
        link.with_value(|value| {
            let ValueView::Structure(value) = value else {
                return;
            };
            for field in value.children() {
                match field.tag().raw() {
                    LINK_TYPE => field.with_value(|child| {
                        if let ValueView::Enumeration(raw) = child {
                            link_type = Some(*raw);
                        }
                    }),
                    LINKED_OBJECT_IDENTIFIER => field.with_value(|child| {
                        if let ValueView::TextString(identifier) = child {
                            linked_identifier = Some((*identifier).to_owned());
                        }
                    }),
                    _ => {}
                }
            }
        });
        assert!(
            link_type == Some(0x10D),
            "Link Type must be PKCS#12 Password Link"
        );
        assert!(
            linked_identifier.as_deref() == Some("password-object-id-42"),
            "PKCS#12 Password Link must target the case-owned SecretData object"
        );
    }

    #[test]
    fn password_link_pkcs12_get_carries_password_in_key_wrapping_specification() {
        let row = serde_json::json!({
            "operation": "Get",
            "key_format": "PKCS#12",
            "parameters": {
                "requested_format_value": "0x00000016",
                "password_link": true
            },
            "wire_enum_values": {
                "key_format_type": {"value": "0x00000016"}
            }
        });
        let request = build_get_request(
            &row,
            "pfx-password-case",
            "private-id",
            None,
            Some("password-object-id-42"),
        )
        .expect("password-protected PKCS#12 Get request");
        let payload = request
            .to_ttlv_payload()
            .expect("password-protected PKCS#12 Get payload");
        let mut password_identifier = None;
        for field in payload.view().children() {
            if field.tag().raw() != KEY_WRAPPING_SPECIFICATION {
                continue;
            }
            field.with_value(|value| {
                let ValueView::Structure(specification) = value else {
                    return;
                };
                for specification_field in specification.children() {
                    if specification_field.tag().raw() != ENCRYPTION_KEY_INFORMATION {
                        continue;
                    }
                    specification_field.with_value(|value| {
                        let ValueView::Structure(information) = value else {
                            return;
                        };
                        password_identifier = information.children().iter().find_map(|item| {
                            if item.tag().raw() != UNIQUE_IDENTIFIER {
                                return None;
                            }
                            item.with_value(|value| match value {
                                ValueView::TextString(actual) => Some((*actual).to_owned()),
                                _ => None,
                            })
                        });
                    });
                }
            });
        }
        assert!(
            password_identifier.as_deref() == Some("password-object-id-42"),
            "PFX password selection must reference its case-owned SecretData identifier"
        );
    }

    #[test]
    fn legacy_pkcs12_response_uses_standard_pkcs12_key_format() {
        let row = serde_json::json!({
            "object_type": "PrivateKey",
            "key_format": "Cosmian-Pkcs12Legacy",
            "parameters": {"requested_format_value": "0x88800001"},
            "wire_enum_values": {
                "key_format_type": {"value": "0x88800001"}
            }
        });
        let value = super::super::kmip_item(
            KEY_VALUE,
            Value::structure(
                super::super::structure([super::super::kmip_item(
                    KEY_MATERIAL,
                    SecretBytes::new(vec![1, 2, 3]).into_ttlv_value(),
                )
                .expect("key material item")])
                .expect("key value structure"),
            ),
        )
        .expect("key value item");
        let key_block = super::super::structure([
            super::super::kmip_item(KEY_FORMAT_TYPE, Value::enumeration(0x16))
                .expect("PKCS12 response format item"),
            value,
        ])
        .expect("private key block");
        let object = super::super::kmip_item(
            PRIVATE_KEY,
            Value::structure(
                super::super::structure([super::super::kmip_item(
                    KEY_BLOCK,
                    Value::structure(key_block),
                )
                .expect("private key block item")])
                .expect("private key object"),
            ),
        )
        .expect("private key object item");

        require_format(Some(&object), &row)
            .expect("Cosmian legacy PKCS12 output is typed as standard PKCS12");
        assert_eq!(item_key_format_values(&object), [0x16]);
    }

    #[test]
    fn certificate_pkcs12_get_and_export_expect_private_key_response() {
        for raw_format in [0x16_u32, 0x8880_0001] {
            let row = serde_json::json!({
                "object_type": "Certificate",
                "key_format": if raw_format == 0x16 { "PKCS#12" } else { "Cosmian-Pkcs12Legacy" },
                "parameters": {"requested_format_value": format!("0x{raw_format:08X}")}
            });
            assert_eq!(
                response_object_type(ObjectType::from_raw(1), &row).raw(),
                4,
                "PKCS#12 output from a Certificate target is a PrivateKey object"
            );
            assert_eq!(response_format_raw(&row), Some(0x16));
        }

        let pkcs7_row = serde_json::json!({
            "object_type": "Certificate",
            "key_format": "Cosmian-PKCS7",
            "parameters": {"requested_format_value": "0x88800002"}
        });
        assert_eq!(
            response_object_type(ObjectType::from_raw(1), &pkcs7_row).raw(),
            1
        );
    }

    #[test]
    fn opaque_format_is_not_serialized_as_key_format_type() {
        let row = serde_json::json!({
            "parameters": {"requested_key_format": "Opaque"},
            "wire_enum_values": {
                "key_format_type": {"value": "0x00000002"}
            }
        });
        let payload = with_requested_format(GetRequest::new(), &row)
            .to_ttlv_payload()
            .expect("Opaque Get payload");
        assert!(
            !payload
                .view()
                .children()
                .iter()
                .any(|field| { field.tag().raw() == KEY_FORMAT_TYPE })
        );
    }

    #[test]
    fn asymmetric_encrypt_wrapping_uses_public_identifier() {
        assert_eq!(
            rsa_wrapping_key_identifier("Encrypt", "private-id", "public-id"),
            Ok("public-id")
        );
        assert_eq!(
            rsa_wrapping_key_identifier("Decrypt", "private-id", "public-id"),
            Ok("private-id")
        );
        assert!(rsa_wrapping_key_identifier("MAC/sign", "private-id", "public-id").is_err());
    }

    #[test]
    fn setup_server_rejections_are_not_reported_as_target_operation_rejections() {
        let create_rejection = std::io::Error::other(
            "case cosmian-528-get-symmetrickey-hmac-sha3-256 Create: KMIP Result Status 1 (Operation Failed) Reason Some((5, \"Operation Not Supported\"))",
        );
        let export_setup_rejection = std::io::Error::other(
            "case cosmian-528-register-publickey-rsa-wrap-oaep-2048-sha3-256-noencoding Export setup: KMIP Result Status 1 (Operation Failed) Reason Some((5, \"Operation Not Supported\"))",
        );
        let get_rejection = std::io::Error::other(
            "case cosmian-528-get-publickey-rsa-wrap-oaep-2048-sha3-256-noencoding Get: KMIP Result Status 1 (Operation Failed) Reason Some((5, \"Operation Not Supported\"))",
        );
        let target_split_rejection = std::io::Error::other(
            "case cosmian-528-create-split-key-aes Create Split Key: KMIP Result Status 1 (Operation Failed) Reason Some((5, \"Operation Not Supported\"))",
        );

        assert_eq!(classify_error(&create_rejection), "setup_rejected");
        assert_eq!(classify_error(&export_setup_rejection), "setup_rejected");
        assert_eq!(classify_error(&get_rejection), "rejected");
        assert_eq!(classify_error(&target_split_rejection), "rejected");
    }

    #[test]
    fn response_validation_failure_records_that_target_request_was_sent() {
        let response_validation = std::io::Error::other(
            "case cosmian-528-export-certificate-rsa-container-cosmian-pkcs7-3072 Export: Certificate response did not preserve expected fields",
        );
        let source_register_rejection = std::io::Error::other(
            "case cosmian-528-get-privatekey Register setup: KMIP Result Status 1 (Operation Failed)",
        );

        let response_class = classify_error(&response_validation);
        assert_eq!(response_class, "response_validation_failed");
        assert_eq!(
            target_request_sent_label(response_class, &response_validation),
            "true"
        );
        let source_class = classify_error(&source_register_rejection);
        assert_eq!(source_class, "setup_rejected");
        assert_eq!(
            target_request_sent_label(source_class, &source_register_rejection),
            "false"
        );
    }

    #[test]
    fn hmac_sha3_import_fixtures_preserve_algorithm_length_and_manifest_format() {
        let rows = crate::common::manifest_rows(FAMILY).expect("transfer manifest rows");
        for format in ["Raw", "TransparentSymmetricKey"] {
            let row = rows
                .iter()
                .find(|row| {
                    row["operation"] == "Import"
                        && row["algorithm"] == "HMAC-SHA3-256"
                        && row["key_format"] == format
                })
                .unwrap_or_else(|| panic!("missing HMAC-SHA3-256 {format} Import row"));
            let case_id =
                crate::common::required_text(row, "case_id").expect("HMAC fixture case ID");
            let object = fixture_object(row, ObjectType::from_raw(2))
                .unwrap_or_else(|error| panic!("{case_id}: HMAC fixture: {error}"));
            // OASIS KMIP v2.1 §11.56, Table 487 assigns Symmetric Key tag 0x42008F.
            assert_eq!(object.tag().raw(), 0x0042_008F, "{case_id}");
            let expected_format = enum_value(row, "key_format_type")
                .unwrap_or_else(|| panic!("{case_id}: missing format enum"));
            assert!(
                item_contains_format(&object, expected_format),
                "{case_id}: fixture omitted its manifest Key Format Type"
            );
            if format == "TransparentSymmetricKey" {
                // OASIS KMIP v2.1 §3.4, Table 18 requires a Key byte string
                // inside the transparent Key Material structure.
                assert!(
                    item_contains_tag(&object, 0x0042_003F),
                    "{case_id}: transparent Key Material omitted Key tag 0x42003F"
                );
            }
            let (algorithm, length) = object.with_value(|value| {
                let ValueView::Structure(fields) = value else {
                    return (None, None);
                };
                let Some(key_block) = fields
                    .children()
                    .iter()
                    .find(|item| item.tag().raw() == KEY_BLOCK)
                else {
                    return (None, None);
                };
                key_block.with_value(|value| {
                    let ValueView::Structure(block) = value else {
                        return (None, None);
                    };
                    let mut algorithm = None;
                    let mut length = None;
                    for item in block.children() {
                        match item.tag().raw() {
                            CRYPTOGRAPHIC_ALGORITHM => item.with_value(|value| {
                                if let ValueView::Enumeration(raw) = value {
                                    algorithm = Some(*raw);
                                }
                            }),
                            CRYPTOGRAPHIC_LENGTH => item.with_value(|value| {
                                if let ValueView::Integer(raw) = value {
                                    length = Some(*raw);
                                }
                            }),
                            _ => {}
                        }
                    }
                    (algorithm, length)
                })
            });
            assert_eq!(
                algorithm,
                enum_value(row, "cryptographic_algorithm"),
                "{case_id}"
            );
            assert_eq!(length, Some(256), "{case_id}");
            let attributes = attributes_for(row, ObjectType::from_raw(2))
                .unwrap_or_else(|error| panic!("{case_id}: HMAC attributes: {error}"));
            let usage = attributes.as_items().iter().find_map(|item| {
                if item.tag().raw() != CRYPTOGRAPHIC_USAGE_MASK {
                    return None;
                }
                item.with_value(|value| match value {
                    ValueView::Integer(value) => Some(*value),
                    _ => None,
                })
            });
            assert_eq!(usage, Some(0xC0), "{case_id}: HMAC UsageMask");
        }
    }

    #[test]
    fn fixture_stems_cover_k1_montgomery_and_edwards_curves() {
        let cases = [
            ("ECDH", "SECP224K1", "ec-SECP224K1"),
            ("ECDSA", "SECP256K1", "ec-SECP256K1"),
            ("ECDH", "CURVE25519", "x25519"),
            ("ECDH", "CURVE448", "x448"),
            ("ECDSA", "CURVEED25519", "ed25519"),
            ("ECDSA", "CURVEED448", "ed448"),
        ];

        for (algorithm, curve, expected_stem) in cases {
            let row = serde_json::json!({
                "algorithm": algorithm,
                "parameters": {"curve": curve}
            });
            assert_eq!(
                fixture_stem(&row).expect("supported curve fixture stem"),
                expected_stem,
                "{algorithm} {curve} fixture stem"
            );
        }
    }

    #[test]
    fn fixture_material_path_rejects_curve_path_traversal() {
        let row = serde_json::json!({
            "algorithm": "ECDSA",
            "parameters": {"curve": "/../../client/client"},
            "fixture": {"encoding": "PEM"},
            "key_format": "PKCS#8"
        });

        assert!(
            fixture_material_path(&row, FixtureMaterial::PrivateKey).is_err(),
            "manifest curve values must not escape the validation fixture directory"
        );
    }

    #[test]
    fn opaque_post_quantum_transfer_rows_are_not_skipped() {
        let rows = crate::common::manifest_rows(FAMILY).expect("transfer manifest rows");
        let probes = configurable_kem_probe_case_ids().collect::<BTreeSet<_>>();
        let expected = rows
            .iter()
            .filter(|row| {
                row["algorithm"] == "ConfigurableKEM"
                    && matches!(
                        row["parameters"]["opaque_fixture_configuration"]["post_quantum"].as_str(),
                        Some("MLKEM512" | "MLKEM768")
                    )
            })
            .map(|row| {
                crate::common::required_text(row, "case_id")
                    .expect("opaque KEM case ID")
                    .to_owned()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(expected.len(), 72);
        assert_eq!(
            probes,
            expected.iter().map(String::as_str).collect(),
            "the explicitly tracked source-gap probes must cover every post-only/hybrid ConfigurableKEM transfer row"
        );
        assert!(
            rows.iter()
                .filter(|row| probes.contains(row["case_id"].as_str().expect("case ID")))
                .all(|row| row["expected_outcome"] != "typed_success"),
            "source-only transfer probes must not be classified as successful interoperability"
        );
    }

    #[test]
    fn successful_transfer_finding_probes_count_as_observed_interoperability() {
        let rows = crate::common::manifest_rows(FAMILY).expect("transfer manifest rows");
        let probes = rows
            .iter()
            .filter(|row| row["expected_outcome"] != "typed_success")
            .collect::<Vec<_>>();
        assert_eq!(probes.len(), 468);

        for row in &probes {
            let case_id = crate::common::required_text(row, "case_id").expect("case ID");
            let expected_outcome =
                crate::common::required_text(row, "expected_outcome").expect("expected outcome");
            let record = transfer_observation_line(case_id, expected_outcome, "rejected", "true");
            assert!(
                record.contains("classification=expected_finding_probe"),
                "{case_id}"
            );
            assert!(
                record.contains(&format!("expected_outcome={expected_outcome}")),
                "{case_id}"
            );
            assert!(record.contains("observed=rejected"), "{case_id}");
            assert!(record.contains("request_sent=true"), "{case_id}");
            assert!(record.contains("interoperability=false"), "{case_id}");
        }

        let successful_probe = probes
            .iter()
            .find(|row| row["observed_result"]["classification"] == "typed_success")
            .expect("successful source-discrepancy transfer row");
        let case_id = crate::common::required_text(successful_probe, "case_id")
            .expect("successful finding case ID");
        let expected_outcome = crate::common::required_text(successful_probe, "expected_outcome")
            .expect("source-discrepancy expected outcome");
        let record = transfer_observation_line(case_id, expected_outcome, "typed_success", "true");
        assert!(record.contains("classification=typed_success"));
        assert!(record.contains(&format!("expected_outcome={expected_outcome}")));
        assert!(record.contains("observed=typed_success"));
        assert!(record.contains("request_sent=true"));
        assert!(record.contains("interoperability=true"));
    }

    #[test]
    fn transfer_manifest_observations_reconcile_without_promoting_rejections() {
        let rows = crate::common::manifest_rows(FAMILY).expect("transfer manifest rows");
        let mut observed_categories = std::collections::BTreeSet::new();

        for row in rows {
            let case_id = crate::common::required_text(&row, "case_id").expect("case ID");
            let expected =
                crate::common::required_text(&row, "expected_outcome").expect("expected outcome");
            let observation = &row["observed_result"];
            let observed = crate::common::required_text(observation, "classification")
                .expect("observed classification");
            let request_sent = observation["target_request_sent"]
                .as_bool()
                .expect("observed target-request delivery")
                .to_string();
            let record = transfer_observation_line(case_id, expected, observed, &request_sent);
            let interoperable = observed == "typed_success" && request_sent == "true";
            assert_eq!(
                observation["interoperability"].as_bool(),
                Some(interoperable),
                "{case_id}: manifest interoperability must follow the typed target outcome"
            );

            assert!(
                record.contains(&format!("interoperability={interoperable}")),
                "{case_id}: {record}"
            );
            if expected != "typed_success" && observed != "typed_success" {
                assert!(
                    record.contains("classification=expected_finding_probe"),
                    "{case_id}: {record}"
                );
            }
            if observed == "rejected" || observed == "setup_rejected" {
                assert!(
                    record.contains("interoperability=false"),
                    "{case_id}: {record}"
                );
            }
            observed_categories.insert(observed.to_owned());
        }

        assert_eq!(
            observed_categories,
            ["rejected", "setup_rejected", "typed_success"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        );
    }

    #[test]
    fn every_transfer_case_constructs_a_serializable_typed_request() {
        let rows = crate::common::manifest_rows(FAMILY).expect("transfer manifest rows");
        let mut serialized = 0;
        for row in rows {
            let case_id = crate::common::required_text(&row, "case_id").expect("transfer case ID");
            let operation =
                crate::common::required_text(&row, "operation").expect("transfer operation");
            let object_type = object_type(
                crate::common::required_text(&row, "object_type").expect("object type"),
            )
            .unwrap_or_else(|error| panic!("{case_id}: object type mapping: {error}"));
            let object = constructor_object(&row, object_type)
                .unwrap_or_else(|error| panic!("{case_id}: object constructor: {error}"));
            let attributes = constructor_attributes(&row, object_type)
                .unwrap_or_else(|error| panic!("{case_id}: attribute constructor: {error}"));
            let request = build_case_request(&row, case_id, object_type, object, attributes)
                .unwrap_or_else(|error| {
                    panic!("{case_id}: typed {operation} constructor: {error}")
                });
            let payload = encode_case_request(request)
                .unwrap_or_else(|error| panic!("{case_id}: typed {operation} serializer: {error}"));
            assert!(
                !payload.view().children().is_empty(),
                "{case_id}: typed {operation} request has an empty payload"
            );
            serialized += 1;
        }
        assert_eq!(serialized, 4_640);
    }

    #[test]
    fn every_explicit_get_and_export_format_selection_is_encoded_as_key_format_type() {
        let rows = crate::common::manifest_rows(FAMILY).expect("transfer manifest rows");
        let mut formatted_operations = 0;
        for row in rows {
            let operation = row["operation"].as_str().expect("operation");
            if !matches!(operation, "Get" | "Export") {
                continue;
            }
            let format_is_selected = requested_format_raw(&row).is_some();
            if !format_is_selected {
                continue;
            }
            formatted_operations += 1;
            let case_id =
                crate::common::required_text(&row, "case_id").expect("formatted operation case ID");
            let expected = enum_value(&row, "key_format_type")
                .unwrap_or_else(|| panic!("{case_id}: selected format has no raw enum value"));
            let payload = if operation == "Get" {
                with_requested_format(
                    GetRequest::new().with_unique_identifier(identifier(case_id)),
                    &row,
                )
                .to_ttlv_payload()
            } else {
                with_requested_format(
                    ExportRequest::new().with_unique_identifier(identifier(case_id)),
                    &row,
                )
                .into_ttlv_payload()
            }
            .unwrap_or_else(|error| panic!("{case_id}: typed {operation} payload: {error}"));
            let found = payload.view().children().iter().any(|field| {
                field.tag().raw() == KEY_FORMAT_TYPE
                    && field.with_value(
                        |value| matches!(value, ValueView::Enumeration(raw) if *raw == expected),
                    )
            });
            assert!(
                found,
                "{case_id}: typed {operation} omitted or changed Key Format Type"
            );
        }
        assert_eq!(formatted_operations, 1_158);
    }

    // This validates every wrapped row's spec and all operation request shapes.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn every_wrapped_transfer_row_preserves_method_encoding_and_kek_identifier() {
        let rows = crate::common::manifest_rows(FAMILY).expect("transfer manifest rows");
        let mut wrapped_count = 0;
        for row in rows {
            if !row["parameters"]["wrapping_encoding"].is_string() {
                continue;
            }
            wrapped_count += 1;
            let case_id =
                crate::common::required_text(&row, "case_id").expect("wrapped case identifier");
            let kek_id = format!("{case_id}-kek");
            let specification = wrapping_specification(&row, &kek_id)
                .unwrap_or_else(|error| panic!("{case_id}: {error}"));
            let expected_encoding = match row["parameters"]["wrapping_encoding"]
                .as_str()
                .expect("encoding string")
            {
                "NoEncoding" => 1,
                "TTLVEncoding" => 2,
                other => panic!("{case_id}: unknown Encoding Option {other}"),
            };
            let expected_method = match row["parameters"]["wrapping_method"]
                .as_str()
                .expect("wrapping method")
            {
                "Encrypt" => 1,
                other => panic!("{case_id}: unknown Wrapping Method {other}"),
            };
            let mut found_encoding = false;
            let mut found_method = false;
            let mut found_kek = false;
            let key_type = row["parameters"]["wrapping_key_type"]
                .as_str()
                .unwrap_or("SymmetricKey");
            let algorithm_name = if key_type == "SymmetricKey" {
                row["parameters"]["kek_algorithm"]
                    .as_str()
                    .or_else(|| row["algorithm"].as_str())
                    .unwrap_or("AES")
            } else {
                row["algorithm"].as_str().unwrap_or("RSA")
            };
            let expected_algorithm = algorithm_value(algorithm_name)
                .unwrap_or_else(|| panic!("{case_id}: unsupported wrapping-key algorithm"));
            let mut found_algorithm = false;
            for field in specification.view().children() {
                match field.tag().raw() {
                    ENCODING_OPTION => field.with_value(|value| {
                        found_encoding =
                            matches!(value, ValueView::Enumeration(raw) if *raw == expected_encoding);
                    }),
                    WRAPPING_METHOD => field.with_value(|value| {
                        found_method =
                            matches!(value, ValueView::Enumeration(raw) if *raw == expected_method);
                    }),
                    ENCRYPTION_KEY_INFORMATION => field.with_value(|value| {
                        let ValueView::Structure(info) = value else {
                            return;
                        };
                        found_kek = info.children().iter().any(|child| {
                            child.tag().raw() == UNIQUE_IDENTIFIER
                                && child.with_value(|identifier| {
                                    matches!(identifier, ValueView::TextString(value) if *value == kek_id)
                                })
                        });
                        for child in info.children() {
                            if child.tag().raw() != CRYPTOGRAPHIC_PARAMETERS {
                                continue;
                            }
                            child.with_value(|value| {
                                let ValueView::Structure(parameters) = value else {
                                    return;
                                };
                                let order = parameters
                                    .children()
                                    .iter()
                                    .map(|parameter| {
                                        match parameter.tag().raw() {
                                            BLOCK_CIPHER_MODE => 0,
                                            PADDING_METHOD => 1,
                                            HASHING_ALGORITHM => 2,
                                            CRYPTOGRAPHIC_ALGORITHM => 5,
                                            IV_LENGTH => 7,
                                            TAG_LENGTH => 8,
                                            MASK_GENERATOR => 15,
                                            MGF_HASHING_ALGORITHM => 16,
                                            tag => panic!(
                                                "{case_id}: unexpected Cryptographic Parameters tag {tag:#x}"
                                            ),
                                        }
                                    })
                                    .collect::<Vec<_>>();
                                assert!(
                                    order.windows(2).all(|pair| pair[0] < pair[1]),
                                    "{case_id}: wrapping Cryptographic Parameters are not serialized in KMIP field order: {order:?}"
                                );
                                found_algorithm = parameters.children().iter().any(|parameter| {
                                    parameter.tag().raw() == CRYPTOGRAPHIC_ALGORITHM
                                        && parameter.with_value(|algorithm| {
                                            matches!(algorithm, ValueView::Enumeration(raw) if *raw == expected_algorithm)
                                        })
                                });
                            });
                        }
                    }),
                    _ => {}
                }
            }
            assert!(found_encoding, "{case_id}: Encoding Option was lost");
            assert!(found_method, "{case_id}: Wrapping Method was lost");
            assert!(found_kek, "{case_id}: case-owned KEK identifier was lost");
            assert!(
                found_algorithm,
                "{case_id}: wrapping Cryptographic Parameters omitted the KEK algorithm"
            );
            assert_ne!(
                case_id, kek_id,
                "target and KEK ownership identifiers must differ"
            );

            let request = GetRequest::new()
                .with_unique_identifier(identifier(case_id))
                .with_key_wrapping_specification(specification);
            let encoded = request
                .to_ttlv_payload()
                .unwrap_or_else(|error| panic!("{case_id}: typed Get request: {error}"));
            assert!(
                encoded
                    .view()
                    .children()
                    .iter()
                    .any(|field| field.tag().raw() == 0x0042_0047),
                "{case_id}: typed Get payload omitted Key Wrapping Specification"
            );

            if case_id == "cosmian-528-export-symmetrickey-aes-wrap-nistkeywrap-256-noencoding" {
                let export_payload =
                    build_export_request(&row, case_id, case_id, Some(&kek_id), None)
                        .unwrap_or_else(|error| panic!("{case_id}: typed Export request: {error}"))
                        .into_ttlv_payload()
                        .unwrap_or_else(|error| panic!("{case_id}: Export TTLV payload: {error}"));
                let payload_view = export_payload.view();
                let payload_tags = payload_view
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect::<Vec<_>>();
                assert_eq!(
                    payload_tags,
                    [UNIQUE_IDENTIFIER, KEY_WRAPPING_SPECIFICATION],
                    "{case_id}: Export Table 218 fields differ from Cosmian export_request()"
                );
                let specification = payload_view
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == KEY_WRAPPING_SPECIFICATION)
                    .expect("Export payload contains Key Wrapping Specification");
                specification.with_value(|value| {
                    let ValueView::Structure(specification) = value else {
                        panic!("{case_id}: Key Wrapping Specification is not a Structure");
                    };
                    let tags = specification
                        .children()
                        .iter()
                        .map(|field| field.tag().raw())
                        .collect::<Vec<_>>();
                    assert_eq!(
                        tags,
                        [WRAPPING_METHOD, ENCRYPTION_KEY_INFORMATION, ENCODING_OPTION],
                        "{case_id}: Key Wrapping Specification differs from Cosmian export_request()"
                    );
                    let encryption_key_information = specification
                        .children()
                        .iter()
                        .find(|field| field.tag().raw() == ENCRYPTION_KEY_INFORMATION)
                        .expect("specification contains Encryption Key Information");
                    encryption_key_information.with_value(|value| {
                        let ValueView::Structure(information) = value else {
                            panic!("{case_id}: Encryption Key Information is not a Structure");
                        };
                        assert_eq!(
                            information
                                .children()
                                .iter()
                                .map(|field| field.tag().raw())
                                .collect::<Vec<_>>(),
                            [UNIQUE_IDENTIFIER, CRYPTOGRAPHIC_PARAMETERS],
                            "{case_id}: Encryption Key Information differs from Cosmian export_request()"
                        );
                    });
                });
            }

            if row["operation"] == "Import"
                && matches!(
                    row["object_type"].as_str(),
                    Some("SymmetricKey" | "PublicKey" | "PrivateKey" | "SplitKey")
                )
            {
                assert_import_case_preserves_wrapping(&row, case_id, &kek_id);
            }
        }
        assert_eq!(wrapped_count, 2_452);
    }

    #[test]
    fn wrapping_key_usage_masks_match_wrap_encrypt_and_unwrap_decrypt_roles() {
        assert_eq!(wrapping_key_usage_mask("SymmetricKey", "Export"), Ok(0x14));
        assert_eq!(wrapping_key_usage_mask("SymmetricKey", "Import"), Ok(0x3C));
        assert_eq!(wrapping_key_usage_mask("PublicKey", "Export"), Ok(0x14));
        assert_eq!(wrapping_key_usage_mask("PrivateKey", "Export"), Ok(0x28));
        assert_eq!(wrapping_key_usage_mask("Certificate", "Export"), Ok(0x14));
        assert!(wrapping_key_usage_mask("SecretData", "Export").is_err());
    }

    #[test]
    fn import_response_accepts_a_server_assigned_nonempty_identifier() {
        let actual = UniqueIdentifier::TextString("server-assigned-id".to_owned());
        assert_eq!(
            import_response_identifier(Some(&actual)),
            Ok("server-assigned-id".to_owned())
        );
        assert!(import_response_identifier(None).is_err());
        assert!(
            import_response_identifier(Some(&UniqueIdentifier::TextString(String::new()))).is_err()
        );
    }

    fn assert_import_case_preserves_wrapping(row: &serde_json::Value, case_id: &str, kek_id: &str) {
        let object_type = object_type(row["object_type"].as_str().expect("object type"))
            .expect("mapped object type");
        let object = constructor_object(row, object_type)
            .unwrap_or_else(|error| panic!("{case_id}: constructor object: {error}"));
        let attributes = constructor_attributes(row, object_type)
            .unwrap_or_else(|error| panic!("{case_id}: constructor attributes: {error}"));
        let expected_wrapping = wrapping_specification(row, kek_id)
            .unwrap_or_else(|error| panic!("{case_id}: wrapping specification: {error}"));
        let import = build_import_request(
            &format!("{case_id}-imported"),
            object_type,
            attributes,
            object,
            true,
        )
        .into_ttlv_payload()
        .unwrap_or_else(|error| panic!("{case_id}: typed Import payload: {error}"));
        assert!(
            import_payload_preserves_wrapping_specification(&import, &expected_wrapping),
            "{case_id}: typed Import omitted or changed Key Wrap Type or Key Wrapping Data"
        );
    }
}
