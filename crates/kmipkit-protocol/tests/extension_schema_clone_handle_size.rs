use kmipkit_protocol::{ProtocolErrorKind, extension};
use kmipkit_ttlv::{ItemType, RawTag, Tag};
use std::mem::size_of;

use kmipkit_protocol::extension::ExtensionSchema;

fn tag(offset: u32) -> Tag {
    RawTag::new(0x0054_0000 + offset)
        .expect("schema test tag fits the KMIP Tag width")
        .try_checked()
        .expect("schema test tag is in the vendor allocation")
}

#[test]
fn extension_schema_clone_handle_stays_pointer_sized() {
    assert!(
        size_of::<ExtensionSchema>() <= 2 * size_of::<usize>(),
        "schema composition clones must copy a shared handle, not recursively owned trees"
    );
}

#[test]
fn structure_construction_rejects_aggregate_nodes_above_the_hard_limit() {
    let leaf = extension::scalar(ItemType::TextString).expect("scalar schema is valid");
    let inner = extension::structure(
        (1..=1_024)
            .map(|offset| {
                extension::required(tag(offset), leaf.clone()).expect("child rule is valid")
            })
            .collect(),
        Vec::new(),
        false,
    )
    .expect("inner structure is below its local limits");
    let outer_children = (1..=99)
        .map(|offset| {
            extension::required(tag(2_000 + offset), inner.clone())
                .expect("outer child rule is valid")
        })
        .collect();

    let result = extension::structure(outer_children, Vec::new(), false);
    let Err(error) = result else {
        panic!("the logical schema tree exceeds the 100,000-node hard limit");
    };

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
}

#[test]
fn structure_construction_rejects_aggregate_constraints_above_the_hard_limit() {
    let mut enumeration =
        extension::scalar(ItemType::Enumeration).expect("enumeration schema is valid");
    for value in 0..2_048 {
        enumeration = extension::with_allowed_enumeration(enumeration, value)
            .expect("constraint list is below its per-rule hard limit");
    }
    let children = (1..=50)
        .map(|offset| {
            extension::required(tag(4_000 + offset), enumeration.clone())
                .expect("child rule is valid")
        })
        .collect();

    let result = extension::structure(children, Vec::new(), false);
    let Err(error) = result else {
        panic!("the repeated logical schema exceeds the 100,000-member hard limit");
    };

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
}
