//! OASIS KMIP Specification v2.1: §§8.1–8.6, 9.2–9.8, 9.10, 9.16, and 9.20;
//! Tables 394–399, 424–425, 432, and 435.
//!
//! These cases validate in-memory message structure and raw-value preservation.
//! They do not encode TTLV bytes, apply client execution policy, or claim
//! operation-schema validity.
//!
//! Traceability: KMIPKIT-0006-FR-001 through FR-007, FR-012, FR-018, FR-019;
//! SC-001 and SC-002.

use kmipkit_protocol::{RequestMessage, ResponseMessage};
use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Tag, Value, ValueView};

const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const ASYNCHRONOUS_INDICATOR: u32 = 0x0042_0007;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0002;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000E;
const BATCH_ORDER_OPTION: u32 = 0x0042_0010;
const REQUEST_HEADER: u32 = 0x0042_0077;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_STATUS: u32 = 0x0042_007F;
const TIME_STAMP: u32 = 0x0042_0092;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const OPERATION: u32 = 0x0042_005C;
const ATTESTATION_CAPABLE_INDICATOR: u32 = 0x0042_00D3;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("the fixture tag is allocated by KMIP 2.1")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("the checked tag and value form an item")
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

fn protocol_version(major: i32, minor: i32) -> Value {
    Value::structure(structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(major)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(minor)),
    ]))
}

fn request_header(
    batch_count: i32,
    time_stamp: Option<i64>,
    options: impl IntoIterator<Item = Item>,
) -> Structure {
    request_header_with_version(2, 1, batch_count, time_stamp, options)
}

fn request_header_with_version(
    major: i32,
    minor: i32,
    batch_count: i32,
    time_stamp: Option<i64>,
    options: impl IntoIterator<Item = Item>,
) -> Structure {
    let mut fields = vec![item(PROTOCOL_VERSION, protocol_version(major, minor))];
    fields.extend(options);
    if let Some(seconds) = time_stamp {
        fields.push(item(TIME_STAMP, Value::date_time(seconds)));
    }
    fields.push(item(BATCH_COUNT, Value::integer(batch_count)));
    structure(fields)
}

fn request_batch_item(id: Option<&[u8]>) -> Structure {
    let mut fields = vec![item(OPERATION, Value::enumeration(1))];
    if let Some(id) = id {
        fields.push(item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id.to_vec())));
    }
    fields.push(item(REQUEST_PAYLOAD, Value::structure(Structure::new())));
    structure(fields)
}

fn request_tree(header: Structure, batch_items: impl IntoIterator<Item = Structure>) -> Structure {
    let mut fields = vec![item(REQUEST_HEADER, Value::structure(header))];
    fields.extend(
        batch_items
            .into_iter()
            .map(|batch_item| item(BATCH_ITEM, Value::structure(batch_item))),
    );
    structure(fields)
}

fn valid_request(time_stamp: Option<i64>) -> Structure {
    request_tree(
        request_header(1, time_stamp, []),
        [request_batch_item(None)],
    )
}

fn response_header(batch_count: i32, time_stamp: Option<i64>) -> Structure {
    response_header_with_version(2, 1, batch_count, time_stamp)
}

fn response_header_with_version(
    major: i32,
    minor: i32,
    batch_count: i32,
    time_stamp: Option<i64>,
) -> Structure {
    let mut fields = vec![item(PROTOCOL_VERSION, protocol_version(major, minor))];
    if let Some(seconds) = time_stamp {
        fields.push(item(TIME_STAMP, Value::date_time(seconds)));
    }
    fields.push(item(BATCH_COUNT, Value::integer(batch_count)));
    structure(fields)
}

fn response_batch_item(status: u32, include_payload: bool) -> Structure {
    let mut fields = vec![item(RESULT_STATUS, Value::enumeration(status))];
    if status == 1 {
        fields.push(item(RESULT_REASON, Value::enumeration(0)));
    }
    if status == 2 {
        fields.push(item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(vec![0xA5]),
        ));
    }
    if include_payload {
        fields.push(item(RESPONSE_PAYLOAD, Value::structure(Structure::new())));
    }
    structure(fields)
}

fn response_tree(header: Structure, batch_items: impl IntoIterator<Item = Structure>) -> Structure {
    let mut fields = vec![item(RESPONSE_HEADER, Value::structure(header))];
    fields.extend(
        batch_items
            .into_iter()
            .map(|batch_item| item(BATCH_ITEM, Value::structure(batch_item))),
    );
    structure(fields)
}

fn valid_response(time_stamp: Option<i64>, status: u32, include_payload: bool) -> Structure {
    response_tree(
        response_header(1, time_stamp),
        [response_batch_item(status, include_payload)],
    )
}

fn raw_enumeration(view: StructureView<'_>, field_tag: u32) -> Option<u32> {
    view.children()
        .iter()
        .find(|child| child.tag().raw() == field_tag)
        .and_then(|child| {
            child.with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            })
        })
}

fn raw_date_time(view: StructureView<'_>, field_tag: u32) -> Option<i64> {
    view.children()
        .iter()
        .find(|child| child.tag().raw() == field_tag)
        .and_then(|child| {
            child.with_value(|value| match value {
                ValueView::DateTime(seconds) => Some(*seconds),
                _ => None,
            })
        })
}

#[test]
fn valid_request_and_response_preserve_envelope_order_and_counts() {
    let request = RequestMessage::try_from_ttlv(valid_request(None))
        .expect("the request has a correctly ordered header and one batch item");
    let response = ResponseMessage::try_from_ttlv(valid_response(Some(1_751_100_123), 0, true))
        .expect("the response has a correctly ordered header and one batch item");

    assert_eq!(request.header().protocol_version().major(), 2);
    assert_eq!(request.header().protocol_version().minor(), 1);
    assert_eq!(request.header().batch_count(), 1);
    assert_eq!(response.header().batch_count(), 1);
    assert_eq!(
        response.with_ttlv(|tree| tree
            .children()
            .iter()
            .map(|child| child.tag().raw())
            .collect::<Vec<_>>()),
        vec![RESPONSE_HEADER, BATCH_ITEM]
    );
}

#[test]
fn request_time_stamp_is_optional_and_preserved_as_raw_date_time() {
    let omitted = RequestMessage::try_from_ttlv(valid_request(None))
        .expect("Table 395 makes request Time Stamp optional");
    let raw_seconds = 1_751_100_123_i64;
    let present = RequestMessage::try_from_ttlv(valid_request(Some(raw_seconds)))
        .expect("Date-Time is accepted in Request Header field order");

    assert_eq!(omitted.header().time_stamp(), None);
    assert_eq!(present.header().time_stamp(), Some(raw_seconds));
    let round_trip = present.into_ttlv();
    let request_header = round_trip.view().children()[0];
    let actual = request_header.with_value(|value| match value {
        ValueView::Structure(header) => header
            .children()
            .iter()
            .find(|child| child.tag().raw() == TIME_STAMP)
            .and_then(|child| {
                child.with_value(|timestamp| match timestamp {
                    ValueView::DateTime(seconds) => Some(*seconds),
                    _ => None,
                })
            }),
        _ => None,
    });
    assert_eq!(actual, Some(raw_seconds));
}

#[test]
fn message_model_preserves_non_2_1_raw_version_values() {
    let request = RequestMessage::try_from_ttlv(request_tree(
        request_header_with_version(-3, i32::MAX, 1, None, []),
        [request_batch_item(None)],
    ))
    .expect("the in-memory message model does not enforce client version policy");
    let response = ResponseMessage::try_from_ttlv(response_tree(
        response_header_with_version(4, -7, 1, Some(1)),
        [response_batch_item(0, true)],
    ))
    .expect("raw version acceptance policy belongs to client execution");

    assert_eq!(request.header().protocol_version().major(), -3);
    assert_eq!(request.header().protocol_version().minor(), i32::MAX);
    assert_eq!(response.header().protocol_version().major(), 4);
    assert_eq!(response.header().protocol_version().minor(), -7);
}

#[test]
fn response_time_stamp_is_required_and_preserved_as_raw_date_time() {
    let missing = ResponseMessage::try_from_ttlv(valid_response(None, 0, true));
    let raw_seconds = 1_751_100_123_i64;
    let present = ResponseMessage::try_from_ttlv(valid_response(Some(raw_seconds), 0, true))
        .expect("Table 398 requires a Date-Time response timestamp");

    assert!(missing.is_err());
    assert_eq!(present.header().time_stamp(), raw_seconds);
    let round_trip = present.into_ttlv();
    let actual = round_trip.view().children()[0].with_value(|value| match value {
        ValueView::Structure(header) => raw_date_time(header, TIME_STAMP),
        _ => None,
    });
    assert_eq!(actual, Some(raw_seconds));
}

#[test]
fn request_batch_options_are_rejected_for_a_single_item() {
    let batch_order = request_tree(
        request_header(1, None, [item(BATCH_ORDER_OPTION, Value::boolean(true))]),
        [request_batch_item(None)],
    );
    let continuation = request_tree(
        request_header(
            1,
            None,
            [item(BATCH_ERROR_CONTINUATION_OPTION, Value::enumeration(0))],
        ),
        [request_batch_item(None)],
    );

    assert!(RequestMessage::try_from_ttlv(batch_order).is_err());
    assert!(RequestMessage::try_from_ttlv(continuation).is_err());
}

#[test]
fn absent_batch_options_keep_effective_defaults_without_materializing_fields() {
    let request = RequestMessage::try_from_ttlv(request_tree(
        request_header(2, None, []),
        [
            request_batch_item(Some(&[0x01])),
            request_batch_item(Some(&[0x02])),
        ],
    ))
    .expect("two identified items form a valid multi-item request");

    assert_eq!(request.header().batch_order_option(), None);
    assert!(request.header().effective_batch_order_option());
    assert_eq!(request.header().batch_error_continuation_option(), None);
    assert_eq!(
        request.header().effective_batch_error_continuation_option(),
        0
    );
    let header_children = request.with_ttlv(|tree| {
        tree.children()[0].with_value(|value| match value {
            ValueView::Structure(header) => header
                .children()
                .iter()
                .map(|child| child.tag().raw())
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        })
    });
    assert!(!header_children.contains(&BATCH_ORDER_OPTION));
    assert!(!header_children.contains(&BATCH_ERROR_CONTINUATION_OPTION));
}

#[test]
fn request_header_defaults_preserve_field_absence() {
    let request = RequestMessage::try_from_ttlv(valid_request(None))
        .expect("the request is valid when optional defaults are absent");

    assert_eq!(request.header().attestation_capable_indicator(), None);
    assert!(!request.header().effective_attestation_capable_indicator());
    assert_eq!(request.header().time_stamp(), None);
    assert!(
        request.into_ttlv().view().children()[0].with_value(|value| match value {
            ValueView::Structure(header) => !header
                .children()
                .iter()
                .any(|child| child.tag().raw() == ATTESTATION_CAPABLE_INDICATOR),
            _ => false,
        })
    );
}

#[test]
fn explicit_attestation_capable_indicator_values_and_presence_are_preserved() {
    for raw in [false, true] {
        let request = RequestMessage::try_from_ttlv(request_tree(
            request_header(
                1,
                None,
                [item(ATTESTATION_CAPABLE_INDICATOR, Value::boolean(raw))],
            ),
            [request_batch_item(None)],
        ))
        .expect("Table 395 represents Attestation Capable Indicator as Boolean");

        assert_eq!(request.header().attestation_capable_indicator(), Some(raw));
        assert_eq!(
            request.header().effective_attestation_capable_indicator(),
            raw
        );
    }
}

#[test]
fn signed_batch_count_must_be_nonnegative_and_match_item_count() {
    let negative = request_tree(
        request_header(i32::MIN, None, []),
        [request_batch_item(None)],
    );
    let mismatch = request_tree(
        request_header(i32::MAX, None, []),
        [request_batch_item(None)],
    );

    assert!(RequestMessage::try_from_ttlv(negative).is_err());
    assert!(RequestMessage::try_from_ttlv(mismatch).is_err());

    let response_negative = response_tree(
        response_header(i32::MIN, Some(1)),
        [response_batch_item(0, true)],
    );
    let response_mismatch = response_tree(
        response_header(i32::MAX, Some(1)),
        [response_batch_item(0, true)],
    );
    assert!(ResponseMessage::try_from_ttlv(response_negative).is_err());
    assert!(ResponseMessage::try_from_ttlv(response_mismatch).is_err());
    assert!(RequestMessage::try_from_ttlv(request_tree(request_header(0, None, []), [],)).is_err());
    assert!(
        ResponseMessage::try_from_ttlv(response_tree(response_header(0, Some(1)), [],)).is_err()
    );
}

#[test]
fn request_known_fields_and_envelope_reject_reordering() {
    let reversed_header = structure([
        item(BATCH_COUNT, Value::integer(1)),
        item(PROTOCOL_VERSION, protocol_version(2, 1)),
    ]);
    let reversed_item = structure([
        item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
        item(OPERATION, Value::enumeration(1)),
    ]);
    let item_first = structure([
        item(BATCH_ITEM, Value::structure(request_batch_item(None))),
        item(
            REQUEST_HEADER,
            Value::structure(request_header(1, None, [])),
        ),
    ]);

    assert!(
        RequestMessage::try_from_ttlv(request_tree(reversed_header, [request_batch_item(None)],))
            .is_err()
    );
    assert!(
        RequestMessage::try_from_ttlv(request_tree(request_header(1, None, []), [reversed_item],))
            .is_err()
    );
    assert!(RequestMessage::try_from_ttlv(item_first).is_err());
}

#[test]
fn response_known_fields_and_envelope_reject_reordering() {
    let reversed_header = structure([
        item(BATCH_COUNT, Value::integer(1)),
        item(PROTOCOL_VERSION, protocol_version(2, 1)),
        item(TIME_STAMP, Value::date_time(1)),
    ]);
    let reversed_item = structure([
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        item(RESULT_STATUS, Value::enumeration(0)),
    ]);
    let item_first = structure([
        item(BATCH_ITEM, Value::structure(response_batch_item(0, true))),
        item(
            RESPONSE_HEADER,
            Value::structure(response_header(1, Some(1))),
        ),
    ]);

    assert!(
        ResponseMessage::try_from_ttlv(response_tree(
            reversed_header,
            [response_batch_item(0, true)],
        ))
        .is_err()
    );
    assert!(
        ResponseMessage::try_from_ttlv(
            response_tree(response_header(1, Some(1)), [reversed_item],)
        )
        .is_err()
    );
    assert!(ResponseMessage::try_from_ttlv(item_first).is_err());
}

#[test]
fn response_payload_is_required_for_non_failure_and_absent_for_failure() {
    let missing_success_payload = valid_response(Some(1), 0, false);
    let failure_with_payload = valid_response(Some(1), 1, true);
    let valid_failure = valid_response(Some(1), 1, false);
    let missing_pending_payload = valid_response(Some(1), 2, false);
    let valid_pending = valid_response(Some(1), 2, true);

    assert!(ResponseMessage::try_from_ttlv(missing_success_payload).is_err());
    assert!(ResponseMessage::try_from_ttlv(failure_with_payload).is_err());
    assert!(ResponseMessage::try_from_ttlv(valid_failure).is_ok());
    assert!(ResponseMessage::try_from_ttlv(missing_pending_payload).is_err());
    assert!(ResponseMessage::try_from_ttlv(valid_pending).is_ok());
}

#[test]
fn raw_request_option_enumerations_survive_parse_and_conversion() {
    let values = [0_u32, 0x8000_0000, u32::MAX];
    for raw in values {
        let request = RequestMessage::try_from_ttlv(request_tree(
            request_header(
                1,
                None,
                [item(ASYNCHRONOUS_INDICATOR, Value::enumeration(raw))],
            ),
            [request_batch_item(None)],
        ))
        .expect("parsing preserves raw Enumeration values without send policy");
        let round_trip = request.into_ttlv();
        let raw_value = round_trip.view().children()[0].with_value(|value| match value {
            ValueView::Structure(header) => raw_enumeration(header, ASYNCHRONOUS_INDICATOR),
            _ => None,
        });
        assert_eq!(raw_value, Some(raw));
    }

    for raw in values {
        let request = RequestMessage::try_from_ttlv(request_tree(
            request_header(
                2,
                None,
                [item(
                    BATCH_ERROR_CONTINUATION_OPTION,
                    Value::enumeration(raw),
                )],
            ),
            [
                request_batch_item(Some(&[0x01])),
                request_batch_item(Some(&[0x02])),
            ],
        ))
        .expect("parsing preserves raw batch option values without send policy");
        let round_trip = request.into_ttlv();
        let raw_value = round_trip.view().children()[0].with_value(|value| match value {
            ValueView::Structure(header) => {
                raw_enumeration(header, BATCH_ERROR_CONTINUATION_OPTION)
            }
            _ => None,
        });
        assert_eq!(raw_value, Some(raw));
    }
}
