//! KMIPKit Proxy: Stateless Zero-DB KMIP bridge for Cloud KMS and KMIP 1.X -> 2.0/2.1/3.0 translation.
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
