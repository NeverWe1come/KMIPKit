//! Public synchronous byte exchange and validated configuration for `KMIPKit`.
//!
//! The low-level [`Transport`] API sends caller-supplied bytes as-is: it does
//! not encode or validate KMIP messages. The caller owns the request bytes and
//! is responsible for their contents. An implementation borrows the request
//! only during the synchronous call, must not log or retain it, and must not
//! retry automatically. The typed client remains a separate API and does not
//! accept an arbitrary caller-provided [`Transport`].
//! [`RawTlsTransport`] provides the raw-TLS adapter and [`HttpsTransport`]
//! provides the HTTPS/HTTP 1.1 adapter. The top-level `kmipkit` facade does not
//! re-export this crate, and `kmipkit-client::Client` does not accept a
//! caller-implemented transport.
//!
//! # Validated transport configuration
//!
//! [`Endpoint::raw_tls`] selects a raw-TLS host and nonzero port;
//! [`Endpoint::https`] selects an absolute HTTPS URI. Endpoint and request
//! target validation happens when [`TransportConfigBuilder::build`] is
//! called. HTTPS targets are origin-form path/query values and default to
//! `/kmip`.
//!
//! Supply the client certificate chain and private key explicitly through
//! [`ClientIdentity`]. [`CertificateInput`] and [`PrivateKeyInput`] require an
//! explicit PEM or DER constructor; private keys must be unencrypted
//! PKCS#8, PKCS#1, or SEC1 encodings supported by the selected provider.
//! Configuration requires one explicit trust source: caller-provided
//! certificates selected with [`TrustSource::certificate_authorities`], or
//! native platform roots selected with [`TrustSource::platform`]. The builder
//! parses and validates the selected material before returning a config; it
//! opens no socket and performs no DNS lookup.
//!
//! The production-adapter TLS policy uses rustls with AWS-LC, TLS 1.3 only,
//! and mutual TLS. Adapters validate the server chain, certificate validity,
//! and hostname; an optional TLS server-name override is available only for IP
//! endpoints. Early data and TLS key logging are disabled. Caller-supplied
//! CRLs are optional, and no CRL or OCSP data is fetched from the network.
//! Platform trust loads native roots through `rustls-native-certs`; when
//! `SSL_CERT_FILE` is set, the platform loader uses that bundle in preference
//! to the native store. Loader errors fail closed. This mode does not import
//! OS-specific distrust or revocation decisions.
//!
//! Session tickets are held in bounded state scoped to one client config (at
//! most 16 tickets, with a one-hour local maximum age). A resumed session
//! inherits the full handshake's peer identity and trust/CRL decision;
//! rebuilding the config applies changed trust inputs and starts with an empty
//! ticket store.
//!
//! A validated config defaults to a 16 MiB positive request limit. The
//! [`Transport::exchange`] contract receives `max_response_bytes` on every
//! call and requires implementations to stop response reads at that cap; the
//! trait has no default response cap. The default client timeouts are 10 s for
//! connect, 30 s for write inactivity, 30 s for read inactivity, and 60 s for
//! the absolute exchange deadline. [`RequestOptions`] carries optional
//! per-exchange overrides, and [`TimeoutPolicy::with_overrides`] shows how
//! those values compose with client defaults. [`RawTlsTransport`] exposes an
//! additive `exchange_with_options` entry point accepting these options;
//! unspecified phases inherit the config's [`TimeoutPolicy`].
//!
//! # Credential files and diagnostics
//!
//! Certificate PEM and DER file constructors read the selected path once
//! with `std::fs::read`. Private-key PEM and DER constructors open the
//! selected path once and read that file through `KMIPKit`'s zeroizing key
//! owner; bounded reads may require more than one `Read::read` call. The path
//! is not retained, normal operating-system path and symlink rules apply, and
//! open/read failures map to [`TransportConfigError::InvalidCredential`]
//! without retaining OS error text or the path.
//!
//! Credential, endpoint, config, and transport-error diagnostics redact
//! credential bytes, endpoint path/query details, and dependency error text.
//! [`TransportError`] exposes safe cause and delivery categories.
//!
//! `KMIPKit` zeroizes initialized bytes in the secret and response allocations
//! it owns, including key-buffer allocations before controlled replacement.
//! The guarantee does not cover spare/uninitialized capacity, caller-created
//! copies, buffers owned by rustls/AWS-LC, the operating system, or other
//! dependencies. Direct callers also retain ownership of their request bytes;
//! [`TransportResponse`] owns and zeroizes its current response allocation on
//! drop.
//!
//! See [ADR-0014](../../../docs/adr/0014-public-transport-exchange-contract.md)
//! for the low-level ownership, response-limit, and cleanup contract.
//!
//! # Examples
//!
//! A direct implementation borrows the request and returns a bounded response
//! wrapper. This example uses only the public transport contract:
//!
//! ```
//! use kmipkit_transport::{Transport, TransportError, TransportResponse};
//!
//! struct EchoTransport;
//!
//! impl Transport for EchoTransport {
//!     fn exchange(
//!         &mut self,
//!         request: &[u8],
//!         max_response_bytes: usize,
//!     ) -> Result<TransportResponse, TransportError> {
//!         assert!(request.len() <= max_response_bytes);
//!         Ok(TransportResponse::new(request.to_vec()))
//!     }
//! }
//!
//! let mut transport = EchoTransport;
//! let response = transport.exchange(b"request", 64).unwrap();
//! assert_eq!(response.as_bytes(), b"request");
//! ```
//!
//! Timeout overrides apply only to phases explicitly set. The other phases
//! retain the client's defaults:
//!
//! ```
//! use std::time::Duration;
//!
//! use kmipkit_transport::{RequestOptions, TimeoutLimit, TimeoutPolicy};
//!
//! let options = RequestOptions::default()
//!     .with_connect(TimeoutLimit::Unbounded)
//!     .with_read(TimeoutLimit::Bounded(Duration::from_secs(5)));
//! let effective = TimeoutPolicy::default().with_overrides(&options);
//!
//! assert_eq!(effective.connect(), TimeoutLimit::Unbounded);
//! assert_eq!(effective.write(), TimeoutLimit::Bounded(Duration::from_secs(30)));
//! assert_eq!(effective.read(), TimeoutLimit::Bounded(Duration::from_secs(5)));
//! assert_eq!(effective.total(), TimeoutLimit::Bounded(Duration::from_secs(60)));
//! ```
//!
//! Input constructors make the encoding explicit. The builder rejects
//! malformed material with a fixed error category before an adapter connects:
//!
//! ```
//! use kmipkit_transport::{
//!     CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput,
//!     TransportConfig, TransportConfigError, TrustSource,
//! };
//!
//! let result = TransportConfig::builder(Endpoint::https("https://kmip.example"))
//!     .client_identity(ClientIdentity::new(
//!         CertificateInput::from_der(Vec::new()),
//!         PrivateKeyInput::from_der(vec![0]),
//!     ))
//!     .trust_source(TrustSource::platform())
//!     .build();
//!
//! assert!(matches!(result, Err(TransportConfigError::InvalidCredential)));
//! ```
//!
//! File constructors read their source during construction and do not retain
//! the path. This executable example uses deliberately malformed bytes (not
//! credentials) to demonstrate that file input is loaded before config
//! validation and that the resulting error stays redacted:
//!
//! ```
//! use std::error::Error;
//! use std::time::{SystemTime, UNIX_EPOCH};
//!
//! use kmipkit_transport::{
//!     CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput,
//!     TransportConfig, TransportConfigError, TrustSource,
//! };
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
//!     let directory = std::env::temp_dir().join(format!(
//!         "kmipkit-transport-doc-{}-{nonce}",
//!         std::process::id()
//!     ));
//!     std::fs::create_dir(&directory)?;
//!     let certificate_path = directory.join("chain.der");
//!     let key_path = directory.join("key.der");
//!     std::fs::write(&certificate_path, [])?;
//!     std::fs::write(&key_path, [0])?;
//!
//!     let identity = ClientIdentity::new(
//!         CertificateInput::from_der_file(&certificate_path)?,
//!         PrivateKeyInput::from_der_file(&key_path)?,
//!     );
//!     std::fs::remove_file(&certificate_path)?;
//!     std::fs::remove_file(&key_path)?;
//!
//!     let error = TransportConfig::builder(Endpoint::https("https://kmip.example"))
//!         .client_identity(identity)
//!         .trust_source(TrustSource::platform())
//!         .build()
//!         .expect_err("an empty certificate chain must be rejected");
//!     assert_eq!(error, TransportConfigError::InvalidCredential);
//!     assert!(!error.to_string().contains(directory.to_string_lossy().as_ref()));
//!     std::fs::remove_dir(directory)?;
//!     Ok(())
//! }
//! ```
//!
//! A successful configuration can load its explicitly selected credentials
//! from files. This compile-checked example expects the named files to contain
//! valid material when run:
//!
//! ```no_run
//! use std::error::Error;
//!
//! use kmipkit_transport::{
//!     CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput,
//!     TransportConfig, TrustSource,
//! };
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let identity = ClientIdentity::new(
//!         CertificateInput::from_pem_file("client-chain.pem")?,
//!         PrivateKeyInput::from_pem_file("client-key.pem")?,
//!     );
//!     let config = TransportConfig::builder(Endpoint::https("https://kmip.example"))
//!         .client_identity(identity)
//!         .trust_source(TrustSource::certificate_authorities(vec![
//!             CertificateInput::from_pem_file("root-ca.pem")?,
//!         ]))
//!         .build()?;
//!     assert_eq!(config.target_uri(), Some("/kmip"));
//!     Ok(())
//! }
//! ```
//!
//! File and key lifecycle limitations are unchanged by these examples: pass
//! protected paths, handle the returned zeroizing owners, and treat all
//! third-party and caller-created copies as caller/external memory.
#![forbid(unsafe_code)]

mod config;
mod error;
mod https;
mod raw_tls;
mod response;
mod secret;
// T019 exercises this private builder before T023 connects production adapters.
#[allow(dead_code)]
mod tls;
// T010 introduces the private timeout seam before later adapter tasks consume it.
#[cfg(test)]
#[path = "../tests/unit/https_transport_tests.rs"]
mod https_transport_tests;
#[cfg(test)]
#[path = "../tests/unit/raw_tls_tests.rs"]
mod raw_tls_tests;
#[allow(dead_code)]
mod timeout;
#[cfg(test)]
#[path = "../tests/support/tls_policy.rs"]
mod tls_policy;
#[cfg(test)]
#[path = "../tests/unit/tls_policy_tests.rs"]
mod tls_policy_tests;
#[cfg(test)]
#[path = "../tests/unit/tls_safety_tests.rs"]
mod tls_safety_tests;
#[cfg(test)]
#[path = "../tests/unit/transport_test_support.rs"]
mod transport_test_support;
// cargo-llvm-cov omits coverage from standalone integration-test binaries.
// These canonical public-API cases also run in the library test target so
// their production-source hits contribute to the aggregate gate.
#[cfg(test)]
extern crate self as kmipkit_transport;
#[cfg(test)]
#[path = "../tests/tls_config.rs"]
mod tls_config_coverage_tests;
// The resolver precedes the production adapters that consume it in later tasks.
#[allow(dead_code)]
mod resolver;
// The worker precedes the production adapters that consume it in later tasks.
#[allow(dead_code)]
mod worker;

pub use config::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RequestOptions,
    RevocationListInput, TimeoutLimit, TimeoutPolicy, TransportConfig, TransportConfigBuilder,
    TransportConfigError, TrustSource,
};
pub use error::{RequestDeliveryState, TransportCauseCategory, TransportError};
pub use https::HttpsTransport;
pub use raw_tls::RawTlsTransport;
pub use response::TransportResponse;

/// A synchronous, bounded exchange of caller-supplied request bytes.
///
/// Implementations borrow the request only for the duration of `exchange`.
/// They must not log, format, or retain its contents after the call returns.
/// Response reads must stop at `max_response_bytes`; implementations must
/// clean up any KMIPKit-owned temporary request or response allocations before
/// releasing them. Failures preserve the strongest available request-delivery
/// evidence and must not expose request or response contents.
pub trait Transport {
    /// Exchanges one request and returns its bounded response.
    ///
    /// `request` remains owned by the caller. This method performs no KMIP
    /// encoding or validation. Implementations must not retry the exchange
    /// automatically, retain the request beyond this synchronous call, or
    /// return a response larger than `max_response_bytes`.
    ///
    /// # Errors
    ///
    /// Returns a redacted error that preserves the strongest available
    /// [`RequestDeliveryState`]. Any partial response allocation owned by
    /// `KMIPKit` must be zeroized before this method returns an error.
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError>;
}
