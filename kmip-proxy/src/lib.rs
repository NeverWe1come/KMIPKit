//! # `kmipkit-proxy` — Stateless Zero-DB Cloud KMS Proxy & KMIP 1.X Bridge
//!
//! **`kmipkit-proxy`** exposes a single TLS/TCP/HTTP listener ([`ProxyServer`]) backed by
//! `#![forbid(unsafe_code)]` and two composable translation engines:
//!
//! 1. **Cloud & Enterprise KMS Proxy (`RequestDispatcher`)**: Maps OASIS KMIP `2.0`, `2.1`, and `3.0`
//!    operations into native cloud KMS and Secret Store APIs across 10 providers
//!    (`aws`, `azure`, `gcp`, `oci`, `vault`, `infisical`, `ibm`, `alibaba`, `scaleway`, `otc`, plus
//!    [`InMemoryMockProvider`]) using:
//!    - [`TagCodec`]: Deterministic RFC 4648 Base32Hex tag-stuffing (`<= 60` chars per tag).
//!    - [`AutoEnvelope`]: Local AES-256-GCM DEK wrapping with `zeroize::Zeroizing` RAM hygiene.
//!    - [`MetadataCache`]: Short-lived TTL cache reducing cloud control-plane calls.
//! 2. **KMIP 1.X Legacy Protocol Bridge (`kmip1x`, default feature `kmip1x-bridge`)**:
//!    Transparently translates bidirectional `KMIP 1.0..1.4 <-> 2.0/2.1/3.0` traffic via
//!    [`Kmip1xBridge`] (Client-to-Server) and [`Kmip1xS2cBridge`] (Server-to-Client), either toward
//!    an external KMIP server ([`KmipKitUpstreamTransport`]) or chained locally into any Cloud KMS
//!    backend ([`LocalDispatcherUpstream`]).
//!
//! ## Wire Protocols & Security
//!
//! * **Protocol Sniffing**: Detects Raw TTLV (`0x42...`) and HTTP/1.1 `POST` (`TTLV`, `JSON`, `XML`)
//!   on the same port with configurable TLS 1.3 (and optional TLS 1.2 for legacy clients).
//! * **Certificate Formats**: Accepts both PEM and DER certificates, keys, and CA chains; generates
//!   ephemeral [`AutoPki`] credentials automatically when `tls_cert_path` is omitted.
//! * **Cargo Features**: `kmip1x-bridge` (default), `aws`, `azure`, `gcp`, `oci`, `vault`,
//!   `infisical`, `ibm`, `alibaba`, `scaleway`, `otc`, and `all-clouds`.
//!
//! ## Quick Example: Chaining KMIP 1.X Clients to a Proxy Backend
//!
//! ```no_run
//! # #[cfg(feature = "kmip1x-bridge")]
//! # async fn example() -> Result<(), String> {
//! use std::sync::Arc;
//! use kmipkit_proxy::{
//!     InMemoryMockProvider, Kmip1xBridge, LocalDispatcherUpstream, ProxyBehaviorConfig,
//!     ProxyConfig, ProxyServer, RequestDispatcher, UpstreamKmipVersion,
//! };
//!
//! let dispatcher = Arc::new(RequestDispatcher::new(
//!     Arc::new(InMemoryMockProvider::default()),
//!     ProxyBehaviorConfig::default(),
//! ));
//! let bridge = Arc::new(Kmip1xBridge::new(
//!     Arc::new(LocalDispatcherUpstream::new(dispatcher)),
//!     UpstreamKmipVersion::V2_1,
//! ));
//! let handle = ProxyServer::start_kmip1x_bridge(ProxyConfig::default(), bridge).await?;
//! handle.shutdown().await
//! # }
//! ```
#![forbid(unsafe_code)]
#![allow(
    unknown_lints,
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
    clippy::bool_to_int_with_if,
    clippy::manual_let_else,
    clippy::single_match_else,
    clippy::manual_range_contains
)]

pub mod cache;
pub mod codec;
pub mod config;
pub(crate) mod crypto;
pub mod dispatcher;
pub mod envelope;
pub mod error;
#[cfg(feature = "kmip1x-bridge")]
pub mod kmip1x;
pub mod pki;
pub mod providers;
pub mod server;
pub mod tag_codec;

#[cfg(feature = "aws")]
mod aws_http;

pub use cache::MetadataCache;
pub use config::{
    KeyCustodyMode, KmipUpstreamConfig, ListenerConfig, OperationPolicyMode, ProxyBehaviorConfig,
    ProxyConfig,
};
pub use dispatcher::RequestDispatcher;
pub use envelope::AutoEnvelope;
pub use error::ProxyError;
#[cfg(feature = "kmip1x-bridge")]
pub use kmip1x::{
    Kmip1xBridge, UpstreamKmipTransport,
    mapping::{OperationPolicyHandling, UpstreamKmipVersion},
    s2c::{Kmip1xS2cBridge, S2cClientTransport},
    translator::TranslatorConfig,
    upstream::{KmipKitUpstreamTransport, LocalDispatcherUpstream},
};
pub use pki::AutoPki;
pub use providers::{CloudKmsProvider, InMemoryMockProvider};
pub use server::{ProxyServer, ProxyServerHandle};
pub use tag_codec::{KmipAttributes, TagCodec};
