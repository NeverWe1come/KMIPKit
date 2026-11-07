//! Outbound client for pushing Server-to-Client requests from TestServer to ClientListener.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use kmipkit_ttlv::{Item, RawTag, Structure, Value};

use crate::codec::Format;
use crate::ops::dispatcher::{
    DISCOVER_VERSIONS_OPERATION, QUERY_OPERATION, TAG_BATCH_COUNT, TAG_BATCH_ITEM, TAG_OPERATION,
    TAG_PROTOCOL_VERSION, TAG_PROTOCOL_VERSION_MAJOR, TAG_PROTOCOL_VERSION_MINOR,
    TAG_REQUEST_HEADER, TAG_REQUEST_MESSAGE, TAG_REQUEST_PAYLOAD, TAG_UNIQUE_IDENTIFIER,
};
use crate::pki::AutoPki;

/// Outbound TLS client capable of pushing Server-to-Client operations to a [`ClientListener`].
pub struct S2cClient {
    connector: TlsConnector,
}

impl S2cClient {
    /// Creates a new `S2cClient` with mutual TLS configured from `pki`.
    pub fn new(pki: &AutoPki) -> Result<Self, String> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store
            .add(rustls::pki_types::CertificateDer::from(
                pki.ca_cert_der.clone(),
            ))
            .map_err(|e| format!("CA cert error: {e}"))?;

        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        let cert_chain = vec![rustls::pki_types::CertificateDer::from(
            pki.client_cert_der.clone(),
        )];
        let key_der = rustls::pki_types::PrivateKeyDer::Pkcs8(
            rustls::pki_types::PrivatePkcs8KeyDer::from(pki.client_key_der.to_vec()),
        );

        let client_tls_config = rustls::ClientConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(|e| format!("TLS 1.3 config error: {e}"))?
            .with_root_certificates(root_store)
            .with_client_auth_cert(cert_chain, key_der)
            .map_err(|e| format!("client auth config error: {e}"))?;

        Ok(Self {
            connector: TlsConnector::from(Arc::new(client_tls_config)),
        })
    }

    /// Sends an S2C request to `addr` over raw TLS TTLV and returns the decoded ResponseMessage Item.
    pub async fn send_raw_ttlv(&self, addr: SocketAddr, request: &Item) -> Result<Item, String> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| format!("connect to {addr} failed: {e}"))?;

        let server_name = rustls::pki_types::ServerName::try_from("localhost".to_string())
            .map_err(|e| format!("invalid server name: {e}"))?;

        let mut tls_stream = self
            .connector
            .connect(server_name, stream)
            .await
            .map_err(|e| format!("TLS connect failed: {e}"))?;

        let req_bytes = Format::Ttlv
            .encode(request)
            .map_err(|e| format!("TTLV encode error: {e}"))?;

        tls_stream
            .write_all(&req_bytes)
            .await
            .map_err(|e| format!("TLS write failed: {e}"))?;
        tls_stream
            .flush()
            .await
            .map_err(|e| format!("TLS flush failed: {e}"))?;

        let mut header = [0u8; 8];
        tls_stream
            .read_exact(&mut header)
            .await
            .map_err(|e| format!("read TTLV header failed: {e}"))?;

        let payload_len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
        let padding = (8 - payload_len % 8) % 8;
        let total = payload_len + padding;

        let mut payload = vec![0u8; total];
        tls_stream
            .read_exact(&mut payload)
            .await
            .map_err(|e| format!("read TTLV payload failed: {e}"))?;

        let mut full_resp = Vec::with_capacity(8 + total);
        full_resp.extend_from_slice(&header);
        full_resp.extend_from_slice(&payload);

        Format::Ttlv
            .decode(&full_resp)
            .map_err(|e| format!("TTLV decode response failed: {e}"))
    }

    /// Sends an S2C request to `addr` over HTTPS using `format` and returns the decoded ResponseMessage Item.
    pub async fn send_https(
        &self,
        addr: SocketAddr,
        request: &Item,
        format: Format,
    ) -> Result<Item, String> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| format!("connect to {addr} failed: {e}"))?;

        let server_name = rustls::pki_types::ServerName::try_from("localhost".to_string())
            .map_err(|e| format!("invalid server name: {e}"))?;

        let mut tls_stream = self
            .connector
            .connect(server_name, stream)
            .await
            .map_err(|e| format!("TLS connect failed: {e}"))?;

        let body_bytes = format
            .encode(request)
            .map_err(|e| format!("encode error: {e}"))?;

        let content_type = match format {
            Format::Ttlv => "application/octet-stream",
            Format::Json => "application/json",
            Format::Xml => "text/xml",
        };

        let http_request = format!(
            "POST /kmip HTTP/1.1\r\nHost: localhost:{}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            addr.port(),
            content_type,
            body_bytes.len(),
        );

        tls_stream
            .write_all(http_request.as_bytes())
            .await
            .map_err(|e| format!("HTTP header write failed: {e}"))?;
        tls_stream
            .write_all(&body_bytes)
            .await
            .map_err(|e| format!("HTTP body write failed: {e}"))?;
        tls_stream
            .flush()
            .await
            .map_err(|e| format!("HTTP flush failed: {e}"))?;

        let mut resp_buf = Vec::new();
        let mut byte = [0u8; 1];
        let mut matched_end = false;
        while resp_buf.len() < 16 * 1024 {
            tls_stream
                .read_exact(&mut byte)
                .await
                .map_err(|e| format!("HTTP read response failed: {e}"))?;
            resp_buf.push(byte[0]);
            if resp_buf.ends_with(b"\r\n\r\n") {
                matched_end = true;
                break;
            }
        }

        if !matched_end {
            return Err("missing CRLF CRLF in HTTP response".to_string());
        }

        let header_str = String::from_utf8_lossy(&resp_buf);
        let mut content_length = 0;
        for line in header_str.lines() {
            if let Some(val) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                content_length = val.trim().parse::<usize>().unwrap_or(0);
            }
        }

        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            tls_stream
                .read_exact(&mut body)
                .await
                .map_err(|e| format!("HTTP read body failed: {e}"))?;
        }

        format
            .decode(&body)
            .map_err(|e| format!("decode response body failed: {e}"))
    }
}

fn raw_item(tag: u32, val: Value) -> Item {
    let t = RawTag::new(tag).unwrap().try_checked().unwrap();
    Item::new(t, val).unwrap()
}

fn wrap_single_request(operation: u32, payload: Structure) -> Item {
    let mut header = Structure::new();
    let mut pv = Structure::new();
    pv.try_push(raw_item(TAG_PROTOCOL_VERSION_MAJOR, Value::integer(2)))
        .unwrap();
    pv.try_push(raw_item(TAG_PROTOCOL_VERSION_MINOR, Value::integer(1)))
        .unwrap();
    header
        .try_push(raw_item(TAG_PROTOCOL_VERSION, Value::structure(pv)))
        .unwrap();
    header
        .try_push(raw_item(TAG_BATCH_COUNT, Value::integer(1)))
        .unwrap();

    let mut batch_item = Structure::new();
    batch_item
        .try_push(raw_item(TAG_OPERATION, Value::enumeration(operation)))
        .unwrap();
    batch_item
        .try_push(raw_item(TAG_REQUEST_PAYLOAD, Value::structure(payload)))
        .unwrap();

    let mut root = Structure::new();
    root.try_push(raw_item(TAG_REQUEST_HEADER, Value::structure(header)))
        .unwrap();
    root.try_push(raw_item(TAG_BATCH_ITEM, Value::structure(batch_item)))
        .unwrap();

    raw_item(TAG_REQUEST_MESSAGE, Value::structure(root))
}

/// Builds an S2C Discover Versions RequestMessage Item.
pub fn build_s2c_discover_versions(versions: &[(u32, u32)]) -> Item {
    let mut pld = Structure::new();
    for (major, minor) in versions {
        let mut pv = Structure::new();
        pv.try_push(raw_item(
            TAG_PROTOCOL_VERSION_MAJOR,
            Value::integer(*major as i32),
        ))
        .unwrap();
        pv.try_push(raw_item(
            TAG_PROTOCOL_VERSION_MINOR,
            Value::integer(*minor as i32),
        ))
        .unwrap();
        pld.try_push(raw_item(TAG_PROTOCOL_VERSION, Value::structure(pv)))
            .unwrap();
    }
    wrap_single_request(DISCOVER_VERSIONS_OPERATION, pld)
}

/// Builds an S2C Notify RequestMessage Item.
pub fn build_s2c_notify(unique_identifier: &str) -> Item {
    let mut pld = Structure::new();
    pld.try_push(raw_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(unique_identifier.to_string()),
    ))
    .unwrap();
    wrap_single_request(0x1B, pld)
}

/// Builds an S2C Put RequestMessage Item.
pub fn build_s2c_put(unique_identifier: &str, object_item: Item) -> Item {
    let mut pld = Structure::new();
    pld.try_push(raw_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(unique_identifier.to_string()),
    ))
    .unwrap();
    pld.try_push(object_item).unwrap();
    wrap_single_request(0x1C, pld)
}

/// Builds an S2C Query RequestMessage Item.
pub fn build_s2c_query(query_functions: &[u32]) -> Item {
    let mut pld = Structure::new();
    for qf in query_functions {
        pld.try_push(raw_item(0x0042_007A, Value::enumeration(*qf)))
            .unwrap();
    }
    wrap_single_request(QUERY_OPERATION, pld)
}
