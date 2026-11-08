//! Fake-transport Set Endpoint Role behavior derived from OASIS KMIP v2.1 §6.1.54.

use kmipkit_protocol::{EndpointRole, SetEndpointRoleRequest};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::codec::CodecLimits;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatchItem, ClientOperation, ClientRequest};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};

const SET_ENDPOINT_ROLE: u32 = 0x0000_0032;
const ENDPOINT_ROLE: u32 = 0x0042_0151;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009d;

#[test]
fn set_endpoint_role_success_execution_and_accessors() {
    let payload = test_structure([
        test_item(ENDPOINT_ROLE, kmipkit_ttlv::Value::enumeration(2)), // Server
        test_item(
            VENDOR_IDENTIFICATION,
            kmipkit_ttlv::Value::text_string("KMIPKitServer".to_string()),
        ),
    ]);
    let response = asynchronous_response_bytes(SET_ENDPOINT_ROLE, 0, None, None, Some(payload));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });

    let req = SetEndpointRoleRequest::new(EndpointRole::SERVER)
        .with_vendor_identification("KMIPKitServer");
    let item_resp = client
        .set_endpoint_role(req, &CodecLimits::defaults())
        .expect("set_endpoint_role succeeds");

    assert_eq!(
        item_resp.outcome().operation(),
        ClientOperation::SetEndpointRole
    );
    assert_eq!(ClientOperation::SetEndpointRole.raw(), 0x32);
    assert!(item_resp.set_endpoint_role().is_some());
    assert_eq!(
        item_resp
            .outcome()
            .set_endpoint_role_response()
            .unwrap()
            .endpoint_role(),
        Some(EndpointRole::SERVER)
    );
    let view = item_resp.outcome().response();
    assert!(view.set_endpoint_role().is_some());
    assert_eq!(
        view.set_endpoint_role().unwrap().vendor_identification(),
        Some("KMIPKitServer")
    );
    assert_eq!(view.result().status().raw(), 0);
    assert!(format!("{view:?}").contains("SetEndpointRole"));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn set_endpoint_role_pending_and_failures() {
    use crate::execute::ClientBatch;

    const CORRELATION: &[u8] = b"set-role-correlation";
    let payload = test_structure([test_item(
        ENDPOINT_ROLE,
        kmipkit_ttlv::Value::enumeration(2),
    )]);
    let response =
        asynchronous_response_bytes(SET_ENDPOINT_ROLE, 2, None, Some(CORRELATION), Some(payload));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let req = SetEndpointRoleRequest::new(EndpointRole::SERVER);
    let batch = ClientBatch::new(ClientBatchItem::set_endpoint_role(req.clone()))
        .with_asynchronous_indicator(2);
    let batch_resp = client
        .execute(batch, &CodecLimits::defaults())
        .expect("execute succeeds");
    let item_resp = batch_resp.get(0).expect("item exists");
    assert!(item_resp.outcome().set_endpoint_role_response().is_some());
    assert_eq!(item_resp.outcome().result().status().raw(), 2);

    // Failure branch in set_endpoint_role_with_options
    let (mut fail_client, _, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let err = fail_client
        .set_endpoint_role(req, &CodecLimits::defaults())
        .unwrap_err();
    assert_eq!(
        err.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::PossiblySent)
    );

    // view.set_endpoint_role() returns None for non-role responses
    let ping_response =
        asynchronous_response_bytes(0x0000_003b, 0, None, None, Some(test_structure([])));
    let (mut ping_client, _, _) = client_for(ExchangeScript::Success {
        response: ping_response,
        request_write_chunks: Vec::new(),
    });
    let ping_resp = ping_client.ping(&CodecLimits::defaults()).expect("ping");
    assert!(ping_resp.outcome().response().set_endpoint_role().is_none());
}

#[test]
fn set_endpoint_role_with_options_and_formatting() {
    let payload = test_structure([
        test_item(ENDPOINT_ROLE, kmipkit_ttlv::Value::enumeration(1)), // Client
    ]);
    let response = asynchronous_response_bytes(SET_ENDPOINT_ROLE, 0, None, None, Some(payload));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let req = SetEndpointRoleRequest::new(EndpointRole::CLIENT);
    let client_req = ClientRequest::set_endpoint_role(req.clone());
    assert!(format!("{client_req:?}").contains("SetEndpointRole"));

    let _batch_item = ClientBatchItem::set_endpoint_role(req.clone());

    let item_resp = client
        .set_endpoint_role_with_options(req, &CodecLimits::defaults(), &RequestOptions::default())
        .expect("set_endpoint_role_with_options succeeds");

    let outcome = item_resp.outcome();
    assert!(format!("{outcome:?}").contains("SetEndpointRole"));
    assert!(format!("{outcome}").contains("SetEndpointRole"));
}
