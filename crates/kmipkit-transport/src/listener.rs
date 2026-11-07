//! TLS 1.3 listeners and role-inversion transports for `KMIPKit`.
//!
//! Provides [`RawTlsListener`], [`HttpsListener`], and [`RawTlsRoleReversedStream`]
//! supporting OASIS KMIP 2.1 Server-to-Client operations (§6.2.1–§6.2.5) and role
//! inversion (§6.1.54, Profiles §5.3.1) with mutual TLS 1.3.

use std::fmt;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::CertificateRevocationListDer;
use rustls::server::WebPkiClientVerifier;
use rustls::sign::{CertifiedKey, SingleCertAndKey};
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};

use crate::config::{
    ClientIdentity, TimeoutPolicy, TransportConfigError, TrustSource, WireEncoding,
};
use crate::error::{RequestDeliveryState, TransportCauseCategory, TransportError};
use crate::response::TransportResponse;

const DEFAULT_MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_TARGET_PATH: &str = "/kmip";

/// Validated configuration for a TLS 1.3 listener.
#[derive(Clone)]
pub struct ListenerTransportConfig {
    bind_host: String,
    bind_port: u16,
    identity: Arc<CertifiedKey>,
    roots: Arc<RootCertStore>,
    revocation_lists: Vec<CertificateRevocationListDer<'static>>,
    wire_encoding: Option<WireEncoding>,
    target_path: String,
    max_request_bytes: usize,
    timeout_policy: TimeoutPolicy,
    server_config: Arc<ServerConfig>,
}

impl fmt::Debug for ListenerTransportConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ListenerTransportConfig")
            .field("bind_host", &self.bind_host)
            .field("bind_port", &self.bind_port)
            .field("wire_encoding", &self.wire_encoding)
            .field("target_path", &self.target_path)
            .field("max_request_bytes", &self.max_request_bytes)
            .finish_non_exhaustive()
    }
}

impl ListenerTransportConfig {
    /// Starts constructing a listener configuration.
    #[must_use]
    pub fn builder(bind_host: impl Into<String>, bind_port: u16) -> ListenerTransportConfigBuilder {
        ListenerTransportConfigBuilder::new(bind_host, bind_port)
    }

    /// Returns the configured bind host.
    #[must_use]
    pub fn bind_host(&self) -> &str {
        &self.bind_host
    }

    /// Returns the configured bind port.
    #[must_use]
    pub fn bind_port(&self) -> u16 {
        self.bind_port
    }

    /// Returns the maximum allowed request size in bytes.
    #[must_use]
    pub fn max_request_bytes(&self) -> usize {
        self.max_request_bytes
    }

    /// Returns the configured timeout policy.
    #[must_use]
    pub fn timeout_policy(&self) -> &TimeoutPolicy {
        &self.timeout_policy
    }

    /// Returns the configured wire encoding, if any.
    #[must_use]
    pub fn wire_encoding(&self) -> Option<WireEncoding> {
        self.wire_encoding
    }

    /// Returns the configured HTTPS target path.
    #[must_use]
    pub fn target_path(&self) -> &str {
        &self.target_path
    }

    /// Returns the compiled rustls server configuration.
    pub(crate) fn server_config(&self) -> &Arc<ServerConfig> {
        &self.server_config
    }

    /// Returns the certified key presented by the listener.
    #[must_use]
    pub fn identity(&self) -> &Arc<CertifiedKey> {
        &self.identity
    }

    /// Returns the root certificate store used to verify peers.
    #[must_use]
    pub fn roots(&self) -> &Arc<RootCertStore> {
        &self.roots
    }

    /// Returns the revocation lists used to verify peers.
    #[must_use]
    pub fn revocation_lists(&self) -> &[CertificateRevocationListDer<'static>] {
        &self.revocation_lists
    }
}

/// Builder for [`ListenerTransportConfig`].
pub struct ListenerTransportConfigBuilder {
    bind_host: String,
    bind_port: u16,
    identity: Option<ClientIdentity>,
    trust_source: Option<TrustSource>,
    revocation_lists: Vec<crate::config::RevocationListInput>,
    wire_encoding: Option<WireEncoding>,
    target_path: Option<String>,
    max_request_bytes: usize,
    timeout_policy: TimeoutPolicy,
}

impl ListenerTransportConfigBuilder {
    /// Creates a builder with the selected bind host and port.
    #[must_use]
    pub fn new(bind_host: impl Into<String>, bind_port: u16) -> Self {
        Self {
            bind_host: bind_host.into(),
            bind_port,
            identity: None,
            trust_source: None,
            revocation_lists: Vec::new(),
            wire_encoding: None,
            target_path: None,
            max_request_bytes: DEFAULT_MAX_REQUEST_BYTES,
            timeout_policy: TimeoutPolicy::default(),
        }
    }

    /// Sets the certificate chain and private key presented by the listener.
    #[must_use]
    pub fn client_identity(mut self, identity: ClientIdentity) -> Self {
        self.identity = Some(identity);
        self
    }

    /// Sets the certificate chain and private key presented by the listener.
    #[must_use]
    pub fn listener_identity(mut self, identity: ClientIdentity) -> Self {
        self.identity = Some(identity);
        self
    }

    /// Sets the explicit trust roots for validating connecting KMIP peers.
    #[must_use]
    pub fn trust_source(mut self, trust_source: TrustSource) -> Self {
        self.trust_source = Some(trust_source);
        self
    }

    /// Sets optional certificate revocation lists for peer verification.
    #[must_use]
    pub fn revocation_lists(mut self, crls: Vec<crate::config::RevocationListInput>) -> Self {
        self.revocation_lists = crls;
        self
    }

    /// Sets the wire encoding expected by the listener.
    #[must_use]
    pub fn wire_encoding(mut self, encoding: WireEncoding) -> Self {
        self.wire_encoding = Some(encoding);
        self
    }

    /// Sets the HTTPS target path (defaults to `/kmip`).
    #[must_use]
    pub fn target_path(mut self, path: impl Into<String>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    /// Sets the maximum allowable request frame size in bytes.
    #[must_use]
    pub fn max_request_bytes(mut self, limit: usize) -> Self {
        self.max_request_bytes = limit;
        self
    }

    /// Sets the timeout policy.
    #[must_use]
    pub fn timeout_policy(mut self, policy: TimeoutPolicy) -> Self {
        self.timeout_policy = policy;
        self
    }

    /// Builds and validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns [`TransportConfigError`] if the identity, trust roots, CRLs, or
    /// endpoint parameters are missing or invalid.
    pub fn build(self) -> Result<ListenerTransportConfig, TransportConfigError> {
        let host = self.bind_host.trim();
        if host.is_empty() || host.chars().any(char::is_whitespace) {
            return Err(TransportConfigError::InvalidEndpoint);
        }

        let identity_input = self
            .identity
            .ok_or(TransportConfigError::InvalidCredential)?;
        let trust_input = self
            .trust_source
            .ok_or(TransportConfigError::InvalidTrust)?;

        let target_path = self
            .target_path
            .unwrap_or_else(|| DEFAULT_TARGET_PATH.to_owned());
        if !target_path.starts_with('/') {
            return Err(TransportConfigError::InvalidEndpoint);
        }

        if self.max_request_bytes == 0 {
            return Err(TransportConfigError::InvalidEndpoint);
        }

        let provider = crate::tls::aws_lc_crypto_provider();

        // Load root certificates
        let root_store = match trust_input {
            TrustSource::CertificateAuthorities(certs) => {
                let mut store = RootCertStore::empty();
                for cert_input in certs {
                    let parsed_certs = cert_input.parse()?;
                    for cert in parsed_certs {
                        store
                            .add(cert)
                            .map_err(|_| TransportConfigError::InvalidTrust)?;
                    }
                }
                store
            }
            TrustSource::Platform => {
                let mut store = RootCertStore::empty();
                let native_certs = rustls_native_certs::load_native_certs();
                for cert in native_certs.certs {
                    let _ = store.add(cert);
                }
                store
            }
            #[allow(unreachable_patterns)]
            _ => return Err(TransportConfigError::InvalidTrust),
        };

        // Parse CRLs
        let mut crl_ders = Vec::new();
        for crl_input in self.revocation_lists {
            crl_ders.extend(crl_input.parse()?);
        }

        // Parse certified key
        let certified_key = identity_input.parse(&provider)?;
        let identity = Arc::new(certified_key);
        let roots = Arc::new(root_store);

        // Build WebPki client verifier (enforcing mTLS!)
        let has_crls = !crl_ders.is_empty();
        let verifier =
            WebPkiClientVerifier::builder_with_provider(Arc::clone(&roots), Arc::clone(&provider))
                .with_crls(crl_ders.clone())
                .build()
                .map_err(|_| {
                    if has_crls {
                        TransportConfigError::InvalidRevocationList
                    } else {
                        TransportConfigError::InvalidTrust
                    }
                })?;

        // Build TLS 1.3 ServerConfig
        let cert_resolver = Arc::new(SingleCertAndKey::from(Arc::clone(&identity)));
        let mut server_config = ServerConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(|_| TransportConfigError::InvalidTlsConfiguration)?
            .with_client_cert_verifier(verifier)
            .with_cert_resolver(cert_resolver);

        server_config.key_log = Arc::new(rustls::NoKeyLog);

        Ok(ListenerTransportConfig {
            bind_host: host.to_owned(),
            bind_port: self.bind_port,
            identity,
            roots,
            revocation_lists: crl_ders,
            wire_encoding: self.wire_encoding,
            target_path,
            max_request_bytes: self.max_request_bytes,
            timeout_policy: self.timeout_policy,
            server_config: Arc::new(server_config),
        })
    }
}

/// A synchronous raw-TLS 1.3 listener for receiving incoming KMIP Server-to-Client requests.
pub struct RawTlsListener {
    listener: TcpListener,
    config: ListenerTransportConfig,
}

fn accept_with_timeout(
    listener: &TcpListener,
    timeout: Option<Duration>,
) -> Result<(TcpStream, SocketAddr), TransportError> {
    if let Some(t) = timeout {
        listener.set_nonblocking(true).map_err(|e| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                e,
            )
        })?;
        let start = std::time::Instant::now();
        loop {
            match listener.accept() {
                Ok((sock, addr)) => {
                    let _ = listener.set_nonblocking(false);
                    let _ = sock.set_nonblocking(false);
                    return Ok((sock, addr));
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if start.elapsed() >= t {
                        let _ = listener.set_nonblocking(false);
                        return Err(TransportError::new(
                            RequestDeliveryState::NotSent,
                            TransportCauseCategory::Timeout,
                            io::Error::new(io::ErrorKind::TimedOut, "accept timed out"),
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => {
                    let _ = listener.set_nonblocking(false);
                    return Err(TransportError::new(
                        RequestDeliveryState::NotSent,
                        TransportCauseCategory::Other,
                        e,
                    ));
                }
            }
        }
    }

    let (sock, addr) = listener.accept().map_err(|err| {
        TransportError::new(
            RequestDeliveryState::NotSent,
            TransportCauseCategory::Other,
            err,
        )
    })?;
    let _ = sock.set_nonblocking(false);
    Ok((sock, addr))
}

impl RawTlsListener {
    /// Binds a raw-TLS listener to the address configured in `config`.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if socket binding fails.
    pub fn bind(config: ListenerTransportConfig) -> Result<Self, TransportError> {
        let addr_str = format!("{}:{}", config.bind_host(), config.bind_port());
        let listener = TcpListener::bind(&addr_str).map_err(|err| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                err,
            )
        })?;
        Ok(Self { listener, config })
    }

    /// Returns the local socket address bound by the listener.
    ///
    /// # Errors
    ///
    /// Returns an IO error if querying the socket fails.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Returns the bound TCP port.
    ///
    /// # Errors
    ///
    /// Returns an IO error if querying the socket fails.
    pub fn port(&self) -> io::Result<u16> {
        self.local_addr().map(|addr| addr.port())
    }

    /// Accepts an incoming TLS 1.3 connection.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if accepting the connection or TLS handshake fails.
    pub fn accept(
        &mut self,
        timeout: Option<Duration>,
    ) -> Result<RawTlsServerConnection, TransportError> {
        let (socket, peer_addr) = accept_with_timeout(&self.listener, timeout)?;

        let read_timeout = match self.config.timeout_policy().read() {
            crate::config::TimeoutLimit::Bounded(d) => Some(d),
            crate::config::TimeoutLimit::Unbounded => None,
        };
        let write_timeout = match self.config.timeout_policy().write() {
            crate::config::TimeoutLimit::Bounded(d) => Some(d),
            crate::config::TimeoutLimit::Unbounded => None,
        };
        let effective_read_timeout = timeout.or(read_timeout);
        let effective_write_timeout = timeout.or(write_timeout);
        let _ = socket.set_read_timeout(effective_read_timeout);
        let _ = socket.set_write_timeout(effective_write_timeout);

        let server_conn =
            ServerConnection::new(Arc::clone(self.config.server_config())).map_err(|err| {
                TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    io::Error::other(err.to_string()),
                )
            })?;

        let mut stream = StreamOwned::new(server_conn, socket);
        while stream.conn.is_handshaking() {
            let (r, w) = stream.conn.complete_io(&mut stream.sock).map_err(|err| {
                TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    err,
                )
            })?;
            if r == 0 && w == 0 {
                return Err(TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "TLS handshake closed prematurely",
                    ),
                ));
            }
        }
        while stream.conn.wants_write() {
            let (_, w) = stream.conn.complete_io(&mut stream.sock).map_err(|err| {
                TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    err,
                )
            })?;
            if w == 0 {
                break;
            }
        }

        Ok(RawTlsServerConnection {
            stream: Some(stream),
            peer_addr,
            max_request_bytes: self.config.max_request_bytes(),
        })
    }
}

/// An established raw-TLS server connection.
pub struct RawTlsServerConnection {
    stream: Option<StreamOwned<ServerConnection, TcpStream>>,
    peer_addr: SocketAddr,
    max_request_bytes: usize,
}

impl RawTlsServerConnection {
    /// Returns the remote peer address.
    #[must_use]
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    /// Reads one incoming KMIP request frame from the TLS connection.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if reading fails or frame exceeds `max_bytes`.
    pub fn recv_request(
        &mut self,
        max_bytes: usize,
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("connection is closed"),
            )
        })?;

        if let Some(t) = timeout {
            let _ = stream.sock.set_read_timeout(Some(t));
        }

        let mut header = [0_u8; 8];
        stream.read_exact(&mut header).map_err(|err| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                err,
            )
        })?;

        let payload_len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
        let padding = (8 - (payload_len % 8)) % 8;
        let total_frame_len = 8 + payload_len + padding;

        let effective_limit = max_bytes.min(self.max_request_bytes);
        if total_frame_len > effective_limit {
            return Err(TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("incoming frame exceeds max_request_bytes"),
            ));
        }

        let mut body = vec![0_u8; payload_len + padding];
        stream.read_exact(&mut body).map_err(|err| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                err,
            )
        })?;

        let mut full_frame = Vec::with_capacity(total_frame_len);
        full_frame.extend_from_slice(&header);
        full_frame.extend_from_slice(&body);

        Ok(TransportResponse::new(full_frame))
    }

    /// Sends a KMIP response frame to the server over TLS.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if writing or flushing fails.
    pub fn send_response(
        &mut self,
        response_bytes: &[u8],
        timeout: Option<Duration>,
    ) -> Result<(), TransportError> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("connection is closed"),
            )
        })?;

        if let Some(t) = timeout {
            let _ = stream.sock.set_write_timeout(Some(t));
        }

        stream.write_all(response_bytes).map_err(|err| {
            TransportError::new(
                RequestDeliveryState::PossiblySent,
                TransportCauseCategory::Other,
                err,
            )
        })?;
        stream.flush().map_err(|err| {
            TransportError::new(
                RequestDeliveryState::PossiblySent,
                TransportCauseCategory::Other,
                err,
            )
        })?;

        Ok(())
    }

    /// Converts this connection into a role-reversed stream.
    ///
    /// # Panics
    ///
    /// Panics if the inner TLS stream has already been taken or closed.
    #[must_use]
    pub fn into_role_reversed(mut self) -> RawTlsRoleReversedStream {
        let stream = self
            .stream
            .take()
            .expect("stream is present before into_role_reversed");
        RawTlsRoleReversedStream {
            channel: Box::new(ServerRoleReversedChannel { stream }),
        }
    }

    /// Gracefully closes this connection.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if closing the socket fails.
    pub fn close(&mut self) -> Result<(), TransportError> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.flush();
            let _ = stream.sock.shutdown(std::net::Shutdown::Both);
        }
        Ok(())
    }
}

/// A synchronous HTTPS / HTTP 1.1 listener for receiving incoming KMIP Server-to-Client requests.
///
/// Implements KMIP Profiles v2.1 §5.3.1 (HTTPS Client behaving as HTTPS Server).
pub struct HttpsListener {
    listener: TcpListener,
    config: ListenerTransportConfig,
}

impl HttpsListener {
    /// Binds an HTTPS listener to the address configured in `config`.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if socket binding fails.
    pub fn bind(config: ListenerTransportConfig) -> Result<Self, TransportError> {
        let addr_str = format!("{}:{}", config.bind_host(), config.bind_port());
        let listener = TcpListener::bind(&addr_str).map_err(|err| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                err,
            )
        })?;
        Ok(Self { listener, config })
    }

    /// Returns the local socket address bound by the listener.
    ///
    /// # Errors
    ///
    /// Returns an IO error if querying the socket fails.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Returns the bound TCP port.
    ///
    /// # Errors
    ///
    /// Returns an IO error if querying the socket fails.
    pub fn port(&self) -> io::Result<u16> {
        self.local_addr().map(|addr| addr.port())
    }

    /// Accepts an incoming HTTPS connection.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if accepting the connection or TLS handshake fails.
    pub fn accept(
        &mut self,
        timeout: Option<Duration>,
    ) -> Result<HttpsServerConnection, TransportError> {
        let (socket, peer_addr) = accept_with_timeout(&self.listener, timeout)?;

        let read_timeout = match self.config.timeout_policy().read() {
            crate::config::TimeoutLimit::Bounded(d) => Some(d),
            crate::config::TimeoutLimit::Unbounded => None,
        };
        let write_timeout = match self.config.timeout_policy().write() {
            crate::config::TimeoutLimit::Bounded(d) => Some(d),
            crate::config::TimeoutLimit::Unbounded => None,
        };
        let effective_read_timeout = timeout.or(read_timeout);
        let effective_write_timeout = timeout.or(write_timeout);
        let _ = socket.set_read_timeout(effective_read_timeout);
        let _ = socket.set_write_timeout(effective_write_timeout);

        let server_conn =
            ServerConnection::new(Arc::clone(self.config.server_config())).map_err(|err| {
                TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    io::Error::other(err.to_string()),
                )
            })?;

        let mut stream = StreamOwned::new(server_conn, socket);
        while stream.conn.is_handshaking() {
            let (r, w) = stream.conn.complete_io(&mut stream.sock).map_err(|err| {
                TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    err,
                )
            })?;
            if r == 0 && w == 0 {
                return Err(TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "TLS handshake closed prematurely",
                    ),
                ));
            }
        }
        while stream.conn.wants_write() {
            let (_, w) = stream.conn.complete_io(&mut stream.sock).map_err(|err| {
                TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    err,
                )
            })?;
            if w == 0 {
                break;
            }
        }

        Ok(HttpsServerConnection {
            stream: Some(stream),
            peer_addr,
            target_path: self.config.target_path().to_owned(),
            max_request_bytes: self.config.max_request_bytes(),
        })
    }
}

/// An established HTTPS server connection.
pub struct HttpsServerConnection {
    stream: Option<StreamOwned<ServerConnection, TcpStream>>,
    peer_addr: SocketAddr,
    target_path: String,
    max_request_bytes: usize,
}

impl HttpsServerConnection {
    /// Returns the remote peer address.
    #[must_use]
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    /// Reads one incoming HTTP/1.1 POST request from the TLS stream.
    ///
    /// Returns the decoded body bytes and the negotiated [`WireEncoding`].
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if HTTP parsing fails, method is not POST,
    /// path does not match, or payload exceeds `max_bytes`.
    pub fn recv_request(
        &mut self,
        max_bytes: usize,
        timeout: Option<Duration>,
    ) -> Result<(TransportResponse, WireEncoding), TransportError> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("connection is closed"),
            )
        })?;

        if let Some(t) = timeout {
            let _ = stream.sock.set_read_timeout(Some(t));
        }

        // Read headers until \r\n\r\n
        let mut header_buf = Vec::with_capacity(1024);
        let mut matched_end = false;
        let mut byte = [0_u8; 1];
        while !matched_end && header_buf.len() < 16 * 1024 {
            stream.read_exact(&mut byte).map_err(|err| {
                TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    err,
                )
            })?;
            header_buf.push(byte[0]);
            if header_buf.ends_with(b"\r\n\r\n") {
                matched_end = true;
            }
        }

        if !matched_end {
            return Err(TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("HTTP header too large or truncated"),
            ));
        }

        let header_str = String::from_utf8_lossy(&header_buf);
        let mut lines = header_str.lines();
        let request_line = lines.next().ok_or_else(|| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("missing HTTP request line"),
            )
        })?;

        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("");

        if !method.eq_ignore_ascii_case("POST") {
            return Err(TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("HTTP method must be POST"),
            ));
        }

        if path != self.target_path {
            return Err(TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("HTTP path mismatch"),
            ));
        }

        let mut content_length = 0;
        let mut content_type = None;

        for line in lines {
            let line_lower = line.to_ascii_lowercase();
            if let Some(val) = line_lower.strip_prefix("content-length:") {
                content_length = val.trim().parse::<usize>().unwrap_or(0);
            } else if let Some(val) = line_lower.strip_prefix("content-type:") {
                content_type = Some(val.trim().to_string());
            }
        }

        let effective_limit = max_bytes.min(self.max_request_bytes);
        if content_length > effective_limit {
            return Err(TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("HTTP body exceeds max_request_bytes"),
            ));
        }

        let mut body = vec![0_u8; content_length];
        if content_length > 0 {
            stream.read_exact(&mut body).map_err(|err| {
                TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    err,
                )
            })?;
        }

        let encoding = match content_type.as_deref() {
            Some(ct) if ct.contains("application/json") => WireEncoding::Json,
            Some(ct) if ct.contains("text/xml") || ct.contains("application/xml") => {
                WireEncoding::Xml
            }
            _ => WireEncoding::Ttlv,
        };

        Ok((TransportResponse::new(body), encoding))
    }

    /// Sends an HTTP/1.1 200 OK response with KMIP payload.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if writing fails.
    pub fn send_response(
        &mut self,
        response_bytes: &[u8],
        encoding: WireEncoding,
        timeout: Option<Duration>,
    ) -> Result<(), TransportError> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("connection is closed"),
            )
        })?;

        if let Some(t) = timeout {
            let _ = stream.sock.set_write_timeout(Some(t));
        }

        let content_type = match encoding {
            WireEncoding::Json => "application/json",
            WireEncoding::Xml => "text/xml",
            _ => "application/octet-stream",
        };

        let response_headers = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n",
            response_bytes.len()
        );

        stream
            .write_all(response_headers.as_bytes())
            .map_err(|err| {
                TransportError::new(
                    RequestDeliveryState::PossiblySent,
                    TransportCauseCategory::Other,
                    err,
                )
            })?;
        stream.write_all(response_bytes).map_err(|err| {
            TransportError::new(
                RequestDeliveryState::PossiblySent,
                TransportCauseCategory::Other,
                err,
            )
        })?;
        stream.flush().map_err(|err| {
            TransportError::new(
                RequestDeliveryState::PossiblySent,
                TransportCauseCategory::Other,
                err,
            )
        })?;

        Ok(())
    }

    /// Gracefully closes this connection.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if closing the socket fails.
    pub fn close(&mut self) -> Result<(), TransportError> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.flush();
            let _ = stream.sock.shutdown(std::net::Shutdown::Both);
        }
        Ok(())
    }
}

trait RoleReversedChannel: Send {
    fn recv(
        &mut self,
        max_bytes: usize,
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError>;
    fn send(&mut self, bytes: &[u8], timeout: Option<Duration>) -> Result<(), TransportError>;
    fn close(&mut self) -> Result<(), TransportError>;
}

struct ServerRoleReversedChannel {
    stream: StreamOwned<ServerConnection, TcpStream>,
}

impl RoleReversedChannel for ServerRoleReversedChannel {
    fn recv(
        &mut self,
        max_bytes: usize,
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        if let Some(t) = timeout {
            let _ = self.stream.sock.set_read_timeout(Some(t));
        }

        let mut header = [0_u8; 8];
        self.stream.read_exact(&mut header).map_err(|err| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                err,
            )
        })?;

        let payload_len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
        let padding = (8 - (payload_len % 8)) % 8;
        let total_frame_len = 8 + payload_len + padding;

        if total_frame_len > max_bytes {
            return Err(TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("incoming frame exceeds max_bytes"),
            ));
        }

        let mut body = vec![0_u8; payload_len + padding];
        self.stream.read_exact(&mut body).map_err(|err| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                err,
            )
        })?;

        let mut full_frame = Vec::with_capacity(total_frame_len);
        full_frame.extend_from_slice(&header);
        full_frame.extend_from_slice(&body);

        Ok(TransportResponse::new(full_frame))
    }

    fn send(&mut self, bytes: &[u8], timeout: Option<Duration>) -> Result<(), TransportError> {
        if let Some(t) = timeout {
            let _ = self.stream.sock.set_write_timeout(Some(t));
        }
        self.stream.write_all(bytes).map_err(|err| {
            TransportError::new(
                RequestDeliveryState::PossiblySent,
                TransportCauseCategory::Other,
                err,
            )
        })?;
        self.stream.flush().map_err(|err| {
            TransportError::new(
                RequestDeliveryState::PossiblySent,
                TransportCauseCategory::Other,
                err,
            )
        })?;
        Ok(())
    }

    fn close(&mut self) -> Result<(), TransportError> {
        let _ = self.stream.flush();
        let _ = self.stream.sock.shutdown(std::net::Shutdown::Both);
        Ok(())
    }
}

/// An established TLS connection operating in role-reversed mode.
///
/// In this mode, the client listens for incoming Server-to-Client requests
/// over the established TLS channel and returns responses.
pub struct RawTlsRoleReversedStream {
    channel: Box<dyn RoleReversedChannel>,
}

impl RawTlsRoleReversedStream {
    /// Reads one incoming server request from the stream.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if reading fails.
    pub fn recv_server_request(
        &mut self,
        max_bytes: usize,
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.channel.recv(max_bytes, timeout)
    }

    /// Sends a client response to the server.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if writing fails.
    pub fn send_client_response(
        &mut self,
        response_bytes: &[u8],
        timeout: Option<Duration>,
    ) -> Result<(), TransportError> {
        self.channel.send(response_bytes, timeout)
    }

    /// Closes the role-reversed stream.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if closing fails.
    pub fn close(&mut self) -> Result<(), TransportError> {
        self.channel.close()
    }
}
