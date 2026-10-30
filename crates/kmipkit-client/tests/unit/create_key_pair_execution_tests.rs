//! Fake-transport Create Key Pair execution tests derived from KMIP 2.1
//! §6.1.9, Tables 189–192, §8.6/Table 399, and §9.13/Table 418. These are
//! table-derived project tests; the pinned OASIS work product has no direct
//! Create Key Pair test case.

use std::cell::RefCell;
use std::rc::Rc;

use kmipkit_protocol::extension as protocol_extension;
use kmipkit_protocol::{AttributeSet, CreateKeyPairRequest};
use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};
use serde_json::Value as JsonValue;

use crate::ClientErrorCategory;
use crate::asynchronous_execution_test_support::{
    client_for_with_request_observer, request_contains,
};
use crate::execute::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
    ZeroizationObserver, encode_message_for_test,
};
use crate::execute_test_support::{test_item, test_structure};
use crate::extension_registry::{self, ClientConfiguration, ClientExtensionRegistry};

const CREATE_KEY_PAIR: u32 = 0x0000_0002;
const PRIVATE_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_0066;
const PUBLIC_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_006f;
const ATTRIBUTES: u32 = 0x0042_0008;
const RESPONSE_HEADER: u32 = 0x0042_007a;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006a;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006b;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000d;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const RESULT_STATUS: u32 = 0x0042_007f;
const RESULT_REASON: u32 = 0x0042_007e;
const RESULT_MESSAGE: u32 = 0x0042_007d;
const RESPONSE_PAYLOAD: u32 = 0x0042_007c;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const RESPONSE_TOO_LARGE: u32 = 0x0000_0002;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const SECRET_SENTINEL: &[u8] = b"KMIP_CREATE_KEY_PAIR_SECRET_SENTINEL_2391";
const RESPONSE_SENTINEL: &str = "KMIP_CREATE_KEY_PAIR_RESPONSE_SENTINEL_8432";
const PENDING_CORRELATION: &[u8] = b"CREATE_KEY_PAIR_PENDING_CORRELATION_EXACT";
const EXTENSION_VENDOR: &str = "CreateKeyPairFixtureVendor";
const EXTENSION_DISCRIMINATOR: &[u8] = b"create_key_pair-fixture-v1";
const DISCRIMINATOR_TAG: u32 = 0x0042_0173;
const SHARED_FIXTURE: &str =
    include_str!("../../../../tests/fixtures/api/shared-capability-parity.json");

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits the 24-bit KMIP field")
        .try_checked()
        .expect("fixture tag is allocated by the KMIP 2.1 catalog")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("fixture item uses a checked tag")
}

fn shared_fixture_case(id: &str) -> JsonValue {
    let fixture: JsonValue =
        serde_json::from_str(SHARED_FIXTURE).expect("shared API parity fixture is valid JSON");
    fixture["cases"]
        .as_array()
        .expect("shared fixture has cases")
        .iter()
        .find(|case| case["id"] == id)
        .cloned()
        .expect("Create Key Pair fixture case exists")
}

fn fixture_tag_list(value: &JsonValue) -> Vec<u32> {
    value
        .as_array()
        .expect("fixture attribute tags are an array")
        .iter()
        .map(|tag| {
            u32::from_str_radix(
                tag.as_str()
                    .expect("fixture attribute tag is text")
                    .strip_prefix("0x")
                    .expect("fixture attribute tag has a hex prefix"),
                16,
            )
            .expect("fixture attribute tag is hexadecimal")
        })
        .collect()
}

fn fixture_attribute_set(tags: &[u32]) -> AttributeSet {
    AttributeSet::try_new(
        tags.iter()
            .map(|raw_tag| item(*raw_tag, Value::enumeration(0xdead_beef))),
    )
    .expect("fixture attribute values form a valid ordered set")
}

fn request_has_key_pair_fixture_groups(request: &[u8], case: &JsonValue) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000f;
    const OPERATION: u32 = 0x0042_005c;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;
    const CREATE_KEY_PAIR: u32 = 2;

    let Ok(root) = decode_with_limits(request, &CodecLimits::defaults()) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        message.children().iter().any(|batch_item| {
            if batch_item.tag().raw() != BATCH_ITEM {
                return false;
            }
            batch_item.with_value(|value| {
                let ValueView::Structure(batch_item) = value else {
                    return false;
                };
                let operation = batch_item.children().iter().find(|field| {
                    field.tag().raw() == OPERATION
                        && field.with_value(|value| {
                            matches!(value, kmipkit_ttlv::ValueView::Enumeration(raw) if *raw == CREATE_KEY_PAIR)
                        })
                });
                let payload = batch_item
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == REQUEST_PAYLOAD);
                operation.is_some()
                    && payload.is_some_and(|payload| {
                        payload.with_value(|value| {
                            let ValueView::Structure(payload) = value else {
                                return false;
                            };
                            let actual = payload
                                .children()
                                .iter()
                                .map(|group| {
                                    let child_tags = group.with_value(|value| match value {
                                        ValueView::Structure(attributes) => attributes
                                            .children()
                                            .iter()
                                            .map(|attribute| attribute.tag().raw())
                                            .collect::<Vec<_>>(),
                                        _ => Vec::new(),
                                    });
                                    (group.tag().raw(), child_tags)
                                })
                                .collect::<Vec<_>>();
                            let input = &case["input"];
                            let expected = vec![
                                (
                                    0x0042_0126,
                                    fixture_tag_list(&input["commonAttributes"]["attributeTagsInOrder"]),
                                ),
                                (
                                    0x0042_0127,
                                    fixture_tag_list(&input["privateKeyAttributes"]["attributeTagsInOrder"]),
                                ),
                                (
                                    0x0042_0128,
                                    fixture_tag_list(&input["publicKeyAttributes"]["attributeTagsInOrder"]),
                                ),
                                (0x0042_0163, Vec::new()),
                                (0x0042_0165, Vec::new()),
                            ];
                            actual == expected
                        })
                    })
            })
        })
    })
}

fn response_batch_item(
    id: Option<&[u8]>,
    status: u32,
    reason: Option<u32>,
    result_message: Option<&str>,
    correlation: Option<&[u8]>,
    payload: Option<Structure>,
) -> Item {
    let mut fields = vec![item(OPERATION, Value::enumeration(CREATE_KEY_PAIR))];
    if let Some(id) = id {
        fields.push(item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id.to_vec())));
    }
    fields.push(item(RESULT_STATUS, Value::enumeration(status)));
    if let Some(reason) = reason {
        fields.push(item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(message) = result_message {
        fields.push(item(RESULT_MESSAGE, Value::text_string(message.to_owned())));
    }
    if let Some(correlation) = correlation {
        fields.push(item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(correlation.to_vec()),
        ));
    }
    if let Some(payload) = payload {
        fields.push(item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    item(BATCH_ITEM, Value::structure(test_structure(fields)))
}

fn success_payload(private_identifier: &str, public_identifier: &str) -> Structure {
    test_structure([
        test_item(
            PRIVATE_KEY_UNIQUE_IDENTIFIER,
            Value::text_string(private_identifier.to_owned()),
        ),
        test_item(
            PUBLIC_KEY_UNIQUE_IDENTIFIER,
            Value::text_string(public_identifier.to_owned()),
        ),
    ])
}

fn response_bytes(items: impl IntoIterator<Item = Item>) -> Vec<u8> {
    let items = items.into_iter().collect::<Vec<_>>();
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(
            BATCH_COUNT,
            Value::integer(i32::try_from(items.len()).expect("fixture count fits i32")),
        ),
    ]);
    let mut message = vec![item(RESPONSE_HEADER, Value::structure(header))];
    message.extend(items);
    encode_message_for_test(test_structure(message), &CodecLimits::defaults())
        .expect("fixture response message is encodable")
}

fn empty_request() -> CreateKeyPairRequest {
    CreateKeyPairRequest::new()
}

fn secret_request() -> CreateKeyPairRequest {
    let vendor_attribute = test_structure([
        test_item(0x0042_009d, Value::text_string("TestVendor".to_owned())),
        test_item(0x0042_000a, Value::text_string("SecretValue".to_owned())),
        test_item(0x0042_000b, Value::byte_string(SECRET_SENTINEL.to_vec())),
    ]);
    let attributes =
        AttributeSet::try_new([test_item(ATTRIBUTES, Value::structure(vendor_attribute))])
            .expect("the Vendor Attribute fixture has its required fields");
    CreateKeyPairRequest::new().with_common_attributes(attributes)
}

fn create_key_pair_item(id: &[u8]) -> ClientBatchItem {
    ClientBatchItem::new(ClientRequest::CreateKeyPair(empty_request()))
        .with_unique_batch_item_id(id.to_vec())
}

fn client_for(script: ExchangeScript) -> (Client, Rc<RefCell<ScriptedTransport>>) {
    let transport = Rc::new(RefCell::new(ScriptedTransport::new(script)));
    let client = Client::for_test(SharedFakeTransport(Rc::clone(&transport)));
    (client, transport)
}

#[derive(Clone)]
struct SharedFakeTransport(Rc<RefCell<ScriptedTransport>>);

impl Transport for SharedFakeTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.0.borrow_mut().exchange(request, max_response_bytes)
    }
}

struct CapturingFakeTransport {
    fake: Rc<RefCell<ScriptedTransport>>,
    captured_request: Rc<RefCell<Option<zeroize::Zeroizing<Vec<u8>>>>>,
}

impl Transport for CapturingFakeTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        *self.captured_request.borrow_mut() = Some(zeroize::Zeroizing::new(request.to_vec()));
        self.fake.borrow_mut().exchange(request, max_response_bytes)
    }
}

#[test]
fn create_key_pair_batch_is_one_exchange_and_associates_reordered_results_by_exact_ids() {
    let first_id = b"create_key_pair-first";
    let second_id = b"create_key_pair-second";
    let response = response_bytes([
        response_batch_item(
            Some(second_id),
            SUCCESS,
            None,
            None,
            None,
            Some(success_payload("private-second", "public-second")),
        ),
        response_batch_item(
            Some(first_id),
            SUCCESS,
            None,
            None,
            None,
            Some(success_payload("private-first", "public-first")),
        ),
    ]);
    let (mut client, fake) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![3, 7, 2],
    });
    let batch = ClientBatch::from_items([
        create_key_pair_item(first_id),
        create_key_pair_item(second_id),
    ]);

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the two CreateKeyPair results associate by Unique Batch Item ID");

    assert_eq!(result.len(), 2);
    assert_eq!(
        result.get(0).and_then(|item| item.unique_batch_item_id()),
        Some(first_id.as_slice())
    );
    assert_eq!(
        result.get(1).and_then(|item| item.unique_batch_item_id()),
        Some(second_id.as_slice())
    );
    let ClientBatchOutcome::CreateKeyPairCompleted(first) = result.get(0).unwrap().outcome() else {
        panic!("first request has a completed CreateKeyPair outcome");
    };
    let ClientBatchOutcome::CreateKeyPairCompleted(second) = result.get(1).unwrap().outcome()
    else {
        panic!("second request has a completed CreateKeyPair outcome");
    };
    assert_eq!(
        first.private_key_unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            "private-first".to_owned()
        ))
    );
    assert_eq!(
        first.public_key_unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            "public-first".to_owned()
        ))
    );
    assert_eq!(
        second.private_key_unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            "private-second".to_owned()
        ))
    );
    assert_eq!(
        second.public_key_unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            "public-second".to_owned()
        ))
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn create_key_pair_maps_shared_fixture_over_one_exchange() {
    let case = shared_fixture_case("creation.key-pair.request-response.v1");
    let input = &case["input"];
    let expected = &case["expectedOutput"];
    let common_tags = fixture_tag_list(&input["commonAttributes"]["attributeTagsInOrder"]);
    let private_tags = fixture_tag_list(&input["privateKeyAttributes"]["attributeTagsInOrder"]);
    let public_tags = fixture_tag_list(&input["publicKeyAttributes"]["attributeTagsInOrder"]);
    assert_eq!(expected["groupsRemainDistinct"], true);
    assert_eq!(input["effectiveTable191ValuesEqual"], true);
    assert_eq!(input["protectionStorageMasks"]["commonPresent"], true);
    assert_eq!(input["protectionStorageMasks"]["privatePresent"], false);
    assert_eq!(input["protectionStorageMasks"]["publicPresent"], true);
    assert_eq!(expected["resultMessage"], serde_json::Value::Null);
    assert_eq!(expected["resultStatus"].as_u64(), Some(u64::from(SUCCESS)));

    let private_identifier = &expected["privateUniqueIdentifier"];
    let private_value = match private_identifier["wireType"].as_str() {
        Some("Enumeration") => Value::enumeration(
            u32::try_from(
                private_identifier["enumeration"]
                    .as_u64()
                    .expect("fixture private identifier is unsigned"),
            )
            .expect("fixture private identifier fits u32"),
        ),
        other => panic!("unsupported fixture private identifier wire type {other:?}"),
    };
    let public_identifier = &expected["publicUniqueIdentifier"];
    let public_value = match public_identifier["wireType"].as_str() {
        Some("TextString") => Value::text_string(
            public_identifier["text"]
                .as_str()
                .expect("fixture public identifier is text")
                .to_owned(),
        ),
        other => panic!("unsupported fixture public identifier wire type {other:?}"),
    };
    let success_payload = test_structure([
        test_item(PRIVATE_KEY_UNIQUE_IDENTIFIER, private_value),
        test_item(PUBLIC_KEY_UNIQUE_IDENTIFIER, public_value),
    ]);
    let response = response_bytes([response_batch_item(
        None,
        u32::try_from(
            expected["resultStatus"]
                .as_u64()
                .expect("fixture status is unsigned"),
        )
        .expect("fixture status fits u32"),
        None,
        None,
        None,
        Some(success_payload),
    )]);
    let request = CreateKeyPairRequest::new()
        .with_common_attributes(fixture_attribute_set(&common_tags))
        .with_private_key_attributes(fixture_attribute_set(&private_tags))
        .with_public_key_attributes(fixture_attribute_set(&public_tags))
        .with_common_protection_storage_masks(Structure::new())
        .with_public_protection_storage_masks(Structure::new());
    let (mut client, fake, request_capture) = client_for_with_request_observer(
        ExchangeScript::Success {
            response,
            request_write_chunks: vec![4, 5],
        },
        ZeroizationObserver::new(None),
    );

    let response = client
        .create_key_pair(request, &CodecLimits::defaults())
        .expect("Client::create_key_pair dispatches through the shared writer");

    let ClientBatchOutcome::CreateKeyPairCompleted(typed) = response.outcome() else {
        panic!("the convenience method returns the typed CreateKeyPair outcome");
    };
    let outcome = response.outcome();
    assert_eq!(outcome.operation(), ClientOperation::CreateKeyPair);
    assert_eq!(outcome.result(), typed.result());
    assert_eq!(outcome.asynchronous_correlation_value(), None);
    let response_view = outcome.response();
    assert_eq!(response_view.result(), typed.result());
    assert!(response_view.discover_versions().is_none());
    assert!(response_view.supported_versions().is_none());
    assert!(response_view.create().is_none());
    assert_eq!(response_view.create_key_pair(), Some(typed));
    assert!(response_view.create_split_key().is_none());
    assert!(response_view.add_attribute().is_none());
    assert!(response_view.adjust_attribute().is_none());
    assert!(response_view.delete_attribute().is_none());
    assert!(response_view.modify_attribute().is_none());
    assert!(response_view.set_attribute().is_none());
    assert!(response_view.get_attributes().is_none());
    assert!(response_view.get_attribute_list().is_none());
    assert!(format!("{response_view:?}").starts_with("CreateKeyPair("));
    assert!(outcome.discover_versions_response().is_none());
    assert!(outcome.create_response().is_none());
    assert!(outcome.create_key_pair_response().is_some());
    assert!(outcome.create_split_key_response().is_none());
    assert!(format!("{outcome:?}").starts_with("CreateKeyPairCompleted("));
    assert_eq!(
        typed.result().status().raw(),
        u32::try_from(
            expected["resultStatus"]
                .as_u64()
                .expect("fixture status is unsigned")
        )
        .expect("fixture status fits u32")
    );
    assert!(typed.result().message().is_none());
    assert_eq!(
        typed.private_key_unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::Enumeration(
            u32::try_from(
                private_identifier["enumeration"]
                    .as_u64()
                    .expect("fixture private identifier is unsigned")
            )
            .expect("fixture private identifier fits u32")
        ))
    );
    assert_eq!(
        typed.public_key_unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            public_identifier["text"]
                .as_str()
                .expect("fixture public identifier is text")
                .to_owned()
        ))
    );
    let request = request_capture.borrow();
    let request = request
        .as_ref()
        .expect("the fake transport observed a request");
    assert!(request_has_key_pair_fixture_groups(request, &case));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn create_key_pair_failure_preserves_result_and_does_not_retry() {
    let response = response_bytes([response_batch_item(
        Some(b"create_key_pair-failure"),
        OPERATION_FAILED,
        Some(RESPONSE_TOO_LARGE),
        Some(RESPONSE_SENTINEL),
        None,
        None,
    )]);
    let (mut client, fake) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(create_key_pair_item(b"create_key_pair-failure"));

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("KMIP Failure is returned as a completed operation result");
    let ClientBatchOutcome::CreateKeyPairCompleted(response) = result.get(0).unwrap().outcome()
    else {
        panic!("server Failure is a completed CreateKeyPair result");
    };
    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(RESPONSE_TOO_LARGE)
    );
    assert!(!format!("{response:?}").contains(RESPONSE_SENTINEL));
    assert!(!format!("{result:?}").contains(RESPONSE_SENTINEL));
    assert!(
        result
            .get(0)
            .is_some_and(|item| !item.outcome().to_string().contains(RESPONSE_SENTINEL))
    );
    assert!(
        fake.borrow()
            .captured_logs()
            .iter()
            .all(|line| !line.contains(RESPONSE_SENTINEL))
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn create_key_pair_pending_retains_operation_result_and_exact_correlation_without_follow_up() {
    let response = response_bytes([response_batch_item(
        Some(b"create_key_pair-pending"),
        OPERATION_PENDING,
        None,
        None,
        Some(PENDING_CORRELATION),
        Some(Structure::new()),
    )]);
    let (mut client, fake) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(create_key_pair_item(b"create_key_pair-pending"))
        .with_asynchronous_indicator(1);

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("Pending is surfaced as an explicit resumable result");
    let ClientBatchOutcome::Pending(pending) = result.get(0).unwrap().outcome() else {
        panic!("the Pending result stays Pending");
    };
    let outcome = result.get(0).unwrap().outcome();
    assert_eq!(outcome.operation(), ClientOperation::CreateKeyPair);
    assert_eq!(outcome.result(), pending.result());
    assert_eq!(
        outcome.asynchronous_correlation_value(),
        Some(PENDING_CORRELATION)
    );
    assert!(outcome.discover_versions_response().is_none());
    assert!(outcome.create_response().is_none());
    assert!(outcome.create_key_pair_response().is_none());
    assert!(outcome.create_split_key_response().is_none());
    assert!(format!("{outcome:?}").starts_with("Pending("));
    let response_view = outcome.response();
    assert_eq!(response_view.result(), pending.result());
    assert!(response_view.discover_versions().is_none());
    assert!(response_view.supported_versions().is_none());
    assert!(response_view.create().is_none());
    assert!(response_view.create_key_pair().is_some());
    assert!(response_view.create_split_key().is_none());
    assert!(response_view.add_attribute().is_none());
    assert!(response_view.adjust_attribute().is_none());
    assert!(response_view.delete_attribute().is_none());
    assert!(response_view.modify_attribute().is_none());
    assert!(response_view.set_attribute().is_none());
    assert!(response_view.get_attributes().is_none());
    assert!(response_view.get_attribute_list().is_none());
    assert!(format!("{response_view:?}").starts_with("CreateKeyPair("));
    assert_eq!(pending.operation(), ClientOperation::CreateKeyPair);
    assert_eq!(pending.result().status().raw(), OPERATION_PENDING);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn create_key_pair_request_owner_is_zeroized_after_success_and_post_write_error() {
    let request = secret_request();
    let success_item = ClientBatchItem::new(ClientRequest::CreateKeyPair(request))
        .with_unique_batch_item_id(b"create_key_pair-secret".to_vec());
    assert!(!format!("{success_item:?}").contains("KMIP_CREATE_KEY_PAIR_SECRET_SENTINEL"));
    let success_response = response_bytes([response_batch_item(
        Some(b"create_key_pair-secret"),
        SUCCESS,
        None,
        None,
        None,
        Some(success_payload("private-secret", "public-secret")),
    )]);
    let success_observer = ZeroizationObserver::new(None);
    let (mut client, success_fake, success_capture) = client_for_with_request_observer(
        ExchangeScript::Success {
            response: success_response,
            request_write_chunks: vec![3, 2],
        },
        success_observer.clone(),
    );
    let success = client
        .execute(ClientBatch::new(success_item), &CodecLimits::defaults())
        .expect("CreateKeyPair completes after one exchange");
    assert!(request_contains(&success_capture, SECRET_SENTINEL));
    assert_eq!(success_observer.result(), Some(true));
    assert_eq!(success_fake.borrow().exchange_count(), 1);
    assert!(!format!("{success:?}").contains("KMIP_CREATE_KEY_PAIR_SECRET_SENTINEL"));

    let failure_observer = ZeroizationObserver::new(None);
    let (mut client, failure_fake, failure_capture) = client_for_with_request_observer(
        ExchangeScript::FailAfterPartialWrite { written_bytes: 3 },
        failure_observer.clone(),
    );
    let error = client
        .execute(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::CreateKeyPair(secret_request()))
                    .with_unique_batch_item_id(b"create_key_pair-secret-error".to_vec()),
            ),
            &CodecLimits::defaults(),
        )
        .expect_err("the fake transport fails after a partial write");
    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert!(request_contains(&failure_capture, SECRET_SENTINEL));
    assert_eq!(failure_observer.result(), Some(true));
    assert_eq!(failure_fake.borrow().exchange_count(), 1);
    assert!(
        !error
            .to_string()
            .contains("KMIP_CREATE_KEY_PAIR_SECRET_SENTINEL")
    );
    assert!(!format!("{error:?}").contains("KMIP_CREATE_KEY_PAIR_SECRET_SENTINEL"));
    assert!(
        failure_fake
            .borrow()
            .captured_logs()
            .iter()
            .all(|line| !line.contains("KMIP_CREATE_KEY_PAIR_SECRET_SENTINEL"))
    );
}

#[test]
fn malformed_create_key_pair_response_redacts_raw_payload_sentinel_after_one_exchange() {
    let malformed = test_structure([
        test_item(
            PRIVATE_KEY_UNIQUE_IDENTIFIER,
            Value::byte_string(RESPONSE_SENTINEL.as_bytes().to_vec()),
        ),
        test_item(
            PUBLIC_KEY_UNIQUE_IDENTIFIER,
            Value::text_string("public-1".to_owned()),
        ),
    ]);
    let response = response_bytes([response_batch_item(
        Some(b"create_key_pair-malformed"),
        SUCCESS,
        None,
        None,
        None,
        Some(malformed),
    )]);
    let (mut client, fake) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let error = client
        .execute(
            ClientBatch::new(create_key_pair_item(b"create_key_pair-malformed")),
            &CodecLimits::defaults(),
        )
        .expect_err("the CreateKeyPair response Object Type must be an Enumeration");

    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert!(!error.to_string().contains(RESPONSE_SENTINEL));
    assert!(!format!("{error:?}").contains(RESPONSE_SENTINEL));
    assert!(
        fake.borrow()
            .captured_logs()
            .iter()
            .all(|line| !line.contains(RESPONSE_SENTINEL))
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

fn registered_registry() -> ClientExtensionRegistry {
    let identity =
        protocol_extension::extension_identity(EXTENSION_VENDOR, "create_key_pair-fixture", "1")
            .expect("the fixture extension identity is valid");
    let compatibility = protocol_extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the fixture extension supports the client");
    let discriminator_tag = tag(DISCRIMINATOR_TAG);
    let path = protocol_extension::ttlv_path(discriminator_tag)
        .expect("the fixture discriminator path is valid");
    let discriminator = protocol_extension::discriminator(
        path,
        Value::byte_string(EXTENSION_DISCRIMINATOR.to_vec()),
    )
    .expect("the fixture discriminator is valid");
    let schema = protocol_extension::structure(
        vec![
            protocol_extension::required(
                discriminator_tag,
                protocol_extension::scalar(ItemType::ByteString)
                    .expect("Byte String is a supported extension type"),
            )
            .expect("the fixture schema rule is valid"),
        ],
        Vec::new(),
        true,
    )
    .expect("the fixture schema is valid");
    let definition =
        protocol_extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("the fixture definition is valid");
    extension_registry::client_extension_registry(vec![definition], protocol_extension::defaults())
        .expect("the fixture registry is valid")
}

fn registered_extension(
    registry: &ClientExtensionRegistry,
) -> crate::extension_registry::ClientRequestMessageExtension {
    let identity =
        protocol_extension::extension_identity(EXTENSION_VENDOR, "create_key_pair-fixture", "1")
            .expect("the fixture extension identity is valid");
    let payload = test_structure([test_item(
        DISCRIMINATOR_TAG,
        Value::byte_string(EXTENSION_DISCRIMINATOR.to_vec()),
    )]);
    let value = extension_registry::validate_extension_value(
        registry,
        identity,
        payload,
        &CodecLimits::defaults(),
    )
    .expect("the extension value matches the registered schema");
    extension_registry::client_request_message_extension(value, false)
        .expect("request criticality is explicit")
}

#[test]
fn create_key_pair_keeps_repeated_registered_message_extensions_in_caller_order() {
    let response = response_bytes([response_batch_item(
        Some(b"create_key_pair-extensions"),
        SUCCESS,
        None,
        None,
        None,
        Some(success_payload("private-ext", "public-ext")),
    )]);
    let registry = registered_registry();
    let extension1 = registered_extension(&registry);
    let extension2 = registered_extension(&registry);
    let fake = Rc::new(RefCell::new(ScriptedTransport::new(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
    )));
    let captured_request = Rc::new(RefCell::new(None));
    let mut client = Client::for_test_with_configuration(
        CapturingFakeTransport {
            fake: Rc::clone(&fake),
            captured_request: Rc::clone(&captured_request),
        },
        ClientConfiguration::new(registry),
    );
    let batch = ClientBatch::new(
        ClientBatchItem::new(ClientRequest::CreateKeyPair(empty_request()))
            .with_extension(extension1)
            .with_extension(extension2)
            .with_unique_batch_item_id(b"create_key_pair-extensions".to_vec()),
    );

    let _ = client
        .execute(batch, &CodecLimits::defaults())
        .expect("both registry-validated extensions are emitted");

    let capture_guard = captured_request.borrow();
    let captured = capture_guard
        .as_deref()
        .expect("the fake retains the request for test inspection");
    let extension_tag = [0x42, 0x00, 0x51, 0x01];
    let count = captured
        .windows(extension_tag.len())
        .filter(|window| *window == extension_tag)
        .count();
    assert_eq!(count, 2);
    let decoded = decode_with_limits(captured, &CodecLimits::defaults())
        .expect("the captured request remains valid TTLV");
    assert_eq!(decoded.item_type(), ItemType::Structure);
    assert_eq!(fake.borrow().exchange_count(), 1);
}
