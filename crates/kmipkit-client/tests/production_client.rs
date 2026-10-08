//! Public production-client contract for KMIPKIT-0013 User Story 4.
//!
//! The loopback peers require verified TLS 1.3 mutual authentication. Client
//! construction is deliberately performed only from immutable client and
//! validated transport configuration; direct adapter cases exercise their
//! bounded byte API separately.
//!
//! The typed Discover Versions exchange follows OASIS KMIP Specification
//! v2.1 §6.1.16, Tables 211–212. The TLS profile context is OASIS KMIP
//! Profiles v2.1 Operating System Profile §5.3.1 items 3–4. Direct adapter
//! tests use opaque TTLV frames and make no profile-conformance claim. The
//! Message Extension structure follows OASIS KMIP Specification v2.1 §9.13,
//! Table 418. Response roots use Response Message (`0x42007B`) under OASIS
//! KMIP Specification v2.1 §8.4 Table 397 and §11.56.
//! Request and response boundaries additionally trace to OASIS KMIP
//! Specification v2.1 §9.12, Table 417, and §§10.1.1–10.1.5; byte-limit and
//! typed-decoding acceptance is defined by KMIPKIT-0013 FR-014 and FR-017.

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use kmipkit_client::extension_registry::{
    ClientConfiguration, client_extension_registry, client_request_message_extension,
    validate_extension_value,
};
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientCauseCategory, ClientRequest, RequestOptions,
};
use kmipkit_protocol::extension;
use kmipkit_protocol::{CancelRequest, PollRequest, ProcessRequest, QueryAsyncRequestsRequest};
use kmipkit_test_support::{EphemeralPki, LoopbackTcpListener};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, HttpsTransport, PrivateKeyInput, RawTlsTransport,
    RequestDeliveryState, TimeoutLimit, TimeoutPolicy, TransportConfig, TrustSource,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Value};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};
use tokio::runtime::Builder;

const SERVER_NAME: &str = "server.kmipkit.test";
const REQUEST_FRAME: [u8; 8] = [0x42, 0x00, 0x78, 0x01, 0, 0, 0, 0];
const SIMPLE_RESPONSE_FRAME: [u8; 8] = [0x42, 0x00, 0x7B, 0x01, 0, 0, 0, 0];
const PEER_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy)]
enum PeerProtocol {
    RawTtlv,
    Https,
}

#[test]
fn validated_raw_tls_and_https_clients_construct_without_network_activity() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let raw_listener = LoopbackTcpListener::bind()
        .expect("raw-TLS loopback port binds")
        .into_inner();
    raw_listener
        .set_nonblocking(true)
        .expect("raw-TLS listener switches to nonblocking mode");
    let raw_port = raw_listener
        .local_addr()
        .expect("raw port is available")
        .port();
    let raw_client = Client::new(
        empty_client_configuration(),
        transport_configuration(
            &pki,
            Endpoint::raw_tls("127.0.0.1", raw_port),
            TimeoutPolicy::default(),
        ),
    )
    .expect("validated raw-TLS client construction succeeds");
    assert!(
        matches!(raw_listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock)
    );
    drop(raw_client);

    let https_listener = LoopbackTcpListener::bind()
        .expect("HTTPS loopback port binds")
        .into_inner();
    https_listener
        .set_nonblocking(true)
        .expect("HTTPS listener switches to nonblocking mode");
    let https_port = https_listener
        .local_addr()
        .expect("HTTPS port is available")
        .port();
    let https_client = Client::new(
        empty_client_configuration(),
        transport_configuration(
            &pki,
            Endpoint::https(format!("https://127.0.0.1:{https_port}")),
            TimeoutPolicy::default(),
        ),
    )
    .expect("validated HTTPS client construction succeeds");
    assert!(
        matches!(https_listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock)
    );
    drop(https_client);
}

#[test]
fn raw_tls_typed_client_executes_synchronously_inside_tokio_with_options() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind().expect("loopback peer binds");
    let port = listener.local_addr().port();
    let peer = spawn_peer(
        listener.into_inner(),
        server_configuration(&pki),
        PeerProtocol::RawTtlv,
        discover_versions_response(),
    );
    let config = transport_configuration(
        &pki,
        Endpoint::raw_tls("127.0.0.1", port),
        TimeoutPolicy::default().with_total(TimeoutLimit::Bounded(Duration::ZERO)),
    );
    let mut client = Client::new(empty_client_configuration(), config)
        .expect("validated raw-TLS client construction succeeds");
    let options = RequestOptions::default().with_total(TimeoutLimit::Bounded(PEER_TIMEOUT));
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("the caller runtime builds");

    let response = runtime.block_on(async {
        client.execute_with_options(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &kmipkit_ttlv::codec::CodecLimits::defaults(),
            &options,
        )
    });

    let response = response.expect("the synchronous client call works inside Tokio");
    assert_eq!(response.len(), 1);
    let captured_request = peer.join().expect("the raw-TLS peer completes");
    assert_discover_versions_request(&captured_request);
}

#[test]
fn https_typed_client_executes_synchronously_inside_tokio_with_options() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind().expect("loopback peer binds");
    let port = listener.local_addr().port();
    let peer = spawn_peer(
        listener.into_inner(),
        server_configuration(&pki),
        PeerProtocol::Https,
        discover_versions_response(),
    );
    let config = transport_configuration(
        &pki,
        Endpoint::https(format!("https://127.0.0.1:{port}")),
        TimeoutPolicy::default().with_total(TimeoutLimit::Bounded(Duration::ZERO)),
    );
    let mut client = Client::new(empty_client_configuration(), config)
        .expect("validated HTTPS client construction succeeds");
    let options = RequestOptions::default().with_total(TimeoutLimit::Bounded(PEER_TIMEOUT));
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("the caller runtime builds");

    let response = runtime.block_on(async {
        client.execute_with_options(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &kmipkit_ttlv::codec::CodecLimits::defaults(),
            &options,
        )
    });

    let response = response.expect("the synchronous client call works inside Tokio");
    assert_eq!(response.len(), 1);
    let captured_request = peer.join().expect("the HTTPS peer completes");
    assert_discover_versions_request(&captured_request);
}

#[test]
fn raw_tls_adapter_timeout_override_wins_and_preserves_exact_bytes() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind().expect("loopback peer binds");
    let port = listener.local_addr().port();
    let peer = spawn_peer(
        listener.into_inner(),
        server_configuration(&pki),
        PeerProtocol::RawTtlv,
        SIMPLE_RESPONSE_FRAME.to_vec(),
    );
    let config = transport_configuration(
        &pki,
        Endpoint::raw_tls("127.0.0.1", port),
        TimeoutPolicy::default().with_total(TimeoutLimit::Bounded(Duration::ZERO)),
    );
    let mut adapter = RawTlsTransport::new(config).expect("raw-TLS adapter is valid");
    let options = RequestOptions::default().with_total(TimeoutLimit::Bounded(PEER_TIMEOUT));

    let response = adapter
        .exchange_with_options(&REQUEST_FRAME, SIMPLE_RESPONSE_FRAME.len(), &options)
        .expect("the per-exchange timeout overrides the zero client default");

    assert_eq!(response.as_bytes(), SIMPLE_RESPONSE_FRAME);
    assert_eq!(
        peer.join().expect("the raw-TLS peer completes"),
        REQUEST_FRAME
    );
}

#[test]
fn https_adapter_timeout_override_wins_and_preserves_exact_bytes() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind().expect("loopback peer binds");
    let port = listener.local_addr().port();
    let peer = spawn_peer(
        listener.into_inner(),
        server_configuration(&pki),
        PeerProtocol::Https,
        SIMPLE_RESPONSE_FRAME.to_vec(),
    );
    let config = transport_configuration(
        &pki,
        Endpoint::https(format!("https://127.0.0.1:{port}")),
        TimeoutPolicy::default().with_total(TimeoutLimit::Bounded(Duration::ZERO)),
    );
    let mut adapter = HttpsTransport::new(config).expect("HTTPS adapter is valid");
    let options = RequestOptions::default().with_total(TimeoutLimit::Bounded(PEER_TIMEOUT));

    let response = adapter
        .exchange_with_options(&REQUEST_FRAME, SIMPLE_RESPONSE_FRAME.len(), &options)
        .expect("the per-exchange timeout overrides the zero client default");

    assert_eq!(response.as_bytes(), SIMPLE_RESPONSE_FRAME);
    assert_eq!(
        peer.join().expect("the HTTPS peer completes"),
        REQUEST_FRAME
    );
}

#[test]
fn raw_tls_adapter_rejects_oversized_request_before_connect() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind()
        .expect("loopback listener binds")
        .into_inner();
    listener
        .set_nonblocking(true)
        .expect("listener switches to nonblocking mode");
    let port = listener
        .local_addr()
        .expect("loopback port is available")
        .port();
    let config = transport_configuration_with_request_limit(
        &pki,
        Endpoint::raw_tls("127.0.0.1", port),
        TimeoutPolicy::default(),
        7,
    );
    let mut adapter = RawTlsTransport::new(config).expect("raw-TLS adapter is valid");

    let error = adapter
        .exchange_with_options(
            &REQUEST_FRAME,
            SIMPLE_RESPONSE_FRAME.len(),
            &RequestOptions::default(),
        )
        .expect_err("an oversized request is rejected before connection");

    assert_eq!(error.delivery_state(), RequestDeliveryState::NotSent);
    assert!(matches!(listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock));
}

#[test]
fn https_adapter_rejects_oversized_request_before_connect() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind()
        .expect("loopback listener binds")
        .into_inner();
    listener
        .set_nonblocking(true)
        .expect("listener switches to nonblocking mode");
    let port = listener
        .local_addr()
        .expect("loopback port is available")
        .port();
    let config = transport_configuration_with_request_limit(
        &pki,
        Endpoint::https(format!("https://127.0.0.1:{port}")),
        TimeoutPolicy::default(),
        7,
    );
    let mut adapter = HttpsTransport::new(config).expect("HTTPS adapter is valid");

    let error = adapter
        .exchange_with_options(
            &REQUEST_FRAME,
            SIMPLE_RESPONSE_FRAME.len(),
            &RequestOptions::default(),
        )
        .expect_err("an oversized request is rejected before connection");

    assert_eq!(error.delivery_state(), RequestDeliveryState::NotSent);
    assert!(matches!(listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock));
}

#[test]
fn typed_client_rejects_encoded_request_over_codec_limit_before_connect() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind()
        .expect("loopback listener binds")
        .into_inner();
    listener
        .set_nonblocking(true)
        .expect("listener switches to nonblocking mode");
    let port = listener
        .local_addr()
        .expect("loopback port is available")
        .port();
    let mut client = Client::new(
        empty_client_configuration(),
        transport_configuration(
            &pki,
            Endpoint::raw_tls("127.0.0.1", port),
            TimeoutPolicy::default(),
        ),
    )
    .expect("validated raw-TLS client construction succeeds");
    let limits = kmipkit_ttlv::codec::CodecLimits::new(
        8,
        kmipkit_ttlv::codec::CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        kmipkit_ttlv::codec::CodecLimits::DEFAULT_MAX_ELEMENTS,
    )
    .expect("a positive message-byte limit is valid");

    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &limits,
        )
        .expect_err("the encoded typed request exceeds the per-call limit");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(matches!(listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock));
}

#[test]
fn typed_client_accepts_a_valid_response_at_the_exact_codec_limit() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind().expect("loopback peer binds");
    let port = listener.local_addr().port();
    let response_bytes = discover_versions_response();
    let response_cap = response_bytes.len();
    let peer = spawn_peer(
        listener.into_inner(),
        server_configuration(&pki),
        PeerProtocol::RawTtlv,
        response_bytes,
    );
    let mut client = Client::new(
        empty_client_configuration(),
        transport_configuration(
            &pki,
            Endpoint::raw_tls("127.0.0.1", port),
            TimeoutPolicy::default(),
        ),
    )
    .expect("validated raw-TLS client construction succeeds");
    let limits = kmipkit_ttlv::codec::CodecLimits::new(
        response_cap,
        kmipkit_ttlv::codec::CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        kmipkit_ttlv::codec::CodecLimits::DEFAULT_MAX_ELEMENTS,
    )
    .expect("the exact response-byte limit is positive");

    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &limits,
        )
        .expect("a valid response exactly at the configured limit is accepted");

    assert_eq!(response.len(), 1);
    assert_discover_versions_request(&peer.join().expect("the raw-TLS peer completes"));
}

#[test]
fn typed_client_rejects_invalid_response_without_exposing_response_bytes() {
    const RESPONSE_SENTINEL: &[u8] = b"SENTINEL";
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind().expect("loopback peer binds");
    let port = listener.local_addr().port();
    let malformed_response = structure(
        0x0042_0078,
        [ttlv_item(0x0042_0069, 0x07, RESPONSE_SENTINEL)],
    );
    let peer = spawn_peer(
        listener.into_inner(),
        server_configuration(&pki),
        PeerProtocol::RawTtlv,
        malformed_response,
    );
    let mut client = Client::new(
        empty_client_configuration(),
        transport_configuration(
            &pki,
            Endpoint::raw_tls("127.0.0.1", port),
            TimeoutPolicy::default(),
        ),
    )
    .expect("validated raw-TLS client construction succeeds");

    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &kmipkit_ttlv::codec::CodecLimits::defaults(),
        )
        .expect_err("a response with an invalid typed message shape is rejected");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    let diagnostics = format!("{error}\n{error:?}");
    assert!(!diagnostics.contains("SENTINEL"));
    assert_discover_versions_request(&peer.join().expect("the raw-TLS peer completes"));
}

#[test]
fn every_typed_operation_has_a_per_exchange_options_variant() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind().expect("loopback peer binds");
    let port = listener.local_addr().port();
    drop(listener);
    let config = transport_configuration(
        &pki,
        Endpoint::raw_tls("127.0.0.1", port),
        TimeoutPolicy::default(),
    );
    let mut client = Client::new(empty_client_configuration(), config)
        .expect("validated raw-TLS client construction succeeds");
    let limits = kmipkit_ttlv::codec::CodecLimits::defaults();
    let no_time = RequestOptions::default().with_total(TimeoutLimit::Bounded(Duration::ZERO));

    for result in [
        client
            .execute_with_options(
                ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
                &limits,
                &no_time,
            )
            .map(|_| ()),
        client
            .execute_poll_with_options(PollRequest::new(b"poll-correlation"), &limits, &no_time)
            .map(|_| ()),
        client
            .execute_cancel_with_options(
                CancelRequest::new(b"cancel-correlation"),
                &limits,
                &no_time,
            )
            .map(|_| ()),
        client
            .execute_process_with_options(
                ProcessRequest::new(b"process-correlation"),
                None,
                &limits,
                &no_time,
            )
            .map(|_| ()),
        client
            .execute_query_async_requests_with_options(
                QueryAsyncRequestsRequest::new(),
                None,
                &limits,
                &no_time,
            )
            .map(|_| ()),
    ] {
        let error = result.expect_err("the immediate total deadline expires before dispatch");
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(
            error.cause_category(),
            Some(ClientCauseCategory::Transport(
                kmipkit_transport::TransportCauseCategory::Timeout
            ))
        );
    }
}

#[test]
fn production_client_rejects_foreign_client_request_message_extension_as_invalid_input_not_sent() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind()
        .expect("loopback listener binds")
        .into_inner();
    listener
        .set_nonblocking(true)
        .expect("listener switches to nonblocking mode");
    let port = listener
        .local_addr()
        .expect("loopback port is available")
        .port();
    let source_configuration = extension_client_configuration();
    let foreign_extension = registered_request_extension(&source_configuration);
    let mut client = Client::new(
        empty_client_configuration(),
        transport_configuration(
            &pki,
            Endpoint::raw_tls("127.0.0.1", port),
            TimeoutPolicy::default(),
        ),
    )
    .expect("validated raw-TLS client construction succeeds");

    let error = client
        .execute_with_options(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::discover_versions())
                    .with_extension(foreign_extension),
            ),
            &kmipkit_ttlv::codec::CodecLimits::defaults(),
            &RequestOptions::default(),
        )
        .expect_err("an extension from another client configuration is rejected");

    assert_eq!(
        error.cause_category(),
        Some(ClientCauseCategory::InvalidInput)
    );
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let diagnostics = format!("{error}\n{error:?}");
    for sentinel in [VENDOR_IDENTIFIER, EXTENSION_NAME, EXTENSION_PAYLOAD] {
        assert!(!diagnostics.contains(sentinel));
    }
    assert!(matches!(listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock));
}

#[test]
fn production_client_accepts_client_request_message_extension_from_its_own_configuration() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let listener = LoopbackTcpListener::bind().expect("loopback peer binds");
    let port = listener.local_addr().port();
    let peer = spawn_peer(
        listener.into_inner(),
        server_configuration(&pki),
        PeerProtocol::RawTtlv,
        discover_versions_response(),
    );
    let client_configuration = extension_client_configuration();
    let request_extension = registered_request_extension(&client_configuration);
    let mut client = Client::new(
        client_configuration,
        transport_configuration(
            &pki,
            Endpoint::raw_tls("127.0.0.1", port),
            TimeoutPolicy::default(),
        ),
    )
    .expect("validated raw-TLS client construction succeeds");

    let response = client
        .execute_with_options(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::discover_versions())
                    .with_extension(request_extension),
            ),
            &kmipkit_ttlv::codec::CodecLimits::defaults(),
            &RequestOptions::default(),
        )
        .expect("the same configuration's extension is valid for execution");

    assert_eq!(response.len(), 1);
    let request = peer.join().expect("the raw-TLS peer completes");
    let expected_extension = request_message_extension_wire();
    assert!(
        request
            .windows(expected_extension.len())
            .any(|window| window == expected_extension),
        "the registered extension's exact TTLV representation reaches the peer"
    );
}

#[test]
fn production_constructor_does_not_accept_a_caller_transport() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/production_transport_injection.rs");
    cases.compile_fail("tests/ui/typed_response_raw_bytes.rs");
}

fn empty_client_configuration() -> ClientConfiguration {
    let registry = client_extension_registry(Vec::new(), kmipkit_protocol::extension::defaults())
        .expect("an empty extension registry is valid");
    ClientConfiguration::new(registry)
}

const VENDOR_IDENTIFIER: &str = "extension.vendor.sentinel";
const EXTENSION_NAME: &str = "registry.identity.sentinel";
const EXTENSION_VERSION: &str = "1";
const EXTENSION_PAYLOAD: &str = "request.payload.sentinel";
const EXTENSION_DISCRIMINATOR_TAG: u32 = 0x0042_0173;

fn extension_client_configuration() -> ClientConfiguration {
    let identity =
        extension::extension_identity(VENDOR_IDENTIFIER, EXTENSION_NAME, EXTENSION_VERSION)
            .expect("extension identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("extension is compatible with KMIP 2.1");
    let tag = vendor_extension_tag(EXTENSION_DISCRIMINATOR_TAG);
    let path = extension::ttlv_path(tag).expect("extension discriminator path is valid");
    let discriminator =
        extension::discriminator(path, Value::text_string(EXTENSION_PAYLOAD.to_owned()))
            .expect("extension discriminator is a text value");
    let schema = extension::structure(
        vec![
            extension::required(
                tag,
                extension::scalar(ItemType::TextString).expect("Text String schema is valid"),
            )
            .expect("required discriminator field is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("extension schema is valid");
    let definition =
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("extension definition is valid");
    let information =
        extension::extension_information("fixture").expect("extension metadata is valid");
    let definition =
        extension::with_information(definition, information).expect("extension metadata attaches");
    let registry = client_extension_registry(vec![definition], extension::defaults())
        .expect("single extension registry is valid");
    ClientConfiguration::new(registry)
}

fn registered_request_extension(
    configuration: &ClientConfiguration,
) -> kmipkit_client::extension_registry::ClientRequestMessageExtension {
    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                vendor_extension_tag(EXTENSION_DISCRIMINATOR_TAG),
                Value::text_string(EXTENSION_PAYLOAD.to_owned()),
            )
            .expect("request discriminator model item is valid"),
        )
        .expect("request extension payload is valid");
    let registered = validate_extension_value(
        configuration.extension_registry(),
        extension::extension_identity(VENDOR_IDENTIFIER, EXTENSION_NAME, EXTENSION_VERSION)
            .expect("extension identity is valid"),
        payload,
        &kmipkit_ttlv::codec::CodecLimits::defaults(),
    )
    .expect("extension value matches its client's registry");
    client_request_message_extension(registered, false)
        .expect("validated extension can be attached to a request")
}

fn vendor_extension_tag(raw: u32) -> kmipkit_ttlv::Tag {
    RawTag::new(raw)
        .expect("test extension tag fits the raw tag representation")
        .try_checked()
        .expect("test extension tag uses the vendor allocation")
}

fn request_message_extension_wire() -> Vec<u8> {
    let discriminator = ttlv_item(
        EXTENSION_DISCRIMINATOR_TAG,
        0x07,
        EXTENSION_PAYLOAD.as_bytes(),
    );
    let vendor_extension = ttlv_item(0x0042_009C, 0x01, &discriminator);
    let vendor = ttlv_item(0x0042_009D, 0x07, VENDOR_IDENTIFIER.as_bytes());
    let criticality = ttlv_item(0x0042_0026, 0x06, &[0, 0, 0, 0, 0, 0, 0, 0]);
    ttlv_item(
        0x0042_0051,
        0x01,
        &[vendor, criticality, vendor_extension].concat(),
    )
}

fn transport_configuration(
    pki: &EphemeralPki,
    endpoint: Endpoint,
    timeouts: TimeoutPolicy,
) -> TransportConfig {
    transport_configuration_with_request_limit(pki, endpoint, timeouts, 16 * 1024 * 1024)
}

fn transport_configuration_with_request_limit(
    pki: &EphemeralPki,
    endpoint: Endpoint,
    timeouts: TimeoutPolicy,
    max_request_bytes: usize,
) -> TransportConfig {
    let client_identity = pki.client_identity();
    let certificates = client_identity
        .certificate_chain_der()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect();
    TransportConfig::builder(endpoint)
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der(certificates),
            PrivateKeyInput::from_der(client_identity.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .tls_server_name(SERVER_NAME)
        .timeouts(timeouts)
        .max_request_bytes(max_request_bytes)
        .build()
        .expect("ephemeral certificate inputs produce a valid transport configuration")
}

fn server_configuration(pki: &EphemeralPki) -> Arc<ServerConfig> {
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("the ephemeral authority is accepted");
    let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .expect("the mTLS client verifier builds");
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("TLS 1.3 server policy builds");
    let identity = pki.server_identity();
    let certificates = identity
        .certificate_chain_der()
        .into_iter()
        .map(|certificate| CertificateDer::from(certificate.to_vec()))
        .collect();
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        identity.private_key_der().to_vec(),
    ));
    Arc::new(
        builder
            .with_client_cert_verifier(verifier)
            .with_single_cert(certificates, private_key)
            .expect("the ephemeral server identity is valid"),
    )
}

fn spawn_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    protocol: PeerProtocol,
    response: Vec<u8>,
) -> JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        listener
            .set_nonblocking(false)
            .expect("peer listener switches to blocking mode");
        let (stream, _) = listener.accept().expect("client connects to the peer");
        stream
            .set_read_timeout(Some(PEER_TIMEOUT))
            .expect("peer read timeout is configured");
        stream
            .set_write_timeout(Some(PEER_TIMEOUT))
            .expect("peer write timeout is configured");
        let connection = ServerConnection::new(configuration)
            .expect("the ephemeral TLS server connection is created");
        let mut tls = StreamOwned::new(connection, stream);
        let captured = match protocol {
            PeerProtocol::RawTtlv => read_ttlv_frame(&mut tls),
            PeerProtocol::Https => read_https_body(&mut tls),
        };
        match protocol {
            PeerProtocol::RawTtlv => tls
                .write_all(&response)
                .expect("raw response bytes are written"),
            PeerProtocol::Https => {
                write!(
                    tls,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    response.len()
                )
                .expect("HTTPS response headers are written");
                tls.write_all(&response)
                    .expect("HTTPS response body is written");
            }
        }
        tls.flush().expect("TLS response is flushed");
        let _ = tls.get_ref().shutdown(Shutdown::Both);
        captured
    })
}

fn read_ttlv_frame(stream: &mut impl Read) -> Vec<u8> {
    let mut frame = [0_u8; 8];
    stream
        .read_exact(&mut frame)
        .expect("request TTLV header is received");
    let value_length = u32::from_be_bytes([frame[4], frame[5], frame[6], frame[7]]) as usize;
    let padded_length = value_length.div_ceil(8) * 8;
    let mut value = vec![0; padded_length];
    stream
        .read_exact(&mut value)
        .expect("request TTLV value is received");
    let mut captured = frame.to_vec();
    captured.extend(value);
    captured
}

fn read_https_body(stream: &mut impl Read) -> Vec<u8> {
    let mut request = Vec::new();
    let body_start = loop {
        let mut buffer = [0_u8; 512];
        let count = stream.read(&mut buffer).expect("HTTP request is received");
        assert_ne!(count, 0, "the client closes before completing its request");
        request.extend_from_slice(&buffer[..count]);
        if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
        assert!(request.len() < 16 * 1024, "HTTP headers remain bounded");
    };
    let headers =
        std::str::from_utf8(&request[..body_start]).expect("HTTP headers are ASCII-compatible");
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length").then(|| {
                value
                    .trim()
                    .parse::<usize>()
                    .expect("content length is numeric")
            })
        })
        .expect("HTTP request has a content length");
    while request.len() < body_start + content_length {
        let mut buffer = [0_u8; 512];
        let count = stream
            .read(&mut buffer)
            .expect("HTTP request body is received");
        assert_ne!(count, 0, "the client closes before completing its body");
        request.extend_from_slice(&buffer[..count]);
        assert!(request.len() <= body_start + content_length);
    }
    request[body_start..].to_vec()
}

fn assert_discover_versions_request(request: &[u8]) {
    assert!(
        request.len() >= 8,
        "request contains its TTLV message header"
    );
    assert_eq!(&request[..3], &[0x42, 0x00, 0x78]);
    assert!(request.windows(4).any(|window| window == [0, 0, 0, 30]));
}

fn discover_versions_response() -> Vec<u8> {
    let protocol_version = structure(
        0x0042_0069,
        [integer(0x0042_006A, 2), integer(0x0042_006B, 1)],
    );
    let header = structure(
        0x0042_007A,
        [
            protocol_version.clone(),
            date_time(0x0042_0092, 1),
            integer(0x0042_000D, 1),
        ],
    );
    let response_payload = structure(0x0042_007C, [protocol_version]);
    let batch_item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            enumeration(0x0042_007F, 0),
            response_payload,
        ],
    );
    structure(0x0042_007B, [header, batch_item])
}

fn structure<const N: usize>(tag: u32, children: [Vec<u8>; N]) -> Vec<u8> {
    let value = children.into_iter().flatten().collect::<Vec<_>>();
    ttlv_item(tag, 0x01, &value)
}

fn integer(tag: u32, value: i32) -> Vec<u8> {
    ttlv_item(tag, 0x02, &value.to_be_bytes())
}

fn enumeration(tag: u32, value: u32) -> Vec<u8> {
    ttlv_item(tag, 0x05, &value.to_be_bytes())
}

fn date_time(tag: u32, value: i64) -> Vec<u8> {
    ttlv_item(tag, 0x09, &value.to_be_bytes())
}

fn ttlv_item(tag: u32, item_type: u8, value: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + value.len().div_ceil(8) * 8);
    encoded.extend_from_slice(&tag.to_be_bytes()[1..]);
    encoded.push(item_type);
    encoded.extend_from_slice(
        &u32::try_from(value.len())
            .expect("test value length fits the TTLV field")
            .to_be_bytes(),
    );
    encoded.extend_from_slice(value);
    encoded.resize(8 + value.len().div_ceil(8) * 8, 0);
    encoded
}
