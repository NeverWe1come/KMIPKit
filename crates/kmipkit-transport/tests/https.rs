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
    if started && !returned_while_gate_was_closed {
        if let Ok(event) = events_rx.recv_timeout(CLEANUP_GATE_OBSERVATION_WINDOW) {
            observed_events.push(event);
        }
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
    request: Option<CapturedRequest>,
}

fn spawn_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    respond: bool,
) -> JoinHandle<PeerObservation> {
    thread::spawn(move || run_peer(&listener, configuration, respond))
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
