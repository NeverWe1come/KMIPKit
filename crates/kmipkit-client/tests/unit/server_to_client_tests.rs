//! Unit tests for Server-to-Client operations and listener dispatcher.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

#[test]
fn server_request_item_payload_debug_formatting() {
    use crate::server_to_client::ServerRequestItemPayload;
    use kmipkit_protocol::QueryFunction;
    use kmipkit_protocol::server_to_client::PutFunction;

    let disc =
        ServerRequestItemPayload::DiscoverVersions(ServerDiscoverVersionsRequest::new(vec![
            ProtocolVersion::from_raw(2, 1),
        ]));
    assert!(format!("{disc:?}").contains("DiscoverVersions"));

    let notif = ServerRequestItemPayload::Notify(ServerNotifyRequest::new(
        "id-1".to_string(),
        None,
        Vec::new(),
    ));
    assert!(format!("{notif:?}").contains("Notify"));

    let put_item = Item::new(tag(0x0042_000B), Value::structure(Structure::new())).unwrap();
    let put = ServerPutRequest::new(PutFunction::NEW, "id-2".to_string(), None, put_item, None);
    let put_payload = ServerRequestItemPayload::Put(put);
    assert!(format!("{put_payload:?}").contains("Put"));

    let query =
        ServerRequestItemPayload::Query(ServerQueryRequest::new(vec![QueryFunction::OPERATIONS]));
    assert!(format!("{query:?}").contains("Query"));

    let role = ServerRequestItemPayload::SetEndpointRole(SetEndpointRoleRequest::new(
        EndpointRole::SERVER,
    ));
    assert!(format!("{role:?}").contains("SetEndpointRole"));

    let unk = ServerRequestItemPayload::Unknown { operation: 0x9999 };
    assert!(format!("{unk:?}").contains("Unknown"));
}

#[test]
fn server_request_item_views_none_branches() {
    let ttlv = make_test_request_batch_ttlv();
    let limits = CodecLimits::defaults();
    let batch = IncomingServerBatch::from_ttlv_bytes(&ttlv, &limits).unwrap();
    let items = batch.items();

    assert!(items[0].as_notify().is_none());
    assert!(items[0].as_put().is_none());
    assert!(items[0].as_query().is_none());
    assert!(items[0].as_set_endpoint_role().is_none());

    assert!(items[1].as_discover_versions().is_none());
    assert!(items[1].as_put().is_none());
    assert!(items[1].as_query().is_none());
    assert!(items[1].as_set_endpoint_role().is_none());

    assert!(items[2].as_discover_versions().is_none());
    assert!(items[2].as_notify().is_none());
    assert!(items[2].as_query().is_none());
    assert!(items[2].as_set_endpoint_role().is_none());

    assert!(items[3].as_discover_versions().is_none());
    assert!(items[3].as_notify().is_none());
    assert!(items[3].as_put().is_none());
    assert!(items[3].as_set_endpoint_role().is_none());

    assert!(items[4].as_discover_versions().is_none());
    assert!(items[4].as_notify().is_none());
    assert!(items[4].as_put().is_none());
    assert!(items[4].as_query().is_none());
}

#[test]
fn dispatcher_unknown_operation_handling() {
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
        .try_push(Item::new(tag(BATCH_COUNT_TAG), Value::integer(1)).unwrap())
        .unwrap();

    let mut root = Structure::new();
    root.try_push(Item::new(tag(REQUEST_HEADER_TAG), Value::structure(header)).unwrap())
        .unwrap();

    // Unknown operation
    let mut item = Structure::new();
    item.try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x9999)).unwrap())
        .unwrap();
    item.try_push(Item::new(tag(UNIQUE_BATCH_ITEM_ID_TAG), Value::byte_string(vec![99])).unwrap())
        .unwrap();
    let p = Structure::new();
    item.try_push(Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(p)).unwrap())
        .unwrap();
    root.try_push(Item::new(tag(BATCH_ITEM_TAG), Value::structure(item)).unwrap())
        .unwrap();

    let msg = Item::new(tag(REQUEST_MESSAGE_TAG), Value::structure(root)).unwrap();
    let mut ttlv = Vec::new();
    encode_item(&msg, &mut ttlv);

    let limits = CodecLimits::defaults();
    let batch = IncomingServerBatch::from_ttlv_bytes(&ttlv, &limits).unwrap();
    let items = batch.items();
    assert_eq!(items[0].operation(), 0x9999);

    let mut handler = TestHandler {
        discover_versions_called: false,
        notify_called: false,
        put_called: false,
        query_called: false,
        set_endpoint_role_called: false,
    };
    let mut dispatcher = ServerRequestDispatcher::new();
    let response = dispatcher.dispatch(&batch, &mut handler).unwrap();
    let res_items: Vec<_> = response.batch_items().collect();
    assert_eq!(res_items.len(), 1);
    assert_eq!(
        res_items[0].result_status(),
        Some(ResultStatus::from_raw(1))
    );
    assert_eq!(
        res_items[0].result_reason(),
        Some(ResultReason::from_raw(0x05))
    );
}

#[test]
fn incoming_server_batch_missing_operation_tag() {
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
        .try_push(Item::new(tag(BATCH_COUNT_TAG), Value::integer(1)).unwrap())
        .unwrap();

    let mut root = Structure::new();
    root.try_push(Item::new(tag(REQUEST_HEADER_TAG), Value::structure(header)).unwrap())
        .unwrap();

    // Batch item with NO operation tag
    let mut item = Structure::new();
    item.try_push(Item::new(tag(UNIQUE_BATCH_ITEM_ID_TAG), Value::byte_string(vec![1])).unwrap())
        .unwrap();
    root.try_push(Item::new(tag(BATCH_ITEM_TAG), Value::structure(item)).unwrap())
        .unwrap();

    let msg = Item::new(tag(REQUEST_MESSAGE_TAG), Value::structure(root)).unwrap();
    let mut ttlv = Vec::new();
    encode_item(&msg, &mut ttlv);

    let limits = CodecLimits::defaults();
    let err = IncomingServerBatch::from_ttlv_bytes(&ttlv, &limits);
    assert!(err.is_err());
}

#[test]
fn client_listener_bind_and_timeout() {
    use crate::server_to_client::ClientListener;
    use kmipkit_test_support::EphemeralPki;
    use kmipkit_transport::listener::ListenerTransportConfig;
    use kmipkit_transport::{CertificateInput, ClientIdentity, PrivateKeyInput, TrustSource};
    use std::time::Duration;

    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let server = pki.server_identity();

    let make_identity = || {
        ClientIdentity::new(
            CertificateInput::from_der(vec![server.certificate_der().to_vec()]),
            PrivateKeyInput::from_der(server.private_key_der().to_vec()),
        )
    };
    let make_trust = || {
        TrustSource::certificate_authorities(vec![CertificateInput::from_der(vec![
            pki.authority_certificate_der().to_vec(),
        ])])
    };

    let config_raw = ListenerTransportConfig::builder("127.0.0.1", 0)
        .listener_identity(make_identity())
        .trust_source(make_trust())
        .build()
        .expect("valid config");

    let mut raw_listener = ClientListener::bind_raw_tls(config_raw).expect("bind raw succeeds");
    assert!(raw_listener.local_addr().is_ok());
    assert!(raw_listener.port().is_ok());

    let err1 = raw_listener.accept(Some(Duration::from_millis(10)));
    assert!(err1.is_err());

    let config_https = ListenerTransportConfig::builder("127.0.0.1", 0)
        .listener_identity(make_identity())
        .trust_source(make_trust())
        .target_path("/kmip")
        .build()
        .expect("valid config");

    let mut https_listener = ClientListener::bind_https(config_https).expect("bind https succeeds");
    assert!(https_listener.local_addr().is_ok());
    assert!(https_listener.port().is_ok());

    let err2 = https_listener.accept(Some(Duration::from_millis(10)));
    assert!(err2.is_err());
}

fn client_tls_config(pki: &kmipkit_test_support::EphemeralPki) -> Arc<rustls::ClientConfig> {
    use rustls::RootCertStore;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

    let client = pki.client_identity();
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("authority cert is valid");

    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("TLS 1.3 only")
        .with_root_certificates(roots)
        .with_client_auth_cert(
            vec![CertificateDer::from(client.certificate_der().to_vec())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(client.private_key_der().to_vec())),
        )
        .expect("valid client identity");

    Arc::new(config)
}

#[test]
fn client_listener_raw_tls_full_exchange_and_role_reversal() {
    use crate::server_to_client::ClientListener;
    use kmipkit_test_support::EphemeralPki;
    use kmipkit_transport::listener::ListenerTransportConfig;
    use kmipkit_transport::{CertificateInput, ClientIdentity, PrivateKeyInput, TrustSource};

    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let server = pki.server_identity();
    let config = ListenerTransportConfig::builder("127.0.0.1", 0)
        .listener_identity(ClientIdentity::new(
            CertificateInput::from_der(vec![server.certificate_der().to_vec()]),
            PrivateKeyInput::from_der(server.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .build()
        .expect("config builds");

    let mut listener = ClientListener::bind_raw_tls(config).expect("bind succeeds");
    let port = listener.port().expect("port");
    let client_config = client_tls_config(&pki);
    let ttlv_req = make_test_request_batch_ttlv();
    let ttlv_req_clone = ttlv_req.clone();

    let client_handle = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect succeeds");
        let server_name =
            rustls::pki_types::ServerName::try_from("server.kmipkit.test").expect("valid dns name");
        let client_conn = rustls::ClientConnection::new(client_config, server_name)
            .expect("client conn succeeds");
        let mut tls = rustls::StreamOwned::new(client_conn, sock);

        // 1. Send first TTLV request batch
        tls.write_all(&ttlv_req_clone).expect("write batch 1");
        tls.flush().expect("flush");

        // Read response
        let mut resp_header = [0_u8; 8];
        tls.read_exact(&mut resp_header).expect("read resp header");
        let payload_len = u32::from_be_bytes([
            resp_header[4],
            resp_header[5],
            resp_header[6],
            resp_header[7],
        ]) as usize;
        let padding = (8 - (payload_len % 8)) % 8;
        let mut resp_body = vec![0_u8; payload_len + padding];
        tls.read_exact(&mut resp_body).expect("read resp body");

        // 2. In role-reversed mode, send a second request batch
        tls.write_all(&ttlv_req_clone)
            .expect("write batch 2 in role-reversal");
        tls.flush().expect("flush");

        // Read response from role-reversed session
        tls.read_exact(&mut resp_header)
            .expect("read resp header 2");
        let payload_len2 = u32::from_be_bytes([
            resp_header[4],
            resp_header[5],
            resp_header[6],
            resp_header[7],
        ]) as usize;
        let padding2 = (8 - (payload_len2 % 8)) % 8;
        let mut resp_body2 = vec![0_u8; payload_len2 + padding2];
        tls.read_exact(&mut resp_body2).expect("read resp body 2");
    });

    let limits = CodecLimits::defaults();
    let mut conn = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");
    assert!(conn.peer_addr().ip().is_loopback());

    // 1. Recv first batch and send response
    let batch = conn
        .recv_batch(&limits, Some(Duration::from_secs(5)))
        .expect("recv batch 1");
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
        .expect("dispatch 1");
    conn.send_response(&response, Some(Duration::from_secs(5)))
        .expect("send response 1");

    // 2. Transition to role-reversed session
    let mut session = conn
        .into_role_reversed()
        .expect("into_role_reversed succeeds");
    let batch2 = session
        .recv_batch(&limits, Some(Duration::from_secs(5)))
        .expect("recv batch 2");
    let response2 = dispatcher
        .dispatch(&batch2, &mut handler)
        .expect("dispatch 2");
    session
        .send_response(&response2, Some(Duration::from_secs(5)))
        .expect("send response 2");
    session.close().expect("close session");

    client_handle
        .join()
        .expect("client thread finishes cleanly");
}

#[test]
fn client_listener_https_full_exchange_and_rejections() {
    use crate::server_to_client::ClientListener;
    use kmipkit_test_support::EphemeralPki;
    use kmipkit_transport::listener::ListenerTransportConfig;
    use kmipkit_transport::{CertificateInput, ClientIdentity, PrivateKeyInput, TrustSource};

    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let server = pki.server_identity();
    let config = ListenerTransportConfig::builder("127.0.0.1", 0)
        .listener_identity(ClientIdentity::new(
            CertificateInput::from_der(vec![server.certificate_der().to_vec()]),
            PrivateKeyInput::from_der(server.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .target_path("/kmip")
        .build()
        .expect("config builds");

    let mut listener = ClientListener::bind_https(config).expect("bind succeeds");
    let port = listener.port().expect("port");
    let client_config = client_tls_config(&pki);
    let ttlv_req = make_test_request_batch_ttlv();

    let client_handle = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect succeeds");
        let server_name =
            rustls::pki_types::ServerName::try_from("server.kmipkit.test").expect("valid dns name");
        let client_conn = rustls::ClientConnection::new(client_config, server_name)
            .expect("client conn succeeds");
        let mut tls = rustls::StreamOwned::new(client_conn, sock);

        let http_req = format!(
            "POST /kmip HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
            ttlv_req.len()
        );
        tls.write_all(http_req.as_bytes()).expect("write headers");
        tls.write_all(&ttlv_req).expect("write body");
        tls.flush().expect("flush");

        let mut resp_buf = Vec::new();
        let mut chunk = [0_u8; 512];
        while !resp_buf.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = tls.read(&mut chunk).expect("read resp headers");
            if n == 0 {
                break;
            }
            resp_buf.extend_from_slice(&chunk[..n]);
        }
        let s = String::from_utf8_lossy(&resp_buf);
        assert!(s.contains("HTTP/1.1 200 OK"));
    });

    let limits = CodecLimits::defaults();
    let mut conn = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");
    assert!(conn.peer_addr().ip().is_loopback());
    let batch = conn
        .recv_batch(&limits, Some(Duration::from_secs(5)))
        .expect("recv batch");
    let mut handler = TestHandler {
        discover_versions_called: false,
        notify_called: false,
        put_called: false,
        query_called: false,
        set_endpoint_role_called: false,
    };
    let mut dispatcher = ServerRequestDispatcher::new();
    let response = dispatcher.dispatch(&batch, &mut handler).expect("dispatch");
    conn.send_response(&response, Some(Duration::from_secs(5)))
        .expect("send response");

    // HTTPS connection CANNOT be converted into role-reversed session
    let err = conn.into_role_reversed();
    assert!(err.is_err());

    client_handle.join().expect("client finishes");
}

#[test]
fn incoming_server_batch_json_xml_and_unsupported() {
    let limits = CodecLimits::defaults();

    // 1. JSON decoding
    let json_bytes = br#"{"tag":"RequestMessage","type":"Structure","value":[{"tag":"RequestHeader","type":"Structure","value":[{"tag":"ProtocolVersion","type":"Structure","value":[{"tag":"ProtocolVersionMajor","type":"Integer","value":2},{"tag":"ProtocolVersionMinor","type":"Integer","value":1}]},{"tag":"MaximumResponseSize","type":"Integer","value":65536},{"tag":"BatchCount","type":"Integer","value":1}]},{"tag":"BatchItem","type":"Structure","value":[{"tag":"Operation","type":"Enumeration","value":"DiscoverVersions"},{"tag":"RequestPayload","type":"Structure","value":[{"tag":"ProtocolVersion","type":"Structure","value":[{"tag":"ProtocolVersionMajor","type":"Integer","value":2},{"tag":"ProtocolVersionMinor","type":"Integer","value":1}]}]}]}]}"#;
    let batch_json = IncomingServerBatch::from_encoded_bytes(
        json_bytes,
        kmipkit_transport::WireEncoding::Json,
        &limits,
    )
    .expect("json batch decodes");
    assert_eq!(batch_json.batch_count(), 1);

    // 2. XML decoding
    let xml_bytes = br#"<RequestMessage><RequestHeader><ProtocolVersion><ProtocolVersionMajor type="Integer" value="2"/><ProtocolVersionMinor type="Integer" value="1"/></ProtocolVersion><MaximumResponseSize type="Integer" value="256"/><BatchCount type="Integer" value="1"/></RequestHeader><BatchItem><Operation type="Enumeration" value="DiscoverVersions"/><RequestPayload><ProtocolVersion><ProtocolVersionMajor type="Integer" value="2"/><ProtocolVersionMinor type="Integer" value="1"/></ProtocolVersion></RequestPayload></BatchItem></RequestMessage>"#;
    let batch_xml = IncomingServerBatch::from_encoded_bytes(
        xml_bytes,
        kmipkit_transport::WireEncoding::Xml,
        &limits,
    )
    .expect("xml batch decodes");
    assert_eq!(batch_xml.batch_count(), 1);

    // 3. Malformed JSON
    let err_json = IncomingServerBatch::from_encoded_bytes(
        b"not-valid-json",
        kmipkit_transport::WireEncoding::Json,
        &limits,
    );
    assert!(err_json.is_err());

    // 4. Malformed XML
    let err_xml = IncomingServerBatch::from_encoded_bytes(
        b"<not-closed-xml",
        kmipkit_transport::WireEncoding::Xml,
        &limits,
    );
    assert!(err_xml.is_err());
}

#[test]
fn client_listener_connection_close_paths() {
    use crate::server_to_client::ClientListener;
    use kmipkit_test_support::EphemeralPki;
    use kmipkit_transport::listener::ListenerTransportConfig;
    use kmipkit_transport::{CertificateInput, ClientIdentity, PrivateKeyInput, TrustSource};

    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let server = pki.server_identity();
    let config = ListenerTransportConfig::builder("127.0.0.1", 0)
        .listener_identity(ClientIdentity::new(
            CertificateInput::from_der(vec![server.certificate_der().to_vec()]),
            PrivateKeyInput::from_der(server.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .build()
        .expect("config builds");

    let mut listener = ClientListener::bind_raw_tls(config).expect("bind succeeds");
    let port = listener.port().expect("port");
    let client_config = client_tls_config(&pki);

    let client_handle = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect succeeds");
        let server_name =
            rustls::pki_types::ServerName::try_from("server.kmipkit.test").expect("valid dns name");
        let client_conn = rustls::ClientConnection::new(client_config, server_name)
            .expect("client conn succeeds");
        let mut tls = rustls::StreamOwned::new(client_conn, sock);
        let req = [0x42, 0x00, 0x77, 0x01, 0, 0, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8];
        let _ = tls.write_all(&req);
        let _ = tls.flush();
    });

    let mut conn = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");
    conn.close().expect("close succeeds");
    client_handle.join().expect("client finishes");

    let cfg_http = ListenerTransportConfig::builder("127.0.0.1", 0)
        .listener_identity(ClientIdentity::new(
            CertificateInput::from_der(vec![server.certificate_der().to_vec()]),
            PrivateKeyInput::from_der(server.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .build()
        .expect("config builds");
    let mut http_listener = ClientListener::bind_https(cfg_http).expect("bind succeeds");
    let http_port = http_listener.port().expect("port");
    let client_config2 = client_tls_config(&pki);
    let client_handle2 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", http_port)).expect("connect succeeds");
        let server_name =
            rustls::pki_types::ServerName::try_from("server.kmipkit.test").expect("valid dns name");
        let client_conn = rustls::ClientConnection::new(client_config2, server_name)
            .expect("client conn succeeds");
        let mut tls = rustls::StreamOwned::new(client_conn, sock);
        let _ = tls.write_all(b"GET / HTTP/1.1\r\n\r\n");
    });
    let mut http_conn = http_listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept");
    http_conn.close().expect("https close succeeds");
    client_handle2.join().expect("client finishes");
}

#[test]
fn encode_structure_view_scalar_types() {
    use kmipkit_protocol::server_to_client::ServerResponseBuilder;

    let mut builder = ServerResponseBuilder::new();
    let mut payload = Structure::new();
    payload
        .try_push(Item::new(tag(0x0042_0001), Value::long_integer(1_234_567_890_123)).unwrap())
        .unwrap();
    payload
        .try_push(Item::new(tag(0x0042_0002), Value::big_integer(vec![1, 2, 3, 4, 5])).unwrap())
        .unwrap();
    payload
        .try_push(Item::new(tag(0x0042_0003), Value::boolean(true)).unwrap())
        .unwrap();
    payload
        .try_push(Item::new(tag(0x0042_0004), Value::date_time(1_700_000_000)).unwrap())
        .unwrap();
    payload
        .try_push(Item::new(tag(0x0042_0005), Value::interval(3600)).unwrap())
        .unwrap();
    payload
        .try_push(
            Item::new(
                tag(0x0042_0006),
                Value::text_string("padding-test".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

    builder
        .add_success_item(0x1E, Some(vec![1, 2, 3]), payload)
        .expect("add success");
    let resp = builder.build().expect("build resp");

    // Verify round-trip encoding through with_ttlv
    let mut bytes = Vec::new();
    resp.with_ttlv(|view| {
        crate::server_to_client::encode_structure_view(0x0042_007B, &view, &mut bytes);
    });
    assert_ne!(bytes, [] as [u8; 0]);

    // Test DateTimeExtended triggers wildcard _ => {} in encode_structure_view
    let mut payload2 = Structure::new();
    payload2
        .try_push(Item::new(tag(0x0042_0007), Value::date_time_extended(999_999)).unwrap())
        .unwrap();
    let mut builder2 = ServerResponseBuilder::new();
    builder2
        .add_success_item(0x1E, None, payload2)
        .expect("add");
    let resp2 = builder2.build().expect("build");
    let mut bytes2 = Vec::new();
    resp2.with_ttlv(|view| {
        crate::server_to_client::encode_structure_view(0x0042_007B, &view, &mut bytes2);
    });
    assert_ne!(bytes2, [] as [u8; 0]);
}

fn make_custom_batch_with_item(build_item: fn(&mut Structure)) -> Vec<u8> {
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
        .try_push(Item::new(tag(BATCH_COUNT_TAG), Value::integer(1)).unwrap())
        .unwrap();

    let mut root = Structure::new();
    root.try_push(Item::new(tag(REQUEST_HEADER_TAG), Value::structure(header)).unwrap())
        .unwrap();

    let mut item = Structure::new();
    build_item(&mut item);
    root.try_push(Item::new(tag(BATCH_ITEM_TAG), Value::structure(item)).unwrap())
        .unwrap();

    let root_item = Item::new(tag(REQUEST_MESSAGE_TAG), Value::structure(root)).unwrap();
    let mut bytes = Vec::new();
    encode_item(&root_item, &mut bytes);
    bytes
}

#[test]
fn incoming_server_batch_item_decoding_error_branches_part1() {
    let limits = CodecLimits::defaults();

    // 1. DiscoverVersions with empty protocol versions list
    let bytes = make_custom_batch_with_item(|item| {
        item.try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x1E)).unwrap())
            .unwrap();
        let payload = Structure::new();
        item.try_push(Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(payload)).unwrap())
            .unwrap();
    });
    let batch = IncomingServerBatch::from_ttlv_bytes(&bytes, &limits).unwrap();
    assert_eq!(batch.batch_count(), 1);

    // 1. DiscoverVersions with invalid payload structure
    let bytes_dv = make_custom_batch_with_item(|item| {
        item.try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x1E)).unwrap())
            .unwrap();
        let mut payload = Structure::new();
        let mut pv = Structure::new();
        pv.try_push(
            Item::new(
                tag(PROTOCOL_VERSION_MAJOR_TAG),
                Value::text_string("bad".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        payload
            .try_push(Item::new(tag(PROTOCOL_VERSION_TAG), Value::structure(pv)).unwrap())
            .unwrap();
        item.try_push(Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(payload)).unwrap())
            .unwrap();
    });
    assert!(IncomingServerBatch::from_ttlv_bytes(&bytes_dv, &limits).is_err());

    // 2. Notify with empty payload (missing required Unique Identifier)
    let bytes_notify = make_custom_batch_with_item(|item| {
        item.try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x1B)).unwrap())
            .unwrap();
        item.try_push(
            Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(Structure::new())).unwrap(),
        )
        .unwrap();
    });
    assert!(IncomingServerBatch::from_ttlv_bytes(&bytes_notify, &limits).is_err());
}

#[test]
fn incoming_server_batch_item_decoding_error_branches_part2() {
    let limits = CodecLimits::defaults();

    // 3. Put with empty payload (missing required Unique Identifier)
    let bytes_put = make_custom_batch_with_item(|item| {
        item.try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x1C)).unwrap())
            .unwrap();
        item.try_push(
            Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(Structure::new())).unwrap(),
        )
        .unwrap();
    });
    assert!(IncomingServerBatch::from_ttlv_bytes(&bytes_put, &limits).is_err());

    // 4. Query with empty payload (missing query functions)
    let bytes_query = make_custom_batch_with_item(|item| {
        item.try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x18)).unwrap())
            .unwrap();
        item.try_push(
            Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(Structure::new())).unwrap(),
        )
        .unwrap();
    });
    let _ = IncomingServerBatch::from_ttlv_bytes(&bytes_query, &limits);

    // 5. SetEndpointRole with empty payload (missing required Endpoint Role)
    let bytes_role = make_custom_batch_with_item(|item| {
        item.try_push(Item::new(tag(OPERATION_TAG), Value::enumeration(0x32)).unwrap())
            .unwrap();
        item.try_push(
            Item::new(tag(REQUEST_PAYLOAD_TAG), Value::structure(Structure::new())).unwrap(),
        )
        .unwrap();
    });
    assert!(IncomingServerBatch::from_ttlv_bytes(&bytes_role, &limits).is_err());

    // 6. Missing operation tag completely
    let bytes = make_custom_batch_with_item(|item| {
        item.try_push(
            Item::new(tag(UNIQUE_BATCH_ITEM_ID_TAG), Value::byte_string(vec![1])).unwrap(),
        )
        .unwrap();
    });
    assert!(IncomingServerBatch::from_ttlv_bytes(&bytes, &limits).is_err());

    // 7. Non-structure root TTLV (Integer instead of Structure)
    let non_struct = Item::new(tag(0x0042_0077), Value::integer(1234)).unwrap();
    let mut non_struct_bytes = Vec::new();
    encode_item(&non_struct, &mut non_struct_bytes);
    assert!(IncomingServerBatch::from_ttlv_bytes(&non_struct_bytes, &limits).is_err());
}
