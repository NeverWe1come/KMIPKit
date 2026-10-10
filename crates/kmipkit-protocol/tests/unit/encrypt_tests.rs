//! Encrypt request tests derived from OASIS KMIP Specification v2.1 §6.1.17,
//! Table 214. Cryptographic Parameters are defined by §4.16, and Data encodings
//! by §7.9, Tables 360–361. These are source-derived tests, not claims that an
//! official OASIS Test Case passed.
//!
//! Traceability: `KMIPKIT-REQ-SPEC-6.1.17-001-001`,
//! `KMIPKIT-REQ-SPEC-6.1.17-001-002`, and `KMIPKIT-ELEM-OP-C2S-ENCRYPT`.

use crate::operation_test_support::item;
use crate::{EncryptRequest, OperationData, SecretBytes, UniqueIdentifier};
use kmipkit_ttlv::{Item, Structure, Value, ValueView};

const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
const DATA_TAG: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE_TAG: u32 = 0x0042_003D;
const CORRELATION_VALUE_TAG: u32 = 0x0042_00D6;
const INIT_INDICATOR_TAG: u32 = 0x0042_00D7;
const FINAL_INDICATOR_TAG: u32 = 0x0042_00D8;
const AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG: u32 = 0x0042_00FE;
const FIRST_VENDOR_PARAMETER_TAG: u32 = 0x0054_1234;
const SECOND_VENDOR_PARAMETER_TAG: u32 = 0x0054_1235;

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
            .expect("the Table 214 test payload stays within the TTLV depth limit");
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

fn base_request(data: OperationData) -> EncryptRequest {
    EncryptRequest::new(
        Some(UniqueIdentifier::TextString("encrypt-key-id".to_owned())),
        Some(data),
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
fn request_emits_present_table_214_members_in_exact_order_and_preserves_values() {
    let data = vec![0x00, 0x7F, 0x80, 0xFF];
    let iv = vec![0x11, 0x22, 0x33];
    let correlation = vec![0x44, 0x55];
    let additional_data = vec![0x66, 0x77, 0x88];
    let request = base_request(OperationData::ByteString(SecretBytes::new(data.clone())))
        .with_cryptographic_parameters(vendor_parameters())
        .with_iv_counter_nonce(SecretBytes::new(iv.clone()))
        .with_correlation_value(SecretBytes::new(correlation.clone()))
        .with_init_indicator(true)
        .with_final_indicator(false)
        .with_authenticated_encryption_additional_data(SecretBytes::new(additional_data.clone()));

    let actual = request
        .to_ttlv_payload()
        .expect("the Table 214 request preserves its supplied optional fields");
    let expected = payload([
        (
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("encrypt-key-id".to_owned()),
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
        ],
        "Table 214 fields must retain their normative order"
    );
    assert_eq!(snapshot_structure(&actual), snapshot_structure(&expected));
}

#[test]
fn request_omits_cryptographic_parameters_when_the_caller_omits_them() {
    let actual = base_request(OperationData::Enumeration(u32::MAX))
        .to_ttlv_payload()
        .expect("omitted Cryptographic Parameters remain absent");

    assert_eq!(member_tags(&actual), [UNIQUE_IDENTIFIER_TAG, DATA_TAG]);
}

#[test]
fn request_preserves_supplied_cryptographic_parameters_members_and_order() {
    let supplied = vendor_parameters();
    let actual = base_request(OperationData::Integer(i32::MIN))
        .with_cryptographic_parameters(supplied)
        .to_ttlv_payload()
        .expect("opaque Cryptographic Parameters members remain ordered");

    let actual_view = actual.view();
    let parameter_item = actual_view
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
fn request_preserves_each_data_encoding_allowed_by_section_7_9() {
    let cases = [
        (
            OperationData::ByteString(SecretBytes::new(vec![0x00, 0x7F, 0x80, 0xFF])),
            ValueSnapshot::ByteString(vec![0x00, 0x7F, 0x80, 0xFF]),
        ),
        (
            OperationData::Enumeration(u32::MAX),
            ValueSnapshot::Enumeration(u32::MAX),
        ),
        (
            OperationData::Integer(i32::MIN),
            ValueSnapshot::Integer(i32::MIN),
        ),
    ];

    for (data, expected_value) in cases {
        let actual = base_request(data)
            .to_ttlv_payload()
            .expect("Data retains its caller-selected §7.9 encoding");
        let actual_view = actual.view();
        let data_item = actual_view
            .children()
            .iter()
            .find(|child| child.tag().raw() == DATA_TAG)
            .expect("the requested Data member is present");

        assert_eq!(
            snapshot_item(data_item),
            ItemSnapshot {
                tag: DATA_TAG,
                value: expected_value,
            },
            "Byte String, Enumeration, and Integer Data must retain the exact wire value"
        );
    }
}
