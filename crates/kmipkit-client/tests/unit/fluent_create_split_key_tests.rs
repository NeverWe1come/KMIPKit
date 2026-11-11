//! Fluent Create Split Key tests derived from KMIP v2.1 §6.1.10,
//! Tables 193–195, and KMIPKIT-0099-FR-001 through FR-007.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use kmipkit_protocol::attribute::typed;
use kmipkit_protocol::{
    Attribute, AttributeDefinition, AttributeSet, ObjectType, ProtocolVersion, RequestMessage,
    SplitKeyMethod, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions, TimeoutLimit, WireEncoding};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::{client_for, client_for_with_options_observer};
use crate::execute::{
    Client, ClientBatchOutcome, ClientOperation, ClientRequest, encode_message_for_test,
};
use crate::execute_test_support::{test_item, test_structure};
use crate::fluent::{CreateSplitKeyBuilder, FluentAttributePosition, FluentBuildError};

const CREATE_SPLIT_KEY_OPERATION: u32 = 0x0000_0028;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("fixture tag fits")
            .try_checked()
            .expect("fixture tag is allocated"),
        value,
    )
    .expect("fixture item construction succeeds")
}

fn valid_builder(client: &mut Client) -> CreateSplitKeyBuilder<'_> {
    client
        .request()
        .create_split_key()
        .object_type(ObjectType::from_raw(2))
        .method(SplitKeyMethod::XOR)
        .parts(3)
        .threshold(2)
}

fn response_bytes(
    status: u32,
    identifiers: &[&str],
    reason: Option<u32>,
    result_message: Option<&str>,
    correlation: Option<&[u8]>,
) -> Vec<u8> {
    let mut batch_fields = vec![
        test_item(OPERATION, Value::enumeration(CREATE_SPLIT_KEY_OPERATION)),
        test_item(RESULT_STATUS, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        batch_fields.push(test_item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(result_message) = result_message {
        batch_fields.push(test_item(
            RESULT_MESSAGE,
            Value::text_string(result_message.to_owned()),
        ));
    }
    if let Some(correlation) = correlation {
        batch_fields.push(test_item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(correlation.to_vec()),
        ));
    }
    if status != 1 {
        let identifiers = test_structure(identifiers.iter().map(|identifier| {
            test_item(
                UNIQUE_IDENTIFIER,
                Value::text_string((*identifier).to_owned()),
            )
        }));
        batch_fields.push(test_item(RESPONSE_PAYLOAD, Value::structure(identifiers)));
    }
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(test_structure(batch_fields))),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("fixture response is valid TTLV")
}

fn captured_message(bytes: &[u8]) -> RequestMessage {
    let decoded = decode_with_limits(bytes, &CodecLimits::defaults())
        .expect("captured request decodes under default limits");
    RequestMessage::try_from_ttlv(
        decoded
            .as_structure()
            .expect("request root is a Structure")
            .try_clone()
            .expect("captured request structure is within the model depth limit"),
    )
    .expect("the fluent builder emits a typed Request Message")
}

#[test]
fn split_key_builder_requires_all_four_explicit_semantic_fields() {
    let cases = [
        (0, FluentBuildError::MissingCreateSplitKeyObjectType),
        (1, FluentBuildError::MissingCreateSplitKeyMethod),
        (2, FluentBuildError::MissingCreateSplitKeyParts),
        (3, FluentBuildError::MissingCreateSplitKeyThreshold),
    ];
    for (missing, expected) in cases {
        let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
        let builder = client.request().create_split_key();
        let builder = match missing {
            0 => builder.method(SplitKeyMethod::XOR).parts(3).threshold(2),
            1 => builder
                .object_type(ObjectType::from_raw(2))
                .parts(3)
                .threshold(2),
            2 => builder
                .object_type(ObjectType::from_raw(2))
                .method(SplitKeyMethod::XOR)
                .threshold(2),
            _ => builder
                .object_type(ObjectType::from_raw(2))
                .method(SplitKeyMethod::XOR)
                .parts(3),
        };
        assert_eq!(builder.validate(), Err(expected));
        assert_eq!(builder.build_request().unwrap_err(), expected);
        let error = builder.execute(&CodecLimits::defaults()).unwrap_err();
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(transport.borrow().exchange_count(), 0);
    }
}

#[test]
fn split_key_builder_preserves_explicit_and_optional_fields_and_empty_attributes() {
    const PRIME_FIELD_SIZE: &[u8] = &[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
    let mut masks = Structure::new();
    masks
        .try_push(item(0x0042_0159, Value::integer(3)))
        .expect("protection mask field fits the Structure depth limit");
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .create_split_key()
        .object_type(ObjectType::from_raw(0xF000_0002))
        .method(SplitKeyMethod::from_raw(0xF000_0003))
        .parts(3)
        .threshold(2)
        .unique_identifier(UniqueIdentifier::from("input-key"))
        .prime_field_size(PRIME_FIELD_SIZE.to_vec())
        .protection_storage_masks(masks)
        .build()
        .expect("explicit future values and optional fields are preserved");

    assert_eq!(request.object_type().raw(), 0xF000_0002);
    assert_eq!(request.split_key_method().raw(), 0xF000_0003);
    assert_eq!(request.split_key_parts(), 3);
    assert_eq!(request.split_key_threshold(), 2);
    assert_eq!(
        request.unique_identifier(),
        Some(&UniqueIdentifier::from("input-key"))
    );
    assert_eq!(request.prime_field_size(), Some(PRIME_FIELD_SIZE));
    assert!(request.protection_storage_masks().is_some());
    assert!(request.attributes().is_empty());
    let payload = request
        .into_ttlv_payload()
        .expect("explicit fields form a valid typed request payload");
    assert_eq!(
        payload
            .view()
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [
            0x0042_0057,
            0x0042_0094,
            0x0042_008B,
            0x0042_008C,
            0x0042_008A,
            0x0042_0062,
            0x0042_0125,
            0x0042_015F,
        ]
    );
    assert!(
        payload.view().children()[6]
            .as_structure()
            .unwrap()
            .children()
            .is_empty()
    );
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn prime_field_method_requires_explicit_size_before_transport() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let builder = client
        .request()
        .create_split_key()
        .object_type(ObjectType::from_raw(2))
        .method(SplitKeyMethod::POLYNOMIAL_SHARING_PRIME_FIELD)
        .parts(3)
        .threshold(2);
    assert_eq!(builder.validate(), Err(FluentBuildError::InvalidRequest));
    assert_eq!(
        builder.build_request().unwrap_err(),
        FluentBuildError::InvalidRequest
    );
    let error = builder.execute(&CodecLimits::defaults()).unwrap_err();
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn split_key_builder_rejects_duplicate_singleton_attributes_before_transport() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let builder = valid_builder(&mut client)
        .attribute(Attribute::<typed::Sensitive>::new(true).unwrap())
        .attribute(Attribute::<typed::Sensitive>::new(false).unwrap());

    assert_eq!(builder.validate(), Err(FluentBuildError::InvalidRequest));
    assert_eq!(
        builder.build_request().unwrap_err(),
        FluentBuildError::InvalidRequest
    );
    let error = builder.execute(&CodecLimits::defaults()).unwrap_err();
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn split_key_builder_preserves_typed_prebuilt_vendor_and_positioned_attributes() {
    let mut prebuilt = AttributeSet::new();
    prebuilt
        .try_push_typed(Attribute::<typed::Sensitive>::new(true).unwrap())
        .unwrap();
    prebuilt
        .try_push_typed(Attribute::<typed::Extractable>::new(false).unwrap())
        .unwrap();
    let explicit_value = item(
        typed::ContactInformation::TAG,
        Value::text_string("opaque-value".to_owned()),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = valid_builder(&mut client)
        .attribute(Attribute::<typed::Description>::new("typed value".to_owned()).unwrap())
        .attributes(prebuilt)
        .attribute_item(explicit_value)
        .vendor_attribute(
            "Vendor_Name.1",
            "region",
            &item(0x0042_005C, Value::text_string("eu".to_owned())),
        )
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 0 },
            item(typed::AlwaysSensitive::TAG, Value::boolean(true)),
        )
        .build()
        .expect("typed, generic, vendor, and positioned attributes build together");
    let items = request.attributes().as_items();
    assert_eq!(
        items
            .iter()
            .map(|attribute| attribute.tag().raw())
            .collect::<Vec<_>>(),
        [
            typed::AlwaysSensitive::TAG,
            typed::Description::TAG,
            typed::Sensitive::TAG,
            typed::Extractable::TAG,
            typed::ContactInformation::TAG,
            typed::VendorAttribute::TAG,
        ]
    );
    let vendor = items[5]
        .as_structure()
        .expect("Vendor Attribute is a Structure");
    assert_eq!(vendor.children().len(), 3);
    assert_eq!(vendor.children()[0].tag().raw(), 0x0042_009D);
    assert_eq!(vendor.children()[1].tag().raw(), 0x0042_000A);
    assert_eq!(vendor.children()[2].tag().raw(), 0x0042_000B);
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn split_key_builder_rejects_malformed_attributes_and_duplicate_positions_but_preserves_extensions()
{
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let invalid_vendor = Attribute::<typed::VendorAttribute>::new(Structure::new())
        .expect("a Vendor Attribute type can hold an empty Structure before set validation");
    assert_eq!(
        valid_builder(&mut client)
            .attribute(invalid_vendor)
            .validate(),
        Err(FluentBuildError::InvalidAttributeValue)
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let duplicate_positions = valid_builder(&mut client)
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 0 },
            item(typed::Sensitive::TAG, Value::boolean(true)),
        )
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 0 },
            item(typed::Extractable::TAG, Value::boolean(false)),
        );
    assert_eq!(
        duplicate_positions.validate(),
        Err(FluentBuildError::DuplicateAttributePosition)
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_eq!(
        valid_builder(&mut client)
            .attribute_at(
                FluentAttributePosition::ObjectAttributes { index: 1 },
                item(typed::Sensitive::TAG, Value::boolean(true)),
            )
            .validate(),
        Err(FluentBuildError::AttributePositionOutOfBounds)
    );

    for tag in [0x0054_0000, 0x0054_8000, 0x0054_FFFF] {
        let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
        let extension = item(tag, Value::boolean(true));
        let builder = match tag {
            0x0054_0000 => valid_builder(&mut client).attribute_item(extension),
            0x0054_8000 => {
                let attributes = AttributeSet::try_new([extension]).unwrap();
                valid_builder(&mut client).attributes(attributes)
            }
            _ => valid_builder(&mut client).attribute_at(
                FluentAttributePosition::ObjectAttributes { index: 0 },
                extension,
            ),
        };
        assert_eq!(builder.validate(), Ok(()));
        let request = builder
            .build_request()
            .expect("explicit extension Items remain raw Object Attribute entries");
        assert_eq!(request.attributes().as_items()[0].tag().raw(), tag);
        assert!(
            request.attributes().as_items()[0]
                .with_value(|value| matches!(value, ValueView::Boolean(true)))
        );
        assert_eq!(transport.borrow().exchange_count(), 0);
    }
}

#[test]
fn split_key_builder_validate_is_non_consuming_and_batch_conversion_is_typed() {
    let (mut client, transport, captured) = client_for(ExchangeScript::FailBeforeWrite);
    let builder =
        valid_builder(&mut client).attribute(Attribute::<typed::Sensitive>::new(true).unwrap());
    assert_eq!(builder.validate(), Ok(()));
    assert_eq!(builder.validate(), Ok(()));
    let request = builder.build_request().unwrap();
    assert_eq!(request.attributes().len(), 1);

    let item = builder.to_batch_item().unwrap();
    assert!(matches!(item.request(), ClientRequest::CreateSplitKey(_)));
    assert_eq!(transport.borrow().exchange_count(), 0);
    assert!(captured.borrow().is_none());
}

#[test]
fn split_key_builder_executes_one_typed_exchange_and_preserves_all_response_ids() {
    let response = response_bytes(
        0,
        &["split-part-1", "split-part-2", "split-part-3"],
        None,
        None,
        None,
    );
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![4, 5],
    });
    let response = valid_builder(&mut client)
        .execute(&CodecLimits::defaults())
        .expect("the existing typed client executes one Create Split Key request");
    let ClientBatchOutcome::CreateSplitKeyCompleted(typed) = response.outcome() else {
        panic!("Success is preserved as a typed Create Split Key result");
    };
    assert_eq!(typed.result().status().raw(), 0);
    assert_eq!(
        typed.unique_identifiers(),
        [
            UniqueIdentifier::from("split-part-1"),
            UniqueIdentifier::from("split-part-2"),
            UniqueIdentifier::from("split-part-3"),
        ]
    );
    assert_eq!(
        response.outcome().operation(),
        ClientOperation::CreateSplitKey
    );
    assert!(
        response
            .outcome()
            .asynchronous_correlation_value()
            .is_none()
    );
    let captured = captured.borrow();
    let message = captured_message(captured.as_ref().expect("request bytes are captured"));
    assert_eq!(message.header().protocol_version(), ProtocolVersion::V2_1);
    assert_eq!(message.batch_items().len(), 1);
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn split_key_builder_preserves_failure_and_pending_correlation_without_follow_up() {
    const CORRELATION: &[u8] = &[0x00, 0xff, 0x80, 0x01, 0x00, 0x7f];

    let failure = response_bytes(1, &[], Some(2), Some("policy failure"), None);
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: failure,
        request_write_chunks: Vec::new(),
    });
    let response = valid_builder(&mut client)
        .execute(&CodecLimits::defaults())
        .expect("server Failure remains a typed operation result");
    let ClientBatchOutcome::CreateSplitKeyCompleted(failed) = response.outcome() else {
        panic!("Failure is represented by the existing typed result");
    };
    assert_eq!(failed.result().status().raw(), 1);
    assert_eq!(
        failed
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(2)
    );
    assert_eq!(failed.unique_identifiers(), []);
    assert_eq!(transport.borrow().exchange_count(), 1);

    let pending = response_bytes(2, &["pending-part"], None, None, Some(CORRELATION));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response: pending,
        request_write_chunks: Vec::new(),
    });
    let response = valid_builder(&mut client)
        .with_asynchronous_indicator(1)
        .execute(&CodecLimits::defaults())
        .expect("Pending remains visible without an implicit Poll");
    let ClientBatchOutcome::Pending(pending) = response.outcome() else {
        panic!("Pending is represented by the existing Pending outcome");
    };
    assert_eq!(pending.operation(), ClientOperation::CreateSplitKey);
    assert_eq!(pending.asynchronous_correlation_value(), CORRELATION);
    assert_eq!(
        pending
            .response()
            .create_split_key()
            .expect("Pending retains its typed operation response")
            .unique_identifiers(),
        [UniqueIdentifier::from("pending-part")]
    );
    let message = captured_message(captured.borrow().as_ref().unwrap());
    assert_eq!(message.header().asynchronous_indicator(), Some(1));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn split_key_builder_preserves_options_and_transport_delivery_state() {
    let response = response_bytes(0, &["one-part"], None, None, None);
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
        .with_wire_encoding(WireEncoding::Ttlv)
        .with_connect(TimeoutLimit::Bounded(Duration::from_secs(2)))
        .with_write(TimeoutLimit::Bounded(Duration::from_secs(3)))
        .with_read(TimeoutLimit::Bounded(Duration::from_secs(4)))
        .with_total(TimeoutLimit::Bounded(Duration::from_secs(5)));
    let response = valid_builder(&mut client)
        .with_options(options)
        .execute(&CodecLimits::defaults())
        .expect("explicit exchange options are forwarded to the client");
    assert_eq!(response.outcome().result().status().raw(), 0);
    assert_eq!(&*observed_options.borrow(), &[options]);
    assert_eq!(transport.borrow().exchange_count(), 1);

    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = valid_builder(&mut client)
        .execute(&CodecLimits::defaults())
        .unwrap_err();
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}
