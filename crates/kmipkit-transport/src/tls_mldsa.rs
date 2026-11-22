//! Certificate signature policy for the OpenSSL provider, including ML-DSA.

use std::sync::{Arc, OnceLock};

use openssl::pkey::{KeyType, PKey, Private};
use openssl::sign::{Signer as OpenSslSigner, Verifier};
use rustls::crypto::{CryptoProvider, KeyProvider, WebPkiSupportedAlgorithms};
use rustls::pki_types::{
    AlgorithmIdentifier, InvalidSignature, PrivateKeyDer, SignatureVerificationAlgorithm,
    SubjectPublicKeyInfoDer, alg_id,
};
use rustls::sign::{Signer, SigningKey, public_key_to_spki};
use rustls::{Error, SignatureAlgorithm, SignatureScheme};
use zeroize::Zeroizing;

pub(super) fn extend(provider: &mut CryptoProvider) {
    static SIGNATURES: OnceLock<WebPkiSupportedAlgorithms> = OnceLock::new();
    static KEY_PROVIDER: OnceLock<MlDsaKeyProvider> = OnceLock::new();

    provider.signature_verification_algorithms = *SIGNATURES.get_or_init(|| {
        let mut all = preserve_rsa_key_limits(provider.signature_verification_algorithms.all);
        let mut mapping: Vec<_> = provider
            .signature_verification_algorithms
            .mapping
            .iter()
            .map(|&(scheme, algorithms)| {
                let guarded: &'static [&'static dyn SignatureVerificationAlgorithm] =
                    preserve_rsa_key_limits(algorithms).leak();
                (scheme, guarded)
            })
            .collect();
        all.extend(
            [&ML_DSA_44, &ML_DSA_65, &ML_DSA_87]
                .map(|alg| alg as &dyn SignatureVerificationAlgorithm),
        );
        mapping.extend([
            (SignatureScheme::ML_DSA_44, ML_DSA_44_VERIFICATION),
            (SignatureScheme::ML_DSA_65, ML_DSA_65_VERIFICATION),
            (SignatureScheme::ML_DSA_87, ML_DSA_87_VERIFICATION),
        ]);
        WebPkiSupportedAlgorithms {
            all: all.leak(),
            mapping: mapping.leak(),
        }
    });
    provider.key_provider = KEY_PROVIDER.get_or_init(|| MlDsaKeyProvider(provider.key_provider));
}

fn preserve_rsa_key_limits(
    algorithms: &[&'static dyn SignatureVerificationAlgorithm],
) -> Vec<&'static dyn SignatureVerificationAlgorithm> {
    algorithms
        .iter()
        .map(|&algorithm| {
            if algorithm.public_key_alg_id() == alg_id::RSA_ENCRYPTION {
                Box::leak(Box::new(RsaKeyLimits(algorithm))) as &dyn SignatureVerificationAlgorithm
            } else {
                algorithm
            }
        })
        .collect()
}

#[derive(Debug)]
struct RsaKeyLimits(&'static dyn SignatureVerificationAlgorithm);

impl SignatureVerificationAlgorithm for RsaKeyLimits {
    fn verify_signature(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), InvalidSignature> {
        let key = openssl::rsa::Rsa::public_key_from_der_pkcs1(public_key)
            .map_err(|_| InvalidSignature)?;
        // Match the RSA modulus range of Rustls' AWS-LC signature algorithms.
        if !(2048..=8192).contains(&key.n().num_bits()) {
            return Err(InvalidSignature);
        }
        self.0.verify_signature(public_key, message, signature)
    }

    fn public_key_alg_id(&self) -> AlgorithmIdentifier {
        self.0.public_key_alg_id()
    }

    fn signature_alg_id(&self) -> AlgorithmIdentifier {
        self.0.signature_alg_id()
    }

    fn fips(&self) -> bool {
        self.0.fips()
    }
}

#[derive(Debug)]
struct MlDsa {
    identifier: AlgorithmIdentifier,
    key_type: KeyType,
    scheme: SignatureScheme,
}

static ML_DSA_44: MlDsa = MlDsa {
    identifier: alg_id::ML_DSA_44,
    key_type: KeyType::ML_DSA_44,
    scheme: SignatureScheme::ML_DSA_44,
};
static ML_DSA_65: MlDsa = MlDsa {
    identifier: alg_id::ML_DSA_65,
    key_type: KeyType::ML_DSA_65,
    scheme: SignatureScheme::ML_DSA_65,
};
static ML_DSA_87: MlDsa = MlDsa {
    identifier: alg_id::ML_DSA_87,
    key_type: KeyType::ML_DSA_87,
    scheme: SignatureScheme::ML_DSA_87,
};

const ML_DSA_44_VERIFICATION: &[&dyn SignatureVerificationAlgorithm] = &[&ML_DSA_44];
const ML_DSA_65_VERIFICATION: &[&dyn SignatureVerificationAlgorithm] = &[&ML_DSA_65];
const ML_DSA_87_VERIFICATION: &[&dyn SignatureVerificationAlgorithm] = &[&ML_DSA_87];

impl SignatureVerificationAlgorithm for MlDsa {
    fn verify_signature(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), InvalidSignature> {
        let spki = public_key_to_spki(&self.identifier, public_key);
        let key = PKey::public_key_from_der(spki.as_ref()).map_err(|_| InvalidSignature)?;
        match Verifier::new_without_digest(&key)
            .and_then(|mut verifier| verifier.verify_oneshot(signature, message))
        {
            Ok(true) => Ok(()),
            _ => Err(InvalidSignature),
        }
    }

    fn public_key_alg_id(&self) -> AlgorithmIdentifier {
        self.identifier
    }

    fn signature_alg_id(&self) -> AlgorithmIdentifier {
        self.identifier
    }
}

#[derive(Debug)]
struct MlDsaKeyProvider(&'static dyn KeyProvider);

impl KeyProvider for MlDsaKeyProvider {
    fn load_private_key(
        &self,
        key_der: PrivateKeyDer<'static>,
    ) -> Result<Arc<dyn SigningKey>, Error> {
        if let PrivateKeyDer::Pkcs8(pkcs8) = &key_der
            && let Ok(key) = PKey::private_key_from_pkcs8(pkcs8.secret_pkcs8_der())
            && let Some(algorithm) = [&ML_DSA_44, &ML_DSA_65, &ML_DSA_87]
                .into_iter()
                .find(|algorithm| key.is_a(algorithm.key_type))
        {
            let _key_der = Zeroizing::new(key_der);
            let spki = key
                .public_key_to_der()
                .map_err(|_| Error::General("invalid ML-DSA public key".into()))?;
            return Ok(Arc::new(MlDsaSigningKey {
                key,
                algorithm,
                spki,
            }));
        }

        self.0.load_private_key(key_der)
    }
}

#[derive(Debug)]
struct MlDsaSigningKey {
    key: PKey<Private>,
    algorithm: &'static MlDsa,
    spki: Vec<u8>,
}

impl SigningKey for MlDsaSigningKey {
    fn choose_scheme(&self, offered: &[SignatureScheme]) -> Option<Box<dyn Signer>> {
        offered.contains(&self.algorithm.scheme).then(|| {
            Box::new(MlDsaSigner {
                key: self.key.clone(),
                scheme: self.algorithm.scheme,
            }) as Box<dyn Signer>
        })
    }

    fn public_key(&self) -> Option<SubjectPublicKeyInfoDer<'_>> {
        Some(SubjectPublicKeyInfoDer::from(self.spki.as_slice()))
    }

    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::Unknown(0)
    }
}

#[derive(Debug)]
struct MlDsaSigner {
    key: PKey<Private>,
    scheme: SignatureScheme,
}

impl Signer for MlDsaSigner {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
        OpenSslSigner::new_without_digest(&self.key)
            .and_then(|mut signer| signer.sign_oneshot_to_vec(message))
            .map_err(|_| Error::General("ML-DSA signing failed".into()))
    }

    fn scheme(&self) -> SignatureScheme {
        self.scheme
    }
}
