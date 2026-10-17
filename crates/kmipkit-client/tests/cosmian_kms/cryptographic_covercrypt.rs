//! Cosmian `CoverCrypt` typed Encrypt/Decrypt fixture regressions.

use kmipkit_protocol::SecretBytes;

/// Builds Cosmian's documented vendor `DataToEncrypt` byte container.
pub(super) fn encode_data_to_encrypt(
    policy: &str,
    metadata: &[u8],
    plaintext: &[u8],
) -> Result<SecretBytes, &'static str> {
    let policy = policy.as_bytes();
    let mut encoded = Vec::with_capacity(policy.len() + metadata.len() + plaintext.len() + 20);
    append_leb128_length(&mut encoded, policy.len())?;
    encoded.extend_from_slice(policy);
    append_leb128_length(&mut encoded, metadata.len())?;
    encoded.extend_from_slice(metadata);
    encoded.extend_from_slice(plaintext);
    Ok(SecretBytes::new(encoded))
}

pub(super) fn split_decrypted_data(data: &[u8]) -> Result<(&[u8], &[u8]), &'static str> {
    let (metadata_length, prefix_length) = read_leb128_length(data)?;
    let plaintext_offset = prefix_length
        .checked_add(metadata_length)
        .ok_or("CoverCrypt metadata length overflows usize")?;
    let metadata = data
        .get(prefix_length..plaintext_offset)
        .ok_or("CoverCrypt metadata is truncated")?;
    let plaintext = data
        .get(plaintext_offset..)
        .ok_or("CoverCrypt plaintext offset is out of range")?;
    Ok((metadata, plaintext))
}

fn read_leb128_length(data: &[u8]) -> Result<(usize, usize), &'static str> {
    let mut metadata_length = 0_usize;
    let mut shift = 0_u32;
    let mut prefix_length = 0_usize;

    loop {
        let byte = *data
            .get(prefix_length)
            .ok_or("CoverCrypt metadata length is truncated")?;
        let chunk = usize::from(byte & 0x7f);
        if shift >= usize::BITS || chunk > (usize::MAX >> shift) {
            return Err("CoverCrypt metadata length overflows usize");
        }
        metadata_length |= chunk << shift;
        prefix_length += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift = shift
            .checked_add(7)
            .filter(|next_shift| *next_shift < usize::BITS)
            .ok_or("CoverCrypt metadata length overflows usize")?;
    }
    Ok((metadata_length, prefix_length))
}

fn append_leb128_length(output: &mut Vec<u8>, mut length: usize) -> Result<(), &'static str> {
    loop {
        let mut byte = u8::try_from(length & 0x7f).map_err(|_| "LEB128 chunk is out of range")?;
        length >>= 7;
        if length != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if length == 0 {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn decrypted_data_splits_metadata_prefix_from_plaintext() {
        let (metadata, plaintext) =
            super::split_decrypted_data(b"\x04metaKMIPKit CoverCrypt integration payload")
                .expect("CoverCrypt decrypted data container");

        assert_eq!(metadata, b"meta");
        assert_eq!(plaintext, b"KMIPKit CoverCrypt integration payload");
    }

    #[test]
    fn decrypted_data_accepts_empty_metadata_prefix_from_cosmian_example() {
        let (metadata, plaintext) =
            super::split_decrypted_data(b"\x00KMIPKit CoverCrypt integration payload")
                .expect("CoverCrypt decrypted data container");

        assert_eq!(metadata, &[] as &[u8]);
        assert_eq!(plaintext, b"KMIPKit CoverCrypt integration payload");
    }

    #[test]
    fn decrypted_data_splits_multibyte_metadata_length() {
        let mut container = vec![0x82, 0x01];
        container.extend(std::iter::repeat_n(b'm', 130));
        container.extend_from_slice(b"plaintext");

        let (metadata, plaintext) =
            super::split_decrypted_data(&container).expect("CoverCrypt decrypted data container");

        assert_eq!(metadata.len(), 130);
        assert!(metadata.iter().all(|byte| *byte == b'm'));
        assert_eq!(plaintext, b"plaintext");
    }

    #[test]
    fn decrypted_data_rejects_truncated_metadata_length_and_metadata() {
        assert!(super::split_decrypted_data(b"\x80").is_err());
        assert!(super::split_decrypted_data(b"\x04abc").is_err());
    }

    #[test]
    fn decrypted_data_rejects_metadata_length_overflow() {
        let mut encoded_length = vec![0xff; (usize::BITS as usize).div_ceil(7)];
        let overflow_marker = 1_u8 << ((usize::BITS - 1) % 7 + 1);
        if let Some(last) = encoded_length.last_mut() {
            *last = overflow_marker;
        }

        assert!(
            super::split_decrypted_data(&encoded_length)
                .expect_err("overflowing length must be rejected")
                .contains("overflows")
        );
    }

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
