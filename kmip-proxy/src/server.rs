//! TCP and TLS 1.2/1.3 listener for KMIPKit Proxy.
//!
//! Provides protocol sniffing (Raw TTLV, HTTPS with TTLV, JSON, XML),
//! TLS 1.3 (and optional TLS 1.2 for legacy KMIP 1.X clients) termination,
//! and connection dispatching to either [`RequestDispatcher`] (Cloud KMS bridge)
//! or [`Kmip1xBridge`] (KMIP 1.X -> 2.0/2.1/3.0 protocol translation bridge).
#![forbid(unsafe_code)]

use kmipkit_ttlv::Item;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;

use crate::codec::Format;
use crate::config::ProxyConfig;
use crate::dispatcher::RequestDispatcher;
#[cfg(feature = "kmip1x-bridge")]
use crate::kmip1x::Kmip1xBridge;
use crate::pki::AutoPki;
use crate::providers::CloudKmsProvider;

/// Internal dispatch target shared across connections.
enum ServerDispatchTarget {
    Cloud(Arc<RequestDispatcher>),
    #[cfg(feature = "kmip1x-bridge")]
    Kmip1xBridge(Arc<Kmip1xBridge>),
}

impl ServerDispatchTarget {
    async fn dispatch(&self, request: &Item) -> Result<Item, String> {
        match self {
            Self::Cloud(d) => d.dispatch(request).await,
            #[cfg(feature = "kmip1x-bridge")]
            Self::Kmip1xBridge(b) => b.dispatch(request).await,
        }
    }
}

/// Running proxy server handle.
pub struct ProxyServerHandle {
    local_addr: SocketAddr,
    pki: Option<AutoPki>,
    shutdown_tx: watch::Sender<bool>,
    task_handle: JoinHandle<Result<(), String>>,
}

impl ProxyServerHandle {
    /// Returns the local socket address bound by the proxy.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Returns the port bound by the proxy.
    pub fn port(&self) -> u16 {
        self.local_addr.port()
    }

    /// Returns the generated PKI if ephemeral PKI was created.
    pub fn pki(&self) -> Option<&AutoPki> {
        self.pki.as_ref()
    }

    /// Shuts down the proxy server gracefully.
    pub async fn shutdown(self) -> Result<(), String> {
        let _ = self.shutdown_tx.send(true);
        self.task_handle
            .await
            .map_err(|e| format!("join error: {e}"))?
    }
}

pub struct ProxyServer;

impl ProxyServer {
    /// Starts the Cloud KMS proxy server with an optional ephemeral PKI.
    pub async fn start(
        config: ProxyConfig,
        provider: Arc<dyn CloudKmsProvider>,
    ) -> Result<ProxyServerHandle, String> {
        let pki = if config.listener.tls_enabled && config.listener.tls_cert_path.is_none() {
            Some(AutoPki::generate().map_err(|e| format!("PKI generation error: {e}"))?)
        } else {
            None
        };
        Self::start_with_optional_pki(config, provider, pki).await
    }

    /// Starts the Cloud KMS proxy server with a pre-existing PKI.
    pub async fn start_with_pki(
        config: ProxyConfig,
        provider: Arc<dyn CloudKmsProvider>,
        pki: AutoPki,
    ) -> Result<ProxyServerHandle, String> {
        Self::start_with_optional_pki(config, provider, Some(pki)).await
    }

    /// Starts the KMIP 1.X -> KMIP 2.0/2.1/3.0 translation proxy server with optional ephemeral PKI.
    #[cfg(feature = "kmip1x-bridge")]
    pub async fn start_kmip1x_bridge(
        config: ProxyConfig,
        bridge: Arc<Kmip1xBridge>,
    ) -> Result<ProxyServerHandle, String> {
        let pki = if config.listener.tls_enabled && config.listener.tls_cert_path.is_none() {
            Some(AutoPki::generate().map_err(|e| format!("PKI generation error: {e}"))?)
        } else {
            None
        };
        Self::start_target(
            config,
            Arc::new(ServerDispatchTarget::Kmip1xBridge(bridge)),
            pki,
        )
        .await
    }

    /// Starts the KMIP 1.X -> KMIP 2.0/2.1/3.0 translation proxy server with a pre-existing PKI.
    #[cfg(feature = "kmip1x-bridge")]
    pub async fn start_kmip1x_bridge_with_pki(
        config: ProxyConfig,
        bridge: Arc<Kmip1xBridge>,
        pki: AutoPki,
    ) -> Result<ProxyServerHandle, String> {
        Self::start_target(
            config,
            Arc::new(ServerDispatchTarget::Kmip1xBridge(bridge)),
            Some(pki),
        )
        .await
    }

    async fn start_with_optional_pki(
        config: ProxyConfig,
        provider: Arc<dyn CloudKmsProvider>,
        pki: Option<AutoPki>,
    ) -> Result<ProxyServerHandle, String> {
        let dispatcher = Arc::new(RequestDispatcher::new(
            provider,
            config.proxy_behavior.clone(),
        ));
        Self::start_target(
            config,
            Arc::new(ServerDispatchTarget::Cloud(dispatcher)),
            pki,
        )
        .await
    }

    async fn start_target(
        config: ProxyConfig,
        target: Arc<ServerDispatchTarget>,
        pki: Option<AutoPki>,
    ) -> Result<ProxyServerHandle, String> {
        crate::crypto::install_rustls_openssl_provider()
            .map_err(|error| format!("failed to install rustls crypto provider: {error}"))?;

        let listener = TcpListener::bind(config.listener.bind_addr)
            .await
            .map_err(|e| format!("bind error on {}: {e}", config.listener.bind_addr))?;

        let local_addr = listener
            .local_addr()
            .map_err(|e| format!("local_addr error: {e}"))?;

        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let tls_acceptor = if config.listener.tls_enabled {
            let server_tls_config = Self::build_tls_config(&config, pki.as_ref())?;
            Some(TlsAcceptor::from(Arc::new(server_tls_config)))
        } else {
            None
        };

        let task_handle = tokio::spawn(async move {
            run_server_loop(listener, tls_acceptor, target, shutdown_rx).await
        });

        Ok(ProxyServerHandle {
            local_addr,
            pki,
            shutdown_tx,
            task_handle,
        })
    }

    fn build_tls_config(
        config: &ProxyConfig,
        pki: Option<&AutoPki>,
    ) -> Result<rustls::ServerConfig, String> {
        let (cert_der, key_der, ca_der) = if let Some(pki) = pki {
            (
                pki.server_cert_der.clone(),
                pki.server_key_der.to_vec(),
                Some(pki.ca_cert_der.clone()),
            )
        } else {
            let cert_path = config
                .listener
                .tls_cert_path
                .as_ref()
                .ok_or("tls_cert_path required when tls_enabled without AutoPki")?;
            let key_path = config
                .listener
                .tls_key_path
                .as_ref()
                .ok_or("tls_key_path required when tls_enabled without AutoPki")?;

            let cert_bytes = std::fs::read(cert_path)
                .map_err(|e| format!("failed reading cert file {}: {e}", cert_path.display()))?;
            let key_bytes = std::fs::read(key_path)
                .map_err(|e| format!("failed reading key file {}: {e}", key_path.display()))?;

            let ca_bytes =
                if let Some(ca_path) = &config.listener.tls_client_ca_path {
                    Some(std::fs::read(ca_path).map_err(|e| {
                        format!("failed reading CA file {}: {e}", ca_path.display())
                    })?)
                } else {
                    None
                };
            (cert_bytes, key_bytes, ca_bytes)
        };

        let crypto_provider = Arc::new(rustls_openssl::default_provider());
        let protocol_versions: &[&rustls::SupportedProtocolVersion] = if config.listener.allow_tls12
        {
            &[&rustls::version::TLS13, &rustls::version::TLS12]
        } else {
            &[&rustls::version::TLS13]
        };

        let builder = rustls::ServerConfig::builder_with_provider(crypto_provider)
            .with_protocol_versions(protocol_versions)
            .map_err(|e| format!("TLS protocol version config error: {e}"))?;

        let cert_chain = vec![rustls::pki_types::CertificateDer::from(cert_der)];
        let private_key = rustls::pki_types::PrivateKeyDer::Pkcs8(
            rustls::pki_types::PrivatePkcs8KeyDer::from(key_der),
        );

        let server_config = if config.listener.require_client_cert {
            let mut root_store = rustls::RootCertStore::empty();
            if let Some(ca) = ca_der {
                root_store
                    .add(rustls::pki_types::CertificateDer::from(ca))
                    .map_err(|e| format!("CA cert error: {e}"))?;
            }
            let verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(root_store))
                .build()
                .map_err(|e| format!("client verifier error: {e}"))?;
            builder
                .with_client_cert_verifier(verifier)
                .with_single_cert(cert_chain, private_key)
        } else {
            builder
                .with_no_client_auth()
                .with_single_cert(cert_chain, private_key)
        }
        .map_err(|e| format!("server TLS config error: {e}"))?;

        Ok(server_config)
    }
}

async fn run_server_loop(
    listener: TcpListener,
    tls_acceptor: Option<TlsAcceptor>,
    target: Arc<ServerDispatchTarget>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), String> {
    loop {
        tokio::select! {
            accept_res = listener.accept() => {
                let (stream, _peer_addr) = accept_res.map_err(|e| format!("accept error: {e}"))?;
                let acceptor = tls_acceptor.clone();
                let target = Arc::clone(&target);

                tokio::spawn(async move {
                    if let Some(acceptor) = acceptor {
                        if let Ok(tls_stream) = acceptor.accept(stream).await {
                            let _ = handle_connection_target(tls_stream, target).await;
                        }
                    } else {
                        let _ = handle_connection_target(stream, target).await;
                    }
                });
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Handles incoming stream for a Cloud KMS `RequestDispatcher`.
pub async fn handle_connection<S>(
    stream: S,
    dispatcher: Arc<RequestDispatcher>,
) -> Result<(), String>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    handle_connection_target(stream, Arc::new(ServerDispatchTarget::Cloud(dispatcher))).await
}

async fn handle_connection_target<S>(
    mut stream: S,
    target: Arc<ServerDispatchTarget>,
) -> Result<(), String>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    loop {
        let mut first_byte = [0u8; 1];
        match stream.read_exact(&mut first_byte).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(());
            }
            Err(e) => return Err(format!("socket read error: {e}")),
        }

        if first_byte[0] == 0x42 {
            handle_raw_ttlv(&mut stream, first_byte[0], &target).await?;
        } else if first_byte[0] == b'P' || first_byte[0] == b'p' {
            let keep_alive = handle_http(&mut stream, first_byte[0], &target).await?;
            if !keep_alive {
                break;
            }
        } else {
            return Err(format!(
                "unrecognized protocol starting with byte: 0x{:02x}",
                first_byte[0]
            ));
        }
    }

    Ok(())
}

async fn handle_raw_ttlv<S>(
    stream: &mut S,
    first_byte: u8,
    target: &ServerDispatchTarget,
) -> Result<(), String>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut header_rest = [0u8; 7];
    stream
        .read_exact(&mut header_rest)
        .await
        .map_err(|e| format!("failed reading TTLV header: {e}"))?;

    let mut full_header = [0u8; 8];
    full_header[0] = first_byte;
    full_header[1..8].copy_from_slice(&header_rest);

    let payload_len = u32::from_be_bytes([
        full_header[4],
        full_header[5],
        full_header[6],
        full_header[7],
    ]) as usize;
    if payload_len > 16 * 1024 * 1024 {
        return Err("TTLV payload exceeds maximum allowed size".to_string());
    }
    let padding = (8 - payload_len % 8) % 8;
    let total_to_read = payload_len + padding;
    if total_to_read > 16 * 1024 * 1024 {
        return Err("TTLV frame exceeds maximum allowed size".to_string());
    }

    let mut payload = vec![0u8; total_to_read];
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|e| format!("failed reading TTLV payload: {e}"))?;

    let mut full_request = Vec::with_capacity(8 + total_to_read);
    full_request.extend_from_slice(&full_header);
    full_request.extend_from_slice(&payload);

    let request_item = Format::Ttlv
        .decode(&full_request)
        .map_err(|e| format!("TTLV decode error: {e}"))?;

    let response_item = target.dispatch(&request_item).await?;
    let response_bytes = Format::Ttlv
        .encode(&response_item)
        .map_err(|e| format!("TTLV encode error: {e}"))?;

    stream
        .write_all(&response_bytes)
        .await
        .map_err(|e| format!("socket write error: {e}"))?;
    stream
        .flush()
        .await
        .map_err(|e| format!("socket flush error: {e}"))?;

    Ok(())
}

async fn handle_http<S>(
    stream: &mut S,
    first_byte: u8,
    target: &ServerDispatchTarget,
) -> Result<bool, String>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut header_buf = Vec::with_capacity(1024);
    header_buf.push(first_byte);

    let mut matched_end = false;
    let mut byte = [0u8; 1];
    while !matched_end && header_buf.len() < 16 * 1024 {
        stream
            .read_exact(&mut byte)
            .await
            .map_err(|e| format!("error reading HTTP header: {e}"))?;
        header_buf.push(byte[0]);
        if header_buf.ends_with(b"\r\n\r\n") {
            matched_end = true;
        }
    }

    if !matched_end {
        return Err("HTTP header too large or truncated".to_string());
    }

    let header_str = String::from_utf8_lossy(&header_buf);
    let mut content_length = 0;
    let mut content_type = None;
    let mut connection_close = false;

    for line in header_str.lines() {
        let line_lower = line.to_ascii_lowercase();
        if let Some(val) = line_lower.strip_prefix("content-length:") {
            content_length = val
                .trim()
                .parse::<usize>()
                .map_err(|e| format!("invalid Content-Length header: {e}"))?;
        } else if let Some(val) = line.strip_prefix("Content-Type:") {
            content_type = Some(val.trim().to_string());
        } else if let Some(val) = line_lower.strip_prefix("content-type:") {
            content_type = Some(val.trim().to_string());
        } else if let Some(val) = line_lower.strip_prefix("connection:") {
            if val.trim() == "close" {
                connection_close = true;
            }
        }
    }

    if content_length > 16 * 1024 * 1024 {
        return Err("HTTP body exceeds maximum allowed size".to_string());
    }
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        stream
            .read_exact(&mut body)
            .await
            .map_err(|e| format!("failed reading HTTP body: {e}"))?;
    }

    let format = Format::detect(content_type.as_deref(), &body);
    let request_item = format.decode(&body)?;
    let response_item = target.dispatch(&request_item).await?;
    let response_bytes = format.encode(&response_item)?;

    let status_line = "HTTP/1.1 200 OK\r\n";
    let conn_header = if connection_close {
        "Connection: close\r\n"
    } else {
        "Connection: keep-alive\r\n"
    };
    let content_type_header = format!("Content-Type: {}\r\n", format.content_type());
    let content_length_header = format!("Content-Length: {}\r\n\r\n", response_bytes.len());

    let mut http_response = Vec::new();
    http_response.extend_from_slice(status_line.as_bytes());
    http_response.extend_from_slice(conn_header.as_bytes());
    http_response.extend_from_slice(content_type_header.as_bytes());
    http_response.extend_from_slice(content_length_header.as_bytes());
    http_response.extend_from_slice(&response_bytes);

    stream
        .write_all(&http_response)
        .await
        .map_err(|e| format!("HTTP response write error: {e}"))?;
    stream
        .flush()
        .await
        .map_err(|e| format!("HTTP response flush error: {e}"))?;

    Ok(!connection_close)
}
