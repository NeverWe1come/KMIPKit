//! OASIS KMIP 2.1 §3.3, Tables 14–17: typed wrapping metadata.

use kmipkit_protocol::{EncryptionKeyInformation, KeyWrappingData, MacSignatureKeyInformation};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

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
    let item = wrapping.into_item().expect("valid wrapping data");
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
        .with_encryption_key_information(
            EncryptionKeyInformation::try_new("wrapping-key".to_owned())
                .expect("required Unique Identifier"),
        )
        .with_iv_counter_nonce(vec![0xA1, 0xA2])
        .with_mac_signature(vec![0xB1, 0xB2])
        .into_item()
        .expect("valid wrapping data");
    assert_eq!(item.tag().raw(), 0x0042_0046);
}

#[test]
fn wrapping_data_requires_encryption_or_mac_signature_key_information() {
    let constructed = KeyWrappingData::try_new(1)
        .expect("known Wrapping Method")
        .into_item();
    assert!(constructed.is_err());

    let mut children = Structure::new();
    children
        .try_push(tagged(0x0042_009E, Value::enumeration(1)))
        .expect("valid Wrapping Method");
    let parsed = KeyWrappingData::try_from_item(
        Item::new(tag(0x0042_0046), Value::structure(children)).expect("valid Key Wrapping Data"),
    );
    assert!(parsed.is_err());
}

#[test]
fn wrapping_reader_checks_required_identifiers_and_keeps_unknown_fields() {
    let item = KeyWrappingData::try_new(0x8000_0001)
        .expect("future Wrapping Method remains open")
        .with_encryption_key_information(
            EncryptionKeyInformation::try_new("wrapping-key".to_owned())
                .expect("key information has a Unique Identifier"),
        )
        .into_item()
        .expect("valid wrapping structure");
    let parsed = KeyWrappingData::try_from_item(item.try_clone().expect("explicit zeroizing copy"))
        .expect("valid wrapping data can be read");
    let tags = parsed
        .with_raw(|view| {
            view.children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>()
        })
        .expect("parsed wrapper lends its tree");
    assert_eq!(tags, [0x0042_009E, 0x0042_0036]);

    let mut malformed = kmipkit_ttlv::Structure::new();
    malformed
        .try_push(
            kmipkit_ttlv::Item::new(
                kmipkit_ttlv::RawTag::new(0x0042_0036)
                    .expect("tag fits")
                    .try_checked()
                    .expect("assigned tag"),
                kmipkit_ttlv::Value::structure(kmipkit_ttlv::Structure::new()),
            )
            .expect("valid child"),
        )
        .expect("valid nesting");
    let malformed = kmipkit_ttlv::Item::new(
        kmipkit_ttlv::RawTag::new(0x0042_0046)
            .expect("tag fits")
            .try_checked()
            .expect("assigned tag"),
        kmipkit_ttlv::Value::structure(malformed),
    )
    .expect("valid test structure");
    assert!(KeyWrappingData::try_from_item(malformed).is_err());
}

fn tagged(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("valid test Item")
}

fn tag(raw: u32) -> kmipkit_ttlv::Tag {
    RawTag::new(raw)
        .expect("tag fits")
        .try_checked()
        .expect("assigned tag")
}
