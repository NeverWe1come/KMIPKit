use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use kmipkit_test_support::EphemeralPki;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

use crate::config::{
    CertificateInput, ClientIdentity, PrivateKeyInput, TransportConfigError, TrustSource,
    WireEncoding,
};
use crate::listener::{
    HttpsListener, ListenerTransportConfig, ListenerTransportConfigBuilder, RawTlsListener,
};

fn test_listener_config(pki: &EphemeralPki) -> ListenerTransportConfigBuilder {
    let server = pki.server_identity();
    let chain = vec![server.certificate_der().to_vec()];
    let key = server.private_key_der().to_vec();

    let identity = ClientIdentity::new(
        CertificateInput::from_der(chain),
        PrivateKeyInput::from_der(key),
    );
    let trust = TrustSource::certificate_authorities(vec![CertificateInput::from_der(vec![
        pki.authority_certificate_der().to_vec(),
    ])]);

    ListenerTransportConfig::builder("127.0.0.1", 0)
        .listener_identity(identity)
        .trust_source(trust)
}

fn client_tls_config(pki: &EphemeralPki) -> Arc<ClientConfig> {
    let client = pki.client_identity();
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("authority cert is valid");

    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
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
fn listener_config_builder_validation() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");

    // Empty host rejected
    let res = ListenerTransportConfig::builder("", 5696).build();
    assert_eq!(res.unwrap_err(), TransportConfigError::InvalidEndpoint);

    // Missing identity rejected
    let res = ListenerTransportConfig::builder("127.0.0.1", 5696).build();
    assert_eq!(res.unwrap_err(), TransportConfigError::InvalidCredential);

    // Missing trust rejected
    let server = pki.server_identity();
    let identity = ClientIdentity::new(
        CertificateInput::from_der(vec![server.certificate_der().to_vec()]),
        PrivateKeyInput::from_der(server.private_key_der().to_vec()),
    );
    let res = ListenerTransportConfig::builder("127.0.0.1", 5696)
        .listener_identity(identity)
        .build();
    assert_eq!(res.unwrap_err(), TransportConfigError::InvalidTrust);

    // Zero max request bytes rejected
    let valid = test_listener_config(&pki).max_request_bytes(0).build();
    assert_eq!(valid.unwrap_err(), TransportConfigError::InvalidEndpoint);

    // Invalid target path rejected
    let valid = test_listener_config(&pki)
        .target_path("invalid_no_slash")
        .build();
    assert_eq!(valid.unwrap_err(), TransportConfigError::InvalidEndpoint);

    // Valid configuration
    let valid = test_listener_config(&pki)
        .target_path("/kmip")
        .wire_encoding(WireEncoding::Ttlv)
        .build()
        .expect("valid config builds successfully");
    assert_eq!(valid.bind_host(), "127.0.0.1");
    assert_eq!(valid.target_path(), "/kmip");
    assert_eq!(valid.wire_encoding(), Some(WireEncoding::Ttlv));
}

#[test]
fn raw_tls_listener_exchange_and_role_reversal() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config = test_listener_config(&pki)
        .build()
        .expect("listener config is valid");

    let mut listener = RawTlsListener::bind(config).expect("bind succeeds");
    let port = listener.port().expect("port is valid");
    let client_config = client_tls_config(&pki);

    // Client thread
    let client_handle = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect succeeds");
        let server_name =
            ServerName::try_from("server.kmipkit.test").expect("valid dns server name");
        let client_conn =
            ClientConnection::new(client_config, server_name).expect("client connection succeeds");
        let mut tls = StreamOwned::new(client_conn, sock);

        // Send a TTLV request frame: 8-byte header + 8-byte payload
        let req_payload = [0x42, 0x00, 0x77, 0x01, 0, 0, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8];
        tls.write_all(&req_payload).expect("write succeeds");
        tls.flush().expect("flush succeeds");

        // Read response frame
        let mut resp = [0_u8; 16];
        tls.read_exact(&mut resp).expect("read response succeeds");
        assert_eq!(
            resp,
            [0x42, 0x00, 0x7A, 0x01, 0, 0, 0, 8, 9, 8, 7, 6, 5, 4, 3, 2]
        );

        // Now in role-reversed mode: client receives request from server
        let mut s2c_req = [0_u8; 16];
        tls.read_exact(&mut s2c_req).expect("read s2c req succeeds");
        assert_eq!(
            s2c_req,
            [
                0x42, 0x00, 0x77, 0x01, 0, 0, 0, 8, 10, 20, 30, 40, 50, 60, 70, 80
            ]
        );

        // And client sends response back
        let s2c_resp = [
            0x42, 0x00, 0x7A, 0x01, 0, 0, 0, 8, 80, 70, 60, 50, 40, 30, 20, 10,
        ];
        tls.write_all(&s2c_resp).expect("write s2c resp succeeds");
        tls.flush().expect("flush succeeds");
    });

    let mut conn = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");
    let req = conn
        .recv_request(1024, Some(Duration::from_secs(5)))
        .expect("recv_request succeeds");
    assert_eq!(
        req.as_bytes(),
        &[0x42, 0x00, 0x77, 0x01, 0, 0, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8]
    );

    let resp_payload = [0x42, 0x00, 0x7A, 0x01, 0, 0, 0, 8, 9, 8, 7, 6, 5, 4, 3, 2];
    conn.send_response(&resp_payload, Some(Duration::from_secs(5)))
        .expect("send_response succeeds");

    // Convert to role-reversed stream
    let mut rr_stream = conn.into_role_reversed();

    // Server sends request to client
    let s2c_req = [
        0x42, 0x00, 0x77, 0x01, 0, 0, 0, 8, 10, 20, 30, 40, 50, 60, 70, 80,
    ];
    rr_stream
        .send_client_response(&s2c_req, Some(Duration::from_secs(5)))
        .expect("send s2c req succeeds");

    // Server reads response from client
    let s2c_resp = rr_stream
        .recv_server_request(1024, Some(Duration::from_secs(5)))
        .expect("recv s2c resp succeeds");
    assert_eq!(
        s2c_resp.as_bytes(),
        &[
            0x42, 0x00, 0x7A, 0x01, 0, 0, 0, 8, 80, 70, 60, 50, 40, 30, 20, 10
        ]
    );

    rr_stream.close().expect("close succeeds");
    client_handle
        .join()
        .expect("client thread finishes cleanly");
}

#[test]
fn https_listener_receives_and_responds() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config = test_listener_config(&pki)
        .target_path("/kmip")
        .build()
        .expect("listener config is valid");

    let mut listener = HttpsListener::bind(config).expect("bind succeeds");
    let port = listener.port().expect("port is valid");
    let client_config = client_tls_config(&pki);

    let client_handle = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect succeeds");
        let server_name =
            ServerName::try_from("server.kmipkit.test").expect("valid dns server name");
        let client_conn =
            ClientConnection::new(client_config, server_name).expect("client connection succeeds");
        let mut tls = StreamOwned::new(client_conn, sock);

        let body = b"{\"RequestHeader\":{}}";
        let http_req = format!(
            "POST /kmip HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            std::str::from_utf8(body).unwrap()
        );
        tls.write_all(http_req.as_bytes()).expect("write succeeds");
        tls.flush().expect("flush succeeds");

        let mut resp_buf = Vec::new();
        let mut chunk = [0_u8; 512];
        while !resp_buf.windows(4).any(|w| w == b"\r\n\r\n")
            || !resp_buf.ends_with(b"{\"ResponseHeader\":{}}")
        {
            let n = tls.read(&mut chunk).expect("read response succeeds");
            if n == 0 {
                break;
            }
            resp_buf.extend_from_slice(&chunk[..n]);
        }
        let resp_str = String::from_utf8_lossy(&resp_buf);
        assert!(resp_str.contains("HTTP/1.1 200 OK"));
        assert!(resp_str.contains("Content-Type: application/json"));
        assert!(resp_str.contains("{\"ResponseHeader\":{}}"));
    });

    let mut conn = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");
    let (req, encoding) = conn
        .recv_request(1024, Some(Duration::from_secs(5)))
        .expect("recv_request succeeds");
    assert_eq!(encoding, WireEncoding::Json);
    assert_eq!(req.as_bytes(), b"{\"RequestHeader\":{}}");

    let resp_body = b"{\"ResponseHeader\":{}}";
    conn.send_response(resp_body, WireEncoding::Json, Some(Duration::from_secs(5)))
        .expect("send_response succeeds");

    conn.close().expect("close succeeds");
    client_handle
        .join()
        .expect("client thread finishes cleanly");
}
