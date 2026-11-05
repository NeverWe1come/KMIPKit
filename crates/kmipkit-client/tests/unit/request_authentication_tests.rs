//! KMIPKIT-0050 Authentication Request Header tests.
//!
//! OASIS KMIP v2.1 §8.2, Table 395 places Authentication after Attestation
//! Type and before Batch Error Continuation Option. The current client omits
//! Attestation Type, so Authentication follows Attestation Capable Indicator.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use kmipkit_protocol::{Authentication, RequestMessage};
use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView, codec::decode};

use crate::execute::{
    Client, ClientBatch, ClientBatchItem, ClientRequest, ZeroizationObserver,
    request_message_for_test,
};
use crate::execute_test_support::{ResponseItemFixture, response_bytes};

const REQUEST_HEADER: u32 = 0x0042_0077;
const AUTHENTICATION: u32 = 0x0042_000C;
const ATTESTATION_CAPABLE_INDICATOR: u32 = 0x0042_00D3;
const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000E;
const CREDENTIAL: u32 = 0x0042_0023;
const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;
const VENDOR_FIELD: u32 = 0x0054_0001;
const SECRET_SENTINEL: &str = "AUTHENTICATION_SECRET_SENTINEL";

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("test tag fits TTLV width")
        .try_checked()
        .expect("test tag is allocated")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("test item is valid")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut tree = Structure::new();
    for field in items {
        tree.try_push(field)
            .expect("test tree is within TTLV limits");
    }
    tree
}

fn authentication() -> Authentication {
    let value = structure([item(
        VENDOR_FIELD,
        Value::text_string(SECRET_SENTINEL.to_owned()),
    )]);
    let credential = structure([
        item(CREDENTIAL_TYPE, Value::enumeration(0x8FFF_0042)),
        item(CREDENTIAL_VALUE, Value::structure(value)),
    ]);
    Authentication::try_from_ttlv(structure([item(CREDENTIAL, Value::structure(credential))]))
        .expect("non-empty unknown Credential is retained as generic TTLV")
}

fn header_tags(request: &RequestMessage) -> Vec<u32> {
    request.with_ttlv(|message| {
        message
            .children()
            .iter()
            .find(|field| field.tag().raw() == REQUEST_HEADER)
            .map(|field| {
                field.with_value(|value| match value {
                    ValueView::Structure(header) => header
                        .children()
                        .iter()
                        .map(|field| field.tag().raw())
                        .collect(),
                    _ => Vec::new(),
                })
            })
            .unwrap_or_default()
    })
}

fn authentication_count(request: &RequestMessage) -> usize {
    header_tags(request)
        .into_iter()
        .filter(|tag| *tag == AUTHENTICATION)
        .count()
}

fn contains_text(structure: &kmipkit_ttlv::StructureView<'_>, expected: &str) -> bool {
    structure.children().iter().any(|field| {
        field.with_value(|value| match value {
            ValueView::TextString(value) => value == expected,
            ValueView::Structure(nested) => contains_text(&nested, expected),
            _ => false,
        })
    })
}

struct AuthenticationObservingTransport {
    script: Rc<RefCell<ScriptedTransport>>,
    request_owner_observer: ZeroizationObserver,
    owner_live_during_exchange: Rc<Cell<bool>>,
    secret_reached_transport: Rc<Cell<bool>>,
}

impl Transport for AuthenticationObservingTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        let secret_reached_transport = decode(request).ok().is_some_and(|root| {
            root.with_value(|value| match value {
                ValueView::Structure(message) => contains_text(&message, SECRET_SENTINEL),
                _ => false,
            })
        });
        self.secret_reached_transport.set(secret_reached_transport);
        self.owner_live_during_exchange
            .set(self.request_owner_observer.result().is_none());
        self.script
            .borrow_mut()
            .exchange(request, max_response_bytes)
    }
}

fn authenticated_client(
    script: ExchangeScript,
    observer: ZeroizationObserver,
) -> (
    Client,
    Rc<RefCell<ScriptedTransport>>,
    Rc<Cell<bool>>,
    Rc<Cell<bool>>,
) {
    let script = Rc::new(RefCell::new(ScriptedTransport::new(script)));
    let owner_live_during_exchange = Rc::new(Cell::new(false));
    let secret_reached_transport = Rc::new(Cell::new(false));
    let transport = AuthenticationObservingTransport {
        script: Rc::clone(&script),
        request_owner_observer: observer.clone(),
        owner_live_during_exchange: Rc::clone(&owner_live_during_exchange),
        secret_reached_transport: Rc::clone(&secret_reached_transport),
    };
    let client = Client::for_test_with_request_observer(transport, observer);
    (
        client,
        script,
        owner_live_during_exchange,
        secret_reached_transport,
    )
}

fn authenticated_batch() -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()))
        .with_authentication(authentication())
}

#[test]
fn authentication_is_emitted_once_in_request_header_order_for_one_item() {
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()))
        .with_authentication(authentication());
    let request = request_message_for_test(batch, &CodecLimits::defaults())
        .expect("typed Authentication is encoded in the request header");
    let tags = header_tags(&request);

    assert_eq!(authentication_count(&request), 1);
    assert!(
        tags.iter()
            .position(|tag| *tag == ATTESTATION_CAPABLE_INDICATOR)
            < tags.iter().position(|tag| *tag == AUTHENTICATION)
    );
}

#[test]
fn authentication_is_one_message_field_for_a_multi_item_batch() {
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::discover_versions()).with_unique_batch_item_id(vec![1]),
        ClientBatchItem::new(ClientRequest::discover_versions()).with_unique_batch_item_id(vec![2]),
    ])
    .with_authentication(authentication())
    .with_batch_error_continuation_option(1);
    let request = request_message_for_test(batch, &CodecLimits::defaults())
        .expect("shared header Authentication is encoded once");
    let tags = header_tags(&request);
    let auth_index = tags
        .iter()
        .position(|tag| *tag == AUTHENTICATION)
        .expect("Authentication is present");
    let batch_error_index = tags
        .iter()
        .position(|tag| *tag == BATCH_ERROR_CONTINUATION_OPTION)
        .expect("batch continuation follows Authentication");

    assert_eq!(authentication_count(&request), 1);
    assert!(auth_index < batch_error_index);
}

#[test]
fn authentication_is_omitted_by_default_and_batch_formatting_is_redacted() {
    let plain = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
    let request = request_message_for_test(plain, &CodecLimits::defaults())
        .expect("unauthenticated request remains supported");
    assert_eq!(authentication_count(&request), 0);

    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()))
        .with_authentication(authentication());
    assert!(!format!("{batch:?}").contains(SECRET_SENTINEL));
    assert!(!batch.to_string().contains(SECRET_SENTINEL));
}

#[test]
fn authentication_owner_lives_through_successful_exchange_and_is_zeroized() {
    let observer = ZeroizationObserver::new(None);
    let (mut client, script, owner_live, secret_reached_transport) = authenticated_client(
        ExchangeScript::Success {
            response: response_bytes((2, 1), &[ResponseItemFixture::success(None)]),
            request_write_chunks: vec![2, 3],
        },
        observer.clone(),
    );

    client
        .execute(authenticated_batch(), &CodecLimits::defaults())
        .expect("authenticated Discover Versions response is valid");

    assert!(secret_reached_transport.get());
    assert!(owner_live.get());
    assert_eq!(observer.result(), Some(true));
    assert_eq!(script.borrow().exchange_count(), 1);
}

#[test]
fn authentication_owner_is_zeroized_after_partial_write_without_retry() {
    let observer = ZeroizationObserver::new(None);
    let (mut client, script, owner_live, secret_reached_transport) = authenticated_client(
        ExchangeScript::FailAfterPartialWrite { written_bytes: 2 },
        observer.clone(),
    );

    let error = client
        .execute(authenticated_batch(), &CodecLimits::defaults())
        .expect_err("the transport fails after a partial request write");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert!(secret_reached_transport.get());
    assert!(owner_live.get());
    assert_eq!(observer.result(), Some(true));
    assert_eq!(script.borrow().exchange_count(), 1);
    assert_eq!(script.borrow().write_call_count(), 1);
    assert!(!format!("{error:?}").contains(SECRET_SENTINEL));
    assert!(!error.to_string().contains(SECRET_SENTINEL));
    assert!(
        script
            .borrow()
            .captured_logs()
            .iter()
            .all(|line| !line.contains(SECRET_SENTINEL))
    );
}
