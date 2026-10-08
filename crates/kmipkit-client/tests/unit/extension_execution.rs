//! Fake-transport response inspection and KMIPKIT-0007 criticality behavior.
//!
//! Traceability: KMIPKIT-0012-FR-006 through FR-008, KMIPKIT-0007-FR-010,
//! KMIP 2.1 §9.13, Table 418. Registry inspection is integrated with typed
//! response mapping while KMIPKIT-0007 retains unknown-extension criticality.

use std::cell::RefCell;
use std::rc::Rc;

use kmipkit_protocol::PollRequest;
use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, Structure, StructureView, Value, ValueView};

use crate::ClientErrorCategory;
use crate::execute::{
    Client, ClientBatch, ClientBatchItem, ClientMessageExtension, ClientRequest,
    encode_message_for_test,
};
use crate::execute_test_support::{test_item, test_structure};
use crate::extension_registry::{self, ClientConfiguration, ClientExtensionRegistry};

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const MESSAGE_EXTENSION: u32 = 0x0042_0051;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const CRITICALITY_INDICATOR: u32 = 0x0042_0026;
const VENDOR_EXTENSION: u32 = 0x0042_009C;
const DISCRIMINATOR_TAG: u32 = 0x0042_0173;
const UNKNOWN_PAYLOAD_TAG: u32 = 0x0042_0174;
const VENDOR: &str = "FixtureVendor";
const DISCRIMINATOR: &[u8] = b"fixture-v1";
const UNKNOWN_ENUMERATION: u32 = u32::MAX;

fn response_with_extension(criticality: bool) -> Vec<u8> {
    response_with_extension_for_operation(
        criticality,
        crate::execute_test_support::DISCOVER_VERSIONS_OPERATION,
    )
}

fn response_with_extension_for_operation(criticality: bool, operation: u32) -> Vec<u8> {
    response_with_extension_vendor(criticality, operation, VENDOR.to_owned())
}

fn response_with_extension_vendor(criticality: bool, operation: u32, vendor: String) -> Vec<u8> {
    let mut extension_payload = Structure::new();
    extension_payload
        .try_push(test_item(
            DISCRIMINATOR_TAG,
            Value::byte_string(DISCRIMINATOR.to_vec()),
        ))
        .expect("the discriminator fits the extension payload");
    extension_payload
        .try_push(test_item(
            UNKNOWN_PAYLOAD_TAG,
            Value::enumeration(UNKNOWN_ENUMERATION),
        ))
        .expect("the unknown field fits the extension payload");

    let response_header = test_structure([
        test_item(
            PROTOCOL_VERSION,
            Value::structure(test_structure([
                test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
                test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
            ])),
        ),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let response_payload = test_structure([test_item(
        PROTOCOL_VERSION,
        Value::structure(test_structure([
            test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
            test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
        ])),
    )]);
    let extension = test_structure([
        test_item(VENDOR_IDENTIFICATION, Value::text_string(vendor)),
        test_item(CRITICALITY_INDICATOR, Value::boolean(criticality)),
        test_item(VENDOR_EXTENSION, Value::structure(extension_payload)),
    ]);
    let batch_item = test_structure([
        test_item(OPERATION, Value::enumeration(operation)),
        test_item(RESULT_STATUS, Value::enumeration(0)),
        test_item(RESPONSE_PAYLOAD, Value::structure(response_payload)),
        test_item(MESSAGE_EXTENSION, Value::structure(extension)),
    ]);
    let tree = test_structure([
        test_item(RESPONSE_HEADER, Value::structure(response_header)),
        test_item(BATCH_ITEM, Value::structure(batch_item)),
    ]);

    encode_message_for_test(tree, &CodecLimits::defaults())
        .expect("the deterministic fake response is valid TTLV")
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

fn client_for(response: Vec<u8>) -> (Client, Rc<RefCell<ScriptedTransport>>) {
    client_for_configuration(response, ClientConfiguration::new(empty_registry()))
}

fn client_for_configuration(
    response: Vec<u8>,
    configuration: ClientConfiguration,
) -> (Client, Rc<RefCell<ScriptedTransport>>) {
    let transport = Rc::new(RefCell::new(ScriptedTransport::new(
        ExchangeScript::Success {
            response,
            request_write_chunks: vec![3, 7, 2],
        },
    )));
    let client = Client::for_test_with_configuration(
        SharedFakeTransport(Rc::clone(&transport)),
        configuration,
    );
    (client, transport)
}

fn extension_parts(extension: &ClientMessageExtension) -> (String, bool, Structure) {
    extension.with_ttlv(|view| {
        let vendor = view
            .children()
            .iter()
            .find(|field| field.tag().raw() == VENDOR_IDENTIFICATION)
            .expect("the response extension includes Vendor Identification")
            .with_value(|value| match value {
                ValueView::TextString(vendor) => vendor.to_owned(),
                _ => panic!("the response Vendor Identification is a Text String"),
            });
        let criticality = view
            .children()
            .iter()
            .find(|field| field.tag().raw() == CRITICALITY_INDICATOR)
            .expect("the response extension includes Criticality Indicator")
            .with_value(|value| match value {
                ValueView::Boolean(value) => *value,
                _ => panic!("the response Criticality Indicator is a Boolean"),
            });
        let payload = view
            .children()
            .iter()
            .find(|field| field.tag().raw() == VENDOR_EXTENSION)
            .expect("the response extension includes Vendor Extension")
            .with_value(|value| match value {
                ValueView::Structure(payload) => copy_structure(&payload),
                _ => panic!("the response Vendor Extension is a Structure"),
            });
        (vendor, criticality, payload)
    })
}

fn copy_structure(view: &StructureView<'_>) -> Structure {
    let mut copied = Structure::new();
    for child in view.children() {
        let value = child.with_value(copy_value);
        copied
            .try_push(Item::new(child.tag(), value).expect("the copied item remains valid"))
            .expect("the copied response Structure remains within the model depth limit");
    }
    copied
}

fn copy_value(view: ValueView<'_>) -> Value {
    match view {
        ValueView::Structure(value) => Value::structure(copy_structure(&value)),
        ValueView::Integer(value) => Value::integer(*value),
        ValueView::LongInteger(value) => Value::long_integer(*value),
        ValueView::BigInteger(value) => Value::big_integer(value.to_vec()),
        ValueView::Enumeration(value) => Value::enumeration(*value),
        ValueView::Boolean(value) => Value::boolean(*value),
        ValueView::TextString(value) => Value::text_string(value.to_owned()),
        ValueView::ByteString(value) => Value::byte_string(value.to_vec()),
        ValueView::DateTime(value) => Value::date_time(*value),
        ValueView::Interval(value) => Value::interval(*value),
        ValueView::DateTimeExtended(value) => Value::date_time_extended(*value),
        _ => panic!("the response fixture contains a supported generic TTLV value"),
    }
}

fn registry() -> ClientExtensionRegistry {
    let identity = kmipkit_protocol::extension::extension_identity(VENDOR, "fixture", "1")
        .expect("the fixture identity is valid");
    let compatibility = kmipkit_protocol::extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the fixture extension supports the client");
    let path = kmipkit_protocol::extension::ttlv_path(
        kmipkit_ttlv::RawTag::new(DISCRIMINATOR_TAG)
            .expect("the discriminator tag fits TTLV")
            .try_checked()
            .expect("the discriminator tag is allocated"),
    )
    .expect("the discriminator path is valid");
    let discriminator = kmipkit_protocol::extension::discriminator(
        path,
        Value::byte_string(DISCRIMINATOR.to_vec()),
    )
    .expect("the discriminator is valid");
    let schema = kmipkit_protocol::extension::structure(
        vec![
            kmipkit_protocol::extension::required(
                kmipkit_ttlv::RawTag::new(DISCRIMINATOR_TAG)
                    .expect("the schema tag fits TTLV")
                    .try_checked()
                    .expect("the schema tag is allocated"),
                kmipkit_protocol::extension::scalar(ItemType::ByteString)
                    .expect("Byte String is supported by the schema"),
            )
            .expect("the discriminator schema rule is valid"),
        ],
        Vec::new(),
        true,
    )
    .expect("the fixture schema is valid");
    let definition = kmipkit_protocol::extension::extension_definition(
        identity,
        compatibility,
        discriminator,
        schema,
    )
    .expect("the fixture extension definition is valid");
    extension_registry::client_extension_registry(
        vec![definition],
        kmipkit_protocol::extension::defaults(),
    )
    .expect("the fixture registry is valid")
}

fn empty_registry() -> ClientExtensionRegistry {
    extension_registry::client_extension_registry(
        Vec::new(),
        kmipkit_protocol::extension::defaults(),
    )
    .expect("an empty registry is valid")
}

fn registered_request_extension(
    registry: &ClientExtensionRegistry,
    criticality: bool,
) -> crate::extension_registry::ClientRequestMessageExtension {
    let identity = kmipkit_protocol::extension::extension_identity(VENDOR, "fixture", "1")
        .expect("the fixture identity is valid");
    let mut payload = Structure::new();
    payload
        .try_push(test_item(
            DISCRIMINATOR_TAG,
            Value::byte_string(DISCRIMINATOR.to_vec()),
        ))
        .expect("the discriminator fits the extension payload");
    let value = extension_registry::validate_extension_value(
        registry,
        identity,
        payload,
        &CodecLimits::defaults(),
    )
    .expect("the extension value belongs to the source registry");
    extension_registry::client_request_message_extension(value, criticality)
        .expect("request criticality is explicit")
}

#[test]
fn an_unregistered_critical_response_extension_is_rejected_without_retry() {
    let (mut client, transport) = client_for(response_with_extension(true));
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &CodecLimits::defaults(),
        )
        .expect_err("§9.13 requires rejection of an unrecognized critical extension");

    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(
        transport.borrow().exchange_count(),
        1,
        "execution does not retry"
    );
}

#[test]
fn oversized_response_extension_vendor_is_rejected_with_protocol_delivery_evidence() {
    let response = response_with_extension_vendor(
        false,
        crate::execute_test_support::DISCOVER_VERSIONS_OPERATION,
        "v".repeat(4_097),
    );
    let (mut client, transport) = client_for(response);

    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &CodecLimits::defaults(),
        )
        .expect_err("response extension text over the registry limit is rejected");

    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
    assert!(!format!("{error}").contains("vvvvvvvv"));
}

#[test]
fn a_registered_critical_response_extension_is_accepted_by_typed_execution() {
    let registered = registry();
    let (mut client, transport) = client_for_configuration(
        response_with_extension(true),
        ClientConfiguration::new(registry()),
    );
    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &CodecLimits::defaults(),
        )
        .expect("a recognized critical extension is not an unknown critical extension");
    let extension = &response.get(0).expect("one response item").extensions()[0];
    let (vendor, criticality, payload) = extension_parts(extension);
    let recognition =
        extension_registry::inspect(&registered, &vendor, payload, &CodecLimits::defaults())
            .expect("the returned generic payload remains inspectable");

    assert!(criticality);
    assert!(extension_registry::is_recognized(&recognition));
    let discriminator = String::from_utf8_lossy(DISCRIMINATOR);
    assert!(!format!("{recognition:?}").contains(discriminator.as_ref()));
    assert!(!format!("{extension:?}").contains(VENDOR));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn a_registered_critical_async_response_extension_is_accepted() {
    let registered = registry();
    let (mut client, transport) = client_for_configuration(
        response_with_extension_for_operation(true, 0x0000_001A),
        ClientConfiguration::new(registry()),
    );
    let outcome = client
        .execute_poll(
            PollRequest::new(b"poll-correlation"),
            &CodecLimits::defaults(),
        )
        .expect("a registered critical extension is recognized in an async response");

    assert_eq!(outcome.operation(), crate::ClientOperation::Poll);
    let extension = &outcome.extensions()[0];
    let (vendor, criticality, payload) = extension_parts(extension);
    let recognition =
        extension_registry::inspect(&registered, &vendor, payload, &CodecLimits::defaults())
            .expect("the returned async generic payload remains inspectable");
    assert!(criticality);
    assert!(extension_registry::is_recognized(&recognition));
    assert!(!format!("{extension:?}").contains(VENDOR));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn a_registered_noncritical_response_payload_is_recognized_and_preserved() {
    let (mut client, transport) = client_for(response_with_extension(false));
    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &CodecLimits::defaults(),
        )
        .expect("an unrecognized noncritical extension is accepted by KMIPKIT-0007");
    let extension = &response.get(0).expect("one response item").extensions()[0];
    let (vendor, criticality, payload) = extension_parts(extension);
    let payload_tags = payload
        .view()
        .children()
        .iter()
        .map(|item| item.tag().raw())
        .collect::<Vec<_>>();

    assert_eq!(vendor, VENDOR);
    assert!(!criticality);
    assert_eq!(payload_tags, [DISCRIMINATOR_TAG, UNKNOWN_PAYLOAD_TAG]);
    assert_eq!(
        payload.view().children()[1].with_value(|value| match value {
            ValueView::Enumeration(value) => *value,
            _ => panic!("the unknown payload field remains an Enumeration"),
        }),
        UNKNOWN_ENUMERATION
    );

    let recognition =
        extension_registry::inspect(&registry(), &vendor, payload, &CodecLimits::defaults())
            .expect("the bounded registered extension payload is inspectable");
    assert!(extension_registry::is_recognized(&recognition));
    assert_eq!(
        kmipkit_protocol::extension::validated_extension_value_identity(
            extension_registry::validated_value(&recognition)
                .expect("the registered response payload receives a typed view"),
        )
        .name(),
        "fixture"
    );
    assert_eq!(
        extension_registry::generic_value(&recognition)
            .view()
            .children()
            .iter()
            .map(|item| item.tag().raw())
            .collect::<Vec<_>>(),
        payload_tags,
        "typed inspection retains the original generic child order"
    );
    assert_eq!(
        transport.borrow().exchange_count(),
        1,
        "execution does not retry"
    );
}

#[test]
fn an_unregistered_noncritical_response_payload_stays_generic_and_preserved() {
    let (mut client, transport) = client_for(response_with_extension(false));
    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &CodecLimits::defaults(),
        )
        .expect("an unrecognized noncritical extension is processed as absent");
    let extension = &response.get(0).expect("one response item").extensions()[0];
    let (vendor, criticality, payload) = extension_parts(extension);
    let recognition = extension_registry::inspect(
        &empty_registry(),
        &vendor,
        payload,
        &CodecLimits::defaults(),
    )
    .expect("the bounded unknown extension payload is inspectable generically");

    assert!(!criticality);
    assert!(!extension_registry::is_recognized(&recognition));
    assert!(extension_registry::validated_value(&recognition).is_none());
    assert_eq!(
        extension_registry::generic_value(&recognition)
            .view()
            .children()
            .iter()
            .map(|item| item.tag().raw())
            .collect::<Vec<_>>(),
        [DISCRIMINATOR_TAG, UNKNOWN_PAYLOAD_TAG]
    );
    assert_eq!(
        transport.borrow().exchange_count(),
        1,
        "execution does not retry"
    );
}

#[test]
fn a_request_extension_from_another_client_registry_is_rejected_before_transport() {
    let source_registry = registry();
    let extension = registered_request_extension(&source_registry, false);
    let (mut client, transport) = client_for_configuration(
        response_with_extension(false),
        ClientConfiguration::new(empty_registry()),
    );

    let error = client
        .execute(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::discover_versions()).with_extension(extension),
            ),
            &CodecLimits::defaults(),
        )
        .expect_err("registered request values are scoped to their client registry");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}
