//! Outbound Message Extension and secret-buffer lifecycle contracts.
//!
//! Traceability: KMIPKIT-0012-FR-005, FR-011, KMIPKIT-0007-OD-006, and
//! KMIP 2.1 §8.3 Table 396 / §9.13 Table 418.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use kmipkit_protocol::extension;
use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::{CodecLimits, DecodeError, decode_with_limits};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

use crate::execute::{Client, ClientBatch, ClientBatchItem, ClientRequest, ZeroizationObserver};
use crate::execute_test_support::{ResponseItemFixture, response_bytes};
use crate::extension_registry::{
    self, ClientConfiguration, ClientExtensionRegistry, ClientRequestMessageExtension,
};

const DISCRIMINATOR_TAG: u32 = 0x0054_0001;
const SECRET_TAG: u32 = 0x0054_0002;
const MESSAGE_EXTENSION_TAG: u32 = 0x0042_0051;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
const CRITICALITY_INDICATOR_TAG: u32 = 0x0042_0026;
const VENDOR_EXTENSION_TAG: u32 = 0x0042_009C;
const SECRET_SENTINEL: &[u8] = b"KMIPKIT_EXTENSION_SECRET_SENTINEL_73";

#[derive(Clone, Debug, Eq, PartialEq)]
struct CapturedExtension {
    vendor: String,
    criticality: bool,
    payload_tags: Vec<u32>,
}

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("extension test tag fits the KMIP width")
        .try_checked()
        .expect("extension test tag uses the vendor allocation")
}

fn request_extension_definition(
    vendor: &str,
    name: &str,
    discriminator: &str,
) -> extension::ExtensionDefinition {
    let identity = extension::extension_identity(vendor, name, "1")
        .expect("request extension identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("request extension supports this client");
    let path =
        extension::ttlv_path(tag(DISCRIMINATOR_TAG)).expect("request discriminator path is valid");
    let discriminator_value =
        extension::discriminator(path, Value::text_string(discriminator.to_owned()))
            .expect("request discriminator is valid");
    let schema = extension::structure(
        vec![
            extension::required(
                tag(DISCRIMINATOR_TAG),
                extension::scalar(ItemType::TextString).expect("Text String is supported"),
            )
            .expect("discriminator schema child is valid"),
            extension::required(
                tag(SECRET_TAG),
                extension::scalar(ItemType::ByteString).expect("Byte String is supported"),
            )
            .expect("secret schema child is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("request extension schema is valid");
    extension::extension_definition(identity.clone(), compatibility, discriminator_value, schema)
        .expect("request extension definition is valid")
}

fn request_configuration(identities: &[(&str, &str, &str)]) -> ClientConfiguration {
    let definitions = identities
        .iter()
        .map(|(vendor, name, discriminator)| {
            request_extension_definition(vendor, name, discriminator)
        })
        .collect();
    let registry =
        extension_registry::client_extension_registry(definitions, extension::defaults())
            .expect("request extension definitions are registered");
    ClientConfiguration::new(registry)
}

fn validated_request_extension(
    registry: &ClientExtensionRegistry,
    vendor: &str,
    name: &str,
    discriminator: &str,
    secret: &[u8],
    criticality: bool,
) -> ClientRequestMessageExtension {
    let identity = extension::extension_identity(vendor, name, "1")
        .expect("request extension identity is valid");
    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                tag(DISCRIMINATOR_TAG),
                Value::text_string(discriminator.to_owned()),
            )
            .expect("discriminator payload item is valid"),
        )
        .expect("discriminator payload fits the Structure");
    payload
        .try_push(
            Item::new(tag(SECRET_TAG), Value::byte_string(secret.to_vec()))
                .expect("secret payload item is valid"),
        )
        .expect("secret payload fits the Structure");
    let value = extension_registry::validate_extension_value(
        registry,
        identity,
        payload,
        &CodecLimits::defaults(),
    )
    .expect("extension payload passes the registered schema");

    extension_registry::client_request_message_extension(value, criticality)
        .expect("request use explicitly supplies criticality")
}

#[test]
fn standalone_values_cannot_be_registered_by_identity_without_the_registry_schema() {
    let identity = extension::extension_identity("vendor.allowed", "extension", "1")
        .expect("the test identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the test range supports this client");
    let path =
        extension::ttlv_path(tag(DISCRIMINATOR_TAG)).expect("the test discriminator path is valid");
    let discriminator =
        extension::discriminator(path, Value::text_string("registered-v1".to_owned()))
            .expect("the test discriminator is valid");
    let schema = extension::structure(
        vec![
            extension::required(
                tag(DISCRIMINATOR_TAG),
                extension::scalar(ItemType::TextString).expect("Text String is supported"),
            )
            .expect("the discriminator rule is valid"),
            extension::required(
                tag(SECRET_TAG),
                extension::scalar(ItemType::ByteString).expect("Byte String is supported"),
            )
            .expect("the secret rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("the registry schema is valid");
    let definition =
        extension::extension_definition(identity.clone(), compatibility, discriminator, schema)
            .expect("the registry definition is valid");
    let registry =
        extension_registry::client_extension_registry(vec![definition], extension::defaults())
            .expect("the test definition is registered");
    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                tag(DISCRIMINATOR_TAG),
                Value::text_string("standalone-only-v1".to_owned()),
            )
            .expect("the standalone discriminator is a valid Item"),
        )
        .expect("the standalone payload is within Structure limits");
    payload
        .try_push(
            Item::new(
                tag(SECRET_TAG),
                Value::byte_string(SECRET_SENTINEL.to_vec()),
            )
            .expect("the standalone secret is a valid Item"),
        )
        .expect("the standalone payload is within Structure limits");

    let error = extension_registry::validate_extension_value(
        &registry,
        identity,
        payload,
        &CodecLimits::defaults(),
    )
    .expect_err("a registry must enforce its own registered discriminator and schema");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(format!("{error}").contains("protocol failure"));
    assert!(!format!("{error}").contains("standalone-only-v1"));
}

fn capture_extensions(bytes: &[u8]) -> Result<Vec<CapturedExtension>, DecodeError> {
    let root = decode_with_limits(bytes, &CodecLimits::defaults())?;
    Ok(root.with_value(|root| {
        let ValueView::Structure(message) = root else {
            return Vec::new();
        };
        message
            .children()
            .iter()
            .filter(|child| child.tag().raw() == BATCH_ITEM_TAG)
            .flat_map(|batch_item| {
                batch_item.with_value(|value| {
                    let ValueView::Structure(batch_item) = value else {
                        return Vec::new();
                    };
                    batch_item
                        .children()
                        .iter()
                        .filter(|child| child.tag().raw() == MESSAGE_EXTENSION_TAG)
                        .filter_map(|extension_item| {
                            extension_item.with_value(|value| {
                                let ValueView::Structure(extension) = value else {
                                    return None;
                                };
                                let vendor = extension
                                    .children()
                                    .iter()
                                    .find(|field| field.tag().raw() == VENDOR_IDENTIFICATION_TAG)?
                                    .with_value(|value| match value {
                                        ValueView::TextString(value) => Some(value.to_owned()),
                                        _ => None,
                                    })?;
                                let criticality = extension
                                    .children()
                                    .iter()
                                    .find(|field| field.tag().raw() == CRITICALITY_INDICATOR_TAG)?
                                    .with_value(|value| match value {
                                        ValueView::Boolean(value) => Some(*value),
                                        _ => None,
                                    })?;
                                let payload_tags = extension
                                    .children()
                                    .iter()
                                    .find(|field| field.tag().raw() == VENDOR_EXTENSION_TAG)?
                                    .with_value(|value| match value {
                                        ValueView::Structure(payload) => Some(
                                            payload
                                                .children()
                                                .iter()
                                                .map(|field| field.tag().raw())
                                                .collect(),
                                        ),
                                        _ => None,
                                    })?;
                                Some(CapturedExtension {
                                    vendor,
                                    criticality,
                                    payload_tags,
                                })
                            })
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect()
    }))
}

struct ObservingTransport {
    inner: Rc<RefCell<ScriptedTransport>>,
    captured: Rc<RefCell<Vec<CapturedExtension>>>,
    owner_observer: ZeroizationObserver,
    owner_live_during_exchange: Rc<Cell<bool>>,
}

struct ClientFixture {
    client: Client,
    transport: Rc<RefCell<ScriptedTransport>>,
    captured: Rc<RefCell<Vec<CapturedExtension>>>,
    owner_live_through_exchange: Rc<Cell<bool>>,
}

impl Transport for ObservingTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.owner_live_during_exchange
            .set(self.owner_observer.result().is_none());
        let captured = capture_extensions(request)
            .expect("the client emits a complete request with valid TTLV framing");
        self.captured.borrow_mut().extend(captured);
        let result = self
            .inner
            .borrow_mut()
            .exchange(request, max_response_bytes);
        self.owner_live_during_exchange
            .set(self.owner_live_during_exchange.get() && self.owner_observer.result().is_none());
        result
    }
}

fn client_for(
    script: ExchangeScript,
    observer: &ZeroizationObserver,
    configuration: ClientConfiguration,
) -> ClientFixture {
    let inner = Rc::new(RefCell::new(ScriptedTransport::new(script)));
    let captured = Rc::new(RefCell::new(Vec::new()));
    let owner_live = Rc::new(Cell::new(false));
    let transport = ObservingTransport {
        inner: Rc::clone(&inner),
        captured: Rc::clone(&captured),
        owner_observer: observer.clone(),
        owner_live_during_exchange: Rc::clone(&owner_live),
    };
    let client = Client::for_test_with_configuration_and_request_observer(
        transport,
        configuration,
        observer.clone(),
    );
    ClientFixture {
        client,
        transport: inner,
        captured,
        owner_live_through_exchange: owner_live,
    }
}

fn response_success() -> Vec<u8> {
    response_bytes((2, 1), &[ResponseItemFixture::success(None)])
}

#[test]
fn repeated_message_extensions_keep_explicit_criticality_and_caller_order() {
    let observer = ZeroizationObserver::new(None);
    let configuration = request_configuration(&[
        ("vendor.alpha", "alpha", "alpha-v1"),
        ("vendor.beta", "beta", "beta-v1"),
    ]);
    let first = validated_request_extension(
        configuration.extension_registry(),
        "vendor.alpha",
        "alpha",
        "alpha-v1",
        SECRET_SENTINEL,
        true,
    );
    let second = validated_request_extension(
        configuration.extension_registry(),
        "vendor.beta",
        "beta",
        "beta-v1",
        SECRET_SENTINEL,
        false,
    );
    let mut fixture = client_for(
        ExchangeScript::Success {
            response: response_success(),
            request_write_chunks: vec![3, 7, 2],
        },
        &observer,
        configuration,
    );
    assert_eq!(
        format!("{first:?}"),
        "ClientRequestMessageExtension([REDACTED])"
    );
    let batch_item = ClientBatchItem::new(ClientRequest::discover_versions())
        .with_extension(first)
        .with_extension(second);

    fixture
        .client
        .execute(ClientBatch::new(batch_item), &CodecLimits::defaults())
        .expect("both validated Message Extensions are encoded through the typed writer");

    assert_eq!(
        *fixture.captured.borrow(),
        [
            CapturedExtension {
                vendor: "vendor.alpha".to_owned(),
                criticality: true,
                payload_tags: vec![DISCRIMINATOR_TAG, SECRET_TAG],
            },
            CapturedExtension {
                vendor: "vendor.beta".to_owned(),
                criticality: false,
                payload_tags: vec![DISCRIMINATOR_TAG, SECRET_TAG],
            },
        ]
    );
    assert!(
        fixture.owner_live_through_exchange.get(),
        "the encoded owner stays alive through exchange"
    );
    assert_eq!(observer.result(), Some(true));
    assert_eq!(fixture.transport.borrow().exchange_count(), 1);
}

#[test]
fn secret_request_owner_lives_through_success_and_zeroizes_before_release() {
    let observer = ZeroizationObserver::new(None);
    let configuration = request_configuration(&[("vendor.secret", "secret", "secret-v1")]);
    let extension = validated_request_extension(
        configuration.extension_registry(),
        "vendor.secret",
        "secret",
        "secret-v1",
        SECRET_SENTINEL,
        true,
    );
    let mut fixture = client_for(
        ExchangeScript::Success {
            response: response_success(),
            request_write_chunks: vec![1, 2, 3, 5],
        },
        &observer,
        configuration,
    );
    let batch_item =
        ClientBatchItem::new(ClientRequest::discover_versions()).with_extension(extension);

    fixture
        .client
        .execute(ClientBatch::new(batch_item), &CodecLimits::defaults())
        .expect("the scripted response succeeds");

    assert!(fixture.owner_live_through_exchange.get());
    assert_eq!(observer.result(), Some(true));
    assert_eq!(fixture.transport.borrow().exchange_count(), 1);
}

#[test]
fn secret_request_lifecycle_reports_delivery_state_without_retry_and_zeroizes_on_errors() {
    let cases = [
        (
            ExchangeScript::FailBeforeWrite,
            RequestDeliveryState::NotSent,
        ),
        (
            ExchangeScript::FailAfterPartialWrite { written_bytes: 7 },
            RequestDeliveryState::PossiblySent,
        ),
        (
            ExchangeScript::FailAfterPartialRead {
                written_bytes: 7,
                response_bytes: b"partial response".to_vec(),
            },
            RequestDeliveryState::ResponseStarted,
        ),
    ];

    for (script, expected_state) in cases {
        let observer = ZeroizationObserver::new(None);
        let configuration = request_configuration(&[("vendor.secret", "secret", "secret-v1")]);
        let extension = validated_request_extension(
            configuration.extension_registry(),
            "vendor.secret",
            "secret",
            "secret-v1",
            SECRET_SENTINEL,
            false,
        );
        let mut fixture = client_for(script, &observer, configuration);
        let batch_item =
            ClientBatchItem::new(ClientRequest::discover_versions()).with_extension(extension);
        let error = fixture
            .client
            .execute(ClientBatch::new(batch_item), &CodecLimits::defaults())
            .expect_err("the scripted transport failure is returned without retry");

        assert_eq!(error.delivery_state(), Some(expected_state));
        assert!(fixture.owner_live_through_exchange.get());
        assert_eq!(observer.result(), Some(true));
        assert_eq!(fixture.transport.borrow().exchange_count(), 1);
    }
}
