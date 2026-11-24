//! Wire protocol, mTLS (TLS 1.2 & 1.3), HTTP keep-alive, TOML/CLI config,
//! security bounds, deterministic fuzzing, and 64-client stress tests.
#![forbid(unsafe_code)]

use std::sync::Arc;

use async_trait::async_trait;
use kmipkit_proxy::codec::Format;
use kmipkit_proxy::config::{CliArgs, OperationPolicyMode, ProxyConfig};
use kmipkit_proxy::kmip1x::mapping::UpstreamKmipVersion;
use kmipkit_proxy::kmip1x::tags::*;
use kmipkit_proxy::kmip1x::ttlv_helpers::{
    build_protocol_version, build_response_message, build_success_batch_item, extract_enumeration,
    extract_integer, find_child, ttlv_item, ttlv_struct,
};
use kmipkit_proxy::kmip1x::{Kmip1xBridge, UpstreamKmipTransport};
use kmipkit_proxy::server::ProxyServer;
use kmipkit_ttlv::{Item, Value};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

struct SimpleEchoUpstream(UpstreamKmipVersion);

#[async_trait]
impl UpstreamKmipTransport for SimpleEchoUpstream {
    async fn send_ttlv(&self, request: &Item) -> Result<Item, String> {
        let (maj, min) = self.0.major_minor();
        let bi = find_child(request, TAG_BATCH_ITEM).unwrap();
        let op = find_child(&bi, TAG_OPERATION)
            .and_then(|i| extract_enumeration(&i))
            .unwrap_or(OP_CREATE);
        let rbi = build_success_batch_item(
            op,
            None,
            vec![
                ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string("wire-key-1".into()),
                )
                .unwrap(),
            ],
        )?;
        build_response_message(maj, min, vec![rbi])
    }
}

fn build_create_1x(minor: i32) -> Item {
    ttlv_struct(
        TAG_REQUEST_MESSAGE,
        vec![
            ttlv_struct(
                TAG_REQUEST_HEADER,
                vec![
                    build_protocol_version(1, minor).unwrap(),
                    ttlv_item(TAG_BATCH_COUNT, Value::integer(1)).unwrap(),
                ],
            )
            .unwrap(),
            ttlv_struct(
                TAG_BATCH_ITEM,
                vec![
                    ttlv_item(TAG_OPERATION, Value::enumeration(OP_CREATE)).unwrap(),
                    ttlv_struct(
                        TAG_REQUEST_PAYLOAD,
                        vec![
                            ttlv_item(
                                TAG_OBJECT_TYPE,
                                Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn build_tls_client_config(
    pki: &kmipkit_proxy::pki::AutoPki,
    protocol: &'static rustls::SupportedProtocolVersion,
    with_client_cert: bool,
) -> Arc<rustls::ClientConfig> {
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from(pki.ca_cert_der.clone()))
        .unwrap();
    let provider = Arc::new(rustls_openssl::default_provider());
    let builder = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[protocol])
        .unwrap()
        .with_root_certificates(roots);

    let cfg = if with_client_cert {
        let cert = CertificateDer::from(pki.client_cert_der.clone());
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(pki.client_key_der.to_vec()));
        builder.with_client_auth_cert(vec![cert], key).unwrap()
    } else {
        builder.with_no_client_auth()
    };
    Arc::new(cfg)
}

async fn send_raw_ttlv_over_stream<S: AsyncReadExt + AsyncWriteExt + Unpin>(
    stream: &mut S,
    req_bytes: &[u8],
) -> Result<Item, String> {
    stream
        .write_all(req_bytes)
        .await
        .map_err(|e| format!("write error: {e}"))?;
    stream
        .flush()
        .await
        .map_err(|e| format!("flush error: {e}"))?;

    let mut hdr = [0u8; 8];
    stream
        .read_exact(&mut hdr)
        .await
        .map_err(|e| format!("read hdr error: {e}"))?;
    let payload_len = u32::from_be_bytes([hdr[4], hdr[5], hdr[6], hdr[7]]) as usize;
    let padding = (8 - payload_len % 8) % 8;
    let mut body = vec![0u8; payload_len + padding];
    stream
        .read_exact(&mut body)
        .await
        .map_err(|e| format!("read body error: {e}"))?;

    let mut full = Vec::with_capacity(8 + body.len());
    full.extend_from_slice(&hdr);
    full.extend_from_slice(&body);
    Format::Ttlv.decode(&full)
}

#[tokio::test]
async fn test_real_mtls_tls13_and_tls12_handshakes_and_unauth_rejection() {
    // 1. Server with allow_tls12 = true: accepts both TLS 1.3 and TLS 1.2 mTLS, rejects no-cert
    let bridge = Arc::new(Kmip1xBridge::new(
        Arc::new(SimpleEchoUpstream(UpstreamKmipVersion::V2_1)),
        UpstreamKmipVersion::V2_1,
    ));
    let mut cfg = ProxyConfig::default();
    cfg.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    cfg.listener.tls_enabled = true;
    cfg.listener.allow_tls12 = true;
    cfg.listener.require_client_cert = true;

    let handle = ProxyServer::start_kmip1x_bridge(cfg, bridge.clone())
        .await
        .unwrap();
    let addr = handle.local_addr();
    let pki = handle.pki().unwrap();

    let req_bytes = Format::Ttlv.encode(&build_create_1x(4)).unwrap();

    // TLS 1.3 mTLS succeeds
    let tls13_cfg = build_tls_client_config(pki, &rustls::version::TLS13, true);
    let tcp = TcpStream::connect(addr).await.unwrap();
    let mut tls13 = TlsConnector::from(tls13_cfg)
        .connect(ServerName::try_from("localhost").unwrap(), tcp)
        .await
        .unwrap();
    let resp13 = send_raw_ttlv_over_stream(&mut tls13, &req_bytes)
        .await
        .unwrap();
    assert_eq!(
        find_child(&resp13, TAG_RESPONSE_HEADER)
            .and_then(|h| find_child(&h, TAG_PROTOCOL_VERSION))
            .and_then(|pv| find_child(&pv, TAG_PROTOCOL_VERSION_MINOR))
            .and_then(|m| extract_integer(&m)),
        Some(4)
    );

    // TLS 1.2 mTLS succeeds
    let tls12_cfg = build_tls_client_config(pki, &rustls::version::TLS12, true);
    let tcp2 = TcpStream::connect(addr).await.unwrap();
    let mut tls12 = TlsConnector::from(tls12_cfg)
        .connect(ServerName::try_from("localhost").unwrap(), tcp2)
        .await
        .unwrap();
    let resp12 = send_raw_ttlv_over_stream(&mut tls12, &req_bytes)
        .await
        .unwrap();
    assert!(find_child(&resp12, TAG_BATCH_ITEM).is_some());

    // Client without certificate fails handshake or stream read
    let no_cert_cfg = build_tls_client_config(pki, &rustls::version::TLS13, false);
    let tcp3 = TcpStream::connect(addr).await.unwrap();
    let no_cert_res = TlsConnector::from(no_cert_cfg)
        .connect(ServerName::try_from("localhost").unwrap(), tcp3)
        .await;
    if let Ok(mut stream) = no_cert_res {
        assert!(
            send_raw_ttlv_over_stream(&mut stream, &req_bytes)
                .await
                .is_err()
        );
    }

    handle.shutdown().await.unwrap();

    // 2. Server with allow_tls12 = false: rejects TLS 1.2
    let mut cfg_strict = ProxyConfig::default();
    cfg_strict.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    cfg_strict.listener.tls_enabled = true;
    cfg_strict.listener.allow_tls12 = false;
    cfg_strict.listener.require_client_cert = true;

    let handle_strict = ProxyServer::start_kmip1x_bridge(cfg_strict, bridge)
        .await
        .unwrap();
    let tls12_rejected_cfg =
        build_tls_client_config(handle_strict.pki().unwrap(), &rustls::version::TLS12, true);
    let tcp4 = TcpStream::connect(handle_strict.local_addr())
        .await
        .unwrap();
    let res12 = TlsConnector::from(tls12_rejected_cfg)
        .connect(ServerName::try_from("localhost").unwrap(), tcp4)
        .await;
    assert!(res12.is_err());
    handle_strict.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_http_keep_alive_multiple_formats_and_malformed_content_length_rejected() {
    let bridge = Arc::new(Kmip1xBridge::new(
        Arc::new(SimpleEchoUpstream(UpstreamKmipVersion::V2_1)),
        UpstreamKmipVersion::V2_1,
    ));
    let mut cfg = ProxyConfig::default();
    cfg.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    cfg.listener.tls_enabled = false;
    let handle = ProxyServer::start_kmip1x_bridge(cfg, bridge).await.unwrap();

    // Sequential keep-alive HTTP requests over a single TCP stream (JSON then XML then TTLV with Connection: close)
    let mut stream = TcpStream::connect(handle.local_addr()).await.unwrap();
    let req_item = build_create_1x(3);

    for (idx, fmt) in [Format::Json, Format::Xml, Format::Ttlv].iter().enumerate() {
        let body = fmt.encode(&req_item).unwrap();
        let conn_hdr = if idx == 2 { "close" } else { "keep-alive" };
        let http_req = format!(
            "POST /kmip HTTP/1.1\r\nHost: localhost\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: {}\r\n\r\n",
            fmt.content_type(),
            body.len(),
            conn_hdr
        );
        stream.write_all(http_req.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
        stream.flush().await.unwrap();

        let mut hdr_bytes = Vec::new();
        let mut b = [0u8; 1];
        while !hdr_bytes.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut b).await.unwrap();
            hdr_bytes.push(b[0]);
        }
        let hdr_str = String::from_utf8_lossy(&hdr_bytes);
        assert!(hdr_str.starts_with("HTTP/1.1 200 OK"));
        let cl = hdr_str
            .lines()
            .find_map(|l| {
                l.to_ascii_lowercase()
                    .strip_prefix("content-length:")
                    .map(|v| v.trim().parse::<usize>().unwrap())
            })
            .unwrap();
        let mut resp_body = vec![0u8; cl];
        stream.read_exact(&mut resp_body).await.unwrap();
        let resp_item = fmt.decode(&resp_body).unwrap();
        assert!(find_child(&resp_item, TAG_RESPONSE_HEADER).is_some());
    }

    // Malformed Content-Length header closes connection cleanly
    let mut bad_stream = TcpStream::connect(handle.local_addr()).await.unwrap();
    bad_stream
        .write_all(b"POST /kmip HTTP/1.1\r\nHost: localhost\r\nContent-Length: notanumber\r\n\r\n")
        .await
        .unwrap();
    let mut tmp = [0u8; 16];
    let read_res = bad_stream.read(&mut tmp).await.unwrap();
    assert_eq!(read_res, 0);

    handle.shutdown().await.unwrap();
}

#[test]
fn test_config_toml_parsing_upstream_versions_and_cli_merge() {
    assert_eq!(
        UpstreamKmipVersion::parse_str("2.0"),
        Ok(UpstreamKmipVersion::V2_0)
    );
    assert_eq!(
        UpstreamKmipVersion::parse_str("v2.1"),
        Ok(UpstreamKmipVersion::V2_1)
    );
    assert_eq!(
        UpstreamKmipVersion::parse_str("3_0"),
        Ok(UpstreamKmipVersion::V3_0)
    );
    assert!(UpstreamKmipVersion::parse_str("1.4").is_err());

    let toml_str = r#"
[listener]
bind_addr = "127.0.0.1:5696"
tls_enabled = true
allow_tls12 = true
require_client_cert = false

[backend]
provider = "kmip-upstream"

[backend.kmip_upstream]
endpoint = "kms.internal:5697"
target_version = "3.0"
wire_encoding = "json"
allow_tls12 = true
timeout_secs = 15
operation_policy_mode = "store_as_vendor_attribute"
"#;
    let parsed: ProxyConfig = toml::from_str(toml_str).unwrap();
    assert!(parsed.listener.allow_tls12);
    assert_eq!(parsed.backend.provider, "kmip-upstream");
    assert_eq!(parsed.backend.kmip_upstream.target_version, "3.0");
    assert_eq!(
        parsed.backend.kmip_upstream.operation_policy_mode,
        OperationPolicyMode::StoreAsVendorAttribute
    );

    let cli = CliArgs {
        config: None,
        listen: Some("127.0.0.1:7777".parse().unwrap()),
        provider: Some("kmip-upstream".into()),
        upstream_endpoint: Some("https://kms.local:8443/kmip".into()),
        upstream_version: Some("2.0".into()),
        allow_tls12: true,
        master_kek_hex: None,
        no_client_auth: true,
        no_tls: true,
    };
    let merged = cli.merge_into(ProxyConfig::default());
    assert_eq!(merged.listener.bind_addr.port(), 7777);
    assert!(!merged.listener.tls_enabled);
    assert!(!merged.listener.require_client_cert);
    assert!(merged.listener.allow_tls12);
    assert_eq!(merged.backend.kmip_upstream.target_version, "2.0");
}

#[tokio::test]
async fn test_deterministic_fuzzing_over_ttlv_and_bridge_dispatch_never_panics() {
    let bridge = Kmip1xBridge::new(
        Arc::new(SimpleEchoUpstream(UpstreamKmipVersion::V2_1)),
        UpstreamKmipVersion::V2_1,
    );
    let base_wire = Format::Ttlv.encode(&build_create_1x(4)).unwrap();

    // 256 deterministic mutations over valid TTLV frames
    for seed in 0usize..256 {
        let mut mutated = base_wire.clone();
        let pos = (seed * 17 + 3) % mutated.len();
        mutated[pos] ^= ((seed * 31 + 7) & 0xFF) as u8;
        if seed % 5 == 0 && mutated.len() > 8 {
            mutated.truncate(mutated.len() - (seed % 8 + 1));
        }
        if let Ok(decoded) = Format::Ttlv.decode(&mutated) {
            let _ = bridge.dispatch(&decoded).await;
        }
    }
}

#[tokio::test]
async fn test_64_concurrent_tcp_clients_mixed_operations_on_live_bridge() {
    let bridge = Arc::new(Kmip1xBridge::new(
        Arc::new(SimpleEchoUpstream(UpstreamKmipVersion::V3_0)),
        UpstreamKmipVersion::V3_0,
    ));
    let mut cfg = ProxyConfig::default();
    cfg.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    cfg.listener.tls_enabled = false;
    let handle = ProxyServer::start_kmip1x_bridge(cfg, bridge).await.unwrap();
    let addr = handle.local_addr();

    let mut tasks = Vec::new();
    for client_id in 0..64 {
        tasks.push(tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            let req = if client_id % 2 == 0 {
                build_create_1x(client_id % 5)
            } else {
                ttlv_struct(
                    TAG_REQUEST_MESSAGE,
                    vec![
                        ttlv_struct(
                            TAG_REQUEST_HEADER,
                            vec![
                                build_protocol_version(1, 4).unwrap(),
                                ttlv_item(TAG_BATCH_COUNT, Value::integer(1)).unwrap(),
                            ],
                        )
                        .unwrap(),
                        ttlv_struct(
                            TAG_BATCH_ITEM,
                            vec![
                                ttlv_item(TAG_OPERATION, Value::enumeration(OP_DISCOVER_VERSIONS))
                                    .unwrap(),
                            ],
                        )
                        .unwrap(),
                    ],
                )
                .unwrap()
            };
            let bytes = Format::Ttlv.encode(&req).unwrap();
            let resp = send_raw_ttlv_over_stream(&mut stream, &bytes)
                .await
                .unwrap();
            let rbi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
            assert_eq!(
                find_child(&rbi, TAG_RESULT_STATUS).and_then(|i| extract_enumeration(&i)),
                Some(STATUS_SUCCESS)
            );
        }));
    }

    for t in tasks {
        t.await.unwrap();
    }
    handle.shutdown().await.unwrap();
}
