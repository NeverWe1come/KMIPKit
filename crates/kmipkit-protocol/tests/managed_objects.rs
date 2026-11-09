//! OASIS KMIP 2.1 §§2.1–2.9, Tables 2–10: typed object wire contracts.

use kmipkit_protocol::{
    Certificate, CertificateRequest, KeyBlock, KeyBlockValue, KeyFormatType, KeyMaterial, KeyValue,
    ManagedObject, ObjectType, OpaqueObject, PgpKey, PrivateKey, PublicKey, SecretData, SplitKey,
    SplitKeyMethod, SymmetricKey,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Value, ValueView};

fn key_block() -> KeyBlock {
    KeyBlock::try_new(KeyFormatType::from_raw(0xFFFF_FFFE))
        .expect("an open future Key Format Type remains representable")
}

fn secret_data_key_block() -> KeyBlock {
    let value = KeyValue::try_new(
        KeyMaterial::byte_string(1, vec![0xA1]).expect("Raw Secret Data material"),
        None,
    )
    .expect("valid Secret Data Key Value");
    KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("Raw format")
        .with_key_value(KeyBlockValue::Plaintext(value))
}

fn item_shape(item: &Item) -> (u32, Vec<(u32, ItemType)>) {
    item.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            panic!("typed managed object must be a Structure")
        };
        (
            item.tag().raw(),
            structure
                .children()
                .iter()
                .map(|child| (child.tag().raw(), child.item_type()))
                .collect(),
        )
    })
}

#[test]
fn every_known_managed_object_uses_its_oasis_structure_tag_and_field_order() {
    let objects = [
        Certificate::try_new(0x8000_0001, vec![0xC1, 0xC2])
            .expect("valid Certificate fields")
            .into_item(),
        CertificateRequest::try_new(0x8000_0002, vec![0xD1])
            .expect("valid Certificate Request fields")
            .into_item(),
        OpaqueObject::try_new(0x8000_0003, vec![0xE1])
            .expect("valid Opaque Object fields")
            .into_item(),
        PgpKey::try_new(7, key_block())
            .expect("valid PGP Key fields")
            .into_item(),
        PrivateKey::try_new(key_block())
            .expect("valid Private Key fields")
            .into_item(),
        PublicKey::try_new(key_block())
            .expect("valid Public Key fields")
            .into_item(),
        SecretData::try_new(0x8000_0004, secret_data_key_block())
            .expect("valid Secret Data fields")
            .into_item(),
        SplitKey::try_new(key_block(), 2, SplitKeyMethod::from_raw(1), 5, 3, None)
            .expect("valid Split Key fields")
            .into_item(),
        SymmetricKey::try_new(key_block())
            .expect("valid Symmetric Key fields")
            .into_item(),
    ];

    let expected = [
        (
            0x0042_0013,
            vec![
                (0x0042_001D, ItemType::Enumeration),
                (0x0042_001E, ItemType::ByteString),
            ],
        ),
        (
            0x0042_0018,
            vec![
                (0x0042_0019, ItemType::Enumeration),
                (0x0042_0140, ItemType::ByteString),
            ],
        ),
        (
            0x0042_005B,
            vec![
                (0x0042_0059, ItemType::Enumeration),
                (0x0042_005A, ItemType::ByteString),
            ],
        ),
        (
            0x0042_00BD,
            vec![
                (0x0042_00BE, ItemType::Integer),
                (0x0042_0040, ItemType::Structure),
            ],
        ),
        (0x0042_0064, vec![(0x0042_0040, ItemType::Structure)]),
        (0x0042_006D, vec![(0x0042_0040, ItemType::Structure)]),
        (
            0x0042_0085,
            vec![
                (0x0042_0086, ItemType::Enumeration),
                (0x0042_0040, ItemType::Structure),
            ],
        ),
        (
            0x0042_0089,
            vec![
                (0x0042_008B, ItemType::Integer),
                (0x0042_0044, ItemType::Integer),
                (0x0042_008C, ItemType::Integer),
                (0x0042_008A, ItemType::Enumeration),
                (0x0042_0040, ItemType::Structure),
            ],
        ),
        (0x0042_008F, vec![(0x0042_0040, ItemType::Structure)]),
    ];
    for (item, expected) in objects.iter().zip(expected) {
        assert_eq!(item_shape(item), expected);
    }
}

#[test]
fn managed_object_dispatch_requires_matching_type_and_structure_tag() {
    let item = Certificate::try_new(1, vec![1, 2, 3])
        .expect("valid certificate")
        .into_item();
    let typed = ManagedObject::try_from_item(ObjectType::from_raw(1), item)
        .expect("matching Object Type and structure tag");
    assert_eq!(typed.object_type().raw(), 1);
    assert_eq!(typed.into_item().tag().raw(), 0x0042_0013);
}

#[test]
fn every_known_object_type_dispatches_only_its_matching_structure_tag() {
    let objects = [
        Certificate::try_new(1, vec![1]).unwrap().into_item(),
        CertificateRequest::try_new(1, vec![1]).unwrap().into_item(),
        OpaqueObject::try_new(1, vec![1]).unwrap().into_item(),
        PgpKey::try_new(1, key_block()).unwrap().into_item(),
        PrivateKey::try_new(key_block()).unwrap().into_item(),
        PublicKey::try_new(key_block()).unwrap().into_item(),
        SecretData::try_new(1, secret_data_key_block())
            .unwrap()
            .into_item(),
        SplitKey::try_new(key_block(), 1, SplitKeyMethod::from_raw(1), 2, 2, None)
            .unwrap()
            .into_item(),
        SymmetricKey::try_new(key_block()).unwrap().into_item(),
    ];
    let object_types = [1, 10, 8, 9, 4, 3, 7, 5, 2];
    for (item, object_type) in objects.iter().zip(object_types) {
        let typed = ManagedObject::try_from_item(
            ObjectType::from_raw(object_type),
            item.try_clone().expect("copy is explicit and zeroizing"),
        )
        .expect("known Object Type maps to the matching object Item tag");
        assert_eq!(typed.object_type().raw(), object_type);
        assert!(format!("{typed:?}").contains("item_tag"));
        assert_eq!(typed.into_item().tag().raw(), item.tag().raw());
    }
}

#[test]
fn known_object_type_rejects_another_object_structure_tag() {
    let certificate = Certificate::try_new(1, vec![1]).unwrap().into_item();
    let error = ManagedObject::try_from_item(ObjectType::from_raw(2), certificate);
    assert!(error.is_err());
}

#[test]
fn unknown_object_types_and_vendor_children_remain_lossless() {
    let extension_tag = RawTag::new(0x0054_0101)
        .expect("24-bit tag")
        .try_checked()
        .expect("vendor extension tag is allocated");
    let mut children = kmipkit_ttlv::Structure::new();
    children
        .try_push(Item::new(extension_tag, Value::byte_string(vec![0x51, 0x52])).unwrap())
        .unwrap();
    let unknown_tag = RawTag::new(0x0054_0102)
        .expect("24-bit tag")
        .try_checked()
        .expect("vendor extension tag is allocated");
    let unknown = Item::new(unknown_tag, Value::structure(children)).unwrap();
    let typed = ManagedObject::unknown(ObjectType::from_raw(0x8000_0001), unknown)
        .expect("unknown values use the generic fallback");
    assert_eq!(typed.object_type().raw(), 0x8000_0001);
    let round_trip = typed.into_item();
    assert_eq!(round_trip.tag().raw(), 0x0054_0102);
    let nested = round_trip.with_value(|value| match value {
        ValueView::Structure(structure) => structure.children()[0].tag().raw(),
        _ => 0,
    });
    assert_eq!(nested, 0x0054_0101);
}

#[test]
fn split_key_rejects_identifier_outside_one_through_parts() {
    let result = SplitKey::try_new(key_block(), 6, SplitKeyMethod::from_raw(1), 5, 3, None);
    assert!(result.is_err());
}

#[test]
fn polynomial_prime_field_requires_prime_field_size() {
    let result = SplitKey::try_new(
        key_block(),
        1,
        SplitKeyMethod::POLYNOMIAL_SHARING_PRIME_FIELD,
        3,
        2,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn split_key_threshold_one_is_not_rejected_by_an_unstated_local_limit() {
    let split = SplitKey::try_new(key_block(), 1, SplitKeyMethod::from_raw(1), 2, 1, None)
        .expect("KMIP 2.1 §2.8 does not impose a threshold minimum of two");
    assert_eq!(
        item_shape(&split.into_item()).1[2],
        (0x0042_008C, ItemType::Integer)
    );

    let parsed = SplitKey::try_from_item(split_key_item(2, 1, 1, 1, false));
    assert!(parsed.is_ok(), "the reader preserves the source threshold");
}

#[test]
fn secret_data_rejects_key_block_algorithm_and_length_fields() {
    let block = secret_data_key_block().with_cryptographic_properties(Some(3), Some(256));
    let result = SecretData::try_new(1, block);
    assert!(result.is_err());
}

#[test]
fn secret_data_requires_a_key_value_in_constructor_and_reader() {
    assert!(SecretData::try_new(1, key_block()).is_err());

    let block = key_block()
        .try_into_item()
        .expect("a Key Block may be structurally read without a Key Value");
    let secret = structure_item(
        0x0042_0085,
        [
            Item::new(tag(0x0042_0086), Value::enumeration(1)).expect("Secret Data Type"),
            block,
        ],
    );
    assert!(SecretData::try_from_item(secret).is_err());
}

#[test]
fn split_key_reader_rejects_invalid_numeric_and_conditional_fields() {
    let invalid_identifier = split_key_item(5, 6, 3, 1, false);
    assert!(SplitKey::try_from_item(invalid_identifier).is_err());

    let missing_prime_field = split_key_item(3, 1, 2, 3, false);
    assert!(SplitKey::try_from_item(missing_prime_field).is_err());
}

#[test]
fn secret_data_reader_rejects_cryptographic_properties_in_key_block() {
    let block = secret_data_key_block()
        .with_cryptographic_properties(Some(3), Some(256))
        .try_into_item()
        .expect("paired Key Block cryptographic fields are structurally valid");
    let secret = structure_item(
        0x0042_0085,
        [
            Item::new(
                RawTag::new(0x0042_0086)
                    .expect("tag fits")
                    .try_checked()
                    .expect("assigned tag"),
                Value::enumeration(1),
            )
            .expect("valid Secret Data Type"),
            block,
        ],
    );
    assert!(SecretData::try_from_item(secret).is_err());
}

fn split_key_item(
    parts: i32,
    identifier: i32,
    threshold: i32,
    method: u32,
    include_prime_field_size: bool,
) -> Item {
    let mut fields = vec![
        Item::new(
            RawTag::new(0x0042_008B)
                .expect("tag fits")
                .try_checked()
                .expect("assigned tag"),
            Value::integer(parts),
        )
        .expect("valid parts"),
        Item::new(
            RawTag::new(0x0042_0044)
                .expect("tag fits")
                .try_checked()
                .expect("assigned tag"),
            Value::integer(identifier),
        )
        .expect("valid identifier"),
        Item::new(
            RawTag::new(0x0042_008C)
                .expect("tag fits")
                .try_checked()
                .expect("assigned tag"),
            Value::integer(threshold),
        )
        .expect("valid threshold"),
        Item::new(
            RawTag::new(0x0042_008A)
                .expect("tag fits")
                .try_checked()
                .expect("assigned tag"),
            Value::enumeration(method),
        )
        .expect("valid method"),
    ];
    if include_prime_field_size {
        fields.push(
            Item::new(
                RawTag::new(0x0042_0062)
                    .expect("tag fits")
                    .try_checked()
                    .expect("assigned tag"),
                Value::big_integer(vec![0x11]),
            )
            .expect("valid prime field size"),
        );
    }
    fields.push(
        key_block()
            .try_into_item()
            .expect("valid Key Block is required"),
    );
    structure_item(0x0042_0089, fields)
}

fn structure_item(tag: u32, fields: impl IntoIterator<Item = Item>) -> Item {
    let mut structure = kmipkit_ttlv::Structure::new();
    for field in fields {
        structure.try_push(field).expect("valid nested test field");
    }
    Item::new(
        RawTag::new(tag)
            .expect("tag fits")
            .try_checked()
            .expect("assigned tag"),
        Value::structure(structure),
    )
    .expect("valid test Structure")
}

#[test]
fn split_key_prime_field_size_precedes_key_block_in_table_nine_order() {
    let item = SplitKey::try_new(
        key_block(),
        1,
        SplitKeyMethod::POLYNOMIAL_SHARING_PRIME_FIELD,
        3,
        2,
        Some(vec![0x11]),
    )
    .expect("Polynomial Sharing Prime Field requires Prime Field Size")
    .into_item();
    assert_eq!(
        item_shape(&item),
        (
            0x0042_0089,
            vec![
                (0x0042_008B, ItemType::Integer),
                (0x0042_0044, ItemType::Integer),
                (0x0042_008C, ItemType::Integer),
                (0x0042_008A, ItemType::Enumeration),
                (0x0042_0062, ItemType::BigInteger),
                (0x0042_0040, ItemType::Structure),
            ],
        )
    );
}

#[test]
fn typed_object_accessors_and_raw_views_cover_all_managed_object_models() {
    let certificate = Certificate::try_new(0x8000_0001, vec![0xD1, 0xD2]).expect("Certificate");
    assert_eq!(certificate.certificate_type(), Some(0x8000_0001));
    assert_eq!(certificate.with_certificate_value(<[u8]>::len), Some(2));
    assert!(!format!("{certificate:?}").contains("D1"));
    assert!(certificate.with_raw(|view| view.children().len()).is_ok());

    let request = CertificateRequest::try_new(2, vec![0xD3]).expect("Certificate Request");
    assert_eq!(request.request_type(), Some(2));
    assert!(request.with_raw(|view| view.children().len()).is_ok());

    let opaque = OpaqueObject::try_new(3, vec![0xD4]).expect("Opaque Object");
    assert_eq!(opaque.opaque_data_type(), Some(3));
    assert!(opaque.with_raw(|view| view.children().len()).is_ok());

    let pgp = PgpKey::try_new(7, key_block()).expect("PGP Key");
    assert_eq!(pgp.pgp_key_version(), Some(7));
    assert!(pgp.with_raw(|view| view.children().len()).is_ok());

    let private = PrivateKey::try_new(key_block()).expect("Private Key");
    assert!(private.with_raw(|view| view.children().len()).is_ok());
    let public = PublicKey::try_new(key_block()).expect("Public Key");
    assert!(public.with_raw(|view| view.children().len()).is_ok());
    let secret = SecretData::try_new(4, secret_data_key_block()).expect("Secret Data");
    assert_eq!(secret.secret_data_type(), Some(4));
    assert!(secret.with_raw(|view| view.children().len()).is_ok());
    let split = SplitKey::try_new(key_block(), 1, SplitKeyMethod::from_raw(1), 2, 2, None)
        .expect("Split Key");
    assert!(split.with_raw(|view| view.children().len()).is_ok());
    let symmetric = SymmetricKey::try_new(key_block()).expect("Symmetric Key");
    assert!(symmetric.with_raw(|view| view.children().len()).is_ok());

    let typed = ManagedObject::try_from_item(ObjectType::from_raw(1), certificate.into_item())
        .expect("typed dispatch");
    assert_eq!(typed.object_type().raw(), 1);
    assert!(format!("{typed:?}").contains("item_tag"));
    assert_eq!(typed.into_item().tag().raw(), 0x0042_0013);
}

#[test]
fn object_schema_readers_reject_wrong_order_duplicates_types_and_missing_fields() {
    let enum_type = || Item::new(tag(0x0042_001D), Value::enumeration(1)).expect("type field");
    let bytes = || Item::new(tag(0x0042_001E), Value::byte_string(vec![1])).expect("value field");

    assert!(Certificate::try_from_item(structure_item(0x0042_0013, [])).is_err());
    assert!(
        Certificate::try_from_item(structure_item(0x0042_0013, [bytes(), enum_type()])).is_err()
    );
    assert!(
        Certificate::try_from_item(structure_item(0x0042_0013, [enum_type(), enum_type()]))
            .is_err()
    );
    assert!(
        Certificate::try_from_item(structure_item(
            0x0042_0013,
            [Item::new(tag(0x0042_001D), Value::integer(1)).expect("wrong type")],
        ))
        .is_err()
    );
    assert!(
        Certificate::try_from_item(structure_item(0x0042_0018, [enum_type(), bytes()])).is_err()
    );

    let known_tag_unknown_type = Certificate::try_new(1, vec![1]).unwrap().into_item();
    assert!(
        ManagedObject::try_from_item(ObjectType::from_raw(99), known_tag_unknown_type).is_err()
    );
    assert!(
        ManagedObject::unknown(ObjectType::from_raw(1), structure_item(0x0054_0101, []),).is_err()
    );
    assert!(
        ManagedObject::unknown(
            ObjectType::from_raw(99),
            Item::new(tag(0x0054_0101), Value::integer(1)).expect("non-structure"),
        )
        .is_err()
    );
}

fn tag(raw: u32) -> kmipkit_ttlv::Tag {
    RawTag::new(raw)
        .expect("tag fits")
        .try_checked()
        .expect("assigned tag")
}
