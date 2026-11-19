use super::REGISTRY_LIMIT_SCHEMA_VISITS;
use crate::ProtocolErrorKind;
use crate::extension::{
    self, ExtensionRegistryLimits, required, scalar, structure, with_allowed_enumeration,
};
use kmipkit_ttlv::{ItemType, RawTag};

fn tag(offset: u32) -> kmipkit_ttlv::Tag {
    RawTag::new(0x0054_0000 + offset)
        .expect("aggregate preflight test tag fits")
        .try_checked()
        .expect("aggregate preflight test tag is valid")
}

fn default_limits() -> ExtensionRegistryLimits {
    extension::defaults()
}

fn visit_count() -> usize {
    REGISTRY_LIMIT_SCHEMA_VISITS.with(std::cell::Cell::get)
}

fn reset_visit_count() {
    REGISTRY_LIMIT_SCHEMA_VISITS.with(|visits| visits.set(0));
}

#[test]
fn aggregate_schema_node_limit_stops_at_first_excess_node() {
    let children = (1..=3)
        .map(|offset| {
            required(
                tag(offset),
                scalar(ItemType::TextString).expect("Text String is a supported scalar"),
            )
            .expect("test child rule is valid")
        })
        .collect();
    let schema = structure(children, Vec::new(), false).expect("wide test schema is valid");
    let mut limits = default_limits();
    limits.max_schema_nodes = 1;
    reset_visit_count();

    let error = schema
        .validate_registry_limits(&limits)
        .expect_err("the root plus its first child exceed the aggregate node limit");

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
    assert_eq!(
        visit_count(),
        2,
        "the remaining sibling schemas are not visited"
    );
}

#[test]
fn aggregate_constraint_limit_stops_at_first_excess_member() {
    let children = (1..=3)
        .map(|offset| {
            let enumeration = with_allowed_enumeration(
                scalar(ItemType::Enumeration).expect("Enumeration is a supported scalar"),
                offset,
            )
            .expect("one enumeration member is valid");
            required(tag(offset), enumeration).expect("test child rule is valid")
        })
        .collect();
    let schema = structure(children, Vec::new(), false).expect("wide test schema is valid");
    let mut limits = default_limits();
    limits.max_total_constraint_members = 1;
    reset_visit_count();

    let error = schema
        .validate_registry_limits(&limits)
        .expect_err("the second enumeration member exceeds the aggregate constraint limit");

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
    assert_eq!(visit_count(), 3, "later sibling schemas are not visited");
}
