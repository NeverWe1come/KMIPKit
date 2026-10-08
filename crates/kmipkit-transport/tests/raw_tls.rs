//! Red-phase behavior contract for KMIPKIT-0013 raw TTLV over TLS.
//!
//! The source-included private adapter seam is intentionally test-only; this
//! target does not prescribe a public adapter type name. The accepted
//! low-level contract is caller-byte exchange over TLS 1.3/mTLS, with one
//! response frame per connection and zeroization of KMIPKit-owned staged
//! request bytes before release.

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use kmipkit_test_support::{EphemeralPki, LoopbackTcpListener, fixtures};
pub use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, CertificateRevocationListParams, DnType,
    ExtendedKeyUsagePurpose, IsCa, Issuer, KeyIdMethod, KeyPair, KeyUsagePurpose,
    RevokedCertParams, SerialNumber, date_time_ymd,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection};

#[path = "../src/config.rs"]
#[allow(dead_code)]
mod config;
#[path = "../src/raw_tls.rs"]
mod raw_tls;
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

const RESPONSE_FRAME: [u8; 8] = [0x42, 0x00, 0x78, 0x01, 0, 0, 0, 0];
const RESPONSE_FRAME_WITH_BODY: [u8; 16] = [
    0x42, 0x00, 0x78, 0x01, 0, 0, 0, 8, 0x42, 0x00, 0x01, 0x01, 0, 0, 0, 0,
];
const TWO_RESPONSE_FRAMES: [u8; 16] = [
    0x42, 0x00, 0x78, 0x01, 0, 0, 0, 0, 0x42, 0x00, 0x78, 0x01, 0, 0, 0, 0,
];
const BAD_ROOT_TAG_FRAME: [u8; 8] = [0x42, 0x00, 0x79, 0x01, 0, 0, 0, 0];
const BAD_ROOT_TYPE_FRAME: [u8; 8] = [0x42, 0x00, 0x78, 0x02, 0, 0, 0, 0];
const UNALIGNED_LENGTH_HEADER: [u8; 8] = [0x42, 0x00, 0x78, 0x01, 0, 0, 0, 1];
const OVERSIZED_LENGTH_HEADER: [u8; 8] = [0x42, 0x00, 0x78, 0x01, 0xff, 0xff, 0xff, 0xf8];
const PARTIAL_RESPONSE_HEADER: [u8; 4] = [0x42, 0x00, 0x78, 0x01];
const TRUNCATED_RESPONSE_BODY: [u8; 12] =
    [0x42, 0x00, 0x78, 0x01, 0, 0, 0, 8, 0xA5, 0x5A, 0xC3, 0x3C];
const RESPONSE_LIMIT: usize = RESPONSE_FRAME.len();
const TLS_TEST_TIMEOUT: Duration = Duration::from_secs(2);
const PEER_ACCEPT_TIMEOUT: Duration = Duration::from_secs(2);
const PEER_ACCEPT_POLL_INTERVAL: Duration = Duration::from_millis(10);
const PEER_OPERATION_TIMEOUT: Duration = Duration::from_secs(6);
const SERVER_NAME: &str = "server.kmipkit.test";

/// Proves the caller's exact bytes reach a TLS 1.3 peer that requires mTLS,
/// that one response frame is returned, that the connection closes, and that
/// the staged request owner is zeroized on success.
#[test]
fn raw_tls_sends_exact_bytes_over_tls13_mtls_and_closes_after_one_frame() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the TLS peer binds loopback");
    let address = listener.local_addr();
    let peer_config = server_config(&pki, true);
    let peer = spawn_peer(
        listener.into_inner(),
        peer_config,
        fixtures::REQUEST_SENTINEL.len(),
        PeerAction::RespondOnce,
    );
    let config = client_config(&pki, &pki, address.port(), SERVER_NAME, Vec::new());
    let observer = secret::SecretBufferObserver::new(fixtures::REQUEST_SENTINEL.len());

    let result = exchange(config, fixtures::REQUEST_SENTINEL, Some(observer.clone()));
    let response = result.expect("the verified TLS 1.3 exchange succeeds");
    assert_eq!(response.as_bytes(), RESPONSE_FRAME);

    let peer = peer.join().expect("the local TLS peer thread completes");
    assert!(peer.handshake_completed, "the mTLS handshake completes");
    assert_eq!(
        peer.protocol_version,
        Some(rustls::ProtocolVersion::TLSv1_3),
        "the peer negotiates TLS 1.3"
    );
    assert!(
        peer.client_identity_present,
        "the peer receives the configured client identity"
    );
    assert!(
        peer.request_bytes.as_slice() == fixtures::REQUEST_SENTINEL,
        "the peer receives the caller bytes unchanged"
    );
    assert!(
        peer.closed_after_response,
        "the raw connection closes after the single response frame"
    );
    assert_request_owner_zeroized(&observer, fixtures::REQUEST_SENTINEL.len());
}

/// Proves the staged request owner is cleaned up when the peer closes after
/// receiving the request without returning a response frame.
#[test]
fn raw_tls_zeroizes_staged_request_after_peer_closes_without_response() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the TLS peer binds loopback");
    let address = listener.local_addr();
    let peer_config = server_config(&pki, true);
    let peer = spawn_peer(
        listener.into_inner(),
        peer_config,
        fixtures::REQUEST_SENTINEL.len(),
        PeerAction::CloseWithoutResponse,
    );
    let config = client_config(&pki, &pki, address.port(), SERVER_NAME, Vec::new());
    let observer = secret::SecretBufferObserver::new(fixtures::REQUEST_SENTINEL.len());

    let result = exchange(config, fixtures::REQUEST_SENTINEL, Some(observer.clone()));
    assert!(result.is_err(), "an incomplete response is rejected");

    let peer = peer.join().expect("the local TLS peer thread completes");
    assert!(peer.handshake_completed, "the mTLS handshake completes");
    assert_eq!(
        result.as_ref().unwrap_err().delivery_state(),
        RequestDeliveryState::PossiblySent,
        "EOF before the response header follows a committed request"
    );
    assert!(
        peer.request_bytes.as_slice() == fixtures::REQUEST_SENTINEL,
        "the peer receives the caller bytes before it closes"
    );
    assert_request_owner_zeroized(&observer, fixtures::REQUEST_SENTINEL.len());
}

#[test]
fn raw_tls_rejects_a_partial_response_header() {
    let (result, peer) = exchange_responding(&PARTIAL_RESPONSE_HEADER, RESPONSE_LIMIT);

    assert_response_started(&result, "a partial response header is rejected");
    assert_peer_closed_after_response(&peer);
}

#[test]
fn raw_tls_rejects_eof_after_a_complete_header_before_the_value() {
    let (result, peer) = exchange_responding(&RESPONSE_FRAME_WITH_BODY[..8], 16);

    assert_response_started(
        &result,
        "EOF after the response header but before its value is rejected",
    );
    assert_peer_closed_after_response(&peer);
}

#[test]
fn raw_tls_rejects_a_response_with_an_invalid_root_tag() {
    let (result, peer) = exchange_responding(&BAD_ROOT_TAG_FRAME, RESPONSE_LIMIT);

    assert_response_started(&result, "an invalid response root tag is rejected");
    assert_peer_closed_after_response(&peer);
}

#[test]
fn raw_tls_rejects_a_response_with_an_invalid_root_type() {
    let (result, peer) = exchange_responding(&BAD_ROOT_TYPE_FRAME, RESPONSE_LIMIT);

    assert_response_started(&result, "an invalid response root type is rejected");
    assert_peer_closed_after_response(&peer);
}

#[test]
fn raw_tls_rejects_an_unaligned_response_length() {
    let (result, peer) = exchange_responding(&UNALIGNED_LENGTH_HEADER, RESPONSE_LIMIT);

    assert_response_started(&result, "an unaligned response value length is rejected");
    assert_peer_closed_after_response(&peer);
}

#[test]
fn raw_tls_rejects_an_aligned_response_length_over_the_limit_before_body_read() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the TLS peer binds loopback");
    let address = listener.local_addr();
    let peer = spawn_peer(
        listener.into_inner(),
        server_config(&pki, true),
        fixtures::REQUEST_SENTINEL.len(),
        PeerAction::RespondBytes(&OVERSIZED_LENGTH_HEADER),
    );
    let config = client_config(&pki, &pki, address.port(), SERVER_NAME, Vec::new());
    let allocation_observer = raw_tls::ResponseAllocationObserver::new();
    let mut adapter = raw_tls::new_for_test_with_response_allocation_observer(
        config,
        allocation_observer.clone(),
    );
    let result = adapter.exchange(fixtures::REQUEST_SENTINEL, RESPONSE_LIMIT);
    let peer = peer.join().expect("the bounded local TLS peer completes");

    assert_response_started(
        &result,
        "a response exceeding the configured cap is rejected",
    );
    assert_peer_closed_after_response(&peer);
    assert_eq!(
        allocation_observer.allocation_count(),
        0,
        "the oversized header is rejected before a response allocation is attempted"
    );
}

#[test]
fn raw_tls_accepts_a_response_exactly_at_the_configured_limit() {
    let (result, peer) =
        exchange_responding(&RESPONSE_FRAME_WITH_BODY, RESPONSE_FRAME_WITH_BODY.len());

    let response = result.expect("a response at the configured limit is accepted");
    assert_eq!(response.as_bytes(), RESPONSE_FRAME_WITH_BODY);
    assert_peer_closed_after_response(&peer);
}

#[test]
fn raw_tls_rejects_a_response_one_byte_over_the_configured_limit() {
    let (result, peer) = exchange_responding(
        &RESPONSE_FRAME_WITH_BODY,
        RESPONSE_FRAME_WITH_BODY.len() - 1,
    );

    assert_response_started(
        &result,
        "a response one byte over the configured cap is rejected",
    );
    assert_peer_closed_after_response(&peer);
}

#[test]
fn raw_tls_rejects_a_truncated_response_body_and_zeroizes_its_owner() {
    let (result, peer) =
        exchange_responding(&TRUNCATED_RESPONSE_BODY, RESPONSE_FRAME_WITH_BODY.len());

    assert_response_started(&result, "a truncated response body is rejected");
    assert_peer_closed_after_response(&peer);
}

#[test]
fn raw_tls_returns_only_the_first_of_two_coalesced_response_frames() {
    let (result, peer) = exchange_action(PeerAction::RespondCoalesced, RESPONSE_LIMIT);

    let response = result.expect("the first complete response frame is accepted");
    assert_eq!(response.as_bytes(), RESPONSE_FRAME);
    assert_peer_closed_after_response(&peer);
}

#[test]
fn raw_tls_reconnects_after_a_failed_response_without_replaying_the_first_request() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the TLS peer binds loopback");
    let address = listener.local_addr();
    let peer_config = server_config(&pki, true);
    let listener = listener.into_inner();
    let first_peer_config = Arc::clone(&peer_config);
    let second_peer_config = Arc::clone(&peer_config);
    let peer = thread::spawn(move || {
        let first = run_peer(
            &listener,
            first_peer_config,
            fixtures::REQUEST_SENTINEL.len(),
            PeerAction::RespondBytes(&BAD_ROOT_TAG_FRAME),
        );
        let second = run_peer(
            &listener,
            second_peer_config,
            fixtures::REQUEST_SENTINEL.len(),
            PeerAction::RespondBytes(&RESPONSE_FRAME),
        );
        (first, second)
    });
    let config = client_config(&pki, &pki, address.port(), SERVER_NAME, Vec::new());
    let mut adapter = raw_tls::new_for_test(config, None);

    let first_result = adapter.exchange(fixtures::REQUEST_SENTINEL, RESPONSE_LIMIT);
    assert_response_started(&first_result, "the malformed first response is rejected");
    let second_result = adapter.exchange(fixtures::REQUEST_SENTINEL, RESPONSE_LIMIT);
    let response = second_result.expect("a later distinct call reconnects successfully");

    assert_eq!(response.as_bytes(), RESPONSE_FRAME);
    let (first_peer, second_peer) = peer
        .join()
        .expect("the bounded TLS peer sequence completes");
    for peer in [&first_peer, &second_peer] {
        assert!(peer.accepted, "each distinct call opens a connection");
        assert!(peer.handshake_completed, "each connection completes mTLS");
        assert_eq!(
            peer.request_bytes.as_slice(),
            fixtures::REQUEST_SENTINEL,
            "each connection receives only its own caller request"
        );
        assert!(
            peer.closed_after_response,
            "each connection closes after its frame"
        );
    }
}

#[cfg(target_pointer_width = "32")]
#[test]
fn raw_tls_rejects_response_length_that_overflows_checked_total_size() {
    let (result, peer) = exchange_responding(&OVERSIZED_LENGTH_HEADER, usize::MAX);

    assert_response_started(&result, "a response whose total size overflows is rejected");
    assert_peer_closed_after_response(&peer);
}

/// Proves the adapter rejects a server chain rooted in a CA the caller did
/// not select.
#[test]
fn raw_tls_rejects_unknown_server_ca() {
    let server_pki = PolicyPki::generate(SERVER_NAME, false);
    let untrusted_pki = EphemeralPki::generate().expect("a distinct test CA is generated");
    let listener = LoopbackTcpListener::bind().expect("the TLS peer binds loopback");
    let address = listener.local_addr();
    let peer = spawn_peer(
        listener.into_inner(),
        server_config(&server_pki, true),
        0,
        PeerAction::CloseAfterHandshake,
    );
    let config = client_config(
        &server_pki,
        &untrusted_pki,
        address.port(),
        SERVER_NAME,
        Vec::new(),
    );

    let result = exchange(config, fixtures::REQUEST_SENTINEL, None);
    assert_not_sent(
        &result,
        "an unknown server CA is rejected before request dispatch",
    );
    let peer = peer.join().expect("the local TLS peer thread completes");
    assert!(peer.accepted, "the local peer accepted the TLS connection");
    assert!(
        !peer.handshake_completed,
        "the server handshake does not complete with an untrusted CA"
    );
}

/// Proves the adapter rejects a server certificate outside its validity
/// period.
#[test]
fn raw_tls_rejects_expired_server_certificate() {
    let pki = PolicyPki::generate(SERVER_NAME, true);
    let listener = LoopbackTcpListener::bind().expect("the TLS peer binds loopback");
    let address = listener.local_addr();
    let peer = spawn_peer(
        listener.into_inner(),
        server_config(&pki, true),
        0,
        PeerAction::CloseAfterHandshake,
    );
    let config = client_config(&pki, &pki, address.port(), SERVER_NAME, Vec::new());

    let result = exchange(config, fixtures::REQUEST_SENTINEL, None);
    assert_not_sent(
        &result,
        "an expired server certificate is rejected before request dispatch",
    );
    let peer = peer.join().expect("the local TLS peer thread completes");
    assert!(peer.accepted, "the local peer accepted the TLS connection");
    assert!(
        !peer.handshake_completed,
        "the expired server certificate prevents handshake completion"
    );
}

/// Proves hostname validation uses the configured verification name.
#[test]
fn raw_tls_rejects_server_name_mismatch() {
    let pki = PolicyPki::generate(SERVER_NAME, false);
    let listener = LoopbackTcpListener::bind().expect("the TLS peer binds loopback");
    let address = listener.local_addr();
    let peer = spawn_peer(
        listener.into_inner(),
        server_config(&pki, true),
        0,
        PeerAction::CloseAfterHandshake,
    );
    let config = client_config(
        &pki,
        &pki,
        address.port(),
        "wrong-server.kmipkit.test",
        Vec::new(),
    );

    let result = exchange(config, fixtures::REQUEST_SENTINEL, None);
    assert_not_sent(
        &result,
        "a hostname mismatch is rejected before request dispatch",
    );
    let peer = peer.join().expect("the local TLS peer thread completes");
    assert!(peer.accepted, "the local peer accepted the TLS connection");
    assert!(
        !peer.handshake_completed,
        "hostname validation prevents handshake completion"
    );
}

/// Proves caller-supplied CRL evidence rejects a revoked server certificate.
#[test]
fn raw_tls_rejects_server_certificate_revoked_by_caller_crl() {
    let pki = PolicyPki::generate(SERVER_NAME, false);
    let listener = LoopbackTcpListener::bind().expect("the TLS peer binds loopback");
    let address = listener.local_addr();
    let peer = spawn_peer(
        listener.into_inner(),
        server_config(&pki, true),
        0,
        PeerAction::CloseAfterHandshake,
    );
    let config = client_config(
        &pki,
        &pki,
        address.port(),
        SERVER_NAME,
        vec![pki.revoking_server_crl()],
    );

    let result = exchange(config, fixtures::REQUEST_SENTINEL, None);
    assert_not_sent(
        &result,
        "a caller-revoked server certificate is rejected before request dispatch",
    );
    let peer = peer.join().expect("the local TLS peer thread completes");
    assert!(peer.accepted, "the local peer accepted the TLS connection");
    assert!(
        !peer.handshake_completed,
        "CRL verification prevents handshake completion"
    );
}

fn exchange(
    config: config::TransportConfig,
    request: &[u8],
    observer: Option<secret::SecretBufferObserver>,
) -> Result<TransportResponse, TransportError> {
    exchange_with_limit(config, request, observer, RESPONSE_LIMIT)
}

fn exchange_with_limit(
    config: config::TransportConfig,
    request: &[u8],
    observer: Option<secret::SecretBufferObserver>,
    max_response_bytes: usize,
) -> Result<TransportResponse, TransportError> {
    let mut adapter = raw_tls::new_for_test(config, observer);
    adapter.exchange(request, max_response_bytes)
}

fn exchange_responding(
    response: &'static [u8],
    max_response_bytes: usize,
) -> (Result<TransportResponse, TransportError>, PeerObservation) {
    exchange_action(PeerAction::RespondBytes(response), max_response_bytes)
}

fn exchange_action(
    action: PeerAction,
    max_response_bytes: usize,
) -> (Result<TransportResponse, TransportError>, PeerObservation) {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let listener = LoopbackTcpListener::bind().expect("the TLS peer binds loopback");
    let address = listener.local_addr();
    let peer = spawn_peer(
        listener.into_inner(),
        server_config(&pki, true),
        fixtures::REQUEST_SENTINEL.len(),
        action,
    );
    let config = client_config(&pki, &pki, address.port(), SERVER_NAME, Vec::new());
    let result = exchange_with_limit(config, fixtures::REQUEST_SENTINEL, None, max_response_bytes);
    let peer = peer.join().expect("the bounded local TLS peer completes");
    assert!(peer.handshake_completed, "the mTLS handshake completes");
    assert_eq!(
        peer.request_bytes.as_slice(),
        fixtures::REQUEST_SENTINEL,
        "the peer receives the caller request unchanged"
    );
    (result, peer)
}

fn assert_response_started(result: &Result<TransportResponse, TransportError>, message: &str) {
    let Err(error) = result else {
        panic!("{message}");
    };
    assert_eq!(
        error.delivery_state(),
        RequestDeliveryState::ResponseStarted,
        "{message}"
    );
}

fn assert_peer_closed_after_response(peer: &PeerObservation) {
    assert!(peer.accepted, "the local peer accepted the TLS connection");
    assert!(
        peer.closed_after_response,
        "the raw connection is closed after its frame"
    );
}

fn assert_request_owner_zeroized(observer: &secret::SecretBufferObserver, request_len: usize) {
    assert_eq!(
        observer.initialized_len(),
        request_len,
        "the request owner observes the full initialized range before release"
    );
    assert!(
        observer.initialized_range_was_zero(),
        "the request owner's initialized bytes are zeroized before release"
    );
}

fn assert_not_sent(result: &Result<TransportResponse, TransportError>, message: &str) {
    let Err(error) = result else {
        panic!("{message}");
    };
    assert_eq!(
        error.delivery_state(),
        RequestDeliveryState::NotSent,
        "{message}"
    );
}

fn client_config(
    identity_pki: &impl PkiMaterial,
    trust_pki: &impl PkiMaterial,
    port: u16,
    server_name: &str,
    revocation_lists: Vec<config::RevocationListInput>,
) -> config::TransportConfig {
    config::TransportConfig::builder(config::Endpoint::raw_tls("127.0.0.1", port))
        .client_identity(config::ClientIdentity::new(
            config::CertificateInput::from_der(identity_pki.client_chain_der()),
            config::PrivateKeyInput::from_der(identity_pki.client_key_der()),
        ))
        .trust_source(config::TrustSource::certificate_authorities(vec![
            config::CertificateInput::from_der(vec![trust_pki.authority_der()]),
        ]))
        .revocation_lists(revocation_lists)
        .tls_server_name(server_name)
        .build()
        .expect("the explicit identity and trust inputs build a valid config")
}

trait PkiMaterial {
    fn authority_der(&self) -> Vec<u8>;
    fn client_chain_der(&self) -> Vec<Vec<u8>>;
    fn client_key_der(&self) -> Vec<u8>;
    fn server_certificate_der(&self) -> Vec<u8>;
    fn server_key_der(&self) -> Vec<u8>;
}

impl PkiMaterial for EphemeralPki {
    fn authority_der(&self) -> Vec<u8> {
        self.authority_certificate_der().to_vec()
    }

    fn client_chain_der(&self) -> Vec<Vec<u8>> {
        self.client_identity()
            .certificate_chain_der()
            .into_iter()
            .map(<[u8]>::to_vec)
            .collect()
    }

    fn client_key_der(&self) -> Vec<u8> {
        self.client_identity().private_key_der().to_vec()
    }

    fn server_certificate_der(&self) -> Vec<u8> {
        self.server_identity().certificate_der().to_vec()
    }

    fn server_key_der(&self) -> Vec<u8> {
        self.server_identity().private_key_der().to_vec()
    }
}

fn server_config(pki: &impl PkiMaterial, require_client_auth: bool) -> Arc<ServerConfig> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the local server uses TLS 1.3 only");
    let mut client_roots = RootCertStore::empty();
    client_roots
        .add(CertificateDer::from(pki.authority_der()))
        .expect("the client test CA is valid");
    let builder = if require_client_auth {
        let verifier = WebPkiClientVerifier::builder(Arc::new(client_roots))
            .build()
            .expect("the peer requires a verified client certificate");
        builder.with_client_cert_verifier(verifier)
    } else {
        builder.with_no_client_auth()
    };
    let config = builder
        .with_single_cert(
            vec![CertificateDer::from(pki.server_certificate_der())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(pki.server_key_der())),
        )
        .expect("the local server identity is valid");
    Arc::new(config)
}

#[derive(Clone, Copy)]
enum PeerAction {
    RespondOnce,
    RespondBytes(&'static [u8]),
    RespondCoalesced,
    CloseWithoutResponse,
    CloseAfterHandshake,
}

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)] // These fields record independent peer observations.
struct PeerObservation {
    accepted: bool,
    handshake_completed: bool,
    protocol_version: Option<rustls::ProtocolVersion>,
    client_identity_present: bool,
    request_bytes: Vec<u8>,
    closed_after_response: bool,
}

fn spawn_peer(
    listener: TcpListener,
    config: Arc<ServerConfig>,
    request_len: usize,
    action: PeerAction,
) -> thread::JoinHandle<PeerObservation> {
    thread::spawn(move || run_peer(&listener, config, request_len, action))
}

fn run_peer(
    listener: &TcpListener,
    config: Arc<ServerConfig>,
    request_len: usize,
    action: PeerAction,
) -> PeerObservation {
    let Ok((mut stream, _)) = accept_before_deadline(listener) else {
        return PeerObservation::default();
    };
    let mut observation = PeerObservation {
        accepted: true,
        ..PeerObservation::default()
    };
    if stream.set_nonblocking(false).is_err()
        || stream.set_read_timeout(Some(TLS_TEST_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(TLS_TEST_TIMEOUT)).is_err()
    {
        return observation;
    }
    let Ok(mut connection) = ServerConnection::new(config) else {
        return observation;
    };
    let deadline = Instant::now() + PEER_OPERATION_TIMEOUT;
    while connection.is_handshaking() {
        if Instant::now() >= deadline || connection.complete_io(&mut stream).is_err() {
            return observation;
        }
    }

    observation.handshake_completed = true;
    observation.protocol_version = connection.protocol_version();
    observation.client_identity_present = connection
        .peer_certificates()
        .is_some_and(|certificates| !certificates.is_empty());
    if request_len == 0
        || !read_plaintext_exact(
            &mut connection,
            &mut stream,
            deadline,
            request_len,
            &mut observation.request_bytes,
        )
    {
        return observation;
    }

    match action {
        PeerAction::RespondOnce | PeerAction::RespondBytes(_) | PeerAction::RespondCoalesced => {
            let response_write = match action {
                PeerAction::RespondOnce => connection.writer().write_all(&RESPONSE_FRAME),
                PeerAction::RespondBytes(bytes) => connection.writer().write_all(bytes),
                PeerAction::RespondCoalesced => connection.writer().write_all(&TWO_RESPONSE_FRAMES),
                PeerAction::CloseWithoutResponse | PeerAction::CloseAfterHandshake => {
                    return observation;
                }
            };
            if response_write.is_err() {
                return observation;
            }
            while connection.wants_write() {
                if Instant::now() >= deadline || connection.write_tls(&mut stream).is_err() {
                    return observation;
                }
            }
            if stream.shutdown(Shutdown::Write).is_err() {
                return observation;
            }
            let mut encrypted = [0_u8; 1024];
            loop {
                if Instant::now() >= deadline {
                    break;
                }
                match stream.read(&mut encrypted) {
                    Ok(0) => {
                        observation.closed_after_response = true;
                        break;
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }
        PeerAction::CloseWithoutResponse | PeerAction::CloseAfterHandshake => {}
    }
    observation
}

fn accept_before_deadline(listener: &TcpListener) -> io::Result<(TcpStream, std::net::SocketAddr)> {
    listener.set_nonblocking(true)?;
    let deadline = Instant::now() + PEER_ACCEPT_TIMEOUT;
    loop {
        match listener.accept() {
            Ok((stream, address)) => return Ok((stream, address)),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                let now = Instant::now();
                if now >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "local TLS peer accept deadline elapsed",
                    ));
                }
                thread::sleep(
                    PEER_ACCEPT_POLL_INTERVAL.min(deadline.saturating_duration_since(now)),
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn read_plaintext_exact(
    connection: &mut ServerConnection,
    stream: &mut TcpStream,
    deadline: Instant,
    expected_len: usize,
    bytes: &mut Vec<u8>,
) -> bool {
    bytes.resize(expected_len, 0);
    let mut initialized = 0;
    while initialized < expected_len {
        if Instant::now() >= deadline {
            return false;
        }
        match connection.reader().read(&mut bytes[initialized..]) {
            Ok(0) => return false,
            Ok(read) => initialized += read,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if connection.complete_io(stream).is_err() {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }
    true
}

const SERVER_SERIAL: u64 = 41;
const CLIENT_SERIAL: u64 = 42;

struct PolicyPki {
    ca_params: CertificateParams,
    ca_key: KeyPair,
    ca: Certificate,
    server: Certificate,
    server_key: KeyPair,
    client: Certificate,
    client_key: KeyPair,
    server_serial: SerialNumber,
}

impl PolicyPki {
    fn generate(server_name: &str, expired_server: bool) -> Self {
        let ca_key = KeyPair::generate().expect("test CA key generation succeeds");
        let mut ca_params = CertificateParams::default();
        ca_params
            .distinguished_name
            .push(DnType::CommonName, "KMIPKit raw TLS test CA");
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        ca_params.not_before = date_time_ymd(2020, 1, 1);
        ca_params.not_after = date_time_ymd(2099, 1, 1);
        let ca = ca_params
            .self_signed(&ca_key)
            .expect("test CA certificate generation succeeds");
        let issuer = Issuer::from_params(&ca_params, &ca_key);
        let server_serial = SerialNumber::from(SERVER_SERIAL);
        let (server, server_key) = policy_leaf(
            server_name,
            ExtendedKeyUsagePurpose::ServerAuth,
            server_serial.clone(),
            expired_server,
            &issuer,
        );
        let (client, client_key) = policy_leaf(
            "client.kmipkit.test",
            ExtendedKeyUsagePurpose::ClientAuth,
            SerialNumber::from(CLIENT_SERIAL),
            false,
            &issuer,
        );
        Self {
            ca_params,
            ca_key,
            ca,
            server,
            server_key,
            client,
            client_key,
            server_serial,
        }
    }

    fn revoking_server_crl(&self) -> config::RevocationListInput {
        let issuer = Issuer::from_params(&self.ca_params, &self.ca_key);
        let crl = CertificateRevocationListParams {
            this_update: date_time_ymd(2020, 1, 1),
            next_update: date_time_ymd(2099, 1, 1),
            crl_number: SerialNumber::from(1_u64),
            issuing_distribution_point: None,
            revoked_certs: vec![RevokedCertParams {
                serial_number: self.server_serial.clone(),
                revocation_time: date_time_ymd(2025, 1, 1),
                reason_code: None,
                invalidity_date: None,
            }],
            key_identifier_method: KeyIdMethod::Sha256,
        }
        .signed_by(&issuer)
        .expect("test CRL signing succeeds");
        config::RevocationListInput::from_der(vec![crl.der().as_ref().to_vec()])
    }
}

impl PkiMaterial for PolicyPki {
    fn authority_der(&self) -> Vec<u8> {
        self.ca.der().as_ref().to_vec()
    }

    fn client_chain_der(&self) -> Vec<Vec<u8>> {
        vec![
            self.client.der().as_ref().to_vec(),
            self.ca.der().as_ref().to_vec(),
        ]
    }

    fn client_key_der(&self) -> Vec<u8> {
        self.client_key.serialize_der()
    }

    fn server_certificate_der(&self) -> Vec<u8> {
        self.server.der().as_ref().to_vec()
    }

    fn server_key_der(&self) -> Vec<u8> {
        self.server_key.serialize_der()
    }
}

fn policy_leaf(
    name: &str,
    usage: ExtendedKeyUsagePurpose,
    serial_number: SerialNumber,
    expired: bool,
    issuer: &Issuer<'_, &KeyPair>,
) -> (Certificate, KeyPair) {
    let key = KeyPair::generate().expect("test leaf key generation succeeds");
    let mut params = CertificateParams::new(vec![name.to_owned()])
        .expect("test subject alternative name is valid");
    params.distinguished_name.push(DnType::CommonName, name);
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    params.extended_key_usages = vec![usage];
    params.serial_number = Some(serial_number);
    if expired {
        params.not_before = date_time_ymd(1998, 1, 1);
        params.not_after = date_time_ymd(1999, 1, 1);
    } else {
        params.not_before = date_time_ymd(2020, 1, 1);
        params.not_after = date_time_ymd(2099, 1, 1);
    }
    let certificate = params
        .signed_by(&key, issuer)
        .expect("test leaf certificate signing succeeds");
    (certificate, key)
}
