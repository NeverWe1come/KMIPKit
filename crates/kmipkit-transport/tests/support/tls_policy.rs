//! Private, configuration-only TLS safety policy shared by transport adapters.

use std::sync::Arc;

use rustls::{ClientConfig, ConfigBuilder, Error, NoKeyLog, WantsVerifier};

/// Creates a client configuration builder pinned to AWS-LC and TLS 1.3.
pub(crate) fn client_config_builder() -> Result<ConfigBuilder<ClientConfig, WantsVerifier>, Error> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    ClientConfig::builder_with_provider(provider).with_protocol_versions(&[&rustls::version::TLS13])
}

/// Applies security switches that must remain disabled on every client config.
pub(crate) fn apply_client_safety_policy(config: &mut ClientConfig) {
    config.key_log = Arc::new(NoKeyLog);
    config.enable_early_data = false;
}
