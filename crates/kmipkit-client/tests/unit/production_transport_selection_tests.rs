use kmipkit_test_support::EphemeralPki;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
};

use super::ClientTransport;

#[test]
fn validated_endpoint_selects_the_matching_production_adapter() {
    let raw = ClientTransport::from_configuration(valid_transport_config(Endpoint::raw_tls(
        "localhost",
        5696,
    )))
    .expect("raw TLS configuration creates the raw TLS adapter");
    assert!(matches!(raw, ClientTransport::RawTls(_)));

    let https = ClientTransport::from_configuration(valid_transport_config(Endpoint::https(
        "https://localhost:5697",
    )))
    .expect("HTTPS configuration creates the HTTPS adapter");
    assert!(matches!(https, ClientTransport::Https(_)));
}

#[test]
fn transport_configuration_failures_keep_validation_delivery_evidence() {
    for source in [
        kmipkit_transport::TransportConfigError::InvalidEndpoint,
        kmipkit_transport::TransportConfigError::InvalidTarget,
    ] {
        let error = super::transport_configuration_error(source);

        assert_eq!(error.category(), crate::ClientErrorCategory::Validation);
        assert_eq!(
            error.cause_category(),
            Some(crate::ClientCauseCategory::InvalidInput)
        );
        assert_eq!(
            error.delivery_state(),
            Some(kmipkit_transport::RequestDeliveryState::NotSent)
        );
    }
}

fn valid_transport_config(endpoint: Endpoint) -> TransportConfig {
    let pki = EphemeralPki::generate().expect("ephemeral client PKI should be generated");
    let identity = pki.client_identity();
    TransportConfig::builder(endpoint)
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der(
                identity
                    .certificate_chain_der()
                    .into_iter()
                    .map(<[u8]>::to_vec)
                    .collect(),
            ),
            PrivateKeyInput::from_der(identity.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .build()
        .expect("ephemeral identity and explicit root should validate")
}
