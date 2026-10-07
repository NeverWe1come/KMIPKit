//! Fake-transport response inspection and KMIPKIT-0007 criticality behavior.
//!
//! Traceability: KMIPKIT-0012-FR-006 through FR-008, KMIPKIT-0007-FR-010,
//! KMIP 2.1 §9.13, Table 418. Registry inspection here consumes the generic
//! subtree returned by the existing response model; execute-time recognition
//! mapping remains the KMIPKIT-0012 T037 implementation responsibility.

use std::cell::RefCell;
use std::rc::Rc;

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
use crate::extension_registry::{self, ClientExtensionRegistry};

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
        test_item(VENDOR_IDENTIFICATION, Value::text_string(VENDOR.to_owned())),
        test_item(CRITICALITY_INDICATOR, Value::boolean(criticality)),
        test_item(VENDOR_EXTENSION, Value::structure(extension_payload)),
    ]);
    let batch_item = test_structure([
        test_item(
            OPERATION,
            Value::enumeration(crate::execute_test_support::DISCOVER_VERSIONS_OPERATION),
        ),
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
    let transport = Rc::new(RefCell::new(ScriptedTransport::new(
        ExchangeScript::Success {
            response,
            request_write_chunks: vec![3, 7, 2],
        },
    )));
    let client = Client::for_test(SharedFakeTransport(Rc::clone(&transport)));
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
                ValueView::Structure(payload) => copy_structure(payload),
                _ => panic!("the response Vendor Extension is a Structure"),
            });
        (vendor, criticality, payload)
    })
}

fn copy_structure(view: StructureView<'_>) -> Structure {
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
        ValueView::Structure(value) => Value::structure(copy_structure(value)),
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
