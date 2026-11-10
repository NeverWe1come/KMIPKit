use kmipkit_protocol::attribute::typed;
use kmipkit_protocol::{Attribute, AttributeDefinition, CreateRequest, UniqueIdentifier};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatchOutcome, ClientOperation, encode_message_for_test};
use crate::execute_test_support::{test_item, test_structure};
use crate::fluent::{CreateSymmetricKeyBuilder, CryptographicUsageMask, FluentBuildError};

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("test tag fits")
            .try_checked()
            .expect("test tag is allocated"),
        value,
    )
    .expect("test item construction succeeds")
}

fn failed_request_client() -> crate::execute::Client {
    client_for(ExchangeScript::FailBeforeWrite).0
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

fn response_bytes(operation: u32, status: u32, correlation: Option<&[u8]>) -> Vec<u8> {
    let mut item_fields = vec![
        test_item(0x0042_005C, Value::enumeration(operation)),
        test_item(0x0042_007F, Value::enumeration(status)),
    ];
    if let Some(correlation) = correlation {
        item_fields.push(test_item(
            0x0042_0006,
            Value::byte_string(correlation.to_vec()),
        ));
        item_fields.push(test_item(0x0042_007C, Value::structure(Structure::new())));
    } else {
        let mut payload = Structure::new();
        payload
            .try_push(test_item(0x0042_0057, Value::enumeration(2)))
            .expect("response Object Type fits");
        payload
            .try_push(test_item(
                0x0042_0094,
                Value::text_string("key-42".to_owned()),
            ))
            .expect("response identifier fits");
        item_fields.push(test_item(0x0042_007C, Value::structure(payload)));
    }
    let mut message = vec![test_item(
        0x0042_007A,
        Value::structure(test_structure([
            test_item(
                0x0042_0069,
                Value::structure(test_structure([
                    test_item(0x0042_006A, Value::integer(2)),
                    test_item(0x0042_006B, Value::integer(1)),
                ])),
            ),
            test_item(0x0042_0092, Value::date_time(1)),
            test_item(0x0042_000D, Value::integer(1)),
        ])),
    )];
    message.push(test_item(
        0x0042_000F,
        Value::structure(test_structure(item_fields)),
    ));
    encode_message_for_test(test_structure(message), &CodecLimits::defaults())
        .expect("fixture response is valid TTLV")
}

#[test]
fn symmetric_key_builder_requires_algorithm_length_and_usage_mask() {
    let mut client = failed_request_client();
    let error = client
        .request()
        .create_symmetric_key()
        .build()
        .err()
        .expect("all security-relevant key properties are required");
    assert_eq!(error, FluentBuildError::MissingCryptographicAlgorithm);

    let mut client = failed_request_client();
    let error = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .build()
        .err()
        .expect("a preset does not silently choose key usage");
    assert_eq!(error, FluentBuildError::MissingCryptographicUsageMask);
}

#[test]
fn aes_256_builder_emits_typed_attributes_and_interpreted_text_name() {
    let mut client = failed_request_client();
    let request = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("payments-key")
        .build()
        .expect("the required typed values are present");

    let attributes = request.attributes().as_items();
    assert_eq!(attributes.len(), 4);
    assert_eq!(
        attributes[0].tag().raw(),
        typed::CryptographicAlgorithm::TAG
    );
    assert_eq!(attributes[0].item_type(), ItemType::Enumeration);
    assert_eq!(read_enumeration(&attributes[0]), Some(3));
    assert_eq!(attributes[1].tag().raw(), typed::CryptographicLength::TAG);
    assert_eq!(read_integer(&attributes[1]), Some(256));
    assert_eq!(
        attributes[2].tag().raw(),
        typed::CryptographicUsageMask::TAG
    );
    assert_eq!(read_integer(&attributes[2]), Some(12));
    assert_eq!(attributes[3].tag().raw(), typed::Name::TAG);
    let name = attributes[3]
        .as_structure()
        .expect("Name is a Structure")
        .children()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(name.len(), 2);
    assert_eq!(name[0].tag().raw(), 0x0042_0097);
    assert_eq!(read_enumeration(name[0]), Some(1));
    assert_eq!(name[1].tag().raw(), 0x0042_0098);
    assert!(name[1].with_value(|value| matches!(
        value,
        ValueView::TextString(name) if name == "payments-key"
    )));
}

#[test]
fn explicit_unknown_attribute_is_preserved_but_extension_tags_are_rejected() {
    let unknown = item(0x0054_0ABC, Value::integer(7));
    let mut client = failed_request_client();
    let builder = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .attribute_item(unknown);
    assert_eq!(
        builder
            .build()
            .err()
            .expect("message extension tags are not attributes"),
        FluentBuildError::MessageExtensionTagNotObjectAttribute
    );
}

#[test]
fn catalogued_attribute_item_rejects_a_wrong_ttlv_type() {
    let wrong_type = item(typed::CryptographicAlgorithm::TAG, Value::integer(3));
    let mut client = failed_request_client();
    let builder = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .attribute_item(wrong_type);
    assert_eq!(
        builder
            .build()
            .err()
            .expect("catalogued tags keep their assigned TTLV type"),
        FluentBuildError::InvalidAttributeValue
    );
}

#[test]
fn explicit_generic_attribute_and_vendor_attribute_are_retained() {
    let unknown = item(0x0042_005C, Value::text_string("expert-value".to_owned()));
    let catalogued = Attribute::<typed::Description>::new("managed by fluent".to_owned())
        .expect("catalogued Text String attribute");
    let mut client = failed_request_client();
    let request = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .attribute(catalogued)
        .attribute_item(unknown)
        .vendor_attribute(
            "Vendor_Name.1",
            "region",
            &item(0x0042_005C, Value::text_string("eu".to_owned())),
        )
        .build()
        .expect("valid expert attributes are preserved");

    let attributes = request.attributes().as_items();
    assert_eq!(attributes.len(), 6);
    assert_eq!(attributes[3].tag().raw(), typed::Description::TAG);
    assert_eq!(attributes[4].tag().raw(), 0x0042_005C);
    assert_eq!(attributes[5].tag().raw(), typed::VendorAttribute::TAG);
    let vendor_fields = attributes[5]
        .as_structure()
        .expect("Vendor Attribute is a Structure")
        .children()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(vendor_fields.len(), 3);
    assert_eq!(vendor_fields[0].tag().raw(), 0x0042_009D);
    assert_eq!(vendor_fields[1].tag().raw(), 0x0042_000A);
    assert_eq!(vendor_fields[2].tag().raw(), 0x0042_000B);
    assert!(vendor_fields[2].with_value(|value| matches!(
        value,
        ValueView::TextString(value) if value == "eu"
    )));
}

#[test]
fn get_and_destroy_builders_require_a_typed_unique_identifier() {
    let mut client = failed_request_client();
    let get = client
        .request()
        .get(UniqueIdentifier::from("key-42"))
        .build();
    assert_eq!(
        get.unique_identifier(),
        Some(&UniqueIdentifier::from("key-42"))
    );

    let mut client = failed_request_client();
    let destroy = client
        .request()
        .destroy(UniqueIdentifier::from("key-42"))
        .build();
    assert_eq!(
        destroy.unique_identifier(),
        Some(&UniqueIdentifier::from("key-42"))
    );
}

#[test]
fn builder_request_is_the_existing_typed_create_request() {
    let mut client = failed_request_client();
    let request: CreateRequest = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .build()
        .expect("the fluent API returns the existing request type");
    assert_eq!(request.object_type().raw(), 2);
}

#[test]
fn detached_create_builder_can_build_reusable_typed_requests_without_a_client() {
    let builder = CreateSymmetricKeyBuilder::detached()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .name("detached-key");

    let first = builder
        .build_request()
        .expect("detached builders construct a typed request");
    let second = builder
        .build_request()
        .expect("building a batch item does not consume the builder state");

    assert_eq!(first.attributes().len(), second.attributes().len());
}

#[test]
fn execute_uses_typed_create_and_returns_the_existing_create_outcome() {
    let response = response_bytes(1, 0, None);
    let (mut client, _, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("payments-key")
        .execute(&CodecLimits::defaults())
        .expect("the existing typed Create route completes the exchange");

    assert_eq!(response.outcome().operation(), ClientOperation::Create);
    match response.outcome() {
        ClientBatchOutcome::CreateCompleted(create) => assert!(matches!(
            create.unique_identifier(),
            Some(UniqueIdentifier::TextString(identifier)) if identifier == "key-42"
        )),
        outcome => panic!("expected typed Create response, got {outcome:?}"),
    }
    assert!(
        crate::asynchronous_execution_test_support::request_contains(
            &captured_request,
            b"payments-key"
        )
    );
}

#[test]
fn execute_exposes_pending_correlation_without_polling() {
    let response = response_bytes(1, 2, Some(b"pending-correlation"));
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .with_asynchronous_indicator(1)
        .execute(&CodecLimits::defaults())
        .expect("Pending remains an explicit typed outcome");

    assert_eq!(response.outcome().operation(), ClientOperation::Create);
    match response.outcome() {
        ClientBatchOutcome::Pending(pending) => {
            assert_eq!(
                pending.asynchronous_correlation_value(),
                b"pending-correlation"
            );
        }
        outcome => panic!("expected Pending, got {outcome:?}"),
    }
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn invalid_fluent_create_execution_keeps_not_sent_delivery_state() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .create_symmetric_key()
        .execute(&CodecLimits::defaults())
        .expect_err("required fluent fields are validated before transport");

    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn every_key_preset_builds_a_typed_request_and_batch_item() {
    for (builder, expected_algorithm, expected_length) in [
        (
            CreateSymmetricKeyBuilder::detached().aes_128(),
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::AES,
            128,
        ),
        (
            CreateSymmetricKeyBuilder::detached().aes_192(),
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::AES,
            192,
        ),
        (
            CreateSymmetricKeyBuilder::detached().hmac_sha256_256(),
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::HMAC_SHA256,
            256,
        ),
    ] {
        let request = builder
            .usage_mask(CryptographicUsageMask::MAC_GENERATE)
            .build_request()
            .expect("presets fill only algorithm and length");
        let attributes = request.attributes().as_items();
        assert_eq!(read_enumeration(&attributes[0]), Some(expected_algorithm));
        assert_eq!(read_integer(&attributes[1]), Some(expected_length));
    }

    let item = CreateSymmetricKeyBuilder::detached()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .to_batch_item()
        .expect("a valid fluent request converts to a batch item");
    assert_eq!(item.operation(), 1);

    let mut client = failed_request_client();
    assert_eq!(
        client
            .request()
            .get(UniqueIdentifier::from("key-42"))
            .to_batch_item()
            .operation(),
        10
    );
    let mut client = failed_request_client();
    assert_eq!(
        client
            .request()
            .destroy(UniqueIdentifier::from("key-42"))
            .to_batch_item()
            .operation(),
        20
    );
}

#[test]
fn create_get_and_destroy_execute_with_exchange_options() {
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: response_bytes(1, 0, None),
        request_write_chunks: Vec::new(),
    });
    let create = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .with_options(RequestOptions::default())
        .execute(&CodecLimits::defaults())
        .expect("Create accepts exchange-local options");
    assert_eq!(create.outcome().operation(), ClientOperation::Create);

    let mut client = failed_request_client();
    let get = client
        .request()
        .get(UniqueIdentifier::from("key-42"))
        .with_options(RequestOptions::default())
        .with_asynchronous_indicator(0)
        .execute(&CodecLimits::defaults());
    assert_eq!(
        get.expect_err("Get reaches the scripted transport")
            .delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );

    let mut client = failed_request_client();
    let destroy = client
        .request()
        .destroy(UniqueIdentifier::from("key-42"))
        .with_options(RequestOptions::default())
        .with_asynchronous_indicator(1)
        .execute(&CodecLimits::defaults());
    assert_eq!(
        destroy
            .expect_err("Destroy reaches the scripted transport")
            .delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
}
