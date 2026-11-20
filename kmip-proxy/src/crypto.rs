//! OpenSSL-backed cryptographic primitives shared by the proxy.
#![forbid(unsafe_code)]

#[cfg(any(
    feature = "azure",
    feature = "gcp",
    feature = "oci",
    feature = "vault",
    feature = "infisical",
    feature = "ibm",
    feature = "alibaba",
    feature = "scaleway",
    feature = "otc"
))]
use std::sync::OnceLock;

use openssl::pkey::PKey;
use openssl::rand::rand_bytes;
use openssl::sha::sha256;
use openssl::sign::{Signer, Verifier};
use zeroize::Zeroizing;

use crate::ProxyError;

#[cfg(any(
    feature = "azure",
    feature = "gcp",
    feature = "oci",
    feature = "vault",
    feature = "infisical",
    feature = "ibm",
    feature = "alibaba",
    feature = "scaleway",
    feature = "otc"
))]
static RUSTLS_PROVIDER: OnceLock<Result<(), String>> = OnceLock::new();

/// Installs OpenSSL as the process-wide rustls crypto provider.
///
/// This is required by Azure's rustls client, which is built without a
/// provider so it can use the same OpenSSL-backed implementation as the
/// proxy listener.
#[cfg(any(
    feature = "azure",
    feature = "gcp",
    feature = "oci",
    feature = "vault",
    feature = "infisical",
    feature = "ibm",
    feature = "alibaba",
    feature = "scaleway",
    feature = "otc"
))]
pub fn install_rustls_openssl_provider() -> Result<(), ProxyError> {
    RUSTLS_PROVIDER
        .get_or_init(|| {
            rustls_openssl::default_provider()
                .install_default()
                .map_err(|_| "another rustls crypto provider is already installed".to_owned())
        })
        .clone()
        .map_err(ProxyError::CryptoError)
}

/// Fills a buffer with cryptographically secure random bytes from OpenSSL.
pub fn fill_random_bytes(buffer: &mut [u8]) -> Result<(), openssl::error::ErrorStack> {
    rand_bytes(buffer)
}

/// Generates a fresh AES-GCM nonce using OpenSSL's cryptographic RNG.
pub fn generate_gcm_nonce() -> Result<[u8; 12], openssl::error::ErrorStack> {
    let mut nonce = [0u8; 12];
    rand_bytes(&mut nonce)?;
    Ok(nonce)
}

/// Generates a hex-encoded random identifier with the given prefix.
#[cfg(any(feature = "vault", feature = "infisical", feature = "ibm", feature = "alibaba"))]
pub fn random_identifier(prefix: &str) -> Result<String, ProxyError> {
    let mut random = [0u8; 16];
    fill_random_bytes(&mut random)
        .map_err(|error| ProxyError::CryptoError(format!("entropy generation failed: {error}")))?;
    Ok(format!("{prefix}{}", hex::encode(random)))
}

/// Computes an HMAC-SHA256 value.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>, openssl::error::ErrorStack> {
    let key = PKey::hmac(key)?;
    let mut signer = Signer::new(openssl::hash::MessageDigest::sha256(), &key)?;
    signer.update(data)?;
    signer.sign_to_vec()
}

/// Verifies an HMAC-SHA256 value using OpenSSL's constant-time verifier.
pub fn verify_hmac_sha256(
    key: &[u8],
    data: &[u8],
    signature: &[u8],
) -> Result<bool, openssl::error::ErrorStack> {
    let key = PKey::hmac(key)?;
    let mut verifier = Verifier::new(openssl::hash::MessageDigest::sha256(), &key)?;
    verifier.update(data)?;
    verifier.verify(signature)
}

/// Computes a SHA-256 digest.
#[must_use]
#[cfg(feature = "gcp")]
pub fn sha256_digest(data: &[u8]) -> [u8; 32] {
    sha256(data)
}

/// Wraps bytes with AES-256-GCM and returns ciphertext followed by its tag.
pub fn aes_256_gcm_encrypt(
    key: &[u8],
    nonce: &[u8; 12],
    plaintext: &[u8],
) -> Result<Vec<u8>, openssl::error::ErrorStack> {
    use openssl::symm::{Cipher, Crypter, Mode};

    let mut crypter = Crypter::new(Cipher::aes_256_gcm(), Mode::Encrypt, key, Some(nonce))?;
    crypter.pad(false);
    let mut output = vec![0; plaintext.len() + Cipher::aes_256_gcm().block_size() + 16];
    let mut written = crypter.update(plaintext, &mut output)?;
    written += crypter.finalize(&mut output[written..])?;
    crypter.get_tag(&mut output[written..written + 16])?;
    output.truncate(written + 16);
    Ok(output)
}

/// Decrypts and authenticates ciphertext followed by its AES-256-GCM tag.
pub fn aes_256_gcm_decrypt(
    key: &[u8],
    nonce: &[u8; 12],
    ciphertext_and_tag: &[u8],
) -> Result<Zeroizing<Vec<u8>>, openssl::error::ErrorStack> {
    use openssl::symm::{Cipher, Crypter, Mode};

    let (ciphertext, tag) =
        ciphertext_and_tag.split_at(ciphertext_and_tag.len().saturating_sub(16));
    if tag.len() != 16 {
        return Err(openssl::error::ErrorStack::get());
    }

    let mut crypter = Crypter::new(Cipher::aes_256_gcm(), Mode::Decrypt, key, Some(nonce))?;
    crypter.pad(false);
    crypter.set_tag(tag)?;
    let mut plaintext = Zeroizing::new(vec![
        0;
        ciphertext.len() + Cipher::aes_256_gcm().block_size()
    ]);
    let mut written = crypter.update(ciphertext, &mut plaintext)?;
    written += crypter.finalize(&mut plaintext[written..])?;
    plaintext.truncate(written);
    Ok(plaintext)
}
