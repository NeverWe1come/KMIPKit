use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use kmipkit_test_support::{EphemeralPki, LoopbackTcpListener};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::{ClientConnection, RootCertStore, ServerConfig, ServerConnection};

#[path = "../src/tls_policy.rs"]
mod tls_policy;

#[test]
fn runtime_client_configuration_disables_key_logging_and_early_data() {
    let pki = EphemeralPki::generate().expect("ephemeral test certificates are generated");
    let config = verified_client_config(&pki);

    assert!(!config.key_log.will_log("CLIENT_HANDSHAKE_TRAFFIC_SECRET"));
    assert!(!config.key_log.will_log("CLIENT_EARLY_TRAFFIC_SECRET"));
    assert!(!config.enable_early_data);

    let mut connection = ClientConnection::new(
        Arc::new(config),
        ServerName::try_from("server.kmipkit.test")
            .expect("fixture hostname is a valid server name"),
    )
    .expect("the policy config creates a client connection");
    assert!(connection.early_data().is_none());
}

#[test]
fn runtime_client_configuration_negotiates_tls13_with_a_local_peer() {
    let pki = EphemeralPki::generate().expect("ephemeral test certificates are generated");
    let listener = LoopbackTcpListener::bind()
        .expect("the TLS peer binds only to an ephemeral loopback address");
    let address = listener.local_addr();
    let server_config = tls13_server_config(&pki);
    let local_listener = listener.into_inner();
    let server = thread::spawn(move || run_tls_server(&local_listener, server_config));

    let mut client = ClientConnection::new(
        Arc::new(verified_client_config(&pki)),
        ServerName::try_from("server.kmipkit.test")
            .expect("fixture hostname is a valid server name"),
    )
    .expect("the policy config creates a client connection");
    let mut stream = TcpStream::connect(address).expect("the local TLS peer accepts connections");
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .expect("the local handshake has a bounded read");
    let client_result = drive_client_handshake(&mut client, &mut stream);
    drop(stream);

    assert!(client_result.is_ok(), "the local TLS 1.3 peer is accepted");
    assert_eq!(
        client.protocol_version(),
        Some(rustls::ProtocolVersion::TLSv1_3)
    );
    assert!(server.join().expect("local TLS server thread completes"));
}

fn tls13_server_config(pki: &EphemeralPki) -> Arc<ServerConfig> {
    let identity = pki.server_identity();
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("TLS 1.3 is available in the test-only server policy");
    let config = builder
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(identity.certificate_der().to_vec())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                identity.private_key_der().to_vec(),
            )),
        )
        .expect("the test server identity is valid");
    Arc::new(config)
}

fn verified_client_config(pki: &EphemeralPki) -> rustls::ClientConfig {
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("the ephemeral test root is valid");
    let mut config = tls_policy::client_config_builder()
        .expect("the TLS 1.3 AWS-LC builder is valid")
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls_policy::apply_client_safety_policy(&mut config);
    config
}

fn run_tls_server(listener: &TcpListener, config: Arc<ServerConfig>) -> bool {
    let Ok((mut stream, _)) = listener.accept() else {
        return false;
    };
    let Ok(mut connection) = ServerConnection::new(config) else {
        return false;
    };
    while connection.is_handshaking() {
        if connection.complete_io(&mut stream).is_err() {
            return false;
        }
    }
    true
}

fn drive_client_handshake(
    connection: &mut ClientConnection,
    stream: &mut TcpStream,
) -> Result<(), std::io::Error> {
    while connection.is_handshaking() {
        connection.complete_io(stream)?;
    }
    Ok(())
}
