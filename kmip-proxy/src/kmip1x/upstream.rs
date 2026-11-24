//! Upstream KMIP 2.0 / 2.1 / 3.0 transport adapter backed by `kmipkit-transport`.
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, HttpsTransport, PrivateKeyInput, RawTlsTransport,
    RequestOptions, TimeoutLimit, TimeoutPolicy, TlsVersionPolicy, TransportConfig, TrustSource,
};
use kmipkit_ttlv::Item;

use crate::codec::Format;
use crate::config::KmipUpstreamConfig;
use crate::kmip1x::UpstreamKmipTransport;

const DEFAULT_MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

enum TransportKind {
    RawTls(Mutex<RawTlsTransport>),
    Https(Mutex<HttpsTransport>),
}

/// Production upstream KMIP 2.0 / 2.1 / 3.0 transport built on `kmipkit-transport`.
///
/// Supports both Raw TLS (`host:port`) and HTTPS (`https://host:port/kmip`) endpoints,
/// with TTLV, JSON, or XML wire encoding toward the upstream KMIP server.
pub struct KmipKitUpstreamTransport {
    transport: TransportKind,
    wire_format: Format,
    max_response_bytes: usize,
}

impl KmipKitUpstreamTransport {
    /// Constructs a production upstream transport from `KmipUpstreamConfig`.
    pub fn from_config(cfg: &KmipUpstreamConfig) -> Result<Self, String> {
        let endpoint = if cfg.endpoint.starts_with("https://") {
            Endpoint::https(cfg.endpoint.clone())
        } else {
            let (host, port_str) = cfg.endpoint.rsplit_once(':').ok_or_else(|| {
                format!(
                    "invalid upstream endpoint '{}', expected host:port or https://...",
                    cfg.endpoint
                )
            })?;
            let port: u16 = port_str
                .parse()
                .map_err(|e| format!("invalid upstream port '{port_str}': {e}"))?;
            Endpoint::raw_tls(host.to_string(), port)
        };

        let cert_path = cfg
            .client_cert_path
            .as_ref()
            .ok_or_else(|| "kmip_upstream.client_cert_path is required".to_string())?;
        let key_path = cfg
            .client_key_path
            .as_ref()
            .ok_or_else(|| "kmip_upstream.client_key_path is required".to_string())?;

        let cert_input = if cert_path.extension().and_then(|e| e.to_str()) == Some("der") {
            CertificateInput::from_der_file(cert_path)
        } else {
            CertificateInput::from_pem_file(cert_path)
        }
        .map_err(|e| format!("invalid client certificate: {e}"))?;
        let key_input = if key_path.extension().and_then(|e| e.to_str()) == Some("der") {
            PrivateKeyInput::from_der_file(key_path)
        } else {
            PrivateKeyInput::from_pem_file(key_path)
        }
        .map_err(|e| format!("invalid client key: {e}"))?;

        let trust = if let Some(ca_path) = &cfg.ca_cert_path {
            let ca_input = if ca_path.extension().and_then(|e| e.to_str()) == Some("der") {
                CertificateInput::from_der_file(ca_path)
            } else {
                CertificateInput::from_pem_file(ca_path)
            }
            .map_err(|e| format!("invalid CA certificate: {e}"))?;
            TrustSource::certificate_authorities(vec![ca_input])
        } else {
            TrustSource::platform()
        };

        let timeout_secs = cfg.timeout_secs.unwrap_or(30);
        let tls_policy = if cfg.allow_tls12 {
            TlsVersionPolicy::Tls12And13Compat
        } else {
            TlsVersionPolicy::Tls13Only
        };

        let transport_config = TransportConfig::builder(endpoint)
            .client_identity(ClientIdentity::new(cert_input, key_input))
            .trust_source(trust)
            .tls_version_policy(tls_policy)
            .timeouts(
                TimeoutPolicy::default()
                    .with_total(TimeoutLimit::Bounded(Duration::from_secs(timeout_secs))),
            )
            .build()
            .map_err(|e| format!("failed building upstream TransportConfig: {e}"))?;

        let wire_format = match cfg.wire_encoding.as_deref().unwrap_or("ttlv") {
            "json" => Format::Json,
            "xml" => Format::Xml,
            _ => Format::Ttlv,
        };

        Self::from_transport_config(transport_config, wire_format)
    }

    /// Constructs a production upstream transport from a pre-built `TransportConfig`.
    pub fn from_transport_config(
        transport_config: TransportConfig,
        wire_format: Format,
    ) -> Result<Self, String> {
        let transport = if transport_config.endpoint().raw_tls_address_is_ok() {
            let raw = RawTlsTransport::new(transport_config)
                .map_err(|e| format!("failed creating RawTlsTransport: {e}"))?;
            TransportKind::RawTls(Mutex::new(raw))
        } else {
            let https = HttpsTransport::new(transport_config)
                .map_err(|e| format!("failed creating HttpsTransport: {e}"))?;
            TransportKind::Https(Mutex::new(https))
        };

        Ok(Self {
            transport,
            wire_format,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
        })
    }
}

trait EndpointKindExt {
    fn raw_tls_address_is_ok(&self) -> bool;
}

impl EndpointKindExt for Endpoint {
    fn raw_tls_address_is_ok(&self) -> bool {
        // Endpoint::Debug prints `kind: "RawTls"` vs `kind: "Https"` without leaking host
        format!("{self:?}").contains("RawTls")
    }
}

#[async_trait]
impl UpstreamKmipTransport for KmipKitUpstreamTransport {
    async fn send_ttlv(&self, request: &Item) -> Result<Item, String> {
        let req_bytes = self.wire_format.encode(request)?;
        let max_bytes = self.max_response_bytes;
        let wire_format = self.wire_format;

        let resp_bytes = match &self.transport {
            TransportKind::RawTls(mutex) => {
                let mut guard = mutex
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let opts = RequestOptions::default();
                let resp = guard
                    .exchange_with_options(&req_bytes, max_bytes, &opts)
                    .map_err(|e| format!("upstream RawTLS exchange failed: {e}"))?;
                resp.as_bytes().to_vec()
            }
            TransportKind::Https(mutex) => {
                let mut guard = mutex
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let opts = RequestOptions::default();
                let resp = guard
                    .exchange_with_options(&req_bytes, max_bytes, &opts)
                    .map_err(|e| format!("upstream HTTPS exchange failed: {e}"))?;
                resp.as_bytes().to_vec()
            }
        };

        wire_format.decode(&resp_bytes)
    }
}

/// In-process adapter allowing `Kmip1xBridge` to wrap any `Arc<crate::dispatcher::Dispatcher>`
/// (for example, chaining KMIP 1.X legacy clients -> KMIP 1.X Bridge -> Cloud KMS Proxy!).
pub struct LocalDispatcherUpstream {
    dispatcher: Arc<crate::dispatcher::RequestDispatcher>,
}

impl LocalDispatcherUpstream {
    #[must_use]
    pub fn new(dispatcher: Arc<crate::dispatcher::RequestDispatcher>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl UpstreamKmipTransport for LocalDispatcherUpstream {
    async fn send_ttlv(&self, request: &Item) -> Result<Item, String> {
        self.dispatcher.dispatch(request).await
    }
}
