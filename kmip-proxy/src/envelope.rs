//! Auto-Envelope key wrapping and memory zeroization engine.
#![forbid(unsafe_code)]

use zeroize::Zeroizing;

use crate::ProxyError;

/// Structure holding an envelope-wrapped key blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedEnvelopeKey {
    /// 12-byte random IV/nonce
    pub nonce: [u8; 12],
    /// Ciphertext with appended 16-byte GCM authentication tag
    pub ciphertext_and_tag: Vec<u8>,
}

impl WrappedEnvelopeKey {
    /// Serializes the wrapped key to bytes: [12-byte nonce | ciphertext + tag].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(12 + self.ciphertext_and_tag.len());
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.ciphertext_and_tag);
        out
    }

    /// Deserializes bytes into a WrappedEnvelopeKey.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProxyError> {
        if bytes.len() < 12 + 16 {
            return Err(ProxyError::EnvelopeError(
                "wrapped key blob is too short".to_string(),
            ));
        }
        let nonce: [u8; 12] = bytes[..12]
            .try_into()
            .map_err(|_| ProxyError::EnvelopeError("wrapped key blob is too short".to_string()))?;
        let ciphertext_and_tag = bytes[12..].to_vec();
        Ok(Self {
            nonce,
            ciphertext_and_tag,
        })
    }
}

pub struct AutoEnvelope;

impl AutoEnvelope {
    /// Generates cryptographically secure random bytes in a zeroized buffer.
    pub fn generate_random_bytes(len: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let mut buf = vec![0u8; len];
        crate::crypto::fill_random_bytes(&mut buf)
            .map_err(|e| ProxyError::Internal(format!("RNG failure: {e}")))?;
        Ok(Zeroizing::new(buf))
    }

    /// Wraps symmetric key material using a 256-bit Key Encryption Key (KEK).
    pub fn wrap_key(
        kek: &[u8],
        raw_key: &Zeroizing<Vec<u8>>,
    ) -> Result<WrappedEnvelopeKey, ProxyError> {
        if kek.len() != 32 {
            return Err(ProxyError::EnvelopeError(
                "Master KEK must be exactly 32 bytes (256 bits)".to_string(),
            ));
        }

        let nonce = crate::crypto::generate_gcm_nonce()
            .map_err(|e| ProxyError::Internal(format!("RNG failure: {e}")))?;
        let in_out = crate::crypto::aes_256_gcm_encrypt(kek, &nonce, raw_key.as_slice())
            .map_err(|e| ProxyError::EnvelopeError(format!("encryption failed: {e}")))?;

        Ok(WrappedEnvelopeKey {
            nonce,
            ciphertext_and_tag: in_out,
        })
    }

    /// Unwraps symmetric key material using a 256-bit Key Encryption Key (KEK).
    /// Returns the decrypted key in a zeroized buffer.
    pub fn unwrap_key(
        kek: &[u8],
        wrapped: &WrappedEnvelopeKey,
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        if kek.len() != 32 {
            return Err(ProxyError::EnvelopeError(
                "Master KEK must be exactly 32 bytes (256 bits)".to_string(),
            ));
        }

        if wrapped.ciphertext_and_tag.len() < 16 {
            return Err(ProxyError::EnvelopeError(
                "wrapped key blob is too short".to_string(),
            ));
        }
        crate::crypto::aes_256_gcm_decrypt(kek, &wrapped.nonce, &wrapped.ciphertext_and_tag)
            .map_err(|_| ProxyError::EnvelopeError("decryption/integrity check failed".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_envelope_roundtrip() {
        let kek = [0x42u8; 32];
        let key_bytes = vec![0x99u8; 32]; // AES-256 key
        let secret = Zeroizing::new(key_bytes.clone());

        let wrapped = AutoEnvelope::wrap_key(&kek, &secret).expect("wrap succeeds");
        assert_ne!(wrapped.ciphertext_and_tag, key_bytes);

        let serialized = wrapped.to_bytes();
        let deserialized =
            WrappedEnvelopeKey::from_bytes(&serialized).expect("deserialize succeeds");
        assert_eq!(deserialized, wrapped);

        let unwrapped = AutoEnvelope::unwrap_key(&kek, &deserialized).expect("unwrap succeeds");
        assert_eq!(*unwrapped, key_bytes);
    }
}
