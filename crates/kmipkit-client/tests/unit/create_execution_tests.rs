//! Fake-transport Create execution tests derived from KMIP 2.1 §6.1.8,
//! Tables 186–188, §8.6/Table 399, and §9.13/Table 418. These are project
//! execution tests, not claims of full `TC-CREATE-SD-1-21` conformance.

use std::cell::RefCell;
use std::rc::Rc;

use kmipkit_protocol::extension as protocol_extension;
use kmipkit_protocol::{AttributeSet, CreateRequest, ObjectType};
use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value};

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

const CREATE: u32 = 0x0000_0001;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
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
const SECRET_SENTINEL: &[u8] = b"KMIP_CREATE_SECRET_SENTINEL_2391";
const RESPONSE_SENTINEL: &str = "KMIP_CREATE_RESPONSE_SENTINEL_8432";
const PENDING_CORRELATION: &[u8] = b"CREATE_PENDING_CORRELATION_EXACT";
const EXTENSION_VENDOR: &str = "CreateFixtureVendor";
const EXTENSION_DISCRIMINATOR: &[u8] = b"create-fixture-v1";
const DISCRIMINATOR_TAG: u32 = 0x0042_0173;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits the 24-bit KMIP field")
        .try_checked()
        .expect("fixture tag is allocated by the KMIP 2.1 catalog")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("fixture item uses a checked tag")
}

fn response_batch_item(
    id: Option<&[u8]>,
    status: u32,
    reason: Option<u32>,
    result_message: Option<&str>,
    correlation: Option<&[u8]>,
    payload: Option<Structure>,
) -> Item {
    let mut fields = vec![item(OPERATION, Value::enumeration(CREATE))];
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

fn success_payload(identifier: &str) -> Structure {
    test_structure([
        test_item(OBJECT_TYPE, Value::enumeration(7)),
        test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
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

fn empty_request() -> CreateRequest {
    CreateRequest::new(ObjectType::from_raw(7), AttributeSet::new())
}

fn secret_request() -> CreateRequest {
    let vendor_attribute = test_structure([
        test_item(0x0042_009d, Value::text_string("TestVendor".to_owned())),
        test_item(0x0042_000a, Value::text_string("SecretValue".to_owned())),
        test_item(0x0042_000b, Value::byte_string(SECRET_SENTINEL.to_vec())),
    ]);
    let attributes =
        AttributeSet::try_new([test_item(ATTRIBUTES, Value::structure(vendor_attribute))])
            .expect("the Vendor Attribute fixture has its required fields");
    CreateRequest::new(ObjectType::from_raw(7), attributes)
}

fn create_item(id: &[u8]) -> ClientBatchItem {
    ClientBatchItem::new(ClientRequest::Create(empty_request()))
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
fn create_batch_is_one_exchange_and_associates_reordered_results_by_exact_ids() {
    let first_id = b"create-first";
    let second_id = b"create-second";
    let response = response_bytes([
        response_batch_item(
            Some(second_id),
            SUCCESS,
            None,
            None,
            None,
            Some(success_payload("object-second")),
        ),
        response_batch_item(
            Some(first_id),
            SUCCESS,
            None,
            None,
            None,
            Some(success_payload("object-first")),
        ),
    ]);
    let (mut client, fake) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![3, 7, 2],
    });
    let batch = ClientBatch::from_items([create_item(first_id), create_item(second_id)]);

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the two Create results associate by Unique Batch Item ID");

    assert_eq!(result.len(), 2);
    assert_eq!(
        result.get(0).and_then(|item| item.unique_batch_item_id()),
        Some(first_id.as_slice())
    );
    assert_eq!(
        result.get(1).and_then(|item| item.unique_batch_item_id()),
        Some(second_id.as_slice())
    );
    let ClientBatchOutcome::CreateCompleted(first) = result.get(0).unwrap().outcome() else {
        panic!("first request has a completed Create outcome");
    };
    let ClientBatchOutcome::CreateCompleted(second) = result.get(1).unwrap().outcome() else {
        panic!("second request has a completed Create outcome");
    };
    assert_eq!(
        first.unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            "object-first".to_owned()
        ))
    );
    assert_eq!(
        second.unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            "object-second".to_owned()
        ))
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn create_convenience_method_uses_one_exchange_and_returns_typed_result() {
    let response = response_bytes([response_batch_item(
        None,
        SUCCESS,
        None,
        None,
        None,
        Some(success_payload("object-convenience")),
    )]);
    let (mut client, fake) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![4, 5],
    });

    let response = client
        .create(empty_request(), &CodecLimits::defaults())
        .expect("Client::create dispatches through the shared writer");

    let ClientBatchOutcome::CreateCompleted(typed) = response.outcome() else {
        panic!("the convenience method returns the typed Create outcome");
    };
    assert_eq!(typed.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            "object-convenience".to_owned()
        ))
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn create_failure_preserves_result_and_does_not_retry() {
    let response = response_bytes([response_batch_item(
        Some(b"create-failure"),
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
    let batch = ClientBatch::new(create_item(b"create-failure"));

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("KMIP Failure is returned as a completed operation result");
    let ClientBatchOutcome::CreateCompleted(response) = result.get(0).unwrap().outcome() else {
        panic!("server Failure is a completed Create result");
    };
    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(|reason| reason.raw()),
        Some(RESPONSE_TOO_LARGE)
    );
    assert!(!format!("{response:?}").contains(RESPONSE_SENTINEL));
    assert!(!format!("{result:?}").contains(RESPONSE_SENTINEL));
    assert!(
        fake.borrow()
            .captured_logs()
            .iter()
            .all(|line| !line.contains(RESPONSE_SENTINEL))
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn create_pending_retains_operation_result_and_exact_correlation_without_follow_up() {
    let response = response_bytes([response_batch_item(
        Some(b"create-pending"),
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
    let batch = ClientBatch::new(create_item(b"create-pending")).with_asynchronous_indicator(1);

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("Pending is surfaced as an explicit resumable result");
    let ClientBatchOutcome::Pending(pending) = result.get(0).unwrap().outcome() else {
        panic!("the Pending result stays Pending");
    };
    assert_eq!(pending.operation(), ClientOperation::Create);
    assert_eq!(pending.result().status().raw(), OPERATION_PENDING);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn create_request_owner_is_zeroized_after_success_and_post_write_error() {
    let request = secret_request();
    let success_item = ClientBatchItem::new(ClientRequest::Create(request))
        .with_unique_batch_item_id(b"create-secret".to_vec());
    assert!(!format!("{success_item:?}").contains("KMIP_CREATE_SECRET_SENTINEL"));
    let success_response = response_bytes([response_batch_item(
        Some(b"create-secret"),
        SUCCESS,
        None,
        None,
        None,
        Some(success_payload("object-secret")),
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
        .expect("Create completes after one exchange");
    assert!(request_contains(&success_capture, SECRET_SENTINEL));
    assert_eq!(success_observer.result(), Some(true));
    assert_eq!(success_fake.borrow().exchange_count(), 1);
    assert!(!format!("{success:?}").contains("KMIP_CREATE_SECRET_SENTINEL"));

    let failure_observer = ZeroizationObserver::new(None);
    let (mut client, failure_fake, failure_capture) = client_for_with_request_observer(
        ExchangeScript::FailAfterPartialWrite { written_bytes: 3 },
        failure_observer.clone(),
    );
    let error = client
        .execute(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::Create(secret_request()))
                    .with_unique_batch_item_id(b"create-secret-error".to_vec()),
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
    assert!(!error.to_string().contains("KMIP_CREATE_SECRET_SENTINEL"));
    assert!(!format!("{error:?}").contains("KMIP_CREATE_SECRET_SENTINEL"));
    assert!(
        failure_fake
            .borrow()
            .captured_logs()
            .iter()
            .all(|line| !line.contains("KMIP_CREATE_SECRET_SENTINEL"))
    );
}

#[test]
fn malformed_create_response_redacts_raw_payload_sentinel_after_one_exchange() {
    let malformed = test_structure([
        test_item(
            OBJECT_TYPE,
            Value::text_string(RESPONSE_SENTINEL.to_owned()),
        ),
        test_item(UNIQUE_IDENTIFIER, Value::text_string("object-1".to_owned())),
    ]);
    let response = response_bytes([response_batch_item(
        Some(b"create-malformed"),
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
            ClientBatch::new(create_item(b"create-malformed")),
            &CodecLimits::defaults(),
        )
        .expect_err("the Create response Object Type must be an Enumeration");

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
    let identity = protocol_extension::extension_identity(EXTENSION_VENDOR, "create-fixture", "1")
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
    let identity = protocol_extension::extension_identity(EXTENSION_VENDOR, "create-fixture", "1")
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
fn create_keeps_repeated_registered_message_extensions_in_caller_order() {
    let response = response_bytes([response_batch_item(
        Some(b"create-extensions"),
        SUCCESS,
        None,
        None,
        None,
        Some(success_payload("object-ext")),
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
        ClientBatchItem::new(ClientRequest::Create(empty_request()))
            .with_extension(extension1)
            .with_extension(extension2)
            .with_unique_batch_item_id(b"create-extensions".to_vec()),
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
