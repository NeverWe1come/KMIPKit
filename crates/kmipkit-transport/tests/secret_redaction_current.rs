//! Verifies that public transport error and debug representations never leak sensitive credentials or session secrets. File inputs and key-owner observation are covered by the paired
//! `secret_redaction` contract target.

#[path = "support/secret_redaction/current.rs"]
mod current;
