use super::*;

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_validate_crypto_response_ttlv(
        item: *mut kmipkit_client_batch_item_response_t,
        operation_index: u32,
    ) -> i32;
}

#[test]
fn compiled_c_consumer_calls_each_crypto_success_ttlv_getter_and_releases_owned_copy() {
    let unknown = ttlv_enumeration(0x0054_0001, 0xfedc_ba98);
    let responses = [
        successful_cryptographic_item(
            test_hash_request(),
            0x27,
            b"c-hash-success",
            vec![ttlv_item(0x0042_00c2, 0x08, b"digest"), unknown.clone()],
        ),
        successful_cryptographic_item(
            ClientRequest::Mac(
                MacRequest::new()
                    .with_unique_identifier(response_fixture_uid())
                    .with_data(OperationData::ByteString(SecretBytes::new(
                        b"mac-input".to_vec(),
                    ))),
            ),
            0x23,
            b"c-mac-success",
            vec![
                ttlv_item(0x0042_0094, 0x07, b"response-id"),
                ttlv_item(0x0042_00c6, 0x08, b"mac-data"),
                unknown.clone(),
            ],
        ),
        successful_cryptographic_item(
            ClientRequest::MacVerify(
                MacVerifyRequest::new()
                    .with_unique_identifier(response_fixture_uid())
                    .with_mac_data(SecretBytes::new(b"mac-input".to_vec())),
            ),
            0x24,
            b"c-mac-verify-success",
            vec![
                ttlv_item(0x0042_0094, 0x07, b"response-id"),
                ttlv_enumeration(0x0042_009B, 0xfedc_ba98),
                unknown.clone(),
            ],
        ),
        successful_cryptographic_item(
            ClientRequest::Sign(
                SignRequest::new()
                    .with_unique_identifier(response_fixture_uid())
                    .with_data(OperationData::ByteString(SecretBytes::new(
                        b"sign-input".to_vec(),
                    ))),
            ),
            0x21,
            b"c-sign-success",
            vec![
                ttlv_item(0x0042_0094, 0x07, b"response-id"),
                ttlv_item(0x0042_00c3, 0x08, b"signature"),
                unknown.clone(),
            ],
        ),
        successful_cryptographic_item(
            ClientRequest::SignatureVerify(
                SignatureVerifyRequest::new()
                    .with_unique_identifier(response_fixture_uid())
                    .with_signature_data(SecretBytes::new(b"signature-input".to_vec())),
            ),
            0x22,
            b"c-signature-verify-success",
            vec![
                ttlv_item(0x0042_0094, 0x07, b"response-id"),
                ttlv_enumeration(0x0042_009B, 0xfedc_ba98),
                ttlv_item(0x0042_00c2, 0x08, b"recovered"),
                unknown,
            ],
        ),
    ];

    for (operation_index, item) in responses.into_iter().enumerate() {
        // SAFETY: the C consumer owns this valid fixture handle, consumes it, checks the
        // ordered success response copy after source release, and releases every C object.
        let result = unsafe {
            kmipkit_client_abi_c_consumer_validate_crypto_response_ttlv(
                item,
                u32::try_from(operation_index).expect("five operation indexes fit u32"),
            )
        };
        assert_eq!(
            1, result,
            "C consumer failed for operation index {operation_index}"
        );
    }
}
