//! KMIPKit-owned object payloads follow the generic TTLV redaction contract.

use kmipkit_protocol::{KeyMaterial, KeyValue};

const SECRET: &[u8] = b"managed-key-secret-sentinel";

#[test]
fn secret_material_and_containing_values_never_format_payload_bytes() {
    let material = KeyMaterial::byte_string(1, SECRET.to_vec()).expect("Raw material bytes");
    let material_debug = format!("{material:?}");
    assert!(!material_debug.contains("managed-key-secret-sentinel"));

    let value = KeyValue::try_new(material, None).expect("valid Key Value");
    let value_debug = format!("{value:?}");
    assert!(!value_debug.contains("managed-key-secret-sentinel"));

    let item = value.into_item();
    let item_debug = format!("{item:?}");
    assert!(!item_debug.contains("managed-key-secret-sentinel"));
}

#[test]
fn typed_secret_owner_moves_into_zeroizing_ttlv_without_hidden_clone() {
    // The move transfers the one owned TTLV payload. Drop-time zeroization is
    // covered by kmipkit-ttlv/tests/value_zeroization.rs for nested Values.
    let material = KeyMaterial::byte_string(1, SECRET.to_vec()).expect("Raw material bytes");
    let value = KeyValue::try_new(material, None).expect("valid Key Value");
    let item = value.into_item();
    let has_secret = item.with_value(|value| match value {
        kmipkit_ttlv::ValueView::Structure(structure) => structure
            .children()
            .iter()
            .any(|field| field.tag().raw() == 0x0042_0043),
        _ => false,
    });
    assert!(has_secret);
}
