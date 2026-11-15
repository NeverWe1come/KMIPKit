use kmipkit_test_support::EphemeralPki;

use super::current::pem;
use super::{config, secret};

fn observed_private_key(bytes: Vec<u8>) -> (config::PrivateKeyInput, secret::SecretBufferObserver) {
    let observer = secret::SecretBufferObserver::new(bytes.len());
    let input = config::PrivateKeyInput::from_pem_with_observer_for_test(bytes, observer.clone());
    (input, observer)
}

fn builder_with_key(
    pki: &EphemeralPki,
    key: config::PrivateKeyInput,
) -> config::TransportConfigBuilder {
    let client = pki.client_identity();
    config::TransportConfig::builder(config::Endpoint::raw_tls("localhost", 5696))
        .client_identity(config::ClientIdentity::new(
            config::CertificateInput::from_der(vec![client.certificate_der().to_vec()]),
            key,
        ))
        .trust_source(config::TrustSource::certificate_authorities(vec![
            config::CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
}

fn assert_rejected_key_is_zeroized(pki: &EphemeralPki, bytes: Vec<u8>) {
    let expected_len = bytes.len();
    let (key, observer) = observed_private_key(bytes);

    let result = builder_with_key(pki, key).build();

    assert!(result.is_err());
    assert_eq!(observer.initialized_len(), expected_len);
    assert!(observer.initialized_range_was_zero());
}

#[test]
fn initialized_key_buffer_is_zeroized_after_successful_configuration_build() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let key_bytes = pem("PRIVATE KEY", pki.client_identity().private_key_der());
    let expected_len = key_bytes.len();
    let (key, observer) = observed_private_key(key_bytes);

    let result = builder_with_key(&pki, key).build();

    assert!(result.is_ok());
    assert_eq!(observer.initialized_len(), expected_len);
    assert!(observer.initialized_range_was_zero());
}

#[test]
fn initialized_key_buffer_is_zeroized_when_encrypted_key_is_rejected() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let bytes = b"-----BEGIN ENCRYPTED PRIVATE KEY-----\nSENTINEL_ONLY\n-----END ENCRYPTED PRIVATE KEY-----\n".to_vec();
    assert_rejected_key_is_zeroized(&pki, bytes);
}

#[test]
fn initialized_key_buffer_is_zeroized_when_ambiguous_keys_are_rejected() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let key = pki.client_identity().private_key_der();
    let bytes = [pem("PRIVATE KEY", key), pem("PRIVATE KEY", key)].concat();
    assert_rejected_key_is_zeroized(&pki, bytes);
}

#[test]
fn initialized_key_buffer_is_zeroized_when_malformed_key_is_rejected() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    assert_rejected_key_is_zeroized(&pki, b"KMIP_MALFORMED_KEY_SENTINEL".to_vec());
}

#[test]
fn initialized_key_buffer_is_zeroized_when_empty_key_is_rejected() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    assert_rejected_key_is_zeroized(&pki, Vec::new());
}

#[test]
fn initialized_key_buffer_is_zeroized_when_certificate_key_mismatch_is_rejected() {
    let certificate_pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let key_pki = EphemeralPki::generate().expect("second ephemeral test PKI is available");
    let bytes = pem("PRIVATE KEY", key_pki.client_identity().private_key_der());
    let expected_len = bytes.len();
    let (key, observer) = observed_private_key(bytes);
    let certificate = certificate_pki.client_identity();
    let builder = config::TransportConfig::builder(config::Endpoint::raw_tls("localhost", 5696))
        .client_identity(config::ClientIdentity::new(
            config::CertificateInput::from_der(vec![certificate.certificate_der().to_vec()]),
            key,
        ))
        .trust_source(config::TrustSource::certificate_authorities(vec![
            config::CertificateInput::from_der(vec![
                certificate_pki.authority_certificate_der().to_vec(),
            ]),
        ]));

    let result = builder.build();

    assert!(result.is_err());
    assert_eq!(observer.initialized_len(), expected_len);
    assert!(observer.initialized_range_was_zero());
}

#[test]
fn initialized_key_buffer_is_zeroized_when_certificate_parsing_fails() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let bytes = pem("PRIVATE KEY", pki.client_identity().private_key_der());
    let expected_len = bytes.len();
    let (key, observer) = observed_private_key(bytes);
    let builder = config::TransportConfig::builder(config::Endpoint::raw_tls("localhost", 5696))
        .client_identity(config::ClientIdentity::new(
            config::CertificateInput::from_pem(b"KMIP_MALFORMED_CERTIFICATE_SENTINEL".to_vec()),
            key,
        ))
        .trust_source(config::TrustSource::certificate_authorities(vec![
            config::CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]));

    let result = builder.build();

    assert!(result.is_err());
    assert_eq!(observer.initialized_len(), expected_len);
    assert!(observer.initialized_range_was_zero());
}

#[test]
fn initialized_key_buffer_is_zeroized_when_configuration_fails_before_key_parsing() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let bytes = pem("PRIVATE KEY", pki.client_identity().private_key_der());
    let expected_len = bytes.len();
    let (key, observer) = observed_private_key(bytes);
    let identity = config::ClientIdentity::new(
        config::CertificateInput::from_der(vec![pki.client_identity().certificate_der().to_vec()]),
        key,
    );

    let result = config::TransportConfig::builder(config::Endpoint::raw_tls("", 5696))
        .client_identity(identity)
        .build();

    assert!(result.is_err());
    assert_eq!(observer.initialized_len(), expected_len);
    assert!(observer.initialized_range_was_zero());
}

#[test]
fn initialized_key_buffer_is_zeroized_when_trust_loading_fails_after_key_parsing() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let bytes = pem("PRIVATE KEY", pki.client_identity().private_key_der());
    let expected_len = bytes.len();
    let (key, observer) = observed_private_key(bytes);
    let identity = config::ClientIdentity::new(
        config::CertificateInput::from_der(vec![pki.client_identity().certificate_der().to_vec()]),
        key,
    );

    let result = config::TransportConfig::builder(config::Endpoint::raw_tls("localhost", 5696))
        .client_identity(identity)
        .trust_source(config::TrustSource::certificate_authorities(Vec::new()))
        .build();

    assert!(result.is_err());
    assert_eq!(observer.initialized_len(), expected_len);
    assert!(observer.initialized_range_was_zero());
}
