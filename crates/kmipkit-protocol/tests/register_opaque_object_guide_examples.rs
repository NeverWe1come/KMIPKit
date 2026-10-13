//! Both language guides must explain the caller-selected Opaque Object context.

const ENGLISH_GUIDE: &str =
    include_str!("../../../docs/user-guide/en/lifecycle-and-certificates.md");
const SPANISH_GUIDE: &str =
    include_str!("../../../docs/user-guide/es/operaciones-ciclo-vida-y-certificados.md");

use kmipkit_protocol::{AttributeSet, ObjectType, OpaqueObjectContext, RegisterRequest};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

#[test]
fn both_guides_cover_default_and_explicit_managed_opaque_context() {
    for guide in [ENGLISH_GUIDE, SPANISH_GUIDE] {
        assert!(guide.contains("```rust,kmipkit-test"));
        assert!(guide.contains("OpaqueObjectContext::Generic"));
        assert!(guide.contains("OpaqueObjectContext::ManagedCryptographicObject"));
        assert!(guide.contains("with_opaque_object_context"));
        assert!(guide.contains("Cryptographic Usage Mask"));
        assert!(guide.contains("Cryptographic Algorithm"));
        assert!(guide.contains("Cryptographic Length"));
        assert!(guide.contains("KMIPKIT-REQ-SPEC-2.3-002"));
        assert!(guide.contains("KMIPKIT-REQ-SPEC-2.3-003"));
    }
}

#[test]
fn documented_context_examples_use_the_standard_register_shape() {
    let generic = opaque_register(
        OpaqueObjectContext::Generic,
        AttributeSet::try_new([]).unwrap(),
    );
    let managed_attributes = AttributeSet::try_new([
        item(0x0042_002c, Value::integer(3)),
        item(0x0042_0028, Value::enumeration(3)),
        item(0x0042_002a, Value::integer(256)),
    ])
    .unwrap();
    let managed = opaque_register(
        OpaqueObjectContext::ManagedCryptographicObject,
        managed_attributes,
    );

    for request in [generic, managed] {
        let payload = request.into_ttlv_payload().unwrap();
        assert_eq!(
            payload
                .view()
                .children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
            [0x0042_0057, 0x0042_0125, 0x0042_005b]
        );
    }
}

fn opaque_register(context: OpaqueObjectContext, attributes: AttributeSet) -> RegisterRequest {
    let mut opaque = Structure::new();
    opaque
        .try_push(item(0x0042_0059, Value::enumeration(1)))
        .unwrap();
    opaque
        .try_push(item(0x0042_005a, Value::byte_string(vec![0x01, 0x02])))
        .unwrap();
    RegisterRequest::new(
        ObjectType::from_raw(8),
        attributes,
        item(0x0042_005b, Value::structure(opaque)),
    )
    .with_opaque_object_context(context)
}

fn item(tag: u32, value: Value) -> Item {
    Item::new(RawTag::new(tag).unwrap().try_checked().unwrap(), value).unwrap()
}
