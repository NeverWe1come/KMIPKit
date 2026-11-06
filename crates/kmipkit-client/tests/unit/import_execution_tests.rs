//! Fake-transport Import contracts from OASIS KMIP v2.1 §6.1.25, Tables 238–240.

use kmipkit_protocol::{
    AttributeSet, ImportRequest, KeyWrapType, ObjectType, ResultReason, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, Value};

use super::{ClientOperation, IMPORT_OPERATION, encode_message_for_test};
use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute_test_support::{test_item, test_structure};

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const SERVER_REJECTED_OBJECT: u32 = 0xF123_4567;

fn attributes() -> AttributeSet {
    AttributeSet::try_new([
        test_item(0x0042_0028, Value::enumeration(3)),
        test_item(0x0042_002A, Value::integer(256)),
    ])
    .expect("caller attributes are valid")
}

fn object() -> Item {
    let wrapping_data = test_structure([
        test_item(0x0042_009E, Value::enumeration(1)),
        test_item(
            0x0042_0036,
            Value::structure(test_structure([test_item(
                0x0042_0094,
                Value::text_string("wrapping-key".to_owned()),
            )])),
        ),
    ]);
    // KMIP Specification v2.1 §3.1/Table 12 represents wrapped Key Value as
    // opaque bytes; the separate Key Wrapping Data describes the wrapper.
    let key_block = test_structure([
        test_item(
            0x0042_0045,
            Value::byte_string(b"synthetic-wrapped-import-object".to_vec()),
        ),
        test_item(0x0042_0046, Value::structure(wrapping_data)),
    ]);
    test_item(
        0x0042_008F,
        Value::structure(test_structure([test_item(
            0x0042_0040,
            Value::structure(key_block),
        )])),
    )
}

fn request() -> ImportRequest {
    ImportRequest::new(
        UniqueIdentifier::TextString("import-key".to_owned()),
        ObjectType::from_raw(2),
        attributes(),
        object(),
    )
    .with_replace_existing(true)
    .with_key_wrap_type(KeyWrapType::from_raw(0xF123_4568))
}

fn server_error_response() -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = test_structure([
        test_item(OPERATION, Value::enumeration(IMPORT_OPERATION)),
        test_item(RESULT_STATUS, Value::enumeration(OPERATION_FAILED)),
        test_item(RESULT_REASON, Value::enumeration(SERVER_REJECTED_OBJECT)),
    ]);
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(batch_item)),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("fixture is a valid KMIP 2.1 response")
}

fn successful_response() -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let payload = test_structure([
        test_item(0x0042_0094, Value::text_string("imported-id".to_owned())),
        test_item(0x0054_0001, Value::enumeration(0xF123_4567)),
    ]);
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(
                BATCH_ITEM,
                Value::structure(test_structure([
                    test_item(OPERATION, Value::enumeration(IMPORT_OPERATION)),
                    test_item(RESULT_STATUS, Value::enumeration(SUCCESS)),
                    test_item(RESPONSE_PAYLOAD, Value::structure(payload)),
                ])),
            ),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("fixture is a valid KMIP 2.1 success response")
}

#[test]
fn explicit_import_sends_supplied_attributes_and_object_once_and_preserves_server_error() {
    assert_eq!(IMPORT_OPERATION, 0x0000_002A);
    let (mut client, fake, request_bytes) = client_for(ExchangeScript::Success {
        response: server_error_response(),
        request_write_chunks: vec![7, 3, 13],
    });
    let response = client
        .import(request(), &CodecLimits::defaults())
        .expect("the KMIP failure is returned as a result, not a local unwrap/import");

    assert_eq!(response.outcome().operation(), ClientOperation::Import);
    assert_eq!(response.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.outcome().result().reason().map(ResultReason::raw),
        Some(SERVER_REJECTED_OBJECT)
    );
    assert!(request_contains(&request_bytes, b"import-key"));
    assert!(request_contains(
        &request_bytes,
        b"synthetic-wrapped-import-object"
    ));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn successful_import_preserves_result_identifier_and_unknown_ordered_response_once() {
    let (mut client, fake, request_bytes) = client_for(ExchangeScript::Success {
        response: successful_response(),
        request_write_chunks: vec![5, 9, 17],
    });
    let response = client
        .import(request(), &CodecLimits::defaults())
        .expect("the successful Import response remains typed");
    let outcome = response.outcome();
    let typed = outcome
        .response()
        .import()
        .expect("the result retains its typed Import response");

    assert_eq!(outcome.operation(), ClientOperation::Import);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&UniqueIdentifier::TextString("imported-id".to_owned()))
    );
    assert_eq!(
        typed.with_ttlv(|view| {
            let fields = view.children();
            fields
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>()
        }),
        vec![OPERATION, RESULT_STATUS, RESPONSE_PAYLOAD]
    );
    assert!(typed.with_ttlv(|view| {
        crate::execute_test_support::structure_contains_tag(&view, 0x0054_0001)
    }));
    assert!(request_contains(&request_bytes, b"import-key"));
    assert!(request_contains(
        &request_bytes,
        b"synthetic-wrapped-import-object"
    ));
    assert_eq!(fake.borrow().exchange_count(), 1);
}
