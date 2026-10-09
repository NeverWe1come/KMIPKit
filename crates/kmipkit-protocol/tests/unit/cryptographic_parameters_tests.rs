//! Cryptographic Parameters tests derived from OASIS KMIP v2.1 §4.16,
//! Table 59; §6.1.11, Table 196; §6.1.17, Table 214; and §11.6, Table 436.
//! These assert local operation payload trees, not transport or server behavior;
//! they are source-derived tests, not official OASIS Test Cases.
//!
//! Traceability: `KMIPKIT-REQ-SPEC-4.16-001-001`,
//! `KMIPKIT-REQ-SPEC-4.16-001-002`, `KMIPKIT-REQ-SPEC-4.16-002`, and
//! `KMIPKIT-REQ-SPEC-4.16-003`.

use crate::{
    DecryptRequest, EncryptRequest, OperationData, ProtocolError, SecretBytes, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
const DATA_TAG: u32 = 0x0042_00C2;
const BLOCK_CIPHER_MODE_TAG: u32 = 0x0042_0011;
const IV_LENGTH_TAG: u32 = 0x0042_00CD;
const TAG_LENGTH_TAG: u32 = 0x0042_00CE;
const VENDOR_PARAMETER_TAG: u32 = 0x0054_1234;
const SECOND_VENDOR_PARAMETER_TAG: u32 = 0x0054_1235;
const CTR_MODE: u32 = 6;
const GCM_MODE: u32 = 9;

#[derive(Debug, Eq, PartialEq)]
enum CapturedParameterValue {
    ByteString(Vec<u8>),
    Enumeration(u32),
    Integer(i32),
    Other,
}

fn item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag)
        .expect("the source-derived parameter tag fits the TTLV tag width")
        .try_checked()
        .expect("the source-derived parameter tag is allocated");
    Item::new(tag, value).expect("checked tag and value form a generic TTLV item")
}

fn parameters(members: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    let mut parameters = Structure::new();
    for (raw_tag, value) in members {
        parameters
            .try_push(item(raw_tag, value))
            .expect("Cryptographic Parameters remain within the structure depth limit");
    }
    parameters
}

fn encrypt_payload(
    cryptographic_parameters: Option<Structure>,
) -> Result<Structure, ProtocolError> {
    let request = EncryptRequest::new(Some(UniqueIdentifier::TextString(
        "object-identifier".to_owned(),
    )))
    .with_data(OperationData::ByteString(SecretBytes::new(vec![
        0x01, 0x02,
    ])));

    match cryptographic_parameters {
        Some(parameters) => request
            .with_cryptographic_parameters(parameters)
            .into_ttlv_payload(),
        None => request.into_ttlv_payload(),
    }
}

fn decrypt_payload(
    cryptographic_parameters: Option<Structure>,
) -> Result<Structure, ProtocolError> {
    let request = DecryptRequest::new(Some(UniqueIdentifier::TextString(
        "object-identifier".to_owned(),
    )))
    .with_data(OperationData::ByteString(SecretBytes::new(vec![
        0x01, 0x02,
    ])));

    match cryptographic_parameters {
        Some(parameters) => request
            .with_cryptographic_parameters(parameters)
            .into_ttlv_payload(),
        None => request.into_ttlv_payload(),
    }
}

fn parameter_members(payload: &Structure) -> Vec<(u32, CapturedParameterValue)> {
    let payload_view = payload.view();
    let parameters_item = payload_view
        .children()
        .iter()
        .find(|item| item.tag().raw() == CRYPTOGRAPHIC_PARAMETERS_TAG)
        .expect("the request contains the supplied Cryptographic Parameters");

    parameters_item.with_value(|value| match value {
        ValueView::Structure(parameters) => parameters
            .children()
            .iter()
            .map(|item| {
                let raw_tag = item.tag().raw();
                let value = item.with_value(|value| match value {
                    ValueView::ByteString(bytes) => {
                        CapturedParameterValue::ByteString(bytes.to_vec())
                    }
                    ValueView::Enumeration(value) => CapturedParameterValue::Enumeration(*value),
                    ValueView::Integer(value) => CapturedParameterValue::Integer(*value),
                    _ => CapturedParameterValue::Other,
                });
                (raw_tag, value)
            })
            .collect(),
        _ => panic!("Cryptographic Parameters is encoded as a Structure"),
    })
}

#[test]
fn variable_iv_mode_requires_iv_length_and_accepts_a_supplied_length() {
    let without_length = parameters([(BLOCK_CIPHER_MODE_TAG, Value::enumeration(CTR_MODE))]);
    assert!(encrypt_payload(Some(without_length)).is_err());

    let with_length = parameters([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(CTR_MODE)),
        (IV_LENGTH_TAG, Value::integer(128)),
    ]);
    let payload = encrypt_payload(Some(with_length))
        .expect("request conversion accepts CTR when its variable IV length is supplied");

    assert_eq!(
        parameter_members(&payload),
        [
            (
                BLOCK_CIPHER_MODE_TAG,
                CapturedParameterValue::Enumeration(CTR_MODE),
            ),
            (IV_LENGTH_TAG, CapturedParameterValue::Integer(128)),
        ]
    );
}

#[test]
fn decrypt_variable_iv_mode_requires_iv_length_and_accepts_a_supplied_length() {
    let without_length = parameters([(BLOCK_CIPHER_MODE_TAG, Value::enumeration(CTR_MODE))]);
    assert!(decrypt_payload(Some(without_length)).is_err());

    let with_length = parameters([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(CTR_MODE)),
        (IV_LENGTH_TAG, Value::integer(128)),
    ]);
    let payload = decrypt_payload(Some(with_length))
        .expect("request conversion accepts CTR when its variable IV length is supplied");

    assert_eq!(
        parameter_members(&payload),
        [
            (
                BLOCK_CIPHER_MODE_TAG,
                CapturedParameterValue::Enumeration(CTR_MODE),
            ),
            (IV_LENGTH_TAG, CapturedParameterValue::Integer(128)),
        ]
    );
}

#[test]
fn gcm_requires_tag_length_and_accepts_a_supplied_length() {
    let without_tag_length = parameters([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(GCM_MODE)),
        (IV_LENGTH_TAG, Value::integer(96)),
    ]);
    assert!(encrypt_payload(Some(without_tag_length)).is_err());

    let with_tag_length = parameters([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(GCM_MODE)),
        (IV_LENGTH_TAG, Value::integer(96)),
        (TAG_LENGTH_TAG, Value::integer(16)),
    ]);
    let payload = encrypt_payload(Some(with_tag_length))
        .expect("request conversion accepts GCM when its IV and tag lengths are supplied");

    assert_eq!(
        parameter_members(&payload),
        [
            (
                BLOCK_CIPHER_MODE_TAG,
                CapturedParameterValue::Enumeration(GCM_MODE),
            ),
            (IV_LENGTH_TAG, CapturedParameterValue::Integer(96)),
            (TAG_LENGTH_TAG, CapturedParameterValue::Integer(16)),
        ]
    );
}

#[test]
fn decrypt_gcm_requires_tag_length_and_accepts_a_supplied_length() {
    let without_tag_length = parameters([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(GCM_MODE)),
        (IV_LENGTH_TAG, Value::integer(96)),
    ]);
    assert!(decrypt_payload(Some(without_tag_length)).is_err());

    let with_tag_length = parameters([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(GCM_MODE)),
        (IV_LENGTH_TAG, Value::integer(96)),
        (TAG_LENGTH_TAG, Value::integer(16)),
    ]);
    let payload = decrypt_payload(Some(with_tag_length))
        .expect("request conversion accepts GCM when its IV and tag lengths are supplied");

    assert_eq!(
        parameter_members(&payload),
        [
            (
                BLOCK_CIPHER_MODE_TAG,
                CapturedParameterValue::Enumeration(GCM_MODE),
            ),
            (IV_LENGTH_TAG, CapturedParameterValue::Integer(96)),
            (TAG_LENGTH_TAG, CapturedParameterValue::Integer(16)),
        ]
    );
}

#[test]
fn unknown_parameter_members_survive_in_original_order_and_encoding() {
    let extension_bytes = [0x00, 0x80, 0xFF];
    let cryptographic_parameters = parameters([
        (
            VENDOR_PARAMETER_TAG,
            Value::byte_string(extension_bytes.to_vec()),
        ),
        (SECOND_VENDOR_PARAMETER_TAG, Value::integer(-1_234_567)),
    ]);
    let payload = encrypt_payload(Some(cryptographic_parameters))
        .expect("an unrecognized vendor parameter is preserved opaquely");

    assert_eq!(
        parameter_members(&payload),
        [
            (
                VENDOR_PARAMETER_TAG,
                CapturedParameterValue::ByteString(extension_bytes.to_vec()),
            ),
            (
                SECOND_VENDOR_PARAMETER_TAG,
                CapturedParameterValue::Integer(-1_234_567),
            ),
        ]
    );
}

#[test]
fn decrypt_unknown_parameter_members_survive_in_original_order_and_encoding() {
    let extension_bytes = [0x00, 0x80, 0xFF];
    let cryptographic_parameters = parameters([
        (
            VENDOR_PARAMETER_TAG,
            Value::byte_string(extension_bytes.to_vec()),
        ),
        (SECOND_VENDOR_PARAMETER_TAG, Value::integer(-1_234_567)),
    ]);
    let payload = decrypt_payload(Some(cryptographic_parameters))
        .expect("an unrecognized vendor parameter is preserved opaquely");

    assert_eq!(
        parameter_members(&payload),
        [
            (
                VENDOR_PARAMETER_TAG,
                CapturedParameterValue::ByteString(extension_bytes.to_vec()),
            ),
            (
                SECOND_VENDOR_PARAMETER_TAG,
                CapturedParameterValue::Integer(-1_234_567),
            ),
        ]
    );
}

#[test]
fn encrypt_does_not_synthesize_omitted_parameters() {
    let payload = encrypt_payload(None)
        .expect("omitted Cryptographic Parameters stay absent in local payload conversion");
    let actual_tags = payload
        .view()
        .children()
        .iter()
        .map(|item| item.tag().raw())
        .collect::<Vec<_>>();

    assert_eq!(actual_tags, [UNIQUE_IDENTIFIER_TAG, DATA_TAG]);
}

#[test]
fn decrypt_does_not_synthesize_omitted_parameters() {
    let payload = decrypt_payload(None)
        .expect("omitted Cryptographic Parameters stay absent in local payload conversion");
    let actual_tags = payload
        .view()
        .children()
        .iter()
        .map(|item| item.tag().raw())
        .collect::<Vec<_>>();

    assert_eq!(actual_tags, [UNIQUE_IDENTIFIER_TAG, DATA_TAG]);
}
