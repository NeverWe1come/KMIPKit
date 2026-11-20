use openssl::bn::BigNumContext;
use openssl::ec::{EcGroup, EcKey, PointConversionForm};
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::sign::Signer;
use rcgen::{
    CertificateParams, KeyIdMethod, PKCS_ECDSA_P256_SHA256, PublicKeyData, SerialNumber,
    SignatureAlgorithm, SigningKey,
};
use zeroize::Zeroizing;

/// OpenSSL signing key for ephemeral certificate fixtures.
pub struct FixtureSigningKey {
    key: PKey<Private>,
    private_der: Zeroizing<Vec<u8>>,
    public_point: Vec<u8>,
}

impl FixtureSigningKey {
    /// Generate a P-256 test key.
    ///
    /// # Errors
    ///
    /// Returns the OpenSSL error if key generation or encoding fails.
    pub fn generate() -> Result<Self, ErrorStack> {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
        let ec = EcKey::generate(&group)?;
        let mut context = BigNumContext::new()?;
        let public_point =
            ec.public_key()
                .to_bytes(&group, PointConversionForm::UNCOMPRESSED, &mut context)?;
        let key = PKey::from_ec_key(ec)?;
        let private_der = Zeroizing::new(key.private_key_to_pkcs8()?);
        Ok(Self {
            key,
            private_der,
            public_point,
        })
    }

    /// Copy the PKCS#8 encoding. The caller owns the copy.
    #[must_use]
    pub fn serialize_der(&self) -> Vec<u8> {
        self.private_der.to_vec()
    }

    /// Set the serial and subject key identifier normally supplied by rcgen's crypto feature.
    pub fn prepare_certificate(&self, parameters: &mut CertificateParams) {
        if parameters.serial_number.is_none() {
            let hash = openssl::sha::sha256(self.der_bytes());
            let mut serial = hash[..20].to_vec();
            serial[0] &= 0x7f;
            parameters.serial_number = Some(SerialNumber::from_slice(&serial));
        }
        parameters.key_identifier_method = self.key_identifier_method();
    }

    /// Derive the same truncated SHA-256 identifier as rcgen's crypto provider.
    #[must_use]
    pub fn key_identifier_method(&self) -> KeyIdMethod {
        let hash = openssl::sha::sha256(&self.subject_public_key_info());
        KeyIdMethod::PreSpecified(hash[..20].to_vec())
    }
}

impl PublicKeyData for FixtureSigningKey {
    fn der_bytes(&self) -> &[u8] {
        &self.public_point
    }

    fn algorithm(&self) -> &'static SignatureAlgorithm {
        &PKCS_ECDSA_P256_SHA256
    }
}

impl SigningKey for FixtureSigningKey {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, rcgen::Error> {
        let sign = || {
            let mut signer = Signer::new(MessageDigest::sha256(), &self.key)?;
            signer.update(message)?;
            signer.sign_to_vec()
        };
        sign().map_err(|_: ErrorStack| rcgen::Error::RemoteKeyError)
    }
}
