//! Fake-transport fluent Create tests derived from OASIS KMIP v2.1 §6.1.8,
//! Tables 186–188; these do not claim full official test-case conformance.

use std::cell::RefCell;
use std::rc::Rc;

use kmipkit_protocol::ProtocolVersion;
use kmipkit_protocol::{AttributeSet, CreateRequest, ObjectType, UniqueIdentifier};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};

use crate::WireEncoding;
use crate::asynchronous_execution_test_support::{client_for, client_for_with_options_observer};
use crate::execute::{ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{ClientError, FluentBuildError};

const CREATE: u32 = 0x0000_0001;
const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;

fn success_payload() -> kmipkit_ttlv::Structure {
    test_structure([
        test_item(OBJECT_TYPE, kmipkit_ttlv::Value::enumeration(7)),
        test_item(
            UNIQUE_IDENTIFIER,
            kmipkit_ttlv::Value::text_string("created-object".to_owned()),
        ),
    ])
}

fn request_operation_and_payload_tags(request: &[u8]) -> Option<(u32, Vec<u32>)> {
    let decoded = decode_with_limits(request, &CodecLimits::defaults()).ok()?;
    decoded.with_value(|value| {
        let ValueView::Structure(root) = value else {
            return None;
        };
        let batch_item = root
            .children()
            .iter()
            .find(|item| item.tag().raw() == BATCH_ITEM)?;
        batch_item.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                return None;
            };
            let operation = batch
                .children()
                .iter()
                .find(|item| item.tag().raw() == OPERATION)?
                .with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                })?;
            let payload = batch
                .children()
                .iter()
                .find(|item| item.tag().raw() == REQUEST_PAYLOAD)?;
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    return None;
                };
                Some((
                    operation,
                    payload
                        .children()
                        .iter()
                        .map(|item| item.tag().raw())
                        .collect(),
                ))
            })
        })
    })
}

#[test]
fn fluent_create_requires_explicit_object_type_and_builds_required_empty_attributes() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let builder = client.request().create();

    assert_eq!(
        builder.validate(),
        Err(FluentBuildError::MissingCreateObjectType)
    );
    let request: CreateRequest = builder
        .object_type(ObjectType::from_raw(7))
        .build()
        .expect("an explicit Object Type completes the typed request");

    assert_eq!(request.object_type().raw(), 7);
    assert!(request.attributes().is_empty());
    assert!(request.protection_storage_masks().is_none());
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn fluent_create_builds_typed_batch_item_without_sending() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let item: ClientBatchItem = client
        .request()
        .create()
        .object_type(ObjectType::from_raw(7))
        .attributes(AttributeSet::new())
        .to_batch_item()
        .expect("the builder converts to the existing typed batch item");
    let batch = ClientBatch::new(item);

    assert_eq!(batch.items().len(), 1);
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn fluent_create_executes_once_and_preserves_typed_response_and_optional_masks() {
    let response = asynchronous_response_bytes(CREATE, 0, None, None, Some(success_payload()));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });

    let result = client
        .request()
        .create()
        .object_type(ObjectType::from_raw(7))
        .attributes(AttributeSet::new())
        .protection_storage_masks(kmipkit_ttlv::Structure::new())
        .execute_outcome_with_limits(&CodecLimits::defaults())
        .expect("the Create operation returns its typed success");

    let ClientBatchOutcome::CreateCompleted(create) = result.outcome() else {
        panic!("the typed result is Create");
    };
    assert_eq!(create.object_type(), Some(ObjectType::from_raw(7)));
    assert_eq!(
        create.unique_identifier(),
        Some(&UniqueIdentifier::from("created-object"))
    );
    assert_eq!(
        request_operation_and_payload_tags(
            captured
                .borrow()
                .as_deref()
                .expect("the request was captured")
        ),
        Some((
            CREATE,
            vec![OBJECT_TYPE, ATTRIBUTES, PROTECTION_STORAGE_MASKS]
        ))
    );
    assert_eq!(result.outcome().operation(), ClientOperation::Create);
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn fluent_create_passes_exchange_options_to_the_existing_typed_executor() {
    let response = asynchronous_response_bytes(CREATE, 0, None, None, Some(success_payload()));
    let observed_options = Rc::new(RefCell::new(Vec::new()));
    let (mut client, transport, _) = client_for_with_options_observer(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        Rc::clone(&observed_options),
    );
    let options = RequestOptions::default()
        .with_protocol_version(ProtocolVersion::V2_1)
        .with_wire_encoding(WireEncoding::Ttlv);

    client
        .request()
        .create()
        .object_type(ObjectType::from_raw(7))
        .with_options(options)
        .execute_outcome_with_limits(&CodecLimits::defaults())
        .expect("the typed Create executor accepts the selected options");

    assert_eq!(&*observed_options.borrow(), &[options]);
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn fluent_create_preserves_pending_correlation_without_follow_up() {
    const CORRELATION: &[u8] = b"CREATE_PENDING_CORRELATION_EXACT";
    let response =
        asynchronous_response_bytes(CREATE, 2, None, Some(CORRELATION), Some(test_structure([])));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let result = client
        .request()
        .create()
        .object_type(ObjectType::from_raw(7))
        .with_asynchronous_indicator(1)
        .execute_outcome_with_limits(&CodecLimits::defaults())
        .expect("the caller allowed a Pending Create result");

    let ClientBatchOutcome::Pending(pending) = result.outcome() else {
        panic!("Create Pending remains explicit");
    };
    assert_eq!(pending.operation(), ClientOperation::Create);
    assert_eq!(pending.result().status().raw(), 2);
    assert_eq!(pending.asynchronous_correlation_value(), CORRELATION);
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn fluent_create_missing_object_type_fails_not_sent_without_transport() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);

    let error: ClientError = client
        .request()
        .create()
        .execute_outcome_with_limits(&CodecLimits::defaults())
        .expect_err("Object Type is required by the Create request table");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}
