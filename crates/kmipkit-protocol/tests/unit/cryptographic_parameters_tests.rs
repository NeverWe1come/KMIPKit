//! Cryptographic Parameters tests derived from OASIS KMIP v2.1 §4.16,
//! Table 59; §11.6, Table 436; and NIST SP 800-38C, Appendix A.1 (supporting
//! evidence that CCM has variable nonce lengths). These assert shared local
//! validation and preservation, not operation-specific payloads or server
//! behavior; they are source-derived tests, not official OASIS Test Cases.
//!
//! Traceability: `KMIPKIT-REQ-SPEC-4.16-001-001`,
//! `KMIPKIT-REQ-SPEC-4.16-001-002`, `KMIPKIT-REQ-SPEC-4.16-002`, and
//! `KMIPKIT-REQ-SPEC-4.16-003`.

use crate::ProtocolError;
use crate::cryptographic_parameters::validate_cryptographic_parameters;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

const BLOCK_CIPHER_MODE_TAG: u32 = 0x0042_0011;
const IV_LENGTH_TAG: u32 = 0x0042_00CD;
const TAG_LENGTH_TAG: u32 = 0x0042_00CE;
const VENDOR_PARAMETER_TAG: u32 = 0x0054_1234;
const SECOND_VENDOR_PARAMETER_TAG: u32 = 0x0054_1235;
const CTR_MODE: u32 = 6;
const CCM_MODE: u32 = 8;
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

fn validated_parameters(parameters: Structure) -> Result<Structure, ProtocolError> {
    validate_cryptographic_parameters(Some(parameters))
        .map(|parameters| parameters.expect("supplied Cryptographic Parameters remain present"))
}

fn parameter_members(parameters: &Structure) -> Vec<(u32, CapturedParameterValue)> {
    let view = parameters.view();
    view.children()
        .iter()
        .map(|item| {
            let raw_tag = item.tag().raw();
            let value = item.with_value(|value| match value {
                ValueView::ByteString(bytes) => CapturedParameterValue::ByteString(bytes.to_vec()),
                ValueView::Enumeration(value) => CapturedParameterValue::Enumeration(*value),
                ValueView::Integer(value) => CapturedParameterValue::Integer(*value),
                _ => CapturedParameterValue::Other,
            });
            (raw_tag, value)
        })
        .collect()
}

#[test]
fn variable_iv_mode_requires_iv_length_and_preserves_supplied_parameters() {
    let without_length = parameters([(BLOCK_CIPHER_MODE_TAG, Value::enumeration(CTR_MODE))]);
    assert!(validated_parameters(without_length).is_err());

    let with_length = parameters([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(CTR_MODE)),
        (IV_LENGTH_TAG, Value::integer(128)),
    ]);
    let retained =
        validated_parameters(with_length).expect("CTR is accepted when its IV length is supplied");

    assert_eq!(
        parameter_members(&retained),
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
fn ccm_requires_iv_length_without_requiring_gcm_tag_length() {
    let without_iv_length = parameters([(BLOCK_CIPHER_MODE_TAG, Value::enumeration(CCM_MODE))]);
    assert!(validated_parameters(without_iv_length).is_err());

    let with_iv_length = parameters([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(CCM_MODE)),
        (IV_LENGTH_TAG, Value::integer(96)),
    ]);
    let retained = validated_parameters(with_iv_length)
        .expect("CCM is accepted when its IV length is supplied without a GCM Tag Length");

    assert_eq!(
        parameter_members(&retained),
        [
            (
                BLOCK_CIPHER_MODE_TAG,
                CapturedParameterValue::Enumeration(CCM_MODE),
            ),
            (IV_LENGTH_TAG, CapturedParameterValue::Integer(96)),
        ]
    );
}

#[test]
fn gcm_requires_tag_length_and_preserves_supplied_parameters() {
    let without_tag_length = parameters([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(GCM_MODE)),
        (IV_LENGTH_TAG, Value::integer(96)),
    ]);
    assert!(validated_parameters(without_tag_length).is_err());

    let with_tag_length = parameters([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(GCM_MODE)),
        (IV_LENGTH_TAG, Value::integer(96)),
        (TAG_LENGTH_TAG, Value::integer(16)),
    ]);
    let retained = validated_parameters(with_tag_length)
        .expect("GCM is accepted when its IV and tag lengths are supplied");

    assert_eq!(
        parameter_members(&retained),
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
    let supplied = parameters([
        (
            VENDOR_PARAMETER_TAG,
            Value::byte_string(extension_bytes.to_vec()),
        ),
        (SECOND_VENDOR_PARAMETER_TAG, Value::integer(-1_234_567)),
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(u32::MAX)),
    ]);
    let retained = validated_parameters(supplied)
        .expect("unknown parameter members and mode values are preserved opaquely");

    assert_eq!(
        parameter_members(&retained),
        [
            (
                VENDOR_PARAMETER_TAG,
                CapturedParameterValue::ByteString(extension_bytes.to_vec()),
            ),
            (
                SECOND_VENDOR_PARAMETER_TAG,
                CapturedParameterValue::Integer(-1_234_567),
            ),
            (
                BLOCK_CIPHER_MODE_TAG,
                CapturedParameterValue::Enumeration(u32::MAX),
            ),
        ]
    );
}

#[test]
fn omitted_cryptographic_parameters_remain_absent_without_validation() {
    assert!(
        validate_cryptographic_parameters(None)
            .expect("omitted parameters need no local inspection")
            .is_none()
    );
}
