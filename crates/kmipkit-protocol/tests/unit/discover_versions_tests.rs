//! Derived operation-schema tests for OASIS KMIP Specification v2.1 §6.1.16,
//! Tables 211–213, and §9.16, Table 421. The source records are
//! `KMIPKIT-REQ-SPEC-6.1.16-001-001`, `KMIPKIT-REQ-SPEC-6.1.16-001-002`,
//! `KMIPKIT-REQ-SPEC-6.1.16-004`, and
//! `KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS`,
//! `KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION-MAJOR/-MINOR`; the client
//! contract is KMIPKIT-0007 FR-001/002/003. Table 212 permits repeated Protocol
//! Version fields and states no uniqueness rule. These are derived tests, not
//! official OASIS vectors.

use crate::{
    DiscoverVersionsError, DiscoverVersionsRequest, DiscoverVersionsResponse, KmipOperationResult,
    ProtocolVersion, RequestMessage, ResponseMessage, ResultMessage, ResultReason, ResultStatus,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const REQUEST_HEADER: u32 = 0x0042_0077;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const EXTENSION_PAYLOAD_TAG: u32 = 0x0042_0173;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
const KMIP_2_1: ProtocolVersion = ProtocolVersion::from_raw(2, 1);

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits the 24-bit KMIP field")
        .try_checked()
        .expect("fixture tag is allocated by the KMIP 2.1 catalog")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("fixture item uses a checked tag")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("fixture structure remains within the model depth limit");
    }
    structure
}

fn protocol_version(major: i32, minor: i32) -> Item {
    item(
        PROTOCOL_VERSION,
        Value::structure(structure([
            item(PROTOCOL_VERSION_MAJOR, Value::integer(major)),
            item(PROTOCOL_VERSION_MINOR, Value::integer(minor)),
        ])),
    )
}

fn response(
    status: u32,
    reason: Option<u32>,
    result_message: Option<&str>,
    payload: Option<Structure>,
) -> ResponseMessage {
    response_for_operation(
        DISCOVER_VERSIONS_OPERATION,
        status,
        reason,
        result_message,
        payload,
    )
}

fn response_for_operation(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    result_message: Option<&str>,
    payload: Option<Structure>,
) -> ResponseMessage {
    let header = structure([
        protocol_version(2, 1),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut response_item = vec![
        item(OPERATION, Value::enumeration(operation)),
        item(RESULT_STATUS, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        response_item.push(item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(result_message) = result_message {
        response_item.push(item(
            RESULT_MESSAGE,
            Value::text_string(result_message.to_owned()),
        ));
    }
    if let Some(payload) = payload {
        response_item.push(item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    let message = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(structure(response_item))),
    ]);
    ResponseMessage::try_from_ttlv(message).expect("fixture is a valid 0006 response message")
}

fn decode_response(
    message: &ResponseMessage,
) -> Result<DiscoverVersionsResponse, DiscoverVersionsError> {
    let item = message
        .batch_items()
        .next()
        .expect("fixture has one response batch item");
    DiscoverVersionsResponse::try_from_response_item(item)
}

#[test]
fn request_advertises_only_kmip_2_1() {
    let request = DiscoverVersionsRequest::new();
    assert_eq!(request.protocol_versions(), [KMIP_2_1]);

    let payload = request
        .to_ttlv_payload()
        .expect("the fixed request payload uses allocated tags");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), PROTOCOL_VERSION);
    let components = fields[0].with_value(|value| match value {
        ValueView::Structure(version) => Some(
            version
                .children()
                .iter()
                .map(|field| {
                    (
                        field.tag().raw(),
                        field.with_value(|value| match value {
                            ValueView::Integer(value) => Some(*value),
                            _ => None,
                        }),
                    )
                })
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(
        components,
        Some(vec![
            (PROTOCOL_VERSION_MAJOR, Some(2)),
            (PROTOCOL_VERSION_MINOR, Some(1)),
        ])
    );
}

#[test]
fn empty_success_response_is_accepted() {
    let message = response(0, None, None, Some(Structure::new()));
    let actual = decode_response(&message).expect("an empty version list is valid");

    assert_eq!(actual.result().status(), ResultStatus::from_raw(0));
    assert_eq!(actual.supported_versions(), Some(&[][..]));
}

#[test]
fn offered_response_versions_are_preserved_in_order_with_repetitions() {
    let payload = structure([protocol_version(2, 1), protocol_version(2, 1)]);
    let message = response(0, None, None, Some(payload));
    let actual = decode_response(&message).expect("repeated offered versions are permitted");

    assert_eq!(actual.supported_versions(), Some(&[KMIP_2_1, KMIP_2_1][..]));
}

#[test]
fn response_rejects_a_protocol_version_that_was_not_offered() {
    let message = response(0, None, None, Some(structure([protocol_version(3, 0)])));

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::UnofferedProtocolVersion
    );
}

fn malformed_version(fields: impl IntoIterator<Item = Item>) -> ResponseMessage {
    response(
        0,
        None,
        None,
        Some(structure([item(
            PROTOCOL_VERSION,
            Value::structure(structure(fields)),
        )])),
    )
}

#[test]
fn response_rejects_a_protocol_version_missing_its_major_component() {
    let message = malformed_version([item(PROTOCOL_VERSION_MINOR, Value::integer(1))]);

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::MalformedProtocolVersion
    );
}

#[test]
fn response_rejects_a_protocol_version_missing_its_minor_component() {
    let message = malformed_version([item(PROTOCOL_VERSION_MAJOR, Value::integer(2))]);

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::MalformedProtocolVersion
    );
}

#[test]
fn response_rejects_protocol_version_components_out_of_table_order() {
    let message = malformed_version([
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
    ]);

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::MalformedProtocolVersion
    );
}

#[test]
fn response_rejects_a_protocol_version_component_with_the_wrong_type() {
    let message = malformed_version([
        item(PROTOCOL_VERSION_MAJOR, Value::text_string("2".to_owned())),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::MalformedProtocolVersion
    );
}

#[test]
fn response_rejects_a_minor_component_with_the_wrong_type() {
    let message = malformed_version([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::text_string("1".to_owned())),
    ]);

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::MalformedProtocolVersion
    );
}

#[test]
fn response_rejects_a_different_operation() {
    let message = response_for_operation(
        0x0000_001F,
        0,
        None,
        None,
        Some(structure([protocol_version(2, 1)])),
    );

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::UnexpectedOperation
    );
}

#[test]
fn response_rejects_repeated_protocol_version_components() {
    let message = malformed_version([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::MalformedProtocolVersion
    );
}

#[test]
fn response_rejects_a_repeated_minor_component() {
    let message = malformed_version([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::MalformedProtocolVersion
    );
}

#[test]
fn response_rejects_unrecognized_protocol_version_components() {
    let message = malformed_version([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(EXTENSION_PAYLOAD_TAG, Value::integer(7)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::MalformedProtocolVersion
    );
}

#[test]
fn response_rejects_protocol_version_field_with_a_non_structure_value() {
    let payload = structure([item(PROTOCOL_VERSION, Value::text_string("2.1".to_owned()))]);
    let message = response(0, None, None, Some(payload));

    assert_eq!(
        decode_response(&message).unwrap_err(),
        DiscoverVersionsError::MalformedProtocolVersion
    );
}

#[test]
fn operation_errors_are_preserved_without_becoming_success_payloads() {
    let message = response(1, Some(5), Some("operation unsupported"), None);
    let expected = KmipOperationResult::new(
        ResultStatus::from_raw(1),
        Some(ResultReason::from_raw(5)),
        Some(ResultMessage::new("operation unsupported".to_owned())),
    )
    .expect("Table 213 Operation Failed / Operation Not Supported is valid");

    let actual = decode_response(&message).expect("operation failures are returned as results");
    assert_eq!(actual.result(), &expected);
    assert_eq!(actual.supported_versions(), None);
}

#[test]
fn conversion_leaves_the_0006_owned_generic_payload_available() {
    let payload = structure([
        protocol_version(2, 1),
        item(0x0042_0173, Value::byte_string(vec![0xA5, 0x5A])),
    ]);
    let message = response(0, None, None, Some(payload));

    let typed = decode_response(&message).expect("the offered version is valid");
    assert_eq!(typed.supported_versions(), Some(&[KMIP_2_1][..]));
    let opaque_bytes = message
        .batch_items()
        .next()
        .and_then(|item| {
            item.with_response_payload(|payload| {
                payload.children().get(1).and_then(|opaque| {
                    opaque.with_value(|value| match value {
                        ValueView::ByteString(bytes) => Some(bytes.to_vec()),
                        _ => None,
                    })
                })
            })
        })
        .flatten();
    assert_eq!(opaque_bytes, Some(vec![0xA5, 0x5A]));
}

#[test]
fn request_message_model_accepts_the_typed_payload_tree() {
    let payload = DiscoverVersionsRequest::new()
        .to_ttlv_payload()
        .expect("the fixed request payload uses allocated tags");
    let request_item = structure([
        item(OPERATION, Value::enumeration(DISCOVER_VERSIONS_OPERATION)),
        item(REQUEST_PAYLOAD, Value::structure(payload)),
    ]);
    let header = structure([protocol_version(2, 1), item(BATCH_COUNT, Value::integer(1))]);
    let message = structure([
        item(REQUEST_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(request_item)),
    ]);

    assert!(RequestMessage::try_from_ttlv(message).is_ok());
}
