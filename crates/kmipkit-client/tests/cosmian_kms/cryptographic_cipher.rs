//! Case-isolated symmetric Encrypt/Decrypt interoperability checks.
//!
//! The matrix IDs and parameters come from the pinned Cosmian 5.28.0 source
//! inventory. Each case sends `KMIPKit` typed requests and checks an Encrypt /
//! Decrypt round trip without printing plaintext, ciphertext, nonce, or tags.

use std::error::Error;

use kmipkit_client::{Client, ClientRequest};
use kmipkit_protocol::{
    ActivateRequest, DecryptRequest, EncryptRequest, OperationData, SecretBytes, UniqueIdentifier,
};
use kmipkit_ttlv::{Structure, Value};
use serde_json::Value as JsonValue;

const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const MASK_GENERATOR: u32 = 0x0042_0101;
const MASK_GENERATOR_HASHING_ALGORITHM: u32 = 0x0042_0102;
const PADDING_METHOD: u32 = 0x0042_005F;
const P_SOURCE: u32 = 0x0042_0103;
const IV_LENGTH: u32 = 0x0042_00CD;
const TAG_LENGTH: u32 = 0x0042_00CE;
const DATA: &[u8] = b"KMIPKit matrix data";
const MAX_RANDOM_NONCE_BYTES: usize = 64;
const FPE_TEST_TWEAK_LENGTH: usize = 13;
const AAD: &[u8] = b"KMIPKit additional data";

/// Runs a symmetric algorithm case. Returns `None` for asymmetric object rows,
/// which the parent cryptographic runner dispatches to its asymmetric path.
pub(super) fn run_symmetric_case(
    row: &JsonValue,
    client: &mut Client,
) -> Result<Option<()>, Box<dyn Error>> {
    if row["object_type"].as_str() != Some("SymmetricKey") {
        return Ok(None);
    }
    let case_id = text(row, "case_id")?;
    let algorithm = text(row, "algorithm")?;
    if !matches!(
        algorithm,
        "AES" | "FPE_FF1" | "ChaCha20" | "ChaCha20Poly1305"
    ) {
        return Err(format!("case {case_id} has no symmetric executor for {algorithm}").into());
    }

    let mut ownership = super::common::CaseOwnership::default();
    let operation_result = (|| {
        let objects = super::common::provision_object(
            client,
            "SymmetricKey",
            case_id,
            row,
            &mut ownership,
        )?;
        let identifier = UniqueIdentifier::TextString(objects.primary.clone());
        super::common::execute_case(
            client,
            case_id,
            ClientRequest::activate(ActivateRequest::new(Some(identifier.clone()))),
            "Activate symmetric cryptographic key",
            |response| {
                let returned = response
                    .activate()
                    .and_then(|result| result.unique_identifier());
                if !matches!(returned, Some(UniqueIdentifier::TextString(value)) if value == &objects.primary)
                {
                    return Err("Activate did not retain the case-owned key identifier".to_owned());
                }
                Ok(())
            },
        )?;
        run_round_trip(client, row, &identifier)
    })()
    .map_err(|error: Box<dyn Error>| error.to_string());

    super::common::finish_operation_with_cleanup(operation_result, || {
        super::common::cleanup_owned_objects(client, case_id, &mut ownership)
    })
    .map_err(|message| -> Box<dyn Error> { message.into() })?;
    Ok(Some(()))
}

fn run_round_trip(
    client: &mut Client,
    row: &JsonValue,
    identifier: &UniqueIdentifier,
) -> Result<(), Box<dyn Error>> {
    let case_id = text(row, "case_id")?;
    let operation = text(row, "operation")?;
    if !matches!(operation, "Encrypt" | "Decrypt") {
        return Err(format!("case {case_id} has non-cipher operation {operation}").into());
    }

    // Encrypt/Decrypt are each the matrix operation under test. The opposite
    // request is the cryptographic oracle for this isolated disposable key.
    // OASIS KMIP v2.1 §6.1.17/Table 214 and §6.1.11/Table 196 define requests;
    // Tables 215 and 197 define their successful result payloads.
    let plaintext = plaintext(row)?;
    let nonce = nonce_for(row)?;
    let additional_data = additional_data(row)?;
    let encrypt_nonce = nonce
        .as_ref()
        .map(|value| value.with_bytes(|bytes| SecretBytes::new(bytes.to_vec())));
    let encryption_row = encryption_fixture_row(row);
    let encrypted = execute_encrypt(
        client,
        &encryption_row,
        identifier.clone(),
        &plaintext,
        encrypt_nonce,
        additional_data.clone(),
    )?;
    let decrypted = execute_decrypt(
        client,
        row,
        identifier.clone(),
        encrypted.data,
        encrypted.iv.or(nonce),
        additional_data,
        decryption_tag(row, encrypted.tag)?,
    )?;
    if !decrypted.with_bytes(|actual| plaintext.with_bytes(|expected| actual == expected)) {
        return Err(format!(
            "case {case_id} {operation}: independent Encrypt/Decrypt round trip did not reproduce the private test input"
        )
        .into());
    }
    Ok(())
}

pub(super) fn offline_target_payload(row: &JsonValue) -> Result<Structure, Box<dyn Error>> {
    let case_id = text(row, "case_id")?;
    let operation = text(row, "operation")?;
    let identifier = UniqueIdentifier::TextString(format!("offline-{case_id}"));
    let parameters = cryptographic_parameters(row)?;
    let data = plaintext(row)?;
    let nonce = nonce_for(row)?;
    let additional_data = additional_data(row)?;

    match operation {
        "Encrypt" => {
            let mut request =
                EncryptRequest::new(Some(identifier), Some(OperationData::ByteString(data)))
                    .with_cryptographic_parameters(parameters);
            if let Some(nonce) = nonce {
                request = request.with_iv_counter_nonce(nonce);
            }
            if let Some(additional_data) = additional_data {
                request = request.with_authenticated_encryption_additional_data(SecretBytes::new(
                    additional_data,
                ));
            }
            Ok(request.to_ttlv_payload()?)
        }
        "Decrypt" => {
            let mut request =
                DecryptRequest::new(Some(identifier), Some(OperationData::ByteString(data)))
                    .with_cryptographic_parameters(parameters);
            if let Some(nonce) = nonce {
                request = request.with_iv_counter_nonce(nonce);
            }
            if let Some(additional_data) = additional_data {
                request = request.with_authenticated_encryption_additional_data(SecretBytes::new(
                    additional_data,
                ));
            }
            if row["parameters"]["mode"] == "GCM" {
                let tag_length = row["parameters"]["tag_length_bytes"]
                    .as_u64()
                    .and_then(|length| usize::try_from(length).ok())
                    .ok_or("GCM Decrypt row has no representable Tag Length")?;
                request = request
                    .with_authenticated_encryption_tag(SecretBytes::new(vec![0x5A; tag_length]));
            }
            Ok(request.to_ttlv_payload()?)
        }
        other => Err(format!("case {case_id} is not a symmetric cipher operation: {other}").into()),
    }
}

struct EncryptedData {
    data: SecretBytes,
    iv: Option<SecretBytes>,
    tag: Option<SecretBytes>,
}

fn encryption_fixture_row(row: &JsonValue) -> JsonValue {
    let mut fixture = row.clone();
    if row["operation"] == "Decrypt" && row["parameters"]["mode"] == "GCM" {
        // Cosmian 5.28.0 symmetric_ciphers.rs returns a full 16-byte GCM tag.
        // Only this prerequisite asks for it; the target Decrypt keeps its
        // original Tag Length and receives the corresponding truncated tag.
        fixture["parameters"]["tag_length_bytes"] = JsonValue::from(16);
    }
    fixture
}

fn validate_encryption_tag(row: &JsonValue, tag: Option<&SecretBytes>) -> Result<(), String> {
    // OASIS KMIP v2.1 §4.16/Table 59: Tag Length is measured in bytes.
    let requested = &row["parameters"]["tag_length_bytes"];
    if requested.is_null() {
        return Ok(());
    }
    let requested = requested
        .as_u64()
        .and_then(|bytes| usize::try_from(bytes).ok())
        .ok_or_else(|| "manifest Tag Length is not a valid byte count".to_owned())?;
    let tag = tag.ok_or_else(|| "successful Encrypt response has no requested tag".to_owned())?;
    if tag.with_bytes(|bytes| bytes.len() == requested) {
        Ok(())
    } else {
        Err("Encrypt returned a tag that did not match the requested Tag Length".to_owned())
    }
}

fn decryption_tag(
    row: &JsonValue,
    tag: Option<SecretBytes>,
) -> Result<Option<SecretBytes>, Box<dyn Error>> {
    if row["operation"] != "Decrypt" || row["parameters"]["mode"] != "GCM" {
        return Ok(tag);
    }
    // NIST SP 800-38D §5.2.1.2 lists permitted tag sizes; §7.1 step 6 takes
    // the leftmost bits. KMIP §4.16/Table 59 expresses that size in bytes.
    let requested = row["parameters"]["tag_length_bytes"]
        .as_u64()
        .and_then(|bytes| usize::try_from(bytes).ok())
        .filter(|bytes| matches!(bytes, 4 | 8 | 12..=16))
        .ok_or("manifest GCM Tag Length is not a permitted byte count")?;
    let tag = tag.ok_or("Encrypt fixture has no GCM authentication tag")?;
    if !tag.with_bytes(|bytes| bytes.len() == 16) {
        return Err("Encrypt fixture did not return a full 16-byte GCM tag".into());
    }
    Ok(Some(tag.with_bytes(|bytes| {
        SecretBytes::new(bytes[..requested].to_vec())
    })))
}

fn execute_encrypt(
    client: &mut Client,
    row: &JsonValue,
    identifier: UniqueIdentifier,
    plaintext: &SecretBytes,
    nonce: Option<SecretBytes>,
    additional_data: Option<Vec<u8>>,
) -> Result<EncryptedData, Box<dyn Error>> {
    let case_id = text(row, "case_id")?;
    let parameters = cryptographic_parameters(row)?;
    let mut request = EncryptRequest::new(
        Some(identifier),
        Some(OperationData::ByteString(SecretBytes::new(
            plaintext.with_bytes(<[u8]>::to_vec),
        ))),
    )
    .with_cryptographic_parameters(parameters);
    if let Some(nonce) = nonce {
        request = request.with_iv_counter_nonce(nonce);
    }
    if let Some(additional_data) = additional_data {
        request = request
            .with_authenticated_encryption_additional_data(SecretBytes::new(additional_data));
    }
    super::common::execute_case(
        client,
        case_id,
        ClientRequest::encrypt(request),
        "Encrypt",
        |response| {
            let result = response
                .encrypt()
                .ok_or_else(|| "successful Encrypt response has no typed result".to_owned())?;
            let data = result
                .data()
                .ok_or_else(|| "successful Encrypt response has no Data".to_owned())?
                .with_bytes(|bytes| SecretBytes::new(bytes.to_vec()));
            let iv = result
                .iv_counter_nonce()
                .map(|value| value.with_bytes(|bytes| SecretBytes::new(bytes.to_vec())));
            let tag = result
                .authenticated_encryption_tag()
                .map(|value| value.with_bytes(|bytes| SecretBytes::new(bytes.to_vec())));
            validate_encryption_tag(row, tag.as_ref())?;
            Ok(EncryptedData { data, iv, tag })
        },
    )
}

fn execute_decrypt(
    client: &mut Client,
    row: &JsonValue,
    identifier: UniqueIdentifier,
    ciphertext: SecretBytes,
    nonce: Option<SecretBytes>,
    additional_data: Option<Vec<u8>>,
    tag: Option<SecretBytes>,
) -> Result<SecretBytes, Box<dyn Error>> {
    let case_id = text(row, "case_id")?;
    let parameters = cryptographic_parameters(row)?;
    let mut request = DecryptRequest::new(
        Some(identifier),
        Some(OperationData::ByteString(ciphertext)),
    )
    .with_cryptographic_parameters(parameters);
    if let Some(nonce) = nonce {
        request = request.with_iv_counter_nonce(nonce);
    }
    if let Some(additional_data) = additional_data {
        request = request
            .with_authenticated_encryption_additional_data(SecretBytes::new(additional_data));
    }
    if let Some(tag) = tag {
        request = request.with_authenticated_encryption_tag(tag);
    }
    super::common::execute_case(
        client,
        case_id,
        ClientRequest::decrypt(request),
        "Decrypt",
        |response| {
            let result = response
                .decrypt()
                .ok_or_else(|| "successful Decrypt response has no typed result".to_owned())?;
            result
                .data()
                .map(|data| data.with_bytes(|bytes| SecretBytes::new(bytes.to_vec())))
                .ok_or_else(|| "successful Decrypt response has no Data".to_owned())
        },
    )
}

fn cryptographic_parameters(row: &JsonValue) -> Result<Structure, Box<dyn Error>> {
    // §4.16, Tables 59–60 define ordered members, IV Length in bits, and Tag
    // Length in bytes. Algorithm-specific defaults remain explicit in the
    // manifest; no request defaults are inferred here.
    let parameters = &row["parameters"];
    let mut output = Structure::new();
    if let Some(mode) = parameters["mode"].as_str() {
        let mode = match mode {
            "ECB" => 2,
            "CBC" => 1,
            "GCM" => 9,
            "AEAD" => 0x12,
            "GCMSIV" | "XTS" | "AESKeyWrapPadding" | "NISTKeyWrap" => {
                row["wire_enum_values"]["block_cipher_mode"]["value"]
                    .as_str()
                    .and_then(parse_hex)
                    .unwrap_or(match mode {
                        "XTS" => 11,
                        "AESKeyWrapPadding" => 12,
                        "NISTKeyWrap" => 13,
                        _ => 0x8000_0002,
                    })
            }
            other => return Err(format!("unmapped symmetric Block Cipher Mode {other}").into()),
        };
        output.try_push(super::kmip_item(
            BLOCK_CIPHER_MODE,
            Value::enumeration(mode),
        )?)?;
    }
    if let Some(padding) = parameters["padding"].as_str() {
        let padding = match padding {
            "None" => 1,
            "PKCS5" => 3,
            other => return Err(format!("unmapped symmetric Padding Method {other}").into()),
        };
        output.try_push(super::kmip_item(
            PADDING_METHOD,
            Value::enumeration(padding),
        )?)?;
    }
    if let Some(hash) = parameters["hash"].as_str() {
        output.try_push(super::kmip_item(
            HASHING_ALGORITHM,
            Value::enumeration(hash_value(hash)?),
        )?)?;
    }
    if let Some(bits) = parameters["iv_length_bits"].as_i64() {
        output.try_push(super::kmip_item(
            IV_LENGTH,
            Value::integer(i32::try_from(bits)?),
        )?)?;
    }
    if let Some(bytes) = parameters["tag_length_bytes"].as_i64() {
        output.try_push(super::kmip_item(
            TAG_LENGTH,
            Value::integer(i32::try_from(bytes)?),
        )?)?;
    }
    if let Some(mgf) = parameters["mgf"].as_str() {
        let raw = match mgf {
            "MGF1" => 1,
            other => return Err(format!("unmapped Mask Generator {other}").into()),
        };
        output.try_push(super::kmip_item(MASK_GENERATOR, Value::enumeration(raw))?)?;
    }
    if let Some(hash) = parameters["mgf_hash"].as_str() {
        output.try_push(super::kmip_item(
            MASK_GENERATOR_HASHING_ALGORITHM,
            Value::enumeration(hash_value(hash)?),
        )?)?;
    }
    if parameters["p_source"].as_str() == Some("empty") {
        output.try_push(super::kmip_item(P_SOURCE, Value::byte_string(Vec::new()))?)?;
    }
    Ok(output)
}

fn hash_value(name: &str) -> Result<u32, Box<dyn Error>> {
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
        other => return Err(format!("unmapped Hashing Algorithm {other}").into()),
    };
    Ok(value)
}

fn plaintext(row: &JsonValue) -> Result<SecretBytes, Box<dyn Error>> {
    if text(row, "algorithm")? == "FPE_FF1" {
        return fpe_plaintext(row).map(|value| SecretBytes::new(value.into_bytes()));
    }
    let mode = row["parameters"]["mode"].as_str().unwrap_or_default();
    let padding = row["parameters"]["padding"].as_str().unwrap_or_default();
    let length = match (mode, padding) {
        ("ECB", "" | "None") | ("CBC", "None") | ("AESKeyWrapPadding" | "NISTKeyWrap", _) => 16,
        ("ECB" | "CBC", "PKCS5") => 15,
        ("XTS", _) => 32,
        _ => DATA.len(),
    };
    let mut bytes = Vec::with_capacity(length);
    while bytes.len() < length {
        let remaining = length - bytes.len();
        bytes.extend_from_slice(&DATA[..remaining.min(DATA.len())]);
    }
    Ok(SecretBytes::new(bytes))
}

fn fpe_plaintext(row: &JsonValue) -> Result<String, Box<dyn Error>> {
    let data_shape = row["parameters"]["data_shape"]
        .as_str()
        .ok_or("FPE case has no data_shape")?;
    let value = match data_shape {
        "integer" => "12345678",
        "float" => "12.5",
        "text" => match row["parameters"]["alphabet"]
            .as_str()
            .ok_or("text FPE case has no alphabet")?
        {
            "numeric" => "12345678",
            "hexadecimal" => "abcdef01",
            "alpha_lower" => "abcdefgh",
            "alpha_upper" => "ABCDEFGH",
            "alpha" => "AbCdEfGh",
            "alpha_numeric" => "A1b2C3d4",
            "chinese" => "中文测试",
            "latin1sup" => "ÉÀÖÇ",
            "latin1sup_alphanum" => "ÉÀÖ",
            "utf" => "中文",
            "ascii_printable" => "A b!",
            "base64" => "Qm90",
            "custom" => "KMIP1234",
            other => return Err(format!("unknown FPE alphabet {other}").into()),
        },
        other => return Err(format!("unknown FPE data shape {other}").into()),
    };
    Ok(value.to_owned())
}

fn nonce_for(row: &JsonValue) -> Result<Option<SecretBytes>, Box<dyn Error>> {
    if text(row, "algorithm")? == "FPE_FF1" {
        return Ok(match row["parameters"]["tweak"].as_str() {
            Some("empty") => Some(SecretBytes::new(Vec::new())),
            Some("nonempty") => {
                let mut tweak = vec![0; FPE_TEST_TWEAK_LENGTH];
                aws_lc_rs::rand::fill(&mut tweak)
                    .map_err(|_| "secure test FPE tweak generation failed")?;
                Some(SecretBytes::new(tweak))
            }
            Some(other) => return Err(format!("unknown FPE tweak class {other}").into()),
            None => None,
        });
    }
    match row["parameters"]["nonce_source"].as_str() {
        Some("caller") => {
            let bytes = row["parameters"]["iv_length_bits"]
                .as_u64()
                .map(|bits| usize::try_from(bits / 8))
                .transpose()?
                .unwrap_or_else(|| match text(row, "algorithm").unwrap_or_default() {
                    "ChaCha20" => 8,
                    "ChaCha20Poly1305" => 12,
                    _ => 16,
                });
            if bytes > MAX_RANDOM_NONCE_BYTES {
                return Err("manifest nonce length exceeds the test input limit".into());
            }
            let mut nonce = vec![0; bytes];
            aws_lc_rs::rand::fill(&mut nonce).map_err(|_| "secure test nonce generation failed")?;
            Ok(Some(SecretBytes::new(nonce)))
        }
        Some("server" | "inapplicable") | None => Ok(None),
        Some(other) => Err(format!("unknown nonce source {other}").into()),
    }
}

fn additional_data(row: &JsonValue) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    if text(row, "algorithm")? == "FPE_FF1" {
        let shape = row["parameters"]["data_shape"]
            .as_str()
            .ok_or("FPE case has no data_shape")?;
        let metadata = match shape {
            "text" => {
                let alphabet = row["parameters"]["alphabet"]
                    .as_str()
                    .ok_or("text FPE case has no alphabet")?;
                let alphabet = if alphabet == "custom" {
                    "KMIP1234"
                } else {
                    alphabet
                };
                format!("{{\"type\":\"text\",\"alphabet\":\"{alphabet}\"}}")
            }
            "integer" => "{\"type\":\"integer\",\"alphabet\":\"numeric\"}".to_owned(),
            "float" => "{\"type\":\"float\"}".to_owned(),
            other => return Err(format!("unknown FPE data shape {other}").into()),
        };
        return Ok(Some(metadata.into_bytes()));
    }
    match row["parameters"]["aad"].as_str() {
        Some("empty") => Ok(Some(Vec::new())),
        Some("nonempty") => Ok(Some(AAD.to_vec())),
        Some("inapplicable") | None => Ok(None),
        Some(other) => Err(format!("unknown AAD class {other}").into()),
    }
}

fn text<'a>(row: &'a JsonValue, member: &str) -> Result<&'a str, Box<dyn Error>> {
    row[member]
        .as_str()
        .ok_or_else(|| format!("cryptographic case has no textual {member}").into())
}

fn parse_hex(value: &str) -> Option<u32> {
    u32::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

#[cfg(test)]
mod tests {
    use super::{additional_data, cryptographic_parameters, nonce_for, plaintext};

    fn gcm_row(operation: &str, length: u64) -> serde_json::Value {
        serde_json::json!({
            "case_id": "offline-gcm-tag-contract",
            "operation": operation,
            "algorithm": "AES",
            "parameters": {"mode": "GCM", "tag_length_bytes": length}
        })
    }

    #[test]
    // OASIS KMIP v2.1 §4.16/Table 59 defines Tag Length in bytes.
    fn encrypt_rejects_a_tag_that_does_not_match_the_requested_length() {
        for length in 12..=16 {
            let row = gcm_row("Encrypt", length);
            let fixture = super::encryption_fixture_row(&row);
            assert_eq!(
                fixture, row,
                "the Encrypt target request must stay unchanged"
            );
            for actual in [0, 11, 12, 13, 14, 15, 16, 17] {
                let tag = super::SecretBytes::new(vec![0; actual]);
                assert_eq!(
                    super::validate_encryption_tag(&fixture, Some(&tag)).is_ok(),
                    u64::try_from(actual).expect("small fixture length") == length,
                    "Encrypt must check its requested Tag Length"
                );
            }
            assert!(super::validate_encryption_tag(&fixture, None).is_err());
        }
    }

    #[test]
    // §6.1.11/Table 196: the requested Decrypt tag remains independent of its Encrypt oracle.
    fn decrypt_gcm_uses_a_full_tag_oracle_without_changing_target_parameters() {
        for length in 12..=16 {
            let row = gcm_row("Decrypt", length);
            let mut expected = row.clone();
            expected["parameters"]["tag_length_bytes"] = serde_json::json!(16);
            let fixture = super::encryption_fixture_row(&row);
            assert_eq!(fixture, expected);
            assert_eq!(row["parameters"]["tag_length_bytes"], length);
            let parameters = cryptographic_parameters(&fixture).expect("oracle parameters");
            assert!(parameters.view().children().iter().any(|item| {
                item.tag().raw() == super::TAG_LENGTH
                    && item
                        .with_value(|value| matches!(value, kmipkit_ttlv::ValueView::Integer(16)))
            }));
        }
    }

    #[test]
    // OASIS KMIP v2.1 §4.16/Table 59; NIST SP 800-38D §7.1 step 6:
    // a shortened GCM tag is the leftmost requested bits.
    fn decrypt_gcm_truncates_the_oracle_tag_to_each_requested_length() {
        let full_tag = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        for length in 12..=16 {
            let row = gcm_row("Decrypt", length);
            let tag = super::decryption_tag(&row, Some(super::SecretBytes::new(full_tag.to_vec())))
                .expect("valid GCM oracle tag")
                .expect("required target tag");
            assert!(tag.with_bytes(
                |bytes| bytes == &full_tag[..usize::try_from(length).expect("small length")]
            ));
        }
    }

    #[test]
    // OASIS KMIP v2.1 §4.16/Table 59, §6.1.17/Table 215; SP 800-38D §5.2.1.2.
    fn decrypt_gcm_rejects_missing_or_incomplete_oracle_tags() {
        let row = gcm_row("Decrypt", 12);
        assert!(super::decryption_tag(&row, None).is_err());
        for actual in [0, 11, 12, 15, 17] {
            assert!(
                super::decryption_tag(&row, Some(super::SecretBytes::new(vec![0; actual])))
                    .is_err()
            );
        }
        for requested in [0, 11, 17] {
            assert!(
                super::decryption_tag(
                    &gcm_row("Decrypt", requested),
                    Some(super::SecretBytes::new(vec![0; 16]))
                )
                .is_err()
            );
        }
    }

    #[test]
    // OASIS KMIP v2.1 §4.16/Table 59: each request retains its explicit parameters.
    fn non_gcm_and_encrypt_target_tags_are_not_truncated() {
        for mut row in [gcm_row("Encrypt", 12), gcm_row("Decrypt", 12)] {
            if row["operation"] == "Decrypt" {
                row["parameters"]["mode"] = serde_json::json!("GCMSIV");
            }
            assert_eq!(super::encryption_fixture_row(&row), row);
            let tag = super::decryption_tag(&row, Some(super::SecretBytes::new(vec![0; 16])))
                .expect("unchanged tag")
                .expect("present tag");
            assert!(tag.with_bytes(|bytes| bytes.len() == 16));
        }
    }

    #[test]
    // OASIS KMIP v2.1 §4.16/Table 59; §11.21/Table 454.
    fn explicit_no_padding_is_serialized_for_each_block_mode() {
        for mode in ["CBC", "ECB"] {
            let row = serde_json::json!({"parameters": {"mode": mode, "padding": "None"}});
            let parameters = cryptographic_parameters(&row).expect("valid block parameters");
            assert!(parameters.view().children().iter().any(|item| {
                item.tag().raw() == super::PADDING_METHOD
                    && item.with_value(|value| matches!(value, kmipkit_ttlv::ValueView::Enumeration(raw) if *raw == 1))
            }), "explicit None padding must reach the wire");
        }
    }

    #[test]
    fn aes_ecb_default_padding_fixture_is_block_aligned() {
        let row = serde_json::json!({
            "algorithm": "AES",
            "parameters": {"mode": "ECB", "padding": null}
        });

        assert_eq!(
            super::plaintext(&row)
                .expect("default AES-ECB plaintext fixture")
                .with_bytes(<[u8]>::len),
            16,
            "Cosmian defaults ECB to no padding, which requires whole AES blocks"
        );
    }

    #[test]
    fn caller_nonces_are_fresh_for_each_encryption_input() {
        let row = serde_json::json!({
            "algorithm": "AES",
            "parameters": {"nonce_source": "caller", "iv_length_bits": 96}
        });

        let first = nonce_for(&row)
            .expect("first secure nonce")
            .expect("caller nonce");
        let second = nonce_for(&row)
            .expect("second secure nonce")
            .expect("caller nonce");
        let matches = first.with_bytes(|first| second.with_bytes(|second| first == second));

        assert!(first.with_bytes(|nonce| nonce.len() == 12));
        assert!(!matches, "separate Encrypt inputs must not reuse a nonce");
    }

    #[test]
    fn nonempty_fpe_tweaks_are_fresh_for_each_encryption_input() {
        let row = serde_json::json!({
            "algorithm": "FPE_FF1",
            "parameters": {"tweak": "nonempty"}
        });

        let first = nonce_for(&row)
            .expect("first secure FPE tweak")
            .expect("nonempty FPE tweak");
        let second = nonce_for(&row)
            .expect("second secure FPE tweak")
            .expect("nonempty FPE tweak");
        let (length, matches) =
            first.with_bytes(|first| second.with_bytes(|second| (first.len(), first == second)));

        assert_eq!(length, 13);
        assert!(
            !matches,
            "separate FPE Encrypt inputs must not reuse a tweak"
        );
    }

    #[test]
    // OASIS KMIP v2.1 §11.6/Table 436 distinguishes AEAD from GCM.
    fn aead_and_gcm_keep_their_distinct_wire_modes() {
        for (mode, expected) in [("AEAD", 0x12), ("GCM", 9)] {
            let row = serde_json::json!({"parameters": {"mode": mode}});
            let parameters = cryptographic_parameters(&row).expect("valid AEAD parameters");
            assert!(parameters.view().children().iter().any(|item| {
                item.tag().raw() == super::BLOCK_CIPHER_MODE
                    && item.with_value(|value| matches!(value, kmipkit_ttlv::ValueView::Enumeration(raw) if *raw == expected))
            }), "the requested block mode must reach the wire");
        }
    }

    #[test]
    // OASIS KMIP v2.1 §6.1.17/Table 214 Data input; vendor fixture constraint:
    // Cosmian 5.28.0 crypto/fpe/alphabet.rs:11–23,298–305,439–442.
    fn ascii_fpe_fixture_meets_the_vendor_minimum_domain_size() {
        let row = serde_json::json!({
            "algorithm": "FPE_FF1",
            "parameters": {"data_shape": "text", "alphabet": "ascii_printable"}
        });
        let value = super::fpe_plaintext(&row).expect("ASCII FPE fixture");
        assert!(
            value
                .chars()
                .all(|character| (' '..='~').contains(&character))
        );
        assert!(
            95_u64.pow(u32::try_from(value.chars().count()).expect("short fixture")) >= 1_000_000
        );
    }

    #[test]
    fn symmetric_crypto_parameters_and_private_semantic_inputs_cover_manifest_rows() {
        let rows = super::super::common::manifest_rows("cosmian_kms::cryptographic")
            .expect("cryptographic matrix manifest");
        let selected = rows
            .iter()
            .filter(|row| {
                matches!(row["operation"].as_str(), Some("Encrypt" | "Decrypt"))
                    && row["object_type"] == "SymmetricKey"
            })
            .collect::<Vec<_>>();
        assert_eq!(selected.len(), 308);
        for row in selected {
            let case_id = row["case_id"].as_str().expect("case ID");
            assert!(
                cryptographic_parameters(row).is_ok(),
                "{case_id}: invalid parameters"
            );
            assert!(plaintext(row).is_ok(), "{case_id}: invalid data shape");
            assert!(nonce_for(row).is_ok(), "{case_id}: invalid nonce shape");
            assert!(additional_data(row).is_ok(), "{case_id}: invalid AAD shape");
        }
    }
}
