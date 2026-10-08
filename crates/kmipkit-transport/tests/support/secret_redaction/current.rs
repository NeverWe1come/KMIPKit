use std::error::Error;
use std::fmt;

use kmipkit_test_support::EphemeralPki;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RequestDeliveryState,
    TransportCauseCategory, TransportConfig, TransportError, TrustSource,
};

const PRIVATE_KEY_SENTINEL: &str = "KMIP_PRIVATE_KEY_INPUT_SENTINEL_7F31";
const ENDPOINT_PATH_SENTINEL: &str = "KMIP_ENDPOINT_PATH_SENTINEL_8A42";
const ENDPOINT_QUERY_SENTINEL: &str = "KMIP_ENDPOINT_QUERY_SENTINEL_9B53";
const TARGET_QUERY_SENTINEL: &str = "KMIP_TARGET_QUERY_SENTINEL_AC64";
const CREDENTIAL_PATH_SENTINEL: &str = "KMIP_CREDENTIAL_PATH_SENTINEL_BD75";
const DNS_ERROR_SENTINEL: &str = "KMIP_DNS_ERROR_SENTINEL_CE86";
const DEPENDENCY_ERROR_SENTINEL: &str = "KMIP_DEPENDENCY_ERROR_SENTINEL_DF97";

#[test]
fn private_key_input_and_identity_debug_omit_untrusted_key_bytes() {
    let key = PrivateKeyInput::from_pem(PRIVATE_KEY_SENTINEL.as_bytes().to_vec());
    let key_debug = format!("{key:?}");
    let identity = ClientIdentity::new(
        CertificateInput::from_pem(b"CERTIFICATE_SENTINEL".to_vec()),
        key,
    );

    assert!(!key_debug.contains(PRIVATE_KEY_SENTINEL));
    assert!(!format!("{identity:?}").contains(PRIVATE_KEY_SENTINEL));
}

#[test]
fn endpoint_path_query_and_target_query_are_redacted_from_configuration_debug() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let client = pki.client_identity();
    let endpoint = Endpoint::https(format!(
        "https://kmip.example.test/{ENDPOINT_PATH_SENTINEL}?tenant={ENDPOINT_QUERY_SENTINEL}"
    ));
    let identity = ClientIdentity::new(
        CertificateInput::from_der(vec![client.certificate_der().to_vec()]),
        PrivateKeyInput::from_der(client.private_key_der().to_vec()),
    );
    let trust = TrustSource::certificate_authorities(vec![CertificateInput::from_der(vec![
        pki.authority_certificate_der().to_vec(),
    ])]);
    let builder = TransportConfig::builder(endpoint)
        .target_uri(format!("/kmip?tenant={TARGET_QUERY_SENTINEL}"))
        .client_identity(identity)
        .trust_source(trust);
    let builder_debug = format!("{builder:?}");
    let config = builder.build().expect("ephemeral identity builds locally");
    let config_debug = format!("{config:?}");

    for diagnostic in [builder_debug, config_debug] {
        assert!(!diagnostic.contains(ENDPOINT_PATH_SENTINEL));
        assert!(!diagnostic.contains(ENDPOINT_QUERY_SENTINEL));
        assert!(!diagnostic.contains(TARGET_QUERY_SENTINEL));
    }
}

#[test]
fn malformed_key_error_and_debug_display_diagnostics_omit_key_and_path_sentinels() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let client = pki.client_identity();
    let key = PrivateKeyInput::from_pem(PRIVATE_KEY_SENTINEL.as_bytes().to_vec());
    let identity = ClientIdentity::new(
        CertificateInput::from_der(vec![client.certificate_der().to_vec()]),
        key,
    );
    let builder = TransportConfig::builder(Endpoint::raw_tls("localhost", 5696))
        .client_identity(identity)
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]));
    let builder_debug = format!("{builder:?}");
    let error = builder
        .build()
        .expect_err("malformed key input is rejected");
    let diagnostic = format!("builder={builder_debug} error={error} error_debug={error:?}");

    assert!(matches!(
        error,
        kmipkit_transport::TransportConfigError::InvalidCredential
    ));
    assert!(!diagnostic.contains(PRIVATE_KEY_SENTINEL));
    assert!(!diagnostic.contains(CREDENTIAL_PATH_SENTINEL));
}

#[test]
fn encrypted_private_key_is_rejected_without_secret_diagnostics() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let encrypted = b"-----BEGIN ENCRYPTED PRIVATE KEY-----\nSENTINEL_ONLY\n-----END ENCRYPTED PRIVATE KEY-----\n";
    assert_invalid_key(&pki, PrivateKeyInput::from_pem(encrypted.to_vec()));
}

#[test]
fn multiple_pem_private_keys_are_rejected_as_ambiguous() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let client = pki.client_identity();
    let first_key = pem("PRIVATE KEY", client.private_key_der());
    let ambiguous = [first_key.clone(), first_key].concat();
    assert_invalid_key(&pki, PrivateKeyInput::from_pem(ambiguous));
}

#[test]
fn empty_private_key_input_is_rejected() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    assert_invalid_key(&pki, PrivateKeyInput::from_pem(Vec::new()));
}

#[test]
fn malformed_private_key_input_is_rejected_without_echoing_input_bytes() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    assert_invalid_key(
        &pki,
        PrivateKeyInput::from_pem(b"MALFORMED_PRIVATE_KEY_SENTINEL".to_vec()),
    );
}

#[test]
fn mismatched_certificate_and_private_key_are_rejected() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let other = EphemeralPki::generate().expect("second ephemeral test PKI is available");
    let client = pki.client_identity();
    let mismatched = ClientIdentity::new(
        CertificateInput::from_der(vec![client.certificate_der().to_vec()]),
        PrivateKeyInput::from_der(other.client_identity().private_key_der().to_vec()),
    );
    let error = config_builder(&pki, mismatched)
        .build()
        .expect_err("mismatched client certificate and key are rejected");
    assert!(matches!(
        error,
        kmipkit_transport::TransportConfigError::InvalidCredential
    ));
}

#[test]
fn transport_error_drops_dns_path_and_dependency_text_from_every_public_representation() {
    let source = SentinelSource {
        display: DNS_ERROR_SENTINEL,
        debug: CREDENTIAL_PATH_SENTINEL,
        cause: Some(Box::new(SentinelSource {
            display: DEPENDENCY_ERROR_SENTINEL,
            debug: "KMIP_OS_ERROR_SENTINEL_E0A8",
            cause: None,
        })),
    };
    let error = TransportError::new(
        RequestDeliveryState::NotSent,
        TransportCauseCategory::Io,
        source,
    );
    let rendered = format!("{error} {error:?}");
    let source_chain = public_error_chain(&error);

    for sentinel in [
        DNS_ERROR_SENTINEL,
        CREDENTIAL_PATH_SENTINEL,
        DEPENDENCY_ERROR_SENTINEL,
        "KMIP_OS_ERROR_SENTINEL_E0A8",
    ] {
        assert!(!rendered.contains(sentinel));
        assert!(!source_chain.contains(sentinel));
    }
    assert_eq!(error.delivery_state(), RequestDeliveryState::NotSent);
}

fn config_builder(
    pki: &EphemeralPki,
    identity: ClientIdentity,
) -> kmipkit_transport::TransportConfigBuilder {
    TransportConfig::builder(Endpoint::raw_tls("localhost", 5696))
        .client_identity(identity)
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
}

fn assert_invalid_key(pki: &EphemeralPki, key: PrivateKeyInput) {
    let identity = ClientIdentity::new(
        CertificateInput::from_der(vec![pki.client_identity().certificate_der().to_vec()]),
        key,
    );
    let error = config_builder(pki, identity)
        .build()
        .expect_err("unsupported or malformed key input is rejected");
    assert!(matches!(
        error,
        kmipkit_transport::TransportConfigError::InvalidCredential
    ));
}

fn public_error_chain(error: &dyn Error) -> String {
    let mut rendered = String::new();
    let mut current = Some(error);
    while let Some(cause) = current {
        use fmt::Write as _;
        let _ = writeln!(rendered, "{} {:?}", cause, cause);
        current = cause.source();
    }
    rendered
}

pub(super) fn pem(label: &str, bytes: &[u8]) -> Vec<u8> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::with_capacity(bytes.len().div_ceil(3) * 4 + label.len() * 2 + 64);
    output.extend_from_slice(format!("-----BEGIN {label}-----\n").as_bytes());
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or_default();
        let third = chunk.get(2).copied().unwrap_or_default();
        let indexes = [
            usize::from(first >> 2),
            usize::from(((first & 0x03) << 4) | (second >> 4)),
            usize::from(((second & 0x0f) << 2) | (third >> 6)),
            usize::from(third & 0x3f),
        ];
        for (index, value) in indexes.into_iter().enumerate() {
            let padded = (index == 2 && chunk.len() < 2) || (index == 3 && chunk.len() < 3);
            output.push(if padded { b'=' } else { ALPHABET[value] });
        }
        output.push(b'\n');
    }
    output.extend_from_slice(format!("-----END {label}-----\n").as_bytes());
    output
}

struct SentinelSource {
    display: &'static str,
    debug: &'static str,
    cause: Option<Box<dyn Error>>,
}

impl fmt::Display for SentinelSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.display)
    }
}

impl fmt::Debug for SentinelSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.debug)
    }
}

impl Error for SentinelSource {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_deref()
    }
}
