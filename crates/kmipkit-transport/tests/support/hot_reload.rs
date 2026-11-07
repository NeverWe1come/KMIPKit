//! Shared test fixtures and utilities for certificate hot-reload tests.

#![allow(dead_code)]

use std::sync::Arc;

use kmipkit_test_support::EphemeralPki;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};

pub struct TempFile {
    pub path: std::path::PathBuf,
}

impl TempFile {
    pub fn write(bytes: &[u8]) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("kmipkit-hotreload-{}-{id}.pem", std::process::id()));
        std::fs::write(&path, bytes).expect("write temp file");
        Self { path }
    }

    pub fn overwrite(&self, bytes: &[u8]) {
        std::fs::write(&self.path, bytes).expect("overwrite temp file");
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn pem(label: &str, bytes: &[u8]) -> Vec<u8> {
    const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = format!("-----BEGIN {label}-----\n").into_bytes();
    for c in bytes.chunks(3) {
        let b = [c[0], *c.get(1).unwrap_or(&0), *c.get(2).unwrap_or(&0)];
        out.push(B64[(b[0] >> 2) as usize]);
        out.push(B64[(((b[0] & 3) << 4) | (b[1] >> 4)) as usize]);
        out.push(if c.len() > 1 {
            B64[(((b[1] & 15) << 2) | (b[2] >> 6)) as usize]
        } else {
            b'='
        });
        out.push(if c.len() > 2 {
            B64[(b[2] & 63) as usize]
        } else {
            b'='
        });
    }
    out.extend_from_slice(format!("\n-----END {label}-----\n").as_bytes());
    out
}

pub fn server_config_with_multiple_roots(
    pki: &EphemeralPki,
    additional_ca_roots: &[&[u8]],
) -> Arc<ServerConfig> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the local server uses TLS 1.3 only");
    let mut client_roots = RootCertStore::empty();
    client_roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("the client test CA is valid");
    for root in additional_ca_roots {
        client_roots
            .add(CertificateDer::from(root.to_vec()))
            .expect("the additional client test CA is valid");
    }
    let verifier = WebPkiClientVerifier::builder(Arc::new(client_roots))
        .build()
        .expect("the peer requires a verified client certificate");
    let config = builder
        .with_client_cert_verifier(verifier)
        .with_single_cert(
            vec![CertificateDer::from(
                pki.server_identity().certificate_der().to_vec(),
            )],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                pki.server_identity().private_key_der().to_vec(),
            )),
        )
        .expect("the local server identity is valid");
    Arc::new(config)
}
