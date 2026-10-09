//! Redaction regressions for the KMIPKIT-0018 lifecycle values.
//!
//! Derived from the project's KMIPKIT-0018 FR-010 security requirement and
//! shared KMIPKIT-0007 error contract. These are project-policy tests, not
//! official OASIS Test Cases.

use std::error::Error;

use kmipkit_protocol::{
    ActivateRequest, ArchiveRequest, DestroyRequest, RecoverRequest, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::Value;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};
use crate::{ClientBatch, ClientBatchItem, ClientRequest};

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;

const LIFECYCLE_ID_SENTINEL: &str = "KMIP_LIFECYCLE_ID_SENTINEL_1846";
const RESULT_MESSAGE_SENTINEL: &str = "KMIP_LIFECYCLE_RESULT_SENTINEL_3175";
const RAW_BODY_SENTINEL: &[u8] = b"KMIP_LIFECYCLE_RAW_BODY_SENTINEL_6284";

#[test]
fn lifecycle_request_and_response_debug_redact_unique_identifiers() {
    let identifier = UniqueIdentifier::TextString(LIFECYCLE_ID_SENTINEL.to_owned());
    let requests = [
        ClientRequest::Activate(ActivateRequest::new(Some(identifier.clone()))),
        ClientRequest::Archive(ArchiveRequest::new(Some(identifier.clone()))),
        ClientRequest::Destroy(DestroyRequest::new(Some(identifier.clone()))),
        ClientRequest::Recover(RecoverRequest::new(Some(identifier))),
    ];

    for request in requests {
        let typed_request_debug = typed_request_debug(&request);
        assert!(
            !typed_request_debug.contains(LIFECYCLE_ID_SENTINEL),
            "lifecycle request Debug must redact its identifier"
        );
        let (mut client, _, _) = client_for(ExchangeScript::Success {
            response: successful_response(operation_for(&request), LIFECYCLE_ID_SENTINEL),
            request_write_chunks: Vec::new(),
        });
        let response = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(request)),
                &CodecLimits::defaults(),
            )
            .expect("the successful lifecycle fixture is well formed");
        let item = response.get(0).expect("one lifecycle response exists");
        let outcome = item.outcome();
        let view = outcome.response();
        let formatted = format!("{outcome:?}{view:?}{}", typed_response_debug(view));
        assert!(
            !formatted.contains(LIFECYCLE_ID_SENTINEL),
            "lifecycle response Debug must redact its identifier"
        );
    }
}

#[test]
fn lifecycle_errors_and_debug_never_include_result_text_or_raw_response_body() {
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: failed_response(0x0000_0012, RESULT_MESSAGE_SENTINEL),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::Activate(
                ActivateRequest::new(None),
            ))),
            &CodecLimits::defaults(),
        )
        .expect("a server Failure is a complete lifecycle response");
    let item = response.get(0).expect("one lifecycle response exists");
    let formatted = format!("{:?}{:?}", item.outcome(), item.outcome().response());
    assert!(
        !formatted.contains(RESULT_MESSAGE_SENTINEL),
        "lifecycle response Debug must redact untrusted Result Message text"
    );

    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: RAW_BODY_SENTINEL.to_vec(),
        request_write_chunks: Vec::new(),
    });
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::Recover(
                RecoverRequest::new(None),
            ))),
            &CodecLimits::defaults(),
        )
        .expect_err("the raw sentinel is not a valid TTLV response");
    let mut chain = Vec::new();
    let mut source = Some(&error as &dyn Error);
    while let Some(cause) = source {
        chain.push(cause.to_string());
        source = cause.source();
    }
    let diagnostics = format!("{error:?}{error}{chain:?}");
    assert!(
        !diagnostics.contains(std::str::from_utf8(RAW_BODY_SENTINEL).expect("ASCII sentinel")),
        "public error formatting and sources must not include raw response bytes"
    );
}

fn operation_for(request: &ClientRequest) -> u32 {
    match request {
        ClientRequest::Activate(_) => 0x0000_0012,
        ClientRequest::Archive(_) => 0x0000_0013,
        ClientRequest::Destroy(_) => 0x0000_0014,
        ClientRequest::Recover(_) => 0x0000_002A,
        _ => unreachable!("test constructs only lifecycle requests"),
    }
}

fn typed_request_debug(request: &ClientRequest) -> String {
    match request {
        ClientRequest::Activate(request) => format!("{request:?}"),
        ClientRequest::Archive(request) => format!("{request:?}"),
        ClientRequest::Destroy(request) => format!("{request:?}"),
        ClientRequest::Recover(request) => format!("{request:?}"),
        _ => unreachable!("test constructs only lifecycle requests"),
    }
}

fn typed_response_debug(response: crate::ClientResponseView<'_>) -> String {
    if let Some(response) = response.activate() {
        format!("{response:?}")
    } else if let Some(response) = response.archive() {
        format!("{response:?}")
    } else if let Some(response) = response.destroy() {
        format!("{response:?}")
    } else if let Some(response) = response.recover() {
        format!("{response:?}")
    } else {
        unreachable!("test response is one of the lifecycle operations")
    }
}

fn successful_response(operation: u32, identifier: &str) -> Vec<u8> {
    let payload = test_structure([test_item(
        UNIQUE_IDENTIFIER,
        Value::text_string(identifier.to_owned()),
    )]);
    lifecycle_response(operation, 0, None, Some(payload))
}

fn failed_response(operation: u32, result_message: &str) -> Vec<u8> {
    lifecycle_response(operation, 1, Some(result_message), None)
}

fn lifecycle_response(
    operation: u32,
    status: u32,
    result_message: Option<&str>,
    payload: Option<kmipkit_ttlv::Structure>,
) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut batch_fields = vec![
        test_item(OPERATION, Value::enumeration(operation)),
        test_item(RESULT_STATUS, Value::enumeration(status)),
    ];
    if status == 1 {
        batch_fields.push(test_item(0x0042_007E, Value::enumeration(0x37)));
    }
    if let Some(message) = result_message {
        batch_fields.push(test_item(0x0042_007D, Value::text_string(message.to_owned())));
    }
    if let Some(payload) = payload {
        batch_fields.push(test_item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(test_structure(batch_fields))),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("lifecycle test response is encodable")
}
