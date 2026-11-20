//! Integration and foundation tests for KMIPKit Proxy.
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

use std::sync::Arc;
use std::time::Duration;

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientRequest, RequestOptions, WireEncoding,
};
use kmipkit_proxy::codec::Format;
use kmipkit_proxy::dispatcher::{
    DECRYPT_OPERATION, DESTROY_OPERATION, ENCRYPT_OPERATION, TAG_BATCH_COUNT, TAG_BATCH_ITEM,
    TAG_DATA, TAG_OPERATION, TAG_PROTOCOL_VERSION, TAG_PROTOCOL_VERSION_MAJOR,
    TAG_PROTOCOL_VERSION_MINOR, TAG_REQUEST_HEADER, TAG_REQUEST_MESSAGE, TAG_REQUEST_PAYLOAD,
    TAG_RESPONSE_PAYLOAD, TAG_RESULT_STATUS, TAG_UNIQUE_IDENTIFIER,
};
use kmipkit_proxy::providers::InMemoryMockProvider;
use kmipkit_proxy::server::ProxyServer;
use kmipkit_proxy::{AutoPki, ProxyConfig};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
    TransportConfig, TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn build_client(endpoint: Endpoint, pki: &AutoPki, encoding: WireEncoding) -> Client {
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

fn ttlv_item(tag: u32, val: Value) -> Item {
    let raw = RawTag::new(tag).unwrap().try_checked().unwrap();
    Item::new(raw, val).unwrap()
}

fn build_raw_request(op: u32, payload_items: Vec<Item>) -> Item {
    let mut header = Structure::new();
    let mut version = Structure::new();
    let _ = version.try_push(ttlv_item(TAG_PROTOCOL_VERSION_MAJOR, Value::integer(2)));
    let _ = version.try_push(ttlv_item(TAG_PROTOCOL_VERSION_MINOR, Value::integer(1)));
    let _ = header.try_push(ttlv_item(TAG_PROTOCOL_VERSION, Value::structure(version)));
    let _ = header.try_push(ttlv_item(TAG_BATCH_COUNT, Value::integer(1)));

    let mut batch_item = Structure::new();
    let _ = batch_item.try_push(ttlv_item(TAG_OPERATION, Value::enumeration(op)));
    if !payload_items.is_empty() {
        let mut payload_struct = Structure::new();
        for item in payload_items {
            let _ = payload_struct.try_push(item);
        }
        let _ = batch_item.try_push(ttlv_item(
            TAG_REQUEST_PAYLOAD,
            Value::structure(payload_struct),
        ));
    }

    let mut message = Structure::new();
    let _ = message.try_push(ttlv_item(TAG_REQUEST_HEADER, Value::structure(header)));
    let _ = message.try_push(ttlv_item(TAG_BATCH_ITEM, Value::structure(batch_item)));

    ttlv_item(TAG_REQUEST_MESSAGE, Value::structure(message))
}

#[tokio::test]
async fn test_proxy_discover_versions_raw_tls_client() {
    let mut config = ProxyConfig::default();
    config.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    let provider = Arc::new(InMemoryMockProvider::default());

    let server = ProxyServer::start(config, provider)
        .await
        .expect("proxy server starts");

    let pki = server.pki().expect("pki was generated");
    let endpoint = Endpoint::raw_tls("127.0.0.1", server.port());
    let mut client = build_client(endpoint, pki, WireEncoding::Ttlv);

    let outcome = tokio::task::spawn_blocking(move || {
        let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
        client
            .execute(batch, &CodecLimits::defaults())
            .expect("discover_versions succeeds")
    })
    .await
    .unwrap();

    assert_eq!(outcome.len(), 1);

    server.shutdown().await.expect("clean shutdown");
}

#[tokio::test]
async fn test_proxy_discover_versions_https_all_encodings() {
    let mut config = ProxyConfig::default();
    config.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    let provider = Arc::new(InMemoryMockProvider::default());

    let server = ProxyServer::start(config, provider)
        .await
        .expect("proxy server starts");

    let pki = server.pki().expect("pki was generated");
    let port = server.port();

    for encoding in [WireEncoding::Ttlv, WireEncoding::Json, WireEncoding::Xml] {
        let endpoint = Endpoint::https(format!("https://127.0.0.1:{port}"));
        let mut client = build_client(endpoint, pki, encoding);

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
async fn test_proxy_full_cryptographic_lifecycle() {
    let mut config = ProxyConfig::default();
    config.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    config.listener.tls_enabled = false; // Plain TCP for direct wire frame testing
    let provider = Arc::new(InMemoryMockProvider::default());

    let server = ProxyServer::start(config, provider)
        .await
        .expect("proxy server starts");

    let port = server.port();
    let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
        .await
        .expect("connect to plain TCP proxy");

    // 1. Create a 256-bit AES symmetric key
    let create_req = build_raw_request(1, Vec::new()); // 1 = Create
    let create_bytes = Format::Ttlv.encode(&create_req).unwrap();
    stream.write_all(&create_bytes).await.unwrap();
    stream.flush().await.unwrap();

    let mut header = [0u8; 8];
    stream.read_exact(&mut header).await.unwrap();
    let len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let pad = (8 - len % 8) % 8;
    let mut payload = vec![0u8; len + pad];
    stream.read_exact(&mut payload).await.unwrap();

    let mut full = header.to_vec();
    full.extend_from_slice(&payload);
    let resp = Format::Ttlv.decode(&full).unwrap();

    // Extract UniqueIdentifier
    let mut created_id = String::new();
    resp.with_value(|v| {
        if let ValueView::Structure(s) = v {
            for batch in s.children() {
                batch.with_value(|bv| {
                    if let ValueView::Structure(bs) = bv {
                        for item in bs.children() {
                            if item.tag().raw() == TAG_RESPONSE_PAYLOAD {
                                item.with_value(|pv| {
                                    if let ValueView::Structure(ps) = pv {
                                        for pitem in ps.children() {
                                            if pitem.tag().raw() == TAG_UNIQUE_IDENTIFIER {
                                                pitem.with_value(|idv| {
                                                    if let ValueView::TextString(id) = idv {
                                                        created_id = id.to_string();
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
    });

    assert!(!created_id.is_empty(), "Unique identifier returned");

    // 2. Encrypt plaintext
    let plaintext = b"stateless-zero-db-kmip-proxy-secret-payload".to_vec();
    let encrypt_req = build_raw_request(
        ENCRYPT_OPERATION,
        vec![
            ttlv_item(
                TAG_UNIQUE_IDENTIFIER,
                Value::text_string(created_id.clone()),
            ),
            ttlv_item(TAG_DATA, Value::byte_string(plaintext.clone())),
        ],
    );
    let encrypt_bytes = Format::Ttlv.encode(&encrypt_req).unwrap();
    stream.write_all(&encrypt_bytes).await.unwrap();
    stream.flush().await.unwrap();

    stream.read_exact(&mut header).await.unwrap();
    let len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let pad = (8 - len % 8) % 8;
    let mut payload = vec![0u8; len + pad];
    stream.read_exact(&mut payload).await.unwrap();

    let mut full = header.to_vec();
    full.extend_from_slice(&payload);
    let enc_resp = Format::Ttlv.decode(&full).unwrap();

    let mut ciphertext = Vec::new();
    enc_resp.with_value(|v| {
        if let ValueView::Structure(s) = v {
            for batch in s.children() {
                batch.with_value(|bv| {
                    if let ValueView::Structure(bs) = bv {
                        for item in bs.children() {
                            if item.tag().raw() == TAG_RESPONSE_PAYLOAD {
                                item.with_value(|pv| {
                                    if let ValueView::Structure(ps) = pv {
                                        for pitem in ps.children() {
                                            if pitem.tag().raw() == TAG_DATA {
                                                pitem.with_value(|dv| {
                                                    if let ValueView::ByteString(bytes) = dv {
                                                        ciphertext = bytes.to_vec();
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
    });

    assert!(!ciphertext.is_empty(), "Ciphertext returned");
    assert_ne!(ciphertext, plaintext, "Ciphertext differs from plaintext");

    // 3. Decrypt ciphertext -> recover plaintext
    let decrypt_req = build_raw_request(
        DECRYPT_OPERATION,
        vec![
            ttlv_item(
                TAG_UNIQUE_IDENTIFIER,
                Value::text_string(created_id.clone()),
            ),
            ttlv_item(TAG_DATA, Value::byte_string(ciphertext)),
        ],
    );
    let decrypt_bytes = Format::Ttlv.encode(&decrypt_req).unwrap();
    stream.write_all(&decrypt_bytes).await.unwrap();
    stream.flush().await.unwrap();

    stream.read_exact(&mut header).await.unwrap();
    let len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let pad = (8 - len % 8) % 8;
    let mut payload = vec![0u8; len + pad];
    stream.read_exact(&mut payload).await.unwrap();

    let mut full = header.to_vec();
    full.extend_from_slice(&payload);
    let dec_resp = Format::Ttlv.decode(&full).unwrap();

    let mut recovered_plaintext = Vec::new();
    dec_resp.with_value(|v| {
        if let ValueView::Structure(s) = v {
            for batch in s.children() {
                batch.with_value(|bv| {
                    if let ValueView::Structure(bs) = bv {
                        for item in bs.children() {
                            if item.tag().raw() == TAG_RESPONSE_PAYLOAD {
                                item.with_value(|pv| {
                                    if let ValueView::Structure(ps) = pv {
                                        for pitem in ps.children() {
                                            if pitem.tag().raw() == TAG_DATA {
                                                pitem.with_value(|dv| {
                                                    if let ValueView::ByteString(bytes) = dv {
                                                        recovered_plaintext = bytes.to_vec();
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
    });

    assert_eq!(
        recovered_plaintext, plaintext,
        "Decrypted plaintext matches original plaintext"
    );

    // 4. Destroy key
    let destroy_req = build_raw_request(
        DESTROY_OPERATION,
        vec![ttlv_item(
            TAG_UNIQUE_IDENTIFIER,
            Value::text_string(created_id.clone()),
        )],
    );
    let destroy_bytes = Format::Ttlv.encode(&destroy_req).unwrap();
    stream.write_all(&destroy_bytes).await.unwrap();
    stream.flush().await.unwrap();

    stream.read_exact(&mut header).await.unwrap();
    let len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let pad = (8 - len % 8) % 8;
    let mut payload = vec![0u8; len + pad];
    stream.read_exact(&mut payload).await.unwrap();

    let mut full = header.to_vec();
    full.extend_from_slice(&payload);
    let destroy_resp = Format::Ttlv.decode(&full).unwrap();

    let mut status = 1;
    destroy_resp.with_value(|v| {
        if let ValueView::Structure(s) = v {
            for batch in s.children() {
                batch.with_value(|bv| {
                    if let ValueView::Structure(bs) = bv {
                        for item in bs.children() {
                            if item.tag().raw() == TAG_RESULT_STATUS {
                                item.with_value(|sv| {
                                    if let ValueView::Enumeration(st) = sv {
                                        status = *st;
                                    }
                                });
                            }
                        }
                    }
                });
            }
        }
    });
    assert_eq!(status, 0, "Destroy succeeded with status 0");

    server.shutdown().await.expect("clean shutdown");
}
