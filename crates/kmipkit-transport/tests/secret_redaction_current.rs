//! Current-public-surface redaction checks for KMIPKIT-0013 FR-004/FR-005 and
//! SC-006. File inputs and key-owner observation are covered by the paired
//! secret_redaction contract target once T022 adds those APIs.

#[path = "support/secret_redaction/current.rs"]
mod current;
