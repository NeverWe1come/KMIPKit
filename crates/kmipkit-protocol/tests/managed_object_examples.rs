//! Compiling KMIPKIT-0049 Rust usage examples.

use kmipkit_protocol::{
    AttributeSet, EncryptionKeyInformation, KeyBlock, KeyBlockValue, KeyFormatType, KeyMaterial,
    KeyValue, KeyWrappingData, ManagedObject, ObjectType, SecretBytes, SymmetricKey,
    TransparentSymmetricKey,
};

#[test]
fn plaintext_typed_object_uses_direct_object_structure() {
    let material =
        KeyMaterial::byte_string(1, vec![0xA1, 0xA2]).expect("Raw Key Material is a Byte String");
    let key_value = KeyValue::try_new(material, Some(AttributeSet::new()))
        .expect("Key Value contains its required Key Material");
    let key_block = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("Raw Key Format Type")
        .with_key_value(KeyBlockValue::Plaintext(key_value));
    let symmetric = SymmetricKey::try_new(key_block).expect("valid Symmetric Key");
    let managed = ManagedObject::try_from_item(ObjectType::from_raw(2), symmetric.into_item())
        .expect("Symmetric Key tag matches Object Type");
    assert_eq!(managed.object_type().raw(), 2);
    assert_eq!(managed.into_item().tag().raw(), 0x0042_008F);
}

#[test]
fn wrapped_object_keeps_wrapping_metadata_and_uses_byte_string() {
    let wrapping = KeyWrappingData::try_new(1)
        .expect("known Wrapping Method")
        .with_encryption_key_information(
            EncryptionKeyInformation::try_new("wrapping-key".to_owned())
                .expect("key information requires Unique Identifier"),
        );
    let key_block = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("Raw Key Format Type")
        .with_key_value(KeyBlockValue::Wrapped(SecretBytes::new(vec![0xB1, 0xB2])))
        .with_key_wrapping_data(wrapping);
    assert_eq!(
        key_block.key_value_type(),
        Some(kmipkit_ttlv::ItemType::ByteString)
    );
    let item = SymmetricKey::try_new(key_block)
        .expect("valid wrapped Symmetric Key")
        .into_item();
    assert_eq!(item.tag().raw(), 0x0042_008F);
}

#[test]
fn transparent_material_unknown_fallback_and_secret_owner_examples_compile() {
    let transparent = TransparentSymmetricKey::try_new(vec![0xC1])
        .expect("transparent key has required Key bytes")
        .into_item();
    assert_eq!(transparent.tag().raw(), 0x0042_0043);

    let extension = kmipkit_ttlv::Item::new(
        kmipkit_ttlv::RawTag::new(0x0054_0102)
            .expect("tag fits")
            .try_checked()
            .expect("vendor extension tag is allocated"),
        kmipkit_ttlv::Value::structure(kmipkit_ttlv::Structure::new()),
    )
    .expect("valid generic structure");
    let unknown = ManagedObject::unknown(ObjectType::from_raw(0x8000_0001), extension)
        .expect("unknown Object Type remains generic");
    assert_eq!(unknown.object_type().raw(), 0x8000_0001);

    let caller_owned_secret = vec![0xD1, 0xD2];
    let secret_owner = KeyMaterial::byte_string(1, caller_owned_secret)
        .expect("Raw key bytes become a KMIPKit-owned zeroizing value");
    drop(secret_owner);
}
