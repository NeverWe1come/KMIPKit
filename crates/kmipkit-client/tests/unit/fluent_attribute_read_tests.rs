//! Fluent attribute-read contracts from KMIPKIT-0108 and KMIP v2.1
//! §§6.1.20–6.1.21.

use kmipkit_protocol::AttributeReference;
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Structure, Value};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};
use crate::{ClientOperation, ClientRequest, FluentBuildError};

const GET_ATTRIBUTES: u32 = 0x0000_000B;
const GET_ATTRIBUTE_LIST: u32 = 0x0000_000C;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIMESTAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ATTRIBUTES: u32 = 0x0042_0125;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;

fn response_bytes(operation: u32, payload: Structure) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIMESTAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch = test_structure([
        test_item(OPERATION, Value::enumeration(operation)),
        test_item(RESULT_STATUS, Value::enumeration(0)),
        test_item(RESPONSE_PAYLOAD, Value::structure(payload)),
    ]);
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(batch)),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("the success fixture is valid KMIP TTLV")
}

fn successful_attributes_payload() -> Structure {
    test_structure([
        test_item(UNIQUE_IDENTIFIER, Value::text_string("object-42".to_owned())),
        test_item(ATTRIBUTES, Value::structure(Structure::new())),
    ])
}

fn successful_attribute_list_payload() -> Structure {
    test_structure([
        test_item(UNIQUE_IDENTIFIER, Value::text_string("object-42".to_owned())),
        test_item(
            ATTRIBUTE_REFERENCE,
            Value::structure(test_structure([
                test_item(
                    VENDOR_IDENTIFICATION,
                    Value::text_string("KMIPKit_TestVendor".to_owned()),
                ),
                test_item(
                    ATTRIBUTE_NAME,
                    Value::text_string("Opaque.ReadableAttribute".to_owned()),
                ),
            ])),
        ),
    ])
}

#[test]
fn get_attributes_builder_preserves_identifier_reference_order_and_duplicates() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let first = AttributeReference::tag(0x0042_002F);
    let second = AttributeReference::name("KMIPKit_TestVendor", "ReadableAttribute");
    let request = client
        .request()
        .get_attributes()
        .unique_identifier("object-42")
        .attribute_reference(first.clone())
        .attribute_reference(second.clone())
        .build_request()
        .expect("distinct references form a typed request");

    assert_eq!(request.unique_identifier(), Some("object-42"));
    assert_eq!(request.attribute_references(), &[first.clone(), second]);

    let mut client = client_for(ExchangeScript::FailBeforeWrite).0;
    let error = client
        .request()
        .get_attributes()
        .attribute_reference(first.clone())
        .attribute_reference(first)
        .validate()
        .expect_err("the typed request rejects a repeated Attribute Reference");
    assert_eq!(error, FluentBuildError::InvalidRequest);
}

#[test]
fn attribute_read_builders_construct_the_matching_typed_batch_items() {
    let mut client = client_for(ExchangeScript::FailBeforeWrite).0;
    let get_attributes = client
        .request()
        .get_attributes()
        .attribute_reference(AttributeReference::tag(0x0042_002F))
        .to_batch_item()
        .expect("Get Attributes converts to a typed batch item");
    assert!(matches!(get_attributes.request(), ClientRequest::GetAttributes(_)));

    let mut client = client_for(ExchangeScript::FailBeforeWrite).0;
    let get_list = client
        .request()
        .get_attribute_list()
        .to_batch_item()
        .expect("Get Attribute List converts to a typed batch item");
    assert!(matches!(get_list.request(), ClientRequest::GetAttributeList(_)));
    assert_eq!(get_list.operation(), ClientOperation::GetAttributeList);
}

#[test]
fn attribute_read_builders_execute_one_exchange_and_keep_typed_response_shapes() {
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: response_bytes(GET_ATTRIBUTES, successful_attributes_payload()),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .get_attributes()
        .unique_identifier("object-42")
        .execute(&CodecLimits::defaults())
        .expect("Get Attributes receives a valid typed response");
    assert_eq!(transport.borrow().exchange_count(), 1);
    let attributes = response
        .get_attributes()
        .expect("the operation-specific response is retained")
        .attributes()
        .expect("a present empty Attributes structure remains present");
    assert!(attributes.as_items().is_empty());

    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: response_bytes(GET_ATTRIBUTE_LIST, successful_attribute_list_payload()),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .get_attribute_list()
        .execute(&CodecLimits::defaults())
        .expect("Get Attribute List receives a valid typed response");
    assert_eq!(transport.borrow().exchange_count(), 1);
    let references = response
        .get_attribute_list()
        .expect("the operation-specific response is retained")
        .attribute_references()
        .expect("the successful response contains the required reference");
    assert_eq!(references.len(), 1);
    assert_eq!(references[0].name_parts(), Some(("KMIPKit_TestVendor", "Opaque.ReadableAttribute")));
}
