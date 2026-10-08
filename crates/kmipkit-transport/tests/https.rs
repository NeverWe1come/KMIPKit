//! Request-building and capture contract for HTTPS TTLV.
//!
//! The local peer requires TLS 1.3 mutual authentication and captures the
//! actual HTTP/1.1 request. The adapter is included from its private source so
//! tests can inject a deterministic resolver and observe KMIPKit-owned body
//! cleanup without adding public test APIs.

use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use kmipkit_test_support::{EphemeralPki, LoopbackTcpListener, fixtures};
pub use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};

#[path = "../src/config.rs"]
#[allow(dead_code)]
mod config;
#[path = "../src/https.rs"]
mod https;
#[path = "../src/resolver.rs"]
#[allow(dead_code)]
mod resolver;
#[path = "../src/secret.rs"]
#[allow(dead_code)]
mod secret;
#[path = "../src/timeout.rs"]
#[allow(dead_code)]
mod timeout;
#[path = "../src/tls.rs"]
#[allow(dead_code)]
mod tls;
#[path = "../src/worker.rs"]
#[allow(dead_code)]
mod worker;

pub use config::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RequestOptions,
    RevocationListInput, TimeoutLimit, TimeoutPolicy, TransportConfig, TransportConfigBuilder,
    TransportConfigError, TrustSource,
};

const SERVER_NAME: &str = "server.kmipkit.test";
const RESPONSE_BODY: &[u8] = b"response";
const PEER_TIMEOUT: Duration = Duration::from_secs(6);
const ACCEPT_TIMEOUT: Duration = Duration::from_secs(2);
const CLEANUP_GATE_OBSERVATION_WINDOW: Duration = Duration::from_millis(250);
const MAX_CAPTURED_REQUEST: usize = 1024 * 1024;

#[test]
fn https_configuration_rejects_absolute_or_different_authority_targets() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    for target in [
        "https://other.kmipkit.test/kmip",
        "//other.kmipkit.test/kmip",
    ] {
        let result = config_builder(&pki, "https://server.kmipkit.test".to_owned())
            .target_uri(target)
            .build();

        assert!(
            matches!(result, Err(config::TransportConfigError::InvalidTarget)),
            "targets with a scheme or separate authority are rejected"
        );
    }
}

#[test]
fn https_configuration_rejects_the_plain_http_scheme() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let result = config_builder(&pki, "http://server.kmipkit.test".to_owned()).build();

    assert!(
        matches!(result, Err(config::TransportConfigError::InvalidEndpoint)),
        "the transport accepts HTTPS endpoints only"
    );
}

#[test]
fn https_default_target_posts_exact_bytes_with_required_headers_and_zeroizes_on_success() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let peer = spawn_peer(listener.into_inner(), server_config(&pki), true);
    let config = client_config(
        &pki,
        format!("https://{SERVER_NAME}:{}", address.port()),
        None,
        None,
    );
    let observer = secret::SecretBufferObserver::new(fixtures::REQUEST_SENTINEL.len());
    let mut adapter =
        https::new_for_test_with_resolver(config, Some(observer.clone()), fixed_resolver(address));

    let result = adapter.exchange(fixtures::REQUEST_SENTINEL, 64);
    let response = result.expect("the verified HTTPS exchange succeeds");
    assert!(
        response.as_bytes() == RESPONSE_BODY,
        "the response body is returned unchanged"
    );

    let peer = peer.join().expect("the bounded HTTPS peer completes");
    assert!(peer.accepted, "the peer accepted one connection");
    assert_eq!(
        peer.protocol_version,
        Some(rustls::ProtocolVersion::TLSv1_3)
    );
    assert!(
        peer.client_identity_present,
        "the peer verified a client identity"
    );
    let request = peer.request.expect("the peer captured one HTTP request");
    assert_request(
        &request,
        "POST",
        "/kmip",
        &format!("{SERVER_NAME}:{}", address.port()),
        fixtures::REQUEST_SENTINEL,
    );
    assert_request_owner_zeroized(&observer, fixtures::REQUEST_SENTINEL.len());
}

#[test]
fn https_cancellation_aborts_the_hyper_connection_driver() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let (peer, request_seen) = spawn_stalled_peer(listener.into_inner(), server_config(&pki));
    let config = config_builder(&pki, format!("https://{SERVER_NAME}:{}", address.port()))
        .build()
        .expect("the explicit identity and trust inputs build a valid HTTPS config");
    let (driver_aborted_tx, driver_aborted_rx) = mpsc::sync_channel(1);
    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
    let adapter = https::new_for_test_with_resolver_and_driver_abort_observer(
        config,
        fixed_resolver(address),
        driver_aborted_tx,
        cancel_rx,
    );
    let (exchange_tx, exchange_rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let mut adapter = adapter;
        let result = adapter.exchange(fixtures::REQUEST_SENTINEL, 64);
        let _ = exchange_tx.send((adapter, result));
    });

    assert!(
        request_seen.recv_timeout(PEER_TIMEOUT).is_ok(),
        "the peer receives the request before the test cancels the exchange"
    );
    cancel_tx
        .send(())
        .expect("the in-flight adapter exchange is ready for cancellation");
    let (adapter, result) = exchange_rx
        .recv_timeout(PEER_TIMEOUT)
        .expect("worker cancellation completes the HTTPS exchange");
    let error = result.expect_err("the peer withholds its response until cancellation");
    assert_eq!(error.delivery_state(), RequestDeliveryState::PossiblySent);
    let driver_aborted = driver_aborted_rx.recv_timeout(ACCEPT_TIMEOUT).is_ok();

    drop(adapter);
    let peer = peer
        .join()
        .expect("the bounded stalled HTTPS peer completes");
    assert!(peer.accepted, "the peer accepted one connection");
    assert!(peer.request.is_some(), "the peer captured the request");
    assert!(
        driver_aborted,
        "the Hyper connection driver is explicitly aborted when exchange cancellation drops its future"
    );
}

#[test]
fn https_exchange_waits_for_driver_cleanup_acknowledgement_before_returning() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let (peer, request_seen) = spawn_stalled_peer(listener.into_inner(), server_config(&pki));
    let config = config_builder(&pki, format!("https://{SERVER_NAME}:{}", address.port()))
        .build()
        .expect("the explicit identity and trust inputs build a valid HTTPS config");
    let (cleanup_release_tx, cleanup_release_rx) = tokio::sync::oneshot::channel();
    let (events_tx, events_rx) = mpsc::channel();
    let cleanup_gate = https::DriverCleanupGateForTest {
        release: cleanup_release_rx,
        events: events_tx.clone(),
    };
    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
    let adapter = https::new_for_test_with_resolver_and_driver_cleanup_gate(
        config,
        fixed_resolver(address),
        cleanup_gate,
        cancel_rx,
    );
    let (exchange_tx, exchange_rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let mut adapter = adapter;
        let result = adapter.exchange(fixtures::REQUEST_SENTINEL, 64);
        let _ = events_tx.send(https::DriverCleanupEventForTest::ExchangeReturned);
        let _ = exchange_tx.send((adapter, result));
    });

    assert!(
        request_seen.recv_timeout(PEER_TIMEOUT).is_ok(),
        "the peer receives the request before the test cancels the exchange"
    );
    cancel_tx
        .send(())
        .expect("the in-flight adapter exchange is ready for cancellation");
    let mut observed_events = Vec::with_capacity(3);
    let started = loop {
        match events_rx.recv_timeout(PEER_TIMEOUT) {
            Ok(event) => {
                observed_events.push(event);
                if event == https::DriverCleanupEventForTest::Started {
                    break true;
                }
            }
            Err(_) => break false,
        }
    };
    let returned_while_gate_was_closed =
        observed_events.contains(&https::DriverCleanupEventForTest::ExchangeReturned);
    if started
        && !returned_while_gate_was_closed
        && let Ok(event) = events_rx.recv_timeout(CLEANUP_GATE_OBSERVATION_WINDOW)
    {
        observed_events.push(event);
    }
    let returned_during_observation =
        observed_events.contains(&https::DriverCleanupEventForTest::ExchangeReturned);
    let _ = cleanup_release_tx.send(());

    while !observed_events.contains(&https::DriverCleanupEventForTest::Acknowledged)
        || !observed_events.contains(&https::DriverCleanupEventForTest::ExchangeReturned)
    {
        match events_rx.recv_timeout(PEER_TIMEOUT) {
            Ok(event) => observed_events.push(event),
            Err(_) => break,
        }
    }
    let (adapter, result) = exchange_rx
        .recv_timeout(PEER_TIMEOUT)
        .expect("the HTTPS exchange eventually returns after cleanup is released");
    let error = result.expect_err("the peer withholds its response until cancellation");
    assert_eq!(error.delivery_state(), RequestDeliveryState::PossiblySent);

    drop(adapter);
    let peer = peer
        .join()
        .expect("the bounded stalled HTTPS peer completes");
    assert!(peer.accepted, "the peer accepted one connection");
    assert!(peer.request.is_some(), "the peer captured the request");
    let acknowledgement_index = observed_events
        .iter()
        .position(|event| *event == https::DriverCleanupEventForTest::Acknowledged);
    let returned_index = observed_events
        .iter()
        .position(|event| *event == https::DriverCleanupEventForTest::ExchangeReturned);
    assert!(started, "the Hyper driver cleanup path started");
    assert!(
        !returned_while_gate_was_closed && !returned_during_observation,
        "the public exchange remains pending while driver cleanup acknowledgment is gated"
    );
    assert!(
        matches!((acknowledgement_index, returned_index), (Some(ack), Some(ret)) if ack < ret),
        "the driver cleanup acknowledgment precedes the public exchange return"
    );
}

#[test]
fn https_configured_target_preserves_ip_authority_and_zeroizes_after_peer_close() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let peer = spawn_peer(listener.into_inner(), server_config(&pki), false);
    let config = client_config(
        &pki,
        format!("https://127.0.0.1:{}", address.port()),
        Some("/custom/kmip?version=1"),
        Some(SERVER_NAME),
    );
    let observer = secret::SecretBufferObserver::new(fixtures::REQUEST_SENTINEL.len());
    let mut adapter =
        https::new_for_test_with_resolver(config, Some(observer.clone()), fixed_resolver(address));

    let result = adapter.exchange(fixtures::REQUEST_SENTINEL, 64);
    let error = result.expect_err("the peer closes without an HTTP response");
    assert_eq!(error.delivery_state(), RequestDeliveryState::PossiblySent);

    let peer = peer.join().expect("the bounded HTTPS peer completes");
    assert!(peer.accepted, "the peer accepted one connection");
    assert_eq!(
        peer.protocol_version,
        Some(rustls::ProtocolVersion::TLSv1_3)
    );
    assert!(
        peer.client_identity_present,
        "the peer verified a client identity"
    );
    let request = peer.request.expect("the peer captured one HTTP request");
    assert_request(
        &request,
        "POST",
        "/custom/kmip?version=1",
        &format!("127.0.0.1:{}", address.port()),
        fixtures::REQUEST_SENTINEL,
    );
    assert_request_owner_zeroized(&observer, fixtures::REQUEST_SENTINEL.len());
}

#[test]
fn https_request_host_serializes_bracketed_ipv6_authority_and_explicit_port() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let config = client_config(
        &pki,
        "https://[2001:db8::7]:8443".to_owned(),
        None,
        Some(SERVER_NAME),
    );

    let request = https::build_request_for_test(&config, fixtures::REQUEST_SENTINEL)
        .expect("the validated HTTPS configuration builds a request");
    let hosts: Vec<_> = request
        .headers()
        .get_all("host")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .collect();

    assert_eq!(hosts.len(), 1, "the request has exactly one Host field");
    assert!(
        hosts[0] == "[2001:db8::7]:8443",
        "Host comes from the endpoint authority, not the TLS verification name"
    );
    assert!(
        request
            .uri()
            .path_and_query()
            .is_some_and(|target| target.as_str() == "/kmip"),
        "the HTTP request target remains origin-form"
    );
}

#[test]
fn https_rejects_invalid_status_response_headers_and_encodings() {
    let cases: [(&str, &[u8]); 12] = [
        (
            "non-200 status",
            b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/octet-stream\r\nContent-Length: 8\r\n\r\nresponse",
        ),
        (
            "missing Content-Type",
            b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\n\r\nresponse",
        ),
        (
            "duplicate Content-Type",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Type: application/octet-stream\r\nContent-Length: 8\r\n\r\nresponse",
        ),
        (
            "duplicate Content-Length",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 8\r\nContent-Length: 8\r\n\r\nresponse",
        ),
        (
            "conflicting Content-Length",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 8\r\nContent-Length: 9\r\n\r\nresponse",
        ),
        (
            "invalid media type",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 8\r\n\r\nresponse",
        ),
        (
            "invalid Content-Length",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: eight\r\n\r\nresponse",
        ),
        (
            "negative Content-Length",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: -1\r\n\r\n",
        ),
        (
            "Transfer-Encoding",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nTransfer-Encoding: chunked\r\n\r\n8\r\nresponse\r\n0\r\n\r\n",
        ),
        (
            "conflicting Transfer-Encoding and Content-Length",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nTransfer-Encoding: chunked\r\nContent-Length: 8\r\n\r\n8\r\nresponse\r\n0\r\n\r\n",
        ),
        (
            "Content-Encoding",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Encoding: gzip\r\nContent-Length: 8\r\n\r\nresponse",
        ),
        (
            "duplicate Content-Encoding",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Encoding: identity\r\nContent-Encoding: gzip\r\nContent-Length: 8\r\n\r\nresponse",
        ),
    ];
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let mut failures = Vec::new();

    for (name, response) in cases {
        let result = exchange_raw_response(&pki, response, 64);
        match result {
            Err(error) if error.delivery_state() == RequestDeliveryState::ResponseStarted => {}
            Err(error) => failures.push(format!(
                "{name}: rejected with delivery state {:?}",
                error.delivery_state()
            )),
            Ok(_) => failures.push(format!("{name}: unexpectedly accepted the response")),
        }
    }

    assert!(
        failures.is_empty(),
        "HTTPS must reject invalid status, headers, and encodings: {failures:?}"
    );
}

#[test]
fn https_rejects_close_delimited_response_without_content_length() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let peer = spawn_clean_tls_close_response_peer(
        listener.into_inner(),
        server_config(&pki),
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\nresponse"
            .to_vec(),
    );
    let config = client_config(
        &pki,
        format!("https://{SERVER_NAME}:{}", address.port()),
        None,
        None,
    );
    let mut adapter = https::new_for_test_with_resolver(config, None, fixed_resolver(address));

    let result = adapter.exchange(fixtures::REQUEST_SENTINEL, 64);
    let peer = peer
        .join()
        .expect("the bounded close-delimited HTTPS peer completes");

    assert!(
        peer.accepted,
        "the response peer accepted one TLS connection"
    );
    assert!(peer.request.is_some(), "the peer captured one HTTP request");
    assert!(
        peer.close_notify_sent,
        "the peer completed TLS shutdown with a flushed close_notify"
    );
    assert!(
        matches!(result, Err(error) if error.delivery_state() == RequestDeliveryState::ResponseStarted),
        "a valid close-delimited response without Content-Length must be rejected"
    );
}

#[test]
fn https_rejects_http_parser_errors_and_truncated_response_bodies() {
    let cases: [(&str, &[u8]); 4] = [
        (
            "malformed status line",
            b"HTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\n\r\n",
        ),
        (
            "malformed header name",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\nBad Header: value\r\n\r\n",
        ),
        (
            "malformed chunk framing",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nTransfer-Encoding: chunked\r\n\r\nnot-a-chunk\r\n",
        ),
        (
            "truncated fixed-length body",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 16\r\n\r\npartial",
        ),
    ];
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let mut failures = Vec::new();

    for (name, response) in cases {
        match exchange_raw_response(&pki, response, 64) {
            Err(error) if error.delivery_state() == RequestDeliveryState::ResponseStarted => {}
            Err(error) => failures.push(format!(
                "{name}: parser rejected with delivery state {:?}",
                error.delivery_state()
            )),
            Ok(_) => failures.push(format!("{name}: unexpectedly accepted the response")),
        }
    }

    assert!(
        failures.is_empty(),
        "malformed and truncated HTTP responses must fail after response bytes: {failures:?}"
    );
}

#[test]
fn https_enforces_64_headers_and_64_kibibyte_parser_input_boundary() {
    const PARSER_LIMIT: usize = 64 * 1024;
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let accepted_64 = response_with_header_count(64);
    let rejected_65 = response_with_header_count(65);
    let accepted_at_limit = response_with_header_block_size(PARSER_LIMIT);
    let rejected_over_limit = response_with_header_block_size(PARSER_LIMIT + 1);

    let exact_headers = exchange_raw_response(&pki, &accepted_64, 1);
    let too_many_headers = exchange_raw_response(&pki, &rejected_65, 1);
    let exact_size = exchange_raw_response(&pki, &accepted_at_limit, 1);
    let oversized = exchange_raw_response(&pki, &rejected_over_limit, 1);
    let incomplete = exchange_raw_response(
        &pki,
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\nX-Incomplete: value\r\n",
        1,
    );

    let mut failures = Vec::new();
    if exact_headers.is_err() {
        failures.push("exactly 64 headers were rejected");
    }
    if !matches!(too_many_headers, Err(error) if error.delivery_state() == RequestDeliveryState::ResponseStarted)
    {
        failures.push("a 65th response header was accepted or lacked response evidence");
    }
    if exact_size.is_err() {
        failures.push("a complete 64 KiB response header block was rejected");
    }
    if !matches!(oversized, Err(error) if error.delivery_state() == RequestDeliveryState::ResponseStarted)
    {
        failures
            .push("a response header block over 64 KiB was accepted or lacked response evidence");
    }
    if !matches!(incomplete, Err(error) if error.delivery_state() == RequestDeliveryState::ResponseStarted)
    {
        failures
            .push("an incomplete response header block was accepted or lacked response evidence");
    }
    assert!(
        failures.is_empty(),
        "parser boundary failures: {failures:?}"
    );
}

#[test]
fn https_accepts_a_response_at_the_exact_body_limit() {
    const BODY: &[u8] = b"boundary";
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        BODY.len()
    );
    let mut wire = response.into_bytes();
    wire.extend_from_slice(BODY);

    let result = exchange_raw_response(&pki, &wire, BODY.len());
    assert!(
        matches!(result, Ok(response) if response.as_bytes() == BODY),
        "a response whose declared and received size equals the cap is accepted unchanged"
    );
}

#[test]
fn https_rejects_declared_oversize_before_body_arrives_or_response_buffer_grows() {
    const LIMIT: usize = 8;
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let config = config_builder(&pki, format!("https://{SERVER_NAME}:{}", address.port()))
        .timeouts(TimeoutPolicy::default().with_read(TimeoutLimit::Bounded(Duration::from_secs(2))))
        .build()
        .expect("the explicit identity and trust inputs build a valid HTTPS config");
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        LIMIT + 1
    )
    .into_bytes();
    let (peer, headers_sent, body_release) = spawn_gated_response_peer(
        listener.into_inner(),
        server_config(&pki),
        header,
        b"oversize!".to_vec(),
    );
    let observer = https::ResponseBufferObserver::new();
    let adapter = https::new_for_test_with_response_buffer_observer(
        config,
        fixed_resolver(address),
        observer.clone(),
    );
    let (result_tx, result_rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let mut adapter = adapter;
        let result = adapter.exchange(fixtures::REQUEST_SENTINEL, LIMIT);
        let _ = result_tx.send(result);
    });

    assert!(
        headers_sent.recv_timeout(PEER_TIMEOUT).is_ok(),
        "the peer sent response headers declaring a body above the cap"
    );
    let early_result = result_rx.recv_timeout(Duration::from_millis(250));
    let rejected_before_body = matches!(
        &early_result,
        Ok(Err(error)) if error.cause_category() != TransportCauseCategory::Timeout
    );
    let _ = body_release.send(());
    let result = match early_result {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => result_rx
            .recv_timeout(PEER_TIMEOUT)
            .expect("the exchange returns after the held response body is released"),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("the exchange result channel remains connected");
        }
    };
    let peer = peer.join().expect("the bounded HTTPS peer completes");

    assert!(
        matches!(result, Err(error) if error.delivery_state() == RequestDeliveryState::ResponseStarted),
        "a response declaring more than the effective cap is rejected"
    );
    assert_eq!(
        observer.allocation_attempts(),
        0,
        "the KMIPKit response body owner does not request a buffer allocation over the cap"
    );
    assert_eq!(
        observer.requested_capacity(),
        0,
        "no KMIPKit response buffer capacity is requested over the declared cap"
    );
    assert!(
        peer.request.is_some(),
        "the peer captured the request before sending the oversized response headers"
    );
    assert!(
        rejected_before_body,
        "declared oversize is rejected from Content-Length before the peer sends body bytes"
    );
}

#[test]
fn https_zeroizes_partial_response_buffer_when_http_body_is_truncated() {
    const PARTIAL_BODY: &[u8] = b"partial-secret-response";
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        PARTIAL_BODY.len() + 5
    );
    let mut response = header.into_bytes();
    response.extend_from_slice(PARTIAL_BODY);
    let peer = spawn_raw_response_peer(listener.into_inner(), server_config(&pki), response);
    let config = client_config(
        &pki,
        format!("https://{SERVER_NAME}:{}", address.port()),
        None,
        None,
    );
    let observer = https::ResponseBufferObserver::new();
    let mut adapter = https::new_for_test_with_response_buffer_observer(
        config,
        fixed_resolver(address),
        observer.clone(),
    );

    let result = adapter.exchange(fixtures::REQUEST_SENTINEL, 64);
    let peer = peer.join().expect("the bounded HTTPS peer completes");

    assert!(
        matches!(result, Err(error) if error.delivery_state() == RequestDeliveryState::ResponseStarted),
        "a body shorter than Content-Length is rejected"
    );
    assert_eq!(
        observer.initialized_len(),
        PARTIAL_BODY.len(),
        "the response owner records the bytes initialized before truncation"
    );
    assert!(
        observer.initialized_range_was_zero(),
        "initialized partial response bytes are zeroized before release"
    );
    assert!(
        observer.replaced_allocations_were_zero(),
        "replaced response allocations are zeroized before release"
    );
    assert!(
        peer.request.is_some(),
        "the peer captured the HTTPS request"
    );
}

#[test]
fn https_never_attributes_a_surplus_response_to_a_later_exchange() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let (peer, surplus_release, surplus_sent) =
        spawn_surplus_response_peer(listener.into_inner(), server_config(&pki));
    let config = client_config(
        &pki,
        format!("https://{SERVER_NAME}:{}", address.port()),
        None,
        None,
    );
    let mut adapter = https::new_for_test_with_resolver(config, None, fixed_resolver(address));

    let first = adapter
        .exchange(fixtures::REQUEST_SENTINEL, 64)
        .expect("the first exchange receives its own response");
    let second = adapter
        .exchange(fixtures::REQUEST_SENTINEL, 64)
        .expect("the second exchange receives its own response");
    let _ = surplus_release.send(());
    assert!(
        surplus_sent.recv_timeout(PEER_TIMEOUT).is_ok(),
        "the peer queued the unsolicited response after two completed exchanges"
    );
    let third = adapter.exchange(fixtures::REQUEST_SENTINEL, 64);
    let third_avoids_surplus = match &third {
        Ok(response) => response.as_bytes() != b"unsolicited",
        Err(_) => true,
    };
    drop(adapter);
    let peer = peer.join().expect("the bounded reuse peer completes");

    let mut failures = Vec::new();
    if first.as_bytes() != b"first" {
        failures.push("the first exchange did not receive its response");
    }
    if second.as_bytes() != b"second" {
        failures.push("the second exchange did not receive its response");
    }
    if peer.request_connection_ids.len() < 2 {
        failures.push("the peer did not observe two actual HTTPS requests");
    }
    if peer.request_connection_ids.first() != peer.request_connection_ids.get(1) {
        failures.push("the first two completed exchanges used different TLS connections");
    }
    if !peer.surplus_write_succeeded {
        failures.push("the unsolicited response could not be queued on the reused TLS connection");
    }
    if !third_avoids_surplus {
        failures.push("the third exchange was given the unsolicited response");
    }
    assert!(
        failures.is_empty(),
        "reused HTTPS connections must discard surplus responses: {failures:?}"
    );
}

fn assert_request(
    request: &CapturedRequest,
    expected_method: &str,
    expected_target: &str,
    expected_host: &str,
    expected_body: &[u8],
) {
    assert_eq!(request.method, expected_method, "the method is exact");
    assert_eq!(request.target, expected_target, "the target is origin-form");
    assert_eq!(request.version, "HTTP/1.1", "the request uses HTTP/1.1");
    let hosts = header_values(request, "host");
    assert_eq!(hosts.len(), 1, "the request has exactly one Host field");
    assert!(hosts[0] == expected_host, "Host is the endpoint authority");
    let content_types = header_values(request, "content-type");
    assert_eq!(
        content_types.len(),
        1,
        "the request has one Content-Type field"
    );
    assert!(
        content_types[0].eq_ignore_ascii_case("application/octet-stream"),
        "the request content type is octet-stream"
    );
    let content_lengths = header_values(request, "content-length");
    assert_eq!(
        content_lengths.len(),
        1,
        "the request has one Content-Length field"
    );
    assert_eq!(content_lengths[0], expected_body.len().to_string());
    let cache_controls = header_values(request, "cache-control");
    assert_eq!(
        cache_controls.len(),
        1,
        "the request has one Cache-Control field"
    );
    assert_eq!(cache_controls[0], "no-cache");
    assert!(
        request.body == expected_body,
        "the caller's bytes are unchanged"
    );
}

fn assert_request_owner_zeroized(observer: &secret::SecretBufferObserver, request_len: usize) {
    assert_eq!(
        observer.initialized_len(),
        request_len,
        "the request owner observes its complete initialized range before release"
    );
    assert!(
        observer.initialized_range_was_zero(),
        "the request owner's initialized bytes are zeroized before release"
    );
}

fn header_values<'a>(request: &'a CapturedRequest, name: &str) -> Vec<&'a str> {
    request
        .headers
        .iter()
        .filter(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
        .collect()
}

fn client_config(
    pki: &EphemeralPki,
    endpoint: String,
    target: Option<&str>,
    tls_server_name: Option<&str>,
) -> config::TransportConfig {
    let mut builder = config_builder(pki, endpoint);
    if let Some(target) = target {
        builder = builder.target_uri(target);
    }
    if let Some(server_name) = tls_server_name {
        builder = builder.tls_server_name(server_name);
    }
    builder
        .build()
        .expect("the explicit identity and trust inputs build a valid HTTPS config")
}

fn config_builder(pki: &EphemeralPki, endpoint: String) -> config::TransportConfigBuilder {
    let client_chain = pki
        .client_identity()
        .certificate_chain_der()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect();
    config::TransportConfig::builder(config::Endpoint::https(endpoint))
        .client_identity(config::ClientIdentity::new(
            config::CertificateInput::from_der(client_chain),
            config::PrivateKeyInput::from_der(pki.client_identity().private_key_der().to_vec()),
        ))
        .trust_source(config::TrustSource::certificate_authorities(vec![
            config::CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
}

fn fixed_resolver(address: SocketAddr) -> resolver::Resolver {
    resolver::Resolver::with_lookup_and_governor(
        move |_host, _port| Ok(vec![address]),
        Arc::new(tokio::sync::Semaphore::new(1)),
    )
}

fn server_config(pki: &EphemeralPki) -> Arc<ServerConfig> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the local server uses TLS 1.3 only");
    let mut client_roots = RootCertStore::empty();
    client_roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("the client test CA is valid");
    let verifier = WebPkiClientVerifier::builder(Arc::new(client_roots))
        .build()
        .expect("the peer requires a verified client certificate");
    let config = builder
        .with_client_cert_verifier(verifier)
        .with_single_cert(
            vec![CertificateDer::from(
                pki.server_identity().certificate_der().to_vec(),
            )],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                pki.server_identity().private_key_der().to_vec(),
            )),
        )
        .expect("the local server identity is valid");
    Arc::new(config)
}

struct CapturedRequest {
    method: String,
    target: String,
    version: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(Default)]
struct PeerObservation {
    accepted: bool,
    protocol_version: Option<rustls::ProtocolVersion>,
    client_identity_present: bool,
    close_notify_sent: bool,
    request: Option<CapturedRequest>,
}

fn spawn_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    respond: bool,
) -> JoinHandle<PeerObservation> {
    thread::spawn(move || run_peer(&listener, configuration, respond))
}

fn exchange_raw_response(
    pki: &EphemeralPki,
    response: &[u8],
    max_response_bytes: usize,
) -> Result<TransportResponse, TransportError> {
    let listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let address = listener.local_addr();
    let peer =
        spawn_raw_response_peer(listener.into_inner(), server_config(pki), response.to_vec());
    let config = client_config(
        pki,
        format!("https://{SERVER_NAME}:{}", address.port()),
        None,
        None,
    );
    let mut adapter = https::new_for_test_with_resolver(config, None, fixed_resolver(address));
    let result = adapter.exchange(fixtures::REQUEST_SENTINEL, max_response_bytes);
    let peer = peer
        .join()
        .expect("the bounded HTTPS response peer completes");
    assert!(
        peer.accepted,
        "the response peer accepted one TLS connection"
    );
    assert!(peer.request.is_some(), "the peer captured one HTTP request");
    result
}

fn response_with_header_count(total_headers: usize) -> Vec<u8> {
    let mut response =
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\n"
            .to_vec();
    for index in 2..total_headers {
        response.extend_from_slice(format!("X-Extra-{index}: value\r\n").as_bytes());
    }
    response.extend_from_slice(b"\r\n");
    response
}

fn response_with_header_block_size(total_size: usize) -> Vec<u8> {
    let prefix = b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\nX-Padding: ";
    let suffix = b"\r\n\r\n";
    let value_len = total_size
        .checked_sub(prefix.len() + suffix.len())
        .expect("the parser boundary exceeds fixed response header text");
    let mut response = Vec::with_capacity(total_size);
    response.extend_from_slice(prefix);
    response.resize(response.len() + value_len, b'a');
    response.extend_from_slice(suffix);
    assert_eq!(response.len(), total_size);
    response
}

fn spawn_raw_response_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    response: Vec<u8>,
) -> JoinHandle<PeerObservation> {
    spawn_response_peer(listener, configuration, response, false)
}

fn spawn_clean_tls_close_response_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    response: Vec<u8>,
) -> JoinHandle<PeerObservation> {
    spawn_response_peer(listener, configuration, response, true)
}

fn spawn_response_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    response: Vec<u8>,
    send_close_notify: bool,
) -> JoinHandle<PeerObservation> {
    thread::spawn(move || {
        let Ok((stream, _)) = accept_before_deadline(&listener) else {
            return PeerObservation::default();
        };
        let mut observation = PeerObservation {
            accepted: true,
            ..PeerObservation::default()
        };
        if stream.set_nonblocking(false).is_err()
            || stream.set_read_timeout(Some(PEER_TIMEOUT)).is_err()
            || stream.set_write_timeout(Some(PEER_TIMEOUT)).is_err()
        {
            return observation;
        }
        let Ok(connection) = ServerConnection::new(configuration) else {
            return observation;
        };
        let mut tls = StreamOwned::new(connection, stream);
        let handshake_deadline = Instant::now() + PEER_TIMEOUT;
        while tls.conn.is_handshaking() {
            if Instant::now() >= handshake_deadline || tls.conn.complete_io(&mut tls.sock).is_err()
            {
                return observation;
            }
        }
        observation.protocol_version = tls.conn.protocol_version();
        observation.client_identity_present = tls
            .conn
            .peer_certificates()
            .is_some_and(|certificates| !certificates.is_empty());
        observation.request = read_request(&mut tls);
        if observation.request.is_some() {
            if tls.write_all(&response).is_err() || tls.flush().is_err() {
                return observation;
            }
            if send_close_notify {
                tls.conn.send_close_notify();
                if tls.flush().is_err() {
                    return observation;
                }
                observation.close_notify_sent = true;
            }
        }
        observation
    })
}

fn spawn_gated_response_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    response_headers: Vec<u8>,
    body: Vec<u8>,
) -> (
    JoinHandle<PeerObservation>,
    mpsc::Receiver<()>,
    mpsc::SyncSender<()>,
) {
    let (headers_sent, headers_sent_rx) = mpsc::sync_channel(1);
    let (body_release, body_release_rx) = mpsc::sync_channel(1);
    let peer = thread::spawn(move || {
        let Ok((stream, _)) = accept_before_deadline(&listener) else {
            return PeerObservation::default();
        };
        let mut observation = PeerObservation {
            accepted: true,
            ..PeerObservation::default()
        };
        if stream.set_nonblocking(false).is_err()
            || stream.set_read_timeout(Some(PEER_TIMEOUT)).is_err()
            || stream.set_write_timeout(Some(PEER_TIMEOUT)).is_err()
        {
            return observation;
        }
        let Ok(connection) = ServerConnection::new(configuration) else {
            return observation;
        };
        let mut tls = StreamOwned::new(connection, stream);
        let handshake_deadline = Instant::now() + PEER_TIMEOUT;
        while tls.conn.is_handshaking() {
            if Instant::now() >= handshake_deadline || tls.conn.complete_io(&mut tls.sock).is_err()
            {
                return observation;
            }
        }
        observation.protocol_version = tls.conn.protocol_version();
        observation.client_identity_present = tls
            .conn
            .peer_certificates()
            .is_some_and(|certificates| !certificates.is_empty());
        observation.request = read_request(&mut tls);
        if observation.request.is_none() {
            return observation;
        }
        if tls.write_all(&response_headers).is_err() || tls.flush().is_err() {
            return observation;
        }
        let _ = headers_sent.send(());
        if body_release_rx.recv_timeout(PEER_TIMEOUT).is_ok() {
            let _ = tls.write_all(&body);
            let _ = tls.flush();
        }
        observation
    });
    (peer, headers_sent_rx, body_release)
}

#[derive(Default)]
struct SurplusPeerObservation {
    request_connection_ids: Vec<usize>,
    surplus_write_succeeded: bool,
}

fn spawn_surplus_response_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
) -> (
    JoinHandle<SurplusPeerObservation>,
    mpsc::SyncSender<()>,
    mpsc::Receiver<()>,
) {
    let observation = Arc::new(std::sync::Mutex::new(SurplusPeerObservation::default()));
    let (surplus_release, surplus_release_rx) = mpsc::sync_channel(1);
    let surplus_release_rx = Arc::new(std::sync::Mutex::new(surplus_release_rx));
    let (surplus_sent, surplus_sent_rx) = mpsc::sync_channel(1);
    let peer_observation = Arc::clone(&observation);
    let peer = thread::spawn(move || {
        let _ = listener.set_nonblocking(true);
        let accept_deadline = Instant::now() + Duration::from_secs(3);
        let mut connection_id = 0_usize;
        let mut handlers = Vec::new();
        while Instant::now() < accept_deadline && connection_id < 3 {
            match listener.accept() {
                Ok((stream, _)) => {
                    connection_id += 1;
                    let id = connection_id;
                    let configuration = Arc::clone(&configuration);
                    let observation = Arc::clone(&peer_observation);
                    let surplus_release = Arc::clone(&surplus_release_rx);
                    let surplus_sent = surplus_sent.clone();
                    handlers.push(thread::spawn(move || {
                        run_surplus_connection(
                            stream,
                            configuration,
                            id,
                            observation,
                            surplus_release,
                            surplus_sent,
                        );
                    }));
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
        for handler in handlers {
            let _ = handler.join();
        }
        let Ok(observation) = Arc::try_unwrap(peer_observation) else {
            return SurplusPeerObservation::default();
        };
        observation
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    });
    (peer, surplus_release, surplus_sent_rx)
}

fn run_surplus_connection(
    stream: TcpStream,
    configuration: Arc<ServerConfig>,
    connection_id: usize,
    observation: Arc<std::sync::Mutex<SurplusPeerObservation>>,
    surplus_release: Arc<std::sync::Mutex<mpsc::Receiver<()>>>,
    surplus_sent: mpsc::SyncSender<()>,
) {
    if stream.set_nonblocking(false).is_err()
        || stream.set_read_timeout(Some(PEER_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(PEER_TIMEOUT)).is_err()
    {
        return;
    }
    let Ok(connection) = ServerConnection::new(configuration) else {
        return;
    };
    let mut tls = StreamOwned::new(connection, stream);
    let handshake_deadline = Instant::now() + PEER_TIMEOUT;
    while tls.conn.is_handshaking() {
        if Instant::now() >= handshake_deadline || tls.conn.complete_io(&mut tls.sock).is_err() {
            return;
        }
    }

    while read_request(&mut tls).is_some() {
        let request_index = {
            let Ok(mut observation) = observation.lock() else {
                return;
            };
            observation.request_connection_ids.push(connection_id);
            observation.request_connection_ids.len() - 1
        };
        let (body, send_surplus) = match request_index {
            0 => (b"first".as_slice(), false),
            1 => (b"second".as_slice(), true),
            _ => (b"third".as_slice(), false),
        };
        if write_response_body(&mut tls, body).is_err() {
            return;
        }
        if send_surplus {
            let Ok(release) = surplus_release.lock() else {
                return;
            };
            if release.recv_timeout(PEER_TIMEOUT).is_err() {
                return;
            }
            let wrote_surplus = write_response_body(&mut tls, b"unsolicited").is_ok();
            if let Ok(mut observation) = observation.lock() {
                observation.surplus_write_succeeded = wrote_surplus;
            }
            let _ = surplus_sent.send(());
            if !wrote_surplus {
                return;
            }
        }
        if request_index >= 2 {
            return;
        }
    }
}

fn write_response_body(
    tls: &mut StreamOwned<ServerConnection, TcpStream>,
    body: &[u8],
) -> io::Result<()> {
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    tls.write_all(headers.as_bytes())?;
    tls.write_all(body)?;
    tls.flush()
}

fn spawn_stalled_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
) -> (JoinHandle<PeerObservation>, mpsc::Receiver<()>) {
    let (request_seen, request_receiver) = mpsc::sync_channel(1);
    let peer = thread::spawn(move || run_stalled_peer(&listener, configuration, &request_seen));
    (peer, request_receiver)
}

fn run_stalled_peer(
    listener: &TcpListener,
    configuration: Arc<ServerConfig>,
    request_seen: &mpsc::SyncSender<()>,
) -> PeerObservation {
    let Ok((stream, _)) = accept_before_deadline(listener) else {
        return PeerObservation::default();
    };
    let mut observation = PeerObservation {
        accepted: true,
        ..PeerObservation::default()
    };
    if stream.set_nonblocking(false).is_err()
        || stream.set_read_timeout(Some(PEER_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(PEER_TIMEOUT)).is_err()
    {
        return observation;
    }
    let Ok(connection) = ServerConnection::new(configuration) else {
        return observation;
    };
    let mut tls = StreamOwned::new(connection, stream);
    let handshake_deadline = Instant::now() + PEER_TIMEOUT;
    while tls.conn.is_handshaking() {
        if Instant::now() >= handshake_deadline || tls.conn.complete_io(&mut tls.sock).is_err() {
            return observation;
        }
    }
    observation.protocol_version = tls.conn.protocol_version();
    observation.client_identity_present = tls
        .conn
        .peer_certificates()
        .is_some_and(|certificates| !certificates.is_empty());
    observation.request = read_request(&mut tls);
    if observation.request.is_none() {
        return observation;
    }
    let _ = request_seen.send(());
    let mut byte = [0_u8; 1];
    let _ = tls.read(&mut byte);
    observation
}

fn run_peer(
    listener: &TcpListener,
    configuration: Arc<ServerConfig>,
    respond: bool,
) -> PeerObservation {
    let Ok((stream, _)) = accept_before_deadline(listener) else {
        return PeerObservation::default();
    };
    let mut observation = PeerObservation {
        accepted: true,
        ..PeerObservation::default()
    };
    if stream.set_nonblocking(false).is_err()
        || stream.set_read_timeout(Some(PEER_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(PEER_TIMEOUT)).is_err()
    {
        return observation;
    }
    let Ok(connection) = ServerConnection::new(configuration) else {
        return observation;
    };
    let mut tls = StreamOwned::new(connection, stream);
    let handshake_deadline = Instant::now() + PEER_TIMEOUT;
    while tls.conn.is_handshaking() {
        if Instant::now() >= handshake_deadline || tls.conn.complete_io(&mut tls.sock).is_err() {
            return observation;
        }
    }
    observation.protocol_version = tls.conn.protocol_version();
    observation.client_identity_present = tls
        .conn
        .peer_certificates()
        .is_some_and(|certificates| !certificates.is_empty());
    observation.request = read_request(&mut tls);
    if observation.request.is_none() {
        return observation;
    }
    if respond {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            RESPONSE_BODY.len()
        );
        if tls.write_all(response.as_bytes()).is_err()
            || tls.write_all(RESPONSE_BODY).is_err()
            || tls.flush().is_err()
        {
            return observation;
        }
    } else {
        let _ = tls.sock.shutdown(Shutdown::Both);
    }
    observation
}

fn read_request(tls: &mut StreamOwned<ServerConnection, TcpStream>) -> Option<CapturedRequest> {
    let mut bytes = Vec::new();
    loop {
        if let Some(request) = parse_complete_request(&bytes) {
            return Some(request);
        }
        if bytes.len() >= MAX_CAPTURED_REQUEST {
            return None;
        }
        let mut chunk = [0_u8; 2048];
        let read = tls.read(&mut chunk).ok()?;
        if read == 0 {
            return None;
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
}

fn parse_complete_request(bytes: &[u8]) -> Option<CapturedRequest> {
    let header_end = find_subslice(bytes, b"\r\n\r\n")?.checked_add(4)?;
    let header_text = std::str::from_utf8(&bytes[..header_end]).ok()?;
    let mut lines = header_text.split("\r\n");
    let mut request_line = lines.next()?.split_ascii_whitespace();
    let method = request_line.next()?.to_owned();
    let target = request_line.next()?.to_owned();
    let version = request_line.next()?.to_owned();
    if request_line.next().is_some() {
        return None;
    }
    let mut headers = Vec::new();
    let mut content_length = None;
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':')?;
        let name = name.to_ascii_lowercase();
        let value = value.trim().to_owned();
        if name == "content-length" {
            if content_length.is_some() {
                return None;
            }
            content_length = Some(value.parse::<usize>().ok()?);
        }
        headers.push((name, value));
    }
    let body_len = content_length?;
    let request_end = header_end.checked_add(body_len)?;
    if bytes.len() < request_end {
        return None;
    }
    Some(CapturedRequest {
        method,
        target,
        version,
        headers,
        body: bytes[header_end..request_end].to_vec(),
    })
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn accept_before_deadline(listener: &TcpListener) -> io::Result<(TcpStream, SocketAddr)> {
    listener.set_nonblocking(true)?;
    let deadline = Instant::now() + ACCEPT_TIMEOUT;
    loop {
        match listener.accept() {
            Ok(connection) => return Ok(connection),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "the bounded HTTPS peer accept expired",
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error),
        }
    }
}
