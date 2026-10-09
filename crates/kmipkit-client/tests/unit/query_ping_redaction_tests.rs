//! Query and Ping diagnostics must not reveal raw request or response payloads.

use std::error::Error;

use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::codec::CodecLimits;

use crate::ClientError;
use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatchOutcome, ClientRequest};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use kmipkit_protocol::{QueryFunction, QueryRequest};

const QUERY: u32 = 0x0000_0018;
const PING: u32 = 0x0000_003b;
const OPERATION: u32 = 0x0042_005c;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;

fn error_chain_text(error: &ClientError) -> String {
    let mut messages = vec![format!("{error}", error = error)];
    let mut current: &(dyn Error + 'static) = error;
    while let Some(source) = current.source() {
        messages.push(source.to_string());
        current = source;
    }
    messages.join("\n")
}

#[test]
fn query_request_and_success_diagnostics_redact_ttlv_text_values() {
    let marker = "KMIPKIT_QUERY_RAW_BODY_SENTINEL";
    let request = QueryRequest::new([QueryFunction::OPERATIONS]).with_object_groups([marker]);
    let query_request_debug = format!("{request:?}");
    let client_request_debug = format!(
        "{:?}",
        ClientRequest::query(
            QueryRequest::new([QueryFunction::OPERATIONS]).with_object_groups([marker])
        )
    );

    let response = asynchronous_response_bytes(
        QUERY,
        0,
        None,
        None,
        Some(test_structure([
            test_item(
                PROTECTION_STORAGE_MASKS,
                kmipkit_ttlv::Value::structure(test_structure([])),
            ),
            test_item(
                0x0042_0012,
                kmipkit_ttlv::Value::text_string(marker.to_owned()),
            ),
        ])),
    );
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let outcome = client
        .query(request, &CodecLimits::defaults())
        .expect("the Query response is structurally valid");
    let ClientBatchOutcome::Query(query) = outcome.outcome() else {
        panic!("the typed outcome is Query");
    };
    let diagnostics = [
        query_request_debug,
        client_request_debug,
        format!("{:?}", ClientRequest::ping()),
        format!("{:?}", outcome),
        format!("{:?}", outcome.outcome()),
        format!("{}", outcome.outcome()),
        format!("{:?}", query),
        format!("{:?}", outcome.outcome().response()),
    ]
    .join("\n");

    assert!(!diagnostics.contains(marker));

    let ping_response = asynchronous_response_bytes(PING, 0, None, None, Some(test_structure([])));
    let (mut ping_client, _, _) = client_for(ExchangeScript::Success {
        response: ping_response,
        request_write_chunks: Vec::new(),
    });
    let ping_outcome = ping_client
        .ping(&CodecLimits::defaults())
        .expect("the Ping response payload is empty");
    let ClientBatchOutcome::Ping(ping) = ping_outcome.outcome() else {
        panic!("the typed outcome is Ping");
    };
    let ping_diagnostics = [
        format!("{:?}", ping_outcome),
        format!("{:?}", ping_outcome.outcome()),
        format!("{}", ping_outcome.outcome()),
        format!("{:?}", ping),
        format!("{:?}", ping_outcome.outcome().response()),
    ]
    .join("\n");
    assert!(!ping_diagnostics.contains(marker));
}

#[test]
fn query_and_ping_error_chains_redact_malformed_response_payloads() {
    let marker = "KMIPKIT_MALFORMED_RAW_BODY_SENTINEL";

    let query_response = asynchronous_response_bytes(
        QUERY,
        0,
        None,
        None,
        Some(test_structure([
            test_item(
                OPERATION,
                kmipkit_ttlv::Value::text_string(marker.to_owned()),
            ),
            test_item(
                PROTECTION_STORAGE_MASKS,
                kmipkit_ttlv::Value::structure(test_structure([])),
            ),
        ])),
    );
    let (mut query_client, _, _) = client_for(ExchangeScript::Success {
        response: query_response,
        request_write_chunks: Vec::new(),
    });
    let query_error = query_client
        .query(
            QueryRequest::new([QueryFunction::OPERATIONS]),
            &CodecLimits::defaults(),
        )
        .expect_err("a known Query field with the wrong Item Type is rejected");
    assert!(
        !format!(
            "{:?}\n{query_error}\n{}",
            query_error,
            error_chain_text(&query_error)
        )
        .contains(marker)
    );

    let ping_response = asynchronous_response_bytes(
        PING,
        0,
        None,
        None,
        Some(test_structure([test_item(
            0x0042_0012,
            kmipkit_ttlv::Value::text_string(marker.to_owned()),
        )])),
    );
    let (mut ping_client, _, _) = client_for(ExchangeScript::Success {
        response: ping_response,
        request_write_chunks: Vec::new(),
    });
    let ping_error = ping_client
        .ping(&CodecLimits::defaults())
        .expect_err("a Ping payload must stay empty");
    assert!(
        !format!(
            "{:?}\n{ping_error}\n{}",
            ping_error,
            error_chain_text(&ping_error)
        )
        .contains(marker)
    );
}
