//! TLS policy tests derived from KMIPKIT-0013 FR-002, FR-003, and FR-006.
//!
//! KMIP Specification v2.1 §10.4 requires channel confidentiality, integrity,
//! and authenticity. The product-specific TLS 1.3, mutual authentication,
//! trust, revocation, and resumption requirements are defined in KMIPKIT-0013.

use std::env;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use config::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RevocationListInput,
    TransportConfig, TransportConfigError, TrustSource,
};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, CertificateRevocationListParams, DnType,
    ExtendedKeyUsagePurpose, IsCa, Issuer, KeyIdMethod, KeyPair, KeyUsagePurpose,
    RevokedCertParams, SerialNumber, date_time_ymd,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::server::WebPkiClientVerifier;
use rustls::{ClientConnection, HandshakeKind, RootCertStore, ServerConfig, ServerConnection};
use tls::{MonotonicClock, TlsClientConfig};

#[path = "../src/config.rs"]
mod config;
#[path = "../src/tls.rs"]
mod tls;

const SERVER_NAME: &str = "server.kmipkit.test";
const SERVER_SERIAL: u64 = 41;
const CLIENT_SERIAL: u64 = 42;

#[test]
fn caller_ca_and_matching_mtls_identity_build_a_tls13_only_config() {
    let pki = TestPki::generate(SERVER_NAME, false);
    let configuration = transport_config(&pki, &pki, None, Vec::new());
    let client = tls::build_client_config(configuration)
        .expect("valid caller roots and a matching client identity build");

    assert!(!client.rustls_config().enable_early_data);
    assert!(
        !client
            .rustls_config()
            .key_log
            .will_log("CLIENT_HANDSHAKE_TRAFFIC_SECRET")
    );
    assert_eq!(client.server_name(), &dns_name(SERVER_NAME));
    let connection = ClientConnection::new(
        Arc::new(client.rustls_config().clone()),
        client.server_name().clone(),
    )
    .expect("the TLS 1.3 client connection is constructible");
    assert!(connection.early_data().is_none());

    let peer = server_config(&pki, true, 2);
    assert_eq!(
        handshake(&client, peer).expect("the trusted mutual-TLS peer succeeds"),
        HandshakeKind::Full
    );

    let other_identity = TestPki::generate("other-client.kmipkit.test", false);
    let mismatched_client =
        tls::build_client_config(transport_config(&pki, &other_identity, None, Vec::new()))
            .expect("a distinct client identity is valid on its own");
    expect_tls_rejection(handshake(&mismatched_client, server_config(&pki, true, 0)));
}

#[test]
fn mismatched_client_certificate_and_private_key_are_rejected_before_tls() {
    let client_pki = TestPki::generate(SERVER_NAME, false);
    let other_pki = TestPki::generate("other.kmipkit.test", false);
    let identity = identity_input(&client_pki, &other_pki);
    let builder = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", 5696))
        .client_identity(identity)
        .trust_source(trust_input(&client_pki));

    assert!(matches!(
        builder.build(),
        Err(TransportConfigError::InvalidCredential)
    ));
}

#[test]
fn untrusted_chain_expired_leaf_and_hostname_mismatch_fail_closed() {
    let pki = TestPki::generate(SERVER_NAME, false);
    let unrelated = TestPki::generate(SERVER_NAME, false);
    let trusted_client = tls::build_client_config(transport_config(&pki, &pki, None, Vec::new()))
        .expect("the caller trust configuration is valid");
    let untrusted_peer = server_config(&unrelated, false, 0);
    expect_tls_rejection(handshake(&trusted_client, untrusted_peer));

    let expired = TestPki::generate_expired(SERVER_NAME);
    let expired_client =
        tls::build_client_config(transport_config(&expired, &expired, None, Vec::new()))
            .expect("the expired test chain can still be configured as a trust input");
    expect_tls_rejection(handshake(
        &expired_client,
        server_config(&expired, false, 0),
    ));

    let wrong_name = tls::build_client_config(transport_config(
        &pki,
        &pki,
        Some("different.kmipkit.test"),
        Vec::new(),
    ))
    .expect("a valid explicit TLS name is accepted for an IP endpoint");
    expect_tls_rejection(handshake(&wrong_name, server_config(&pki, false, 0)));
}

#[test]
fn explicit_server_name_override_applies_only_to_ip_endpoints() {
    let pki = TestPki::generate(SERVER_NAME, false);
    let overridden =
        tls::build_client_config(transport_config(&pki, &pki, Some(SERVER_NAME), Vec::new()))
            .expect("an IP endpoint may use a separate certificate-verification name");
    assert_eq!(overridden.server_name(), &dns_name(SERVER_NAME));
    assert!(!overridden.rustls_config().enable_sni);
    handshake(&overridden, server_config(&pki, false, 0))
        .expect("the explicit verification name matches the server certificate");

    let no_override = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", 5696))
        .client_identity(identity_input(&pki, &pki))
        .trust_source(trust_input(&pki))
        .build()
        .expect("an IP endpoint may rely on its IP subject alternative name");
    let no_override = tls::build_client_config(no_override)
        .expect("the caller omitted the optional TLS-name override");
    expect_tls_rejection(handshake(&no_override, server_config(&pki, false, 0)));

    let dns_endpoint = TransportConfig::builder(Endpoint::raw_tls(SERVER_NAME, 5696))
        .client_identity(identity_input(&pki, &pki))
        .trust_source(trust_input(&pki))
        .tls_server_name("other.kmipkit.test");
    assert!(matches!(
        dns_endpoint.build(),
        Err(TransportConfigError::InvalidServerName)
    ));
}

#[test]
fn caller_crls_accept_current_evidence_and_reject_expired_unmatched_or_revoking_lists() {
    let pki = TestPki::generate(SERVER_NAME, false);
    let valid = tls::build_client_config(transport_config(
        &pki,
        &pki,
        None,
        vec![pki.crl(false, false)],
    ))
    .expect("a valid applicable caller CRL configures");
    handshake(&valid, server_config(&pki, false, 0))
        .expect("a current applicable non-revoking CRL accepts the server");

    let expired_crl = tls::build_client_config(transport_config(
        &pki,
        &pki,
        None,
        vec![pki.crl(false, true)],
    ))
    .expect("a syntactically valid expired CRL is supplied");
    expect_tls_rejection(handshake(&expired_crl, server_config(&pki, false, 0)));

    let unrelated = TestPki::generate(SERVER_NAME, false);
    let non_applicable = tls::build_client_config(transport_config(
        &pki,
        &pki,
        None,
        vec![unrelated.crl(false, false)],
    ))
    .expect("a valid CRL with a different issuer is supplied");
    expect_tls_rejection(handshake(&non_applicable, server_config(&pki, false, 0)));

    let revoked = tls::build_client_config(transport_config(
        &pki,
        &pki,
        None,
        vec![pki.crl(true, false)],
    ))
    .expect("a valid CRL revoking this leaf is supplied");
    expect_tls_rejection(handshake(&revoked, server_config(&pki, false, 0)));
}

#[test]
fn tls13_resumption_uses_new_connections_and_is_bounded_to_sixteen_tickets() {
    let pki = TestPki::generate(SERVER_NAME, false);
    let client = tls::build_client_config(transport_config(&pki, &pki, None, Vec::new()))
        .expect("the client configuration is valid");
    let peer = server_config(&pki, false, 32);

    assert_eq!(
        handshake(&client, Arc::clone(&peer)).expect("the first TLS connection succeeds"),
        HandshakeKind::Full
    );
    assert_eq!(client.cached_ticket_count(), 16);
    assert_eq!(
        handshake(&client, peer).expect("a second TCP connection resumes"),
        HandshakeKind::Resumed
    );
}

#[test]
fn each_identity_gets_an_isolated_session_cache() {
    let pki = TestPki::generate(SERVER_NAME, false);
    let other_identity = TestPki::generate("other-client.kmipkit.test", false);
    let first = tls::build_client_config(transport_config(&pki, &pki, None, Vec::new()))
        .expect("the first identity configures");
    let second =
        tls::build_client_config(transport_config(&pki, &other_identity, None, Vec::new()))
            .expect("the second identity configures with the same server trust");
    let peer = server_config(&pki, false, 2);

    assert_eq!(
        handshake(&first, Arc::clone(&peer)).unwrap(),
        HandshakeKind::Full
    );
    assert!(first.cached_ticket_count() > 0);
    assert_eq!(second.cached_ticket_count(), 0);
    assert_eq!(handshake(&second, peer).unwrap(), HandshakeKind::Full);
}

#[test]
fn local_ticket_age_expires_at_one_hour_and_rebuilt_trust_starts_empty() {
    let pki = TestPki::generate(SERVER_NAME, false);
    let peer = server_config(&pki, false, 2);

    let before_expiry_clock = Arc::new(TestClock::default());
    let before_expiry = tls::build_client_config_with_clock(
        transport_config(&pki, &pki, None, Vec::new()),
        Arc::clone(&before_expiry_clock) as Arc<dyn MonotonicClock>,
    )
    .expect("the client accepts an injectable monotonic clock");
    assert_eq!(
        handshake(&before_expiry, Arc::clone(&peer)).unwrap(),
        HandshakeKind::Full
    );
    assert!(before_expiry.cached_ticket_count() > 0);
    before_expiry_clock.advance(Duration::from_secs(60 * 60 - 1));
    assert_eq!(
        handshake(&before_expiry, Arc::clone(&peer)).unwrap(),
        HandshakeKind::Resumed
    );

    let expiry_clock = Arc::new(TestClock::default());
    let expiring = tls::build_client_config_with_clock(
        transport_config(&pki, &pki, None, Vec::new()),
        Arc::clone(&expiry_clock) as Arc<dyn MonotonicClock>,
    )
    .expect("a separate client starts an independent session cache");
    assert_eq!(
        handshake(&expiring, Arc::clone(&peer)).unwrap(),
        HandshakeKind::Full
    );
    assert!(expiring.cached_ticket_count() > 0);
    expiry_clock.advance(Duration::from_secs(60 * 60));
    assert_eq!(
        handshake(&expiring, Arc::clone(&peer)).unwrap(),
        HandshakeKind::Full
    );

    let changed_trust = TestPki::generate(SERVER_NAME, false);
    let (rebuilt, verifier) = tls::build_client_config_with_test_verifier(
        transport_config(&changed_trust, &pki, None, Vec::new()),
        expiry_clock,
    )
    .expect("a rebuilt client applies the new trust set");
    assert_eq!(rebuilt.cached_ticket_count(), 0);
    expect_tls_rejection(handshake(&rebuilt, peer));
    assert_eq!(
        verifier.calls(),
        1,
        "the rebuilt client attempts a full verification"
    );
}

#[test]
fn resumed_handshake_does_not_reinvoke_the_full_handshake_trust_verifier() {
    let pki = TestPki::generate(SERVER_NAME, false);
    let clock = Arc::new(TestClock::default());
    let (client, verifier) = tls::build_client_config_with_test_verifier(
        transport_config(&pki, &pki, None, vec![pki.crl(false, false)]),
        clock,
    )
    .expect("the test verifier wraps the production WebPKI verifier");
    let peer = server_config(&pki, false, 2);

    assert_eq!(
        handshake(&client, Arc::clone(&peer)).unwrap(),
        HandshakeKind::Full
    );
    assert_eq!(verifier.calls(), 1);
    verifier.reject_new_full_handshakes();
    assert_eq!(handshake(&client, peer).unwrap(), HandshakeKind::Resumed);
    assert_eq!(verifier.calls(), 1);
}

#[test]
fn ssl_cert_file_override_and_loader_errors_are_isolated_in_child_processes() {
    let pki = TestPki::generate(SERVER_NAME, false);
    let bundle = temporary_path(".pem");
    fs::write(&bundle, pem("CERTIFICATE", pki.ca.der().as_ref()))
        .expect("the isolated root bundle is written");
    let valid = run_ssl_cert_file_child(&bundle, "valid");
    let _ = fs::remove_file(&bundle);
    assert!(
        valid.status.success(),
        "valid SSL_CERT_FILE child failed: {}",
        String::from_utf8_lossy(&valid.stderr)
    );

    let missing = temporary_path("-missing.pem");
    let invalid = run_ssl_cert_file_child(&missing, "invalid");
    assert!(
        invalid.status.success(),
        "invalid SSL_CERT_FILE child failed: {}",
        String::from_utf8_lossy(&invalid.stderr)
    );

    let native = run_native_roots_child();
    assert!(
        native.status.success(),
        "native roots child failed: {}",
        String::from_utf8_lossy(&native.stderr)
    );
}

#[test]
fn ssl_cert_file_child() {
    let Ok(mode) = env::var("KMIPKIT_SSL_CERT_FILE_TEST") else {
        return;
    };
    let loaded = rustls_native_certs::load_native_certs();
    let identity_pki = TestPki::generate(SERVER_NAME, false);
    let builder = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", 5696))
        .client_identity(identity_input(&identity_pki, &identity_pki))
        .trust_source(TrustSource::platform());

    match mode.as_str() {
        "valid" => {
            let expected =
                fs::read(env::var_os("SSL_CERT_FILE").expect("parent sets SSL_CERT_FILE"))
                    .expect("the override bundle exists");
            let expected_certificates = CertificateDer::pem_slice_iter(&expected)
                .collect::<Result<Vec<_>, _>>()
                .expect("the override bundle is valid PEM");
            assert!(loaded.errors.is_empty());
            assert_eq!(loaded.certs, expected_certificates);
            assert!(builder.build().is_ok());
        }
        "invalid" => {
            assert!(!loaded.errors.is_empty());
            assert!(matches!(
                builder.build(),
                Err(TransportConfigError::PlatformTrustUnavailable)
            ));
        }
        "native" => {
            assert!(loaded.errors.is_empty());
            if loaded.certs.is_empty() {
                assert!(matches!(
                    builder.build(),
                    Err(TransportConfigError::InvalidTrust)
                ));
            } else {
                assert!(builder.build().is_ok());
            }
        }
        _ => panic!("unknown child mode"),
    }
}

struct TestPki {
    ca_params: CertificateParams,
    ca_key: KeyPair,
    ca: Certificate,
    server: Certificate,
    server_key: KeyPair,
    client: Certificate,
    client_key: KeyPair,
    server_serial: SerialNumber,
}

impl TestPki {
    fn generate(server_name: &str, expired: bool) -> Self {
        let ca_key = KeyPair::generate().expect("test CA key generation succeeds");
        let mut ca_params = CertificateParams::default();
        ca_params
            .distinguished_name
            .push(DnType::CommonName, "KMIPKit TLS test CA");
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        ca_params.not_before = date_time_ymd(2020, 1, 1);
        ca_params.not_after = date_time_ymd(2099, 1, 1);
        let ca = ca_params
            .self_signed(&ca_key)
            .expect("test CA certificate generation succeeds");
        let issuer = Issuer::from_params(&ca_params, &ca_key);
        let server_serial = SerialNumber::from(SERVER_SERIAL);
        let (server, server_key) = leaf_certificate(
            server_name,
            ExtendedKeyUsagePurpose::ServerAuth,
            server_serial.clone(),
            expired,
            &issuer,
        );
        let (client, client_key) = leaf_certificate(
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

    fn generate_expired(server_name: &str) -> Self {
        Self::generate(server_name, true)
    }

    fn crl(&self, revoke_server: bool, expired: bool) -> RevocationListInput {
        let issuer = Issuer::from_params(&self.ca_params, &self.ca_key);
        let (this_update, next_update) = if expired {
            (date_time_ymd(1998, 1, 1), date_time_ymd(1999, 1, 1))
        } else {
            (date_time_ymd(2020, 1, 1), date_time_ymd(2099, 1, 1))
        };
        let revoked_certs = if revoke_server {
            vec![RevokedCertParams {
                serial_number: self.server_serial.clone(),
                revocation_time: date_time_ymd(2025, 1, 1),
                reason_code: None,
                invalidity_date: None,
            }]
        } else {
            Vec::new()
        };
        let crl = CertificateRevocationListParams {
            this_update,
            next_update,
            crl_number: SerialNumber::from(1_u64),
            issuing_distribution_point: None,
            revoked_certs,
            key_identifier_method: KeyIdMethod::Sha256,
        }
        .signed_by(&issuer)
        .expect("test CRL signing succeeds");
        RevocationListInput::from_der(crl.der().as_ref().to_vec())
    }
}

fn leaf_certificate(
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

fn transport_config(
    endpoint_pki: &TestPki,
    identity_pki: &TestPki,
    tls_server_name: Option<&str>,
    revocation_lists: Vec<RevocationListInput>,
) -> TransportConfig {
    let endpoint_host = if tls_server_name.is_some() {
        "127.0.0.1"
    } else {
        SERVER_NAME
    };
    let mut builder = TransportConfig::builder(Endpoint::raw_tls(endpoint_host, 5696))
        .client_identity(identity_input(identity_pki, identity_pki))
        .trust_source(trust_input(endpoint_pki))
        .revocation_lists(revocation_lists);
    if let Some(server_name) = tls_server_name {
        builder = builder.tls_server_name(server_name);
    }
    builder
        .build()
        .expect("valid TLS test configuration builds")
}

fn identity_input(chain_pki: &TestPki, key_pki: &TestPki) -> ClientIdentity {
    ClientIdentity::new(
        CertificateInput::from_der(vec![
            chain_pki.client.der().as_ref().to_vec(),
            chain_pki.ca.der().as_ref().to_vec(),
        ]),
        PrivateKeyInput::from_der(key_pki.client_key.serialize_der()),
    )
}

fn trust_input(pki: &TestPki) -> TrustSource {
    TrustSource::certificate_authorities(vec![CertificateInput::from_der(vec![
        pki.ca.der().as_ref().to_vec(),
    ])])
}

fn server_config(pki: &TestPki, require_client_auth: bool, tickets: usize) -> Arc<ServerConfig> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the test server uses TLS 1.3 only");
    let builder = if require_client_auth {
        let mut roots = RootCertStore::empty();
        roots
            .add(CertificateDer::from(pki.ca.der().as_ref().to_vec()))
            .expect("the client CA fixture is valid");
        let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
            .build()
            .expect("the server verifies required mTLS clients");
        builder.with_client_cert_verifier(verifier)
    } else {
        builder.with_no_client_auth()
    };
    let mut config = builder
        .with_single_cert(
            vec![
                CertificateDer::from(pki.server.der().as_ref().to_vec()),
                CertificateDer::from(pki.ca.der().as_ref().to_vec()),
            ],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(pki.server_key.serialize_der())),
        )
        .expect("the test server identity is valid");
    config.send_tls13_tickets = tickets;
    Arc::new(config)
}

#[derive(Debug, Eq, PartialEq)]
enum HandshakeFailure {
    ClientRejected,
    ClientIo,
    Peer,
    Thread,
}

fn handshake(
    client: &TlsClientConfig,
    peer_config: Arc<ServerConfig>,
) -> Result<HandshakeKind, HandshakeFailure> {
    let mut connection = ClientConnection::new(
        Arc::new(client.rustls_config().clone()),
        client.server_name().clone(),
    )
    .map_err(|_| HandshakeFailure::ClientIo)?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|_| HandshakeFailure::Peer)?;
    let address = listener.local_addr().map_err(|_| HandshakeFailure::Peer)?;
    let mut stream = TcpStream::connect(address).map_err(|_| HandshakeFailure::ClientIo)?;
    let (peer_stream, _) = listener.accept().map_err(|_| HandshakeFailure::Peer)?;
    drop(listener);
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|_| HandshakeFailure::ClientIo)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|_| HandshakeFailure::ClientIo)?;
    let peer = thread::spawn(move || run_server_handshake(peer_stream, peer_config));
    let client_result = run_client_handshake(&mut connection, &mut stream);
    drop(stream);
    let peer_result = peer.join().map_err(|_| HandshakeFailure::Thread)?;
    match client_result {
        Err(HandshakeFailure::ClientRejected) => Err(HandshakeFailure::ClientRejected),
        Err(HandshakeFailure::ClientIo) if peer_result.is_err() => Err(HandshakeFailure::Peer),
        Err(error) => Err(error),
        Ok(kind) => {
            peer_result.map_err(|()| HandshakeFailure::Peer)?;
            Ok(kind)
        }
    }
}

fn run_client_handshake(
    connection: &mut ClientConnection,
    stream: &mut TcpStream,
) -> Result<HandshakeKind, HandshakeFailure> {
    while connection.is_handshaking() {
        if let Err(error) = connection.complete_io(stream) {
            return if error.kind() == std::io::ErrorKind::InvalidData {
                Err(HandshakeFailure::ClientRejected)
            } else {
                Err(HandshakeFailure::ClientIo)
            };
        }
    }
    let kind = connection
        .handshake_kind()
        .ok_or(HandshakeFailure::ClientIo)?;
    if connection.protocol_version() != Some(rustls::ProtocolVersion::TLSv1_3) {
        return Err(HandshakeFailure::ClientIo);
    }
    while connection.wants_write() {
        connection
            .write_tls(stream)
            .map_err(|_| HandshakeFailure::ClientIo)?;
    }
    loop {
        let count = connection
            .read_tls(stream)
            .map_err(|_| HandshakeFailure::ClientIo)?;
        if count == 0 {
            break;
        }
        connection
            .process_new_packets()
            .map_err(|_| HandshakeFailure::ClientRejected)?;
    }
    Ok(kind)
}

fn expect_tls_rejection(result: Result<HandshakeKind, HandshakeFailure>) {
    assert_eq!(result, Err(HandshakeFailure::ClientRejected));
}

fn run_server_handshake(mut stream: TcpStream, config: Arc<ServerConfig>) -> Result<(), ()> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|_| ())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|_| ())?;
    let mut connection = ServerConnection::new(config).map_err(|_| ())?;
    while connection.is_handshaking() {
        connection.complete_io(&mut stream).map_err(|_| ())?;
    }
    while connection.wants_write() {
        connection.write_tls(&mut stream).map_err(|_| ())?;
    }
    connection.send_close_notify();
    while connection.wants_write() {
        connection.write_tls(&mut stream).map_err(|_| ())?;
    }
    Ok(())
}

fn dns_name(value: &str) -> ServerName<'static> {
    ServerName::try_from(value.to_owned()).expect("the test server name is valid")
}

fn temporary_path(suffix: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "kmipkit-tls-policy-{}-{}{}",
        std::process::id(),
        TEMPORARY_FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        suffix
    ))
}

fn run_ssl_cert_file_child(path: &std::path::Path, mode: &str) -> std::process::Output {
    Command::new(env::current_exe().expect("current test executable exists"))
        .arg("--exact")
        .arg("ssl_cert_file_child")
        .arg("--nocapture")
        .env("SSL_CERT_FILE", path)
        .env_remove("SSL_CERT_DIR")
        .env("KMIPKIT_SSL_CERT_FILE_TEST", mode)
        .output()
        .expect("isolated SSL_CERT_FILE child starts")
}

fn run_native_roots_child() -> std::process::Output {
    Command::new(env::current_exe().expect("current test executable exists"))
        .arg("--exact")
        .arg("ssl_cert_file_child")
        .arg("--nocapture")
        .env_remove("SSL_CERT_FILE")
        .env_remove("SSL_CERT_DIR")
        .env("KMIPKIT_SSL_CERT_FILE_TEST", "native")
        .output()
        .expect("isolated native-roots child starts")
}

fn pem(label: &str, bytes: &[u8]) -> Vec<u8> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::with_capacity(bytes.len().div_ceil(3) * 4 + label.len() * 2 + 64);
    output.extend_from_slice(format!("-----BEGIN {label}-----\n").as_bytes());
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or_default();
        let third = chunk.get(2).copied().unwrap_or_default();
        let indexes = [
            usize::from(first >> 2),
            usize::from(((first & 0x03) << 4) | (second >> 4)),
            usize::from(((second & 0x0f) << 2) | (third >> 6)),
            usize::from(third & 0x3f),
        ];
        for (index, value) in indexes.into_iter().enumerate() {
            let padded = (index == 2 && chunk.len() < 2) || (index == 3 && chunk.len() < 3);
            output.push(if padded { b'=' } else { ALPHABET[value] });
        }
        output.push(b'\n');
    }
    output.extend_from_slice(format!("-----END {label}-----\n").as_bytes());
    output
}

#[derive(Default)]
struct TestClock(std::sync::atomic::AtomicU64);

impl TestClock {
    fn advance(&self, duration: Duration) {
        self.0.fetch_add(
            duration
                .as_nanos()
                .try_into()
                .expect("test duration fits u64"),
            std::sync::atomic::Ordering::SeqCst,
        );
    }
}

impl MonotonicClock for TestClock {
    fn now(&self) -> Duration {
        Duration::from_nanos(self.0.load(std::sync::atomic::Ordering::SeqCst))
    }
}

static TEMPORARY_FILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
