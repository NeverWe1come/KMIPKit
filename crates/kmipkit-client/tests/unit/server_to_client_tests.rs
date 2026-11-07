//! Unit tests for Server-to-Client operations and listener dispatcher.

use kmipkit_protocol::server_to_client::{
    ServerCapabilities, ServerDiscoverVersionsRequest, ServerDiscoverVersionsResponse,
    ServerNotifyRequest, ServerNotifyResponse, ServerPutRequest, ServerPutResponse,
    ServerQueryRequest, ServerQueryResponse,
};
use kmipkit_protocol::{
    EndpointRole, KmipOperationResult, ProtocolVersion, ResultReason, ResultStatus,
    SetEndpointRoleRequest, SetEndpointRoleResponse,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

use crate::server_to_client::{IncomingServerBatch, ServerRequestDispatcher, ServerRequestHandler};
use crate::{ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation};

const REQUEST_MESSAGE_TAG: u32 = 0x0042_0078;
const REQUEST_HEADER_TAG: u32 = 0x0042_0077;
const PROTOCOL_VERSION_TAG: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR_TAG: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR_TAG: u32 = 0x0042_006B;
const MAXIMUM_RESPONSE_SIZE_TAG: u32 = 0x0042_004F;
const BATCH_COUNT_TAG: u32 = 0x0042_000D;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const OPERATION_TAG: u32 = 0x0042_005C;
const UNIQUE_BATCH_ITEM_ID_TAG: u32 = 0x0042_0093;
const REQUEST_PAYLOAD_TAG: u32 = 0x0042_0079;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw).unwrap().try_checked().unwrap()
}

#[allow(clippy::too_many_lines)]
fn make_test_request_batch_ttlv() -> Vec<u8> {
    let mut header = Structure::new();
    let mut pv = Structure::new();
    pv.try_push(Item::new(tag(PROTOCOL_VERSION_MAJOR_TAG), Value::integer(2)).unwrap())
        .unwrap();
    pv.try_push(Item::new(tag(PROTOCOL_VERSION_MINOR_TAG), Value::integer(1)).unwrap())
        .unwrap();
    header
        .try_push(Item::new(tag(PROTOCOL_VERSION_TAG), Value::structure(pv)).unwrap())
        .unwrap();
    header
        .try_push(Item::new(tag(MAXIMUM_RESPONSE_SIZE_TAG), Value::integer(65536)).unwrap())
        .unwrap();
    header
        .try_push(Item::new(tag(BATCH_COUNT_TAG), Value::integer(5)).unwrap())
        .unwrap();

    let mut root = Structure::new();
    root.try_push(Item::new(tag(REQUEST_HEADER_TAG), Value::structure(header)).unwrap())
        .unwrap();

    // 1. Discover Versions
    let mut item1 = Structure::new();
    item1
        .try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x1E)).unwrap())
        .unwrap();
    item1
        .try_push(Item::new(tag(UNIQUE_BATCH_ITEM_ID_TAG), Value::byte_string(vec![1])).unwrap())
        .unwrap();
    let mut p1 = Structure::new();
    let mut req_pv = Structure::new();
    req_pv
        .try_push(Item::new(tag(PROTOCOL_VERSION_MAJOR_TAG), Value::integer(2)).unwrap())
        .unwrap();
    req_pv
        .try_push(Item::new(tag(PROTOCOL_VERSION_MINOR_TAG), Value::integer(1)).unwrap())
        .unwrap();
    p1.try_push(Item::new(tag(PROTOCOL_VERSION_TAG), Value::structure(req_pv)).unwrap())
        .unwrap();
    item1
        .try_push(Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(p1)).unwrap())
        .unwrap();
    root.try_push(Item::new(tag(BATCH_ITEM_TAG), Value::structure(item1)).unwrap())
        .unwrap();

    // 2. Notify
    let mut item2 = Structure::new();
    item2
        .try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x1B)).unwrap())
        .unwrap();
    item2
        .try_push(Item::new(tag(UNIQUE_BATCH_ITEM_ID_TAG), Value::byte_string(vec![2])).unwrap())
        .unwrap();
    let mut p2 = Structure::new();
    p2.try_push(
        Item::new(
            tag(0x0042_0094),
            Value::text_string("obj-notify-1".to_string()),
        )
        .unwrap(),
    )
    .unwrap();
    item2
        .try_push(Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(p2)).unwrap())
        .unwrap();
    root.try_push(Item::new(tag(BATCH_ITEM_TAG), Value::structure(item2)).unwrap())
        .unwrap();

    // 3. Put
    let mut item3 = Structure::new();
    item3
        .try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x1C)).unwrap())
        .unwrap();
    item3
        .try_push(Item::new(tag(UNIQUE_BATCH_ITEM_ID_TAG), Value::byte_string(vec![3])).unwrap())
        .unwrap();
    let mut p3 = Structure::new();
    p3.try_push(
        Item::new(
            tag(0x0042_0094),
            Value::text_string("obj-put-1".to_string()),
        )
        .unwrap(),
    )
    .unwrap();
    p3.try_push(Item::new(tag(0x0042_0088), Value::structure(Structure::new())).unwrap())
        .unwrap();
    item3
        .try_push(Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(p3)).unwrap())
        .unwrap();
    root.try_push(Item::new(tag(BATCH_ITEM_TAG), Value::structure(item3)).unwrap())
        .unwrap();

    // 4. Query
    let mut item4 = Structure::new();
    item4
        .try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x18)).unwrap())
        .unwrap();
    item4
        .try_push(Item::new(tag(UNIQUE_BATCH_ITEM_ID_TAG), Value::byte_string(vec![4])).unwrap())
        .unwrap();
    let mut p4 = Structure::new();
    p4.try_push(Item::new(tag(0x0042_007A), Value::enumeration(0x01)).unwrap())
        .unwrap(); // QueryFunction = QueryOperations
    item4
        .try_push(Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(p4)).unwrap())
        .unwrap();
    root.try_push(Item::new(tag(BATCH_ITEM_TAG), Value::structure(item4)).unwrap())
        .unwrap();

    // 5. Set Endpoint Role
    let mut item5 = Structure::new();
    item5
        .try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x32)).unwrap())
        .unwrap();
    item5
        .try_push(Item::new(tag(UNIQUE_BATCH_ITEM_ID_TAG), Value::byte_string(vec![5])).unwrap())
        .unwrap();
    let req5 = SetEndpointRoleRequest::new(EndpointRole::SERVER);
    let p5 = req5.to_ttlv_payload().unwrap();
    item5
        .try_push(Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(p5)).unwrap())
        .unwrap();
    root.try_push(Item::new(tag(BATCH_ITEM_TAG), Value::structure(item5)).unwrap())
        .unwrap();

    // Encode to TTLV bytes
    let item = Item::new(tag(REQUEST_MESSAGE_TAG), Value::structure(root)).unwrap();
    let mut out = Vec::new();
    encode_item(&item, &mut out);
    out
}

fn encode_item(item: &Item, out: &mut Vec<u8>) {
    let raw_tag = item.tag().raw().to_be_bytes();
    let tag_bytes = [raw_tag[1], raw_tag[2], raw_tag[3]];
    item.with_value(|val| match val {
        kmipkit_ttlv::ValueView::Structure(s) => {
            let mut children_bytes = Vec::new();
            for c in s.children() {
                encode_item(c, &mut children_bytes);
            }
            write_header(out, tag_bytes, 0x01, children_bytes.len());
            out.extend_from_slice(&children_bytes);
        }
        kmipkit_ttlv::ValueView::Integer(i) => {
            write_header(out, tag_bytes, 0x02, 4);
            out.extend_from_slice(&i.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]);
        }
        kmipkit_ttlv::ValueView::Enumeration(e) => {
            write_header(out, tag_bytes, 0x05, 4);
            out.extend_from_slice(&e.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]);
        }
        kmipkit_ttlv::ValueView::TextString(s) => {
            let b = s.as_bytes();
            write_header(out, tag_bytes, 0x07, b.len());
            out.extend_from_slice(b);
            let rem = b.len() % 8;
            if rem > 0 {
                out.extend(std::iter::repeat_n(0u8, 8 - rem));
            }
        }
        kmipkit_ttlv::ValueView::ByteString(b) => {
            write_header(out, tag_bytes, 0x08, b.len());
            out.extend_from_slice(b);
            let rem = b.len() % 8;
            if rem > 0 {
                out.extend(std::iter::repeat_n(0u8, 8 - rem));
            }
        }
        _ => {}
    });
}

fn write_header(out: &mut Vec<u8>, tag: [u8; 3], typ: u8, len: usize) {
    out.extend_from_slice(&tag);
    out.push(typ);
    out.extend_from_slice(&u32::try_from(len).unwrap_or(0).to_be_bytes());
}

#[allow(clippy::struct_excessive_bools)]
struct TestHandler {
    discover_versions_called: bool,
    notify_called: bool,
    put_called: bool,
    query_called: bool,
    set_endpoint_role_called: bool,
}

impl ServerRequestHandler for TestHandler {
    fn handle_discover_versions(
        &mut self,
        _request: &ServerDiscoverVersionsRequest,
    ) -> Result<ServerDiscoverVersionsResponse, KmipOperationResult> {
        self.discover_versions_called = true;
        Ok(ServerDiscoverVersionsResponse::new(vec![
            ProtocolVersion::from_raw(2, 1),
        ]))
    }

    fn handle_notify(
        &mut self,
        _request: &ServerNotifyRequest,
    ) -> Result<ServerNotifyResponse, KmipOperationResult> {
        self.notify_called = true;
        Ok(ServerNotifyResponse::new())
    }

    fn handle_put(
        &mut self,
        _request: &ServerPutRequest,
    ) -> Result<ServerPutResponse, KmipOperationResult> {
        self.put_called = true;
        Ok(ServerPutResponse::new())
    }

    fn handle_query(
        &mut self,
        request: &ServerQueryRequest,
    ) -> Result<ServerQueryResponse, KmipOperationResult> {
        self.query_called = true;
        let mut caps = ServerCapabilities::new();
        caps.set_operations(vec![0x18, 0x1E]);
        Ok(ServerQueryResponse::respond(request, &caps))
    }

    fn handle_set_endpoint_role(
        &mut self,
        _request: &SetEndpointRoleRequest,
    ) -> Result<SetEndpointRoleResponse, KmipOperationResult> {
        self.set_endpoint_role_called = true;
        Ok(SetEndpointRoleResponse::success(EndpointRole::SERVER, None))
    }
}

#[test]
fn incoming_server_batch_decoding_and_dispatch() {
    let ttlv = make_test_request_batch_ttlv();
    let limits = CodecLimits::defaults();

    let batch =
        IncomingServerBatch::from_ttlv_bytes(&ttlv, &limits).expect("batch should decode cleanly");
    assert_eq!(batch.protocol_version(), ProtocolVersion::from_raw(2, 1));
    assert_eq!(batch.batch_count(), 5);

    let items = batch.items();
    assert_eq!(items.len(), 5);

    assert_eq!(items[0].operation(), 0x1E);
    assert_eq!(items[0].unique_batch_item_id(), Some(&[1u8][..]));
    assert!(items[0].as_discover_versions().is_some());

    assert_eq!(items[1].operation(), 0x1B);
    assert_eq!(items[1].unique_batch_item_id(), Some(&[2u8][..]));
    assert!(items[1].as_notify().is_some());

    assert_eq!(items[2].operation(), 0x1C);
    assert_eq!(items[2].unique_batch_item_id(), Some(&[3u8][..]));
    assert!(items[2].as_put().is_some());

    assert_eq!(items[3].operation(), 0x18);
    assert_eq!(items[3].unique_batch_item_id(), Some(&[4u8][..]));
    assert!(items[3].as_query().is_some());

    assert_eq!(items[4].operation(), 0x32);
    assert_eq!(items[4].unique_batch_item_id(), Some(&[5u8][..]));
    assert!(items[4].as_set_endpoint_role().is_some());

    // Dispatch
    let mut handler = TestHandler {
        discover_versions_called: false,
        notify_called: false,
        put_called: false,
        query_called: false,
        set_endpoint_role_called: false,
    };
    let mut dispatcher = ServerRequestDispatcher::new();
    let response = dispatcher
        .dispatch(&batch, &mut handler)
        .expect("dispatch should succeed");

    assert!(handler.discover_versions_called);
    assert!(handler.notify_called);
    assert!(handler.put_called);
    assert!(handler.query_called);
    assert!(handler.set_endpoint_role_called);

    assert_eq!(response.header().batch_count(), 5);
}

struct FailingHandler;

impl ServerRequestHandler for FailingHandler {
    fn handle_discover_versions(
        &mut self,
        _request: &ServerDiscoverVersionsRequest,
    ) -> Result<ServerDiscoverVersionsResponse, KmipOperationResult> {
        Err(KmipOperationResult::new(
            ResultStatus::from_raw(1),
            Some(ResultReason::from_raw(0x100)),
            None,
        )
        .unwrap())
    }

    fn handle_notify(
        &mut self,
        _request: &ServerNotifyRequest,
    ) -> Result<ServerNotifyResponse, KmipOperationResult> {
        Err(KmipOperationResult::new(
            ResultStatus::from_raw(1),
            Some(ResultReason::from_raw(0x0C)),
            None,
        )
        .unwrap())
    }

    fn handle_put(
        &mut self,
        _request: &ServerPutRequest,
    ) -> Result<ServerPutResponse, KmipOperationResult> {
        Err(KmipOperationResult::new(
            ResultStatus::from_raw(1),
            Some(ResultReason::from_raw(0x01)),
            None,
        )
        .unwrap())
    }

    fn handle_query(
        &mut self,
        _request: &ServerQueryRequest,
    ) -> Result<ServerQueryResponse, KmipOperationResult> {
        Err(KmipOperationResult::new(
            ResultStatus::from_raw(1),
            Some(ResultReason::from_raw(0x05)),
            None,
        )
        .unwrap())
    }

    fn handle_set_endpoint_role(
        &mut self,
        _request: &SetEndpointRoleRequest,
    ) -> Result<SetEndpointRoleResponse, KmipOperationResult> {
        Err(KmipOperationResult::new(
            ResultStatus::from_raw(1),
            Some(ResultReason::from_raw(0x04)),
            None,
        )
        .unwrap())
    }
}

#[test]
fn dispatcher_handles_failures() {
    let ttlv = make_test_request_batch_ttlv();
    let limits = CodecLimits::defaults();
    let batch = IncomingServerBatch::from_ttlv_bytes(&ttlv, &limits).unwrap();

    let mut handler = FailingHandler;
    let mut dispatcher = ServerRequestDispatcher::new();
    let response = dispatcher
        .dispatch(&batch, &mut handler)
        .expect("dispatch should succeed");

    let items: Vec<_> = response.batch_items().collect();
    assert_eq!(items.len(), 5);
    for item in items {
        assert_eq!(item.result_status(), Some(ResultStatus::from_raw(1)));
    }
}

#[test]
fn set_endpoint_role_client_batch_item_and_outcome() {
    let req = SetEndpointRoleRequest::new(EndpointRole::SERVER);
    let item = ClientBatchItem::set_endpoint_role(req);
    let batch = ClientBatch::new(item);
    assert_eq!(batch.items().len(), 1);

    let resp = SetEndpointRoleResponse::success(EndpointRole::SERVER, None);
    let outcome = ClientBatchOutcome::SetEndpointRole(resp);
    assert_eq!(outcome.operation(), ClientOperation::SetEndpointRole);
    assert_eq!(outcome.result().status(), ResultStatus::from_raw(0));
    assert!(outcome.asynchronous_correlation_value().is_none());
    assert!(outcome.set_endpoint_role_response().is_some());
    assert_eq!(
        outcome
            .set_endpoint_role_response()
            .unwrap()
            .endpoint_role(),
        Some(EndpointRole::SERVER)
    );
    assert!(outcome.response().set_endpoint_role().is_some());
}

#[test]
fn malformed_incoming_batch_returns_error() {
    let limits = CodecLimits::defaults();
    assert!(IncomingServerBatch::from_ttlv_bytes(&[], &limits).is_err());
    assert!(IncomingServerBatch::from_ttlv_bytes(&[0x00, 0x01, 0x02], &limits).is_err());
}
