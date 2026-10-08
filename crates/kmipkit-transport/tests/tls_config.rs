//! Derived tests for KMIPKIT-0013 FR-003, FR-004, FR-009, FR-011, and FR-017.
//!
//! The default HTTPS target also verifies OASIS KMIP Profiles v2.1 OS §5.3.1
//! item 3 (`SHOULD`), while caller-selected origin-form targets cover item 4.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use kmipkit_test_support::EphemeralPki;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, HttpsTransport, PrivateKeyInput, RawTlsTransport,
    RequestOptions, RevocationListInput, TimeoutLimit, TimeoutPolicy, TransportConfig,
    TransportConfigBuilder, TransportConfigError, TrustSource,
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
        "https://kmip.example.test:",
        "https://kmip.example.test:0",
        "https://kmip.example.test:65536",
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
        "/path with whitespace",
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
fn empty_and_malformed_revocation_list_inputs_are_rejected() {
    let cases = [
        ("empty PEM", RevocationListInput::from_pem(Vec::new())),
        (
            "malformed PEM",
            RevocationListInput::from_pem(b"not a PEM revocation list".to_vec()),
        ),
        ("empty DER list", RevocationListInput::from_der(Vec::new())),
        (
            "empty DER entry",
            RevocationListInput::from_der(vec![Vec::new()]),
        ),
    ];

    for (name, revocation_list) in cases {
        let result = valid_builder(Endpoint::raw_tls("localhost", 5696), false)
            .revocation_lists(vec![revocation_list])
            .build();
        assert!(
            matches!(result, Err(TransportConfigError::InvalidRevocationList)),
            "{name} must be rejected as an invalid revocation list"
        );
    }
}

#[test]
fn empty_certificate_and_empty_caller_trust_are_rejected() {
    let pki = EphemeralPki::generate().expect("ephemeral client PKI should be generated");
    let client = pki.client_identity();
    let missing_certificate = ClientIdentity::new(
        CertificateInput::from_pem(Vec::new()),
        PrivateKeyInput::from_der(client.private_key_der().to_vec()),
    );
    let result = TransportConfig::builder(Endpoint::raw_tls("localhost", 5696))
        .client_identity(missing_certificate)
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .build();
    assert!(matches!(
        result,
        Err(TransportConfigError::InvalidCredential)
    ));

    let (identity, _) = identity_and_trust(false);
    let result = TransportConfig::builder(Endpoint::raw_tls("localhost", 5696))
        .client_identity(identity)
        .trust_source(TrustSource::certificate_authorities(Vec::new()))
        .build();
    assert!(matches!(result, Err(TransportConfigError::InvalidTrust)));
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

#[test]
fn write_timeout_overrides_remain_distinct_from_unbounded_and_other_phases() {
    let defaults = TimeoutPolicy::default();
    let policy = defaults.with_write(TimeoutLimit::Unbounded);
    assert_eq!(policy.connect(), defaults.connect());
    assert_eq!(policy.write(), TimeoutLimit::Unbounded);
    assert_eq!(policy.read(), defaults.read());
    assert_eq!(policy.total(), defaults.total());

    let options = RequestOptions::default().with_write(TimeoutLimit::Bounded(Duration::ZERO));
    let effective = defaults.with_overrides(&options);
    assert_eq!(effective.write(), TimeoutLimit::Bounded(Duration::ZERO));
}

#[test]
fn bracketed_ipv6_endpoint_adapters_construct_without_network_activity() {
    let raw = valid_builder(Endpoint::raw_tls("[::1]", 5696), false)
        .build()
        .expect("bracketed IPv6 raw-TLS endpoint should validate");
    RawTlsTransport::new(raw).expect("raw-TLS adapter should normalize bracketed IPv6");

    let https = valid_builder(Endpoint::https("https://[::1]"), false)
        .build()
        .expect("bracketed IPv6 HTTPS endpoint should validate");
    HttpsTransport::new(https).expect("HTTPS adapter should normalize bracketed IPv6");
}

#[test]
fn adapters_reject_configurations_for_the_other_transport_scheme() {
    let raw = valid_builder(Endpoint::raw_tls("localhost", 5696), false)
        .build()
        .expect("raw TLS configuration should validate");
    assert!(matches!(
        HttpsTransport::new(raw),
        Err(TransportConfigError::InvalidEndpoint)
    ));

    let https = valid_builder(Endpoint::https("https://localhost:5697"), false)
        .build()
        .expect("HTTPS configuration should validate");
    assert!(matches!(
        RawTlsTransport::new(https),
        Err(TransportConfigError::InvalidEndpoint)
    ));
}

#[test]
fn every_public_configuration_error_has_a_fixed_safe_display() {
    let cases = [
        (
            TransportConfigError::InvalidEndpoint,
            "invalid transport endpoint",
        ),
        (
            TransportConfigError::InvalidTarget,
            "invalid HTTPS request target",
        ),
        (
            TransportConfigError::InvalidRequestLimit,
            "request limit must be positive",
        ),
        (
            TransportConfigError::MissingIdentity,
            "client identity is required",
        ),
        (
            TransportConfigError::MissingTrust,
            "an explicit trust source is required",
        ),
        (
            TransportConfigError::InvalidCredential,
            "client credential input is invalid",
        ),
        (
            TransportConfigError::InvalidTrust,
            "trust certificates are invalid",
        ),
        (
            TransportConfigError::InvalidRevocationList,
            "certificate revocation list input is invalid",
        ),
        (
            TransportConfigError::InvalidServerName,
            "TLS verification name is invalid",
        ),
        (
            TransportConfigError::InvalidTlsConfiguration,
            "TLS configuration is invalid",
        ),
        (
            TransportConfigError::PlatformTrustUnavailable,
            "platform trust is unavailable",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn pem_and_der_credential_file_inputs_build_valid_transport_configs() {
    let pki = EphemeralPki::generate().expect("ephemeral client PKI should be generated");
    let client = pki.client_identity();
    let client_certificate = client
        .certificate_chain_der()
        .into_iter()
        .next()
        .expect("client identity should have a leaf certificate");

    let pem_certificate = TemporaryCredentialFile::write(&pem("CERTIFICATE", client_certificate));
    let pem_key = TemporaryCredentialFile::write(&pem("PRIVATE KEY", client.private_key_der()));
    let pem_authority =
        TemporaryCredentialFile::write(&pem("CERTIFICATE", pki.authority_certificate_der()));
    let pem_config = TransportConfig::builder(Endpoint::raw_tls("localhost", 5696))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file(&pem_certificate.path)
                .expect("PEM client certificate should be read"),
            PrivateKeyInput::from_pem_file(&pem_key.path).expect("PEM private key should be read"),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file(&pem_authority.path)
                .expect("PEM trust root should be read"),
        ]))
        .build()
        .expect("PEM credential files should build a config");
    assert_eq!(pem_config.target_uri(), None);

    let der_certificate = TemporaryCredentialFile::write(client_certificate);
    let der_key = TemporaryCredentialFile::write(client.private_key_der());
    let der_authority = TemporaryCredentialFile::write(pki.authority_certificate_der());
    let der_config = TransportConfig::builder(Endpoint::raw_tls("localhost", 5696))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der_file(&der_certificate.path)
                .expect("DER client certificate should be read"),
            PrivateKeyInput::from_der_file(&der_key.path).expect("DER private key should be read"),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der_file(&der_authority.path)
                .expect("DER trust root should be read"),
        ]))
        .build()
        .expect("DER credential files should build a config");
    assert_eq!(der_config.target_uri(), None);
}

#[test]
fn all_credential_file_read_failures_use_the_redacted_invalid_credential_category() {
    let path = TemporaryCredentialFile::missing_path();

    assert!(matches!(
        CertificateInput::from_pem_file(&path),
        Err(TransportConfigError::InvalidCredential)
    ));
    assert!(matches!(
        CertificateInput::from_der_file(&path),
        Err(TransportConfigError::InvalidCredential)
    ));
    assert!(matches!(
        PrivateKeyInput::from_pem_file(&path),
        Err(TransportConfigError::InvalidCredential)
    ));
    assert!(matches!(
        PrivateKeyInput::from_der_file(&path),
        Err(TransportConfigError::InvalidCredential)
    ));
}

#[test]
fn configuration_debug_redacts_every_credential_endpoint_and_target_shape() {
    let certificate_sentinel = "certificate-debug-sentinel";
    let key_sentinel = "private-key-debug-sentinel";
    let revocation_sentinel = "revocation-debug-sentinel";
    let endpoint_sentinel = "endpoint-debug-sentinel";
    let target_sentinel = "target-debug-sentinel";
    let name_sentinel = "server-name-debug-sentinel";

    for endpoint in [
        Endpoint::raw_tls(endpoint_sentinel, 5696),
        Endpoint::https(format!("https://{endpoint_sentinel}:5696")),
    ] {
        let debug = format!("{endpoint:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains(endpoint_sentinel));
    }

    let pem_certificate = CertificateInput::from_pem(certificate_sentinel.as_bytes().to_vec());
    let der_certificate =
        CertificateInput::from_der(vec![certificate_sentinel.as_bytes().to_vec()]);
    let pem_key = PrivateKeyInput::from_pem(key_sentinel.as_bytes().to_vec());
    let der_key = PrivateKeyInput::from_der(key_sentinel.as_bytes().to_vec());
    let pem_revocation = RevocationListInput::from_pem(revocation_sentinel.as_bytes().to_vec());
    let der_revocation =
        RevocationListInput::from_der(vec![revocation_sentinel.as_bytes().to_vec()]);

    for debug in [
        format!("{pem_certificate:?}"),
        format!("{der_certificate:?}"),
        format!("{pem_key:?}"),
        format!("{der_key:?}"),
        format!("{pem_revocation:?}"),
        format!("{der_revocation:?}"),
        format!(
            "{:?}",
            TrustSource::certificate_authorities(vec![CertificateInput::from_pem(
                certificate_sentinel.as_bytes().to_vec()
            ),])
        ),
        format!("{:?}", TrustSource::platform()),
        format!(
            "{:?}",
            ClientIdentity::new(
                CertificateInput::from_der(vec![certificate_sentinel.as_bytes().to_vec()]),
                PrivateKeyInput::from_der(key_sentinel.as_bytes().to_vec()),
            )
        ),
    ] {
        assert!(!debug.contains(certificate_sentinel));
        assert!(!debug.contains(key_sentinel));
        assert!(!debug.contains(revocation_sentinel));
    }

    let builder =
        TransportConfig::builder(Endpoint::https(format!("https://{endpoint_sentinel}:5696")))
            .target_uri(format!("/{target_sentinel}?token={key_sentinel}"))
            .tls_server_name(name_sentinel)
            .client_identity(ClientIdentity::new(
                CertificateInput::from_pem(certificate_sentinel.as_bytes().to_vec()),
                PrivateKeyInput::from_pem(key_sentinel.as_bytes().to_vec()),
            ))
            .trust_source(TrustSource::certificate_authorities(vec![
                CertificateInput::from_pem(certificate_sentinel.as_bytes().to_vec()),
            ]))
            .revocation_lists(vec![RevocationListInput::from_pem(
                revocation_sentinel.as_bytes().to_vec(),
            )]);
    let builder_debug = format!("{builder:?}");
    for sentinel in [
        endpoint_sentinel,
        target_sentinel,
        key_sentinel,
        name_sentinel,
        certificate_sentinel,
        revocation_sentinel,
    ] {
        assert!(!builder_debug.contains(sentinel));
    }

    let config = valid_builder(Endpoint::https("https://127.0.0.1:5696"), false)
        .target_uri(format!("/{target_sentinel}?token={key_sentinel}"))
        .tls_server_name(name_sentinel)
        .build()
        .expect("valid HTTPS configuration should build");
    let config_debug = format!("{config:?}");
    for sentinel in [target_sentinel, key_sentinel, name_sentinel] {
        assert!(!config_debug.contains(sentinel));
    }
    assert!(config_debug.contains("[REDACTED]"));
}

struct TemporaryCredentialFile {
    path: PathBuf,
}

impl TemporaryCredentialFile {
    fn write(bytes: &[u8]) -> Self {
        static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);
        let sequence = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "kmipkit-transport-credential-{}-{sequence}.tmp",
            std::process::id()
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("temporary credential file should be created uniquely");
        file.write_all(bytes)
            .expect("temporary credential bytes should be written");
        Self { path }
    }

    fn missing_path() -> PathBuf {
        static NEXT_MISSING: AtomicUsize = AtomicUsize::new(0);
        let sequence = NEXT_MISSING.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "kmipkit-transport-credential-missing-{}-{sequence}.tmp",
            std::process::id()
        ))
    }
}

impl Drop for TemporaryCredentialFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
