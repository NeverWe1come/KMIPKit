use hyper::http::Method;
use hyper::http::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, HOST};
use kmipkit_test_support::EphemeralPki;

use crate::config::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
};

use super::build_request_for_test;

fn valid_https_configuration() -> TransportConfig {
    let pki = EphemeralPki::generate().expect("the ephemeral test PKI is generated");
    let client = pki.client_identity();
    TransportConfig::builder(Endpoint::https("https://kmip.example"))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der(
                client
                    .certificate_chain_der()
                    .into_iter()
                    .map(<[u8]>::to_vec)
                    .collect(),
            ),
            PrivateKeyInput::from_der(client.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .build()
        .expect("the explicit HTTPS test inputs build a valid configuration")
}

#[test]
fn request_builder_keeps_the_kmip_http1_method_headers_and_authority() {
    let configuration = valid_https_configuration();
    let request = build_request_for_test(&configuration, b"request bytes")
        .expect("a validated HTTPS route builds its request");

    assert_eq!(request.method(), Method::POST);
    assert_eq!(request.uri().to_string(), "/kmip");
    assert_eq!(request.headers()[CONTENT_TYPE], "application/octet-stream");
    assert_eq!(request.headers()[CONTENT_LENGTH], "13");
    assert_eq!(request.headers()[CACHE_CONTROL], "no-cache");
    assert_eq!(request.headers()[HOST], "kmip.example");
}
