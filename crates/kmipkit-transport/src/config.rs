//! Validated, immutable configuration for production KMIP transports.

use std::error::Error;
use std::fmt;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use hyper::Uri;
use rustls::RootCertStore;
use rustls::crypto::CryptoProvider;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, CertificateRevocationListDer, PrivateKeyDer, ServerName};
use rustls::sign::CertifiedKey;
use zeroize::Zeroizing;

use crate::secret::{SecretBuffer, SecretPrivateKeyDer};

const DEFAULT_MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;

/// An endpoint description. Validation occurs when it is placed in a config builder.
pub struct Endpoint(EndpointInput);

enum EndpointInput {
    RawTls { host: String, port: u16 },
    Https { uri: String },
}

impl Endpoint {
    /// Creates a raw-TLS endpoint from a host and TCP port.
    #[must_use]
    pub fn raw_tls(host: impl Into<String>, port: u16) -> Self {
        Self(EndpointInput::RawTls {
            host: host.into(),
            port,
        })
    }

    /// Creates an HTTPS endpoint from an absolute URI string.
    #[must_use]
    pub fn https(uri: impl Into<String>) -> Self {
        Self(EndpointInput::Https { uri: uri.into() })
    }

    fn validate(&self) -> Result<(), TransportConfigError> {
        match &self.0 {
            EndpointInput::RawTls { host, port }
                if host.trim().is_empty()
                    || host.chars().any(char::is_whitespace)
                    || *port == 0 =>
            {
                Err(TransportConfigError::InvalidEndpoint)
            }
            EndpointInput::RawTls { .. } => Ok(()),
            EndpointInput::Https { uri } => validate_https_endpoint(uri),
        }
    }

    fn is_https(&self) -> bool {
        matches!(self.0, EndpointInput::Https { .. })
    }

    fn host(&self) -> Result<String, TransportConfigError> {
        match &self.0 {
            EndpointInput::RawTls { host, .. } => Ok(host.clone()),
            EndpointInput::Https { uri } => {
                let parsed = Uri::try_from(uri.as_str())
                    .map_err(|_| TransportConfigError::InvalidEndpoint)?;
                parsed
                    .authority()
                    .map(|authority| authority.host().to_owned())
                    .ok_or(TransportConfigError::InvalidEndpoint)
            }
        }
    }

    pub(crate) fn raw_tls_address(&self) -> Result<(String, u16), TransportConfigError> {
        match &self.0 {
            EndpointInput::RawTls { host, port } => {
                let host = host
                    .strip_prefix('[')
                    .and_then(|value| value.strip_suffix(']'))
                    .unwrap_or(host);
                Ok((host.to_owned(), *port))
            }
            EndpointInput::Https { .. } => Err(TransportConfigError::InvalidEndpoint),
        }
    }

    pub(crate) fn https_connection_details(
        &self,
    ) -> Result<(String, u16, String), TransportConfigError> {
        let Self(EndpointInput::Https { uri }) = self else {
            return Err(TransportConfigError::InvalidEndpoint);
        };
        let parsed =
            Uri::try_from(uri.as_str()).map_err(|_| TransportConfigError::InvalidEndpoint)?;
        let authority = parsed
            .authority()
            .ok_or(TransportConfigError::InvalidEndpoint)?;
        let authority_host = authority.host();
        let host = authority_host
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
            .unwrap_or(authority_host);
        Ok((
            host.to_owned(),
            authority.port_u16().unwrap_or(443),
            authority.as_str().to_owned(),
        ))
    }

    fn server_name(
        &self,
        override_name: Option<String>,
    ) -> Result<(ServerName<'static>, bool), TransportConfigError> {
        let host = self.host()?;
        let host = host
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
            .unwrap_or(&host);
        let is_ip = host.parse::<IpAddr>().is_ok();
        if override_name.is_some() && !is_ip {
            return Err(TransportConfigError::InvalidServerName);
        }
        let name = override_name.unwrap_or_else(|| host.to_owned());
        let server_name =
            ServerName::try_from(name).map_err(|_| TransportConfigError::InvalidServerName)?;
        Ok((server_name, !is_ip))
    }
}

impl fmt::Debug for Endpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.0 {
            EndpointInput::RawTls { .. } => "RawTls",
            EndpointInput::Https { .. } => "Https",
        };
        formatter
            .debug_struct("Endpoint")
            .field("kind", &kind)
            .field("value", &"[REDACTED]")
            .finish()
    }
}

fn validate_https_endpoint(value: &str) -> Result<(), TransportConfigError> {
    if value.contains('#') {
        return Err(TransportConfigError::InvalidEndpoint);
    }
    let uri = Uri::try_from(value).map_err(|_| TransportConfigError::InvalidEndpoint)?;
    if !uri
        .scheme_str()
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https"))
    {
        return Err(TransportConfigError::InvalidEndpoint);
    }
    let authority = uri
        .authority()
        .ok_or(TransportConfigError::InvalidEndpoint)?;
    if authority.host().is_empty() || authority.as_str().contains('@') {
        return Err(TransportConfigError::InvalidEndpoint);
    }
    let suffix = authority
        .as_str()
        .strip_prefix(authority.host())
        .ok_or(TransportConfigError::InvalidEndpoint)?;
    if !suffix.is_empty() {
        let Some(port_text) = suffix.strip_prefix(':') else {
            return Err(TransportConfigError::InvalidEndpoint);
        };
        if port_text.is_empty()
            || !port_text
                .chars()
                .all(|character| character.is_ascii_digit())
        {
            return Err(TransportConfigError::InvalidEndpoint);
        }
        if !matches!(port_text.parse::<u16>(), Ok(port) if port > 0) {
            return Err(TransportConfigError::InvalidEndpoint);
        }
    }
    Ok(())
}

/// Caller-provided certificate chain with an explicit encoding.
pub struct CertificateInput {
    bytes: CertificateBytes,
}

enum CertificateBytes {
    Pem(Zeroizing<Vec<u8>>),
    Der(Vec<Zeroizing<Vec<u8>>>),
}

impl CertificateInput {
    /// Creates a PEM-encoded certificate chain.
    #[must_use]
    pub fn from_pem(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: CertificateBytes::Pem(Zeroizing::new(bytes.into())),
        }
    }

    /// Reads one PEM-encoded certificate chain from the selected path.
    ///
    /// The path is read once during construction and is not retained. File
    /// errors use the same redacted category as malformed certificate input.
    ///
    /// # Errors
    ///
    /// Returns [`TransportConfigError::InvalidCredential`] if the file cannot
    /// be read.
    pub fn from_pem_file(path: impl AsRef<Path>) -> Result<Self, TransportConfigError> {
        let bytes = std::fs::read(path).map_err(|_| TransportConfigError::InvalidCredential)?;
        Ok(Self::from_pem(bytes))
    }

    /// Creates a DER-encoded certificate chain in leaf-to-root order.
    #[must_use]
    pub fn from_der(certificates: Vec<Vec<u8>>) -> Self {
        Self {
            bytes: CertificateBytes::Der(certificates.into_iter().map(Zeroizing::new).collect()),
        }
    }

    /// Reads one DER-encoded certificate from the selected path.
    ///
    /// The path is read once during construction and is not retained. File
    /// errors use the same redacted category as malformed certificate input.
    ///
    /// # Errors
    ///
    /// Returns [`TransportConfigError::InvalidCredential`] if the file cannot
    /// be read.
    pub fn from_der_file(path: impl AsRef<Path>) -> Result<Self, TransportConfigError> {
        let bytes = std::fs::read(path).map_err(|_| TransportConfigError::InvalidCredential)?;
        Ok(Self::from_der(vec![bytes]))
    }

    fn parse(self) -> Result<Vec<CertificateDer<'static>>, TransportConfigError> {
        let certificates = match self.bytes {
            CertificateBytes::Pem(bytes) => CertificateDer::pem_slice_iter(bytes.as_slice())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| TransportConfigError::InvalidCredential)?,
            CertificateBytes::Der(certificates) => certificates
                .into_iter()
                .map(|bytes| CertificateDer::from(bytes.to_vec()))
                .collect(),
        };
        if certificates.is_empty()
            || certificates
                .iter()
                .any(|certificate| certificate.is_empty())
        {
            return Err(TransportConfigError::InvalidCredential);
        }
        Ok(certificates)
    }
}

impl fmt::Debug for CertificateInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CertificateInput")
            .field(
                "encoding",
                &match &self.bytes {
                    CertificateBytes::Pem(_) => "PEM",
                    CertificateBytes::Der(_) => "DER",
                },
            )
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

enum RevocationListBytes {
    Pem(Zeroizing<Vec<u8>>),
    Der(Vec<Zeroizing<Vec<u8>>>),
}

/// Caller-provided certificate revocation list with an explicit encoding.
pub struct RevocationListInput {
    bytes: RevocationListBytes,
}

impl RevocationListInput {
    /// Creates a PEM-encoded certificate revocation list.
    #[must_use]
    pub fn from_pem(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: RevocationListBytes::Pem(Zeroizing::new(bytes.into())),
        }
    }

    /// Creates one or more DER-encoded certificate revocation lists.
    #[must_use]
    pub fn from_der(revocation_lists: Vec<Vec<u8>>) -> Self {
        Self {
            bytes: RevocationListBytes::Der(
                revocation_lists.into_iter().map(Zeroizing::new).collect(),
            ),
        }
    }

    fn parse(self) -> Result<Vec<CertificateRevocationListDer<'static>>, TransportConfigError> {
        let lists = match self.bytes {
            RevocationListBytes::Pem(bytes) => {
                CertificateRevocationListDer::pem_slice_iter(bytes.as_slice())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| TransportConfigError::InvalidRevocationList)?
            }
            RevocationListBytes::Der(lists) => lists
                .into_iter()
                .map(|bytes| CertificateRevocationListDer::from(bytes.to_vec()))
                .collect(),
        };
        if lists.is_empty() || lists.iter().any(|list| list.is_empty()) {
            return Err(TransportConfigError::InvalidRevocationList);
        }
        Ok(lists)
    }
}

impl fmt::Debug for RevocationListInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RevocationListInput")
            .field(
                "encoding",
                &match &self.bytes {
                    RevocationListBytes::Pem(_) => "PEM",
                    RevocationListBytes::Der(_) => "DER",
                },
            )
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

enum PrivateKeyBytes {
    Pem(SecretBuffer),
    Der(SecretBuffer),
}

/// Caller-provided private key with an explicit encoding.
pub struct PrivateKeyInput {
    bytes: PrivateKeyBytes,
}

impl PrivateKeyInput {
    /// Creates a PEM-encoded unencrypted PKCS#8, PKCS#1, or SEC1 private key.
    #[must_use]
    pub fn from_pem(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: PrivateKeyBytes::Pem(SecretBuffer::new(bytes.into())),
        }
    }

    /// Reads a PEM-encoded unencrypted private key from the selected path.
    ///
    /// The path is read once during construction and is not retained. File
    /// errors use the same redacted category as malformed key input.
    ///
    /// # Errors
    ///
    /// Returns [`TransportConfigError::InvalidCredential`] if the file cannot
    /// be read.
    pub fn from_pem_file(path: impl AsRef<Path>) -> Result<Self, TransportConfigError> {
        let file =
            std::fs::File::open(path).map_err(|_| TransportConfigError::InvalidCredential)?;
        let bytes =
            SecretBuffer::read_from(file).map_err(|_| TransportConfigError::InvalidCredential)?;
        Ok(Self {
            bytes: PrivateKeyBytes::Pem(bytes),
        })
    }

    /// Creates a DER-encoded unencrypted private key.
    #[must_use]
    pub fn from_der(bytes: Vec<u8>) -> Self {
        Self {
            bytes: PrivateKeyBytes::Der(SecretBuffer::new(bytes)),
        }
    }

    /// Reads a DER-encoded unencrypted private key from the selected path.
    ///
    /// The path is read once during construction and is not retained. File
    /// errors use the same redacted category as malformed key input.
    ///
    /// # Errors
    ///
    /// Returns [`TransportConfigError::InvalidCredential`] if the file cannot
    /// be read.
    pub fn from_der_file(path: impl AsRef<Path>) -> Result<Self, TransportConfigError> {
        let file =
            std::fs::File::open(path).map_err(|_| TransportConfigError::InvalidCredential)?;
        let bytes =
            SecretBuffer::read_from(file).map_err(|_| TransportConfigError::InvalidCredential)?;
        Ok(Self {
            bytes: PrivateKeyBytes::Der(bytes),
        })
    }

    #[cfg(test)]
    #[allow(dead_code)] // Called only by the source-including secret-redaction contract target.
    pub(crate) fn from_pem_with_observer_for_test(
        bytes: Vec<u8>,
        observer: crate::secret::SecretBufferObserver,
    ) -> Self {
        Self {
            bytes: PrivateKeyBytes::Pem(SecretBuffer::with_observer(bytes, observer)),
        }
    }

    fn parse(self) -> Result<PrivateKeyDer<'static>, TransportConfigError> {
        let key = match self.bytes {
            PrivateKeyBytes::Pem(bytes) => {
                let mut keys = PrivateKeyDer::pem_slice_iter(bytes.as_slice())
                    .map(|key| key.map(SecretPrivateKeyDer::new))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| TransportConfigError::InvalidCredential)?;
                if keys.len() != 1 {
                    return Err(TransportConfigError::InvalidCredential);
                }
                keys.pop()
                    .and_then(SecretPrivateKeyDer::into_inner)
                    .ok_or(TransportConfigError::InvalidCredential)?
            }
            PrivateKeyBytes::Der(bytes) => {
                let key = PrivateKeyDer::try_from(bytes.as_slice())
                    .map_err(|_| TransportConfigError::InvalidCredential)?;
                SecretPrivateKeyDer::new(key.clone_key())
                    .into_inner()
                    .ok_or(TransportConfigError::InvalidCredential)?
            }
        };
        Ok(key)
    }
}

impl fmt::Debug for PrivateKeyInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PrivateKeyInput")
            .field(
                "encoding",
                &match &self.bytes {
                    PrivateKeyBytes::Pem(_) => "PEM",
                    PrivateKeyBytes::Der(_) => "DER",
                },
            )
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

/// The mTLS certificate chain and private key presented by this client.
pub struct ClientIdentity {
    certificate_chain: CertificateInput,
    private_key: PrivateKeyInput,
}

impl ClientIdentity {
    /// Creates a client identity from an explicitly encoded chain and key.
    #[must_use]
    pub fn new(certificate_chain: CertificateInput, private_key: PrivateKeyInput) -> Self {
        Self {
            certificate_chain,
            private_key,
        }
    }

    fn parse(self, provider: &CryptoProvider) -> Result<CertifiedKey, TransportConfigError> {
        let certificates = self.certificate_chain.parse()?;
        let private_key = self.private_key.parse()?;
        CertifiedKey::from_der(certificates, private_key, provider)
            .map_err(|_| TransportConfigError::InvalidCredential)
    }
}

impl fmt::Debug for ClientIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientIdentity")
            .field("certificate_chain", &self.certificate_chain)
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

/// Explicit trust roots selected by the caller.
#[non_exhaustive]
pub enum TrustSource {
    /// One or more caller-supplied root certificates.
    CertificateAuthorities(Vec<CertificateInput>),
    /// Native platform roots, including the `SSL_CERT_FILE` override supported by the platform loader.
    Platform,
}

impl TrustSource {
    /// Selects caller-supplied CA certificates.
    #[must_use]
    pub fn certificate_authorities(certificates: Vec<CertificateInput>) -> Self {
        Self::CertificateAuthorities(certificates)
    }

    /// Explicitly selects native platform roots.
    #[must_use]
    pub const fn platform() -> Self {
        Self::Platform
    }

    fn load(self) -> Result<RootCertStore, TransportConfigError> {
        let mut store = RootCertStore::empty();
        match self {
            Self::CertificateAuthorities(certificates) => {
                for input in certificates {
                    for certificate in input.parse()? {
                        store
                            .add(certificate)
                            .map_err(|_| TransportConfigError::InvalidTrust)?;
                    }
                }
            }
            Self::Platform => {
                let loaded = rustls_native_certs::load_native_certs();
                if !loaded.errors.is_empty() {
                    return Err(TransportConfigError::PlatformTrustUnavailable);
                }
                for certificate in loaded.certs {
                    store
                        .add(certificate)
                        .map_err(|_| TransportConfigError::InvalidTrust)?;
                }
            }
        }
        if store.is_empty() {
            return Err(TransportConfigError::InvalidTrust);
        }
        Ok(store)
    }
}

impl fmt::Debug for TrustSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CertificateAuthorities(certificates) => formatter
                .debug_struct("TrustSource::CertificateAuthorities")
                .field("count", &certificates.len())
                .finish(),
            Self::Platform => formatter.write_str("TrustSource::Platform"),
        }
    }
}

/// A timeout is either bounded, including zero for an immediate deadline, or unbounded.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeoutLimit {
    /// Use a duration; zero means the deadline is immediate.
    Bounded(Duration),
    /// Do not impose this phase deadline.
    Unbounded,
}

/// Immutable default connect, write, read, and total deadlines for one client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeoutPolicy {
    connect: TimeoutLimit,
    write: TimeoutLimit,
    read: TimeoutLimit,
    total: TimeoutLimit,
}

impl TimeoutPolicy {
    /// Returns the connect deadline.
    #[must_use]
    pub const fn connect(self) -> TimeoutLimit {
        self.connect
    }

    /// Returns the write inactivity deadline.
    #[must_use]
    pub const fn write(self) -> TimeoutLimit {
        self.write
    }

    /// Returns the read inactivity deadline.
    #[must_use]
    pub const fn read(self) -> TimeoutLimit {
        self.read
    }

    /// Returns the absolute total exchange deadline.
    #[must_use]
    pub const fn total(self) -> TimeoutLimit {
        self.total
    }

    /// Returns a policy with any supplied exchange overrides applied.
    #[must_use]
    pub fn with_overrides(self, options: &RequestOptions) -> Self {
        Self {
            connect: options.connect.unwrap_or(self.connect),
            write: options.write.unwrap_or(self.write),
            read: options.read.unwrap_or(self.read),
            total: options.total.unwrap_or(self.total),
        }
    }

    /// Sets the client's connect deadline.
    #[must_use]
    pub const fn with_connect(mut self, value: TimeoutLimit) -> Self {
        self.connect = value;
        self
    }

    /// Sets the client's write inactivity deadline.
    #[must_use]
    pub const fn with_write(mut self, value: TimeoutLimit) -> Self {
        self.write = value;
        self
    }

    /// Sets the client's read inactivity deadline.
    #[must_use]
    pub const fn with_read(mut self, value: TimeoutLimit) -> Self {
        self.read = value;
        self
    }

    /// Sets the client's total exchange deadline.
    #[must_use]
    pub const fn with_total(mut self, value: TimeoutLimit) -> Self {
        self.total = value;
        self
    }
}

impl Default for TimeoutPolicy {
    fn default() -> Self {
        Self {
            connect: TimeoutLimit::Bounded(Duration::from_secs(10)),
            write: TimeoutLimit::Bounded(Duration::from_secs(30)),
            read: TimeoutLimit::Bounded(Duration::from_secs(30)),
            total: TimeoutLimit::Bounded(Duration::from_secs(60)),
        }
    }
}

/// Optional per-exchange timeout overrides shared by typed and direct adapter calls.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RequestOptions {
    connect: Option<TimeoutLimit>,
    write: Option<TimeoutLimit>,
    read: Option<TimeoutLimit>,
    total: Option<TimeoutLimit>,
}

impl RequestOptions {
    /// Overrides the connect deadline for one exchange.
    #[must_use]
    pub const fn with_connect(mut self, value: TimeoutLimit) -> Self {
        self.connect = Some(value);
        self
    }

    /// Overrides the write inactivity deadline for one exchange.
    #[must_use]
    pub const fn with_write(mut self, value: TimeoutLimit) -> Self {
        self.write = Some(value);
        self
    }

    /// Overrides the read inactivity deadline for one exchange.
    #[must_use]
    pub const fn with_read(mut self, value: TimeoutLimit) -> Self {
        self.read = Some(value);
        self
    }

    /// Overrides the total exchange deadline for one exchange.
    #[must_use]
    pub const fn with_total(mut self, value: TimeoutLimit) -> Self {
        self.total = Some(value);
        self
    }
}

/// Validated configuration for one endpoint, trust source, client identity, and policy.
pub struct TransportConfig {
    endpoint: Endpoint,
    target_uri: Option<String>,
    timeouts: TimeoutPolicy,
    max_request_bytes: usize,
    trust: Arc<RootCertStore>,
    identity: Arc<CertifiedKey>,
    crypto_provider: Arc<CryptoProvider>,
    revocation_lists: Vec<CertificateRevocationListDer<'static>>,
    tls_server_name: ServerName<'static>,
    enable_sni: bool,
}

impl TransportConfig {
    /// Starts a builder for a raw-TLS or HTTPS endpoint.
    #[must_use]
    pub fn builder(endpoint: Endpoint) -> TransportConfigBuilder {
        TransportConfigBuilder {
            endpoint,
            target_uri: None,
            timeouts: TimeoutPolicy::default(),
            max_request_bytes: DEFAULT_MAX_REQUEST_BYTES,
            trust: None,
            identity: None,
            revocation_lists: Vec::new(),
            tls_server_name: None,
        }
    }

    /// Returns the configured endpoint description.
    #[must_use]
    pub const fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Returns the HTTPS origin-form target, or `None` for raw TLS.
    #[must_use]
    pub fn target_uri(&self) -> Option<&str> {
        self.target_uri.as_deref()
    }

    /// Returns the configured maximum request size.
    #[must_use]
    pub const fn max_request_bytes(&self) -> usize {
        self.max_request_bytes
    }

    /// Returns this client's default timeout policy.
    #[must_use]
    pub const fn timeouts(&self) -> TimeoutPolicy {
        self.timeouts
    }

    pub(crate) fn tls_roots(&self) -> &Arc<RootCertStore> {
        &self.trust
    }

    pub(crate) fn tls_identity(&self) -> &Arc<CertifiedKey> {
        &self.identity
    }

    pub(crate) fn tls_crypto_provider(&self) -> &Arc<CryptoProvider> {
        &self.crypto_provider
    }

    pub(crate) fn tls_revocation_lists(&self) -> &[CertificateRevocationListDer<'static>] {
        &self.revocation_lists
    }

    pub(crate) fn tls_server_name(&self) -> &ServerName<'static> {
        &self.tls_server_name
    }

    pub(crate) const fn tls_sni_enabled(&self) -> bool {
        self.enable_sni
    }
}

impl fmt::Debug for TransportConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportConfig")
            .field("endpoint", &self.endpoint)
            .field(
                "target_uri",
                &self.target_uri.as_ref().map(|_| "[REDACTED]"),
            )
            .field("timeouts", &self.timeouts)
            .field("max_request_bytes", &self.max_request_bytes)
            .field("trust", &"[REDACTED]")
            .field("identity", &"[REDACTED]")
            .field("crypto_provider", &"AWS-LC")
            .field("revocation_lists", &"[REDACTED]")
            .field("tls_server_name", &"[REDACTED]")
            .field("enable_sni", &self.enable_sni)
            .finish()
    }
}

/// Builder that validates all inputs before producing a production config.
pub struct TransportConfigBuilder {
    endpoint: Endpoint,
    target_uri: Option<String>,
    timeouts: TimeoutPolicy,
    max_request_bytes: usize,
    trust: Option<TrustSource>,
    identity: Option<ClientIdentity>,
    revocation_lists: Vec<RevocationListInput>,
    tls_server_name: Option<String>,
}

impl TransportConfigBuilder {
    /// Selects the required client certificate chain and private key.
    #[must_use]
    pub fn client_identity(mut self, identity: ClientIdentity) -> Self {
        self.identity = Some(identity);
        self
    }

    /// Selects caller CA certificates or platform trust explicitly.
    #[must_use]
    pub fn trust_source(mut self, trust: TrustSource) -> Self {
        self.trust = Some(trust);
        self
    }

    /// Sets caller-provided certificate revocation lists.
    #[must_use]
    pub fn revocation_lists(mut self, lists: Vec<RevocationListInput>) -> Self {
        self.revocation_lists = lists;
        self
    }

    /// Sets the certificate verification name for an IP endpoint.
    #[must_use]
    pub fn tls_server_name(mut self, server_name: impl Into<String>) -> Self {
        self.tls_server_name = Some(server_name.into());
        self
    }

    /// Sets the HTTPS origin-form request target.
    #[must_use]
    pub fn target_uri(mut self, target: impl Into<String>) -> Self {
        self.target_uri = Some(target.into());
        self
    }

    /// Sets per-client timeout defaults.
    #[must_use]
    pub const fn timeouts(mut self, timeouts: TimeoutPolicy) -> Self {
        self.timeouts = timeouts;
        self
    }

    /// Sets the positive direct-transport request limit.
    #[must_use]
    pub const fn max_request_bytes(mut self, max_request_bytes: usize) -> Self {
        self.max_request_bytes = max_request_bytes;
        self
    }

    /// Validates the endpoint, target, trust, identity, and positive limits.
    ///
    /// # Errors
    ///
    /// Returns a fixed, redacted configuration error when an input is invalid
    /// or a required trust source or client identity is missing.
    pub fn build(self) -> Result<TransportConfig, TransportConfigError> {
        self.endpoint.validate()?;
        if self.max_request_bytes == 0 {
            return Err(TransportConfigError::InvalidRequestLimit);
        }
        let target_uri = match (self.endpoint.is_https(), self.target_uri) {
            (true, Some(target)) => Some(validate_target(target)?),
            (true, None) => Some("/kmip".to_owned()),
            (false, Some(_)) => return Err(TransportConfigError::InvalidTarget),
            (false, None) => None,
        };
        let identity_input = self.identity.ok_or(TransportConfigError::MissingIdentity)?;
        let trust_source = self.trust.ok_or(TransportConfigError::MissingTrust)?;
        let (tls_server_name, enable_sni) = self.endpoint.server_name(self.tls_server_name)?;
        let crypto_provider = crate::tls::aws_lc_crypto_provider();
        let identity = Arc::new(identity_input.parse(&crypto_provider)?);
        let trust = Arc::new(trust_source.load()?);
        let mut revocation_lists = Vec::new();
        for list in self.revocation_lists {
            revocation_lists.extend(list.parse()?);
        }
        Ok(TransportConfig {
            endpoint: self.endpoint,
            target_uri,
            timeouts: self.timeouts,
            max_request_bytes: self.max_request_bytes,
            trust,
            identity,
            crypto_provider,
            revocation_lists,
            tls_server_name,
            enable_sni,
        })
    }
}

impl fmt::Debug for TransportConfigBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportConfigBuilder")
            .field("endpoint", &self.endpoint)
            .field(
                "target_uri",
                &self.target_uri.as_ref().map(|_| "[REDACTED]"),
            )
            .field("timeouts", &self.timeouts)
            .field("max_request_bytes", &self.max_request_bytes)
            .field("trust", &self.trust)
            .field("identity", &self.identity)
            .field("revocation_lists", &self.revocation_lists)
            .field(
                "tls_server_name",
                &self.tls_server_name.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

fn validate_target(value: String) -> Result<String, TransportConfigError> {
    if !value.starts_with('/')
        || value.starts_with("//")
        || value.contains('#')
        || value.chars().any(char::is_whitespace)
    {
        return Err(TransportConfigError::InvalidTarget);
    }
    let uri = Uri::try_from(value.as_str()).map_err(|_| TransportConfigError::InvalidTarget)?;
    if uri.scheme().is_some() || uri.authority().is_some() {
        return Err(TransportConfigError::InvalidTarget);
    }
    Ok(value)
}

/// A safe category for configuration validation failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TransportConfigError {
    /// The endpoint URI or raw-TLS host/port is invalid.
    InvalidEndpoint,
    /// The HTTPS request target is not origin-form or was set for raw TLS.
    InvalidTarget,
    /// The request limit is zero.
    InvalidRequestLimit,
    /// The configuration omitted its required mutual-TLS identity.
    MissingIdentity,
    /// The configuration omitted an explicit trust source.
    MissingTrust,
    /// Caller certificate or private-key material is malformed or mismatched.
    InvalidCredential,
    /// Caller trust certificates are malformed or empty.
    InvalidTrust,
    /// Caller-provided revocation-list material is malformed or empty.
    InvalidRevocationList,
    /// The TLS verification name is invalid or used with a DNS endpoint.
    InvalidServerName,
    /// The selected TLS security configuration could not be constructed.
    InvalidTlsConfiguration,
    /// Platform trust could not be loaded cleanly.
    PlatformTrustUnavailable,
}

impl fmt::Display for TransportConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidEndpoint => "invalid transport endpoint",
            Self::InvalidTarget => "invalid HTTPS request target",
            Self::InvalidRequestLimit => "request limit must be positive",
            Self::MissingIdentity => "client identity is required",
            Self::MissingTrust => "an explicit trust source is required",
            Self::InvalidCredential => "client credential input is invalid",
            Self::InvalidTrust => "trust certificates are invalid",
            Self::InvalidRevocationList => "certificate revocation list input is invalid",
            Self::InvalidServerName => "TLS verification name is invalid",
            Self::InvalidTlsConfiguration => "TLS configuration is invalid",
            Self::PlatformTrustUnavailable => "platform trust is unavailable",
        })
    }
}

impl Error for TransportConfigError {}
