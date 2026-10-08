use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use kmipkit_test_support::EphemeralPki;

use super::config;
use super::current::pem;

const CREDENTIAL_PATH_SENTINEL: &str = "KMIP_CREDENTIAL_PATH_SENTINEL_F1B9";

#[test]
fn pem_file_sources_follow_symlinks_and_are_not_reread_after_source_construction() {
    let directory = TemporaryDirectory::new();
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let identity = pki.client_identity();
    let cert_target = directory.path().join("client-chain.pem");
    let key_target = directory.path().join("client-key.pem");
    let cert_link = directory.path().join("selected-cert.pem");
    let key_link = directory.path().join("selected-key.pem");
    let certificates = [
        pem("CERTIFICATE", identity.certificate_der()),
        pem("CERTIFICATE", pki.authority_certificate_der()),
    ]
    .concat();
    fs::write(&cert_target, certificates).expect("temporary certificate fixture is written");
    fs::write(&key_target, pem("PRIVATE KEY", identity.private_key_der()))
        .expect("temporary key fixture is written");
    let cert_source = selected_path(&cert_target, &cert_link);
    let key_source = selected_path(&key_target, &key_link);

    let certificate = config::CertificateInput::from_pem_file(&cert_source)
        .expect("the explicitly selected PEM certificate path is read");
    let private_key = config::PrivateKeyInput::from_pem_file(&key_source)
        .expect("the explicitly selected PEM key path is read");
    let path_diagnostics = format!("certificate={certificate:?} private_key={private_key:?}");
    assert!(!path_diagnostics.contains(CREDENTIAL_PATH_SENTINEL));

    if cert_source == cert_link {
        fs::remove_file(&cert_link).expect("selected certificate symlink is removed");
    }
    if key_source == key_link {
        fs::remove_file(&key_link).expect("selected key symlink is removed");
    }
    fs::remove_file(&cert_target).expect("certificate target is removed after source read");
    fs::remove_file(&key_target).expect("key target is removed after source read");

    let trust =
        config::TrustSource::certificate_authorities(vec![config::CertificateInput::from_der(
            vec![pki.authority_certificate_der().to_vec()],
        )]);
    let builder = config::TransportConfig::builder(config::Endpoint::raw_tls("localhost", 5696))
        .client_identity(config::ClientIdentity::new(certificate, private_key))
        .trust_source(trust);
    assert!(!format!("{builder:?}").contains(CREDENTIAL_PATH_SENTINEL));
    let built = builder
        .build()
        .expect("removed source paths are not reread");
    assert!(!format!("{built:?}").contains(CREDENTIAL_PATH_SENTINEL));
}

#[test]
fn der_file_sources_select_der_explicitly_and_file_errors_omit_the_path() {
    let directory = TemporaryDirectory::new();
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is available");
    let identity = pki.client_identity();
    let cert_path = directory.path().join("client.der");
    let key_path = directory.path().join("client-key.der");
    fs::write(&cert_path, identity.certificate_der())
        .expect("temporary leaf certificate is written");
    fs::write(&key_path, identity.private_key_der()).expect("temporary key fixture is written");

    let certificate = config::CertificateInput::from_der_file(&cert_path)
        .expect("the explicitly selected DER certificate path is read");
    let private_key = config::PrivateKeyInput::from_der_file(&key_path)
        .expect("the explicitly selected DER key path is read");
    fs::remove_file(&cert_path).expect("certificate source is removed after source read");
    fs::remove_file(&key_path).expect("key source is removed after source read");

    let error =
        config::PrivateKeyInput::from_pem_file(directory.path().join(CREDENTIAL_PATH_SENTINEL))
            .expect_err("a missing selected credential path is rejected");
    let diagnostics = format!("{error} {error:?} {}", public_error_chain(&error));
    assert!(!diagnostics.contains(CREDENTIAL_PATH_SENTINEL));

    let identity = config::ClientIdentity::new(certificate, private_key);
    let config = config::TransportConfig::builder(config::Endpoint::raw_tls("localhost", 5696))
        .client_identity(identity)
        .trust_source(config::TrustSource::certificate_authorities(vec![
            config::CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .build();
    assert!(config.is_ok());
}

fn public_error_chain(error: &dyn Error) -> String {
    let mut rendered = String::new();
    let mut current = Some(error);
    while let Some(cause) = current {
        let _ = writeln!(rendered, "{} {:?}", cause, cause);
        current = cause.source();
    }
    rendered
}

fn create_file_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(original, link)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(original, link)
    }
}

fn selected_path(original: &Path, link: &Path) -> PathBuf {
    match create_file_symlink(original, link) {
        Ok(()) => link.to_path_buf(),
        #[cfg(windows)]
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::Unsupported
            ) =>
        {
            original.to_path_buf()
        }
        Err(_) => panic!("selected path alias creation failed"),
    }
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new() -> Self {
        static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "kmipkit-{CREDENTIAL_PATH_SENTINEL}-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("unique temporary test directory is created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
