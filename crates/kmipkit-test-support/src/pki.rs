//! Ephemeral certificate fixtures for tests that exercise verified TLS.

use std::error::Error;
use std::fmt;

use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose,
};
use zeroize::Zeroizing;

/// A short, redacted error returned while creating an ephemeral test PKI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PkiFixtureError;

impl fmt::Display for PkiFixtureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ephemeral test PKI generation failed")
    }
}

impl Error for PkiFixtureError {}

/// A generated root CA with distinct client and server identities.
pub struct EphemeralPki {
    authority_certificate_der: Vec<u8>,
    client_identity: EphemeralIdentity,
    server_identity: EphemeralIdentity,
}

impl EphemeralPki {
    /// Creates a fresh root CA and fresh client and server credentials.
    ///
    /// The credentials are generated for tests only and are never loaded from
    /// files or shared with an external service.
    ///
    /// # Errors
    ///
    /// Returns a redacted error if AWS-LC key generation or certificate
    /// construction fails.
    pub fn generate() -> Result<Self, PkiFixtureError> {
        let authority_key = KeyPair::generate().map_err(|_| PkiFixtureError)?;
        let mut authority_params = CertificateParams::default();
        authority_params
            .distinguished_name
            .push(DnType::CommonName, "KMIPKit ephemeral test authority");
        authority_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        authority_params.key_usages = vec![KeyUsagePurpose::KeyCertSign];
        let authority_certificate = authority_params
            .self_signed(&authority_key)
            .map_err(|_| PkiFixtureError)?;
        let authority_certificate_der = authority_certificate.der().as_ref().to_vec();
        let issuer = Issuer::from_params(&authority_params, &authority_key);

        let client_identity = generate_identity(
            "client.kmipkit.test",
            ExtendedKeyUsagePurpose::ClientAuth,
            &issuer,
            &authority_certificate_der,
        )?;
        let server_identity = generate_identity(
            "server.kmipkit.test",
            ExtendedKeyUsagePurpose::ServerAuth,
            &issuer,
            &authority_certificate_der,
        )?;

        Ok(Self {
            authority_certificate_der,
            client_identity,
            server_identity,
        })
    }

    /// Returns the public DER encoding of the generated trust root.
    #[must_use]
    pub fn authority_certificate_der(&self) -> &[u8] {
        &self.authority_certificate_der
    }

    /// Returns the generated client identity.
    #[must_use]
    pub fn client_identity(&self) -> &EphemeralIdentity {
        &self.client_identity
    }

    /// Returns the generated server identity.
    #[must_use]
    pub fn server_identity(&self) -> &EphemeralIdentity {
        &self.server_identity
    }
}

impl fmt::Debug for EphemeralPki {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EphemeralPki")
            .field(
                "authority_certificate_bytes",
                &self.authority_certificate_der.len(),
            )
            .field("client_identity", &self.client_identity)
            .field("server_identity", &self.server_identity)
            .finish()
    }
}

/// One ephemeral certificate and its zeroizing PKCS#8 private-key bytes.
pub struct EphemeralIdentity {
    certificate: Vec<u8>,
    authority_certificate: Vec<u8>,
    private_key: Zeroizing<Vec<u8>>,
}

impl EphemeralIdentity {
    /// Returns the public DER encoding of the leaf certificate.
    #[must_use]
    pub fn certificate_der(&self) -> &[u8] {
        &self.certificate
    }

    /// Returns the public certificate chain in leaf-then-root order.
    #[must_use]
    pub fn certificate_chain_der(&self) -> [&[u8]; 2] {
        [&self.certificate, &self.authority_certificate]
    }

    /// Returns this test identity's private key in PKCS#8 DER format.
    ///
    /// The returned borrow does not transfer ownership; the fixture zeroizes
    /// its owned key bytes when dropped.
    #[must_use]
    pub fn private_key_der(&self) -> &[u8] {
        &self.private_key
    }
}

impl fmt::Debug for EphemeralIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EphemeralIdentity")
            .field("certificate_bytes", &self.certificate.len())
            .field(
                "authority_certificate_bytes",
                &self.authority_certificate.len(),
            )
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

fn generate_identity(
    common_name: &str,
    extended_key_usage: ExtendedKeyUsagePurpose,
    issuer: &Issuer<'_, &KeyPair>,
    authority_certificate_der: &[u8],
) -> Result<EphemeralIdentity, PkiFixtureError> {
    let key_pair = KeyPair::generate().map_err(|_| PkiFixtureError)?;
    let mut params =
        CertificateParams::new(vec![common_name.to_owned()]).map_err(|_| PkiFixtureError)?;
    params
        .distinguished_name
        .push(DnType::CommonName, common_name);
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    params.extended_key_usages = vec![extended_key_usage];
    let certificate: Certificate = params
        .signed_by(&key_pair, issuer)
        .map_err(|_| PkiFixtureError)?;

    Ok(EphemeralIdentity {
        certificate: certificate.der().as_ref().to_vec(),
        authority_certificate: authority_certificate_der.to_vec(),
        private_key: Zeroizing::new(key_pair.serialize_der()),
    })
}
