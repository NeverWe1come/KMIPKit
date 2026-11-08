//! Unit tests for `LinkNormalizer` (KMIP 2.x generic link <-> KMIP 3.0 typed link) (KMIPKIT-0071).

use crate::link_normalizer::{LinkNormalizer, LinkType};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

fn ttlv_item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag).unwrap().try_checked().unwrap();
    Item::new(tag, value).unwrap()
}

#[test]
fn test_generic_link_converts_to_discrete_v3_typed_link() {
    let target_id = 0xCAFE_BABE;

    // Build a KMIP 2.x generic Link Structure (0x0042_0047)
    let mut link_struct = Structure::new();
    link_struct
        .try_push(ttlv_item(0x0042_0048, Value::enumeration(0x0000_0101))) // Certificate link type
        .unwrap();
    link_struct
        .try_push(ttlv_item(0x0042_0049, Value::enumeration(target_id))) // Linked Object Identifier
        .unwrap();

    let generic_link = ttlv_item(0x0042_0047, Value::structure(link_struct));

    let typed_link =
        LinkNormalizer::normalize_to_v3(&generic_link).expect("valid conversion to v3");
    // KMIP 3.0 §4.35.1 / §11.58 assigns Certificate Link tag 0x0042_0190.
    assert_eq!(typed_link.tag().raw(), 0x0042_0190);

    // Roundtrip back to v2 generic link
    let roundtripped =
        LinkNormalizer::normalize_to_v2(&typed_link).expect("valid conversion to v2");
    assert_eq!(roundtripped.tag().raw(), 0x0042_0047);
}

#[test]
fn test_all_fourteen_kmip_21_link_values_map_to_semantic_v3_attributes() {
    // KMIP 2.1 §11.28, Table 465, and KMIP 3.0 §§4.35.1–4.35.20,
    // §11.58. This is deliberately a semantic table, not an arithmetic range.
    let link_cases = [
        (0x0000_0101, 0x0042_0190), // Certificate Link
        (0x0000_0102, 0x0042_019A), // Public Key Link
        (0x0000_0103, 0x0042_0199), // Private Key Link
        (0x0000_0104, 0x0042_0192), // Derivation Base Object -> Derivation Object Link
        (0x0000_0105, 0x0042_0193), // Derived Key -> Derived Object Link
        (0x0000_0106, 0x0042_019C), // Replacement Object Link
        (0x0000_0107, 0x0042_019B), // Replaced Object Link
        (0x0000_0108, 0x0042_0195), // Parent Link
        (0x0000_0109, 0x0042_0191), // Child Link
        (0x0000_010A, 0x0042_0198), // Previous Link
        (0x0000_010B, 0x0042_0194), // Next Link
        (0x0000_010C, 0x0042_0196), // PKCS#12 Certificate Link
        (0x0000_010D, 0x0042_0197), // PKCS#12 Password Link
        (0x0000_010E, 0x0042_019D), // Wrapping Key Link
    ];

    for (v2_enum, v3_tag) in link_cases {
        assert!(
            RawTag::new(v3_tag)
                .expect("KMIP tag is 24-bit")
                .try_checked()
                .is_ok()
        );
        let link_type = LinkType::from_v2_enumeration(v2_enum)
            .unwrap_or_else(|| panic!("assigned KMIP 2.1 Link Type {v2_enum:#010X}"));
        assert_eq!(link_type.v2_enumeration(), v2_enum);
        assert_eq!(link_type.v3_tag(), v3_tag);
        assert_eq!(LinkType::from_v3_tag(v3_tag), Some(link_type));

        let mut link_struct = Structure::new();
        link_struct
            .try_push(ttlv_item(0x0042_0048, Value::enumeration(v2_enum)))
            .unwrap();
        link_struct
            .try_push(ttlv_item(0x0042_0049, Value::enumeration(0xCAFE_BABE)))
            .unwrap();

        let generic_link = ttlv_item(0x0042_0047, Value::structure(link_struct));

        let v3_item = LinkNormalizer::normalize_to_v3(&generic_link).unwrap();
        assert_eq!(v3_item.tag().raw(), v3_tag);

        let v2_item = LinkNormalizer::normalize_to_v2(&v3_item).unwrap();
        assert_eq!(v2_item.tag().raw(), 0x0042_0047);
        let restored_type = v2_item.with_value(|view| match view {
            kmipkit_ttlv::ValueView::Structure(structure) => structure
                .children()
                .iter()
                .find(|child| child.tag().raw() == 0x0042_0048)
                .map_or(0, |child| {
                    child.with_value(|value| match value {
                        kmipkit_ttlv::ValueView::Enumeration(raw) => *raw,
                        _ => 0,
                    })
                }),
            _ => 0,
        });
        assert_eq!(restored_type, v2_enum);
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

    // KMIP 3.0 §4.35 and §11.58 assign all 20 discrete link attribute tags.
    // Six have no KMIP 2.1 Link Type equivalent and must not be normalized.
    for v3_only_tag in [
        0x0042_01A0, // Credential Link
        0x0042_01A5, // Password Link
        0x0042_01B3, // Group Link
        0x0042_01B4, // Split Key Base Link
        0x0042_01B5, // Joined Split Key Parts Link
        0x0042_01BC, // Certificate Request Link
    ] {
        assert!(
            RawTag::new(v3_only_tag)
                .expect("KMIP tag is 24-bit")
                .try_checked()
                .is_ok()
        );
        assert!(LinkType::from_v3_tag(v3_only_tag).is_none());
        let item = ttlv_item(v3_only_tag, Value::text_string("target".into()));
        assert!(LinkNormalizer::normalize_to_v2(&item).is_err());
    }

    // KMIP 3.0 Object Defaults is not a link tag even though it was previously
    // misclassified by the invalid contiguous-range implementation.
    assert!(LinkType::from_v3_tag(0x0042_0153).is_none());
    assert!(
        LinkNormalizer::normalize_to_v2(&ttlv_item(
            0x0042_0153,
            Value::text_string("target".into()),
        ))
        .is_err()
    );
}

#[test]
fn test_link_identifier_enumeration_and_integer_forms_roundtrip_without_coercion() {
    for (identifier, is_enumeration, expected_raw) in [
        (
            Value::enumeration(0xCAFE_BABE),
            true,
            i64::from(0xCAFE_BABEu32),
        ),
        (Value::integer(-7), false, -7),
    ] {
        let mut link_struct = Structure::new();
        link_struct
            .try_push(ttlv_item(0x0042_0048, Value::enumeration(0x0000_0101)))
            .unwrap();
        link_struct
            .try_push(ttlv_item(0x0042_0049, identifier))
            .unwrap();
        let generic = ttlv_item(0x0042_0047, Value::structure(link_struct));

        let discrete = LinkNormalizer::normalize_to_v3(&generic)
            .expect("the identifier encoding has a v3 equivalent");
        assert_eq!(discrete.tag().raw(), 0x0042_0190);
        let discrete_raw = discrete.with_value(|value| match (is_enumeration, value) {
            (true, kmipkit_ttlv::ValueView::Enumeration(raw)) => i64::from(*raw),
            (false, kmipkit_ttlv::ValueView::Integer(raw)) => i64::from(*raw),
            _ => 0,
        });
        assert_eq!(discrete_raw, expected_raw);
        let restored = LinkNormalizer::normalize_to_v2(&discrete)
            .expect("the identifier encoding has a v2 equivalent");
        let restored_raw = restored.with_value(|view| match view {
            kmipkit_ttlv::ValueView::Structure(structure) => structure
                .children()
                .iter()
                .find(|child| child.tag().raw() == 0x0042_0049)
                .map_or(0, |child| {
                    child.with_value(|value| match (is_enumeration, value) {
                        (true, kmipkit_ttlv::ValueView::Enumeration(raw)) => i64::from(*raw),
                        (false, kmipkit_ttlv::ValueView::Integer(raw)) => i64::from(*raw),
                        _ => 0,
                    })
                }),
            _ => 0,
        });
        assert_eq!(restored_raw, expected_raw);
    }
}

#[test]
fn test_generic_link_with_unmapped_children_is_rejected_without_dropping_them() {
    let mut link_struct = Structure::new();
    link_struct
        .try_push(ttlv_item(0x0042_0048, Value::enumeration(0x0000_0101)))
        .unwrap();
    link_struct
        .try_push(ttlv_item(
            0x0042_0049,
            Value::text_string("linked-obj-uuid".to_string()),
        ))
        .unwrap();
    link_struct
        .try_push(ttlv_item(0x0042_0001, Value::integer(99)))
        .unwrap();
    let generic = ttlv_item(0x0042_0047, Value::structure(link_struct));

    assert!(LinkNormalizer::normalize_to_v3(&generic).is_err());
    assert_eq!(generic.item_type(), kmipkit_ttlv::ItemType::Structure);
}

#[test]
fn test_text_string_identifiers_without_v3_encoding_are_rejected() {
    let mut link_struct = Structure::new();
    link_struct
        .try_push(ttlv_item(0x0042_0048, Value::enumeration(0x0000_0101)))
        .unwrap();
    link_struct
        .try_push(ttlv_item(
            0x0042_0049,
            Value::text_string("linked-obj-uuid".to_string()),
        ))
        .unwrap();
    let generic = ttlv_item(0x0042_0047, Value::structure(link_struct));

    assert!(LinkNormalizer::normalize_to_v3(&generic).is_err());
    assert!(
        LinkNormalizer::normalize_to_v2(&ttlv_item(
            0x0042_0190,
            Value::text_string("linked-obj-uuid".to_string()),
        ))
        .is_err()
    );
}
