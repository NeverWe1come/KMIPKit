//! Fluent Create Key Pair tests derived from KMIP v2.1 §6.1.9,
//! Tables 189–191, and KMIPKIT-0097-FR-001 through FR-004.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use kmipkit_protocol::attribute::typed;
use kmipkit_protocol::{
    Attribute, AttributeDefinition, CreateKeyPairRequest, CryptographicAlgorithm, ProtocolVersion,
    RequestMessage, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions, TimeoutLimit, WireEncoding};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::{client_for, client_for_with_options_observer};
use crate::execute::{
    Client, ClientBatch, ClientBatchOutcome, ClientOperation, ClientRequest,
    decode_request_message_for_test,
};
use crate::execute_test_support::{
    asynchronous_response_bytes, asynchronous_response_bytes_with_version, test_item,
    test_structure,
};
use crate::fluent::{CreateKeyPairBuilder, CryptographicUsageMask, FluentBuildError};

const RSA: u32 = 4;
const DSA: u32 = 5;
const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
const PRIVATE_KEY_ATTRIBUTES: u32 = 0x0042_0127;
const PUBLIC_KEY_ATTRIBUTES: u32 = 0x0042_0128;
const PRIVATE_NAME: &str = "signing-private-key";
const PUBLIC_NAME: &str = "signing-public-key";
const PRIVATE_SENTINEL: &str = "PRIVATE_KEY_PAIR_INPUT_SENTINEL";

fn attribute_tags(group: &Item) -> Vec<u32> {
    group
        .as_structure()
        .expect("attribute group is a Structure")
        .children()
        .iter()
        .map(|attribute| attribute.tag().raw())
        .collect()
}

fn read_integer(item: &Item) -> Option<i32> {
    item.with_value(|value| match value {
        ValueView::Integer(value) => Some(*value),
        _ => None,
    })
}

fn read_enumeration(item: &Item) -> Option<u32> {
    item.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    })
}

fn assert_name(item: &Item, expected: &str) {
    let fields = item.as_structure().expect("Name is a Structure").children();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].tag().raw(), 0x0042_0097);
    assert_eq!(read_enumeration(&fields[0]), Some(1));
    assert_eq!(fields[1].tag().raw(), 0x0042_0098);
    assert!(fields[1].with_value(|value| matches!(
        value,
        ValueView::TextString(value) if value == expected
    )));
}

fn payload_groups(payload: &Structure) -> Vec<&Item> {
    payload.view().children().iter().collect()
}

#[test]
fn key_pair_builder_requires_distinct_explicit_algorithm_and_length_values() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let algorithm_error = client
        .request()
        .create_key_pair()
        .length(2048)
        .private_name(PRIVATE_SENTINEL)
        .build()
        .expect_err("algorithm is required even when length is explicit");
    assert_eq!(
        algorithm_error,
        FluentBuildError::MissingCreateKeyPairAlgorithm
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let length_error = client
        .request()
        .create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(RSA))
        .private_name(PRIVATE_SENTINEL)
        .build()
        .expect_err("algorithm does not select a key length");
    assert_eq!(length_error, FluentBuildError::MissingCreateKeyPairLength);
    assert_ne!(algorithm_error, length_error);

    for error in [algorithm_error, length_error] {
        assert!(!format!("{error:?} {error}").contains(PRIVATE_SENTINEL));
    }
}

#[test]
fn key_pair_builder_places_semantic_attributes_in_their_exact_key_groups() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request: CreateKeyPairRequest = client
        .request()
        .create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(RSA))
        .length(2048)
        .private_usage_mask(CryptographicUsageMask::SIGN)
        .private_name(PRIVATE_NAME)
        .public_usage_mask(CryptographicUsageMask::VERIFY)
        .public_name(PUBLIC_NAME)
        .build()
        .expect("explicit semantic values form a typed Create Key Pair request");
    let payload = request
        .into_ttlv_payload()
        .expect("the existing typed request accepts the grouped attributes");
    let groups = payload_groups(&payload);
    assert_eq!(
        groups
            .iter()
            .map(|group| group.tag().raw())
            .collect::<Vec<_>>(),
        [
            COMMON_ATTRIBUTES,
            PRIVATE_KEY_ATTRIBUTES,
            PUBLIC_KEY_ATTRIBUTES
        ]
    );
    assert_eq!(
        attribute_tags(groups[0]),
        [
            typed::CryptographicAlgorithm::TAG,
            typed::CryptographicLength::TAG
        ]
    );
    let common = groups[0].as_structure().unwrap().children();
    assert_eq!(read_enumeration(&common[0]), Some(RSA));
    assert_eq!(read_integer(&common[1]), Some(2048));

    for (group, mask, name) in [
        (groups[1], CryptographicUsageMask::SIGN.raw(), PRIVATE_NAME),
        (groups[2], CryptographicUsageMask::VERIFY.raw(), PUBLIC_NAME),
    ] {
        assert_eq!(
            attribute_tags(group),
            [typed::CryptographicUsageMask::TAG, typed::Name::TAG]
        );
        let attributes = group.as_structure().unwrap().children();
        assert_eq!(read_integer(&attributes[0]), Some(mask));
        assert_name(&attributes[1], name);
    }
}

#[test]
fn key_pair_builder_does_not_synthesize_key_usage_or_curve_attributes() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(RSA))
        .length(3072)
        .build()
        .expect("usage masks and names remain optional caller choices");
    assert!(!request.has_private_key_attributes());
    assert!(!request.has_public_key_attributes());
    let payload = request.into_ttlv_payload().unwrap();
    let groups = payload_groups(&payload);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].tag().raw(), COMMON_ATTRIBUTES);
    assert_eq!(
        attribute_tags(groups[0]),
        [
            typed::CryptographicAlgorithm::TAG,
            typed::CryptographicLength::TAG
        ]
    );
}

#[test]
fn conflicting_table_191_key_algorithms_are_rejected_before_transport() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(RSA))
        .length(2048)
        .private_attribute(Attribute::<typed::CryptographicAlgorithm>::new(RSA).unwrap())
        .public_attribute(Attribute::<typed::CryptographicAlgorithm>::new(DSA).unwrap())
        .private_name(PRIVATE_SENTINEL)
        .execute(&CodecLimits::defaults())
        .expect_err("Table 191 rejects distinct effective algorithms for the pair");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
    assert!(!format!("{error:?} {error}").contains(PRIVATE_SENTINEL));
}

fn grouped_key_pair(client: &mut Client) -> CreateKeyPairBuilder<'_> {
    client
        .request()
        .create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(RSA))
        .length(2048)
        .common_attribute(Attribute::<typed::Sensitive>::new(true).unwrap())
        .private_usage_mask(CryptographicUsageMask::SIGN)
        .private_name(PRIVATE_NAME)
        .private_attribute(Attribute::<typed::Extractable>::new(false).unwrap())
        .public_usage_mask(CryptographicUsageMask::VERIFY)
        .public_name(PUBLIC_NAME)
        .public_attribute(Attribute::<typed::Extractable>::new(true).unwrap())
}

fn assert_grouped_key_pair(payload: &Structure) {
    let groups = payload_groups(payload);
    assert_eq!(
        groups
            .iter()
            .map(|group| group.tag().raw())
            .collect::<Vec<_>>(),
        [
            COMMON_ATTRIBUTES,
            PRIVATE_KEY_ATTRIBUTES,
            PUBLIC_KEY_ATTRIBUTES
        ]
    );
    assert_eq!(
        attribute_tags(groups[0]),
        [
            typed::CryptographicAlgorithm::TAG,
            typed::CryptographicLength::TAG,
            typed::Sensitive::TAG
        ]
    );
    let common = groups[0].as_structure().unwrap().children();
    assert_eq!(read_enumeration(&common[0]), Some(RSA));
    assert_eq!(read_integer(&common[1]), Some(2048));
    assert!(common[2].with_value(|value| matches!(value, ValueView::Boolean(true))));
    for (group, mask, name, extractable) in [
        (
            groups[1],
            CryptographicUsageMask::SIGN.raw(),
            PRIVATE_NAME,
            false,
        ),
        (
            groups[2],
            CryptographicUsageMask::VERIFY.raw(),
            PUBLIC_NAME,
            true,
        ),
    ] {
        assert_eq!(
            attribute_tags(group),
            [
                typed::CryptographicUsageMask::TAG,
                typed::Name::TAG,
                typed::Extractable::TAG
            ]
        );
        let attributes = group.as_structure().unwrap().children();
        assert_eq!(read_integer(&attributes[0]), Some(mask));
        assert_name(&attributes[1], name);
        assert!(attributes[2].with_value(|value| matches!(
            value, ValueView::Boolean(actual) if *actual == extractable
        )));
    }
}

fn key_pair_success_payload() -> Structure {
    test_structure([
        test_item(
            0x0042_0066,
            Value::text_string("server-private-id".to_owned()),
        ),
        test_item(
            0x0042_006f,
            Value::text_string("server-public-id".to_owned()),
        ),
    ])
}

fn captured_message(bytes: &[u8]) -> RequestMessage {
    let root = decode_with_limits(bytes, &CodecLimits::defaults()).unwrap();
    RequestMessage::try_from_ttlv(root.as_structure().unwrap().try_clone().unwrap())
        .expect("the real client emits a valid Request Message")
}

#[test]
fn key_pair_batch_conversion_preserves_typed_request_and_ordered_attribute_groups() {
    let (mut client, transport, captured) = client_for(ExchangeScript::FailBeforeWrite);
    let item = grouped_key_pair(&mut client).to_batch_item().unwrap();
    assert_eq!(item.operation(), 2);
    assert!(matches!(item.request(), ClientRequest::CreateKeyPair(_)));
    let message =
        decode_request_message_for_test(ClientBatch::new(item), &CodecLimits::defaults()).unwrap();
    let payload = message
        .batch_items()
        .next()
        .unwrap()
        .with_request_payload(|payload| payload.try_clone().unwrap())
        .unwrap();
    assert_grouped_key_pair(&payload);
    assert_eq!(transport.borrow().exchange_count(), 0);
    assert!(captured.borrow().is_none());
}

#[test]
fn key_pair_builder_executes_one_typed_exchange_without_inferring_response_attributes() {
    let response = asynchronous_response_bytes(2, 0, None, None, Some(key_pair_success_payload()));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![4, 5],
    });
    let response = grouped_key_pair(&mut client)
        .execute(&CodecLimits::defaults())
        .unwrap();
    let ClientBatchOutcome::CreateKeyPairCompleted(typed) = response.outcome() else {
        panic!("Success returns the existing typed CreateKeyPair response");
    };
    assert_eq!(typed.result().status().raw(), 0);
    assert_eq!(
        typed.private_key_unique_identifier(),
        Some(&UniqueIdentifier::from("server-private-id"))
    );
    assert_eq!(
        typed.public_key_unique_identifier(),
        Some(&UniqueIdentifier::from("server-public-id"))
    );
    assert_eq!(
        response.outcome().operation(),
        ClientOperation::CreateKeyPair
    );
    assert!(
        response
            .outcome()
            .asynchronous_correlation_value()
            .is_none()
    );
    assert!(response.outcome().response().get_attributes().is_none());
    assert!(response.outcome().response().get_attribute_list().is_none());

    let capture = captured.borrow();
    let message = captured_message(capture.as_ref().unwrap());
    assert_eq!(message.header().protocol_version(), ProtocolVersion::V2_1);
    assert_eq!(message.header().asynchronous_indicator(), None);
    assert_eq!(message.batch_items().len(), 1);
    let item = message.batch_items().next().unwrap();
    assert_eq!(item.operation(), Some(2));
    let payload = item
        .with_request_payload(|payload| payload.try_clone().unwrap())
        .unwrap();
    assert_grouped_key_pair(&payload);
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn key_pair_builder_preserves_failure_without_success_fields_or_retry() {
    let response = asynchronous_response_bytes(2, 1, Some(2), None, None);
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let response = grouped_key_pair(&mut client)
        .execute(&CodecLimits::defaults())
        .unwrap();
    let ClientBatchOutcome::CreateKeyPairCompleted(typed) = response.outcome() else {
        panic!("server Failure remains a completed typed operation result");
    };
    assert_eq!(typed.result().status().raw(), 1);
    assert_eq!(
        typed
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(2)
    );
    assert!(typed.private_key_unique_identifier().is_none());
    assert!(typed.public_key_unique_identifier().is_none());
    assert!(
        response
            .outcome()
            .asynchronous_correlation_value()
            .is_none()
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn key_pair_builder_preserves_pending_and_exact_binary_correlation_without_polling() {
    // Includes NUL and non-UTF-8 bytes so string conversion or truncation cannot pass.
    const CORRELATION: &[u8] = &[0, 0xff, 0x80, 0x01, 0, 0x7f];
    let response =
        asynchronous_response_bytes(2, 2, None, Some(CORRELATION), Some(Structure::new()));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let response = grouped_key_pair(&mut client)
        .with_asynchronous_indicator(1)
        .execute(&CodecLimits::defaults())
        .unwrap();
    let outcome = response.outcome();
    let ClientBatchOutcome::Pending(pending) = outcome else {
        panic!("Pending must remain visible through the existing outcome");
    };
    assert_eq!(pending.operation(), ClientOperation::CreateKeyPair);
    assert_eq!(pending.result().status().raw(), 2);
    assert_eq!(pending.asynchronous_correlation_value(), CORRELATION);
    assert_eq!(outcome.asynchronous_correlation_value(), Some(CORRELATION));
    assert!(outcome.create_key_pair_response().is_none());
    let view = outcome.response();
    let typed = view.create_key_pair().unwrap();
    assert_eq!(typed.result().status().raw(), 2);
    assert!(typed.private_key_unique_identifier().is_none());
    assert!(typed.public_key_unique_identifier().is_none());
    let capture = captured.borrow();
    assert_eq!(
        captured_message(capture.as_ref().unwrap())
            .header()
            .asynchronous_indicator(),
        Some(1)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn key_pair_builder_preserves_exchange_options_and_explicit_prohibited_async_indicator() {
    let response = asynchronous_response_bytes_with_version(
        ProtocolVersion::V2_0,
        2,
        0,
        None,
        None,
        Some(key_pair_success_payload()),
    );
    let observed_options = Rc::new(RefCell::new(Vec::new()));
    let (mut client, transport, captured) = client_for_with_options_observer(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        Rc::clone(&observed_options),
    );
    let options = RequestOptions::default()
        .with_protocol_version(ProtocolVersion::V2_0)
        .with_wire_encoding(WireEncoding::Ttlv)
        .with_connect(TimeoutLimit::Bounded(Duration::from_secs(2)))
        .with_write(TimeoutLimit::Bounded(Duration::from_secs(3)))
        .with_read(TimeoutLimit::Bounded(Duration::from_secs(4)))
        .with_total(TimeoutLimit::Bounded(Duration::from_secs(5)));
    let response = grouped_key_pair(&mut client)
        .with_options(options)
        .with_asynchronous_indicator(3)
        .execute(&CodecLimits::defaults())
        .unwrap();
    assert_eq!(response.outcome().result().status().raw(), 0);
    assert_eq!(&*observed_options.borrow(), &[options]);
    let capture = captured.borrow();
    let message = captured_message(capture.as_ref().unwrap());
    assert_eq!(message.header().protocol_version(), ProtocolVersion::V2_0);
    assert_eq!(message.header().asynchronous_indicator(), Some(3));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn key_pair_builder_missing_required_values_are_not_sent() {
    for missing_algorithm in [true, false] {
        let (mut client, transport, captured) = client_for(ExchangeScript::FailBeforeWrite);
        let builder = client
            .request()
            .create_key_pair()
            .private_name(PRIVATE_SENTINEL);
        let builder = if missing_algorithm {
            builder.length(2048)
        } else {
            builder.algorithm(CryptographicAlgorithm::from_raw(RSA))
        };
        let error = builder.execute(&CodecLimits::defaults()).unwrap_err();
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(transport.borrow().exchange_count(), 0);
        assert!(captured.borrow().is_none());
        assert!(!format!("{error:?} {error}").contains(PRIVATE_SENTINEL));
    }
}

#[test]
fn key_pair_builder_transport_failure_preserves_delivery_state_without_retry() {
    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = grouped_key_pair(&mut client)
        .execute(&CodecLimits::defaults())
        .unwrap_err();
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn key_pair_builder_invalid_async_indicator_is_not_sent() {
    let (mut client, transport, captured) = client_for(ExchangeScript::FailBeforeWrite);
    let error = grouped_key_pair(&mut client)
        .with_asynchronous_indicator(0)
        .execute(&CodecLimits::defaults())
        .expect_err("the typed client rejects unassigned Asynchronous Indicator values");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
    assert!(captured.borrow().is_none());
}
