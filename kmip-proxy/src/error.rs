//! Error types for KMIPKit Proxy.
#![forbid(unsafe_code)]

use std::fmt;

/// Unified error enum for KMIPKit Proxy operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyError {
    /// Configuration parsing or validation error.
    ConfigError(String),
    /// Cryptographic or envelope wrapping error.
    CryptoError(String),
    /// Cloud KMS provider error.
    ProviderError(String),
    /// Cloud KMS provider not configured or missing credentials.
    ProviderNotConfigured(String),
    /// Requested key or secret item not found in KMS.
    ItemNotFound(String),
    /// Requested operation or algorithm unsupported by provider.
    Unsupported(String),
    /// Key wrapping / auto-envelope error.
    EnvelopeError(String),
    /// Network or TLS transport error.
    NetworkError(String),
    /// KMIP protocol decoding, encoding, or dispatch error.
    ProtocolError(String),
    /// Internal unexpected error.
    Internal(String),
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            Self::CryptoError(msg) => write!(f, "Cryptographic error: {msg}"),
            Self::ProviderError(msg) => write!(f, "Provider error: {msg}"),
            Self::ProviderNotConfigured(msg) => write!(f, "Provider not configured: {msg}"),
            Self::ItemNotFound(id) => write!(f, "Item not found: {id}"),
            Self::Unsupported(msg) => write!(f, "Unsupported operation: {msg}"),
            Self::EnvelopeError(msg) => write!(f, "Envelope error: {msg}"),
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::ProtocolError(msg) => write!(f, "Protocol error: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for ProxyError {}
