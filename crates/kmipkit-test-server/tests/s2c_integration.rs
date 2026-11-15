//! Integration tests for OASIS KMIP 2.1 Server-to-Client operations and Role Reversal.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::similar_names,
    clippy::doc_markdown,
    clippy::format_push_string,
    clippy::semicolon_if_nothing_returned,
    clippy::unreadable_literal,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::collapsible_if,
    clippy::items_after_statements,
    clippy::manual_is_multiple_of,
    clippy::return_self_not_must_use,
    clippy::unnecessary_wraps,
    clippy::vec_init_then_push,
    clippy::bool_to_int_with_if
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use kmipkit_client::Client;
use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::server_to_client::{
    ClientListener, ServerRequestDispatcher, ServerRequestHandler,
};
use kmipkit_protocol::server_to_client::{
    ServerCapabilities, ServerDiscoverVersionsRequest, ServerDiscoverVersionsResponse,
    ServerNotifyRequest, ServerNotifyResponse, ServerPutRequest, ServerPutResponse,
    ServerQueryRequest, ServerQueryResponse,
};
use kmipkit_protocol::{
    EndpointRole, KmipOperationResult, ProtocolVersion, SetEndpointRoleRequest,
    SetEndpointRoleResponse,
};
use kmipkit_test_server::codec::Format;
use kmipkit_test_server::{ServerConfig, TestServer};
use kmipkit_transport::listener::ListenerTransportConfig;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
    TransportConfig, TrustSource, WireEncoding,
};
use kmipkit_ttlv::codec::CodecLimits;

#[allow(clippy::struct_field_names)]
struct TrackingHandler {
    discover_versions_called: Arc<AtomicBool>,
    notify_called: Arc<AtomicBool>,
    put_called: Arc<AtomicBool>,
    query_called: Arc<AtomicBool>,
    set_endpoint_role_called: Arc<AtomicBool>,
}

impl ServerRequestHandler for TrackingHandler {
    fn handle_discover_versions(
        &mut self,
        _request: &ServerDiscoverVersionsRequest,
    ) -> Result<ServerDiscoverVersionsResponse, KmipOperationResult> {
        self.discover_versions_called.store(true, Ordering::SeqCst);
        Ok(ServerDiscoverVersionsResponse::new(vec![
            ProtocolVersion::from_raw(2, 1),
        ]))
    }

    fn handle_notify(
        &mut self,
        _request: &ServerNotifyRequest,
    ) -> Result<ServerNotifyResponse, KmipOperationResult> {
        self.notify_called.store(true, Ordering::SeqCst);
        Ok(ServerNotifyResponse::new())
    }

    fn handle_put(
        &mut self,
        _request: &ServerPutRequest,
    ) -> Result<ServerPutResponse, KmipOperationResult> {
        self.put_called.store(true, Ordering::SeqCst);
        Ok(ServerPutResponse::new())
    }

    fn handle_query(
        &mut self,
        request: &ServerQueryRequest,
    ) -> Result<ServerQueryResponse, KmipOperationResult> {
        self.query_called.store(true, Ordering::SeqCst);
        let mut caps = ServerCapabilities::new();
        caps.set_operations(vec![0x18, 0x1E]);
        Ok(ServerQueryResponse::respond(request, &caps))
    }

    fn handle_set_endpoint_role(
        &mut self,
        _request: &SetEndpointRoleRequest,
    ) -> Result<SetEndpointRoleResponse, KmipOperationResult> {
        self.set_endpoint_role_called.store(true, Ordering::SeqCst);
        Ok(SetEndpointRoleResponse::success(EndpointRole::SERVER, None))
    }
}

fn make_listener_config(pki: &kmipkit_test_server::AutoPki) -> ListenerTransportConfig {
    let identity = ClientIdentity::new(
        CertificateInput::from_der(vec![pki.server_cert_der.clone()]),
        PrivateKeyInput::from_der(pki.server_key_der.to_vec()),
    );
    let trust_source =
        TrustSource::certificate_authorities(vec![CertificateInput::from_der(vec![
            pki.ca_cert_der.clone(),
        ])]);

    ListenerTransportConfig::builder("127.0.0.1", 0)
        .listener_identity(identity)
        .trust_source(trust_source)
        .build()
        .expect("listener config should build")
}

fn build_client(
    endpoint: Endpoint,
    pki: &kmipkit_test_server::AutoPki,
    encoding: WireEncoding,
) -> Client {
    let certificates = vec![pki.client_cert_der.clone()];
    let private_key = pki.client_key_der.to_vec();

    let transport_config = TransportConfig::builder(endpoint)
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der(certificates),
            PrivateKeyInput::from_der(private_key),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.ca_cert_der.clone()]),
        ]))
        .timeouts(
            TimeoutPolicy::default().with_total(TimeoutLimit::Bounded(Duration::from_secs(5))),
        )
        .build()
        .expect("transport config builds cleanly");

    let registry = client_extension_registry(Vec::new(), kmipkit_protocol::extension::defaults())
        .expect("empty registry is valid");
    let client_config = ClientConfiguration::new(registry).with_wire_encoding(encoding);

    Client::new(client_config, transport_config).expect("client builds cleanly")
}

#[tokio::test]
async fn test_client_listener_raw_tls_receives_s2c_operations_from_server() {
    let server = TestServer::start(ServerConfig::default())
        .await
        .expect("server starts");
    let pki = server.pki();

    let listener_config = make_listener_config(pki);
    let mut listener = ClientListener::bind_raw_tls(listener_config).expect("listener binds");
    let listener_addr = listener.local_addr().expect("listener addr");

    let discover_versions_called = Arc::new(AtomicBool::new(false));
    let notify_called = Arc::new(AtomicBool::new(false));
    let query_called = Arc::new(AtomicBool::new(false));

    let dv_flag = Arc::clone(&discover_versions_called);
    let notif_flag = Arc::clone(&notify_called);
    let q_flag = Arc::clone(&query_called);

    let listener_task = tokio::task::spawn_blocking(move || {
        let limits = CodecLimits::defaults();
        let timeout = Some(Duration::from_secs(5));

        // Handle 3 incoming connections for the 3 S2C operations
        for _ in 0..3 {
            let mut conn = listener.accept(timeout).expect("accept connection");
            let batch = conn.recv_batch(&limits, timeout).expect("recv batch");

            let mut handler = TrackingHandler {
                discover_versions_called: Arc::clone(&dv_flag),
                notify_called: Arc::clone(&notif_flag),
                put_called: Arc::new(AtomicBool::new(false)),
                query_called: Arc::clone(&q_flag),
                set_endpoint_role_called: Arc::new(AtomicBool::new(false)),
            };
            let mut dispatcher = ServerRequestDispatcher::new();
            let response = dispatcher.dispatch(&batch, &mut handler).expect("dispatch");
            conn.send_response(&response, timeout)
                .expect("send response");
        }
    });

    // Server triggers S2C operations to the client listener
    let dv_resp = server
        .trigger_s2c_discover_versions(listener_addr, &[(2, 1)])
        .await
        .expect("trigger discover versions");
    assert_eq!(dv_resp.tag().raw(), 0x0042_007B);

    let notif_resp = server
        .trigger_s2c_notify(listener_addr, "obj-push-123")
        .await
        .expect("trigger notify");
    assert_eq!(notif_resp.tag().raw(), 0x0042_007B);

    let query_resp = server
        .trigger_s2c_query(listener_addr, &[0x01])
        .await
        .expect("trigger query");
    assert_eq!(query_resp.tag().raw(), 0x0042_007B);

    listener_task.await.expect("listener task finished");

    assert!(discover_versions_called.load(Ordering::SeqCst));
    assert!(notify_called.load(Ordering::SeqCst));
    assert!(query_called.load(Ordering::SeqCst));

    server.shutdown().await.expect("server clean shutdown");
}

#[tokio::test]
async fn test_client_listener_https_receives_s2c_operations_from_server() {
    let server = TestServer::start(ServerConfig::default())
        .await
        .expect("server starts");
    let pki = server.pki();

    let listener_config = make_listener_config(pki);
    let mut listener = ClientListener::bind_https(listener_config).expect("listener binds");
    let listener_addr = listener.local_addr().expect("listener addr");

    let notify_called = Arc::new(AtomicBool::new(false));
    let notif_flag = Arc::clone(&notify_called);

    let listener_task = tokio::task::spawn_blocking(move || {
        let limits = CodecLimits::defaults();
        let timeout = Some(Duration::from_secs(5));

        for _ in 0..3 {
            let mut conn = listener.accept(timeout).expect("accept connection");
            let batch = conn.recv_batch(&limits, timeout).expect("recv batch");

            let mut handler = TrackingHandler {
                discover_versions_called: Arc::new(AtomicBool::new(false)),
                notify_called: Arc::clone(&notif_flag),
                put_called: Arc::new(AtomicBool::new(false)),
                query_called: Arc::new(AtomicBool::new(false)),
                set_endpoint_role_called: Arc::new(AtomicBool::new(false)),
            };
            let mut dispatcher = ServerRequestDispatcher::new();
            let response = dispatcher.dispatch(&batch, &mut handler).expect("dispatch");
            conn.send_response(&response, timeout)
                .expect("send response");
        }
    });

    // Test TTLV over HTTPS (KMIP Profiles v2.1 §5.3.1)
    for _ in 0..3 {
        let resp = server
            .trigger_s2c_notify_https(listener_addr, "obj-https-notify", Format::Ttlv)
            .await
            .expect("trigger notify https");
        assert_eq!(resp.tag().raw(), 0x0042_007B);
    }

    listener_task.await.expect("listener task finished");
    assert!(notify_called.load(Ordering::SeqCst));

    server.shutdown().await.expect("server clean shutdown");
}

#[tokio::test]
async fn test_role_reversal_client_sets_role_and_receives_server_push() {
    let server = TestServer::start(ServerConfig::default())
        .await
        .expect("server starts");
    let pki = server.pki();

    let endpoint = Endpoint::raw_tls("127.0.0.1", server.port());
    let mut client = build_client(endpoint, pki, WireEncoding::Ttlv);

    let item_resp = tokio::task::spawn_blocking(move || {
        let limits = CodecLimits::defaults();
        let set_role_req = SetEndpointRoleRequest::new(EndpointRole::SERVER);
        client
            .set_endpoint_role(set_role_req, &limits)
            .expect("set_endpoint_role request should succeed")
    })
    .await
    .unwrap();

    assert_eq!(
        item_resp.outcome().operation(),
        kmipkit_client::ClientOperation::SetEndpointRole
    );
    assert_eq!(
        item_resp.set_endpoint_role().unwrap().endpoint_role(),
        Some(EndpointRole::SERVER)
    );

    // Also test RoleReversedSession through ClientListener
    let listener_config = make_listener_config(pki);
    let mut listener = ClientListener::bind_raw_tls(listener_config).expect("listener binds");
    let listener_addr = listener.local_addr().expect("listener addr");

    let session_task = tokio::task::spawn_blocking(move || {
        let timeout = Some(Duration::from_secs(5));
        let limits = CodecLimits::defaults();
        let conn = listener.accept(timeout).expect("accept connection");
        let mut session = conn.into_role_reversed().expect("into role reversed");

        let batch = session.recv_batch(&limits, timeout).expect("recv batch");
        assert_eq!(batch.items().len(), 1);

        let mut handler = TrackingHandler {
            discover_versions_called: Arc::new(AtomicBool::new(false)),
            notify_called: Arc::new(AtomicBool::new(false)),
            put_called: Arc::new(AtomicBool::new(false)),
            query_called: Arc::new(AtomicBool::new(false)),
            set_endpoint_role_called: Arc::new(AtomicBool::new(false)),
        };
        let mut dispatcher = ServerRequestDispatcher::new();
        let response = dispatcher.dispatch(&batch, &mut handler).expect("dispatch");
        session
            .send_response(&response, timeout)
            .expect("send response");
    });

    let s2c = kmipkit_test_server::S2cClient::new(pki).expect("s2c client");
    let notify_req = kmipkit_test_server::build_s2c_notify("reversed-item-42");
    let resp = s2c
        .send_raw_ttlv(listener_addr, &notify_req)
        .await
        .expect("send notify over reversed stream");
    assert_eq!(resp.tag().raw(), 0x0042_007B);

    session_task.await.expect("session task completes");
    server.shutdown().await.expect("server clean shutdown");
}
