//! OASIS KMIP Specification v2.1: §§9.2, 9.6, 9.10, 10.1.2; Tables 395–399,
//! 421, 432, and 435. These derived property tests exercise in-memory conversion;
//! they are not official OASIS fixtures and do not assert wire validity.
//!
//! Traceability: KMIPKIT-0006-FR-013, FR-014, FR-017, FR-022; SC-003,
//! SC-004, and SC-006.

use kmipkit_protocol::{RequestMessage, ResponseMessage};
use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Tag, Value, ValueView};
use quickcheck::{Arbitrary, Gen, QuickCheck};

const PROPERTY_SEED: u64 = 0x4B4D_4950_4B49_5430;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const ASYNCHRONOUS_INDICATOR: u32 = 0x0042_0007;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0002;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const REQUEST_HEADER: u32 = 0x0042_0077;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_STATUS: u32 = 0x0042_007F;
const TIME_STAMP: u32 = 0x0042_0092;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000E;
const BATCH_ORDER_OPTION: u32 = 0x0042_0010;
const OPERATION: u32 = 0x0042_005C;
const ATTESTATION_TYPE: u32 = 0x0042_0003;
const SERVER_CORRELATION_VALUE: u32 = 0x0042_0106;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;

#[derive(Clone, Debug)]
struct GeneratedCase {
    raw_enumeration: u32,
    raw_bit_mask: u32,
    extension_tag: u32,
    payload: Vec<u8>,
    timestamp: i64,
    item_count: usize,
}

impl Arbitrary for GeneratedCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        let extension_tag = 0x0054_0000 | (u32::arbitrary(generator) & 0x0000_FFFF);
        let payload_len = usize::arbitrary(generator) % 1025;
        let payload = (0..payload_len).map(|_| u8::arbitrary(generator)).collect();
        Self {
            raw_enumeration: u32::arbitrary(generator),
            raw_bit_mask: u32::arbitrary(generator),
            extension_tag,
            payload,
            timestamp: i64::arbitrary(generator),
            item_count: 1 + (usize::arbitrary(generator) % 8),
        }
    }
}

#[derive(Clone, Debug)]
struct CorrelationSample(Vec<u8>);

impl Arbitrary for CorrelationSample {
    fn arbitrary(generator: &mut Gen) -> Self {
        let length = 1 + (usize::arbitrary(generator) % 1024);
        Self((0..length).map(|_| u8::arbitrary(generator)).collect())
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Node {
    tag: u32,
    value: NodeValue,
}

#[derive(Debug, Eq, PartialEq)]
enum NodeValue {
    Structure(Vec<Node>),
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
}

fn checked_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the property generator emits a 24-bit tag")
        .try_checked()
        .expect("the property generator emits an allocation-valid tag")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(checked_tag(raw), value).expect("a checked tag and Value form an Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("the generated test tree stays below the model depth limit");
    }
    structure
}

fn version_value() -> Value {
    Value::structure(structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]))
}

fn opaque_payload(case: &GeneratedCase) -> Structure {
    let nested = structure([
        item(case.extension_tag, Value::byte_string(case.payload.clone())),
        item(case.extension_tag, Value::enumeration(case.raw_enumeration)),
    ]);
    structure([
        item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(case.raw_bit_mask as i32),
        ),
        item(case.extension_tag, Value::structure(nested)),
        item(
            case.extension_tag,
            Value::integer(case.raw_enumeration as i32),
        ),
    ])
}

fn request_tree(case: &GeneratedCase) -> Structure {
    let item_count = case.item_count as i32;
    let mut header_fields = vec![
        item(PROTOCOL_VERSION, version_value()),
        item(case.extension_tag, Value::enumeration(case.raw_enumeration)),
        item(
            SERVER_CORRELATION_VALUE,
            Value::text_string("server-correlation".into()),
        ),
        item(
            ASYNCHRONOUS_INDICATOR,
            Value::enumeration(case.raw_enumeration),
        ),
        item(ATTESTATION_TYPE, Value::enumeration(case.raw_enumeration)),
        item(ATTESTATION_TYPE, Value::enumeration(case.raw_bit_mask)),
    ];
    if case.item_count > 1 {
        header_fields.push(item(
            BATCH_ERROR_CONTINUATION_OPTION,
            Value::enumeration(case.raw_enumeration),
        ));
        header_fields.push(item(BATCH_ORDER_OPTION, Value::boolean(true)));
    }
    header_fields.push(item(BATCH_COUNT, Value::integer(item_count)));
    let mut message_fields = vec![item(
        REQUEST_HEADER,
        Value::structure(structure(header_fields)),
    )];
    message_fields.push(item(
        case.extension_tag,
        Value::byte_string(case.payload.clone()),
    ));
    for index in 0..case.item_count {
        let mut batch_fields = vec![item(OPERATION, Value::enumeration(case.raw_enumeration))];
        if case.item_count > 1 {
            batch_fields.push(item(
                UNIQUE_BATCH_ITEM_ID,
                Value::byte_string(vec![index as u8 + 1]),
            ));
        }
        batch_fields.push(item(
            REQUEST_PAYLOAD,
            Value::structure(opaque_payload(case)),
        ));
        message_fields.push(item(BATCH_ITEM, Value::structure(structure(batch_fields))));
    }
    structure(message_fields)
}

fn response_tree(case: &GeneratedCase) -> Structure {
    let header = structure([
        item(PROTOCOL_VERSION, version_value()),
        item(TIME_STAMP, Value::date_time(case.timestamp)),
        item(ATTESTATION_TYPE, Value::enumeration(case.raw_enumeration)),
        item(ATTESTATION_TYPE, Value::enumeration(case.raw_bit_mask)),
        item(
            SERVER_CORRELATION_VALUE,
            Value::text_string("server-response".into()),
        ),
        item(BATCH_COUNT, Value::integer(case.item_count as i32)),
    ]);
    let mut message_fields = vec![item(RESPONSE_HEADER, Value::structure(header))];
    message_fields.push(item(
        case.extension_tag,
        Value::enumeration(case.raw_enumeration),
    ));
    for _ in 0..case.item_count {
        let batch_item = structure([
            item(OPERATION, Value::enumeration(case.raw_enumeration)),
            item(RESULT_STATUS, Value::enumeration(0)),
            item(RESPONSE_PAYLOAD, Value::structure(opaque_payload(case))),
        ]);
        message_fields.push(item(BATCH_ITEM, Value::structure(batch_item)));
    }
    structure(message_fields)
}

fn pending_response(correlation: &[u8]) -> Structure {
    let header = structure([
        item(PROTOCOL_VERSION, version_value()),
        item(TIME_STAMP, Value::date_time(1_751_100_123)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(RESULT_STATUS, Value::enumeration(2)),
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(correlation.to_vec()),
        ),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);
    structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ])
}

fn snapshot(view: StructureView<'_>) -> Vec<Node> {
    view.children()
        .iter()
        .map(|child| {
            let value = child.with_value(|value| match value {
                ValueView::Structure(nested) => NodeValue::Structure(snapshot(nested)),
                ValueView::Integer(raw) => NodeValue::Integer(*raw),
                ValueView::LongInteger(raw) => NodeValue::LongInteger(*raw),
                ValueView::BigInteger(raw) => NodeValue::BigInteger(raw.to_vec()),
                ValueView::Enumeration(raw) => NodeValue::Enumeration(*raw),
                ValueView::Boolean(raw) => NodeValue::Boolean(*raw),
                ValueView::TextString(raw) => NodeValue::TextString(raw.to_owned()),
                ValueView::ByteString(raw) => NodeValue::ByteString(raw.to_vec()),
                ValueView::DateTime(raw) => NodeValue::DateTime(*raw),
                ValueView::Interval(raw) => NodeValue::Interval(*raw),
                ValueView::DateTimeExtended(raw) => NodeValue::DateTimeExtended(*raw),
                _ => unreachable!("the ValueView domain is explicitly covered above"),
            });
            Node {
                tag: child.tag().raw(),
                value,
            }
        })
        .collect()
}

fn request_round_trip_preserves_generated_tree(case: GeneratedCase) -> bool {
    let source = request_tree(&case);
    let before = snapshot(source.view());
    let Ok(parsed) = RequestMessage::try_from_ttlv(source) else {
        return false;
    };
    let round_trip = parsed.into_ttlv();
    snapshot(round_trip.view()) == before
}

fn response_round_trip_preserves_generated_tree(case: GeneratedCase) -> bool {
    let source = response_tree(&case);
    let before = snapshot(source.view());
    let Ok(parsed) = ResponseMessage::try_from_ttlv(source) else {
        return false;
    };
    let round_trip = parsed.into_ttlv();
    snapshot(round_trip.view()) == before
}

fn asynchronous_correlation_round_trip_preserves_exact_bytes(sample: CorrelationSample) -> bool {
    let source = pending_response(&sample.0);
    let Ok(parsed) = ResponseMessage::try_from_ttlv(source) else {
        return false;
    };
    let round_trip = parsed.into_ttlv();
    let actual = round_trip.view().children()[1].with_value(|value| match value {
        ValueView::Structure(batch_item) => batch_item
            .children()
            .iter()
            .find(|child| child.tag().raw() == ASYNCHRONOUS_CORRELATION_VALUE)
            .and_then(|child| {
                child.with_value(|value| match value {
                    ValueView::ByteString(bytes) => Some(bytes.to_vec()),
                    _ => None,
                })
            }),
        _ => None,
    });
    actual.as_deref() == Some(sample.0.as_slice())
}

#[test]
fn request_message_conversion_preserves_bounded_generated_trees() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(64, PROPERTY_SEED))
        .tests(256)
        .quickcheck(request_round_trip_preserves_generated_tree as fn(GeneratedCase) -> bool);
}

#[test]
fn response_message_conversion_preserves_bounded_generated_trees() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(64, PROPERTY_SEED))
        .tests(256)
        .quickcheck(response_round_trip_preserves_generated_tree as fn(GeneratedCase) -> bool);
}

#[test]
fn asynchronous_correlation_bytes_survive_generated_pending_round_trips() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(1024, PROPERTY_SEED))
        .tests(256)
        .quickcheck(
            asynchronous_correlation_round_trip_preserves_exact_bytes
                as fn(CorrelationSample) -> bool,
        );
}

#[test]
fn duplicate_known_singleton_header_field_is_rejected() {
    let request_header = structure([
        item(PROTOCOL_VERSION, version_value()),
        item(PROTOCOL_VERSION, version_value()),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(OPERATION, Value::enumeration(1)),
        item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
    ]);
    let request = structure([
        item(REQUEST_HEADER, Value::structure(request_header)),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ]);

    assert!(RequestMessage::try_from_ttlv(request).is_err());
}

#[test]
fn duplicate_response_timestamp_is_rejected() {
    let response_header = structure([
        item(PROTOCOL_VERSION, version_value()),
        item(TIME_STAMP, Value::date_time(1)),
        item(TIME_STAMP, Value::date_time(2)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(RESULT_STATUS, Value::enumeration(0)),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);
    let response = structure([
        item(RESPONSE_HEADER, Value::structure(response_header)),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ]);

    assert!(ResponseMessage::try_from_ttlv(response).is_err());
}

#[test]
fn message_formatting_and_validation_errors_redact_payload_sentinels() {
    const SECRET_SENTINEL: &str = "KMIPKIT-TEST-SECRET-DO-NOT-PRINT";
    const RESULT_SENTINEL: &str = "KMIPKIT-TEST-RESULT-DO-NOT-PRINT";
    let secret_tag = 0x0054_0010;
    let payload = structure([item(
        secret_tag,
        Value::byte_string(SECRET_SENTINEL.as_bytes().to_vec()),
    )]);
    let request_header = structure([
        item(PROTOCOL_VERSION, version_value()),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let request_batch_item = structure([
        item(OPERATION, Value::enumeration(1)),
        item(REQUEST_PAYLOAD, Value::structure(payload)),
    ]);
    let request = structure([
        item(REQUEST_HEADER, Value::structure(request_header)),
        item(BATCH_ITEM, Value::structure(request_batch_item)),
    ]);
    let request = RequestMessage::try_from_ttlv(request)
        .expect("an allocated opaque field remains valid generic payload");
    let debug = format!("{request:?}");
    let display = format!("{request}");
    assert!(!debug.contains(SECRET_SENTINEL));
    assert!(!display.contains(SECRET_SENTINEL));

    let response_header = structure([
        item(PROTOCOL_VERSION, version_value()),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let failure_item = structure([
        item(RESULT_STATUS, Value::enumeration(1)),
        item(RESULT_REASON, Value::enumeration(0)),
        item(0x0042_0080, Value::text_string(RESULT_SENTINEL.to_owned())),
    ]);
    let response = structure([
        item(RESPONSE_HEADER, Value::structure(response_header)),
        item(BATCH_ITEM, Value::structure(failure_item)),
    ]);
    let response = ResponseMessage::try_from_ttlv(response)
        .expect("a Failure may carry a Result Message and no Response Payload");
    let debug = format!("{response:?}");
    let display = format!("{response}");
    assert!(!debug.contains(RESULT_SENTINEL));
    assert!(!display.contains(RESULT_SENTINEL));

    let success_with_result_message = structure([
        item(RESULT_STATUS, Value::enumeration(0)),
        item(0x0042_0080, Value::text_string(RESULT_SENTINEL.to_owned())),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);
    let response_header = structure([
        item(PROTOCOL_VERSION, version_value()),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let invalid_response = structure([
        item(RESPONSE_HEADER, Value::structure(response_header)),
        item(BATCH_ITEM, Value::structure(success_with_result_message)),
    ]);
    let error = ResponseMessage::try_from_ttlv(invalid_response)
        .expect_err("Success forbids Result Message");
    assert!(!format!("{error:?}").contains(RESULT_SENTINEL));
    assert!(!error.to_string().contains(RESULT_SENTINEL));

    let malformed_request = structure([
        item(
            REQUEST_HEADER,
            Value::structure(structure([
                item(PROTOCOL_VERSION, version_value()),
                item(PROTOCOL_VERSION, version_value()),
                item(BATCH_COUNT, Value::integer(1)),
            ])),
        ),
        item(
            BATCH_ITEM,
            Value::structure(structure([
                item(OPERATION, Value::enumeration(1)),
                item(
                    REQUEST_PAYLOAD,
                    Value::structure(structure([item(
                        secret_tag,
                        Value::byte_string(SECRET_SENTINEL.as_bytes().to_vec()),
                    )])),
                ),
            ])),
        ),
    ]);
    let error = RequestMessage::try_from_ttlv(malformed_request)
        .expect_err("duplicate singleton fields are rejected without retaining payloads");
    assert!(!format!("{error:?}").contains(SECRET_SENTINEL));
    assert!(!error.to_string().contains(SECRET_SENTINEL));
}
