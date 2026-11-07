//! Unit tests for `LinkNormalizer` (KMIP 2.x generic link <-> KMIP 3.0 typed link) (KMIPKIT-0071).

use crate::link_normalizer::{LinkNormalizer, LinkType};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

fn ttlv_item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag).unwrap().try_checked().unwrap();
    Item::new(tag, value).unwrap()
}

#[test]
fn test_generic_link_converts_to_discrete_v3_typed_link() {
    let target_id = "target-cert-id-12345";

    // Build a KMIP 2.x generic Link Structure (0x0042_0047)
    let mut link_struct = Structure::new();
    link_struct
        .try_push(ttlv_item(0x0042_0048, Value::enumeration(0x0000_0101))) // Certificate link type
        .unwrap();
    link_struct
        .try_push(ttlv_item(
            0x0042_0049,
            Value::text_string(target_id.to_string()),
        )) // Linked Object Identifier
        .unwrap();

    let generic_link = ttlv_item(0x0042_0047, Value::structure(link_struct));

    let typed_link =
        LinkNormalizer::normalize_to_v3(&generic_link).expect("valid conversion to v3");
    // Should have tag 0x0042_0153 (Certificate Link)
    assert_eq!(typed_link.tag().raw(), 0x0042_0153);

    // Roundtrip back to v2 generic link
    let roundtripped =
        LinkNormalizer::normalize_to_v2(&typed_link).expect("valid conversion to v2");
    assert_eq!(roundtripped.tag().raw(), 0x0042_0047);
}

#[test]
fn test_all_eleven_supported_link_types_roundtrip() {
    let link_cases = [
        (LinkType::Certificate, 0x0000_0101, 0x0042_0153),
        (LinkType::PublicKey, 0x0000_0102, 0x0042_0154),
        (LinkType::PrivateKey, 0x0000_0103, 0x0042_0155),
        (LinkType::PreviousKey, 0x0000_0104, 0x0042_0156),
        (LinkType::NextKey, 0x0000_0105, 0x0042_0157),
        (LinkType::DerivationBaseObject, 0x0000_0106, 0x0042_0158),
        (LinkType::DerivedKey, 0x0000_0107, 0x0042_0159),
        (LinkType::SplitKey, 0x0000_0108, 0x0042_015A),
        (LinkType::WrappingKey, 0x0000_0109, 0x0042_015B),
        (LinkType::ParentObject, 0x0000_010A, 0x0042_015C),
        (LinkType::ChildObject, 0x0000_010B, 0x0042_015D),
    ];

    for (_link_type, v2_enum, v3_tag) in link_cases {
        let mut link_struct = Structure::new();
        link_struct
            .try_push(ttlv_item(0x0042_0048, Value::enumeration(v2_enum)))
            .unwrap();
        link_struct
            .try_push(ttlv_item(
                0x0042_0049,
                Value::text_string("linked-obj-uuid".to_string()),
            ))
            .unwrap();

        let generic_link = ttlv_item(0x0042_0047, Value::structure(link_struct));

        let v3_item = LinkNormalizer::normalize_to_v3(&generic_link).unwrap();
        assert_eq!(v3_item.tag().raw(), v3_tag);

        let v2_item = LinkNormalizer::normalize_to_v2(&v3_item).unwrap();
        assert_eq!(v2_item.tag().raw(), 0x0042_0047);
    }
}

#[test]
fn test_link_normalizer_error_and_edge_cases() {
    // Unknown v2 enum
    assert!(LinkType::from_v2_enumeration(0x9999).is_none());

    // Unknown v3 tag
    assert!(LinkType::from_v3_tag(0x0042_9999).is_none());

    // Wrong tag for normalize_to_v3
    let wrong_tag_item = ttlv_item(0x0042_0001, Value::enumeration(1));
    assert!(LinkNormalizer::normalize_to_v3(&wrong_tag_item).is_err());

    // Generic link with unknown children and missing fields
    let mut incomplete_struct = Structure::new();
    incomplete_struct
        .try_push(ttlv_item(0x0042_0001, Value::integer(42)))
        .unwrap();
    let incomplete_item = ttlv_item(0x0042_0047, Value::structure(incomplete_struct));
    assert!(LinkNormalizer::normalize_to_v3(&incomplete_item).is_err());

    // Generic link with link type but no id
    let mut no_id_struct = Structure::new();
    no_id_struct
        .try_push(ttlv_item(0x0042_0048, Value::enumeration(0x0000_0101)))
        .unwrap();
    let no_id_item = ttlv_item(0x0042_0047, Value::structure(no_id_struct));
    assert!(LinkNormalizer::normalize_to_v3(&no_id_item).is_err());

    // Wrong tag for normalize_to_v2 (e.g. 0x0042_0001 is not a v3 link tag)
    let invalid_v3_tag_item = ttlv_item(0x0042_0001, Value::text_string("test".into()));
    assert!(LinkNormalizer::normalize_to_v2(&invalid_v3_tag_item).is_err());

    // normalize_to_v2 with structured v3 item containing Linked Object Identifier (0x0042_0049)
    let mut structured_v3 = Structure::new();
    structured_v3
        .try_push(ttlv_item(
            0x0042_0049,
            Value::text_string("linked-id-nested".into()),
        ))
        .unwrap();
    let structured_v3_item = ttlv_item(0x0042_0153, Value::structure(structured_v3));
    let v2_converted = LinkNormalizer::normalize_to_v2(&structured_v3_item).unwrap();
    assert_eq!(v2_converted.tag().raw(), 0x0042_0047);
}
