//! Red-phase contract tests for KMIPKIT-0013 FR-004/FR-005 and SC-006.
//!
//! The file-source and test-only secret-owner observer APIs below are the
//! smallest additive surface implied by the approved TLS Configuration
//! contract. This target intentionally cannot compile until T022 implements
//! them. Current public-surface checks also live in secret_redaction_current
//! so they remain independently runnable during this Red phase.

#[path = "../src/config.rs"]
mod config;
#[path = "../src/secret.rs"]
mod secret;
#[path = "../src/tls.rs"]
mod tls;

#[path = "support/secret_redaction/current.rs"]
mod current;
#[path = "support/secret_redaction/file_sources.rs"]
mod file_sources;
#[path = "support/secret_redaction/zeroization.rs"]
mod zeroization;
