//! Decrypt request tests derived from OASIS KMIP Specification v2.1 §6.1.11,
//! Table 196. Cryptographic Parameters are defined by §4.16; Authenticated
//! Encryption Additional Data and Tag by §§7.3–7.4; Correlation Value by §7.8;
//! Data encodings by §7.9, Tables 360–361; Init Indicator by §7.17; and Final
//! Indicator by §7.14. Fixture-derived assertions use Decrypt items from OASIS
//! KMIP Test Cases v2.1 Committee Note 01, §2.101, TC-STREAM-ENCDEC-1-21 steps 2,
//! 6, 7, and 8. They are operation-item evidence, not a complete official-case
//! pass.
//!
//! Traceability: `KMIPKIT-REQ-SPEC-6.1.11-001-001`,
//! `KMIPKIT-REQ-SPEC-6.1.11-001-002`, `KMIPKIT-REQ-SPEC-6.1.11-005`,
//! `KMIPKIT-REQ-SPEC-6.1.11-006`, and `KMIPKIT-ELEM-OP-C2S-DECRYPT`.

use crate::operation_test_support::item;
use crate::{DecryptRequest, OperationData, SecretBytes, UniqueIdentifier};
use kmipkit_ttlv::{Item, Structure, Value, ValueView};

const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
const DATA_TAG: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE_TAG: u32 = 0x0042_003D;
const CORRELATION_VALUE_TAG: u32 = 0x0042_00D6;
const INIT_INDICATOR_TAG: u32 = 0x0042_00D7;
const FINAL_INDICATOR_TAG: u32 = 0x0042_00D8;
const AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG: u32 = 0x0042_00FE;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;
const BLOCK_CIPHER_MODE_TAG: u32 = 0x0042_0011;
const PADDING_METHOD_TAG: u32 = 0x0042_0042;
const FIRST_VENDOR_PARAMETER_TAG: u32 = 0x0054_1234;
const SECOND_VENDOR_PARAMETER_TAG: u32 = 0x0054_1235;
const FIXTURE_UNIQUE_IDENTIFIER: &str = "kmipkit-test-unique-id-0";
const FIXTURE_CORRELATION_VALUE: &[u8] = b"kmipkit-test-correlation-value-0";

#[derive(Debug, Eq, PartialEq)]
enum ValueSnapshot {
    Structure(Vec<ItemSnapshot>),
    Integer(i32),
    LongInteger(i64),
    BigInteger(Vec<u8>),
    Enumeration(u32),
    Boolean(bool),
    TextString(String),
    ByteString(Vec<u8>),
    DateTime(i64),
    Interval(u32),
    DateTimeExtended(i64),
    Unsupported,
}

#[derive(Debug, Eq, PartialEq)]
struct ItemSnapshot {
    tag: u32,
    value: ValueSnapshot,
}

fn payload(items: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    let mut payload = Structure::new();
    for (raw_tag, value) in items {
        payload
            .try_push(item(raw_tag, value))
            .expect("the Table 196 test payload stays within the TTLV depth limit");
    }
    payload
}

fn vendor_parameters() -> Structure {
    payload([
        (
            FIRST_VENDOR_PARAMETER_TAG,
            Value::byte_string(vec![0x00, 0x80, 0xFF]),
        ),
        (SECOND_VENDOR_PARAMETER_TAG, Value::integer(-1_234_567)),
    ])
}

fn fixture_cbc_pkcs5_parameters() -> Structure {
    // §11.6 assigns CBC value 1; §11.37 assigns PKCS5 value 3.
    payload([
        (BLOCK_CIPHER_MODE_TAG, Value::enumeration(1)),
        (PADDING_METHOD_TAG, Value::enumeration(3)),
    ])
}

fn base_request(data: OperationData) -> DecryptRequest {
    DecryptRequest::new(
        Some(UniqueIdentifier::TextString("decrypt-key-id".to_owned())),
        Some(data),
    )
}

fn fixture_request(data: &[u8]) -> DecryptRequest {
    DecryptRequest::new(
        Some(UniqueIdentifier::TextString(
            FIXTURE_UNIQUE_IDENTIFIER.to_owned(),
        )),
        Some(OperationData::ByteString(SecretBytes::new(data.to_vec()))),
    )
}

fn member_tags(structure: &Structure) -> Vec<u32> {
    structure
        .view()
        .children()
        .iter()
        .map(|child| child.tag().raw())
        .collect()
}

fn snapshot_structure(structure: &Structure) -> Vec<ItemSnapshot> {
    structure
        .view()
        .children()
        .iter()
        .map(snapshot_item)
        .collect()
}

fn snapshot_item(item: &Item) -> ItemSnapshot {
    ItemSnapshot {
        tag: item.tag().raw(),
        value: item.with_value(snapshot_value),
    }
}

fn snapshot_value(value: ValueView<'_>) -> ValueSnapshot {
    match value {
        ValueView::Structure(value) => {
            ValueSnapshot::Structure(value.children().iter().map(snapshot_item).collect())
        }
        ValueView::Integer(value) => ValueSnapshot::Integer(*value),
        ValueView::LongInteger(value) => ValueSnapshot::LongInteger(*value),
        ValueView::BigInteger(value) => ValueSnapshot::BigInteger(value.to_vec()),
        ValueView::Enumeration(value) => ValueSnapshot::Enumeration(*value),
        ValueView::Boolean(value) => ValueSnapshot::Boolean(*value),
        ValueView::TextString(value) => ValueSnapshot::TextString((*value).to_owned()),
        ValueView::ByteString(value) => ValueSnapshot::ByteString(value.to_vec()),
        ValueView::DateTime(value) => ValueSnapshot::DateTime(*value),
        ValueView::Interval(value) => ValueSnapshot::Interval(*value),
        ValueView::DateTimeExtended(value) => ValueSnapshot::DateTimeExtended(*value),
        _ => ValueSnapshot::Unsupported,
    }
}

#[test]
fn request_emits_present_table_196_members_in_exact_order_and_accepts_decrypt_tag() {
    let data = vec![0x00, 0x7F, 0x80, 0xFF];
    let iv = vec![0x11, 0x22, 0x33];
    let correlation = vec![0x44, 0x55];
    let additional_data = vec![0x66, 0x77, 0x88];
    let authentication_tag = vec![0x99, 0xAA, 0xBB, 0xCC];
    let request = base_request(OperationData::ByteString(SecretBytes::new(data.clone())))
        .with_cryptographic_parameters(vendor_parameters())
        .with_iv_counter_nonce(SecretBytes::new(iv.clone()))
        .with_correlation_value(SecretBytes::new(correlation.clone()))
        .with_init_indicator(true)
        .with_final_indicator(false)
        .with_authenticated_encryption_additional_data(SecretBytes::new(additional_data.clone()))
        .with_authenticated_encryption_tag(SecretBytes::new(authentication_tag.clone()));

    let actual = request
        .to_ttlv_payload()
        .expect("the Table 196 request preserves its supplied optional fields");
    let expected = payload([
        (
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("decrypt-key-id".to_owned()),
        ),
        (
            CRYPTOGRAPHIC_PARAMETERS_TAG,
            Value::structure(vendor_parameters()),
        ),
        (DATA_TAG, Value::byte_string(data)),
        (IV_COUNTER_NONCE_TAG, Value::byte_string(iv)),
        (CORRELATION_VALUE_TAG, Value::byte_string(correlation)),
        (INIT_INDICATOR_TAG, Value::boolean(true)),
        (FINAL_INDICATOR_TAG, Value::boolean(false)),
        (
            AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG,
            Value::byte_string(additional_data),
        ),
        (
            AUTHENTICATED_ENCRYPTION_TAG,
            Value::byte_string(authentication_tag),
        ),
    ]);

    assert_eq!(
        member_tags(&actual),
        [
            UNIQUE_IDENTIFIER_TAG,
            CRYPTOGRAPHIC_PARAMETERS_TAG,
            DATA_TAG,
            IV_COUNTER_NONCE_TAG,
            CORRELATION_VALUE_TAG,
            INIT_INDICATOR_TAG,
            FINAL_INDICATOR_TAG,
            AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG,
            AUTHENTICATED_ENCRYPTION_TAG,
        ],
        "Table 196 fields, including the Decrypt-only tag input, must retain normative order"
    );
    assert_eq!(snapshot_structure(&actual), snapshot_structure(&expected));
}

#[test]
fn request_omits_cryptographic_parameters_and_iv_when_the_caller_omits_them() {
    let actual = base_request(OperationData::Enumeration(u32::MAX))
        .to_ttlv_payload()
        .expect("omitted Cryptographic Parameters and IV remain absent");

    assert_eq!(member_tags(&actual), [UNIQUE_IDENTIFIER_TAG, DATA_TAG]);
}

#[test]
fn request_preserves_supplied_cryptographic_parameters_members_and_order() {
    let supplied = vendor_parameters();
    let actual = base_request(OperationData::Integer(i32::MIN))
        .with_cryptographic_parameters(supplied)
        .to_ttlv_payload()
        .expect("opaque Cryptographic Parameters members remain ordered");

    let parameter_item = actual
        .view()
        .children()
        .iter()
        .find(|child| child.tag().raw() == CRYPTOGRAPHIC_PARAMETERS_TAG)
        .expect("the supplied Cryptographic Parameters member is present");
    let actual_parameters = parameter_item.with_value(|value| match value {
        ValueView::Structure(value) => Some(
            value
                .children()
                .iter()
                .map(snapshot_item)
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });

    assert_eq!(
        actual_parameters,
        Some(vec![
            ItemSnapshot {
                tag: FIRST_VENDOR_PARAMETER_TAG,
                value: ValueSnapshot::ByteString(vec![0x00, 0x80, 0xFF]),
            },
            ItemSnapshot {
                tag: SECOND_VENDOR_PARAMETER_TAG,
                value: ValueSnapshot::Integer(-1_234_567),
            },
        ]),
        "§4.16 generic members and vendor values must survive in their original order"
    );
}

#[test]
fn fixture_derived_decrypt_operation_items_preserve_steps_2_6_7_and_8() {
    let iv = vec![
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
        0x16,
    ];
    let step_2_data = vec![
        0x79, 0xAB, 0xC5, 0xC2, 0x38, 0x68, 0xAD, 0x84, 0xD3, 0x88, 0xCE, 0x61, 0x11, 0x0A, 0x62,
        0x74, 0x2B, 0xDA, 0x19, 0xD6, 0x94, 0xBB, 0xCB, 0x75, 0x7D, 0xD0, 0x66, 0x17, 0xC0, 0xD8,
        0x0F, 0xB1, 0xDF, 0x2E, 0x71, 0x86, 0x4A, 0xD9, 0x63, 0x3D, 0x7D, 0x79, 0x7E, 0x30, 0x86,
        0x0D, 0xF0, 0x0D,
    ];
    let step_6_data = vec![
        0x79, 0xAB, 0xC5, 0xC2, 0x38, 0x68, 0xAD, 0x84, 0xD3, 0x88, 0xCE, 0x61, 0x11, 0x0A, 0x62,
        0x74,
    ];
    let step_7_data = vec![
        0x2B, 0xDA, 0x19, 0xD6, 0x94, 0xBB, 0xCB, 0x75, 0x7D, 0xD0, 0x66, 0x17, 0xC0, 0xD8, 0x0F,
        0xB1,
    ];
    let step_8_data = vec![
        0xDF, 0x2E, 0x71, 0x86, 0x4A, 0xD9, 0x63, 0x3D, 0x7D, 0x79, 0x7E, 0x30, 0x86, 0x0D, 0xF0,
        0x0D,
    ];
    let step_2_request = fixture_request(&step_2_data)
        .with_cryptographic_parameters(fixture_cbc_pkcs5_parameters())
        .with_iv_counter_nonce(SecretBytes::new(iv.clone()));
    let step_6_request = fixture_request(&step_6_data)
        .with_cryptographic_parameters(fixture_cbc_pkcs5_parameters())
        .with_iv_counter_nonce(SecretBytes::new(iv.clone()))
        .with_init_indicator(true);
    let step_7_request = fixture_request(&step_7_data)
        .with_correlation_value(SecretBytes::new(FIXTURE_CORRELATION_VALUE.to_vec()));
    let step_8_request = fixture_request(&step_8_data)
        .with_correlation_value(SecretBytes::new(FIXTURE_CORRELATION_VALUE.to_vec()))
        .with_final_indicator(true);

    let cases = [
        (
            "TC-STREAM-ENCDEC-1-21 step=2",
            step_2_request,
            payload([
                (
                    UNIQUE_IDENTIFIER_TAG,
                    Value::text_string(FIXTURE_UNIQUE_IDENTIFIER.to_owned()),
                ),
                (
                    CRYPTOGRAPHIC_PARAMETERS_TAG,
                    Value::structure(fixture_cbc_pkcs5_parameters()),
                ),
                (DATA_TAG, Value::byte_string(step_2_data)),
                (IV_COUNTER_NONCE_TAG, Value::byte_string(iv.clone())),
            ]),
        ),
        (
            "TC-STREAM-ENCDEC-1-21 step=6",
            step_6_request,
            payload([
                (
                    UNIQUE_IDENTIFIER_TAG,
                    Value::text_string(FIXTURE_UNIQUE_IDENTIFIER.to_owned()),
                ),
                (
                    CRYPTOGRAPHIC_PARAMETERS_TAG,
                    Value::structure(fixture_cbc_pkcs5_parameters()),
                ),
                (DATA_TAG, Value::byte_string(step_6_data)),
                (IV_COUNTER_NONCE_TAG, Value::byte_string(iv)),
                (INIT_INDICATOR_TAG, Value::boolean(true)),
            ]),
        ),
        (
            "TC-STREAM-ENCDEC-1-21 step=7",
            step_7_request,
            payload([
                (
                    UNIQUE_IDENTIFIER_TAG,
                    Value::text_string(FIXTURE_UNIQUE_IDENTIFIER.to_owned()),
                ),
                (DATA_TAG, Value::byte_string(step_7_data)),
                (
                    CORRELATION_VALUE_TAG,
                    Value::byte_string(FIXTURE_CORRELATION_VALUE.to_vec()),
                ),
            ]),
        ),
        (
            "TC-STREAM-ENCDEC-1-21 step=8",
            step_8_request,
            payload([
                (
                    UNIQUE_IDENTIFIER_TAG,
                    Value::text_string(FIXTURE_UNIQUE_IDENTIFIER.to_owned()),
                ),
                (DATA_TAG, Value::byte_string(step_8_data)),
                (
                    CORRELATION_VALUE_TAG,
                    Value::byte_string(FIXTURE_CORRELATION_VALUE.to_vec()),
                ),
                (FINAL_INDICATOR_TAG, Value::boolean(true)),
            ]),
        ),
    ];

    for (step_identity, request, expected) in cases {
        let actual = request
            .to_ttlv_payload()
            .expect("fixture-derived Decrypt request item is preserved");

        assert_eq!(
            snapshot_structure(&actual),
            snapshot_structure(&expected),
            "{step_identity} fixture-derived operation-item evidence must retain exact values and order"
        );
    }
}
