//! Cosmian CoverCrypt typed Encrypt/Decrypt fixture regressions.

#[cfg(test)]
mod tests {
    #[test]
    fn data_to_encrypt_uses_vendor_leb128_policy_metadata_and_plaintext_layout() {
        let encoded = super::encode_data_to_encrypt("policy", b"meta", b"cleartext")
            .expect("CoverCrypt data encoding");
        assert!(encoded.with_bytes(|bytes| bytes == b"\x06policy\x04metacleartext"));
    }

    #[test]
    fn data_to_encrypt_encodes_multibyte_leb128_policy_lengths() {
        let policy = "p".repeat(130);
        let encoded = super::encode_data_to_encrypt(&policy, &[], b"cleartext")
            .expect("CoverCrypt data encoding");
        assert!(encoded.with_bytes(|bytes| {
            bytes.starts_with(b"\x82\x01")
                && bytes.get(2..132) == Some(policy.as_bytes())
                && bytes.get(132) == Some(&0)
                && bytes.get(133..) == Some(b"cleartext")
        }));
    }
}
