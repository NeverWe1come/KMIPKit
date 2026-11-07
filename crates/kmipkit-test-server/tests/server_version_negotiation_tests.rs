//! Tests for KMIP 2.0 version negotiation and enforcement in kmipkit-test-server.
#![allow(clippy::similar_names)]

use kmipkit_test_server::codec::Format;
use kmipkit_test_server::ops::dispatcher::*;
use kmipkit_test_server::{ServerConfig, TestServer};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn ttlv_item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag).unwrap().try_checked().unwrap();
    Item::new(tag, value).unwrap()
}

async fn build_raw_tls_stream(
    pki: &kmipkit_test_server::AutoPki,
    port: u16,
) -> tokio_rustls::client::TlsStream<tokio::net::TcpStream> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store
        .add(rustls::pki_types::CertificateDer::from(
            pki.ca_cert_der.clone(),
        ))
        .unwrap();

    let cert_chain = vec![rustls::pki_types::CertificateDer::from(
        pki.client_cert_der.clone(),
    )];
    let key_der = rustls::pki_types::PrivateKeyDer::Pkcs8(
        rustls::pki_types::PrivatePkcs8KeyDer::from(pki.client_key_der.to_vec()),
    );

    let provider = std::sync::Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let client_config = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .unwrap()
        .with_root_certificates(root_store)
        .with_client_auth_cert(cert_chain, key_der)
        .unwrap();

    let connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(client_config));
    let server_name = rustls::pki_types::ServerName::try_from("localhost").unwrap();

    let std_stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    std_stream.set_nonblocking(true).unwrap();
    let tokio_stream = tokio::net::TcpStream::from_std(std_stream).unwrap();

    connector.connect(server_name, tokio_stream).await.unwrap()
}

#[tokio::test]
async fn test_server_responds_with_requested_kmip20_version() {
    let dir = std::env::temp_dir().join(format!("kmip20-resp-ver-{}", std::process::id()));
    let config = ServerConfig::new()
        .with_data_dir(&dir)
        .with_clean(true);
    let handle = TestServer::start(config).await.unwrap();

    let mut stream = build_raw_tls_stream(handle.pki(), handle.port()).await;

    // Build Request Message with ProtocolVersion (2, 0)
    let mut version_struct = Structure::new();
    version_struct
        .try_push(ttlv_item(TAG_PROTOCOL_VERSION_MAJOR, Value::integer(2)))
        .unwrap();
    version_struct
        .try_push(ttlv_item(TAG_PROTOCOL_VERSION_MINOR, Value::integer(0)))
        .unwrap();

    let mut header_struct = Structure::new();
    header_struct
        .try_push(ttlv_item(
            TAG_PROTOCOL_VERSION,
            Value::structure(version_struct),
        ))
        .unwrap();
    header_struct
        .try_push(ttlv_item(TAG_BATCH_COUNT, Value::integer(1)))
        .unwrap();

    let mut item_struct = Structure::new();
    item_struct
        .try_push(ttlv_item(
            TAG_OPERATION,
            Value::enumeration(DISCOVER_VERSIONS_OPERATION),
        ))
        .unwrap();

    let mut msg_struct = Structure::new();
    msg_struct
        .try_push(ttlv_item(TAG_REQUEST_HEADER, Value::structure(header_struct)))
        .unwrap();
    msg_struct
        .try_push(ttlv_item(TAG_BATCH_ITEM, Value::structure(item_struct)))
        .unwrap();

    let request_item = ttlv_item(TAG_REQUEST_MESSAGE, Value::structure(msg_struct));
    let wire_bytes = Format::Ttlv.encode(&request_item).unwrap();

    stream.write_all(&wire_bytes).await.unwrap();

    let mut resp_buf = vec![0u8; 4096];
    let n = stream.read(&mut resp_buf).await.unwrap();
    let resp_item = Format::Ttlv.decode(&resp_buf[..n]).unwrap();

    // Verify response header protocol version is strictly (2, 0)
    resp_item.with_value(|view| {
        if let kmipkit_ttlv::ValueView::Structure(s) = view {
            let header = s.children().first().unwrap();
            assert_eq!(header.tag().raw(), TAG_RESPONSE_HEADER);
            header.with_value(|h_view| {
                if let kmipkit_ttlv::ValueView::Structure(h_struct) = h_view {
                    let pv = h_struct.children().first().unwrap();
                    assert_eq!(pv.tag().raw(), TAG_PROTOCOL_VERSION);
                    pv.with_value(|pv_view| {
                        if let kmipkit_ttlv::ValueView::Structure(pv_struct) = pv_view {
                            let major = &pv_struct.children()[0];
                            let minor = &pv_struct.children()[1];
                            assert!(major.with_value(|v| matches!(v, kmipkit_ttlv::ValueView::Integer(2))));
                            assert!(minor.with_value(|v| matches!(v, kmipkit_ttlv::ValueView::Integer(0))));
                        }
                    });
                }
            });
        }
    });

    handle.shutdown().await.unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_server_in_kmip20_mode_rejects_kmip21_operations() {
    let dir = std::env::temp_dir().join(format!("kmip20-reject-21-{}", std::process::id()));
    let config = ServerConfig::new()
        .with_data_dir(&dir)
        .with_clean(true)
        .with_protocol_version(2, 0);
    let handle = TestServer::start(config).await.unwrap();

    let mut stream = build_raw_tls_stream(handle.pki(), handle.port()).await;

    // Build Request Message with ProtocolVersion (2, 0) requesting Export (0x3A)
    let mut version_struct = Structure::new();
    version_struct
        .try_push(ttlv_item(TAG_PROTOCOL_VERSION_MAJOR, Value::integer(2)))
        .unwrap();
    version_struct
        .try_push(ttlv_item(TAG_PROTOCOL_VERSION_MINOR, Value::integer(0)))
        .unwrap();

    let mut header_struct = Structure::new();
    header_struct
        .try_push(ttlv_item(
            TAG_PROTOCOL_VERSION,
            Value::structure(version_struct),
        ))
        .unwrap();
    header_struct
        .try_push(ttlv_item(TAG_BATCH_COUNT, Value::integer(1)))
        .unwrap();

    let mut item_struct = Structure::new();
    item_struct
        .try_push(ttlv_item(TAG_OPERATION, Value::enumeration(0x0000_003A))) // Export
        .unwrap();

    let mut msg_struct = Structure::new();
    msg_struct
        .try_push(ttlv_item(TAG_REQUEST_HEADER, Value::structure(header_struct)))
        .unwrap();
    msg_struct
        .try_push(ttlv_item(TAG_BATCH_ITEM, Value::structure(item_struct)))
        .unwrap();

    let request_item = ttlv_item(TAG_REQUEST_MESSAGE, Value::structure(msg_struct));
    let wire_bytes = Format::Ttlv.encode(&request_item).unwrap();

    stream.write_all(&wire_bytes).await.unwrap();

    let mut resp_buf = vec![0u8; 4096];
    let n = stream.read(&mut resp_buf).await.unwrap();
    let resp_item = Format::Ttlv.decode(&resp_buf[..n]).unwrap();

    // Verify batch item response has STATUS_OPERATION_FAILED and REASON_OPERATION_NOT_SUPPORTED
    resp_item.with_value(|view| {
        if let kmipkit_ttlv::ValueView::Structure(s) = view {
            let batch_item = &s.children()[1];
            assert_eq!(batch_item.tag().raw(), TAG_BATCH_ITEM);
            batch_item.with_value(|b_view| {
                if let kmipkit_ttlv::ValueView::Structure(b_struct) = b_view {
                    let mut found_status = false;
                    let mut found_reason = false;
                    for child in b_struct.children() {
                        if child.tag().raw() == TAG_RESULT_STATUS {
                            assert!(child.with_value(|v| matches!(v, kmipkit_ttlv::ValueView::Enumeration(val) if *val == STATUS_OPERATION_FAILED)));
                            found_status = true;
                        }
                        if child.tag().raw() == TAG_RESULT_REASON {
                            assert!(child.with_value(|v| matches!(v, kmipkit_ttlv::ValueView::Enumeration(val) if *val == REASON_OPERATION_NOT_SUPPORTED)));
                            found_reason = true;
                        }
                    }
                    assert!(found_status, "expected STATUS_OPERATION_FAILED in response");
                    assert!(found_reason, "expected REASON_OPERATION_NOT_SUPPORTED in response");
                }
            });
        }
    });

    handle.shutdown().await.unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}
