//! Verifies diagnostic redaction of secret material and session state across transport errors.
//!
//! File-source behavior and the crate-internal test-only secret-owner observer
//! are covered alongside current public-surface checks in
//! `secret_redaction_current`.

// This focused contract target uses only configuration and key-handling APIs.
#[allow(dead_code)]
#[path = "../src/config.rs"]
mod config;
#[path = "../src/secret.rs"]
mod secret;
// `config.rs` references the OpenSSL provider; TLS behavior is covered elsewhere.
#[allow(dead_code)]
#[path = "../src/tls.rs"]
mod tls;

#[path = "support/secret_redaction/current.rs"]
mod current;
#[path = "support/secret_redaction/file_sources.rs"]
mod file_sources;
#[path = "support/secret_redaction/zeroization.rs"]
mod zeroization;
