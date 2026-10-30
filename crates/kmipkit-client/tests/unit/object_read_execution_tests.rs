//! Fake-transport Get and Locate execution contracts derived from OASIS KMIP
//! v2.1 §§6.1.19 and 6.1.28, Tables 220–222 and 247–249; §§8.6 and 9.1,
//! Tables 399–400; §§9.2 and 11.3, Tables 431–432; §§11.46–11.47,
//! Tables 479–480; and ID Placeholder behavior in §§6.1, 6.1.8/Table 187,
//! and 9.8; §12.3, Table 498. Traceability: KMIPKIT-0017-FR-001, FR-003,
//! FR-006–FR-011, and SC-002. FR-009 is verified here by preserving the Locate
//! identifiers returned by the server without client-side filtering. These
//! are client execution contracts, not claims of official OASIS case passes.

use kmipkit_protocol::{
    AttributeSet, CreateRequest, GetRequest, LocateRequest, ObjectGroupMember, ObjectType,
    RecoverRequest, StorageStatusMask, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Structure, Value};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;
use zeroize::Zeroize;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use crate::execute_test_support::{
    asynchronous_response_bytes, one_item_response_bytes, operation_batch_response_bytes,
    test_item, test_structure,
};

const CREATE_OPERATION: u32 = 0x0000_0001;
const GET_OPERATION: u32 = 0x0000_000A;
const LOCATE_OPERATION: u32 = 0x0000_0008;
const RECOVER_OPERATION: u32 = 0x0000_0016;
const RESPONSE_MESSAGE: &str = "GET_RESULT_MESSAGE_SENTINEL";
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const PENDING_CORRELATION: &[u8] = b"GET_PENDING_CORRELATION_EXACT";

const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ACTIVATION_DATE: u32 = 0x0042_0001;
const MAXIMUM_ITEMS: u32 = 0x0042_004F;
const OFFSET_ITEMS: u32 = 0x0042_00D4;
const STORAGE_STATUS_MASK: u32 = 0x0042_008E;
const OBJECT_GROUP_MEMBER: u32 = 0x0042_00AC;
const ATTRIBUTES: u32 = 0x0042_0125;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;

const CREATE_BATCH_ID: &[u8] = b"get-placeholder-create";
const GET_BATCH_ID: &[u8] = b"get-placeholder-read";
// Unique Batch Item IDs identify message frames; they are not KMIP Unique Identifiers.
const LOCATE_BATCH_ID: &[u8] = b"locate-search-frame";
const LOCATE_GET_BATCH_ID: &[u8] = b"locate-get-frame";
const ARCHIVED_LOCATE_BATCH_ID: &[u8] = b"archived-locate-frame";
const ARCHIVED_RECOVER_BATCH_ID: &[u8] = b"archived-recover-frame";
const ARCHIVED_GET_BATCH_ID: &[u8] = b"archived-get-frame";

fn get_with_explicit_identifier() -> GetRequest {
    GetRequest::new().with_unique_identifier(UniqueIdentifier::TextString(
        "client-requested-object-id".to_owned(),
    ))
}

#[derive(Debug, Eq, PartialEq)]
enum WireValue {
    Structure(Vec<(u32, WireValue)>),
    Integer(i32),
    Enumeration(u32),
    TextString(String),
    ByteString(Vec<u8>),
    DateTime(i64),
}

#[derive(Debug, Eq, PartialEq)]
struct RequestItemSnapshot {
    operation: u32,
    unique_batch_item_id: Vec<u8>,
    payload: WireValue,
}

fn request_item_snapshot(request: &[u8], index: usize) -> Option<RequestItemSnapshot> {
    let document = decode_with_limits(request, &CodecLimits::defaults()).ok()?;
    document.with_value(|message| {
        let ValueView::Structure(message) = message else {
            return None;
        };
        let batch_item = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .nth(index)?;
        batch_item.with_value(|batch_item| {
            let ValueView::Structure(batch_item) = batch_item else {
                return None;
            };
            let operation = batch_item
                .children()
                .iter()
                .find(|item| item.tag().raw() == OPERATION)?
                .with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                })?;
            let unique_batch_item_id = batch_item
                .children()
                .iter()
                .find(|item| item.tag().raw() == UNIQUE_BATCH_ITEM_ID)?
                .with_value(|value| match value {
                    ValueView::ByteString(value) => Some(value.to_vec()),
                    _ => None,
                })?;
            let payload = batch_item
                .children()
                .iter()
                .find(|item| item.tag().raw() == REQUEST_PAYLOAD)?
                .with_value(wire_value_snapshot);
            Some(RequestItemSnapshot {
                operation,
                unique_batch_item_id,
                payload,
            })
        })
    })
}

struct CallerDirectedArchivedTransport {
    responses: VecDeque<Vec<u8>>,
    requests: Rc<RefCell<Vec<RequestItemSnapshot>>>,
}

impl Transport for CallerDirectedArchivedTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        let snapshot = request_item_snapshot(request, 0).ok_or_else(|| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("the archived retrieval request cannot be decoded"),
            )
        })?;
        self.requests.borrow_mut().push(snapshot);
        self.responses
            .pop_front()
            .map(TransportResponse::new)
            .ok_or_else(|| {
                TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    io::Error::other("the archived retrieval sequence has no scripted response"),
                )
            })
    }
}

impl Drop for CallerDirectedArchivedTransport {
    fn drop(&mut self) {
        for response in &mut self.responses {
            response.zeroize();
        }
    }
}

fn wire_value_snapshot(value: ValueView<'_>) -> WireValue {
    match value {
        ValueView::Structure(structure) => WireValue::Structure(
            structure
                .children()
                .iter()
                .map(|item| (item.tag().raw(), item.with_value(wire_value_snapshot)))
                .collect(),
        ),
        ValueView::Integer(value) => WireValue::Integer(*value),
        ValueView::Enumeration(value) => WireValue::Enumeration(*value),
        ValueView::TextString(value) => WireValue::TextString(value.to_owned()),
        ValueView::ByteString(value) => WireValue::ByteString(value.to_vec()),
        ValueView::DateTime(value) => WireValue::DateTime(*value),
        _ => panic!("the captured object-read request uses only asserted TTLV values"),
    }
}

fn wire_field(payload: &WireValue, tag: u32) -> Option<&WireValue> {
    let WireValue::Structure(fields) = payload else {
        return None;
    };
    fields
        .iter()
        .find_map(|(field_tag, value)| (*field_tag == tag).then_some(value))
}

fn empty_create_request() -> CreateRequest {
    CreateRequest::new(ObjectType::from_raw(7), AttributeSet::new())
}

fn successful_get_payload(identifier: &str) -> Structure {
    test_structure([
        test_item(OBJECT_TYPE, Value::enumeration(2)),
        test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
        test_item(
            SYMMETRIC_KEY,
            Value::structure(test_structure([test_item(
                KEY_BLOCK,
                Value::structure(test_structure([
                    test_item(KEY_FORMAT_TYPE, Value::enumeration(1)),
                    test_item(
                        KEY_VALUE,
                        Value::structure(test_structure([test_item(
                            KEY_MATERIAL,
                            Value::byte_string(b"opaque-get-response".to_vec()),
                        )])),
                    ),
                ])),
            )])),
        ),
    ])
}

fn successful_create_payload(identifier: &str) -> Structure {
    test_structure([
        test_item(OBJECT_TYPE, Value::enumeration(7)),
        test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
    ])
}

fn successful_locate_payload(identifiers: &[&str]) -> Structure {
    test_structure(identifiers.iter().map(|identifier| {
        test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string((*identifier).to_owned()),
        )
    }))
}

fn request_item_shape(request: &[u8], index: usize) -> Option<(u32, bool)> {
    let document = decode_with_limits(request, &CodecLimits::defaults()).ok()?;
    document.with_value(|message| {
        let ValueView::Structure(message) = message else {
            return None;
        };
        let batch_item = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .nth(index)?;
        batch_item.with_value(|batch_item| {
            let ValueView::Structure(batch_item) = batch_item else {
                return None;
            };
            let operation = batch_item
                .children()
                .iter()
                .find(|item| item.tag().raw() == OPERATION)?
                .with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                })?;
            let payload_has_identifier = batch_item
                .children()
                .iter()
                .find(|item| item.tag().raw() == REQUEST_PAYLOAD)?
                .with_value(|payload| match payload {
                    ValueView::Structure(payload) => payload
                        .children()
                        .iter()
                        .any(|item| item.tag().raw() == UNIQUE_IDENTIFIER),
                    _ => false,
                });
            Some((operation, payload_has_identifier))
        })
    })
}

#[test]
fn get_success_preserves_completed_object_data_in_one_exchange() {
    let response = one_item_response_bytes(
        GET_OPERATION,
        0,
        None,
        None,
        None,
        None,
        Some(successful_get_payload("server-assigned-get-id")),
    );
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![3, 7],
    });
    let request = ClientRequest::get(get_with_explicit_identifier());
    assert_eq!(format!("{request:?}"), "Get([REDACTED])");

    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request)),
            &CodecLimits::defaults(),
        )
        .expect("the server returns a completed Get response");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(
        request_item_shape(
            captured_request
                .borrow()
                .as_deref()
                .expect("the request reached the fake transport"),
            0,
        ),
        Some((GET_OPERATION, true)),
        "a standalone Get identifies its object explicitly"
    );
    let outcome = response.get(0).expect("one result is returned").outcome();
    assert_eq!(outcome.operation(), ClientOperation::Get);
    assert_eq!(outcome.result().status().raw(), 0);
    let get_response = outcome
        .get_response()
        .expect("a completed Get exposes its typed response");
    assert_eq!(get_response.result().status().raw(), 0);
    assert_eq!(get_response.object_type(), Some(ObjectType::from_raw(2)));
    assert_eq!(
        get_response.unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "server-assigned-get-id".to_owned()
        ))
    );
    assert_eq!(
        get_response.object().map(|object| object.tag().raw()),
        Some(SYMMETRIC_KEY)
    );
    assert_eq!(outcome.response().result().status().raw(), 0);

    let response_debug = format!("{:?}", outcome.response());
    let outcome_debug = format!("{outcome:?}");
    let outcome_display = outcome.to_string();
    assert!(response_debug.contains("Get"));
    assert!(outcome_debug.contains("GetCompleted"));
    assert!(outcome_display.contains("GetCompleted"));
    assert!(outcome_display.contains("status 0"));
    assert!(!response_debug.contains("opaque-get-response"));
    assert!(!outcome_debug.contains("opaque-get-response"));
}

#[test]
fn get_failure_preserves_result_status_reason_and_message_without_retry() {
    let response = one_item_response_bytes(
        GET_OPERATION,
        1,
        Some(OBJECT_NOT_FOUND),
        Some(RESPONSE_MESSAGE),
        None,
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::Get(
                get_with_explicit_identifier(),
            ))),
            &CodecLimits::defaults(),
        )
        .expect("the operation failure remains a typed Get result");
    let outcome = response.get(0).expect("one result is returned").outcome();
    let get_response = outcome
        .get_response()
        .expect("a completed failed Get retains its operation result");

    assert_eq!(outcome.operation(), ClientOperation::Get);
    assert_eq!(get_response.result().status().raw(), 1);
    assert_eq!(
        get_response
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(OBJECT_NOT_FOUND)
    );
    assert_eq!(
        get_response
            .result()
            .message()
            .map(kmipkit_protocol::ResultMessage::as_str),
        Some(RESPONSE_MESSAGE)
    );
    assert!(get_response.object().is_none());
    let response_debug = format!("{:?}", outcome.response());
    let outcome_debug = format!("{outcome:?}");
    let outcome_display = outcome.to_string();
    assert!(!response_debug.contains(RESPONSE_MESSAGE));
    assert!(!outcome_debug.contains(RESPONSE_MESSAGE));
    assert!(!outcome_display.contains(RESPONSE_MESSAGE));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn get_pending_preserves_correlation_and_typed_result_without_follow_up() {
    let response = asynchronous_response_bytes(
        GET_OPERATION,
        2,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Get(
        get_with_explicit_identifier(),
    )))
    .with_asynchronous_indicator(2);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the requested asynchronous mode permits Pending");
    let outcome = response.get(0).expect("one result is returned").outcome();
    let ClientBatchOutcome::Pending(pending) = outcome else {
        panic!("a valid Get Pending result remains an explicit Pending outcome");
    };

    assert_eq!(pending.operation(), ClientOperation::Get);
    assert_eq!(pending.result().status().raw(), 2);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert!(outcome.get_response().is_none());
    assert_eq!(
        outcome
            .response()
            .get()
            .map(|get| get.result().status().raw()),
        Some(2),
        "the unified response view preserves the Get Pending result"
    );
    assert_eq!(
        fake.borrow().exchange_count(),
        1,
        "Pending is not polled again"
    );
}

#[test]
fn get_transport_failures_preserve_delivery_state_without_retry() {
    for (script, expected_state) in [
        (
            ExchangeScript::FailBeforeWrite,
            RequestDeliveryState::NotSent,
        ),
        (
            ExchangeScript::FailAfterPartialWrite { written_bytes: 3 },
            RequestDeliveryState::PossiblySent,
        ),
        (
            ExchangeScript::FailAfterPartialRead {
                written_bytes: 3,
                response_bytes: vec![0x42, 0x00, 0x7A, 0x01],
            },
            RequestDeliveryState::ResponseStarted,
        ),
    ] {
        let (mut client, fake, _) = client_for(script);
        let error = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(ClientRequest::Get(
                    get_with_explicit_identifier(),
                ))),
                &CodecLimits::defaults(),
            )
            .expect_err("the scripted transport failure is returned to the caller");

        assert_eq!(error.delivery_state(), Some(expected_state));
        assert_eq!(
            fake.borrow().exchange_count(),
            1,
            "delivery {expected_state:?}"
        );
    }
}

#[test]
fn get_without_identifier_follows_create_and_keeps_the_batch_order() {
    let response = operation_batch_response_bytes([
        (
            CREATE_OPERATION,
            CREATE_BATCH_ID.to_vec(),
            successful_create_payload("server-created-id"),
        ),
        (
            GET_OPERATION,
            GET_BATCH_ID.to_vec(),
            successful_get_payload("server-created-id"),
        ),
    ]);
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 11],
    });
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Create(empty_create_request()))
            .with_unique_batch_item_id(CREATE_BATCH_ID.to_vec()),
        ClientBatchItem::new(ClientRequest::Get(GetRequest::new()))
            .with_unique_batch_item_id(GET_BATCH_ID.to_vec()),
    ]);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("Create establishes server-side ID Placeholder for the following Get");

    let captured_request = captured_request.borrow();
    let request = captured_request
        .as_deref()
        .expect("the unchanged batch reaches the fake transport");
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(
        request_item_shape(request, 0),
        Some((CREATE_OPERATION, false))
    );
    assert_eq!(
        request_item_shape(request, 1),
        Some((GET_OPERATION, false)),
        "the Get payload omits Unique Identifier instead of synthesizing one"
    );
    let first = response.get(0).expect("Create remains batch item zero");
    let second = response.get(1).expect("Get remains batch item one");
    assert_eq!(first.unique_batch_item_id(), Some(CREATE_BATCH_ID));
    assert_eq!(second.unique_batch_item_id(), Some(GET_BATCH_ID));
    assert_eq!(first.outcome().operation(), ClientOperation::Create);
    assert_eq!(second.outcome().operation(), ClientOperation::Get);
    assert_eq!(
        second
            .outcome()
            .get_response()
            .and_then(|get| get.unique_identifier()),
        Some(&UniqueIdentifier::TextString(
            "server-created-id".to_owned()
        ))
    );
}

#[test]
fn locate_batch_preserves_criteria_results_and_server_placeholder_boundary() {
    // KMIP v2.1 §6.1.28 leaves ID Placeholder empty for multi-match Locate; a single result permits the later Get.
    let attributes = AttributeSet::try_new([
        test_item(ACTIVATION_DATE, Value::date_time(100)),
        test_item(ACTIVATION_DATE, Value::date_time(200)),
    ])
    .expect("the two Activation Date values form an ordered date-range criterion");
    let locate = LocateRequest::new(attributes)
        .with_maximum_items(1)
        .with_offset_items(0)
        .with_storage_status_mask(StorageStatusMask::from_raw(3))
        .with_object_group_member(ObjectGroupMember::from_raw(1));
    let locate_request = ClientRequest::locate(locate);
    assert_eq!(format!("{locate_request:?}"), "Locate([REDACTED])");
    let response = operation_batch_response_bytes([
        (
            LOCATE_OPERATION,
            LOCATE_BATCH_ID.to_vec(),
            successful_locate_payload(&["server-id-z"]),
        ),
        (
            GET_OPERATION,
            LOCATE_GET_BATCH_ID.to_vec(),
            successful_get_payload("server-id-z"),
        ),
    ]);
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![7, 13],
    });
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(locate_request).with_unique_batch_item_id(LOCATE_BATCH_ID.to_vec()),
        ClientBatchItem::new(ClientRequest::Get(GetRequest::new()))
            .with_unique_batch_item_id(LOCATE_GET_BATCH_ID.to_vec()),
    ]);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the Locate/Get batch is sent as submitted for server-side placeholder handling");

    assert_eq!(fake.borrow().exchange_count(), 1);
    let captured_request = captured_request.borrow();
    let request = captured_request
        .as_deref()
        .expect("the submitted batch reaches the fake transport");
    let locate_request =
        request_item_snapshot(request, 0).expect("the first request frame is captured");
    assert_locate_request_preserves_criteria(&locate_request);

    let get_request =
        request_item_snapshot(request, 1).expect("the second request frame is captured");
    assert_eq!(get_request.operation, GET_OPERATION);
    assert_eq!(get_request.unique_batch_item_id, LOCATE_GET_BATCH_ID);
    assert_eq!(
        get_request.payload,
        WireValue::Structure(Vec::new()),
        "the placeholder-dependent Get keeps Unique Identifier omitted"
    );

    let locate_result = response.get(0).expect("Locate remains result item zero");
    let get_result = response.get(1).expect("Get remains result item one");
    assert_eq!(locate_result.unique_batch_item_id(), Some(LOCATE_BATCH_ID));
    assert_eq!(get_result.unique_batch_item_id(), Some(LOCATE_GET_BATCH_ID));
    assert_eq!(locate_result.outcome().operation(), ClientOperation::Locate);
    assert_eq!(get_result.outcome().operation(), ClientOperation::Get);
    assert_eq!(
        locate_result
            .outcome()
            .locate_response()
            .expect("the Locate result remains typed")
            .unique_identifiers(),
        &[UniqueIdentifier::TextString("server-id-z".to_owned()),],
        "Locate identifiers preserve server order and repetitions"
    );
    assert_eq!(
        locate_result.outcome().response().result().status().raw(),
        0
    );
    let response_debug = format!("{:?}", locate_result.outcome().response());
    let outcome_debug = format!("{:?}", locate_result.outcome());
    let outcome_display = locate_result.outcome().to_string();
    assert!(response_debug.contains("Locate"));
    assert!(outcome_debug.contains("LocateCompleted"));
    assert!(outcome_display.contains("LocateCompleted"));
    assert!(outcome_display.contains("status 0"));
    assert!(!response_debug.contains("server-id-z"));
    assert!(!outcome_debug.contains("server-id-z"));
    assert!(!outcome_display.contains("server-id-z"));
    assert!(
        get_result.outcome().get_response().is_some(),
        "the subsequent Get response is exposed without a client-selected identifier"
    );
}

#[test]
fn caller_directs_archived_locate_recover_then_get_as_three_exchanges() {
    // OASIS KMIP v2.1 §6.1.28 requires Recover then Get to obtain archived
    // contents; §6.1.42 and §6.1.19 define those separate operations.
    let archived_identifier = "caller-selected-archived-object";
    let responses = [
        one_item_response_bytes(
            LOCATE_OPERATION,
            0,
            None,
            None,
            Some(ARCHIVED_LOCATE_BATCH_ID),
            None,
            Some(successful_locate_payload(&[archived_identifier])),
        ),
        one_item_response_bytes(
            RECOVER_OPERATION,
            0,
            None,
            None,
            Some(ARCHIVED_RECOVER_BATCH_ID),
            None,
            Some(test_structure([test_item(
                UNIQUE_IDENTIFIER,
                Value::text_string(archived_identifier.to_owned()),
            )])),
        ),
        one_item_response_bytes(
            GET_OPERATION,
            0,
            None,
            None,
            Some(ARCHIVED_GET_BATCH_ID),
            None,
            Some(successful_get_payload(archived_identifier)),
        ),
    ];
    let requests = Rc::new(RefCell::new(Vec::new()));
    let mut client = Client::for_test(CallerDirectedArchivedTransport {
        responses: responses.into_iter().collect(),
        requests: Rc::clone(&requests),
    });

    let located_identifier = execute_archived_locate(&mut client, &requests);
    assert_eq!(
        located_identifier,
        UniqueIdentifier::TextString(archived_identifier.to_owned())
    );

    let recovered_identifier = execute_archived_recover(&mut client, &requests, located_identifier);

    execute_archived_get(&mut client, &requests, recovered_identifier);

    let requests = requests.borrow();
    assert_eq!(
        requests
            .iter()
            .map(|request| request.operation)
            .collect::<Vec<_>>(),
        [LOCATE_OPERATION, RECOVER_OPERATION, GET_OPERATION]
    );
    assert_eq!(
        wire_field(&requests[0].payload, STORAGE_STATUS_MASK),
        Some(&WireValue::Integer(0x2)),
        "Locate explicitly requests archived objects"
    );
    for request in &requests[1..] {
        assert_eq!(
            wire_field(&request.payload, UNIQUE_IDENTIFIER),
            Some(&WireValue::TextString(archived_identifier.to_owned()))
        );
    }
}

fn execute_archived_locate(
    client: &mut Client,
    requests: &Rc<RefCell<Vec<RequestItemSnapshot>>>,
) -> UniqueIdentifier {
    let response = client
        .execute(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::Locate(
                    LocateRequest::new(AttributeSet::new())
                        .with_storage_status_mask(StorageStatusMask::from_raw(0x2)),
                ))
                .with_unique_batch_item_id(ARCHIVED_LOCATE_BATCH_ID.to_vec()),
            ),
            &CodecLimits::defaults(),
        )
        .expect("the caller's archived-object search completes");
    assert_eq!(
        requests.borrow().len(),
        1,
        "Locate causes one exchange only"
    );
    response
        .get(0)
        .expect("Locate result item exists")
        .outcome()
        .locate_response()
        .and_then(|response| response.unique_identifiers().first())
        .cloned()
        .expect("the server-returned archived identifier is exposed to the caller")
}

fn execute_archived_recover(
    client: &mut Client,
    requests: &Rc<RefCell<Vec<RequestItemSnapshot>>>,
    located_identifier: UniqueIdentifier,
) -> UniqueIdentifier {
    let response = client
        .execute(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::Recover(RecoverRequest::new(Some(
                    located_identifier,
                ))))
                .with_unique_batch_item_id(ARCHIVED_RECOVER_BATCH_ID.to_vec()),
            ),
            &CodecLimits::defaults(),
        )
        .expect("the caller explicitly starts Recover after reading Locate");
    assert_eq!(
        requests.borrow().len(),
        2,
        "Recover causes one exchange only"
    );
    let recover_item = response.get(0).expect("Recover result item exists");
    assert_eq!(recover_item.outcome().operation(), ClientOperation::Recover);
    assert_eq!(recover_item.outcome().result().status().raw(), 0);
    recover_item
        .outcome()
        .recover_response()
        .and_then(|response| response.unique_identifier())
        .cloned()
        .expect("successful Recover exposes the identifier for the caller's Get")
}

fn execute_archived_get(
    client: &mut Client,
    requests: &Rc<RefCell<Vec<RequestItemSnapshot>>>,
    recovered_identifier: UniqueIdentifier,
) {
    let response = client
        .execute(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::Get(
                    GetRequest::new().with_unique_identifier(recovered_identifier),
                ))
                .with_unique_batch_item_id(ARCHIVED_GET_BATCH_ID.to_vec()),
            ),
            &CodecLimits::defaults(),
        )
        .expect("the caller explicitly starts Get after Recover Success");
    assert_eq!(requests.borrow().len(), 3, "Get causes one exchange only");
    let get_item = response.get(0).expect("Get result item exists");
    assert_eq!(get_item.outcome().operation(), ClientOperation::Get);
    assert!(get_item.outcome().get_response().is_some());
}

#[test]
fn locate_without_storage_status_mask_preserves_all_server_returned_identifiers() {
    let returned_identifiers = [
        "server-returned-fixture-alpha",
        "server-returned-archived-labeled-fixture-id",
        "server-returned-destroyed-labeled-fixture-id",
    ];
    let response = operation_batch_response_bytes([(
        LOCATE_OPERATION,
        LOCATE_BATCH_ID.to_vec(),
        successful_locate_payload(&returned_identifiers),
    )]);
    let (mut client, _, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(
        ClientBatchItem::new(ClientRequest::Locate(LocateRequest::new(
            AttributeSet::new(),
        )))
        .with_unique_batch_item_id(LOCATE_BATCH_ID.to_vec()),
    );

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the server returns a typed Locate response");

    let request = captured_request.borrow();
    let locate_request = request_item_snapshot(
        request
            .as_deref()
            .expect("the Locate request reaches the fake transport"),
        0,
    )
    .expect("the submitted Locate request is captured");
    assert_eq!(
        wire_field(&locate_request.payload, STORAGE_STATUS_MASK),
        None,
        "an omitted Storage Status Mask stays absent on the wire"
    );

    let locate_response = response
        .get(0)
        .expect("one Locate result is returned")
        .outcome()
        .locate_response()
        .expect("the Locate result remains typed");
    assert_eq!(
        locate_response.unique_identifiers(),
        &returned_identifiers.map(|identifier| UniqueIdentifier::TextString(identifier.to_owned())),
        "the client exposes every returned identifier in server order; fixture labels do not assert object status"
    );
}

#[test]
fn locate_forwards_assigned_and_unknown_storage_status_bits_unchanged() {
    let raw_mask = 0x8000_0007;
    let response = operation_batch_response_bytes([(
        LOCATE_OPERATION,
        LOCATE_BATCH_ID.to_vec(),
        successful_locate_payload(&[]),
    )]);
    let (mut client, _, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let locate = LocateRequest::new(AttributeSet::new())
        .with_storage_status_mask(StorageStatusMask::from_raw(raw_mask));
    let batch = ClientBatch::new(
        ClientBatchItem::new(ClientRequest::Locate(locate))
            .with_unique_batch_item_id(LOCATE_BATCH_ID.to_vec()),
    );

    client
        .execute(batch, &CodecLimits::defaults())
        .expect("the server returns a typed Locate response");

    let request = captured_request.borrow();
    let locate_request = request_item_snapshot(
        request
            .as_deref()
            .expect("the Locate request reaches the fake transport"),
        0,
    )
    .expect("the submitted Locate request is captured");
    let Some(WireValue::Integer(encoded_mask)) =
        wire_field(&locate_request.payload, STORAGE_STATUS_MASK)
    else {
        panic!("the explicit Storage Status Mask is encoded as an Integer");
    };
    assert_eq!(
        encoded_mask.to_be_bytes(),
        raw_mask.to_be_bytes(),
        "the assigned Online, Archived, and Destroyed bits plus the unknown bit reach the wire unchanged"
    );
}

fn assert_locate_request_preserves_criteria(request: &RequestItemSnapshot) {
    assert_eq!(request.operation, LOCATE_OPERATION);
    assert_eq!(request.unique_batch_item_id, LOCATE_BATCH_ID);
    assert_eq!(
        wire_field(&request.payload, MAXIMUM_ITEMS),
        Some(&WireValue::Integer(1)),
        "Maximum Items reaches the server unchanged"
    );
    assert_eq!(
        wire_field(&request.payload, OFFSET_ITEMS),
        Some(&WireValue::Integer(0)),
        "an explicit zero Offset Items remains present"
    );
    assert_eq!(
        wire_field(&request.payload, STORAGE_STATUS_MASK),
        Some(&WireValue::Integer(3)),
        "the caller's Storage Status Mask reaches the server unchanged"
    );
    assert_eq!(
        wire_field(&request.payload, OBJECT_GROUP_MEMBER),
        Some(&WireValue::Enumeration(1)),
        "the caller's Object Group Member reaches the server unchanged"
    );
    assert_eq!(
        wire_field(&request.payload, ATTRIBUTES),
        Some(&WireValue::Structure(vec![
            (ACTIVATION_DATE, WireValue::DateTime(100)),
            (ACTIVATION_DATE, WireValue::DateTime(200)),
        ])),
        "repeated Locate criteria retain their input values and order"
    );
}
