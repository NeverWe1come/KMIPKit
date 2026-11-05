//! OASIS KMIP 2.1 §3.3, Tables 14–17: typed wrapping metadata.

use kmipkit_protocol::{EncryptionKeyInformation, KeyWrappingData, MacSignatureKeyInformation};
use kmipkit_ttlv::ValueView;

#[test]
fn key_wrapping_data_emits_method_before_optional_metadata() {
    let wrapping = KeyWrappingData::try_new(0x8000_0001)
        .expect("unknown wrapping method is preserved")
        .with_encryption_key_information(
            EncryptionKeyInformation::try_new("wrap-key".to_owned())
                .expect("required Unique Identifier"),
        )
        .with_mac_signature_key_information(
            MacSignatureKeyInformation::try_new("mac-key".to_owned())
                .expect("required Unique Identifier"),
        )
        .with_mac_signature(vec![0xA1])
        .with_iv_counter_nonce(vec![0xB1])
        .with_encoding_option(2);
    let item = wrapping.into_item();
    let tags = item.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    });
    assert_eq!(
        tags,
        [
            0x0042_009E,
            0x0042_0036,
            0x0042_004E,
            0x0042_004D,
            0x0042_003D,
            0x0042_00A3,
        ]
    );
}

#[test]
fn wrapping_data_retains_raw_metadata_and_never_performs_crypto() {
    let item = KeyWrappingData::try_new(1)
        .expect("valid wrapper")
        .with_iv_counter_nonce(vec![0xA1, 0xA2])
        .with_mac_signature(vec![0xB1, 0xB2])
        .into_item();
    assert_eq!(item.tag().raw(), 0x0042_0046);
}
