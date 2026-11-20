//! Cryptographic operations for the KMIP test server.

use openssl::hash::{MessageDigest, hash as openssl_hash};
use openssl::pkey::PKey;
use openssl::rand::rand_bytes;
use openssl::sign::Signer;
use openssl::symm::{Cipher, decrypt_aead, encrypt_aead};

pub fn generate_random_bytes(len: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; len];
    rand_bytes(&mut buf).map_err(|_| "RNG failure".to_string())?;
    Ok(buf)
}

fn digest_algorithm(algo_enum: u32) -> Option<MessageDigest> {
    match algo_enum {
        0x0000_0005 => Some(MessageDigest::sha256()),
        0x0000_0006 => Some(MessageDigest::sha384()),
        0x0000_0007 => Some(MessageDigest::sha512()),
        _ => None,
    }
}

pub fn hash(algo_enum: u32, data: &[u8]) -> Result<Vec<u8>, String> {
    let algorithm = digest_algorithm(algo_enum)
        .ok_or_else(|| format!("unsupported hash algorithm {algo_enum:#010x}"))?;
    openssl_hash(algorithm, data)
        .map(|digest| digest.to_vec())
        .map_err(|e| format!("hash failed: {e}"))
}

pub fn hmac(algo_enum: u32, key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let algorithm = digest_algorithm(algo_enum)
        .ok_or_else(|| format!("unsupported HMAC algorithm {algo_enum:#010x}"))?;
    // An empty HMAC key and a single zero byte both pad to the same block.
    let key = if key.is_empty() { &[0u8] } else { key };
    let sign = || {
        let key = PKey::hmac(key)?;
        let mut signer = Signer::new(algorithm, &key)?;
        signer.update(data)?;
        signer.sign_to_vec()
    };
    sign().map_err(|e: openssl::error::ErrorStack| format!("HMAC failed: {e}"))
}

pub fn hmac_verify_tag(algo_enum: u32, key: &[u8], data: &[u8], mac: &[u8]) -> bool {
    hmac(algo_enum, key, data)
        .is_ok_and(|tag| tag.len() == mac.len() && openssl::memcmp::eq(&tag, mac))
}

const GCM_IV_LEN: usize = 12;
const GCM_TAG_LEN: usize = 16;

fn aes_gcm_cipher(key: &[u8]) -> Result<Cipher, String> {
    match key.len() {
        16 => Ok(Cipher::aes_128_gcm()),
        32 => Ok(Cipher::aes_256_gcm()),
        _ => Err("invalid AES key length (must be 16 or 32 bytes)".to_string()),
    }
}

fn normalize_iv(iv: &[u8]) -> Vec<u8> {
    let mut normalized = iv[..iv.len().min(GCM_IV_LEN)].to_vec();
    normalized.resize(GCM_IV_LEN, 0);
    normalized
}

pub fn encrypt_aes(
    key: &[u8],
    iv: Option<&[u8]>,
    data: &[u8],
) -> Result<(Vec<u8>, Option<Vec<u8>>), String> {
    let cipher = aes_gcm_cipher(key)?;
    let iv_bytes = match iv.filter(|iv| !iv.is_empty()) {
        Some(iv) => normalize_iv(iv),
        None => generate_random_bytes(GCM_IV_LEN)?,
    };
    let mut tag = [0u8; GCM_TAG_LEN];
    let mut ciphertext = encrypt_aead(cipher, key, Some(&iv_bytes), &[], data, &mut tag)
        .map_err(|e| format!("encryption failed: {e}"))?;
    ciphertext.extend_from_slice(&tag);
    Ok((ciphertext, Some(iv_bytes)))
}

pub fn decrypt_aes(key: &[u8], iv: Option<&[u8]>, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = aes_gcm_cipher(key)?;
    let iv_bytes = normalize_iv(
        iv.filter(|iv| !iv.is_empty())
            .ok_or_else(|| "IV is required for AES decryption".to_string())?,
    );
    let (ciphertext, tag) = ciphertext
        .split_at_checked(ciphertext.len().saturating_sub(GCM_TAG_LEN))
        .filter(|(_, tag)| tag.len() == GCM_TAG_LEN)
        .ok_or_else(|| "decryption failed: invalid authentication tag".to_string())?;
    decrypt_aead(cipher, key, Some(&iv_bytes), &[], ciphertext, tag)
        .map_err(|e| format!("decryption failed: {e}"))
}
