use kmipkit_ttlv::KmipTag;

#[test]
fn authenticated_encryption_additional_data_has_a_name() {
    assert_eq!(
        KmipTag::AuthenticatedEncryptionAdditionalData.name(),
        "Authenticated Encryption Additional Data"
    );
}
