//! In-memory and file-exportable self-contained PKI for test servers.

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose,
};
use zeroize::Zeroizing;

/// Error returned when ephemeral PKI creation fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PkiError;

impl fmt::Display for PkiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ephemeral PKI generation failed")
    }
}

impl Error for PkiError {}

/// Formats a DER payload into a PEM-encoded string with 64-char lines.
pub fn pem_encode(label: &str, der: &[u8]) -> String {
    const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = format!("-----BEGIN {label}-----\n");
    let mut line_len = 0;
    for chunk in der.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let chars = [
            B64[(b[0] >> 2) as usize] as char,
            B64[(((b[0] & 3) << 4) | (b[1] >> 4)) as usize] as char,
            if chunk.len() > 1 {
                B64[(((b[1] & 15) << 2) | (b[2] >> 6)) as usize] as char
            } else {
                '='
            },
            if chunk.len() > 2 {
                B64[(b[2] & 63) as usize] as char
            } else {
                '='
            },
        ];
        for c in chars {
            out.push(c);
            line_len += 1;
            if line_len == 64 {
                out.push('\n');
                line_len = 0;
            }
        }
    }
    if line_len > 0 {
        out.push('\n');
    }
    out.push_str(&format!("-----END {label}-----\n"));
    out
}

/// Fully autonomous PKI containing root CA, server TLS, and client mTLS identities.
pub struct AutoPki {
    pub ca_cert_pem: String,
    pub server_cert_pem: String,
    pub server_key_pem: String,
    pub client_cert_pem: String,
    pub client_key_pem: String,
    pub ca_cert_der: Vec<u8>,
    pub server_cert_der: Vec<u8>,
    pub server_key_der: Zeroizing<Vec<u8>>,
    pub client_cert_der: Vec<u8>,
    pub client_key_der: Zeroizing<Vec<u8>>,
}

impl AutoPki {
    /// Generates a complete fresh in-memory PKI.
    ///
    /// # Errors
    ///
    /// Returns [`PkiError`] if keypair or certificate generation fails.
    pub fn generate() -> Result<Self, PkiError> {
        let ca_key = KeyPair::generate().map_err(|_| PkiError)?;
        let mut ca_params = CertificateParams::default();
        ca_params
            .distinguished_name
            .push(DnType::CommonName, "KMIPKit Test Server Authority");
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        let ca_cert = ca_params.self_signed(&ca_key).map_err(|_| PkiError)?;
        let ca_cert_der = ca_cert.der().as_ref().to_vec();
        let ca_cert_pem = pem_encode("CERTIFICATE", &ca_cert_der);
        let issuer = Issuer::from_params(&ca_params, &ca_key);

        // Server TLS Identity (SAN includes localhost and 127.0.0.1)
        let server_key = KeyPair::generate().map_err(|_| PkiError)?;
        let mut server_params = CertificateParams::new(vec![
            "localhost".to_owned(),
            "127.0.0.1".to_owned(),
            "server.kmipkit.test".to_owned(),
        ])
        .map_err(|_| PkiError)?;
        server_params
            .distinguished_name
            .push(DnType::CommonName, "localhost");
        server_params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        server_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        let server_cert: Certificate = server_params
            .signed_by(&server_key, &issuer)
            .map_err(|_| PkiError)?;
        let server_cert_der = server_cert.der().as_ref().to_vec();
        let server_key_der = Zeroizing::new(server_key.serialize_der());
        let server_cert_pem = pem_encode("CERTIFICATE", &server_cert_der);
        let server_key_pem = pem_encode("PRIVATE KEY", &server_key_der);

        // Client mTLS Identity
        let client_key = KeyPair::generate().map_err(|_| PkiError)?;
        let mut client_params =
            CertificateParams::new(vec!["client.kmipkit.test".to_owned()]).map_err(|_| PkiError)?;
        client_params
            .distinguished_name
            .push(DnType::CommonName, "kmipkit-test-client");
        client_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        client_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
        let client_cert: Certificate = client_params
            .signed_by(&client_key, &issuer)
            .map_err(|_| PkiError)?;
        let client_cert_der = client_cert.der().as_ref().to_vec();
        let client_key_der = Zeroizing::new(client_key.serialize_der());
        let client_cert_pem = pem_encode("CERTIFICATE", &client_cert_der);
        let client_key_pem = pem_encode("PRIVATE KEY", &client_key_der);

        Ok(Self {
            ca_cert_pem,
            server_cert_pem,
            server_key_pem,
            client_cert_pem,
            client_key_pem,
            ca_cert_der,
            server_cert_der,
            server_key_der,
            client_cert_der,
            client_key_der,
        })
    }

    /// Exports all certificates and keys as PEM files to the chosen directory.
    ///
    /// # Errors
    ///
    /// Returns an IO error if directory creation or file writing fails.
    pub fn export_to_directory(&self, dir: &Path) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        fs::write(dir.join("ca.crt"), &self.ca_cert_pem)?;
        fs::write(dir.join("server.crt"), &self.server_cert_pem)?;
        fs::write(dir.join("server.key"), &self.server_key_pem)?;
        fs::write(dir.join("client.crt"), &self.client_cert_pem)?;
        fs::write(dir.join("client.key"), &self.client_key_pem)?;
        Ok(())
    }
}
