//! End-to-end integration tests for kmipkit-test-server.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::similar_names,
    clippy::doc_markdown,
    clippy::format_push_string,
    clippy::semicolon_if_nothing_returned,
    clippy::unreadable_literal,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::collapsible_if,
    clippy::items_after_statements,
    clippy::manual_is_multiple_of,
    clippy::return_self_not_must_use,
    clippy::unnecessary_wraps,
    clippy::vec_init_then_push,
    clippy::bool_to_int_with_if
)]

use std::time::Duration;

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientRequest, RequestOptions, WireEncoding,
};
use kmipkit_test_server::codec::Format;
use kmipkit_test_server::ops::dispatcher::*;
use kmipkit_test_server::{ServerConfig, TestServer};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
    TransportConfig, TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn build_client(
    endpoint: Endpoint,
    pki: &kmipkit_test_server::AutoPki,
    encoding: WireEncoding,
) -> Client {
    let certificates = vec![pki.client_cert_der.clone()];
    let private_key = pki.client_key_der.to_vec();

    let transport_config = TransportConfig::builder(endpoint)
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der(certificates),
            PrivateKeyInput::from_der(private_key),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.ca_cert_der.clone()]),
        ]))
        .timeouts(
            TimeoutPolicy::default().with_total(TimeoutLimit::Bounded(Duration::from_secs(5))),
        )
        .build()
        .expect("transport config builds cleanly");

    let registry = client_extension_registry(Vec::new(), kmipkit_protocol::extension::defaults())
        .expect("empty registry is valid");
    let client_config = ClientConfiguration::new(registry).with_wire_encoding(encoding);

    Client::new(client_config, transport_config).expect("client builds cleanly")
}

#[tokio::test]
async fn raw_tls_client_executes_discover_versions() {
    let server = TestServer::start(ServerConfig::default())
        .await
        .expect("test server starts");

    let endpoint = Endpoint::raw_tls("127.0.0.1", server.port());
    let mut client = build_client(endpoint, server.pki(), WireEncoding::Ttlv);

    let outcome = tokio::task::spawn_blocking(move || {
        let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
        client
            .execute(batch, &CodecLimits::defaults())
            .expect("discover_versions over raw-TLS succeeds")
    })
    .await
    .unwrap();

    assert_eq!(outcome.len(), 1);

    server.shutdown().await.expect("clean shutdown");
}

#[tokio::test]
async fn https_client_executes_discover_versions_in_all_encodings() {
    let server = TestServer::start(ServerConfig::default())
        .await
        .expect("test server starts");

    let port = server.port();

    for encoding in [WireEncoding::Ttlv, WireEncoding::Json, WireEncoding::Xml] {
        let endpoint = Endpoint::https(format!("https://127.0.0.1:{port}"));
        let mut client = build_client(endpoint, server.pki(), encoding);

        let outcome = tokio::task::spawn_blocking(move || {
            let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
            client
                .execute_with_options(
                    batch,
                    &CodecLimits::defaults(),
                    &RequestOptions::default().with_wire_encoding(encoding),
                )
                .unwrap_or_else(|e| {
                    panic!("discover_versions failed for encoding {encoding:?}: {e}")
                })
        })
        .await
        .unwrap();

        assert_eq!(outcome.len(), 1);
    }

    server.shutdown().await.expect("clean shutdown");
}

#[tokio::test]
async fn full_lifecycle_and_crypto_direct_flow() {
    let server = TestServer::start(ServerConfig::default())
        .await
        .expect("test server starts");

    let mut tls = connect_tls(&server).await;

    // 1. Create a SymmetricKey
    let create_req = build_request(
        2,
        1,
        CREATE_OPERATION,
        vec![
            item(TAG_OBJECT_TYPE, Value::enumeration(2)), // SymmetricKey
        ],
    );
    let create_resp = send_and_recv(&mut tls, &create_req).await;
    let key_id = extract_text_field(&create_resp, TAG_UNIQUE_IDENTIFIER).expect("key id returned");
    assert_ne!(key_id, "");

    // 2. Activate the Key
    let activate_req = build_request(
        2,
        1,
        ACTIVATE_OPERATION,
        vec![item(
            TAG_UNIQUE_IDENTIFIER,
            Value::text_string(key_id.clone()),
        )],
    );
    let activate_resp = send_and_recv(&mut tls, &activate_req).await;
    assert_status_success(&activate_resp);

    // 3. Get the Key Material
    let get_req = build_request(
        2,
        1,
        GET_OPERATION,
        vec![item(
            TAG_UNIQUE_IDENTIFIER,
            Value::text_string(key_id.clone()),
        )],
    );
    let get_resp = send_and_recv(&mut tls, &get_req).await;
    assert_status_success(&get_resp);

    // 4. Encrypt Data
    let plaintext = b"Confidential KMIP payload 2026";
    let encrypt_req = build_request(
        2,
        1,
        ENCRYPT_OPERATION,
        vec![
            item(TAG_UNIQUE_IDENTIFIER, Value::text_string(key_id.clone())),
            item(TAG_DATA, Value::byte_string(plaintext.to_vec())),
        ],
    );
    let encrypt_resp = send_and_recv(&mut tls, &encrypt_req).await;
    assert_status_success(&encrypt_resp);
    let ciphertext = extract_byte_field(&encrypt_resp, TAG_DATA).expect("ciphertext returned");
    let iv = extract_byte_field(&encrypt_resp, TAG_IV_COUNTER_NONCE).expect("iv returned");
    assert_ne!(ciphertext, plaintext);

    // 5. Decrypt Data
    let decrypt_req = build_request(
        2,
        1,
        DECRYPT_OPERATION,
        vec![
            item(TAG_UNIQUE_IDENTIFIER, Value::text_string(key_id.clone())),
            item(TAG_DATA, Value::byte_string(ciphertext)),
            item(TAG_IV_COUNTER_NONCE, Value::byte_string(iv)),
        ],
    );
    let decrypt_resp = send_and_recv(&mut tls, &decrypt_req).await;
    assert_status_success(&decrypt_resp);
    let decrypted = extract_byte_field(&decrypt_resp, TAG_DATA).expect("decrypted text returned");
    assert_eq!(decrypted, plaintext);

    // 6. Locate Keys
    let locate_req = build_request(2, 1, LOCATE_OPERATION, vec![]);
    let locate_resp = send_and_recv(&mut tls, &locate_req).await;
    assert_status_success(&locate_resp);

    // 7. Hash Data
    let hash_req = build_request(
        2,
        1,
        HASH_OPERATION,
        vec![item(TAG_DATA, Value::byte_string(b"Test message".to_vec()))],
    );
    let hash_resp = send_and_recv(&mut tls, &hash_req).await;
    assert_status_success(&hash_resp);
    let digest = extract_byte_field(&hash_resp, TAG_DATA).expect("digest returned");
    assert_eq!(digest.len(), 32);

    // 8. RNG Retrieve
    let rng_req = build_request(
        2,
        1,
        RNG_RETRIEVE_OPERATION,
        vec![item(TAG_DATA_LENGTH, Value::integer(48))],
    );
    let rng_resp = send_and_recv(&mut tls, &rng_req).await;
    assert_status_success(&rng_resp);
    let rng_data = extract_byte_field(&rng_resp, TAG_DATA).expect("random bytes returned");
    assert_eq!(rng_data.len(), 48);

    // 9. Destroy Key
    let destroy_req = build_request(
        2,
        1,
        DESTROY_OPERATION,
        vec![item(
            TAG_UNIQUE_IDENTIFIER,
            Value::text_string(key_id.clone()),
        )],
    );
    let destroy_resp = send_and_recv(&mut tls, &destroy_req).await;
    assert_status_success(&destroy_resp);

    // 10. KMIP 1.2 Protocol Compatibility
    let v12_req = build_request(1, 2, DISCOVER_VERSIONS_OPERATION, vec![]);
    let v12_resp = send_and_recv(&mut tls, &v12_req).await;
    assert_status_success(&v12_resp);

    server.shutdown().await.expect("clean shutdown");
}

// Helpers

async fn connect_tls(
    server: &kmipkit_test_server::TestServerHandle,
) -> tokio_rustls::client::TlsStream<tokio::net::TcpStream> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store
        .add(rustls::pki_types::CertificateDer::from(
            server.pki().ca_cert_der.clone(),
        ))
        .unwrap();

    let cert_chain = vec![rustls::pki_types::CertificateDer::from(
        server.pki().client_cert_der.clone(),
    )];
    let key_der = rustls::pki_types::PrivateKeyDer::Pkcs8(
        rustls::pki_types::PrivatePkcs8KeyDer::from(server.pki().client_key_der.to_vec()),
    );

    let client_config = rustls::ClientConfig::builder_with_provider(std::sync::Arc::new(
        rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13])
    .unwrap()
    .with_root_certificates(root_store)
    .with_client_auth_cert(cert_chain, key_der)
    .unwrap();

    let connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(client_config));
    let tcp = tokio::net::TcpStream::connect(server.local_addr())
        .await
        .unwrap();
    let server_name = rustls::pki_types::ServerName::try_from("127.0.0.1".to_string()).unwrap();
    connector.connect(server_name, tcp).await.unwrap()
}

async fn send_and_recv(
    tls: &mut tokio_rustls::client::TlsStream<tokio::net::TcpStream>,
    req: &Item,
) -> Item {
    let req_bytes = Format::Ttlv.encode(req).unwrap();
    tls.write_all(&req_bytes).await.unwrap();
    tls.flush().await.unwrap();

    let mut header = [0u8; 8];
    tls.read_exact(&mut header).await.unwrap();
    let len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let mut payload = vec![0u8; len];
    tls.read_exact(&mut payload).await.unwrap();
    let mut resp_bytes = header.to_vec();
    resp_bytes.extend_from_slice(&payload);

    Format::Ttlv.decode(&resp_bytes).unwrap()
}

fn item(tag: u32, val: Value) -> Item {
    let raw = RawTag::new(tag).unwrap().try_checked().unwrap();
    Item::new(raw, val).unwrap()
}

fn build_request(major: i32, minor: i32, op: u32, payload_items: Vec<Item>) -> Item {
    let mut version = Structure::new();
    let _ = version.try_push(item(TAG_PROTOCOL_VERSION_MAJOR, Value::integer(major)));
    let _ = version.try_push(item(TAG_PROTOCOL_VERSION_MINOR, Value::integer(minor)));

    let mut header = Structure::new();
    let _ = header.try_push(item(TAG_PROTOCOL_VERSION, Value::structure(version)));
    let _ = header.try_push(item(TAG_BATCH_COUNT, Value::integer(1)));

    let mut payload = Structure::new();
    for p in payload_items {
        let _ = payload.try_push(p);
    }

    let mut batch_item = Structure::new();
    let _ = batch_item.try_push(item(TAG_OPERATION, Value::enumeration(op)));
    let _ = batch_item.try_push(item(TAG_REQUEST_PAYLOAD, Value::structure(payload)));

    let mut req = Structure::new();
    let _ = req.try_push(item(TAG_REQUEST_HEADER, Value::structure(header)));
    let _ = req.try_push(item(TAG_BATCH_ITEM, Value::structure(batch_item)));

    item(TAG_REQUEST_MESSAGE, Value::structure(req))
}

fn assert_status_success(resp: &Item) {
    let mut status = None;
    resp.with_value(|view| {
        if let ValueView::Structure(s) = view {
            for child in s.children() {
                if child.tag().raw() == TAG_BATCH_ITEM {
                    child.with_value(|bv| {
                        if let ValueView::Structure(bs) = bv {
                            for bchild in bs.children() {
                                if bchild.tag().raw() == TAG_RESULT_STATUS {
                                    bchild.with_value(|sv| {
                                        if let ValueView::Enumeration(e) = sv {
                                            status = Some(*e);
                                        }
                                    });
                                }
                            }
                        }
                    });
                }
            }
        }
    });
    assert_eq!(status, Some(STATUS_SUCCESS));
}

fn extract_text_field(resp: &Item, tag: u32) -> Option<String> {
    let mut result = None;
    resp.with_value(|view| {
        if let ValueView::Structure(s) = view {
            for child in s.children() {
                if child.tag().raw() == TAG_BATCH_ITEM {
                    child.with_value(|bv| {
                        if let ValueView::Structure(bs) = bv {
                            for bchild in bs.children() {
                                if bchild.tag().raw() == TAG_RESPONSE_PAYLOAD {
                                    bchild.with_value(|pv| {
                                        if let ValueView::Structure(ps) = pv {
                                            for pchild in ps.children() {
                                                if pchild.tag().raw() == tag {
                                                    pchild.with_value(|tv| {
                                                        if let ValueView::TextString(s) = tv {
                                                            result = Some(s.to_string());
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    });
                }
            }
        }
    });
    result
}

fn extract_byte_field(resp: &Item, tag: u32) -> Option<Vec<u8>> {
    let mut result = None;
    resp.with_value(|view| {
        if let ValueView::Structure(s) = view {
            for child in s.children() {
                if child.tag().raw() == TAG_BATCH_ITEM {
                    child.with_value(|bv| {
                        if let ValueView::Structure(bs) = bv {
                            for bchild in bs.children() {
                                if bchild.tag().raw() == TAG_RESPONSE_PAYLOAD {
                                    bchild.with_value(|pv| {
                                        if let ValueView::Structure(ps) = pv {
                                            for pchild in ps.children() {
                                                if pchild.tag().raw() == tag {
                                                    pchild.with_value(|tv| {
                                                        if let ValueView::ByteString(b) = tv {
                                                            result = Some(b.to_vec());
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    });
                }
            }
        }
    });
    result
}
