//! Client registry enforcement for schema-specific resource limits.
//!
//! Traceability: KMIPKIT-0012-FR-012. These configured registry limits are
//! `KMIPKit` policy. OASIS KMIP v2.1 §9.13, Table 418 defines the enclosing
//! Message Extension and Vendor Extension Structure, not schema resource
//! limits for vendor payloads.

use kmipkit_client::{ClientError, extension_registry};
use kmipkit_protocol::{ProtocolErrorKind, extension};
use kmipkit_ttlv::{ItemType, RawTag, Tag, Value};

const TAG_BASE: u32 = 0x0054_0000;
const DISCRIMINATOR_TAG: u32 = 1;
const FIRST_PAYLOAD_TAG: u32 = 100;

fn tag(offset: u32) -> Tag {
    RawTag::new(TAG_BASE + offset)
        .expect("schema limit fixture tag fits in the KMIP Tag width")
        .try_checked()
        .expect("schema limit fixture uses the KMIP extension allocation")
}

fn identity(name: &str) -> extension::ExtensionIdentity {
    extension::extension_identity("example.vendor", name, "1")
        .expect("schema limit fixture identity is valid")
}

fn discriminator() -> extension::Discriminator {
    let path =
        extension::ttlv_path(tag(DISCRIMINATOR_TAG)).expect("the discriminator path is non-empty");
    extension::discriminator(path, Value::text_string("schema-limits-v1".into()))
        .expect("the discriminator is a valid scalar")
}

fn definition(
    name: &str,
    mut payload_rules: Vec<extension::ExtensionChildRule>,
    order: Vec<extension::ExtensionOrderConstraint>,
) -> extension::ExtensionDefinition {
    let mut children = vec![
        extension::required(
            tag(DISCRIMINATOR_TAG),
            extension::scalar(ItemType::TextString).expect("Text String is supported"),
        )
        .expect("the discriminator child rule is valid"),
    ];
    children.append(&mut payload_rules);
    let schema = extension::structure(children, order, false)
        .expect("schema limit fixture is structurally valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the fixture supports this workspace version");
    extension::extension_definition(identity(name), compatibility, discriminator(), schema)
        .expect("the discriminator is declared by the root schema")
}

fn limits(
    schema_nodes: u64,
    child_rules: u64,
    constraint_members_per_rule: u64,
    total_constraint_members: u64,
    depth: u64,
) -> extension::ExtensionRegistryLimits {
    extension::with_values(
        256,
        schema_nodes,
        child_rules,
        4_096,
        1_048_576,
        4_096,
        1_048_576,
        constraint_members_per_rule,
        total_constraint_members,
        200_000,
        1_048_576,
        depth,
    )
    .expect("the selected schema limits are within their hard maxima")
}

fn required_integer(offset: u32) -> extension::ExtensionChildRule {
    extension::required(
        tag(offset),
        extension::scalar(ItemType::Integer).expect("Integer is supported"),
    )
    .expect("Integer payload rule is valid")
}

fn allowed_enum(values: std::ops::Range<u32>) -> extension::ExtensionSchema {
    values.fold(
        extension::scalar(ItemType::Enumeration).expect("Enumeration is supported"),
        |schema, value| {
            extension::with_allowed_enumeration(schema, value)
                .expect("the enum set remains below the hard per-rule maximum")
        },
    )
}

fn client_error_kind(result: Result<extension_registry::ClientExtensionRegistry, ClientError>) {
    let error = result.expect_err("configured schema limits reject this definition");
    assert!(matches!(
        error,
        ClientError::Protocol { error, .. }
            if error.kind() == ProtocolErrorKind::ResourceLimit
    ));
}

#[test]
fn configured_schema_depth_limit_is_enforced_for_nested_nodes() {
    let nested = extension::structure(
        vec![
            extension::required(
                tag(900),
                extension::structure(vec![required_integer(901)], Vec::new(), false)
                    .expect("middle Structure schema is valid"),
            )
            .expect("nested Structure rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("nested Structure schema is valid");
    let nested_definition = definition(
        "configured-depth",
        vec![
            extension::required(tag(FIRST_PAYLOAD_TAG), nested)
                .expect("nested payload rule is valid"),
        ],
        Vec::new(),
    );
    client_error_kind(extension_registry::client_extension_registry(
        vec![nested_definition],
        limits(100, 256, 256, 16_384, 2),
    ));
}

#[test]
fn configured_child_width_limit_is_enforced() {
    let exact_width = definition(
        "configured-width-exact",
        vec![required_integer(FIRST_PAYLOAD_TAG)],
        Vec::new(),
    );
    assert!(
        extension_registry::client_extension_registry(
            vec![exact_width],
            limits(100, 2, 256, 16_384, 64),
        )
        .is_ok(),
        "the discriminator and one payload rule exactly fit the configured width"
    );

    let too_wide = definition(
        "configured-width",
        vec![
            required_integer(FIRST_PAYLOAD_TAG),
            required_integer(FIRST_PAYLOAD_TAG + 1),
        ],
        Vec::new(),
    );
    client_error_kind(extension_registry::client_extension_registry(
        vec![too_wide],
        limits(100, 2, 256, 16_384, 64),
    ));
}

#[test]
fn configured_per_rule_constraint_limit_is_enforced() {
    let exact_members = definition(
        "configured-rule-members-exact",
        vec![
            extension::required(tag(FIRST_PAYLOAD_TAG), allowed_enum(0..1))
                .expect("single-member enum payload rule is valid"),
        ],
        Vec::new(),
    );
    assert!(
        extension_registry::client_extension_registry(
            vec![exact_members],
            limits(100, 256, 1, 16_384, 64),
        )
        .is_ok(),
        "one enum value exactly fits the configured per-rule constraint limit"
    );

    let enum_schema = allowed_enum(0..2);
    let constrained = definition(
        "configured-rule-members",
        vec![
            extension::required(tag(FIRST_PAYLOAD_TAG), enum_schema)
                .expect("enum payload rule is valid"),
        ],
        Vec::new(),
    );
    client_error_kind(extension_registry::client_extension_registry(
        vec![constrained],
        limits(100, 256, 1, 16_384, 64),
    ));
}

#[test]
fn configured_per_rule_order_edge_limit_is_enforced() {
    let exact_edge = definition(
        "configured-order-members-exact",
        vec![required_integer(200), required_integer(201)],
        vec![
            extension::extension_order_constraint(tag(200), tag(201))
                .expect("the exact-boundary edge is valid"),
        ],
    );
    assert!(
        extension_registry::client_extension_registry(
            vec![exact_edge],
            limits(100, 256, 1, 16_384, 64),
        )
        .is_ok(),
        "one order edge exactly fits the configured per-rule constraint limit"
    );

    let order = vec![
        extension::extension_order_constraint(tag(200), tag(201))
            .expect("first order edge is valid"),
        extension::extension_order_constraint(tag(201), tag(202))
            .expect("second order edge is valid"),
    ];
    let constrained = definition(
        "configured-order-members",
        vec![
            required_integer(200),
            required_integer(201),
            required_integer(202),
        ],
        order,
    );
    client_error_kind(extension_registry::client_extension_registry(
        vec![constrained],
        limits(100, 256, 1, 16_384, 64),
    ));
}

#[test]
fn configured_aggregate_node_and_constraint_counters_accept_exact_and_reject_one_over() {
    let exact_nodes = definition(
        "node-exact",
        vec![required_integer(FIRST_PAYLOAD_TAG)],
        Vec::new(),
    );
    assert!(
        extension_registry::client_extension_registry(
            vec![exact_nodes],
            limits(3, 256, 256, 16_384, 64),
        )
        .is_ok()
    );
    let extra_node = definition(
        "node-over",
        vec![required_integer(FIRST_PAYLOAD_TAG)],
        Vec::new(),
    );
    client_error_kind(extension_registry::client_extension_registry(
        vec![extra_node],
        limits(2, 256, 256, 16_384, 64),
    ));

    let two_members = definition(
        "constraint-exact",
        vec![
            extension::required(tag(FIRST_PAYLOAD_TAG), allowed_enum(0..1))
                .expect("first enum rule is valid"),
            extension::required(tag(FIRST_PAYLOAD_TAG + 1), allowed_enum(1..2))
                .expect("second enum rule is valid"),
        ],
        Vec::new(),
    );
    assert!(
        extension_registry::client_extension_registry(
            vec![two_members],
            limits(100, 256, 1, 2, 64),
        )
        .is_ok()
    );
    let over_total = definition(
        "constraint-over",
        vec![
            extension::required(tag(FIRST_PAYLOAD_TAG), allowed_enum(0..1))
                .expect("first enum rule is valid"),
            extension::required(tag(FIRST_PAYLOAD_TAG + 1), allowed_enum(1..2))
                .expect("second enum rule is valid"),
        ],
        Vec::new(),
    );
    client_error_kind(extension_registry::client_extension_registry(
        vec![over_total],
        limits(100, 256, 1, 1, 64),
    ));

    let mixed_constraints = definition(
        "mixed-constraints",
        vec![
            extension::required(tag(200), allowed_enum(0..1)).expect("enum payload rule is valid"),
            required_integer(201),
        ],
        vec![
            extension::extension_order_constraint(tag(200), tag(201))
                .expect("mixed order edge is valid"),
        ],
    );
    assert!(
        extension_registry::client_extension_registry(
            vec![mixed_constraints],
            limits(100, 256, 1, 2, 64),
        )
        .is_ok()
    );
}

fn enum_rules(count: u32, values_per_rule: u32) -> Vec<extension::ExtensionChildRule> {
    (0..count)
        .map(|index| {
            extension::required(
                tag(FIRST_PAYLOAD_TAG + index),
                allowed_enum(0..values_per_rule),
            )
            .expect("enum limit fixture child rule is valid")
        })
        .collect()
}

#[test]
fn default_child_width_boundary_is_enforced() {
    let at_default_width = definition(
        "default-width-exact",
        (0..255)
            .map(|offset| required_integer(FIRST_PAYLOAD_TAG + offset))
            .collect(),
        Vec::new(),
    );
    assert!(
        extension_registry::client_extension_registry(
            vec![at_default_width],
            extension::defaults(),
        )
        .is_ok()
    );
    let above_default_width = definition(
        "default-width-over",
        (0..256)
            .map(|offset| required_integer(FIRST_PAYLOAD_TAG + offset))
            .collect(),
        Vec::new(),
    );
    client_error_kind(extension_registry::client_extension_registry(
        vec![above_default_width],
        extension::defaults(),
    ));
}

#[test]
fn default_per_rule_constraint_boundary_is_enforced() {
    let at_default_members = definition(
        "default-rule-exact",
        vec![
            extension::required(tag(FIRST_PAYLOAD_TAG), allowed_enum(0..256))
                .expect("256 enum values fit the default per-rule limit"),
        ],
        Vec::new(),
    );
    assert!(
        extension_registry::client_extension_registry(
            vec![at_default_members],
            extension::defaults(),
        )
        .is_ok()
    );
    let above_default_members = definition(
        "default-rule-over",
        vec![
            extension::required(tag(FIRST_PAYLOAD_TAG), allowed_enum(0..257))
                .expect("257 enum values remain below the hard per-rule limit"),
        ],
        Vec::new(),
    );
    client_error_kind(extension_registry::client_extension_registry(
        vec![above_default_members],
        extension::defaults(),
    ));
}

#[test]
fn default_aggregate_schema_node_and_constraint_boundaries_are_enforced() {
    let exact_nodes = definition_with_schema_node_total("default-nodes-exact", 16_384);
    let raised_width = limits(16_384, 4_096, 4_096, 16_384, 64);
    assert!(extension_registry::client_extension_registry(vec![exact_nodes], raised_width).is_ok());
    let over_nodes = definition_with_schema_node_total("default-nodes-over", 16_385);
    assert!(matches!(
        extension_registry::client_extension_registry(vec![over_nodes], raised_width),
        Err(ClientError::Protocol { error, .. })
            if error.kind() == ProtocolErrorKind::ResourceLimit
    ));

    let at_total = definition("default-total-exact", enum_rules(64, 256), Vec::new());
    assert!(
        extension_registry::client_extension_registry(vec![at_total], extension::defaults(),)
            .is_ok()
    );
    let mut over_rules = enum_rules(64, 256);
    let first = over_rules.remove(0);
    let more_members = extension::required(tag(FIRST_PAYLOAD_TAG), allowed_enum(0..257))
        .expect("one rule can hold 257 values under the hard maximum");
    drop(first);
    over_rules.push(more_members);
    let over_total = definition("default-total-over", over_rules, Vec::new());
    assert!(matches!(
        extension_registry::client_extension_registry(vec![over_total], extension::defaults()),
        Err(ClientError::Protocol { error, .. })
            if error.kind() == ProtocolErrorKind::ResourceLimit
    ));
}

#[test]
fn hard_aggregate_constraint_boundary_accepts_exact_total_and_rejects_one_over() {
    let build_rules = |last_rule_values| {
        (0..25_u32)
            .map(|index| {
                let value_count = if index == 24 { last_rule_values } else { 4_096 };
                extension::required(tag(FIRST_PAYLOAD_TAG + index), allowed_enum(0..value_count))
                    .expect("hard-boundary enum rule is valid")
            })
            .collect::<Vec<_>>()
    };
    let hard_limits = limits(100_000, 4_096, 4_096, 100_000, 64);
    let exact = definition("hard-constraints-exact", build_rules(1_696), Vec::new());
    assert!(extension_registry::client_extension_registry(vec![exact], hard_limits).is_ok());

    let over = definition("hard-constraints-over", build_rules(1_697), Vec::new());
    client_error_kind(extension_registry::client_extension_registry(
        vec![over],
        hard_limits,
    ));
}

#[test]
fn hard_aggregate_schema_node_boundary_accepts_exact_total_and_rejects_one_over() {
    let hard_limits = limits(100_000, 4_096, 4_096, 100_000, 64);
    let exact = definition_with_schema_node_total_and_groups("hard-nodes-exact", 100_000, 25);
    assert!(extension_registry::client_extension_registry(vec![exact], hard_limits).is_ok());

    let over = definition_with_schema_node_total_and_groups("hard-nodes-over", 100_001, 25);
    client_error_kind(extension_registry::client_extension_registry(
        vec![over],
        hard_limits,
    ));
}

fn definition_with_schema_node_total(
    name: &str,
    total_nodes: usize,
) -> extension::ExtensionDefinition {
    definition_with_schema_node_total_and_groups(name, total_nodes, 4)
}

fn definition_with_schema_node_total_and_groups(
    name: &str,
    total_nodes: usize,
    group_count: u32,
) -> extension::ExtensionDefinition {
    // The root and discriminator scalar, plus each nested group Structure,
    // account for `2 + group_count` schema nodes; the rest are leaf nodes.
    let leaves = total_nodes - 2 - group_count as usize;
    let base = leaves / group_count as usize;
    let remainder = leaves % group_count as usize;
    let mut payload_rules = Vec::new();
    for group in 0..group_count {
        let leaf_count = base + usize::from((group as usize) < remainder);
        let nested_children = (0..leaf_count)
            .map(|index| {
                let offset = 10_000 + u32::try_from(index).expect("fixture leaf index fits u32");
                required_integer(offset)
            })
            .collect();
        let nested = extension::structure(nested_children, Vec::new(), false)
            .expect("nested fixture width remains below the hard maximum");
        payload_rules.push(
            extension::required(tag(2_000 + group), nested)
                .expect("nested group payload rule is valid"),
        );
    }
    definition(name, payload_rules, Vec::new())
}

#[test]
fn hard_schema_limit_configuration_rejects_one_above_each_schema_maximum() {
    for field in [
        (100_001, 4_096, 4_096, 100_000, 64),
        (100_000, 4_097, 4_096, 100_000, 64),
        (100_000, 4_096, 4_097, 100_000, 64),
        (100_000, 4_096, 4_096, 100_001, 64),
        (100_000, 4_096, 4_096, 100_000, 65),
    ] {
        let result = extension::with_values(
            1_024,
            field.0,
            field.1,
            4_096,
            16 * 1_024 * 1_024,
            4_096,
            16 * 1_024 * 1_024,
            field.2,
            field.3,
            200_000,
            4_194_304,
            field.4,
        );
        assert_eq!(
            result
                .expect_err("configured values cannot exceed the hard schema maximum")
                .kind(),
            ProtocolErrorKind::ResourceLimit
        );
    }
}
