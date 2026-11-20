//! In-memory and file-exportable self-contained PKI for test servers.
#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use openssl::asn1::{Asn1Integer, Asn1Time};
use openssl::bn::BigNum;
use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::x509::extension::{
    BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName,
};
use openssl::x509::{X509, X509Builder, X509Name, X509NameBuilder};
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

/// Formats a DER payload into a PEM-encoded string with 64-character lines.
pub fn pem_encode(label: &str, der: &[u8]) -> String {
    const BASE64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = format!("-----BEGIN {label}-----\n");
    let mut line_length = 0;
    for chunk in der.chunks(3) {
        let bytes = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let encoded = [
            BASE64[(bytes[0] >> 2) as usize] as char,
            BASE64[(((bytes[0] & 3) << 4) | (bytes[1] >> 4)) as usize] as char,
            if chunk.len() > 1 {
                BASE64[(((bytes[1] & 15) << 2) | (bytes[2] >> 6)) as usize] as char
            } else {
                '='
            },
            if chunk.len() > 2 {
                BASE64[(bytes[2] & 63) as usize] as char
            } else {
                '='
            },
        ];
        for character in encoded {
            output.push(character);
            line_length += 1;
            if line_length == 64 {
                output.push('\n');
                line_length = 0;
            }
        }
    }
    if line_length > 0 {
        output.push('\n');
    }
    output.push_str(&format!("-----END {label}-----\n"));
    output
}

fn generate_key() -> Result<PKey<Private>, PkiError> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).map_err(|_| PkiError)?;
    let key = EcKey::generate(&group).map_err(|_| PkiError)?;
    PKey::from_ec_key(key).map_err(|_| PkiError)
}

fn common_name(value: &str) -> Result<X509Name, PkiError> {
    let mut name = X509NameBuilder::new().map_err(|_| PkiError)?;
    name.append_entry_by_nid(Nid::COMMONNAME, value)
        .map_err(|_| PkiError)?;
    Ok(name.build())
}

fn serial_number(value: u32) -> Result<Asn1Integer, PkiError> {
    let number = BigNum::from_u32(value).map_err(|_| PkiError)?;
    Asn1Integer::from_bn(&number).map_err(|_| PkiError)
}

fn set_validity(builder: &mut X509Builder) -> Result<(), PkiError> {
    let not_before = Asn1Time::days_from_now(0).map_err(|_| PkiError)?;
    let not_after = Asn1Time::days_from_now(3650).map_err(|_| PkiError)?;
    builder.set_not_before(&not_before).map_err(|_| PkiError)?;
    builder.set_not_after(&not_after).map_err(|_| PkiError)
}

fn create_ca(key: &PKey<Private>) -> Result<X509, PkiError> {
    let name = common_name("KMIPKit Test Server Authority")?;
    let mut builder = X509::builder().map_err(|_| PkiError)?;
    builder.set_version(2).map_err(|_| PkiError)?;
    let serial = serial_number(1)?;
    builder.set_serial_number(&serial).map_err(|_| PkiError)?;
    builder.set_subject_name(&name).map_err(|_| PkiError)?;
    builder.set_issuer_name(&name).map_err(|_| PkiError)?;
    builder.set_pubkey(key).map_err(|_| PkiError)?;
    set_validity(&mut builder)?;

    let constraints = {
        BasicConstraints::new()
            .critical()
            .ca()
            .pathlen(0)
            .build()
            .map_err(|_| PkiError)?
    };
    builder
        .append_extension(constraints)
        .map_err(|_| PkiError)?;
    let key_usage = KeyUsage::new()
        .critical()
        .key_cert_sign()
        .crl_sign()
        .build()
        .map_err(|_| PkiError)?;
    builder.append_extension(key_usage).map_err(|_| PkiError)?;
    builder
        .sign(key, MessageDigest::sha256())
        .map_err(|_| PkiError)?;
    Ok(builder.build())
}

fn create_leaf(
    key: &PKey<Private>,
    ca: &X509,
    ca_key: &PKey<Private>,
    common_name_value: &str,
    dns_names: &[&str],
    ip_addresses: &[&str],
    serial: u32,
    server_auth: bool,
) -> Result<X509, PkiError> {
    let name = common_name(common_name_value)?;
    let mut builder = X509::builder().map_err(|_| PkiError)?;
    builder.set_version(2).map_err(|_| PkiError)?;
    let serial_number = serial_number(serial)?;
    builder
        .set_serial_number(&serial_number)
        .map_err(|_| PkiError)?;
    builder.set_subject_name(&name).map_err(|_| PkiError)?;
    builder
        .set_issuer_name(ca.subject_name())
        .map_err(|_| PkiError)?;
    builder.set_pubkey(key).map_err(|_| PkiError)?;
    set_validity(&mut builder)?;

    let subject_alt_name = {
        let context = builder.x509v3_context(Some(ca), None);
        let mut names = SubjectAlternativeName::new();
        for name in dns_names {
            names.dns(name);
        }
        for address in ip_addresses {
            names.ip(address);
        }
        names.build(&context).map_err(|_| PkiError)?
    };
    builder
        .append_extension(subject_alt_name)
        .map_err(|_| PkiError)?;

    let key_usage = KeyUsage::new()
        .critical()
        .digital_signature()
        .key_encipherment()
        .build()
        .map_err(|_| PkiError)?;
    builder.append_extension(key_usage).map_err(|_| PkiError)?;
    let extended_key_usage = if server_auth {
        ExtendedKeyUsage::new().server_auth().build()
    } else {
        ExtendedKeyUsage::new().client_auth().build()
    }
    .map_err(|_| PkiError)?;
    builder
        .append_extension(extended_key_usage)
        .map_err(|_| PkiError)?;
    builder
        .sign(ca_key, MessageDigest::sha256())
        .map_err(|_| PkiError)?;
    Ok(builder.build())
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
        let ca_key = generate_key()?;
        let ca_cert = create_ca(&ca_key)?;
        let ca_cert_der = ca_cert.to_der().map_err(|_| PkiError)?;
        let ca_cert_pem =
            String::from_utf8(ca_cert.to_pem().map_err(|_| PkiError)?).map_err(|_| PkiError)?;

        let server_key = generate_key()?;
        let server_cert = create_leaf(
            &server_key,
            &ca_cert,
            &ca_key,
            "localhost",
            &["localhost", "server.kmipkit.test"],
            &["127.0.0.1"],
            2,
            true,
        )?;
        let server_cert_der = server_cert.to_der().map_err(|_| PkiError)?;
        let server_key_der =
            Zeroizing::new(server_key.private_key_to_pkcs8().map_err(|_| PkiError)?);
        let server_cert_pem =
            String::from_utf8(server_cert.to_pem().map_err(|_| PkiError)?).map_err(|_| PkiError)?;
        let server_key_pem = String::from_utf8(
            server_key
                .private_key_to_pem_pkcs8()
                .map_err(|_| PkiError)?,
        )
        .map_err(|_| PkiError)?;

        let client_key = generate_key()?;
        let client_cert = create_leaf(
            &client_key,
            &ca_cert,
            &ca_key,
            "kmipkit-test-client",
            &["client.kmipkit.test"],
            &[],
            3,
            false,
        )?;
        let client_cert_der = client_cert.to_der().map_err(|_| PkiError)?;
        let client_key_der =
            Zeroizing::new(client_key.private_key_to_pkcs8().map_err(|_| PkiError)?);
        let client_cert_pem =
            String::from_utf8(client_cert.to_pem().map_err(|_| PkiError)?).map_err(|_| PkiError)?;
        let client_key_pem = String::from_utf8(
            client_key
                .private_key_to_pem_pkcs8()
                .map_err(|_| PkiError)?,
        )
        .map_err(|_| PkiError)?;

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
