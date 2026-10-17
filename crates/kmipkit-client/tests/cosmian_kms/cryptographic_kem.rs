//! Cosmian KEM response semantics and vendor serialization helpers.

#[cfg(test)]
mod tests {
    use kmipkit_protocol::SecretBytes;

    use super::super::cryptographic_kem::decode_configurable_kem_encrypt_data;

    #[test]
    fn configurable_kem_encrypt_data_decodes_shared_secret_and_encapsulation() {
        let encoded = SecretBytes::new(vec![3, 0x11, 0x22, 0x33, 2, 0xaa, 0xbb]);

        let (shared_secret, encapsulation) = decode_configurable_kem_encrypt_data(encoded)
            .expect("well-formed Cosmian Serializable tuple");

        assert!(shared_secret.with_bytes(|bytes| bytes == [0x11, 0x22, 0x33]));
        assert!(encapsulation.with_bytes(|bytes| bytes == [0xaa, 0xbb]));
    }

    #[test]
    fn configurable_kem_encrypt_data_supports_multibyte_lengths() {
        let mut encoded = vec![0x82, 0x01];
        encoded.extend(std::iter::repeat_n(0x5a, 130));
        encoded.extend([1, 0xa5]);

        let (shared_secret, encapsulation) = decode_configurable_kem_encrypt_data(
            SecretBytes::new(encoded),
        )
        .expect("LEB128 vector lengths");

        assert!(shared_secret.with_bytes(|bytes| bytes.len() == 130 && bytes.iter().all(|b| *b == 0x5a)));
        assert!(encapsulation.with_bytes(|bytes| bytes == [0xa5]));
    }

    #[test]
    fn configurable_kem_encrypt_data_rejects_empty_components() {
        for encoded in [vec![0, 1, 0xa5], vec![1, 0x5a, 0]] {
            assert!(decode_configurable_kem_encrypt_data(SecretBytes::new(encoded)).is_err());
        }
    }

    #[test]
    fn configurable_kem_encrypt_data_rejects_truncated_or_trailing_bytes() {
        for encoded in [
            vec![3, 0x11],
            vec![1, 0x11, 3, 0xaa],
            vec![1, 0x11, 1, 0xaa, 0xff],
            vec![0x80; 10],
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02],
        ] {
            assert!(decode_configurable_kem_encrypt_data(SecretBytes::new(encoded)).is_err());
        }
    }
}
