//! Fake-transport Export contracts from OASIS KMIP v2.1 §6.1.18, Tables 217–219.

use kmipkit_protocol::{
    ExportRequest, KeyCompressionType, KeyFormatType, KeyWrapType, ResultReason, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;

use super::{ClientOperation, EXPORT_OPERATION, encode_message_for_test};
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
const OPERATION_FAILED: u32 = 1;
const SERVER_REJECTED_OPTION: u32 = 0xF123_4567;

fn request() -> ExportRequest {
    ExportRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("export-key".to_owned()))
        .with_key_format_type(KeyFormatType::from_raw(0xF123_4568))
        .with_key_wrap_type(KeyWrapType::from_raw(0xF123_4569))
        .with_key_compression_type(KeyCompressionType::from_raw(0xF123_456A))
        .with_key_wrapping_specification(test_structure([test_item(
            0x0054_1234,
            Value::text_string("synthetic-wrapping-specification".to_owned()),
        )]))
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
    let item = test_structure([
        test_item(OPERATION, Value::enumeration(EXPORT_OPERATION)),
        test_item(RESULT_STATUS, Value::enumeration(OPERATION_FAILED)),
        test_item(RESULT_REASON, Value::enumeration(SERVER_REJECTED_OPTION)),
    ]);
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(item)),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("fixture is a valid KMIP 2.1 response")
}

#[test]
fn explicit_export_options_reach_the_server_once_and_preserve_its_failure() {
    assert_eq!(EXPORT_OPERATION, 0x0000_002B);
    let (mut client, fake, request_bytes) = client_for(ExchangeScript::Success {
        response: server_error_response(),
        request_write_chunks: vec![4, 8, 3],
    });
    let response = client
        .export(request(), &CodecLimits::defaults())
        .expect("the KMIP failure is returned as a result, not a local validation error");

    assert_eq!(response.outcome().operation(), ClientOperation::Export);
    assert_eq!(response.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.outcome().result().reason().map(ResultReason::raw),
        Some(SERVER_REJECTED_OPTION)
    );
    assert!(request_contains(&request_bytes, b"export-key"));
    assert!(request_contains(
        &request_bytes,
        b"synthetic-wrapping-specification"
    ));
    assert_eq!(fake.borrow().exchange_count(), 1);
}
