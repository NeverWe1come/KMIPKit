//! Public `Item` ownership contract for retaining a Get Any Object value.
//!
//! KMIP payload context: OASIS KMIP Specification v2.1 §6.1.19, Table 221
//! defines the successful Get response's Any Object. TTLV framing and tag
//! allocation follow §§10.1.1–10.1.3 and §11.56. This synthetic generic TTLV
//! fixture exercises the owned-copy contract only; it is not an official
//! OASIS conformance vector or a schema-valid Symmetric Key object.

use kmipkit_ttlv::codec::decode;
use kmipkit_ttlv::{Item, ItemType, ValueView};

const OBJECT_TAG: u32 = 0x0042_008F; // Symmetric Key
const KEY_MATERIAL_TAG: u32 = 0x0042_0043;
const UNKNOWN_REPEATED_TAG: u32 = 0x0054_0001;
const UNKNOWN_NESTED_TAG: u32 = 0x0054_0002;
const UNKNOWN_ENUMERATION_TAG: u32 = 0x0054_0003;
const SECRET_SENTINEL: &str = "object-secret-sentinel";

// Symmetric Key Structure containing a known child, repeated vendor-range
// descendants, and a nested unknown Enumeration. The item ordering is
// intentional and the payload is opaque to this contract test.
const OBJECT_WIRE: &[u8] = &[
    0x42, 0x00, 0x8F, 0x01, 0x00, 0x00, 0x00, 0x58, 0x42, 0x00, 0x43, 0x08, 0x00, 0x00, 0x00, 0x03,
    0x31, 0x72, 0xA4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x00, 0x01, 0x07, 0x00, 0x00, 0x00, 0x16,
    0x6F, 0x62, 0x6A, 0x65, 0x63, 0x74, 0x2D, 0x73, 0x65, 0x63, 0x72, 0x65, 0x74, 0x2D, 0x73, 0x65,
    0x6E, 0x74, 0x69, 0x6E, 0x65, 0x6C, 0x00, 0x00, 0x54, 0x00, 0x01, 0x08, 0x00, 0x00, 0x00, 0x03,
    0xA1, 0x00, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x10,
    0x54, 0x00, 0x03, 0x05, 0x00, 0x00, 0x00, 0x04, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00, 0x00, 0x00,
];

#[test]
fn try_clone_preserves_decoded_ttlv_and_owns_nested_payloads() {
    let original = decode(OBJECT_WIRE).expect("the synthetic TTLV fixture is well formed");
    let cloned = original
        .try_clone()
        .expect("an in-limit generic Item can be copied");

    assert_eq!(cloned.tag().raw(), OBJECT_TAG);
    assert_eq!(cloned.item_type(), ItemType::Structure);
    assert_eq!(encode_fixture_item(&cloned), OBJECT_WIRE);
    let child_tags = cloned.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(|child| child.tag().raw())
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    });
    assert_eq!(
        child_tags,
        [
            KEY_MATERIAL_TAG,
            UNKNOWN_REPEATED_TAG,
            UNKNOWN_REPEATED_TAG,
            UNKNOWN_NESTED_TAG,
        ]
    );
    let nested_enumeration = cloned.with_value(|value| match value {
        ValueView::Structure(structure) => {
            structure.children()[3].with_value(|nested| match nested {
                ValueView::Structure(nested) => {
                    let child = &nested.children()[0];
                    (
                        child.tag().raw(),
                        child.with_value(|value| match value {
                            ValueView::Enumeration(raw) => Some(*raw),
                            _ => None,
                        }),
                    )
                }
                _ => (0, None),
            })
        }
        _ => (0, None),
    });
    assert_eq!(
        nested_enumeration,
        (UNKNOWN_ENUMERATION_TAG, Some(0xDEAD_BEEF))
    );
    assert_ne!(
        byte_string_pointer(&original, &[2]),
        byte_string_pointer(&cloned, &[2])
    );

    let debug = format!("{cloned:?}");
    assert!(
        !debug.contains(SECRET_SENTINEL),
        "owned generic Item diagnostics must redact nested payloads"
    );

    drop(original);
    assert_eq!(encode_fixture_item(&cloned), OBJECT_WIRE);
}

fn byte_string_pointer(item: &Item, path: &[usize]) -> *const u8 {
    if let Some((first, remaining)) = path.split_first() {
        item.with_value(|value| match value {
            ValueView::Structure(structure) => {
                byte_string_pointer(&structure.children()[*first], remaining)
            }
            _ => std::ptr::null(),
        })
    } else {
        item.with_value(|value| match value {
            ValueView::ByteString(bytes) => bytes.as_ptr(),
            _ => std::ptr::null(),
        })
    }
}

// This test-local encoder covers the types in OBJECT_WIRE and gives the clone
// test an independent byte-level ordering/value assertion. Production TTLV
// encoding is outside this feature's public-surface Red test.
fn encode_fixture_item(item: &Item) -> Vec<u8> {
    let (item_type, payload) = item.with_value(|value| match value {
        ValueView::Structure(structure) => {
            let payload = structure
                .children()
                .iter()
                .flat_map(encode_fixture_item)
                .collect();
            (0x01, payload)
        }
        ValueView::Enumeration(raw) => (0x05, raw.to_be_bytes().to_vec()),
        ValueView::TextString(text) => (0x07, text.as_bytes().to_vec()),
        ValueView::ByteString(bytes) => (0x08, bytes.to_vec()),
        _ => {
            panic!("the fixture contains only Structure, Enumeration, Text String, and Byte String")
        }
    });

    let raw_tag = item.tag().raw().to_be_bytes();
    let mut encoded = vec![raw_tag[1], raw_tag[2], raw_tag[3], item_type];
    let payload_length = u32::try_from(payload.len()).expect("test fixture length fits in u32");
    encoded.extend_from_slice(&payload_length.to_be_bytes());
    encoded.extend_from_slice(&payload);
    let padding = (8 - payload.len() % 8) % 8;
    encoded.resize(encoded.len() + padding, 0);
    encoded
}
