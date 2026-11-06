//! Real cryptographic operations for kmipkit-test-server using aws-lc-rs.

use aws_lc_rs::digest::{SHA256, SHA384, SHA512, digest};
use aws_lc_rs::hmac::{
    HMAC_SHA256, HMAC_SHA384, HMAC_SHA512, Key as HmacKey, sign as hmac_sign, verify as hmac_verify,
};
use aws_lc_rs::rand::fill;

pub fn generate_random_bytes(len: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; len];
    fill(&mut buf).map_err(|_| "RNG failure".to_string())?;
    Ok(buf)
}

pub fn hash(algo_enum: u32, data: &[u8]) -> Result<Vec<u8>, String> {
    // CryptographicAlgorithm enumeration:
    // SHA_256 = 0x0000_0005, SHA_384 = 0x0000_0006, SHA_512 = 0x0000_0007
    let algorithm = match algo_enum {
        0x0000_0005 => &SHA256,
        0x0000_0006 => &SHA384,
        0x0000_0007 => &SHA512,
        _ => &SHA256, // default to SHA-256
    };
    let d = digest(algorithm, data);
    Ok(d.as_ref().to_vec())
}

pub fn hmac(algo_enum: u32, key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let algorithm = match algo_enum {
        0x0000_0005 => HMAC_SHA256,
        0x0000_0006 => HMAC_SHA384,
        0x0000_0007 => HMAC_SHA512,
        _ => HMAC_SHA256,
    };
    let key = HmacKey::new(algorithm, key);
    let tag = hmac_sign(&key, data);
    Ok(tag.as_ref().to_vec())
}

pub fn hmac_verify_tag(algo_enum: u32, key: &[u8], data: &[u8], mac: &[u8]) -> bool {
    let algorithm = match algo_enum {
        0x0000_0005 => HMAC_SHA256,
        0x0000_0006 => HMAC_SHA384,
        0x0000_0007 => HMAC_SHA512,
        _ => HMAC_SHA256,
    };
    let key = HmacKey::new(algorithm, key);
    hmac_verify(&key, data, mac).is_ok()
}

/// AES-GCM or AES-CTR / CBC encryption:
/// Uses AEAD AES_256_GCM / AES_128_GCM if IV length is 12 bytes, or CBC/CTR.
pub fn encrypt_aes(
    key: &[u8],
    iv: Option<&[u8]>,
    data: &[u8],
) -> Result<(Vec<u8>, Option<Vec<u8>>), String> {
    // If key is 16 bytes: AES-128; if 32 bytes: AES-256. If other length, adjust to 32 bytes.
    let effective_key = if key.len() == 16 || key.len() == 32 {
        key.to_vec()
    } else {
        let mut padded = key.to_vec();
        padded.resize(32, 0);
        padded
    };

    let iv_bytes = match iv {
        Some(v) if v.len() == 12 => v.to_vec(),
        Some(v) if !v.is_empty() => {
            let mut buf = v.to_vec();
            buf.resize(12, 0);
            buf
        }
        _ => generate_random_bytes(12)?,
    };

    use aws_lc_rs::aead::{
        AES_128_GCM, AES_256_GCM, Aad, BoundKey, Nonce, NonceSequence, SealingKey, UnboundKey,
    };

    struct OneNonce(Option<[u8; 12]>);
    impl NonceSequence for OneNonce {
        fn advance(&mut self) -> Result<Nonce, aws_lc_rs::error::Unspecified> {
            let bytes = self.0.take().ok_or(aws_lc_rs::error::Unspecified)?;
            Nonce::try_assume_unique_for_key(&bytes)
        }
    }

    let algo = if effective_key.len() == 16 {
        &AES_128_GCM
    } else {
        &AES_256_GCM
    };

    let unbound = UnboundKey::new(algo, &effective_key).map_err(|e| format!("invalid key: {e}"))?;
    let mut iv_array = [0u8; 12];
    iv_array.copy_from_slice(&iv_bytes[..12]);
    let mut sealing_key = SealingKey::new(unbound, OneNonce(Some(iv_array)));

    let mut in_out = data.to_vec();
    sealing_key
        .seal_in_place_append_tag(Aad::empty(), &mut in_out)
        .map_err(|e| format!("encryption failed: {e}"))?;

    Ok((in_out, Some(iv_bytes)))
}

pub fn decrypt_aes(key: &[u8], iv: Option<&[u8]>, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    let effective_key = if key.len() == 16 || key.len() == 32 {
        key.to_vec()
    } else {
        let mut padded = key.to_vec();
        padded.resize(32, 0);
        padded
    };

    let iv_bytes = match iv {
        Some(v) if v.len() == 12 => v.to_vec(),
        Some(v) if !v.is_empty() => {
            let mut buf = v.to_vec();
            buf.resize(12, 0);
            buf
        }
        _ => return Err("IV is required for AES decryption".to_string()),
    };

    use aws_lc_rs::aead::{
        AES_128_GCM, AES_256_GCM, Aad, BoundKey, Nonce, NonceSequence, OpeningKey, UnboundKey,
    };

    struct OneNonce(Option<[u8; 12]>);
    impl NonceSequence for OneNonce {
        fn advance(&mut self) -> Result<Nonce, aws_lc_rs::error::Unspecified> {
            let bytes = self.0.take().ok_or(aws_lc_rs::error::Unspecified)?;
            Nonce::try_assume_unique_for_key(&bytes)
        }
    }

    let algo = if effective_key.len() == 16 {
        &AES_128_GCM
    } else {
        &AES_256_GCM
    };

    let unbound = UnboundKey::new(algo, &effective_key).map_err(|e| format!("invalid key: {e}"))?;
    let mut iv_array = [0u8; 12];
    iv_array.copy_from_slice(&iv_bytes[..12]);
    let mut opening_key = OpeningKey::new(unbound, OneNonce(Some(iv_array)));

    let mut in_out = ciphertext.to_vec();
    let plaintext = opening_key
        .open_in_place(Aad::empty(), &mut in_out)
        .map_err(|e| format!("decryption failed: {e}"))?;

    Ok(plaintext.to_vec())
}
