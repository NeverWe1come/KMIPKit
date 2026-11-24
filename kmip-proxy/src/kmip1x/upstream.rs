//! Upstream KMIP 2.0 / 2.1 / 3.0 transport adapter backed by `kmipkit-transport`.
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, HttpsTransport, PrivateKeyInput, RawTlsTransport,
    RequestOptions, TimeoutLimit, TimeoutPolicy, TlsVersionPolicy, TransportConfig, TrustSource,
    WireEncoding,
};
use kmipkit_ttlv::Item;
use tokio::sync::Semaphore;

use crate::codec::Format;
use crate::config::KmipUpstreamConfig;
use crate::kmip1x::UpstreamKmipTransport;

const DEFAULT_MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_POOL_SIZE: usize = 8;

enum TransportKind {
    RawTls(RawTlsTransport),
    Https(HttpsTransport),
}

impl TransportKind {
    fn from_transport_config(transport_config: TransportConfig) -> Result<Self, String> {
        if transport_config.endpoint().raw_tls_address_is_ok() {
            let raw = RawTlsTransport::new(transport_config)
                .map_err(|e| format!("failed creating RawTlsTransport: {e}"))?;
            Ok(Self::RawTls(raw))
        } else {
            let https = HttpsTransport::new(transport_config)
                .map_err(|e| format!("failed creating HttpsTransport: {e}"))?;
            Ok(Self::Https(https))
        }
    }

    fn exchange(
        &mut self,
        req_bytes: &[u8],
        max_bytes: usize,
        opts: &RequestOptions,
    ) -> Result<Vec<u8>, String> {
        match self {
            Self::RawTls(raw) => {
                let resp = raw
                    .exchange_with_options(req_bytes, max_bytes, opts)
                    .map_err(|e| format!("upstream RawTLS exchange failed: {e}"))?;
                Ok(resp.as_bytes().to_vec())
            }
            Self::Https(https) => {
                let resp = https
                    .exchange_with_options(req_bytes, max_bytes, opts)
                    .map_err(|e| format!("upstream HTTPS exchange failed: {e}"))?;
                Ok(resp.as_bytes().to_vec())
            }
        }
    }
}

/// Production upstream KMIP 2.0 / 2.1 / 3.0 transport built on `kmipkit-transport`.
///
/// Uses a bounded concurrent connection pool (`max_connections`, default 8) of
/// persistent Raw TLS or HTTPS transports, executing blocking TLS I/O off the Tokio
/// worker threads via `spawn_blocking` and setting the matching wire encoding.
pub struct KmipKitUpstreamTransport {
    pool: Arc<Mutex<Vec<TransportKind>>>,
    semaphore: Arc<Semaphore>,
    upstream_config: Option<KmipUpstreamConfig>,
    wire_format: Format,
    max_response_bytes: usize,
}

impl KmipKitUpstreamTransport {
    /// Constructs a pooled production upstream transport from `KmipUpstreamConfig`.
    pub fn from_config(cfg: &KmipUpstreamConfig) -> Result<Self, String> {
        let initial_config = build_transport_config(cfg)?;
        let initial_transport = TransportKind::from_transport_config(initial_config)?;
        let wire_format = wire_format_from_config(cfg);
        let pool_size = cfg
            .max_connections
            .unwrap_or(DEFAULT_POOL_SIZE)
            .clamp(1, 128);

        Ok(Self {
            pool: Arc::new(Mutex::new(vec![initial_transport])),
            semaphore: Arc::new(Semaphore::new(pool_size)),
            upstream_config: Some(cfg.clone()),
            wire_format,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
        })
    }

    /// Constructs a single-connection upstream transport from a pre-built `TransportConfig`.
    pub fn from_transport_config(
        transport_config: TransportConfig,
        wire_format: Format,
    ) -> Result<Self, String> {
        let transport = TransportKind::from_transport_config(transport_config)?;
        Ok(Self {
            pool: Arc::new(Mutex::new(vec![transport])),
            semaphore: Arc::new(Semaphore::new(1)),
            upstream_config: None,
            wire_format,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
        })
    }
}

fn wire_format_from_config(cfg: &KmipUpstreamConfig) -> Format {
    match cfg.wire_encoding.as_deref().unwrap_or("ttlv") {
        "json" => Format::Json,
        "xml" => Format::Xml,
        _ => Format::Ttlv,
    }
}

fn build_transport_config(cfg: &KmipUpstreamConfig) -> Result<TransportConfig, String> {
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

    TransportConfig::builder(endpoint)
        .client_identity(ClientIdentity::new(cert_input, key_input))
        .trust_source(trust)
        .tls_version_policy(tls_policy)
        .timeouts(
            TimeoutPolicy::default()
                .with_total(TimeoutLimit::Bounded(Duration::from_secs(timeout_secs))),
        )
        .build()
        .map_err(|e| format!("failed building upstream TransportConfig: {e}"))
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

        let _permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| "upstream transport pool closed".to_string())?;

        let pooled = {
            let mut guard = self
                .pool
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.pop()
        };

        let upstream_cfg = self.upstream_config.clone();
        let pool_ref = Arc::clone(&self.pool);

        let transport_wire = match wire_format {
            Format::Ttlv => WireEncoding::Ttlv,
            Format::Json => WireEncoding::Json,
            Format::Xml => WireEncoding::Xml,
        };

        let resp_bytes = tokio::task::spawn_blocking(move || {
            let mut transport = match pooled {
                Some(t) => t,
                None => {
                    let cfg = upstream_cfg
                        .as_ref()
                        .ok_or_else(|| "no upstream transport available in pool".to_string())?;
                    TransportKind::from_transport_config(build_transport_config(cfg)?)?
                }
            };

            let opts = RequestOptions::default().with_wire_encoding(transport_wire);
            let res = transport.exchange(&req_bytes, max_bytes, &opts);

            // Return healthy transport to pool (or keep single non-factory transport)
            if res.is_ok() || upstream_cfg.is_none() {
                let mut guard = pool_ref
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                guard.push(transport);
            }
            res
        })
        .await
        .map_err(|e| format!("upstream blocking task join error: {e}"))??;

        wire_format.decode(&resp_bytes)
    }
}

/// In-process adapter allowing `Kmip1xBridge` to wrap any `Arc<crate::dispatcher::RequestDispatcher>`
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
