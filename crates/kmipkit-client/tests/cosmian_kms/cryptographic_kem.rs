//! Cosmian KEM response semantics and vendor serialization helpers.

pub(super) fn decode_configurable_kem_encrypt_data(
    encoded: &kmipkit_protocol::SecretBytes,
) -> Result<(kmipkit_protocol::SecretBytes, kmipkit_protocol::SecretBytes), &'static str> {
    encoded.with_bytes(|bytes| {
        let mut offset = 0;
        let shared_secret_length = decode_component_length(bytes, &mut offset)?;
        if shared_secret_length == 0 {
            return Err("ConfigurableKEM shared secret is empty");
        }
        let shared_secret_end = offset
            .checked_add(shared_secret_length)
            .ok_or("ConfigurableKEM shared secret length overflows")?;
        let shared_secret = bytes
            .get(offset..shared_secret_end)
            .ok_or("ConfigurableKEM shared secret is truncated")?
            .to_vec();
        offset = shared_secret_end;

        let encapsulation_length = decode_component_length(bytes, &mut offset)?;
        if encapsulation_length == 0 {
            return Err("ConfigurableKEM encapsulation is empty");
        }
        let encapsulation_end = offset
            .checked_add(encapsulation_length)
            .ok_or("ConfigurableKEM encapsulation length overflows")?;
        let encapsulation = bytes
            .get(offset..encapsulation_end)
            .ok_or("ConfigurableKEM encapsulation is truncated")?
            .to_vec();
        if encapsulation_end != bytes.len() {
            return Err("ConfigurableKEM response has trailing bytes");
        }

        Ok((
            kmipkit_protocol::SecretBytes::new(shared_secret),
            kmipkit_protocol::SecretBytes::new(encapsulation),
        ))
    })
}

fn decode_component_length(bytes: &[u8], offset: &mut usize) -> Result<usize, &'static str> {
    let mut value = 0_u64;
    for index in 0..10 {
        let byte = *bytes
            .get(*offset)
            .ok_or("ConfigurableKEM response is missing a component length")?;
        *offset = offset
            .checked_add(1)
            .ok_or("ConfigurableKEM response offset overflows")?;

        let payload = u64::from(byte & 0x7f);
        let shift = index * 7;
        if shift == 63 && payload > 1 {
            return Err("ConfigurableKEM component length overflows");
        }
        value |= payload << shift;
        if byte & 0x80 == 0 {
            return usize::try_from(value)
                .map_err(|_| "ConfigurableKEM component length is too large");
        }
    }
    Err("ConfigurableKEM component length is not a valid LEB128 value")
}

#[cfg(test)]
mod tests {
    use kmipkit_protocol::SecretBytes;

    use super::super::cryptographic_kem::decode_configurable_kem_encrypt_data;

    #[test]
    fn configurable_kem_encrypt_data_decodes_shared_secret_and_encapsulation() {
        let encoded = SecretBytes::new(vec![3, 0x11, 0x22, 0x33, 2, 0xaa, 0xbb]);

        let (shared_secret, encapsulation) = decode_configurable_kem_encrypt_data(&encoded)
            .expect("well-formed Cosmian Serializable tuple");

        assert!(shared_secret.with_bytes(|bytes| bytes == [0x11, 0x22, 0x33]));
        assert!(encapsulation.with_bytes(|bytes| bytes == [0xaa, 0xbb]));
    }

    #[test]
    fn configurable_kem_encrypt_data_supports_multibyte_lengths() {
        let mut encoded = vec![0x82, 0x01];
        encoded.extend(std::iter::repeat_n(0x5a, 130));
        encoded.extend([1, 0xa5]);

        let encoded = SecretBytes::new(encoded);
        let (shared_secret, encapsulation) =
            decode_configurable_kem_encrypt_data(&encoded).expect("LEB128 vector lengths");

        assert!(
            shared_secret
                .with_bytes(|bytes| bytes.len() == 130 && bytes.iter().all(|b| *b == 0x5a))
        );
        assert!(encapsulation.with_bytes(|bytes| bytes == [0xa5]));
    }

    #[test]
    fn configurable_kem_encrypt_data_rejects_empty_components() {
        for encoded in [vec![0, 1, 0xa5], vec![1, 0x5a, 0]] {
            let encoded = SecretBytes::new(encoded);
            assert!(decode_configurable_kem_encrypt_data(&encoded).is_err());
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
            let encoded = SecretBytes::new(encoded);
            assert!(decode_configurable_kem_encrypt_data(&encoded).is_err());
        }
    }
}
