use std::sync::Arc;

use rustls::{ClientConfig, RootCertStore};

use super::build_http1_tls_config;

#[test]
fn http1_tls_config_clears_nonempty_alpn_without_mutating_source() {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let mut source_config = ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the test provider supports TLS 1.3")
        .with_root_certificates(RootCertStore::empty())
        .with_no_client_auth();
    source_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let source_config = Arc::new(source_config);

    let http1_config = build_http1_tls_config(&source_config);

    assert_eq!(
        source_config.alpn_protocols,
        [b"h2".to_vec(), b"http/1.1".to_vec()]
    );
    assert_eq!(http1_config.alpn_protocols, Vec::<Vec<u8>>::new());
}
