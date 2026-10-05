//! OASIS KMIP Specification v2.1: §§8.3, 8.6, 9.1–9.2, 9.9–9.10, 9.13,
//! 9.19, 9.21; Tables 396, 399, 400, 408–409, and 418.
//!
//! Traceability: KMIPKIT-0006-FR-004, FR-008, FR-010, FR-011, FR-014, FR-020,
//! FR-021; SC-002, SC-003, SC-004, SC-006.

use kmipkit_protocol::{MessageValidationErrorKind, RequestMessage, ResponseMessage, ResultStatus};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};
use quickcheck::{Arbitrary, Gen, QuickCheck};

const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_HEADER: u32 = 0x0042_0077;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_STATUS: u32 = 0x0042_007F;
const TIME_STAMP: u32 = 0x0042_0092;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const CLIENT_CORRELATION_VALUE: u32 = 0x0042_0105;
const CRITICALITY_INDICATOR: u32 = 0x0042_0026;
const MESSAGE_EXTENSION: u32 = 0x0042_0051;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const SERVER_CORRELATION_VALUE: u32 = 0x0042_0106;
const VENDOR_EXTENSION: u32 = 0x0042_009C;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const PROPERTY_SEED: u64 = 0x4B4D_4950_4B49_5430;

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

fn version() -> Value {
    Value::structure(structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]))
}

fn request(items: impl IntoIterator<Item = (Vec<u8>,)>) -> Structure {
    request_optional_ids(items.into_iter().map(|(id,)| Some(id)))
}

fn request_optional_ids(items: impl IntoIterator<Item = Option<Vec<u8>>>) -> Structure {
    let items: Vec<_> = items.into_iter().collect();
    let count = i32::try_from(items.len()).expect("test batch count fits i32");
    let header = structure([
        item(PROTOCOL_VERSION, version()),
        item(BATCH_COUNT, Value::integer(count)),
    ]);
    let mut message = vec![item(REQUEST_HEADER, Value::structure(header))];
    message.extend(items.into_iter().map(|id| {
        let mut fields = vec![item(OPERATION, Value::enumeration(1))];
        if let Some(id) = id {
            fields.push(item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id)));
        }
        fields.push(item(REQUEST_PAYLOAD, Value::structure(Structure::new())));
        item(BATCH_ITEM, Value::structure(structure(fields)))
    }));
    structure(message)
}

fn response(id: &[u8]) -> Structure {
    let header = structure([
        item(PROTOCOL_VERSION, version()),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(
            BATCH_ITEM,
            Value::structure(structure([
                item(OPERATION, Value::enumeration(1)),
                item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id.to_vec())),
                item(RESULT_STATUS, Value::enumeration(0)),
                item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
            ])),
        ),
    ])
}

fn message_extension(vendor: &str, criticality: bool) -> Structure {
    structure([
        item(VENDOR_IDENTIFICATION, Value::text_string(vendor.to_owned())),
        item(CRITICALITY_INDICATOR, Value::boolean(criticality)),
        item(VENDOR_EXTENSION, Value::structure(Structure::new())),
    ])
}

fn request_with_extensions(extensions: impl IntoIterator<Item = Structure>) -> Structure {
    let mut batch_fields = vec![item(OPERATION, Value::enumeration(1))];
    batch_fields.push(item(REQUEST_PAYLOAD, Value::structure(Structure::new())));
    batch_fields.extend(
        extensions
            .into_iter()
            .map(|extension| item(MESSAGE_EXTENSION, Value::structure(extension))),
    );
    request_tree_with_batch(structure(batch_fields), 1)
}

fn request_tree_with_batch(batch_item: Structure, batch_count: i32) -> Structure {
    structure([
        item(
            REQUEST_HEADER,
            Value::structure(structure([
                item(PROTOCOL_VERSION, version()),
                item(BATCH_COUNT, Value::integer(batch_count)),
            ])),
        ),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ])
}

#[test]
fn multi_item_request_rejects_repeated_unique_batch_item_ids() {
    let duplicate_id = vec![0x41, 0x00, 0x42];
    let tree = request([(duplicate_id.clone(),), (duplicate_id,)]);

    assert!(RequestMessage::try_from_ttlv(tree).is_err());
}

#[test]
fn response_batch_item_preserves_the_exact_echoed_id() {
    let id = [0x00, 0x91, 0x00, 0xF2];
    let response = ResponseMessage::try_from_ttlv(response(&id))
        .expect("a success response may echo its request item ID");

    assert_eq!(
        response
            .batch_items()
            .next()
            .expect("the response has one item")
            .with_unique_batch_item_id(<[u8]>::to_vec),
        Some(id.to_vec())
    );
}

#[test]
fn a_single_item_request_may_omit_its_unique_id() {
    let message = RequestMessage::try_from_ttlv(request_optional_ids([None]))
        .expect("Table 396 makes the ID optional for a single item");

    assert_eq!(
        message
            .batch_items()
            .next()
            .expect("the request has one item")
            .with_unique_batch_item_id(<[u8]>::to_vec),
        None
    );
}

#[test]
fn every_multi_item_request_requires_an_id() {
    let message = request_optional_ids([Some(vec![0x01]), None]);

    assert!(RequestMessage::try_from_ttlv(message).is_err());
}

#[derive(Clone, Debug)]
struct IdCase(Vec<Vec<u8>>);

impl Arbitrary for IdCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        let item_count = 1 + (usize::arbitrary(generator) % 8);
        let ids = (0..item_count)
            .map(|index| {
                let payload_length = usize::arbitrary(generator) % 32;
                let mut id = vec![u8::try_from(index).unwrap_or_default()];
                id.extend((0..payload_length).map(|_| u8::arbitrary(generator)));
                id
            })
            .collect();
        Self(ids)
    }
}

#[allow(clippy::needless_pass_by_value)]
fn ids_remain_attached_to_items_in_source_order(case: IdCase) -> bool {
    let expected = case.0.clone();
    let tree = request(case.0.into_iter().map(|id| (id,)));
    let Ok(message) = RequestMessage::try_from_ttlv(tree) else {
        return false;
    };
    let actual: Vec<_> = message
        .batch_items()
        .map(|batch_item| batch_item.with_unique_batch_item_id(<[u8]>::to_vec))
        .collect();
    actual == expected.into_iter().map(Some).collect::<Vec<_>>()
}

#[test]
fn request_item_ids_remain_associated_in_seeded_batch_order() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(64, PROPERTY_SEED))
        .tests(256)
        .quickcheck(ids_remain_attached_to_items_in_source_order as fn(IdCase) -> bool);
}

#[test]
fn repeated_request_extensions_preserve_order_and_expose_typed_fields() {
    let tree = request_with_extensions([
        message_extension("Vendor_A", false),
        message_extension("Vendor.B", true),
    ]);
    let message = RequestMessage::try_from_ttlv(tree)
        .expect("Table 396 permits repeated, well-formed Message Extensions");
    let batch_item = message.batch_items().next().expect("one item exists");

    assert_eq!(batch_item.message_extension_count(), 2);
    let first = batch_item
        .message_extension(0)
        .expect("first extension exists");
    let second = batch_item
        .message_extension(1)
        .expect("second extension exists");
    assert_eq!(
        first.with_vendor_identification(str::to_owned),
        Some("Vendor_A".to_owned())
    );
    assert_eq!(first.criticality_indicator(), Some(false));
    assert_eq!(
        second.with_vendor_identification(str::to_owned),
        Some("Vendor.B".to_owned())
    );
    assert_eq!(second.criticality_indicator(), Some(true));
    assert!(
        second
            .with_vendor_extension(|extension| extension.children().is_empty())
            .unwrap_or_default()
    );
    assert!(batch_item.message_extension(2).is_none());
}

#[test]
fn malformed_request_message_extension_is_rejected() {
    let missing_vendor_extension = structure([
        item(
            VENDOR_IDENTIFICATION,
            Value::text_string("Vendor".to_owned()),
        ),
        item(CRITICALITY_INDICATOR, Value::boolean(false)),
    ]);
    let wrong_order = structure([
        item(
            VENDOR_IDENTIFICATION,
            Value::text_string("Vendor".to_owned()),
        ),
        item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        item(CRITICALITY_INDICATOR, Value::boolean(false)),
    ]);
    let invalid_vendor_characters = message_extension("bad vendor", false);
    let empty_vendor_identification = message_extension("", false);
    let wrong_vendor_type = structure([
        item(VENDOR_IDENTIFICATION, Value::enumeration(1)),
        item(CRITICALITY_INDICATOR, Value::boolean(false)),
        item(VENDOR_EXTENSION, Value::structure(Structure::new())),
    ]);

    for extension in [
        missing_vendor_extension,
        wrong_order,
        invalid_vendor_characters,
        empty_vendor_identification,
        wrong_vendor_type,
    ] {
        let tree = request_with_extensions([extension]);
        assert!(RequestMessage::try_from_ttlv(tree).is_err());
    }
}

#[test]
fn response_message_extension_is_a_singleton() {
    let batch_item = structure([
        item(RESULT_STATUS, Value::enumeration(0)),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        item(
            MESSAGE_EXTENSION,
            Value::structure(message_extension("Vendor", false)),
        ),
        item(
            MESSAGE_EXTENSION,
            Value::structure(message_extension("OtherVendor", true)),
        ),
    ]);
    let response = structure([
        item(
            RESPONSE_HEADER,
            Value::structure(structure([
                item(PROTOCOL_VERSION, version()),
                item(TIME_STAMP, Value::date_time(1)),
                item(BATCH_COUNT, Value::integer(1)),
            ])),
        ),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ]);

    let error = ResponseMessage::try_from_ttlv(response)
        .expect_err("Table 399 permits at most one Message Extension");
    assert_eq!(error.kind(), MessageValidationErrorKind::DuplicateField);
}

#[test]
fn client_correlation_values_are_preserved_at_message_header_scope() {
    let request = request_tree_with_header_fields(item(
        CLIENT_CORRELATION_VALUE,
        Value::text_string("request-17".to_owned()),
    ));
    let request = RequestMessage::try_from_ttlv(request).expect("request header is valid");
    assert_eq!(
        request
            .header()
            .with_client_correlation_value(str::to_owned),
        Some("request-17".to_owned())
    );

    let response = response_with_correlations();
    let response = ResponseMessage::try_from_ttlv(response).expect("response header is valid");
    assert_eq!(
        response
            .header()
            .with_client_correlation_value(str::to_owned),
        Some("request-17".to_owned())
    );
    assert_eq!(
        response
            .header()
            .with_server_correlation_value(str::to_owned),
        Some("server-29".to_owned())
    );
}

#[test]
fn known_header_field_with_wrong_ttlv_type_reports_safe_category() {
    let tree = request_tree_with_header_fields(item(0x0042_0050, Value::boolean(true)));
    let error = RequestMessage::try_from_ttlv(tree)
        .expect_err("Maximum Response Size is an Integer in Table 395");

    assert_eq!(error.kind(), MessageValidationErrorKind::WrongItemType);
    assert!(!error.to_string().contains("true"));
}

fn request_tree_with_header_fields(extra: Item) -> Structure {
    structure([
        item(
            REQUEST_HEADER,
            Value::structure(structure([
                item(PROTOCOL_VERSION, version()),
                extra,
                item(BATCH_COUNT, Value::integer(1)),
            ])),
        ),
        item(
            BATCH_ITEM,
            Value::structure(structure([
                item(OPERATION, Value::enumeration(1)),
                item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
            ])),
        ),
    ])
}

fn response_with_correlations() -> Structure {
    structure([
        item(
            RESPONSE_HEADER,
            Value::structure(structure([
                item(PROTOCOL_VERSION, version()),
                item(TIME_STAMP, Value::date_time(1)),
                item(
                    CLIENT_CORRELATION_VALUE,
                    Value::text_string("request-17".to_owned()),
                ),
                item(
                    SERVER_CORRELATION_VALUE,
                    Value::text_string("server-29".to_owned()),
                ),
                item(BATCH_COUNT, Value::integer(1)),
            ])),
        ),
        item(
            BATCH_ITEM,
            Value::structure(structure([
                item(RESULT_STATUS, Value::enumeration(0)),
                item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
            ])),
        ),
    ])
}

fn response_tree_with_batch_items(
    batch_items: impl IntoIterator<Item = Structure>,
    batch_count: i32,
) -> Structure {
    let header = structure([
        item(PROTOCOL_VERSION, version()),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(batch_count)),
    ]);
    let mut message = vec![item(RESPONSE_HEADER, Value::structure(header))];
    message.extend(
        batch_items
            .into_iter()
            .map(|batch_item| item(BATCH_ITEM, Value::structure(batch_item))),
    );
    structure(message)
}

#[test]
fn unknown_result_status_round_trips_without_pending_only_fields() {
    let unknown_status = 0xF123_4567;
    let batch_item = structure([
        item(RESULT_STATUS, Value::enumeration(unknown_status)),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);
    let response = ResponseMessage::try_from_ttlv(response_tree_with_batch_items([batch_item], 1))
        .expect("an unknown raw Result Status is not treated as Pending");
    let status = response
        .batch_items()
        .next()
        .expect("one response item exists")
        .result_status()
        .expect("Result Status is required");

    assert_eq!(status.raw(), unknown_status);
    let round_trip = response.into_ttlv();
    assert_eq!(
        round_trip.view().children()[1].with_value(|value| match value {
            kmipkit_ttlv::ValueView::Structure(batch) =>
                batch.children()[0].with_value(|value| match value {
                    kmipkit_ttlv::ValueView::Enumeration(raw) => Some(*raw),
                    _ => None,
                }),
            _ => None,
        }),
        Some(unknown_status)
    );
}

#[test]
fn pending_requires_correlation_and_forbids_result_message() {
    let missing_correlation = structure([
        item(RESULT_STATUS, Value::enumeration(2)),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);
    let invalid_result_message = structure([
        item(RESULT_STATUS, Value::enumeration(2)),
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(vec![0x00, 0xA5]),
        ),
        item(
            RESULT_MESSAGE,
            Value::text_string("pending-result-message".to_owned()),
        ),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);

    assert!(
        ResponseMessage::try_from_ttlv(response_tree_with_batch_items([missing_correlation], 1,))
            .is_err()
    );
    assert!(
        ResponseMessage::try_from_ttlv(
            response_tree_with_batch_items([invalid_result_message], 1,)
        )
        .is_err()
    );
}

#[test]
fn one_response_batch_can_mix_completed_and_pending_results() {
    let completed = structure([
        item(RESULT_STATUS, Value::enumeration(0)),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);
    let pending = structure([
        item(RESULT_STATUS, Value::enumeration(2)),
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(vec![0x00, 0xA5]),
        ),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);
    let response =
        ResponseMessage::try_from_ttlv(response_tree_with_batch_items([completed, pending], 2))
            .expect("completed and Pending items may coexist in a response batch");
    let statuses: Vec<_> = response
        .batch_items()
        .filter_map(|batch_item| batch_item.result_status().map(ResultStatus::raw))
        .collect();

    assert_eq!(statuses, [0, 2]);
}
