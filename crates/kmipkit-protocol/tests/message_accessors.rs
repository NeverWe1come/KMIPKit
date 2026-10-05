//! OASIS KMIP Specification v2.1 §§8.1–8.6 and 9.2–9.21;
//! Tables 395–399, 408–409, 418, and 421.
//!
//! Traceability: KMIPKIT-0006-FR-001 through FR-005, FR-008 through FR-011,
//! FR-018 through FR-022; SC-001, SC-002, SC-003, and SC-006.

use kmipkit_protocol::{RequestMessage, ResponseMessage};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const ASYNCHRONOUS_INDICATOR: u32 = 0x0042_0007;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const ATTESTATION_CAPABLE_INDICATOR: u32 = 0x0042_00D3;
const ATTESTATION_TYPE: u32 = 0x0042_00C7;
const AUTHENTICATION: u32 = 0x0042_000C;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000E;
const BATCH_ITEM: u32 = 0x0042_000F;
const BATCH_ORDER_OPTION: u32 = 0x0042_0010;
const CLIENT_CORRELATION_VALUE: u32 = 0x0042_0105;
const CRITICALITY_INDICATOR: u32 = 0x0042_0026;
const EPHEMERAL: u32 = 0x0042_0154;
const MAXIMUM_RESPONSE_SIZE: u32 = 0x0042_0050;
const MESSAGE_EXTENSION: u32 = 0x0042_0051;
const NONCE: u32 = 0x0042_00C8;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_HEADER: u32 = 0x0042_0077;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_STATUS: u32 = 0x0042_007F;
const SERVER_CORRELATION_VALUE: u32 = 0x0042_0106;
const SERVER_HASHED_PASSWORD: u32 = 0x0042_0155;
const TIME_STAMP: u32 = 0x0042_0092;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const VENDOR_EXTENSION: u32 = 0x0042_009C;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("the fixture tag is allocated by KMIP 2.1")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("the fixture tag and value form an item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("the fixture stays within the model depth limit");
    }
    structure
}

fn version() -> Value {
    Value::structure(structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]))
}

fn extension(vendor: &str, critical: bool) -> Value {
    Value::structure(structure([
        item(VENDOR_IDENTIFICATION, Value::text_string(vendor.to_owned())),
        item(CRITICALITY_INDICATOR, Value::boolean(critical)),
        item(VENDOR_EXTENSION, Value::structure(Structure::new())),
    ]))
}

fn request() -> RequestMessage {
    let header = structure([
        item(PROTOCOL_VERSION, version()),
        item(MAXIMUM_RESPONSE_SIZE, Value::integer(4096)),
        item(
            CLIENT_CORRELATION_VALUE,
            Value::text_string("client-correlation".to_owned()),
        ),
        item(
            SERVER_CORRELATION_VALUE,
            Value::text_string("server-correlation".to_owned()),
        ),
        item(ASYNCHRONOUS_INDICATOR, Value::enumeration(1)),
        item(ATTESTATION_CAPABLE_INDICATOR, Value::boolean(true)),
        item(ATTESTATION_TYPE, Value::enumeration(2)),
        item(ATTESTATION_TYPE, Value::enumeration(3)),
        item(AUTHENTICATION, Value::structure(Structure::new())),
        item(BATCH_ERROR_CONTINUATION_OPTION, Value::enumeration(1)),
        item(BATCH_ORDER_OPTION, Value::boolean(false)),
        item(TIME_STAMP, Value::date_time(1234)),
        item(BATCH_COUNT, Value::integer(2)),
    ]);
    let first_batch = structure([
        item(OPERATION, Value::enumeration(7)),
        item(EPHEMERAL, Value::boolean(true)),
        item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(vec![0xA1, 0xA2])),
        item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
        item(MESSAGE_EXTENSION, extension("Vendor_One", false)),
        item(MESSAGE_EXTENSION, extension("Vendor.Two", true)),
    ]);
    let second_batch = structure([
        item(OPERATION, Value::enumeration(8)),
        item(EPHEMERAL, Value::boolean(false)),
        item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(vec![0xB1])),
        item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
    ]);
    let tree = structure([
        item(REQUEST_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(first_batch)),
        item(BATCH_ITEM, Value::structure(second_batch)),
    ]);
    RequestMessage::try_from_ttlv(tree).expect("the request fixture is structurally valid")
}

fn response() -> ResponseMessage {
    let header = structure([
        item(PROTOCOL_VERSION, version()),
        item(TIME_STAMP, Value::date_time(5678)),
        item(NONCE, Value::structure(Structure::new())),
        item(SERVER_HASHED_PASSWORD, Value::byte_string(vec![0xC1, 0xC2])),
        item(ATTESTATION_TYPE, Value::enumeration(4)),
        item(ATTESTATION_TYPE, Value::enumeration(5)),
        item(
            CLIENT_CORRELATION_VALUE,
            Value::text_string("client-correlation".to_owned()),
        ),
        item(
            SERVER_CORRELATION_VALUE,
            Value::text_string("server-correlation".to_owned()),
        ),
        item(BATCH_COUNT, Value::integer(3)),
    ]);
    let failure = structure([
        item(OPERATION, Value::enumeration(7)),
        item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(vec![0xA1])),
        item(RESULT_STATUS, Value::enumeration(1)),
        item(RESULT_REASON, Value::enumeration(1)),
        item(
            RESULT_MESSAGE,
            Value::text_string("failure detail".to_owned()),
        ),
    ]);
    let success = structure([
        item(OPERATION, Value::enumeration(8)),
        item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(vec![0xB1])),
        item(RESULT_STATUS, Value::enumeration(0)),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        item(MESSAGE_EXTENSION, extension("ResponseVendor", true)),
    ]);
    let pending = structure([
        item(OPERATION, Value::enumeration(9)),
        item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(vec![0xC1])),
        item(RESULT_STATUS, Value::enumeration(2)),
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(vec![0xD1, 0xD2]),
        ),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);
    let tree = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(failure)),
        item(BATCH_ITEM, Value::structure(success)),
        item(BATCH_ITEM, Value::structure(pending)),
    ]);
    ResponseMessage::try_from_ttlv(tree).expect("the response fixture is structurally valid")
}

#[test]
fn request_header_accessors_expose_every_represented_field() {
    let message = request();
    let header = message.header();

    assert_eq!(header.maximum_response_size(), Some(4096));
    assert_eq!(
        header.with_client_correlation_value(str::to_owned),
        Some("client-correlation".to_owned())
    );
    assert_eq!(header.asynchronous_indicator(), Some(1));
    assert_eq!(header.effective_asynchronous_indicator(), 1);
    assert_eq!(header.attestation_capable_indicator(), Some(true));
    assert!(header.effective_attestation_capable_indicator());
    assert_eq!(header.attestation_types(), [2, 3]);
    assert_eq!(
        header.with_authentication(|authentication| authentication.children().len()),
        Some(0)
    );
    assert_eq!(header.batch_error_continuation_option(), Some(1));
    assert_eq!(header.effective_batch_error_continuation_option(), 1);
    assert_eq!(header.batch_order_option(), Some(false));
    assert!(!header.effective_batch_order_option());
    assert_eq!(header.time_stamp(), Some(1234));
}

#[test]
fn response_header_accessors_expose_every_represented_field() {
    let message = response();
    let header = message.header();

    assert_eq!(header.time_stamp(), 5678);
    assert_eq!(header.with_nonce(|nonce| nonce.children().len()), Some(0));
    assert_eq!(
        header.with_server_hashed_password(<[u8]>::to_vec),
        Some(vec![0xC1, 0xC2])
    );
    assert_eq!(header.attestation_types(), [4, 5]);
    assert_eq!(
        header.with_client_correlation_value(str::to_owned),
        Some("client-correlation".to_owned())
    );
    assert_eq!(
        header.with_server_correlation_value(str::to_owned),
        Some("server-correlation".to_owned())
    );
}

#[test]
fn request_batch_and_extension_accessors_preserve_order_and_values() {
    let message = request();
    let mut items = message.batch_items();
    let first = items.next().expect("first request item exists");
    let second = items.next().expect("second request item exists");

    assert_eq!(first.operation(), Some(7));
    assert_eq!(first.ephemeral(), Some(true));
    assert_eq!(
        first.with_unique_batch_item_id(<[u8]>::to_vec),
        Some(vec![0xA1, 0xA2])
    );
    assert_eq!(
        first.with_request_payload(|payload| payload.children().len()),
        Some(0)
    );
    assert_eq!(first.message_extension_count(), 2);
    assert!(first.message_extension(2).is_none());
    let extension = first.message_extension(1).expect("second extension exists");
    assert_eq!(
        extension.with_vendor_identification(str::to_owned),
        Some("Vendor.Two".to_owned())
    );
    assert_eq!(extension.criticality_indicator(), Some(true));
    assert_eq!(extension.with_ttlv(|value| value.children().len()), Some(3));
    assert_eq!(
        extension.with_vendor_extension(|value| value.children().len()),
        Some(0)
    );

    assert_eq!(second.operation(), Some(8));
    assert_eq!(second.ephemeral(), Some(false));
    assert_eq!(second.message_extension_count(), 0);
}

#[test]
fn response_batch_and_extension_accessors_preserve_all_result_shapes() {
    let message = response();
    let mut items = message.batch_items();
    let failure = items.next().expect("failure item exists");
    let success = items.next().expect("success item exists");
    let pending = items.next().expect("pending item exists");

    assert_eq!(failure.operation(), Some(7));
    assert_eq!(
        failure.with_unique_batch_item_id(<[u8]>::to_vec),
        Some(vec![0xA1])
    );
    assert_eq!(
        failure
            .result_status()
            .map(kmipkit_protocol::ResultStatus::raw),
        Some(1)
    );
    assert_eq!(
        failure
            .result_reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(1)
    );
    assert_eq!(
        failure.with_result_message(str::to_owned),
        Some("failure detail".to_owned())
    );
    assert_eq!(failure.with_response_payload(|_| ()), None);
    assert_eq!(
        failure.with_asynchronous_correlation_value(<[u8]>::to_vec),
        None
    );

    assert_eq!(success.operation(), Some(8));
    assert_eq!(
        success.with_response_payload(|payload| payload.children().len()),
        Some(0)
    );
    assert_eq!(success.message_extension_count(), 1);
    assert!(success.message_extension(1).is_none());
    let extension = success
        .message_extension(0)
        .expect("response extension exists");
    assert_eq!(
        extension.with_vendor_identification(str::to_owned),
        Some("ResponseVendor".to_owned())
    );
    assert_eq!(extension.criticality_indicator(), Some(true));
    assert_eq!(extension.with_ttlv(|value| value.children().len()), Some(3));
    assert_eq!(
        extension.with_vendor_extension(|value| value.children().len()),
        Some(0)
    );

    assert_eq!(pending.operation(), Some(9));
    assert_eq!(
        pending
            .result_status()
            .map(kmipkit_protocol::ResultStatus::raw),
        Some(2)
    );
    assert_eq!(pending.result_reason(), None);
    assert_eq!(pending.with_result_message(str::to_owned), None);
    assert_eq!(
        pending.with_asynchronous_correlation_value(<[u8]>::to_vec),
        Some(vec![0xD1, 0xD2])
    );
    assert_eq!(
        pending.with_response_payload(|payload| payload.children().len()),
        Some(0)
    );
}
