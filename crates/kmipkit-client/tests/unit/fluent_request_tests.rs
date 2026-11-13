use kmipkit_protocol::attribute::typed;
use kmipkit_protocol::{
    AdjustmentType, Attribute, AttributeDefinition, AttributeReference, CheckRequest,
    CreateRequest, CurrentAttribute, DiscoverVersionsRequest, GetConstraintsRequest,
    GetUsageAllocationRequest, NewAttribute, PingRequest, ProtocolVersion, QueryFunction,
    QueryRequest, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatchOutcome, ClientOperation, encode_message_for_test};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::fluent::{
    AttributeAdjustmentValue, CreateSymmetricKeyBuilder, CryptographicUsageMask,
    FluentAttributePosition, FluentBuildError,
};

const PING_OPERATION: u32 = 0x0000_003b;
const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001e;
const QUERY_OPERATION: u32 = 0x0000_0018;
const CHECK_OPERATION: u32 = 0x0000_0009;
const GET_CONSTRAINTS_OPERATION: u32 = 0x0000_0038;
const GET_USAGE_ALLOCATION_OPERATION: u32 = 0x0000_0011;

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

fn attribute_success_response_bytes(operation: u32) -> Vec<u8> {
    let mut payload = Structure::new();
    payload
        .try_push(test_item(
            0x0042_0094,
            Value::text_string("key-42".to_owned()),
        ))
        .expect("attribute operation response identifier fits");
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
        Value::structure(test_structure([
            test_item(0x0042_005C, Value::enumeration(operation)),
            test_item(0x0042_007F, Value::enumeration(0)),
            test_item(0x0042_007C, Value::structure(payload)),
        ])),
    ));
    encode_message_for_test(test_structure(message), &CodecLimits::defaults())
        .expect("attribute operation fixture response is valid TTLV")
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

    let error = CreateSymmetricKeyBuilder::detached()
        .algorithm(kmipkit_protocol::CryptographicAlgorithm::from_raw(3))
        .build()
        .err()
        .expect("algorithm alone does not select a key length or usage policy");
    assert_eq!(error, FluentBuildError::MissingCryptographicLength);
}

#[test]
fn fluent_build_errors_have_stable_user_facing_messages() {
    let cases = [
        (
            FluentBuildError::MissingCryptographicAlgorithm,
            "Create Symmetric Key requires Cryptographic Algorithm",
        ),
        (
            FluentBuildError::MissingCryptographicLength,
            "Create Symmetric Key requires Cryptographic Length",
        ),
        (
            FluentBuildError::MissingCryptographicUsageMask,
            "Create Symmetric Key requires Cryptographic Usage Mask",
        ),
        (
            FluentBuildError::InvalidAttributeValue,
            "fluent Object Attribute value is invalid",
        ),
        (
            FluentBuildError::InvalidProtocolVersions,
            "fluent Discover Versions offers are empty or unsupported",
        ),
        (
            FluentBuildError::MissingAddAttributeValue,
            "Add Attribute requires a New Attribute",
        ),
        (
            FluentBuildError::MissingModifyAttributeValue,
            "Modify Attribute requires a New Attribute",
        ),
        (
            FluentBuildError::MissingSetAttributeValue,
            "Set Attribute requires a New Attribute",
        ),
        (
            FluentBuildError::MissingAdjustAttributeReference,
            "Adjust Attribute requires an Attribute Reference",
        ),
        (
            FluentBuildError::MissingAdjustAttributeType,
            "Adjust Attribute requires an Adjustment Type",
        ),
        (
            FluentBuildError::MessageExtensionTagNotObjectAttribute,
            "KMIP message extension tags are not Object Attributes",
        ),
        (
            FluentBuildError::DuplicateAttributePosition,
            "fluent Object Attribute positions must be unique",
        ),
        (
            FluentBuildError::AttributePositionOutOfBounds,
            "fluent Object Attribute position is outside the final attribute list",
        ),
        (
            FluentBuildError::InvalidRequest,
            "fluent request failed typed request validation",
        ),
        (
            FluentBuildError::InvalidTtlvValue,
            "fluent TTLV value could not be represented",
        ),
        (
            FluentBuildError::MissingClientForExecution,
            "detached fluent builder has no client for execution",
        ),
    ];
    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
    }
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
fn explicit_extension_tag_attribute_item_is_preserved_without_reinterpretation() {
    let unknown = item(0x0054_0ABC, Value::integer(7));
    let mut client = failed_request_client();
    let request = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .attribute_item(unknown)
        .build_request()
        .expect("an explicitly typed unknown tag is retained as a raw Item");

    let extension = request
        .attributes()
        .as_items()
        .iter()
        .find(|attribute| attribute.tag().raw() == 0x0054_0ABC)
        .expect("the extension tag remains present");
    assert!(extension.with_value(|value| matches!(value, ValueView::Integer(7))));
}

#[test]
fn vendor_attribute_copies_every_supported_ttlv_value_kind() {
    let mut nested = Structure::new();
    nested
        .try_push(item(0x0042_005C, Value::text_string("nested".to_owned())))
        .expect("nested structure item is valid");
    let values = [
        Value::structure(nested),
        Value::integer(-7),
        Value::long_integer(-9),
        Value::big_integer(vec![0x01, 0x02]),
        Value::enumeration(3),
        Value::boolean(true),
        Value::text_string("text".to_owned()),
        Value::byte_string(vec![0x00, 0xFF]),
        Value::date_time(1),
        Value::interval(2),
        Value::date_time_extended(3),
    ];
    let mut builder = CreateSymmetricKeyBuilder::detached()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT);
    for value in values {
        let source = item(0x0042_005C, value);
        builder = builder.vendor_attribute("Vendor_Name.1", "sample", &source);
    }
    let request = builder
        .build_request()
        .expect("vendor value cloning preserves all supported TTLV value kinds");
    assert_eq!(request.attributes().len(), 14);
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
    let explicit = item(
        typed::ContactInformation::TAG,
        Value::text_string("expert-value".to_owned()),
    );
    let catalogued = Attribute::<typed::Description>::new("managed by fluent".to_owned())
        .expect("catalogued Text String attribute");
    let mut client = failed_request_client();
    let request = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .attribute(catalogued)
        .attribute_item(explicit)
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
    assert_eq!(attributes[4].tag().raw(), typed::ContactInformation::TAG);
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
fn expert_attribute_positions_apply_to_the_final_ordered_attribute_sequence() {
    let request = CreateSymmetricKeyBuilder::detached()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .name("ordered-key")
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 0 },
            item(
                typed::ContactInformation::TAG,
                Value::text_string("first".to_owned()),
            ),
        )
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 2 },
            item(
                typed::Description::TAG,
                Value::text_string("middle".to_owned()),
            ),
        )
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 6 },
            item(typed::Comment::TAG, Value::text_string("last".to_owned())),
        )
        .build_request()
        .expect("explicit positions fit the final attribute sequence");

    let attributes = request.attributes().as_items();
    assert_eq!(attributes.len(), 7);
    assert_eq!(attributes[0].tag().raw(), typed::ContactInformation::TAG);
    assert_eq!(
        attributes[1].tag().raw(),
        typed::CryptographicAlgorithm::TAG
    );
    assert_eq!(attributes[2].tag().raw(), typed::Description::TAG);
    assert_eq!(attributes[3].tag().raw(), typed::CryptographicLength::TAG);
    assert_eq!(
        attributes[4].tag().raw(),
        typed::CryptographicUsageMask::TAG
    );
    assert_eq!(attributes[5].tag().raw(), typed::Name::TAG);
    assert_eq!(attributes[6].tag().raw(), typed::Comment::TAG);
    for (position, expected) in [(0, "first"), (2, "middle"), (6, "last")] {
        assert!(attributes[position].with_value(|value| matches!(
            value,
            ValueView::TextString(actual) if actual == expected
        )));
    }
}

#[test]
fn expert_attribute_positions_reject_duplicate_and_out_of_range_indices() {
    let duplicate = CreateSymmetricKeyBuilder::detached()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 0 },
            item(
                typed::ContactInformation::TAG,
                Value::text_string("one".to_owned()),
            ),
        )
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 0 },
            item(
                typed::ContactInformation::TAG,
                Value::text_string("two".to_owned()),
            ),
        )
        .build_request();
    assert_eq!(
        duplicate.err(),
        Some(FluentBuildError::DuplicateAttributePosition)
    );

    let out_of_range = CreateSymmetricKeyBuilder::detached()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 4 },
            item(
                typed::ContactInformation::TAG,
                Value::text_string("too late".to_owned()),
            ),
        )
        .build_request();
    assert_eq!(
        out_of_range.err(),
        Some(FluentBuildError::AttributePositionOutOfBounds)
    );
}

#[test]
fn expert_attribute_positions_keep_type_validation_and_extension_tags_raw() {
    let wrong_type = CreateSymmetricKeyBuilder::detached()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 0 },
            item(typed::CryptographicAlgorithm::TAG, Value::integer(3)),
        )
        .build_request();
    assert_eq!(
        wrong_type.err(),
        Some(FluentBuildError::InvalidAttributeValue)
    );

    let extension = CreateSymmetricKeyBuilder::detached()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 0 },
            item(0x0054_0001, Value::integer(7)),
        )
        .build_request()
        .expect("a positioned explicit extension Item remains unchanged");
    assert_eq!(
        extension.attributes().as_items()[0].tag().raw(),
        0x0054_0001
    );
}

#[test]
fn rejected_expert_attribute_position_is_not_sent() {
    let mut client = failed_request_client();
    let error = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 0 },
            item(
                typed::ContactInformation::TAG,
                Value::text_string("one".to_owned()),
            ),
        )
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 0 },
            item(
                typed::ContactInformation::TAG,
                Value::text_string("two".to_owned()),
            ),
        )
        .execute(&CodecLimits::defaults())
        .expect_err("duplicate positions fail before exchange");

    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
    assert!(!error.to_string().contains("one"));
    assert!(!error.to_string().contains("two"));
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

    let detached = CreateSymmetricKeyBuilder::detached()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT);
    let error = detached
        .execute(&CodecLimits::defaults())
        .expect_err("a detached builder cannot execute without a client");
    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
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
fn ping_fluent_builder_builds_the_existing_typed_request_and_executes_once() {
    let request = failed_request_client()
        .request()
        .ping()
        .build_request()
        .expect("Ping's empty payload is a valid typed request");
    assert_eq!(request, PingRequest::new());
    let batch_item = failed_request_client()
        .request()
        .ping()
        .to_batch_item()
        .expect("Ping converts to the existing typed batch item");
    assert_eq!(batch_item.operation(), PING_OPERATION);

    let response =
        asynchronous_response_bytes(PING_OPERATION, 0, None, None, Some(test_structure([])));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });
    let result = client
        .request()
        .ping()
        .execute(&CodecLimits::defaults())
        .expect("the scripted Ping response is valid");

    assert_eq!(
        result.outcome().operation(),
        crate::execute::ClientOperation::Ping
    );
    assert_eq!(result.outcome().result().status().raw(), 0);
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn discover_versions_builder_preserves_offers_and_executes_once() {
    let request = failed_request_client()
        .request()
        .discover_versions()
        .protocol_versions(vec![ProtocolVersion::V2_0, ProtocolVersion::V2_1])
        .build_request()
        .expect("catalogued protocol offers build a typed Discover Versions request");
    assert_eq!(
        request.protocol_versions(),
        &[ProtocolVersion::V2_0, ProtocolVersion::V2_1]
    );
    assert_eq!(
        DiscoverVersionsRequest::new().protocol_versions(),
        &[ProtocolVersion::V2_1]
    );
    let batch_item = failed_request_client()
        .request()
        .discover_versions()
        .to_batch_item()
        .expect("Discover Versions converts to the existing typed batch item");
    assert_eq!(batch_item.operation(), DISCOVER_VERSIONS_OPERATION);

    let response = asynchronous_response_bytes(
        DISCOVER_VERSIONS_OPERATION,
        0,
        None,
        None,
        Some(test_structure([])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });
    let result = client
        .request()
        .discover_versions()
        .execute(&CodecLimits::defaults())
        .expect("the scripted empty Discover Versions result is valid");

    assert_eq!(
        result.outcome().operation(),
        crate::execute::ClientOperation::DiscoverVersions
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn discover_versions_builder_rejects_invalid_offers_before_transport() {
    assert_eq!(
        failed_request_client()
            .request()
            .discover_versions()
            .protocol_versions(Vec::new())
            .build_request()
            .expect_err("an explicit empty offer is invalid"),
        FluentBuildError::InvalidProtocolVersions
    );
    assert_eq!(
        failed_request_client()
            .request()
            .discover_versions()
            .protocol_versions(vec![ProtocolVersion::from_raw(1, 4)])
            .validate()
            .expect_err("an uncatalogued version cannot be offered"),
        FluentBuildError::InvalidProtocolVersions
    );

    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .discover_versions()
        .protocol_versions(Vec::new())
        .execute(&CodecLimits::defaults())
        .expect_err("invalid offers fail before transport");
    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn query_builder_preserves_functions_and_groups_and_executes_once() {
    let request = failed_request_client()
        .request()
        .query()
        .functions(vec![QueryFunction::OPERATIONS, QueryFunction::OBJECTS])
        .object_groups(vec!["group-b".to_owned(), "group-a".to_owned()])
        .build_request()
        .expect("explicit Query functions form a typed request");
    assert_eq!(
        request.functions(),
        &[QueryFunction::OPERATIONS, QueryFunction::OBJECTS]
    );
    assert_eq!(
        request.object_groups(),
        Some(["group-b".to_owned(), "group-a".to_owned()].as_slice())
    );
    let _: QueryRequest = request;
    let batch_item = failed_request_client()
        .request()
        .query()
        .functions(vec![QueryFunction::OPERATIONS])
        .to_batch_item()
        .expect("Query converts to the existing typed batch item");
    assert_eq!(batch_item.operation(), QUERY_OPERATION);

    let response = asynchronous_response_bytes(
        QUERY_OPERATION,
        0,
        None,
        None,
        Some(test_structure([test_item(
            0x0042_015f,
            Value::structure(test_structure([])),
        )])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });
    let result = client
        .request()
        .query()
        .functions(vec![QueryFunction::OPERATIONS])
        .execute(&CodecLimits::defaults())
        .expect("the scripted Query response is valid");

    assert_eq!(
        result.outcome().operation(),
        crate::execute::ClientOperation::Query
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn query_builder_rejects_empty_functions_before_transport() {
    assert_eq!(
        failed_request_client()
            .request()
            .query()
            .build_request()
            .expect_err("Query requires at least one function"),
        FluentBuildError::InvalidRequest
    );

    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .query()
        .execute(&CodecLimits::defaults())
        .expect_err("an empty Query is rejected before transport");
    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn check_builder_builds_typed_policy_request_and_executes_once() {
    let identifier = UniqueIdentifier::TextString("checked-key".to_owned());
    let request = failed_request_client()
        .request()
        .check()
        .unique_identifier(identifier.clone())
        .usage_limits_count(3)
        .cryptographic_usage_mask(2)
        .lease_time(60)
        .build_request()
        .expect("explicit Check policy fields build a typed request");
    assert_eq!(request.unique_identifier(), Some(&identifier));
    let _: CheckRequest = request;
    let batch_item = failed_request_client()
        .request()
        .check()
        .to_batch_item()
        .expect("Check converts to the existing typed batch item");
    assert_eq!(batch_item.operation(), CHECK_OPERATION);

    let response = asynchronous_response_bytes(
        CHECK_OPERATION,
        0,
        None,
        None,
        Some(test_structure([test_item(
            0x0042_0094,
            Value::text_string("checked-key".to_owned()),
        )])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });
    let result = client
        .request()
        .check()
        .unique_identifier(identifier)
        .execute(&CodecLimits::defaults())
        .expect("the scripted Check result is valid");

    assert_eq!(
        result.outcome().operation(),
        crate::execute::ClientOperation::Check
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn get_constraints_builder_builds_typed_request_and_executes_once() {
    failed_request_client()
        .request()
        .get_constraints()
        .validate()
        .expect("the empty typed Get Constraints request is valid");
    let request = failed_request_client()
        .request()
        .get_constraints()
        .build_request()
        .expect("Get Constraints is an empty typed request");
    assert_eq!(request, GetConstraintsRequest::new());
    let batch_item = failed_request_client()
        .request()
        .get_constraints()
        .to_batch_item()
        .expect("Get Constraints converts to the existing typed batch item");
    assert_eq!(batch_item.operation(), GET_CONSTRAINTS_OPERATION);

    let response = asynchronous_response_bytes(
        GET_CONSTRAINTS_OPERATION,
        0,
        None,
        None,
        Some(test_structure([test_item(
            0x0042_0168,
            Value::structure(test_structure([])),
        )])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });
    let result = client
        .request()
        .get_constraints()
        .execute(&CodecLimits::defaults())
        .expect("the scripted Get Constraints response is valid");

    assert_eq!(
        result.outcome().operation(),
        crate::execute::ClientOperation::GetConstraints
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn get_usage_allocation_builder_requires_count_and_executes_typed_request() {
    assert_eq!(
        failed_request_client()
            .request()
            .get_usage_allocation()
            .build_request()
            .expect_err("Get Usage Allocation requires its requested count"),
        FluentBuildError::MissingGetUsageAllocationCount
    );

    let identifier = UniqueIdentifier::TextString("limited-key".to_owned());
    let request = failed_request_client()
        .request()
        .get_usage_allocation()
        .usage_limits_count(3)
        .unique_identifier(identifier.clone())
        .build_request()
        .expect("an explicit count forms a typed request");
    assert_eq!(request.usage_limits_count(), 3);
    assert_eq!(request.unique_identifier(), Some(&identifier));
    let _: GetUsageAllocationRequest = request;

    let batch_item = failed_request_client()
        .request()
        .get_usage_allocation()
        .usage_limits_count(3)
        .to_batch_item()
        .expect("Get Usage Allocation converts to the existing typed batch item");
    assert_eq!(batch_item.operation(), GET_USAGE_ALLOCATION_OPERATION);

    let response = asynchronous_response_bytes(
        GET_USAGE_ALLOCATION_OPERATION,
        0,
        None,
        None,
        Some(test_structure([test_item(
            0x0042_0094,
            Value::text_string("limited-key".to_owned()),
        )])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });
    let result = client
        .request()
        .get_usage_allocation()
        .usage_limits_count(3)
        .execute(&CodecLimits::defaults())
        .expect("the scripted usage allocation response is valid");

    assert_eq!(
        result.outcome().operation(),
        crate::execute::ClientOperation::GetUsageAllocation
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
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

#[test]
fn attribute_value_mutation_builders_preserve_typed_selectors_and_existing_requests() {
    let mut client = failed_request_client();
    let add = client
        .request()
        .add_attribute()
        .unique_identifier("key-42")
        .new_attribute(NewAttribute::from_typed(
            Attribute::<typed::Comment>::new("new-comment".to_owned())
                .expect("Comment is a catalogued Text String"),
        ))
        .build_request()
        .expect("Add Attribute has its required New Attribute");
    assert_eq!(add.unique_identifier(), Some("key-42"));
    assert_eq!(add.new_attribute().item().tag().raw(), typed::Comment::TAG);
    assert_eq!(
        client
            .request()
            .add_attribute()
            .new_attribute(NewAttribute::from_typed(
                Attribute::<typed::Comment>::new("batch-comment".to_owned())
                    .expect("Comment is a catalogued Text String"),
            ))
            .to_batch_item()
            .expect("Add Attribute converts to the existing batch item")
            .operation(),
        13
    );

    let modify = client
        .request()
        .modify_attribute()
        .unique_identifier("key-42")
        .current_attribute(CurrentAttribute::from_typed(
            Attribute::<typed::Comment>::new("old-comment".to_owned())
                .expect("Comment is a catalogued Text String"),
        ))
        .new_attribute(NewAttribute::from_typed(
            Attribute::<typed::Comment>::new("updated-comment".to_owned())
                .expect("Comment is a catalogued Text String"),
        ))
        .build_request()
        .expect("Modify Attribute has its required New Attribute");
    assert_eq!(modify.unique_identifier(), Some("key-42"));
    assert_eq!(
        modify
            .current_attribute()
            .expect("the optional current selector is preserved")
            .item()
            .tag()
            .raw(),
        typed::Comment::TAG
    );
    assert_eq!(
        modify.new_attribute().item().tag().raw(),
        typed::Comment::TAG
    );
    assert_eq!(
        client
            .request()
            .modify_attribute()
            .new_attribute(NewAttribute::from_typed(
                Attribute::<typed::Comment>::new("batch-update".to_owned())
                    .expect("Comment is a catalogued Text String"),
            ))
            .to_batch_item()
            .expect("Modify Attribute converts to the existing batch item")
            .operation(),
        14
    );

    let set = client
        .request()
        .set_attribute()
        .unique_identifier("key-42")
        .new_attribute(NewAttribute::from_typed(
            Attribute::<typed::Comment>::new("replacement-comment".to_owned())
                .expect("Comment is a catalogued Text String"),
        ))
        .build_request()
        .expect("Set Attribute has its required New Attribute");
    assert_eq!(set.unique_identifier(), Some("key-42"));
    assert_eq!(set.new_attribute().item().tag().raw(), typed::Comment::TAG);
    assert_eq!(
        client
            .request()
            .set_attribute()
            .new_attribute(NewAttribute::from_typed(
                Attribute::<typed::Comment>::new("batch-replacement".to_owned())
                    .expect("Comment is a catalogued Text String"),
            ))
            .to_batch_item()
            .expect("Set Attribute converts to the existing batch item")
            .operation(),
        49
    );
}

#[test]
fn attribute_selector_mutation_builders_preserve_explicit_values_and_requests() {
    let mut client = failed_request_client();
    let delete = client
        .request()
        .delete_attribute()
        .unique_identifier("key-42")
        .current_attribute(CurrentAttribute::from_typed(
            Attribute::<typed::Comment>::new("old-comment".to_owned())
                .expect("Comment is a catalogued Text String"),
        ))
        .build_request()
        .expect("Delete Attribute selectors form a valid typed request");
    assert_eq!(delete.unique_identifier(), Some("key-42"));
    assert_eq!(
        delete
            .current_attribute()
            .expect("the selected current value is preserved")
            .item()
            .tag()
            .raw(),
        typed::Comment::TAG
    );
    assert_eq!(
        client
            .request()
            .delete_attribute()
            .unique_identifier("key-42")
            .attribute_reference(AttributeReference::tag(typed::Comment::TAG))
            .to_batch_item()
            .expect("Delete Attribute converts to the existing batch item")
            .operation(),
        15
    );

    let adjust = client
        .request()
        .adjust_attribute()
        .unique_identifier("key-42")
        .attribute_reference(AttributeReference::tag(typed::UsageLimits::TAG))
        .adjustment_type(AdjustmentType::INCREMENT)
        .adjustment_value(AttributeAdjustmentValue::integer(1))
        .build_request()
        .expect("Adjust Attribute retains the selected type and caller value");
    assert_eq!(adjust.unique_identifier(), Some("key-42"));
    assert_eq!(
        adjust.attribute_reference().tag_value(),
        Some(typed::UsageLimits::TAG)
    );
    assert_eq!(adjust.adjustment_type(), AdjustmentType::INCREMENT);
    assert!(adjust.adjustment_value().is_some());
    assert_eq!(
        client
            .request()
            .adjust_attribute()
            .attribute_reference(AttributeReference::tag(typed::UsageLimits::TAG))
            .adjustment_type(AdjustmentType::INCREMENT)
            .adjustment_value(AttributeAdjustmentValue::integer(1))
            .to_batch_item()
            .expect("Adjust Attribute converts to the existing batch item")
            .operation(),
        48
    );
}

#[test]
fn invalid_attribute_mutation_is_rejected_without_sending() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .add_attribute()
        .new_attribute(NewAttribute::new(item(
            typed::CryptographicLength::TAG,
            Value::text_string("wrong-ttlv-type".to_owned()),
        )))
        .execute(&CodecLimits::defaults())
        .expect_err("a catalogued attribute with the wrong TTLV type is rejected locally");

    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn adjust_attribute_requires_explicit_reference_and_type_before_sending() {
    let mut client = failed_request_client();
    assert_eq!(
        client
            .request()
            .adjust_attribute()
            .build_request()
            .expect_err("the reference is required"),
        FluentBuildError::MissingAdjustAttributeReference
    );

    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .adjust_attribute()
        .attribute_reference(AttributeReference::tag(typed::UsageLimits::TAG))
        .execute(&CodecLimits::defaults())
        .expect_err("an explicit adjustment type is required before transport");
    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn attribute_mutation_builders_execute_the_matching_typed_route_once() {
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: attribute_success_response_bytes(13),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .add_attribute()
        .unique_identifier("key-42")
        .new_attribute(NewAttribute::from_typed(
            Attribute::<typed::Comment>::new("added".to_owned())
                .expect("Comment is a catalogued Text String"),
        ))
        .execute(&CodecLimits::defaults())
        .expect("Add Attribute uses its existing typed client route");
    assert_eq!(
        response.outcome().operation(),
        crate::execute::ClientOperation::AddAttribute
    );
    assert_eq!(fake.borrow().exchange_count(), 1);

    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: attribute_success_response_bytes(14),
        request_write_chunks: Vec::new(),
    });
    let builder = client
        .request()
        .modify_attribute()
        .unique_identifier("key-42")
        .current_attribute(CurrentAttribute::from_typed(
            Attribute::<typed::Comment>::new("before-modify".to_owned())
                .expect("Comment is a catalogued Text String"),
        ))
        .new_attribute(NewAttribute::from_typed(
            Attribute::<typed::Comment>::new("modified".to_owned())
                .expect("Comment is a catalogued Text String"),
        ));
    assert_eq!(builder.validate(), Ok(()));
    let response = builder
        .execute(&CodecLimits::defaults())
        .expect("Modify Attribute uses its existing typed client route");
    assert_eq!(
        response.outcome().operation(),
        crate::execute::ClientOperation::ModifyAttribute
    );
    assert_eq!(fake.borrow().exchange_count(), 1);

    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: attribute_success_response_bytes(49),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .set_attribute()
        .unique_identifier("key-42")
        .new_attribute(NewAttribute::from_typed(
            Attribute::<typed::Comment>::new("set".to_owned())
                .expect("Comment is a catalogued Text String"),
        ))
        .execute(&CodecLimits::defaults())
        .expect("Set Attribute uses its existing typed client route");
    assert_eq!(
        response.outcome().operation(),
        crate::execute::ClientOperation::SetAttribute
    );
    assert_eq!(fake.borrow().exchange_count(), 1);

    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: attribute_success_response_bytes(15),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .delete_attribute()
        .unique_identifier("key-42")
        .attribute_reference(AttributeReference::tag(typed::Comment::TAG))
        .execute(&CodecLimits::defaults())
        .expect("Delete Attribute uses its existing typed client route");
    assert_eq!(
        response.outcome().operation(),
        crate::execute::ClientOperation::DeleteAttribute
    );
    assert_eq!(fake.borrow().exchange_count(), 1);

    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: attribute_success_response_bytes(48),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .adjust_attribute()
        .unique_identifier("key-42")
        .attribute_reference(AttributeReference::tag(typed::UsageLimits::TAG))
        .adjustment_type(AdjustmentType::INCREMENT)
        .adjustment_value(AttributeAdjustmentValue::integer(1))
        .execute(&CodecLimits::defaults())
        .expect("Adjust Attribute uses its existing typed client route");
    assert_eq!(
        response.outcome().operation(),
        crate::execute::ClientOperation::AdjustAttribute
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
