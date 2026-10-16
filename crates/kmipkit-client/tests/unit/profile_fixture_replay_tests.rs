//! Pinned OASIS fixture replay through the public typed client API.
//!
//! These tests establish deterministic fixture replay only. They do not claim
//! server interoperability, profile conformance, or certification.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;

use kmipkit_protocol::{
    ActivateRequest, AddAttributeRequest, AttributeReference, AttributeSet, CheckRequest,
    CreateRequest, CurrentAttribute, DestroyRequest, GetAttributesRequest, GetRequest,
    InteropRequest, LocateRequest, LogRequest, ModifyAttributeRequest, NewAttribute, ObjectType,
    OpaqueObjectContext, RegisterRequest, RevocationReason, RevokeRequest, SecretText,
    UniqueIdentifier,
};
use kmipkit_test_support::oasis_crypto_fixtures::{
    OasisProfileExchange, OasisProfileFixture, OasisProfileFixtureBindings,
};
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, Structure, ValueView};
use zeroize::Zeroizing;

use crate::execute::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientRequest,
    encode_message_for_test,
};

const ROOT_MESSAGE: u32 = 0x0042_0078;
const REQUEST_HEADER: u32 = 0x0042_0077;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013b;
const NEW_ATTRIBUTE: u32 = 0x0042_013d;
const REVOCATION_REASON: u32 = 0x0042_0081;
const REVOCATION_REASON_CODE: u32 = 0x0042_0082;
const REVOCATION_MESSAGE: u32 = 0x0042_0080;
const COMPROMISE_OCCURRENCE_DATE: u32 = 0x0042_0021;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002c;
const INTEROP_FUNCTION: u32 = 0x0042_0160;
const INTEROP_IDENTIFIER: u32 = 0x0042_0161;
const LOG_MESSAGE: u32 = 0x0042_0141;
const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000e;
const CLIENT_CORRELATION_VALUE: u32 = 0x0042_0105;
const RESULT_STATUS: u32 = 0x0042_007f;
const FIXTURE_NOW: i64 = 1_700_000_000;
const FIXTURE_UNIQUE_IDENTIFIER_0: &str = "kmipkit-test-unique-id-0";

struct ReplayTransport {
    responses: VecDeque<Zeroizing<Vec<u8>>>,
    requests: Vec<Zeroizing<Vec<u8>>>,
}

struct SharedReplayTransport(Rc<RefCell<ReplayTransport>>);

impl ReplayTransport {
    fn new(responses: Vec<Vec<u8>>) -> Self {
        Self {
            responses: responses.into_iter().map(Zeroizing::new).collect(),
            requests: Vec::new(),
        }
    }
}

impl Transport for ReplayTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.requests.push(Zeroizing::new(request.to_vec()));
        let Some(mut response) = self.responses.pop_front() else {
            return Err(TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("profile fixture replay response queue exhausted"),
            ));
        };
        if response.len() > max_response_bytes {
            return Err(TransportError::new(
                RequestDeliveryState::PossiblySent,
                TransportCauseCategory::Other,
                io::Error::other("profile fixture replay response exceeded size limit"),
            ));
        }
        Ok(TransportResponse::new(std::mem::take(&mut *response)))
    }
}

impl Transport for SharedReplayTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.0.borrow_mut().exchange(request, max_response_bytes)
    }
}

const PINNED_CASES: &[(&str, &str)] = &[
    (
        "BL-M-1-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-1-21.xml"),
    ),
    (
        "BL-M-2-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-2-21.xml"),
    ),
    (
        "BL-M-3-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-3-21.xml"),
    ),
    (
        "BL-M-4-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-4-21.xml"),
    ),
    (
        "BL-M-5-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-5-21.xml"),
    ),
    (
        "BL-M-6-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-6-21.xml"),
    ),
    (
        "BL-M-7-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-7-21.xml"),
    ),
    (
        "BL-M-8-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-8-21.xml"),
    ),
    (
        "BL-M-9-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-9-21.xml"),
    ),
    (
        "BL-M-10-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-10-21.xml"),
    ),
    (
        "BL-M-11-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-11-21.xml"),
    ),
    (
        "BL-M-12-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-12-21.xml"),
    ),
    (
        "BL-M-13-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/BL-M-13-21.xml"),
    ),
    (
        "SKLC-M-1-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/SKLC-M-1-21.xml"),
    ),
    (
        "SKLC-M-2-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/SKLC-M-2-21.xml"),
    ),
    (
        "SKLC-M-3-21",
        include_str!("../../../../specification/oasis/kmip-2.1/fixtures/SKLC-M-3-21.xml"),
    ),
];
const EXPECTED_RESULTS: &[&str] = &[
    "BL-M-1-21=REPLAYED exchanges=7 operations=7",
    "BL-M-2-21=REPLAYED exchanges=5 operations=7",
    "BL-M-3-21=REPLAYED exchanges=7 operations=7",
    "BL-M-4-21=PINNED_BUT_UNEXECUTED step=1 error=Validation/Some(InvalidInput)/Some(NotSent) sent_exchanges=1 operations_attempted=2 later_steps_unexecuted=5",
    "BL-M-5-21=REPLAYED exchanges=7 operations=7",
    "BL-M-6-21=REPLAYED exchanges=6 operations=6",
    "BL-M-7-21=PINNED_BUT_UNEXECUTED step=2 error=Validation/Some(InvalidInput)/Some(NotSent) sent_exchanges=2 operations_attempted=3 later_steps_unexecuted=3",
    "BL-M-8-21=REPLAYED exchanges=7 operations=7",
    "BL-M-9-21=REPLAYED exchanges=5 operations=5",
    "BL-M-10-21=PINNED_BUT_UNEXECUTED step=1 error=Validation/Some(InvalidInput)/Some(NotSent) sent_exchanges=1 operations_attempted=2 later_steps_unexecuted=3",
    "BL-M-11-21=REPLAYED exchanges=5 operations=5",
    "BL-M-12-21=REPLAYED exchanges=5 operations=5",
    "BL-M-13-21=REPLAYED exchanges=6 operations=6",
    "SKLC-M-1-21=REPLAYED exchanges=3 operations=3",
    "SKLC-M-2-21=REPLAYED exchanges=8 operations=8",
    "SKLC-M-3-21=REPLAYED exchanges=8 operations=8",
];

#[test]
// This one matrix deliberately keeps all 16 exact case IDs and expected outcomes together.
#[allow(clippy::too_many_lines)]
fn every_available_profile_fixture_records_replay_or_typed_blocker() {
    let limits = CodecLimits::defaults();
    let mut results = Vec::with_capacity(PINNED_CASES.len());

    for (case_id, xml) in PINNED_CASES {
        let fixture = OasisProfileFixture::from_xml(
            case_id,
            xml,
            OasisProfileFixtureBindings {
                now: FIXTURE_NOW,
                unique_identifier_0: FIXTURE_UNIQUE_IDENTIFIER_0,
            },
        )
        .unwrap_or_else(|error| panic!("{case_id}: fixture parsing failed: {error}"));
        let responses = fixture
            .exchanges()
            .iter()
            .map(|exchange| {
                encode_message_for_test(
                    exchange
                        .response_message()
                        .view()
                        .try_clone()
                        .unwrap_or_else(|_| panic!("{case_id}: response tree clone failed")),
                    &limits,
                )
                .unwrap_or_else(|_| panic!("{case_id}: response fixture encoding failed"))
            })
            .collect();
        let transport = Rc::new(RefCell::new(ReplayTransport::new(responses)));
        let mut client = Client::for_test(SharedReplayTransport(Rc::clone(&transport)));
        let mut operation_count = 0;
        let mut completed_exchanges = 0;
        let mut blocked = None;

        for exchange in fixture.exchanges() {
            let (batch, expected_batch_items) = typed_batch_for_exchange(case_id, exchange);
            operation_count += batch.items().len();
            let response = match client.execute(batch, &limits) {
                Ok(response) => response,
                Err(error) => {
                    assert_eq!(
                        error.category(),
                        crate::ClientErrorCategory::Validation,
                        "{case_id} step {} failed outside local typed validation",
                        exchange.step()
                    );
                    assert_eq!(
                        error.cause_category(),
                        Some(crate::ClientCauseCategory::InvalidInput),
                        "{case_id} step {} has an unexpected validation cause",
                        exchange.step()
                    );
                    assert_eq!(
                        error.delivery_state(),
                        Some(RequestDeliveryState::NotSent),
                        "{case_id} step {} failed after delivery began",
                        exchange.step()
                    );
                    blocked = Some(format!(
                        "{case_id}=PINNED_BUT_UNEXECUTED step={} error={:?}/{:?}/{:?}",
                        exchange.step(),
                        error.category(),
                        error.cause_category(),
                        error.delivery_state()
                    ));
                    break;
                }
            };
            completed_exchanges += 1;
            assert_eq!(
                response.len(),
                expected_batch_items.len(),
                "{case_id} step {} response count",
                exchange.step()
            );
            let fixture_response_view = exchange.response_message().view();
            let expected_response_items = fixture_response_view
                .children()
                .iter()
                .filter(|item| item.tag().raw() == BATCH_ITEM)
                .collect::<Vec<_>>();
            assert_eq!(
                expected_response_items.len(),
                response.len(),
                "{case_id} step {} fixture response batch-item count",
                exchange.step()
            );
            for (index, operation) in exchange.operations().iter().enumerate() {
                let item_response = response.get(index).unwrap_or_else(|| {
                    panic!("{case_id} step {}: typed result missing", exchange.step())
                });
                assert!(
                    response_matches_operation(item_response.outcome(), *operation),
                    "{case_id} step {} result {index} has a typed outcome for another operation",
                    exchange.step()
                );
                let expected_response_fields =
                    structure_items(expected_response_items[index], case_id, exchange.step());
                let expected_status = find_item(&expected_response_fields, RESULT_STATUS)
                    .map(|item| enumeration(item, case_id, exchange.step()));
                assert_eq!(
                    outcome_result_status(item_response.outcome()),
                    expected_status,
                    "{case_id} step {} typed result status {index} differs from fixture response",
                    exchange.step()
                );
            }

            let transport_state = transport.borrow();
            let captured_request = transport_state.requests.last().unwrap_or_else(|| {
                panic!(
                    "{case_id} step {}: request was not captured",
                    exchange.step()
                )
            });
            verify_emitted_request(
                case_id,
                exchange.step(),
                captured_request,
                &expected_batch_items,
                exchange.request_message(),
                &limits,
            );
        }
        assert_eq!(
            transport.borrow().requests.len(),
            completed_exchanges,
            "{case_id} one fake transport request per completed exchange"
        );
        if blocked.is_none() {
            assert!(
                transport.borrow().responses.is_empty(),
                "{case_id} all fixture responses consumed"
            );
            assert_eq!(
                completed_exchanges,
                fixture.exchanges().len(),
                "{case_id} all exchanges replayed"
            );
            results.push(format!(
                "{case_id}=REPLAYED exchanges={completed_exchanges} operations={operation_count}"
            ));
        } else {
            let remaining = fixture.exchanges().len() - completed_exchanges - 1;
            results.push(format!(
                "{} sent_exchanges={completed_exchanges} operations_attempted={operation_count} later_steps_unexecuted={remaining}",
                blocked.unwrap_or_else(|| panic!("{case_id} blocker status disappeared"))
            ));
        }
    }
    assert_eq!(
        results.len(),
        16,
        "all available profile cases have results"
    );
    assert_eq!(
        results.iter().map(String::as_str).collect::<Vec<_>>(),
        EXPECTED_RESULTS,
        "each pinned case retains its exact replay or blocker result"
    );
    for result in &results {
        println!("{result}");
    }
}

fn verify_emitted_request(
    case_id: &str,
    step: usize,
    bytes: &[u8],
    expected_batch_items: &[Item],
    expected_request: &Structure,
    limits: &CodecLimits,
) {
    let actual_root = decode_with_limits(bytes, limits)
        .unwrap_or_else(|_| panic!("{case_id} step {step}: emitted request TTLV did not decode"));
    assert_eq!(
        actual_root.tag().raw(),
        ROOT_MESSAGE,
        "{case_id} step {step} root tag"
    );
    let actual_root_items = structure_items(&actual_root, case_id, step);
    let actual_batch_items = actual_root_items
        .iter()
        .filter(|item| item.tag().raw() == BATCH_ITEM)
        .collect::<Vec<_>>();
    assert_eq!(
        actual_batch_items.len(),
        expected_batch_items.len(),
        "{case_id} step {step} batch-item count"
    );
    for (index, (actual, expected)) in actual_batch_items
        .iter()
        .zip(expected_batch_items.iter())
        .enumerate()
    {
        assert!(
            items_equal(actual, expected),
            "{case_id} step {step} batch item {index} differs from fixture"
        );
    }
    let expected_request_view = expected_request.view();
    let expected_header = expected_request_view
        .children()
        .iter()
        .find(|item| item.tag().raw() == REQUEST_HEADER)
        .unwrap_or_else(|| panic!("{case_id} step {step}: fixture request header missing"));
    let actual_header = actual_root_items
        .iter()
        .find(|item| item.tag().raw() == REQUEST_HEADER)
        .unwrap_or_else(|| panic!("{case_id} step {step}: emitted request header missing"));
    let expected_header_items = structure_items(expected_header, case_id, step);
    let actual_header_items = structure_items(actual_header, case_id, step);
    for tag in [BATCH_ERROR_CONTINUATION_OPTION, CLIENT_CORRELATION_VALUE] {
        let expected = find_item(&expected_header_items, tag);
        let actual = find_item(&actual_header_items, tag);
        assert_eq!(
            expected.is_some(),
            actual.is_some(),
            "{case_id} step {step} selected header field presence {tag:#08x}"
        );
        if let (Some(expected), Some(actual)) = (expected, actual) {
            assert!(
                items_equal(actual, expected),
                "{case_id} step {step} selected header field {tag:#08x} differs"
            );
        }
    }
}

fn response_matches_operation(outcome: &ClientBatchOutcome, operation: u32) -> bool {
    matches!(
        (operation, outcome),
        (0x0000_0001, ClientBatchOutcome::CreateCompleted(_))
            | (0x0000_0003, ClientBatchOutcome::Register(_))
            | (0x0000_0008, ClientBatchOutcome::LocateCompleted(_))
            | (0x0000_0009, ClientBatchOutcome::Check(_))
            | (0x0000_000a, ClientBatchOutcome::GetCompleted(_))
            | (0x0000_000b, ClientBatchOutcome::GetAttributes(_))
            | (0x0000_000d, ClientBatchOutcome::AddAttribute(_))
            | (0x0000_000e, ClientBatchOutcome::ModifyAttribute(_))
            | (0x0000_0012, ClientBatchOutcome::Activate(_))
            | (0x0000_0013, ClientBatchOutcome::Revoke(_))
            | (0x0000_0014, ClientBatchOutcome::Destroy(_))
            | (0x0000_002c, ClientBatchOutcome::Log(_))
            | (0x0000_0034, ClientBatchOutcome::Interop(_))
    )
}

fn outcome_result_status(outcome: &ClientBatchOutcome) -> Option<u32> {
    match outcome {
        ClientBatchOutcome::Check(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::Register(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::Revoke(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::Activate(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::Destroy(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::GetCompleted(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::LocateCompleted(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::CreateCompleted(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::AddAttribute(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::ModifyAttribute(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::GetAttributes(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::Interop(response) => Some(response.result().status().raw()),
        ClientBatchOutcome::Log(response) => Some(response.result().status().raw()),
        _ => None,
    }
}

fn items_equal(left: &Item, right: &Item) -> bool {
    left.tag().raw() == right.tag().raw()
        && left.with_value(|left_value| {
            right.with_value(|right_value| match (left_value, right_value) {
                (ValueView::Structure(left), ValueView::Structure(right)) => {
                    let left = left.children();
                    let right = right.children();
                    left.len() == right.len()
                        && left
                            .iter()
                            .zip(right)
                            .all(|(left, right)| items_equal(left, right))
                }
                (ValueView::Integer(left), ValueView::Integer(right)) => left == right,
                (ValueView::LongInteger(left), ValueView::LongInteger(right))
                | (ValueView::DateTime(left), ValueView::DateTime(right)) => left == right,
                (ValueView::BigInteger(left), ValueView::BigInteger(right))
                | (ValueView::ByteString(left), ValueView::ByteString(right)) => left == right,
                (ValueView::Enumeration(left), ValueView::Enumeration(right))
                | (ValueView::Interval(left), ValueView::Interval(right)) => left == right,
                (ValueView::Boolean(left), ValueView::Boolean(right)) => left == right,
                (ValueView::TextString(left), ValueView::TextString(right)) => left == right,
                (ValueView::DateTimeExtended(left), ValueView::DateTimeExtended(right)) => {
                    left == right
                }
                _ => false,
            })
        })
}

fn find_item(items: &[Item], tag: u32) -> Option<&Item> {
    items.iter().find(|item| item.tag().raw() == tag)
}

fn typed_batch_for_exchange(
    case_id: &str,
    exchange: &OasisProfileExchange,
) -> (ClientBatch, Vec<Item>) {
    let request_message = exchange.request_message();
    let request_view = request_message.view();
    let request_children = request_view.children();
    let header = request_children
        .iter()
        .find(|item| item.tag().raw() == REQUEST_HEADER)
        .unwrap_or_else(|| panic!("{case_id} step {}: request header missing", exchange.step()));
    let header_items = structure_items(header, case_id, exchange.step());
    let mut batch_items = Vec::new();
    let mut expected_batch_items = Vec::new();
    for batch_item in request_children
        .iter()
        .filter(|item| item.tag().raw() == BATCH_ITEM)
    {
        let children = structure_items(batch_item, case_id, exchange.step());
        let operation = find_item(&children, OPERATION)
            .unwrap_or_else(|| panic!("{case_id} step {}: operation missing", exchange.step()));
        let payload = find_item(&children, REQUEST_PAYLOAD).unwrap_or_else(|| {
            panic!(
                "{case_id} step {}: request payload missing",
                exchange.step()
            )
        });
        let operation_raw = enumeration(operation, case_id, exchange.step());
        let request = typed_request(operation_raw, payload, case_id, exchange.step());
        expected_batch_items.push(batch_item.try_clone().unwrap_or_else(|_| {
            panic!(
                "{case_id} step {}: expected item clone failed",
                exchange.step()
            )
        }));
        let batch_item = if let Some(id) = find_item(&children, UNIQUE_BATCH_ITEM_ID) {
            ClientBatchItem::new(request).with_unique_batch_item_id(bytes(
                id,
                case_id,
                exchange.step(),
            ))
        } else {
            ClientBatchItem::new(request)
        };
        batch_items.push(batch_item);
    }

    let mut batch = ClientBatch::from_items(batch_items);
    if let Some(option) = find_item(&header_items, BATCH_ERROR_CONTINUATION_OPTION) {
        batch = batch.with_batch_error_continuation_option(enumeration(
            option,
            case_id,
            exchange.step(),
        ));
    }
    if let Some(correlation) = find_item(&header_items, CLIENT_CORRELATION_VALUE) {
        batch = batch.with_client_correlation_value(text(correlation, case_id, exchange.step()));
    }
    (batch, expected_batch_items)
}

// Keep the supported operation mapping exhaustive and fail-closed in one place.
#[allow(clippy::too_many_lines)]
fn typed_request(operation: u32, payload: &Item, case_id: &str, step: usize) -> ClientRequest {
    let fields = structure_items(payload, case_id, step);
    match operation {
        0x0000_0001 => {
            let object_type = ObjectType::from_raw(enumeration(
                required_item(&fields, OBJECT_TYPE, case_id, step),
                case_id,
                step,
            ));
            let attributes = attribute_set(
                required_item(&fields, ATTRIBUTES, case_id, step),
                case_id,
                step,
            );
            ClientRequest::Create(CreateRequest::new(object_type, attributes))
        }
        0x0000_0003 => {
            let object_type = ObjectType::from_raw(enumeration(
                required_item(&fields, OBJECT_TYPE, case_id, step),
                case_id,
                step,
            ));
            let attributes = attribute_set(
                required_item(&fields, ATTRIBUTES, case_id, step),
                case_id,
                step,
            );
            let excluded_tags = [OBJECT_TYPE, ATTRIBUTES];
            let object = fields
                .iter()
                .find(|item| !excluded_tags.contains(&item.tag().raw()))
                .unwrap_or_else(|| panic!("{case_id} step {step}: Register object missing"))
                .try_clone()
                .unwrap_or_else(|_| panic!("{case_id} step {step}: Register object clone failed"));
            let request = RegisterRequest::new(object_type, attributes, object);
            let request = if fields.iter().any(|item| item.tag().raw() == 0x0042_005b) {
                request.with_opaque_object_context(OpaqueObjectContext::Generic)
            } else {
                request
            };
            ClientRequest::Register(request)
        }
        0x0000_0008 => {
            let attributes = attribute_set(
                required_item(&fields, ATTRIBUTES, case_id, step),
                case_id,
                step,
            );
            ClientRequest::Locate(LocateRequest::new(attributes))
        }
        0x0000_0009 => {
            let identifier = optional_item(&fields, UNIQUE_IDENTIFIER)
                .map(|item| unique_identifier(item, case_id, step));
            let mask = optional_item(&fields, CRYPTOGRAPHIC_USAGE_MASK)
                .map(|item| integer(item, case_id, step));
            let mut request = CheckRequest::new();
            if let Some(identifier) = identifier {
                request = request.with_unique_identifier(identifier);
            }
            if let Some(mask) = mask {
                request = request.with_cryptographic_usage_mask(mask);
            }
            ClientRequest::Check(request)
        }
        0x0000_000a => {
            let mut request = GetRequest::new();
            if let Some(identifier) = optional_item(&fields, UNIQUE_IDENTIFIER) {
                request =
                    request.with_unique_identifier(unique_identifier(identifier, case_id, step));
            }
            ClientRequest::Get(request)
        }
        0x0000_000b => {
            let identifier =
                optional_item(&fields, UNIQUE_IDENTIFIER).map(|item| text(item, case_id, step));
            let references = fields
                .iter()
                .filter(|item| item.tag().raw() == ATTRIBUTE_REFERENCE)
                .map(|item| AttributeReference::tag(enumeration(item, case_id, step)))
                .collect::<Vec<_>>();
            let request =
                GetAttributesRequest::try_new(identifier, references).unwrap_or_else(|_| {
                    panic!("{case_id} step {step}: invalid typed Attribute Reference list")
                });
            ClientRequest::GetAttributes(request)
        }
        0x0000_000d => {
            let identifier =
                optional_item(&fields, UNIQUE_IDENTIFIER).map(|item| text(item, case_id, step));
            let new_attribute = required_item(&fields, NEW_ATTRIBUTE, case_id, step);
            let attribute = structure_items(new_attribute, case_id, step)
                .into_iter()
                .next()
                .unwrap_or_else(|| panic!("{case_id} step {step}: Add Attribute item missing"));
            ClientRequest::AddAttribute(AddAttributeRequest::new(
                identifier,
                NewAttribute::new(attribute),
            ))
        }
        0x0000_000e => {
            let identifier =
                optional_item(&fields, UNIQUE_IDENTIFIER).map(|item| text(item, case_id, step));
            let new_attribute = required_item(&fields, NEW_ATTRIBUTE, case_id, step);
            let attribute = structure_items(new_attribute, case_id, step)
                .into_iter()
                .next()
                .unwrap_or_else(|| panic!("{case_id} step {step}: Modify Attribute item missing"));
            ClientRequest::ModifyAttribute(ModifyAttributeRequest::new(
                identifier,
                None::<CurrentAttribute>,
                NewAttribute::new(attribute),
            ))
        }
        0x0000_0012 => {
            let identifier = optional_item(&fields, UNIQUE_IDENTIFIER)
                .map(|item| unique_identifier(item, case_id, step));
            ClientRequest::Activate(ActivateRequest::new(identifier))
        }
        0x0000_0013 => {
            let reason = required_item(&fields, REVOCATION_REASON, case_id, step);
            let reason_fields = structure_items(reason, case_id, step);
            let code = enumeration(
                required_item(&reason_fields, REVOCATION_REASON_CODE, case_id, step),
                case_id,
                step,
            );
            let mut request = RevokeRequest::new(RevocationReason::from_raw(code));
            if let Some(identifier) = optional_item(&fields, UNIQUE_IDENTIFIER) {
                request =
                    request.with_unique_identifier(unique_identifier(identifier, case_id, step));
            }
            if let Some(message) = optional_item(&reason_fields, REVOCATION_MESSAGE) {
                request = request.with_revocation_message(text(message, case_id, step));
            }
            if let Some(date) = optional_item(&fields, COMPROMISE_OCCURRENCE_DATE) {
                request = request.with_compromise_occurrence_date(date_time(date, case_id, step));
            }
            ClientRequest::Revoke(request)
        }
        0x0000_0014 => {
            let identifier = optional_item(&fields, UNIQUE_IDENTIFIER)
                .map(|item| unique_identifier(item, case_id, step));
            ClientRequest::Destroy(DestroyRequest::new(identifier))
        }
        0x0000_002c => {
            let message = text(
                required_item(&fields, LOG_MESSAGE, case_id, step),
                case_id,
                step,
            );
            ClientRequest::Log(LogRequest::new(SecretText::new(message)))
        }
        0x0000_0034 => {
            let function = enumeration(
                required_item(&fields, INTEROP_FUNCTION, case_id, step),
                case_id,
                step,
            );
            let identifier = text(
                required_item(&fields, INTEROP_IDENTIFIER, case_id, step),
                case_id,
                step,
            );
            ClientRequest::Interop(InteropRequest::new(function, SecretText::new(identifier)))
        }
        _ => panic!("{case_id} step {step}: unsupported typed operation {operation:#010x}"),
    }
}

fn attribute_set(item: &Item, case_id: &str, step: usize) -> AttributeSet {
    let attributes = structure_items(item, case_id, step);
    AttributeSet::try_new(attributes)
        .unwrap_or_else(|_| panic!("{case_id} step {step}: typed Attributes validation failed"))
}

fn required_item<'a>(items: &'a [Item], tag: u32, case_id: &str, step: usize) -> &'a Item {
    optional_item(items, tag)
        .unwrap_or_else(|| panic!("{case_id} step {step}: required field tag {tag:#08x} missing"))
}

fn optional_item(items: &[Item], tag: u32) -> Option<&Item> {
    items.iter().find(|item| item.tag().raw() == tag)
}

fn structure_items(item: &Item, case_id: &str, step: usize) -> Vec<Item> {
    item.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(|child| {
                child.try_clone().unwrap_or_else(|_| {
                    panic!("{case_id} step {step}: generic fixture item clone failed")
                })
            })
            .collect(),
        _ => panic!("{case_id} step {step}: expected a Structure TTLV value"),
    })
}

fn enumeration(item: &Item, case_id: &str, step: usize) -> u32 {
    item.with_value(|value| match value {
        ValueView::Enumeration(value) => *value,
        _ => panic!("{case_id} step {step}: expected an Enumeration TTLV value"),
    })
}

fn integer(item: &Item, case_id: &str, step: usize) -> i32 {
    item.with_value(|value| match value {
        ValueView::Integer(value) => *value,
        _ => panic!("{case_id} step {step}: expected an Integer TTLV value"),
    })
}

fn date_time(item: &Item, case_id: &str, step: usize) -> i64 {
    item.with_value(|value| match value {
        ValueView::DateTime(value) => *value,
        _ => panic!("{case_id} step {step}: expected a DateTime TTLV value"),
    })
}

fn text(item: &Item, case_id: &str, step: usize) -> String {
    item.with_value(|value| match value {
        ValueView::TextString(value) => value.to_owned(),
        _ => panic!("{case_id} step {step}: expected a TextString TTLV value"),
    })
}

fn bytes(item: &Item, case_id: &str, step: usize) -> Vec<u8> {
    item.with_value(|value| match value {
        ValueView::ByteString(value) => value.to_vec(),
        _ => panic!("{case_id} step {step}: expected a ByteString TTLV value"),
    })
}

fn unique_identifier(item: &Item, case_id: &str, step: usize) -> UniqueIdentifier {
    item.with_value(|value| match value {
        ValueView::TextString(value) => UniqueIdentifier::TextString(value.to_owned()),
        ValueView::Enumeration(value) => UniqueIdentifier::Enumeration(*value),
        ValueView::Integer(value) => UniqueIdentifier::Integer(*value),
        _ => panic!("{case_id} step {step}: unsupported Unique Identifier Item Type"),
    })
}
