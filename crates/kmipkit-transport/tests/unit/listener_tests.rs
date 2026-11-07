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

#[test]
fn listener_config_debug_and_getters() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let valid = test_listener_config(&pki)
        .target_path("/kmip")
        .wire_encoding(WireEncoding::Ttlv)
        .max_request_bytes(1024 * 1024)
        .build()
        .expect("valid config builds successfully");

    assert!(format!("{valid:?}").contains("ListenerTransportConfig"));
    assert_eq!(valid.bind_host(), "127.0.0.1");
    assert_eq!(valid.bind_port(), 0);
    assert_eq!(valid.max_request_bytes(), 1024 * 1024);
    assert_eq!(valid.target_path(), "/kmip");
    assert_eq!(valid.wire_encoding(), Some(WireEncoding::Ttlv));
    assert!(!valid.roots().is_empty());
    assert_eq!(valid.revocation_lists(), []);
    let _ = valid.identity();
    let _ = valid.timeout_policy();
    let _ = valid.server_config();

    // Platform trust source
    let server = pki.server_identity();
    let identity = ClientIdentity::new(
        CertificateInput::from_der(vec![server.certificate_der().to_vec()]),
        PrivateKeyInput::from_der(server.private_key_der().to_vec()),
    );
    let platform_res = ListenerTransportConfig::builder("127.0.0.1", 5696)
        .listener_identity(identity)
        .trust_source(TrustSource::Platform)
        .build();
    assert!(platform_res.is_ok());
}

#[test]
fn listener_accept_timeouts_and_local_addr() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config1 = test_listener_config(&pki)
        .build()
        .expect("listener config is valid");

    let mut raw_listener = RawTlsListener::bind(config1).expect("bind succeeds");
    assert!(raw_listener.local_addr().is_ok());
    assert!(raw_listener.port().is_ok());

    let err = raw_listener.accept(Some(Duration::from_millis(10)));
    assert!(err.is_err());

    let config2 = test_listener_config(&pki)
        .target_path("/kmip")
        .build()
        .expect("listener config is valid");

    let mut https_listener = HttpsListener::bind(config2).expect("bind succeeds");
    assert!(https_listener.local_addr().is_ok());
    assert!(https_listener.port().is_ok());

    let err2 = https_listener.accept(Some(Duration::from_millis(10)));
    assert!(err2.is_err());
}

#[test]
fn https_listener_validation_and_codecs() {
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

        // 1. Wrong method: GET instead of POST
        let get_req = b"GET /kmip HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        tls.write_all(get_req).expect("write succeeds");
        tls.flush().expect("flush succeeds");
    });

    let mut conn = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");
    assert!(conn.peer_addr().ip().is_loopback());

    let err = conn.recv_request(1024, Some(Duration::from_secs(5)));
    assert!(err.is_err());

    client_handle.join().expect("client finishes");
}

#[test]
fn https_listener_xml_and_ttlv_exchanges() {
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

        // XML request
        let xml_body = b"<RequestMessage/>";
        let xml_req = format!(
            "POST /kmip HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/xml\r\nContent-Length: {}\r\n\r\n{}",
            xml_body.len(),
            std::str::from_utf8(xml_body).unwrap()
        );
        tls.write_all(xml_req.as_bytes()).expect("write succeeds");
        tls.flush().expect("flush succeeds");

        // Read XML response
        let mut resp_buf = Vec::new();
        let mut chunk = [0_u8; 512];
        while !resp_buf.windows(4).any(|w| w == b"\r\n\r\n")
            || !resp_buf.ends_with(b"<ResponseMessage/>")
        {
            let n = tls.read(&mut chunk).expect("read response");
            if n == 0 {
                break;
            }
            resp_buf.extend_from_slice(&chunk[..n]);
        }
        let resp_str = String::from_utf8_lossy(&resp_buf);
        assert!(resp_str.contains("Content-Type: text/xml"));
        assert!(resp_str.contains("<ResponseMessage/>"));
    });

    let mut conn = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");

    let (req, enc) = conn
        .recv_request(1024, Some(Duration::from_secs(5)))
        .expect("recv xml succeeds");
    assert_eq!(enc, WireEncoding::Xml);
    assert_eq!(req.as_bytes(), b"<RequestMessage/>");

    conn.send_response(
        b"<ResponseMessage/>",
        WireEncoding::Xml,
        Some(Duration::from_secs(5)),
    )
    .expect("send xml response succeeds");

    conn.close().expect("close succeeds");
    client_handle.join().expect("client finishes");
}

#[test]
fn raw_tls_server_connection_closed_and_limits() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config = test_listener_config(&pki)
        .max_request_bytes(64)
        .build()
        .expect("listener config is valid");

    let mut listener = RawTlsListener::bind(config).expect("bind succeeds");
    let port = listener.port().expect("port is valid");
    let client_config = client_tls_config(&pki);

    // 1. Test closed connection errors
    let client_handle = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect succeeds");
        let server_name =
            ServerName::try_from("server.kmipkit.test").expect("valid dns server name");
        let client_conn =
            ClientConnection::new(client_config, server_name).expect("client connection succeeds");
        let mut tls = StreamOwned::new(client_conn, sock);

        // Send a frame that claims to have 1000 bytes payload (exceeding limit of 64)
        let header = [0x42, 0x00, 0x77, 0x01, 0, 0, 0x03, 0xE8];
        tls.write_all(&header).expect("write header");
        tls.flush().expect("flush");
    });

    let mut conn = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");
    assert!(conn.peer_addr().ip().is_loopback());

    // Should fail with frame exceeds max_request_bytes
    let err = conn.recv_request(64, Some(Duration::from_secs(5)));
    assert!(err.is_err());

    // Close the connection
    conn.close().expect("close succeeds");

    // Operations on closed connection return error
    let err_recv = conn.recv_request(64, Some(Duration::from_secs(1)));
    assert!(err_recv.is_err());

    let err_send = conn.send_response(&[0; 8], Some(Duration::from_secs(1)));
    assert!(err_send.is_err());

    client_handle.join().expect("client finishes");
}

#[test]
fn raw_tls_role_reversed_limits_and_close() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config = test_listener_config(&pki)
        .build()
        .expect("listener config is valid");

    let mut listener = RawTlsListener::bind(config).expect("bind succeeds");
    let port = listener.port().expect("port is valid");
    let client_config = client_tls_config(&pki);

    let client_handle = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect succeeds");
        let server_name =
            ServerName::try_from("server.kmipkit.test").expect("valid dns server name");
        let client_conn =
            ClientConnection::new(client_config, server_name).expect("client connection succeeds");
        let mut tls = StreamOwned::new(client_conn, sock);

        // Send normal initial request
        let initial = [0x42, 0x00, 0x77, 0x01, 0, 0, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8];
        tls.write_all(&initial).expect("write initial");
        tls.flush().expect("flush");

        // Read normal initial response
        let mut resp = [0_u8; 16];
        tls.read_exact(&mut resp).expect("read initial response");

        // Now client sends an oversized frame to server's role-reversed reader (claims 500 bytes)
        let big_header = [0x42, 0x00, 0x77, 0x01, 0, 0, 0x01, 0xF4];
        tls.write_all(&big_header).expect("write big header");
        tls.flush().expect("flush");
    });

    let mut conn = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");
    let _ = conn
        .recv_request(1024, Some(Duration::from_secs(5)))
        .expect("recv succeeds");
    conn.send_response(
        &[0x42, 0x00, 0x7A, 0x01, 0, 0, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8],
        Some(Duration::from_secs(5)),
    )
    .expect("send succeeds");

    let mut rr = conn.into_role_reversed();

    // Server attempts to read with a max limit of 64 bytes -> exceeds limit
    let err = rr.recv_server_request(64, Some(Duration::from_secs(5)));
    assert!(err.is_err());

    rr.close().expect("close succeeds");
    client_handle.join().expect("client finishes");
}

#[test]
fn https_server_connection_closed_and_limits() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config = test_listener_config(&pki)
        .target_path("/kmip")
        .max_request_bytes(100)
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

        // 1. Path mismatch
        let bad_path = b"POST /wrong-path HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\n\r\n";
        tls.write_all(bad_path).expect("write bad path");
        tls.flush().expect("flush");
    });

    let mut conn = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");

    let err = conn.recv_request(1024, Some(Duration::from_secs(5)));
    assert!(err.is_err());

    conn.close().expect("close succeeds");

    // Operations on closed connection return error
    let err_recv = conn.recv_request(1024, Some(Duration::from_secs(1)));
    assert!(err_recv.is_err());

    let err_send = conn.send_response(b"{}", WireEncoding::Json, Some(Duration::from_secs(1)));
    assert!(err_send.is_err());

    client_handle.join().expect("client finishes");

    // 2. Body exceeds limit test
    let client_config2 = client_tls_config(&pki);
    let client_handle2 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect succeeds");
        let server_name =
            ServerName::try_from("server.kmipkit.test").expect("valid dns server name");
        let client_conn =
            ClientConnection::new(client_config2, server_name).expect("client connection succeeds");
        let mut tls = StreamOwned::new(client_conn, sock);

        let over_limit = b"POST /kmip HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 500\r\n\r\n";
        tls.write_all(over_limit).expect("write over limit");
        tls.flush().expect("flush");
    });

    let mut conn2 = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");

    let err2 = conn2.recv_request(100, Some(Duration::from_secs(5)));
    assert!(err2.is_err());

    client_handle2.join().expect("client finishes");
}

#[test]
fn listener_config_crl_and_invalid_inputs() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let server = pki.server_identity();

    // Invalid certificate DER for identity
    let bad_identity = ClientIdentity::new(
        CertificateInput::from_der(vec![vec![0xFF, 0x00]]),
        PrivateKeyInput::from_der(server.private_key_der().to_vec()),
    );
    let err = ListenerTransportConfig::builder("127.0.0.1", 5696)
        .listener_identity(bad_identity)
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .build();
    assert!(err.is_err());

    // Invalid trust DER
    let make_good_identity = || {
        ClientIdentity::new(
            CertificateInput::from_der(vec![server.certificate_der().to_vec()]),
            PrivateKeyInput::from_der(server.private_key_der().to_vec()),
        )
    };
    let err2 = ListenerTransportConfig::builder("127.0.0.1", 5696)
        .listener_identity(make_good_identity())
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![vec![0xAA, 0xBB]]),
        ]))
        .build();
    assert!(err2.is_err());

    // Valid with empty CRLs list
    let valid = ListenerTransportConfig::builder("127.0.0.1", 5696)
        .listener_identity(make_good_identity())
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .revocation_lists(Vec::new())
        .build();
    assert!(valid.is_ok());
}

#[test]
fn listener_builder_setters_and_unbounded_policy() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let server = pki.server_identity();
    let identity = ClientIdentity::new(
        CertificateInput::from_der(vec![server.certificate_der().to_vec()]),
        PrivateKeyInput::from_der(server.private_key_der().to_vec()),
    );
    let trust = TrustSource::certificate_authorities(vec![CertificateInput::from_der(vec![
        pki.authority_certificate_der().to_vec(),
    ])]);

    let unbounded_policy = crate::config::TimeoutPolicy::default()
        .with_read(crate::config::TimeoutLimit::Unbounded)
        .with_write(crate::config::TimeoutLimit::Unbounded);

    let config = ListenerTransportConfig::builder("127.0.0.1", 0)
        .client_identity(identity)
        .trust_source(trust)
        .timeout_policy(unbounded_policy)
        .wire_encoding(WireEncoding::Ttlv)
        .build()
        .expect("config builds");

    let mut listener = RawTlsListener::bind(config).expect("bind succeeds");
    let port = listener.port().expect("port");
    let client_config = client_tls_config(&pki);

    let client_handle = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect succeeds");
        let server_name =
            ServerName::try_from("server.kmipkit.test").expect("valid dns server name");
        let client_conn =
            ClientConnection::new(client_config, server_name).expect("client connection succeeds");
        let mut tls = StreamOwned::new(client_conn, sock);

        let req = [0x42, 0x00, 0x77, 0x01, 0, 0, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8];
        tls.write_all(&req).expect("write");
        tls.flush().expect("flush");

        let mut resp = [0_u8; 16];
        tls.read_exact(&mut resp).expect("read");
    });

    let mut conn = listener
        .accept(None)
        .expect("accept with None timeout succeeds");
    let req = conn.recv_request(1024, None).expect("recv without timeout");
    assert_eq!(req.as_bytes().len(), 16);
    conn.send_response(
        &[0x42, 0x00, 0x7A, 0x01, 0, 0, 0, 8, 9, 8, 7, 6, 5, 4, 3, 2],
        None,
    )
    .expect("send");
    conn.close().expect("close");
    client_handle.join().expect("client finishes");
}

#[test]
fn listener_bind_port_conflict() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config1 = test_listener_config(&pki).build().expect("config");
    let listener = RawTlsListener::bind(config1).expect("bind succeeds");
    let port = listener.port().expect("port");

    let make_config = || {
        ListenerTransportConfig::builder("127.0.0.1", port)
            .listener_identity(ClientIdentity::new(
                CertificateInput::from_der(vec![pki.server_identity().certificate_der().to_vec()]),
                PrivateKeyInput::from_der(pki.server_identity().private_key_der().to_vec()),
            ))
            .trust_source(TrustSource::certificate_authorities(vec![
                CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
            ]))
            .build()
            .expect("config builds")
    };

    let err1 = RawTlsListener::bind(make_config());
    assert!(err1.is_err());

    let err2 = HttpsListener::bind(make_config());
    assert!(err2.is_err());
}

#[test]
fn listener_tls_handshake_premature_eof() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config1 = test_listener_config(&pki).build().expect("config");
    let mut raw_listener = RawTlsListener::bind(config1).expect("bind succeeds");
    let port1 = raw_listener.port().expect("port");

    let handle1 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port1)).expect("connect");
        drop(sock);
    });

    let err1 = raw_listener.accept(Some(Duration::from_secs(5)));
    assert!(err1.is_err());
    handle1.join().expect("join");

    let config2 = test_listener_config(&pki).build().expect("config");
    let mut https_listener = HttpsListener::bind(config2).expect("bind succeeds");
    let port2 = https_listener.port().expect("port");

    let handle2 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port2)).expect("connect");
        drop(sock);
    });

    let err2 = https_listener.accept(Some(Duration::from_secs(5)));
    assert!(err2.is_err());
    handle2.join().expect("join");
}

#[test]
fn https_server_connection_more_edge_cases() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config = test_listener_config(&pki)
        .target_path("/kmip")
        .timeout_policy(
            crate::config::TimeoutPolicy::default()
                .with_read(crate::config::TimeoutLimit::Unbounded)
                .with_write(crate::config::TimeoutLimit::Unbounded),
        )
        .build()
        .expect("config");

    let mut listener = HttpsListener::bind(config).expect("bind succeeds");
    let port = listener.port().expect("port");
    let client_config = client_tls_config(&pki);

    // 1. Missing request line (client sends \r\n\r\n)
    let client_config1 = Arc::clone(&client_config);
    let handle1 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        let server_name = ServerName::try_from("server.kmipkit.test").unwrap();
        let client_conn = ClientConnection::new(client_config1, server_name).unwrap();
        let mut tls = StreamOwned::new(client_conn, sock);
        tls.write_all(b"\r\n\r\n").unwrap();
        tls.flush().unwrap();
    });

    let mut conn = listener.accept(None).expect("accept with None succeeds");
    let err = conn.recv_request(1024, None);
    assert!(err.is_err());
    conn.close().expect("close");
    handle1.join().unwrap();

    // 2. TTLV response (send_response with WireEncoding::Ttlv -> application/octet-stream)
    let client_config2 = Arc::clone(&client_config);
    let handle2 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        let server_name = ServerName::try_from("server.kmipkit.test").unwrap();
        let client_conn = ClientConnection::new(client_config2, server_name).unwrap();
        let mut tls = StreamOwned::new(client_conn, sock);

        let req = b"POST /kmip HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\n\r\n";
        tls.write_all(req).unwrap();
        tls.flush().unwrap();

        let mut resp = Vec::new();
        let mut buf = [0_u8; 512];
        loop {
            let n = tls.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            resp.extend_from_slice(&buf[..n]);
            if resp.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }
        let s = String::from_utf8_lossy(&resp);
        assert!(s.contains("Content-Type: application/octet-stream"));
    });

    let mut conn2 = listener
        .accept(Some(Duration::from_secs(5)))
        .expect("accept succeeds");
    let (body, enc) = conn2
        .recv_request(1024, Some(Duration::from_secs(5)))
        .expect("recv succeeds");
    assert_eq!(enc, WireEncoding::Ttlv);
    assert_eq!(body.as_bytes().len(), 0);

    conn2
        .send_response(&[0x42, 0, 0, 1], WireEncoding::Ttlv, None)
        .expect("send ttlv succeeds");
    conn2.close().expect("close");
    handle2.join().unwrap();
}

#[test]
fn test_https_listener_request_parsing_errors() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config = test_listener_config(&pki)
        .target_path("/kmip")
        .max_request_bytes(1024)
        .build()
        .expect("config succeeds");

    let mut listener = HttpsListener::bind(config).expect("bind succeeds");
    let port = listener.port().expect("port");
    let client_config = client_tls_config(&pki);

    // Test 1: Method not POST
    let cfg1 = Arc::clone(&client_config);
    let h1 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let server_name = ServerName::try_from("server.kmipkit.test").unwrap();
        let client_conn = ClientConnection::new(cfg1, server_name).unwrap();
        let mut tls = StreamOwned::new(client_conn, sock);
        tls.write_all(b"GET /kmip HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .unwrap();
        tls.flush().unwrap();
        let mut buf = [0_u8; 64];
        let _ = tls.read(&mut buf);
    });
    let mut conn = listener.accept(Some(Duration::from_secs(5))).unwrap();
    let err = conn.recv_request(1024, Some(Duration::from_secs(5)));
    assert!(err.is_err());
    let _ = conn.close();
    h1.join().unwrap();

    // Test 2: Path mismatch
    let cfg2 = Arc::clone(&client_config);
    let h2 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let server_name = ServerName::try_from("server.kmipkit.test").unwrap();
        let client_conn = ClientConnection::new(cfg2, server_name).unwrap();
        let mut tls = StreamOwned::new(client_conn, sock);
        tls.write_all(b"POST /other HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\n\r\n")
            .unwrap();
        tls.flush().unwrap();
        let mut buf = [0_u8; 64];
        let _ = tls.read(&mut buf);
    });
    let mut conn = listener.accept(Some(Duration::from_secs(5))).unwrap();
    let err = conn.recv_request(1024, Some(Duration::from_secs(5)));
    assert!(err.is_err());
    let _ = conn.close();
    h2.join().unwrap();

    // Test 3: Body exceeds max bytes
    let cfg3 = Arc::clone(&client_config);
    let h3 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let server_name = ServerName::try_from("server.kmipkit.test").unwrap();
        let client_conn = ClientConnection::new(cfg3, server_name).unwrap();
        let mut tls = StreamOwned::new(client_conn, sock);
        tls.write_all(b"POST /kmip HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 999999\r\n\r\n")
            .unwrap();
        tls.flush().unwrap();
        let mut buf = [0_u8; 64];
        let _ = tls.read(&mut buf);
    });
    let mut conn = listener.accept(Some(Duration::from_secs(5))).unwrap();
    let err = conn.recv_request(1024, Some(Duration::from_secs(5)));
    assert!(err.is_err());
    let _ = conn.close();
    h3.join().unwrap();
}

#[test]
fn test_https_listener_request_content_type_encodings() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config = test_listener_config(&pki)
        .target_path("/kmip")
        .max_request_bytes(1024)
        .build()
        .expect("config succeeds");

    let mut listener = HttpsListener::bind(config).expect("bind succeeds");
    let port = listener.port().expect("port");
    let client_config = client_tls_config(&pki);

    // Test 1: JSON content-type
    let cfg4 = Arc::clone(&client_config);
    let h4 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let server_name = ServerName::try_from("server.kmipkit.test").unwrap();
        let client_conn = ClientConnection::new(cfg4, server_name).unwrap();
        let mut tls = StreamOwned::new(client_conn, sock);
        tls.write_all(b"POST /kmip HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}").unwrap();
        tls.flush().unwrap();
        let mut buf = [0_u8; 64];
        let _ = tls.read(&mut buf);
    });
    let mut conn = listener.accept(Some(Duration::from_secs(5))).unwrap();
    let (body, enc) = conn
        .recv_request(1024, Some(Duration::from_secs(5)))
        .unwrap();
    assert_eq!(enc, WireEncoding::Json);
    assert_eq!(body.as_bytes(), b"{}");
    conn.send_response(b"{}", WireEncoding::Json, None).unwrap();
    let _ = conn.close();
    h4.join().unwrap();

    // Test 2: XML content-type
    let cfg5 = Arc::clone(&client_config);
    let h5 = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let server_name = ServerName::try_from("server.kmipkit.test").unwrap();
        let client_conn = ClientConnection::new(cfg5, server_name).unwrap();
        let mut tls = StreamOwned::new(client_conn, sock);
        tls.write_all(b"POST /kmip HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: text/xml\r\nContent-Length: 4\r\n\r\n<a/>").unwrap();
        tls.flush().unwrap();
        let mut buf = [0_u8; 64];
        let _ = tls.read(&mut buf);
    });
    let mut conn = listener.accept(Some(Duration::from_secs(5))).unwrap();
    let (body, enc) = conn
        .recv_request(1024, Some(Duration::from_secs(5)))
        .unwrap();
    assert_eq!(enc, WireEncoding::Xml);
    assert_eq!(body.as_bytes(), b"<a/>");
    conn.send_response(b"<a/>", WireEncoding::Xml, None)
        .unwrap();
    let _ = conn.close();
    h5.join().unwrap();
}

#[test]
fn test_raw_tls_server_connection_into_role_reversed_stream() {
    let pki = EphemeralPki::generate().expect("pki generation succeeds");
    let config = test_listener_config(&pki).build().expect("config succeeds");

    let mut listener = RawTlsListener::bind(config).expect("bind succeeds");
    let port = listener.port().expect("port");
    let client_config = client_tls_config(&pki);

    let client_h = thread::spawn(move || {
        let sock = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let server_name = ServerName::try_from("server.kmipkit.test").unwrap();
        let client_conn = ClientConnection::new(client_config, server_name).unwrap();
        let mut tls = StreamOwned::new(client_conn, sock);

        // Client sends an 8-byte frame (empty TTLV structure: tag=0x42007B, type=1, len=0)
        let frame = [0x42, 0x00, 0x7B, 0x01, 0x00, 0x00, 0x00, 0x00];
        tls.write_all(&frame).unwrap();
        tls.flush().unwrap();

        // Client reads response frame
        let mut resp = [0_u8; 8];
        tls.read_exact(&mut resp).unwrap();
        assert_eq!(resp, frame);
    });

    let conn = listener.accept(Some(Duration::from_secs(5))).unwrap();
    let mut rr_stream = conn.into_role_reversed();

    // Verify recv
    let received = rr_stream
        .recv_server_request(1024, Some(Duration::from_secs(5)))
        .expect("recv succeeds");
    assert_eq!(received.as_bytes().len(), 8);

    // Verify send
    rr_stream
        .send_client_response(received.as_bytes(), Some(Duration::from_secs(5)))
        .expect("send succeeds");
    rr_stream.close().expect("close succeeds");

    client_h.join().unwrap();
}
