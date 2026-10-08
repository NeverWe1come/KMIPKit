//! Private TLS 1.3 client construction and configuration-owned session storage.

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use rustls::ClientConfig as RustlsClientConfig;
use rustls::client::WantsClientCert;
use rustls::client::{
    ClientConfig, ClientSessionStore, Resumption, Tls12ClientSessionValue, Tls13ClientSessionValue,
    WebPkiServerVerifier,
};
use rustls::crypto::aws_lc_rs;
use rustls::pki_types::ServerName;
use rustls::sign::SingleCertAndKey;
use rustls::{ConfigBuilder, WantsVerifier};

#[cfg(test)]
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
#[cfg(test)]
use rustls::pki_types::{CertificateDer, UnixTime};
#[cfg(test)]
use rustls::{DigitallySignedStruct, Error, SignatureScheme};

use crate::config::TransportConfig;
use crate::config::TransportConfigError;

const MAX_TICKETS: usize = 16;
const MAX_LOCAL_TICKET_AGE: Duration = Duration::from_hours(1);

/// Selects the AWS-LC provider retained by one validated transport configuration.
pub(crate) fn aws_lc_crypto_provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(aws_lc_rs::default_provider())
}

/// Monotonic elapsed time used to age cached TLS tickets.
pub(crate) trait MonotonicClock: Send + Sync {
    /// Returns a duration that never moves backwards for this clock instance.
    fn now(&self) -> Duration;
}

struct SystemMonotonicClock {
    started_at: Instant,
}

impl SystemMonotonicClock {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

impl MonotonicClock for SystemMonotonicClock {
    fn now(&self) -> Duration {
        self.started_at.elapsed()
    }
}

struct StoredTicket {
    server_name: ServerName<'static>,
    value: Tls13ClientSessionValue,
    inserted_at: Duration,
}

/// One immutable client's bounded, non-persistent TLS 1.3 ticket store.
struct TlsSessionStore {
    tickets: Mutex<VecDeque<StoredTicket>>,
    clock: Arc<dyn MonotonicClock>,
}

impl TlsSessionStore {
    fn new(clock: Arc<dyn MonotonicClock>) -> Self {
        Self {
            tickets: Mutex::new(VecDeque::new()),
            clock,
        }
    }

    fn lock_tickets(&self) -> MutexGuard<'_, VecDeque<StoredTicket>> {
        self.tickets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn discard_expired(&self, tickets: &mut VecDeque<StoredTicket>) {
        let now = self.clock.now();
        tickets.retain(|ticket| now.saturating_sub(ticket.inserted_at) < MAX_LOCAL_TICKET_AGE);
    }

    fn count(&self) -> usize {
        let mut tickets = self.lock_tickets();
        self.discard_expired(&mut tickets);
        tickets.len()
    }
}

impl fmt::Debug for TlsSessionStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TlsSessionStore")
            .field("ticket_count", &self.count())
            .field("ticket_material", &"[REDACTED]")
            .finish()
    }
}

impl ClientSessionStore for TlsSessionStore {
    fn set_kx_hint(&self, _server_name: ServerName<'static>, _group: rustls::NamedGroup) {}

    fn kx_hint(&self, _server_name: &ServerName<'_>) -> Option<rustls::NamedGroup> {
        None
    }

    fn set_tls12_session(
        &self,
        _server_name: ServerName<'static>,
        _value: Tls12ClientSessionValue,
    ) {
    }

    fn tls12_session(&self, _server_name: &ServerName<'_>) -> Option<Tls12ClientSessionValue> {
        None
    }

    fn remove_tls12_session(&self, _server_name: &ServerName<'static>) {}

    fn insert_tls13_ticket(
        &self,
        server_name: ServerName<'static>,
        value: Tls13ClientSessionValue,
    ) {
        let mut tickets = self.lock_tickets();
        self.discard_expired(&mut tickets);
        while tickets.len() >= MAX_TICKETS {
            tickets.pop_front();
        }
        tickets.push_back(StoredTicket {
            server_name,
            value,
            inserted_at: self.clock.now(),
        });
    }

    fn take_tls13_ticket(
        &self,
        server_name: &ServerName<'static>,
    ) -> Option<Tls13ClientSessionValue> {
        let mut tickets = self.lock_tickets();
        self.discard_expired(&mut tickets);
        let index = tickets
            .iter()
            .position(|ticket| ticket.server_name == *server_name)?;
        tickets.remove(index).map(|ticket| ticket.value)
    }
}

/// TLS configuration paired with the verification name used by its endpoint.
pub(crate) struct TlsClientConfig {
    rustls_config: Arc<ClientConfig>,
    server_name: ServerName<'static>,
    session_store: Arc<TlsSessionStore>,
}

impl TlsClientConfig {
    pub(crate) fn rustls_config(&self) -> &RustlsClientConfig {
        &self.rustls_config
    }

    #[allow(dead_code)] // The raw TLS source-including contract is a separate test target.
    pub(crate) fn rustls_config_arc(&self) -> &Arc<ClientConfig> {
        &self.rustls_config
    }

    pub(crate) fn server_name(&self) -> &ServerName<'static> {
        &self.server_name
    }

    pub(crate) fn cached_ticket_count(&self) -> usize {
        self.session_store.count()
    }
}

impl fmt::Debug for TlsClientConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TlsClientConfig")
            .field("server_name", &"[REDACTED]")
            .field("cached_ticket_count", &self.cached_ticket_count())
            .field("tls_policy", &"TLS 1.3, no early data, no key logging")
            .finish()
    }
}

/// Builds the AWS-LC-backed TLS 1.3 client configuration for one transport.
pub(crate) fn build_client_config(
    configuration: &TransportConfig,
) -> Result<TlsClientConfig, TransportConfigError> {
    build_client_config_with_clock(configuration, Arc::new(SystemMonotonicClock::new()))
}

pub(crate) fn build_client_config_with_clock(
    configuration: &TransportConfig,
    clock: Arc<dyn MonotonicClock>,
) -> Result<TlsClientConfig, TransportConfigError> {
    let provider = Arc::clone(configuration.tls_crypto_provider());
    let verifier = build_webpki_verifier(configuration, Arc::clone(&provider))?;
    let builder = tls13_builder(provider)?.with_webpki_verifier(verifier);
    Ok(finish_client_config(configuration, clock, builder))
}

fn build_webpki_verifier(
    configuration: &TransportConfig,
    provider: Arc<rustls::crypto::CryptoProvider>,
) -> Result<Arc<WebPkiServerVerifier>, TransportConfigError> {
    let has_revocation_lists = !configuration.tls_revocation_lists().is_empty();
    WebPkiServerVerifier::builder_with_provider(Arc::clone(configuration.tls_roots()), provider)
        .with_crls(configuration.tls_revocation_lists().to_vec())
        .enforce_revocation_expiration()
        .build()
        .map_err(|_| {
            if has_revocation_lists {
                TransportConfigError::InvalidRevocationList
            } else {
                TransportConfigError::InvalidTrust
            }
        })
}

fn tls13_builder(
    provider: Arc<rustls::crypto::CryptoProvider>,
) -> Result<ConfigBuilder<ClientConfig, WantsVerifier>, TransportConfigError> {
    RustlsClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|_| TransportConfigError::InvalidTlsConfiguration)
}

fn finish_client_config(
    configuration: &TransportConfig,
    clock: Arc<dyn MonotonicClock>,
    builder: ConfigBuilder<ClientConfig, WantsClientCert>,
) -> TlsClientConfig {
    let builder = builder.with_client_cert_resolver(Arc::new(SingleCertAndKey::from(Arc::clone(
        configuration.tls_identity(),
    ))));
    let session_store = Arc::new(TlsSessionStore::new(clock));
    let mut rustls_config = builder;
    rustls_config.resumption =
        Resumption::store(Arc::clone(&session_store) as Arc<dyn ClientSessionStore>);
    rustls_config.enable_early_data = false;
    rustls_config.enable_sni = configuration.tls_sni_enabled();
    rustls_config.key_log = Arc::new(rustls::NoKeyLog);

    TlsClientConfig {
        rustls_config: Arc::new(rustls_config),
        server_name: configuration.tls_server_name().clone(),
        session_store,
    }
}

#[cfg(test)]
pub(crate) fn build_client_config_with_test_verifier(
    configuration: &TransportConfig,
    clock: Arc<dyn MonotonicClock>,
) -> Result<(TlsClientConfig, Arc<CountingVerifier>), TransportConfigError> {
    let provider = Arc::clone(configuration.tls_crypto_provider());
    let inner = build_webpki_verifier(configuration, Arc::clone(&provider))?;
    let verifier = Arc::new(CountingVerifier::new(inner));
    let builder = tls13_builder(provider)?
        .dangerous()
        .with_custom_certificate_verifier(Arc::clone(&verifier) as Arc<dyn ServerCertVerifier>);
    let config = finish_client_config(configuration, clock, builder);
    Ok((config, verifier))
}

#[cfg(test)]
pub(crate) struct CountingVerifier {
    inner: Arc<dyn ServerCertVerifier>,
    calls: std::sync::atomic::AtomicUsize,
    reject: std::sync::atomic::AtomicBool,
}

#[cfg(test)]
impl CountingVerifier {
    fn new(inner: Arc<dyn ServerCertVerifier>) -> Self {
        Self {
            inner,
            calls: std::sync::atomic::AtomicUsize::new(0),
            reject: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub(crate) fn calls(&self) -> usize {
        self.calls.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn reject_new_full_handshakes(&self) {
        self.reject.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
impl fmt::Debug for CountingVerifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CountingVerifier")
            .field("calls", &self.calls())
            .finish()
    }
}

#[cfg(test)]
impl ServerCertVerifier for CountingVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.reject.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Error::General(
                "test verifier rejected a new full handshake".to_owned(),
            ));
        }
        self.inner
            .verify_server_cert(end_entity, intermediates, server_name, ocsp_response, now)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}
