use kmipkit_ttlv::codec::decode;
use kmipkit_ttlv::{ItemType, ValueView};

// OASIS KMIP Specification v2.1, §§10.1.1–10.1.3 and §11.23; traceability
// specification requirement-NR-001, specification requirement-NR-002, and specification requirement-NR-004.
#[test]
fn external_callers_can_decode_a_complete_public_item() {
    let wire = [
        0x42, 0x01, 0x73, 0x02, 0, 0, 0, 4, 0x80, 0, 0, 0, 0, 0, 0, 0,
    ];
    let item = decode(&wire).expect("the external fixture is a valid Integer item");

    assert_eq!(item.tag().raw(), 0x0042_0173);
    assert_eq!(item.item_type(), ItemType::Integer);
    assert!(
        item.with_value(|value| matches!(value, ValueView::Integer(actual) if *actual == i32::MIN))
    );
}
