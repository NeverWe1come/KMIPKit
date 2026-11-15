//! Fake-transport Derive Key execution contracts from KMIP 2.1 §6.1.14 Tables 205–207 and §7.12.

use kmipkit_protocol::{
    AttributeSet, DerivationMethod, DerivationParameters, DeriveKeyRequest, ObjectType,
    ResultReason, SecretBytes, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;

use super::{ClientOperation, DERIVE_KEY_OPERATION, encode_message_for_test};
use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute_test_support::{test_item, test_structure};

const RESPONSE_HEADER: u32 = 0x0042_007a;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006a;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006b;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000d;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const RESULT_STATUS: u32 = 0x0042_007f;
const RESULT_REASON: u32 = 0x0042_007e;
const OPERATION_FAILED: u32 = 1;
const SERVER_REJECTED_BOTH_INPUTS: u32 = 0xF123_4567;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;

fn request() -> DeriveKeyRequest {
    let attributes = AttributeSet::try_new([
        test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
    ])
    .expect("symmetric-key output attributes have valid TTLV types");
    DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("secret-data-id".to_owned())],
        DerivationMethod::from_raw(1),
        DerivationParameters::new()
            .with_derivation_data(SecretBytes::new(b"synthetic-derivation-input".to_vec()))
            .with_salt(SecretBytes::new(b"synthetic-salt".to_vec()))
            .with_iteration_count(120_000),
        attributes,
    )
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
        test_item(OPERATION, Value::enumeration(DERIVE_KEY_OPERATION)),
        test_item(RESULT_STATUS, Value::enumeration(OPERATION_FAILED)),
        test_item(
            RESULT_REASON,
            Value::enumeration(SERVER_REJECTED_BOTH_INPUTS),
        ),
    ]);
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(batch_item)),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("the fixture is a valid KMIP 2.1 response")
}

#[test]
fn both_derivation_inputs_reach_the_server_and_its_error_is_returned_once() {
    assert_eq!(DERIVE_KEY_OPERATION, 0x0000_0005);
    let (mut client, fake, request_bytes) = client_for(ExchangeScript::Success {
        response: server_error_response(),
        request_write_chunks: vec![2, 7, 3],
    });
    let response = client
        .derive_key(request(), &CodecLimits::defaults())
        .expect("the operation's server failure is a protocol result, not local validation");

    assert_eq!(response.outcome().operation(), ClientOperation::DeriveKey);
    assert_eq!(response.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.outcome().result().reason().map(ResultReason::raw),
        Some(SERVER_REJECTED_BOTH_INPUTS)
    );
    assert!(request_contains(
        &request_bytes,
        b"synthetic-derivation-input"
    ));
    assert!(request_contains(&request_bytes, b"secret-data-id"));
    assert_eq!(fake.borrow().exchange_count(), 1);
}
