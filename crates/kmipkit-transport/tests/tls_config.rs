//! Derived tests for KMIPKIT-0013 FR-003, FR-004, FR-009, FR-011, and FR-017.
//!
//! The default HTTPS target also verifies OASIS KMIP Profiles v2.1 OS §5.3.1
//! item 3 (`SHOULD`), while caller-selected origin-form targets cover item 4.

use std::time::Duration;

use kmipkit_test_support::EphemeralPki;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RequestOptions, TimeoutLimit,
    TimeoutPolicy, TransportConfig, TransportConfigBuilder, TrustSource,
};

const DEFAULT_MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;

fn identity_and_trust(pem_encoded: bool) -> (ClientIdentity, TrustSource) {
    let pki = EphemeralPki::generate().expect("ephemeral client PKI should be generated");
    let client = pki.client_identity();
    let identity = if pem_encoded {
        let certificates: Vec<u8> = client
            .certificate_chain_der()
            .into_iter()
            .flat_map(|certificate| pem("CERTIFICATE", certificate))
            .collect();
        ClientIdentity::new(
            CertificateInput::from_pem(certificates),
            PrivateKeyInput::from_pem(pem("PRIVATE KEY", client.private_key_der())),
        )
    } else {
        ClientIdentity::new(
            CertificateInput::from_der(
                client
                    .certificate_chain_der()
                    .into_iter()
                    .map(<[u8]>::to_vec)
                    .collect(),
            ),
            PrivateKeyInput::from_der(client.private_key_der().to_vec()),
        )
    };
    let root = if pem_encoded {
        CertificateInput::from_pem(pem("CERTIFICATE", pki.authority_certificate_der()))
    } else {
        CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()])
    };
    (identity, TrustSource::certificate_authorities(vec![root]))
}

fn valid_builder(endpoint: Endpoint, pem_encoded: bool) -> TransportConfigBuilder {
    let (identity, trust) = identity_and_trust(pem_encoded);
    TransportConfig::builder(endpoint)
        .client_identity(identity)
        .trust_source(trust)
}

fn pem(label: &str, bytes: &[u8]) -> Vec<u8> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = Vec::with_capacity(bytes.len().div_ceil(3) * 4 + label.len() * 2 + 64);
    encoded.extend_from_slice(format!("-----BEGIN {label}-----\n").as_bytes());
    let mut column = 0;
    for group in bytes.chunks(3) {
        let first = group[0];
        let second = group.get(1).copied().unwrap_or_default();
        let third = group.get(2).copied().unwrap_or_default();
        let indexes = [
            usize::from(first >> 2),
            usize::from(((first & 0x03) << 4) | (second >> 4)),
            usize::from(((second & 0x0f) << 2) | (third >> 6)),
            usize::from(third & 0x3f),
        ];
        for (index, value) in indexes.into_iter().enumerate() {
            let padding = (index == 2 && group.len() < 2) || (index == 3 && group.len() < 3);
            encoded.push(if padding { b'=' } else { ALPHABET[value] });
            column += 1;
            if column == 64 {
                encoded.push(b'\n');
                column = 0;
            }
        }
    }
    if column != 0 {
        encoded.push(b'\n');
    }
    encoded.extend_from_slice(format!("-----END {label}-----\n").as_bytes());
    encoded
}

#[test]
fn raw_tls_endpoint_requires_a_host_and_nonzero_port() {
    assert!(
        valid_builder(Endpoint::raw_tls("localhost", 5696), false)
            .build()
            .is_ok()
    );
    assert!(
        valid_builder(Endpoint::raw_tls("", 5696), false)
            .build()
            .is_err()
    );
    assert!(
        valid_builder(Endpoint::raw_tls("localhost", 0), false)
            .build()
            .is_err()
    );
}

#[test]
fn https_endpoint_must_be_absolute_https_without_userinfo_or_fragment() {
    assert!(
        valid_builder(Endpoint::https("https://kmip.example.test:5696"), false)
            .build()
            .is_ok()
    );
    for endpoint in [
        "http://kmip.example.test:5696",
        "kmip.example.test:5696",
        "https://user:password@kmip.example.test:5696",
        "https://kmip.example.test:invalid",
        "https://kmip.example.test:5696/#fragment",
    ] {
        assert!(
            valid_builder(Endpoint::https(endpoint), false)
                .build()
                .is_err(),
            "endpoint should be rejected: {endpoint}"
        );
    }
}

#[test]
fn https_request_target_defaults_to_kmip_and_accepts_only_origin_form() {
    let default = valid_builder(Endpoint::https("https://kmip.example.test"), false)
        .build()
        .expect("valid HTTPS endpoint builds");
    assert_eq!(default.target_uri(), Some("/kmip"));

    let custom = valid_builder(Endpoint::https("https://kmip.example.test:5696"), false)
        .target_uri("/custom/kmip?tenant=blue")
        .build()
        .expect("origin-form path and query build");
    assert_eq!(custom.target_uri(), Some("/custom/kmip?tenant=blue"));

    for target in [
        "kmip",
        "//attacker.example.test/kmip",
        "https://attacker.example.test/kmip",
        "/kmip#fragment",
    ] {
        assert!(
            valid_builder(Endpoint::https("https://kmip.example.test"), false)
                .target_uri(target)
                .build()
                .is_err(),
            "target should be rejected: {target}"
        );
    }
    assert!(
        valid_builder(Endpoint::raw_tls("localhost", 5696), false)
            .target_uri("/kmip")
            .build()
            .is_err()
    );
}

#[test]
fn trust_roots_and_client_identity_are_both_explicitly_required() {
    let (identity, trust) = identity_and_trust(false);
    assert!(
        TransportConfig::builder(Endpoint::raw_tls("localhost", 5696))
            .client_identity(identity)
            .build()
            .is_err()
    );

    let (identity, _) = identity_and_trust(false);
    assert!(
        TransportConfig::builder(Endpoint::raw_tls("localhost", 5696))
            .client_identity(identity)
            .trust_source(TrustSource::platform())
            .build()
            .is_ok()
    );

    assert!(
        TransportConfig::builder(Endpoint::raw_tls("localhost", 5696))
            .trust_source(trust)
            .build()
            .is_err()
    );
}

#[test]
fn explicit_pem_and_der_inputs_build_without_automatic_format_detection() {
    assert!(
        valid_builder(Endpoint::raw_tls("localhost", 5696), false)
            .build()
            .is_ok()
    );
    assert!(
        valid_builder(Endpoint::raw_tls("localhost", 5696), true)
            .build()
            .is_ok()
    );
}

#[test]
fn request_limit_defaults_to_sixteen_mib_and_accepts_only_positive_overrides() {
    let default = valid_builder(Endpoint::raw_tls("localhost", 5696), false)
        .build()
        .expect("default config is valid");
    assert_eq!(default.max_request_bytes(), DEFAULT_MAX_REQUEST_BYTES);

    let small = valid_builder(Endpoint::raw_tls("localhost", 5696), false)
        .max_request_bytes(1)
        .build()
        .expect("positive request limit is valid");
    assert_eq!(small.max_request_bytes(), 1);
    assert!(
        valid_builder(Endpoint::raw_tls("localhost", 5696), false)
            .max_request_bytes(0)
            .build()
            .is_err()
    );
}

#[test]
fn timeout_defaults_keep_zero_distinct_from_unbounded_and_support_client_overrides() {
    let policy = TimeoutPolicy::default();
    assert_eq!(
        policy.connect(),
        TimeoutLimit::Bounded(Duration::from_secs(10))
    );
    assert_eq!(
        policy.write(),
        TimeoutLimit::Bounded(Duration::from_secs(30))
    );
    assert_eq!(
        policy.read(),
        TimeoutLimit::Bounded(Duration::from_secs(30))
    );
    assert_eq!(
        policy.total(),
        TimeoutLimit::Bounded(Duration::from_secs(60))
    );
    assert_ne!(
        TimeoutLimit::Bounded(Duration::ZERO),
        TimeoutLimit::Unbounded
    );

    let policy = policy
        .with_connect(TimeoutLimit::Bounded(Duration::ZERO))
        .with_total(TimeoutLimit::Unbounded);
    assert_eq!(policy.connect(), TimeoutLimit::Bounded(Duration::ZERO));
    assert_eq!(policy.total(), TimeoutLimit::Unbounded);
}

#[test]
fn one_request_options_value_overrides_the_shared_typed_and_adapter_policy() {
    let defaults = TimeoutPolicy::default();
    let options = RequestOptions::default()
        .with_connect(TimeoutLimit::Unbounded)
        .with_read(TimeoutLimit::Bounded(Duration::ZERO));
    let effective = defaults.with_overrides(&options);

    assert_eq!(effective.connect(), TimeoutLimit::Unbounded);
    assert_eq!(effective.write(), defaults.write());
    assert_eq!(effective.read(), TimeoutLimit::Bounded(Duration::ZERO));
    assert_eq!(effective.total(), defaults.total());
}
