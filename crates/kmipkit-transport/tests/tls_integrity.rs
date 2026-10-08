//! Derived negative TLS integrity checks using a loopback record-corrupting relay.

use std::error::Error as _;
use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use kmipkit_test_support::{EphemeralPki, LoopbackTcpListener, fixtures};
use kmipkit_transport::{
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

pub use config::{RequestOptions, TimeoutLimit, TimeoutPolicy, TransportConfig};

const SERVER_NAME: &str = "server.kmipkit.test";
const RESPONSE_SENTINEL: &[u8] = b"TLS_RESPONSE_SENTINEL_0123456789";
const TEST_TIMEOUT: Duration = Duration::from_secs(6);
const RAW_RESPONSE: [u8; 40] = [
    0x42, 0x00, 0x78, 0x01, 0, 0, 0, 32, b'T', b'L', b'S', b'_', b'R', b'E', b'S', b'P', b'O',
    b'N', b'S', b'E', b'_', b'S', b'E', b'N', b'T', b'I', b'N', b'E', b'L', b'_', b'0', b'1', b'2',
    b'3', b'4', b'5', b'6', b'7', b'8', b'9',
];

#[test]
fn raw_tls_rejects_a_tampered_post_handshake_application_record() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let peer_listener = LoopbackTcpListener::bind().expect("the raw TLS peer binds loopback");
    let peer_address = peer_listener.local_addr();
    let (peer, request_seen, release_response) = spawn_gated_peer(
        peer_listener.into_inner(),
        server_config(&pki),
        PeerProtocol::RawTtlv,
    );
    let relay = TlsRecordRelay::start(peer_address).expect("the TLS record relay binds loopback");
    let relay_address = relay.address();
    let configuration = raw_tls_configuration(&pki, relay_address.port());
    let mut adapter =
        raw_tls::new_for_test_with_resolver(configuration, None, fixed_resolver(relay_address));
    let client = thread::spawn(move || {
        let result = adapter.exchange(fixtures::REQUEST_SENTINEL, RAW_RESPONSE.len());
        (adapter, result)
    });

    // The peer reports only after mTLS has completed and it has read the
    // decrypted request. The relay remains passive until this event.
    let peer_after_request = request_seen
        .recv_timeout(TEST_TIMEOUT)
        .expect("the peer completes mTLS and reads the raw request before its response");
    assert_peer_received_request(&peer_after_request, fixtures::REQUEST_SENTINEL);
    relay.arm_response_record_corruption();
    release_response
        .send(())
        .expect("the gated peer accepts the release after the relay is armed");

    let (_adapter, result) = client
        .join()
        .expect("the raw TLS exchange thread completes");
    let peer = peer.join().expect("the raw TLS peer thread completes");
    let relay = relay.join();

    assert_peer_response_was_written(&peer);
    assert!(relay.accepted, "the relay carried the real TLS connection");
    assert_eq!(relay.corrupted_application_data_records, 1);
    assert!(
        relay.client_connection_terminated,
        "the failed raw TLS connection closes"
    );
    assert_tamper_error(result);
}

#[test]
fn https_invalidates_a_connection_after_a_tampered_post_handshake_application_record() {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let peer_listener = LoopbackTcpListener::bind().expect("the HTTPS peer binds loopback");
    let peer_address = peer_listener.local_addr();
    let (peer, request_seen, release_response) = spawn_gated_peer(
        peer_listener.into_inner(),
        server_config(&pki),
        PeerProtocol::Https,
    );
    let relay = TlsRecordRelay::start(peer_address).expect("the TLS record relay binds loopback");
    let relay_address = relay.address();
    let endpoint = format!("https://{SERVER_NAME}:{}", relay_address.port());
    let configuration = https_configuration(&pki, endpoint);
    let mut adapter =
        https::new_for_test_with_resolver(configuration, None, fixed_resolver(relay_address));
    let client = thread::spawn(move || {
        let result = adapter.exchange(fixtures::REQUEST_SENTINEL, 128);
        (adapter, result)
    });

    // The peer reports only after mTLS has completed and it has read the
    // decrypted request; the relay is armed before the peer releases data.
    let peer_after_request = request_seen
        .recv_timeout(TEST_TIMEOUT)
        .expect("the peer completes mTLS and reads the HTTPS request before its response");
    assert_peer_received_request(&peer_after_request, fixtures::REQUEST_SENTINEL);
    relay.arm_response_record_corruption();
    release_response
        .send(())
        .expect("the gated peer accepts the release after the relay is armed");

    let (adapter, result) = client.join().expect("the HTTPS exchange thread completes");
    let peer = peer.join().expect("the HTTPS peer thread completes");
    let relay = relay.join();

    assert_peer_response_was_written(&peer);
    assert!(relay.accepted, "the relay carried the real TLS connection");
    assert_eq!(relay.corrupted_application_data_records, 1);
    assert!(
        !https::has_cached_connection_for_test(&adapter),
        "the TLS-failed HTTPS connection is invalidated before another exchange"
    );
    assert_tamper_error(result);
}

fn assert_tamper_error(result: Result<TransportResponse, TransportError>) {
    let error = result.expect_err("the TLS record authentication failure rejects the response");
    assert_eq!(error.cause_category(), TransportCauseCategory::Tls);
    assert_eq!(error.delivery_state(), RequestDeliveryState::PossiblySent);
    assert_eq!(
        error.source().map(ToString::to_string).as_deref(),
        Some("TLS failure"),
        "only the fixed safe TLS category is exposed as a source"
    );
    let diagnostics = format!("{error} {error:?}");
    assert!(!diagnostics.contains("TLS_RESPONSE_SENTINEL_0123456789"));
    assert!(!diagnostics.contains("KMIPKIT_LOW_LEVEL_REQUEST_SENTINEL_73"));
}

fn assert_peer_received_request(observation: &PeerObservation, expected: &[u8]) {
    assert_eq!(
        observation.failure, None,
        "peer fixture reaches request gate"
    );
    assert!(
        observation.handshake_completed,
        "the TLS handshake completed"
    );
    assert_eq!(
        observation.protocol_version,
        Some(rustls::ProtocolVersion::TLSv1_3)
    );
    assert!(
        observation.client_identity_present,
        "mTLS authenticated the client"
    );
    assert_eq!(observation.request_body, expected);
}

fn assert_peer_response_was_written(observation: &PeerObservation) {
    assert!(
        observation.response_written,
        "the peer released a protected response"
    );
    assert_eq!(observation.failure, None);
}

struct PeerObservation {
    handshake_completed: bool,
    protocol_version: Option<rustls::ProtocolVersion>,
    client_identity_present: bool,
    request_body: Vec<u8>,
    response_written: bool,
    failure: Option<String>,
}

enum PeerProtocol {
    RawTtlv,
    Https,
}

fn spawn_gated_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    protocol: PeerProtocol,
) -> (
    JoinHandle<PeerObservation>,
    mpsc::Receiver<PeerObservation>,
    mpsc::SyncSender<()>,
) {
    let (request_seen, request_seen_receiver) = mpsc::sync_channel(1);
    let (release_response, release_receiver) = mpsc::sync_channel(1);
    let peer = thread::spawn(move || {
        let Ok((stream, _)) = accept_before_deadline(&listener) else {
            return report_peer_failure(
                &request_seen,
                failed_peer_observation(),
                "peer accept failed",
            );
        };
        let mut observation = failed_peer_observation();
        if stream.set_nonblocking(false).is_err()
            || stream.set_read_timeout(Some(TEST_TIMEOUT)).is_err()
            || stream.set_write_timeout(Some(TEST_TIMEOUT)).is_err()
        {
            return report_peer_failure(&request_seen, observation, "peer socket setup failed");
        }
        let Ok(connection) = ServerConnection::new(configuration) else {
            return report_peer_failure(&request_seen, observation, "TLS server creation failed");
        };
        let mut tls = StreamOwned::new(connection, stream);
        let handshake_deadline = Instant::now() + TEST_TIMEOUT;
        while tls.conn.is_handshaking() {
            if Instant::now() >= handshake_deadline {
                return report_peer_failure(
                    &request_seen,
                    observation,
                    "TLS handshake deadline expired",
                );
            }
            if let Err(error) = tls.conn.complete_io(&mut tls.sock) {
                observation.failure = Some(format!("TLS handshake failed: {error}"));
                return report_peer_failure(&request_seen, observation, "TLS handshake failed");
            }
        }
        observation.handshake_completed = true;
        observation.protocol_version = tls.conn.protocol_version();
        observation.client_identity_present = tls
            .conn
            .peer_certificates()
            .is_some_and(|certificates| !certificates.is_empty());
        observation.request_body = match match protocol {
            PeerProtocol::RawTtlv => read_exact_request(&mut tls, fixtures::REQUEST_SENTINEL.len()),
            PeerProtocol::Https => read_https_request_body(&mut tls),
        } {
            Ok(body) => body,
            Err(error) => {
                observation.failure = Some(format!("request read failed: {}", error.kind()));
                return report_peer_failure(&request_seen, observation, "request read failed");
            }
        };
        if observation.request_body.is_empty() {
            return report_peer_failure(
                &request_seen,
                observation,
                "peer received an empty request",
            );
        }
        let _ = request_seen.send(PeerObservation {
            handshake_completed: observation.handshake_completed,
            protocol_version: observation.protocol_version,
            client_identity_present: observation.client_identity_present,
            request_body: observation.request_body.clone(),
            response_written: false,
            failure: None,
        });
        if release_receiver.recv_timeout(TEST_TIMEOUT).is_ok() {
            let response = match protocol {
                PeerProtocol::RawTtlv => RAW_RESPONSE.to_vec(),
                PeerProtocol::Https => https_response(),
            };
            observation.response_written = tls.write_all(&response).is_ok() && tls.flush().is_ok();
        }
        observation
    });
    (peer, request_seen_receiver, release_response)
}

fn failed_peer_observation() -> PeerObservation {
    PeerObservation {
        handshake_completed: false,
        protocol_version: None,
        client_identity_present: false,
        request_body: Vec::new(),
        response_written: false,
        failure: None,
    }
}

fn report_peer_failure(
    request_seen: &mpsc::SyncSender<PeerObservation>,
    mut observation: PeerObservation,
    failure: &str,
) -> PeerObservation {
    if observation.failure.is_none() {
        observation.failure = Some(failure.to_owned());
    }
    let _ = request_seen.send(PeerObservation {
        handshake_completed: observation.handshake_completed,
        protocol_version: observation.protocol_version,
        client_identity_present: observation.client_identity_present,
        request_body: observation.request_body.clone(),
        response_written: observation.response_written,
        failure: observation.failure.clone(),
    });
    observation
}

fn read_exact_request(reader: &mut impl Read, length: usize) -> io::Result<Vec<u8>> {
    let mut request = vec![0; length];
    reader.read_exact(&mut request)?;
    Ok(request)
}

fn read_https_request_body(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut request = Vec::new();
    loop {
        if let Some((header_end, body_len)) = parsed_http_request_boundary(&request)
            && request.len() >= header_end + body_len
        {
            return Ok(request[header_end..header_end + body_len].to_vec());
        }
        if request.len() > 16 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "test HTTPS request exceeded its bounded fixture size",
            ));
        }
        let mut buffer = [0; 1024];
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "test HTTPS request ended before its body",
            ));
        }
        request.extend_from_slice(&buffer[..read]);
    }
}

fn parsed_http_request_boundary(request: &[u8]) -> Option<(usize, usize)> {
    let header_end = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")?
        + 4;
    let headers = std::str::from_utf8(&request[..header_end]).ok()?;
    let content_length = headers
        .split("\r\n")
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))?
        .1
        .trim()
        .parse::<usize>()
        .ok()?;
    Some((header_end, content_length))
}

fn https_response() -> Vec<u8> {
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        RESPONSE_SENTINEL.len()
    );
    [headers.as_bytes(), RESPONSE_SENTINEL].concat()
}

fn server_config(pki: &EphemeralPki) -> Arc<ServerConfig> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the local peer supports TLS 1.3 only");
    let mut client_roots = RootCertStore::empty();
    client_roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("the client test CA is valid");
    let verifier = WebPkiClientVerifier::builder(Arc::new(client_roots))
        .build()
        .expect("the peer requires a verified client certificate");
    let mut configuration = builder
        .with_client_cert_verifier(verifier)
        .with_single_cert(
            vec![CertificateDer::from(
                pki.server_identity().certificate_der().to_vec(),
            )],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                pki.server_identity().private_key_der().to_vec(),
            )),
        )
        .expect("the generated server identity is valid");
    configuration.send_tls13_tickets = 0;
    Arc::new(configuration)
}

fn raw_tls_configuration(pki: &EphemeralPki, port: u16) -> config::TransportConfig {
    config::TransportConfig::builder(config::Endpoint::raw_tls("127.0.0.1", port))
        .client_identity(client_identity(pki))
        .trust_source(trust_source(pki))
        .tls_server_name(SERVER_NAME)
        .build()
        .expect("the raw-TLS config uses explicit identity and trust inputs")
}

fn https_configuration(pki: &EphemeralPki, endpoint: String) -> config::TransportConfig {
    config::TransportConfig::builder(config::Endpoint::https(endpoint))
        .client_identity(client_identity(pki))
        .trust_source(trust_source(pki))
        .build()
        .expect("the HTTPS config uses explicit identity and trust inputs")
}

fn client_identity(pki: &EphemeralPki) -> config::ClientIdentity {
    let chain = pki
        .client_identity()
        .certificate_chain_der()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect();
    config::ClientIdentity::new(
        config::CertificateInput::from_der(chain),
        config::PrivateKeyInput::from_der(pki.client_identity().private_key_der().to_vec()),
    )
}

fn trust_source(pki: &EphemeralPki) -> config::TrustSource {
    config::TrustSource::certificate_authorities(vec![config::CertificateInput::from_der(vec![
        pki.authority_certificate_der().to_vec(),
    ])])
}

fn fixed_resolver(address: SocketAddr) -> resolver::Resolver {
    resolver::Resolver::with_lookup_and_governor(
        move |_host, _port| Ok(vec![address]),
        Arc::new(tokio::sync::Semaphore::new(1)),
    )
}

fn accept_before_deadline(listener: &TcpListener) -> io::Result<(TcpStream, SocketAddr)> {
    listener.set_nonblocking(true)?;
    let deadline = Instant::now() + TEST_TIMEOUT;
    loop {
        match listener.accept() {
            Ok(connection) => return Ok(connection),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "the bounded test peer accept expired",
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error),
        }
    }
}

struct TlsRecordRelay {
    address: SocketAddr,
    armed: Arc<AtomicBool>,
    task: JoinHandle<RelayObservation>,
}

struct RelayObservation {
    accepted: bool,
    client_connection_terminated: bool,
    corrupted_application_data_records: usize,
}

impl TlsRecordRelay {
    fn start(target: SocketAddr) -> io::Result<Self> {
        let listener = LoopbackTcpListener::bind()?.into_inner();
        let address = listener.local_addr()?;
        let armed = Arc::new(AtomicBool::new(false));
        let corruption_count = Arc::new(AtomicUsize::new(0));
        let relay_armed = Arc::clone(&armed);
        let relay_corruption_count = Arc::clone(&corruption_count);
        let task = thread::spawn(move || {
            let Ok((client, _)) = accept_before_deadline(&listener) else {
                return failed_relay_observation(false);
            };
            let Ok(server) = TcpStream::connect(target) else {
                return failed_relay_observation(true);
            };
            for stream in [&client, &server] {
                if stream.set_nonblocking(false).is_err()
                    || stream.set_read_timeout(Some(TEST_TIMEOUT)).is_err()
                    || stream.set_write_timeout(Some(TEST_TIMEOUT)).is_err()
                {
                    return failed_relay_observation(true);
                }
            }
            let Ok(client_reader) = client.try_clone() else {
                return failed_relay_observation(true);
            };
            let Ok(server_writer) = server.try_clone() else {
                return failed_relay_observation(true);
            };
            let client_to_server = thread::spawn(move || {
                let mut reader = client_reader;
                let mut writer = server_writer;
                let result = io::copy(&mut reader, &mut writer);
                let _ = writer.shutdown(Shutdown::Write);
                result
            });
            let Ok(server_reader) = server.try_clone() else {
                let _ = client_to_server.join();
                return failed_relay_observation(true);
            };
            let mut client_writer = client;
            let server_to_client = forward_server_records(
                server_reader,
                &mut client_writer,
                &relay_armed,
                &relay_corruption_count,
            );
            let _ = client_writer.shutdown(Shutdown::Write);
            let client_connection_terminated = client_to_server.join().is_ok_and(|result| {
                result.is_ok() || result.is_err_and(|error| error.kind() != io::ErrorKind::TimedOut)
            });
            let _ = server_to_client;
            RelayObservation {
                accepted: true,
                client_connection_terminated,
                corrupted_application_data_records: relay_corruption_count.load(Ordering::Acquire),
            }
        });
        Ok(Self {
            address,
            armed,
            task,
        })
    }

    fn address(&self) -> SocketAddr {
        self.address
    }

    fn arm_response_record_corruption(&self) {
        self.armed.store(true, Ordering::Release);
    }

    fn join(self) -> RelayObservation {
        self.task
            .join()
            .expect("the TLS record relay thread completes")
    }
}

fn failed_relay_observation(accepted: bool) -> RelayObservation {
    RelayObservation {
        accepted,
        client_connection_terminated: false,
        corrupted_application_data_records: 0,
    }
}

fn forward_server_records(
    mut source: TcpStream,
    destination: &mut TcpStream,
    armed: &AtomicBool,
    corruption_count: &AtomicUsize,
) -> io::Result<()> {
    let mut pending = Vec::new();
    let mut input = [0; 4096];
    loop {
        let read = source.read(&mut input)?;
        if read == 0 {
            if pending.is_empty() {
                return Ok(());
            }
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "the test relay received a partial TLS record",
            ));
        }
        pending.extend_from_slice(&input[..read]);
        let mut consumed = 0;
        while pending.len() - consumed >= 5 {
            let record = &pending[consumed..];
            let fragment_len = usize::from(u16::from_be_bytes([record[3], record[4]]));
            let record_len = 5 + fragment_len;
            if record.len() < record_len {
                break;
            }
            let mut complete_record = record[..record_len].to_vec();
            consumed += record_len;
            let is_tls13_application_record = complete_record[0] == 0x17
                && complete_record[1..3] == [0x03, 0x03]
                && fragment_len >= 17;
            let armed_for_response = armed.load(Ordering::Acquire);
            let is_first_corruption = is_tls13_application_record
                && armed_for_response
                && corruption_count
                    .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok();
            if is_first_corruption {
                let tag_last_byte = complete_record.len() - 1;
                complete_record[tag_last_byte] ^= 1;
            }
            destination.write_all(&complete_record)?;
        }
        if consumed > 0 {
            pending.drain(..consumed);
        }
    }
}
