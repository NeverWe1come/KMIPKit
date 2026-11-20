//! Auto-Envelope key wrapping and memory zeroization engine.
#![forbid(unsafe_code)]

use aws_lc_rs::aead::{
    AES_256_GCM, Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey,
};
use zeroize::Zeroizing;

use crate::ProxyError;

struct OneNonce(Option<[u8; 12]>);

impl NonceSequence for OneNonce {
    fn advance(&mut self) -> Result<Nonce, aws_lc_rs::error::Unspecified> {
        let bytes = self.0.take().ok_or(aws_lc_rs::error::Unspecified)?;
        Nonce::try_assume_unique_for_key(&bytes)
    }
}

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
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&bytes[..12]);
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
        aws_lc_rs::rand::fill(&mut buf)
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

        let mut nonce = [0u8; 12];
        aws_lc_rs::rand::fill(&mut nonce)
            .map_err(|e| ProxyError::Internal(format!("RNG failure: {e}")))?;

        let unbound = UnboundKey::new(&AES_256_GCM, kek)
            .map_err(|e| ProxyError::EnvelopeError(format!("invalid KEK: {e}")))?;
        let mut sealing_key = SealingKey::new(unbound, OneNonce(Some(nonce)));

        let mut in_out = raw_key.as_slice().to_vec();
        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut in_out)
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

        let unbound = UnboundKey::new(&AES_256_GCM, kek)
            .map_err(|e| ProxyError::EnvelopeError(format!("invalid KEK: {e}")))?;
        let mut opening_key = OpeningKey::new(unbound, OneNonce(Some(wrapped.nonce)));

        let mut in_out = wrapped.ciphertext_and_tag.clone();
        let decrypted = opening_key
            .open_in_place(Aad::empty(), &mut in_out)
            .map_err(|e| {
                ProxyError::EnvelopeError(format!("decryption/integrity check failed: {e}"))
            })?;

        Ok(Zeroizing::new(decrypted.to_vec()))
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
