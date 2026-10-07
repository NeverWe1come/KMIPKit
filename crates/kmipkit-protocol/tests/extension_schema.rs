//! Complete vendor-schema validation tests.
//!
//! Traceability: KMIPKIT-0012-FR-005 through FR-007 and FR-012. The schema
//! grammar and its resource limits are `KMIPKit` commitments. OASIS KMIP v2.1
//! §9.13, Table 418 defines the enclosing Message Extension and
//! Vendor Extension Structure; it does not define vendor-specific child
//! schemas or these validation rules.

use kmipkit_protocol::{ProtocolError, ProtocolErrorKind, extension};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value};

const TAG_BASE: u32 = 0x0054_0000;
const DISCRIMINATOR_OFFSET: u32 = 1;
const PAYLOAD_OFFSET: u32 = 2;
const NESTED_OFFSET: u32 = 3;
const DISCRIMINATOR: &str = "schema-fixture-v1";

fn tag(offset: u32) -> Tag {
    RawTag::new(TAG_BASE + offset)
        .expect("schema fixture tag fits the KMIP Tag width")
        .try_checked()
        .expect("schema fixture tag is in the vendor allocation")
}

fn item(offset: u32, value: Value) -> Item {
    Item::new(tag(offset), value).expect("a checked schema fixture tag forms an Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut result = Structure::new();
    for item in items {
        result
            .try_push(item)
            .expect("schema fixture stays within the TTLV model depth");
    }
    result
}

fn discriminator_rule() -> extension::ExtensionChildRule {
    extension::required(
        tag(DISCRIMINATOR_OFFSET),
        extension::scalar(ItemType::TextString).expect("Text String is a supported scalar"),
    )
    .expect("the discriminator schema child is valid")
}

fn definition(
    rules: Vec<extension::ExtensionChildRule>,
    order: Vec<extension::ExtensionOrderConstraint>,
    preserve_undeclared: bool,
) -> extension::ExtensionDefinition {
    let mut children = vec![discriminator_rule()];
    children.extend(rules);
    let schema = extension::structure(children, order, preserve_undeclared)
        .expect("the test's root schema is structurally coherent");
    let identity = extension::extension_identity("example.vendor", "schema-test", "1")
        .expect("the schema test identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the schema fixture supports the workspace version");
    let path = extension::ttlv_path(tag(DISCRIMINATOR_OFFSET))
        .expect("the discriminator path is non-empty");
    let discriminator = extension::discriminator(path, Value::text_string(DISCRIMINATOR.into()))
        .expect("the discriminator is a valid scalar");
    extension::extension_definition(identity, compatibility, discriminator, schema)
        .expect("the discriminator resolves through the root schema")
}

fn extension_value(children: impl IntoIterator<Item = Item>) -> Structure {
    let mut values = vec![item(
        DISCRIMINATOR_OFFSET,
        Value::text_string(DISCRIMINATOR.into()),
    )];
    values.extend(children);
    structure(values)
}

fn required_payload(schema: extension::ExtensionSchema) -> extension::ExtensionChildRule {
    extension::required(tag(PAYLOAD_OFFSET), schema).expect("the payload child schema is valid")
}

fn assert_invalid_schema(result: Result<extension::ValidatedExtensionValue, ProtocolError>) {
    let error = result.expect_err("the payload violates its declared schema");
    assert_eq!(error.kind(), ProtocolErrorKind::InvalidSchema);
}

fn validate_one(
    schema: extension::ExtensionSchema,
    value: Value,
) -> Result<extension::ValidatedExtensionValue, ProtocolError> {
    let definition = definition(vec![required_payload(schema)], Vec::new(), false);
    extension::validate(
        &definition,
        extension_value([item(PAYLOAD_OFFSET, value)]),
        &CodecLimits::defaults(),
    )
}

#[test]
fn every_supported_scalar_item_type_is_checked_against_its_declared_type() {
    let cases = [
        (ItemType::Integer, Value::integer(-7)),
        (ItemType::LongInteger, Value::long_integer(-7)),
        (ItemType::BigInteger, Value::big_integer(vec![0x01, 0x02])),
        (ItemType::Enumeration, Value::enumeration(7)),
        (ItemType::Boolean, Value::boolean(true)),
        (ItemType::TextString, Value::text_string("vendor".into())),
        (ItemType::ByteString, Value::byte_string(vec![0x00, 0xff])),
        (ItemType::DateTime, Value::date_time(-7)),
        (ItemType::Interval, Value::interval(7)),
        (ItemType::DateTimeExtended, Value::date_time_extended(-7)),
    ];

    for (item_type, value) in cases {
        let schema = extension::scalar(item_type).expect("the declared scalar type is supported");
        assert!(
            validate_one(schema, value).is_ok(),
            "a value with declared type {item_type:?} must validate"
        );
    }

    let schema =
        extension::scalar(ItemType::Enumeration).expect("Enumeration is a supported scalar type");
    assert_invalid_schema(validate_one(schema, Value::integer(7)));
}

#[test]
fn structure_nodes_recurse_and_reject_missing_or_wrongly_typed_required_children() {
    let nested_schema = extension::structure(
        vec![
            extension::required(
                tag(NESTED_OFFSET),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("the nested required field is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("the nested Structure schema is valid");
    let definition = definition(vec![required_payload(nested_schema)], Vec::new(), false);

    let valid = extension_value([item(
        PAYLOAD_OFFSET,
        Value::structure(structure([item(NESTED_OFFSET, Value::integer(4))])),
    )]);
    assert!(extension::validate(&definition, valid, &CodecLimits::defaults()).is_ok());

    let missing = extension_value([item(PAYLOAD_OFFSET, Value::structure(Structure::new()))]);
    assert_invalid_schema(extension::validate(
        &definition,
        missing,
        &CodecLimits::defaults(),
    ));

    let wrong_type = extension_value([item(
        PAYLOAD_OFFSET,
        Value::structure(structure([item(
            NESTED_OFFSET,
            Value::structure(Structure::new()),
        )])),
    )]);
    assert_invalid_schema(extension::validate(
        &definition,
        wrong_type,
        &CodecLimits::defaults(),
    ));
}

#[test]
fn definitions_reject_unresolvable_or_wrong_type_discriminator_paths() {
    let identity = extension::extension_identity("example.vendor", "bad-path", "1")
        .expect("the malformed definition identity is otherwise valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the malformed definition supports the workspace version");
    let schema = extension::structure(vec![discriminator_rule()], Vec::new(), false)
        .expect("the root schema is valid");
    let missing_path = extension::ttlv_path(tag(70)).expect("the test path is non-empty");
    let discriminator =
        extension::discriminator(missing_path, Value::text_string(DISCRIMINATOR.into()))
            .expect("the discriminator scalar is valid");
    assert_eq!(
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect_err("a discriminator path must resolve through declared Structure rules")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );

    let identity = extension::extension_identity("example.vendor", "wrong-type", "1")
        .expect("the second malformed definition identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the second malformed definition supports this client");
    let wrong_type_schema = extension::structure(
        vec![
            extension::required(
                tag(DISCRIMINATOR_OFFSET),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("the wrong-type child is structurally valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("the wrong-type schema is structurally valid");
    let path =
        extension::ttlv_path(tag(DISCRIMINATOR_OFFSET)).expect("the second test path is non-empty");
    let discriminator = extension::discriminator(path, Value::text_string(DISCRIMINATOR.into()))
        .expect("the discriminator remains a text scalar");
    assert_eq!(
        extension::extension_definition(identity, compatibility, discriminator, wrong_type_schema,)
            .expect_err("terminal schema type must match the discriminator scalar type")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );
}

#[test]
fn validation_rejects_missing_repeated_and_wrong_type_discriminator_values() {
    let definition = definition(Vec::new(), Vec::new(), false);
    assert_invalid_schema(extension::validate(
        &definition,
        structure([]),
        &CodecLimits::defaults(),
    ));
    assert_invalid_schema(extension::validate(
        &definition,
        structure([
            item(
                DISCRIMINATOR_OFFSET,
                Value::text_string(DISCRIMINATOR.into()),
            ),
            item(
                DISCRIMINATOR_OFFSET,
                Value::text_string(DISCRIMINATOR.into()),
            ),
        ]),
        &CodecLimits::defaults(),
    ));
    assert_invalid_schema(extension::validate(
        &definition,
        structure([item(DISCRIMINATOR_OFFSET, Value::integer(1))]),
        &CodecLimits::defaults(),
    ));
}

#[test]
fn required_optional_and_repeated_cardinality_are_enforced() {
    let integer = extension::scalar(ItemType::Integer).expect("Integer is supported");
    let required_definition = definition(
        vec![
            extension::required(tag(PAYLOAD_OFFSET), integer)
                .expect("required payload rule is valid"),
        ],
        Vec::new(),
        false,
    );
    assert_invalid_schema(extension::validate(
        &required_definition,
        extension_value([]),
        &CodecLimits::defaults(),
    ));

    let optional_definition = definition(
        vec![
            extension::optional(
                tag(PAYLOAD_OFFSET),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("optional payload rule is valid"),
        ],
        Vec::new(),
        false,
    );
    assert!(
        extension::validate(
            &optional_definition,
            extension_value([]),
            &CodecLimits::defaults(),
        )
        .is_ok()
    );
    assert_invalid_schema(extension::validate(
        &optional_definition,
        extension_value([
            item(PAYLOAD_OFFSET, Value::integer(1)),
            item(PAYLOAD_OFFSET, Value::integer(2)),
        ]),
        &CodecLimits::defaults(),
    ));

    let repeated_definition = definition(
        vec![
            extension::repeated(
                tag(PAYLOAD_OFFSET),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("repeated payload rule is valid"),
        ],
        Vec::new(),
        false,
    );
    assert!(
        extension::validate(
            &repeated_definition,
            extension_value([
                item(PAYLOAD_OFFSET, Value::integer(1)),
                item(PAYLOAD_OFFSET, Value::integer(2)),
                item(PAYLOAD_OFFSET, Value::integer(3)),
            ]),
            &CodecLimits::defaults(),
        )
        .is_ok()
    );
}

#[test]
fn order_edges_require_every_before_occurrence_to_precede_every_after_occurrence() {
    let before = 10;
    let after = 11;
    let order = vec![
        extension::extension_order_constraint(tag(before), tag(after))
            .expect("distinct tags form an order edge"),
    ];
    let base_definition = definition(
        vec![
            extension::repeated(
                tag(before),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("the first repeated rule is valid"),
            extension::repeated(
                tag(after),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("the second repeated rule is valid"),
        ],
        order,
        false,
    );

    let ordered = extension_value([
        item(before, Value::integer(1)),
        item(before, Value::integer(2)),
        item(after, Value::integer(3)),
        item(after, Value::integer(4)),
    ]);
    assert!(extension::validate(&base_definition, ordered, &CodecLimits::defaults()).is_ok());

    let interleaved = extension_value([
        item(before, Value::integer(1)),
        item(after, Value::integer(2)),
        item(before, Value::integer(3)),
    ]);
    assert_invalid_schema(extension::validate(
        &base_definition,
        interleaved,
        &CodecLimits::defaults(),
    ));
}

#[test]
fn order_edge_allows_an_absent_optional_endpoint() {
    let before = 12;
    let after = 13;
    let definition = definition(
        vec![
            extension::required(
                tag(before),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("the required before rule is valid"),
            extension::optional(
                tag(after),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("the optional after rule is valid"),
        ],
        vec![
            extension::extension_order_constraint(tag(before), tag(after))
                .expect("distinct tags form an order edge"),
        ],
        false,
    );

    assert!(
        extension::validate(
            &definition,
            extension_value([item(before, Value::integer(1))]),
            &CodecLimits::defaults(),
        )
        .is_ok()
    );
}

#[test]
fn schema_construction_rejects_duplicate_self_and_cyclic_order_edges() {
    let before = 20;
    let after = 21;
    let rule = |offset| {
        extension::required(
            tag(offset),
            extension::scalar(ItemType::Integer).expect("Integer is supported"),
        )
        .expect("the order-rule child is valid")
    };
    let edge = extension::extension_order_constraint(tag(before), tag(after))
        .expect("distinct tags form an order edge");

    let duplicate = extension::structure(vec![rule(before), rule(after)], vec![edge, edge], false);
    assert_eq!(
        duplicate
            .expect_err("duplicate order edges are malformed")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );

    let self_edge = extension::extension_order_constraint(tag(before), tag(before));
    assert_eq!(
        self_edge
            .expect_err("an order edge cannot point to itself")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );

    let cycle = extension::structure(
        vec![rule(before), rule(after)],
        vec![
            edge,
            extension::extension_order_constraint(tag(after), tag(before))
                .expect("the reverse edge is syntactically distinct"),
        ],
        false,
    );
    assert_eq!(
        cycle
            .expect_err("a directed cycle cannot define a valid child order")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );
}

#[test]
fn text_and_byte_string_lengths_include_both_configured_boundaries() {
    for (item_type, minimum, maximum, inside, below, above) in [
        (
            ItemType::TextString,
            2,
            4,
            Value::text_string("abcd".into()),
            Value::text_string("x".into()),
            Value::text_string("abcde".into()),
        ),
        (
            ItemType::ByteString,
            2,
            4,
            Value::byte_string(vec![1, 2]),
            Value::byte_string(vec![1]),
            Value::byte_string(vec![1, 2, 3, 4, 5]),
        ),
    ] {
        let schema = extension::with_maximum_length(
            extension::with_minimum_length(
                extension::scalar(item_type).expect("length scalar type is supported"),
                minimum,
            )
            .expect("minimum length does not exceed maximum"),
            maximum,
        )
        .expect("maximum length is not below minimum");
        assert!(validate_one(schema.clone(), inside).is_ok());

        let schema = extension::with_maximum_length(
            extension::with_minimum_length(
                extension::scalar(item_type).expect("length scalar type is supported"),
                minimum,
            )
            .expect("minimum length does not exceed maximum"),
            maximum,
        )
        .expect("maximum length is not below minimum");
        assert_invalid_schema(validate_one(schema.clone(), below));

        let schema = extension::with_maximum_length(
            extension::with_minimum_length(
                extension::scalar(item_type).expect("length scalar type is supported"),
                minimum,
            )
            .expect("minimum length does not exceed maximum"),
            maximum,
        )
        .expect("maximum length is not below minimum");
        assert_invalid_schema(validate_one(schema, above));
    }
}

fn signed_value(item_type: ItemType, value: i64) -> Value {
    match item_type {
        ItemType::Integer => Value::integer(i32::try_from(value).expect("test integer fits i32")),
        ItemType::LongInteger => Value::long_integer(value),
        ItemType::DateTime => Value::date_time(value),
        ItemType::DateTimeExtended => Value::date_time_extended(value),
        _ => panic!("test fixture must request a supported signed type"),
    }
}

#[test]
fn signed_and_unsigned_numeric_ranges_are_inclusive_and_reject_outliers() {
    for item_type in [
        ItemType::Integer,
        ItemType::LongInteger,
        ItemType::DateTime,
        ItemType::DateTimeExtended,
    ] {
        for value in [-2, 0, 2] {
            let schema = extension::with_signed_range(
                extension::scalar(item_type).expect("signed scalar type is supported"),
                -2,
                2,
            )
            .expect("the signed range is ordered");
            assert!(validate_one(schema, signed_value(item_type, value)).is_ok());
        }
        for value in [-3, 3] {
            let schema = extension::with_signed_range(
                extension::scalar(item_type).expect("signed scalar type is supported"),
                -2,
                2,
            )
            .expect("the signed range is ordered");
            assert_invalid_schema(validate_one(schema, signed_value(item_type, value)));
        }
    }

    for item_type in [ItemType::Enumeration, ItemType::Interval] {
        for value in [3_i32, 5, 7] {
            let schema = extension::with_unsigned_range(
                extension::scalar(item_type).expect("unsigned scalar type is supported"),
                3,
                7,
            )
            .expect("the unsigned range is ordered");
            let value = match item_type {
                ItemType::Enumeration => Value::enumeration(value.cast_unsigned()),
                ItemType::Interval => Value::interval(value.cast_unsigned()),
                _ => unreachable!("the table contains unsigned range types only"),
            };
            assert!(validate_one(schema, value).is_ok());
        }
        let schema = extension::with_unsigned_range(
            extension::scalar(item_type).expect("unsigned scalar type is supported"),
            3,
            7,
        )
        .expect("the unsigned range is ordered");
        let value = match item_type {
            ItemType::Enumeration => Value::enumeration(8),
            ItemType::Interval => Value::interval(8),
            _ => unreachable!("the table contains unsigned range types only"),
        };
        assert_invalid_schema(validate_one(schema, value));
    }
}

#[test]
fn allowed_enumerations_accept_only_declared_values() {
    let schema = extension::with_allowed_enumeration(
        extension::with_allowed_enumeration(
            extension::scalar(ItemType::Enumeration).expect("Enumeration is supported"),
            7,
        )
        .expect("the first enum value is valid"),
        9,
    )
    .expect("the second enum value is valid");

    assert!(validate_one(schema.clone(), Value::enumeration(7)).is_ok());
    assert!(validate_one(schema.clone(), Value::enumeration(9)).is_ok());
    assert_invalid_schema(validate_one(schema, Value::enumeration(8)));
}

#[test]
fn bitmask_rules_require_declared_bits_and_all_required_bits() {
    let schema = extension::with_required_bit_mask(
        extension::with_allowed_bit_mask(
            extension::scalar(ItemType::Integer).expect("Integer is supported"),
            0b0111,
        )
        .expect("allowed mask is declared"),
        0b0001,
    )
    .expect("required mask is declared");

    assert!(validate_one(schema.clone(), Value::integer(0b0001)).is_ok());
    assert!(validate_one(schema.clone(), Value::integer(0b0111)).is_ok());
    assert_invalid_schema(validate_one(schema.clone(), Value::integer(0b1001)));
    assert_invalid_schema(validate_one(schema, Value::integer(0b0010)));
}

#[test]
fn undeclared_children_are_rejected_or_preserved_without_reordering() {
    let rule =
        required_payload(extension::scalar(ItemType::Integer).expect("Integer is supported"));
    let rejected = definition(vec![rule.clone()], Vec::new(), false);
    assert_invalid_schema(extension::validate(
        &rejected,
        extension_value([
            item(PAYLOAD_OFFSET, Value::integer(7)),
            item(30, Value::text_string("unknown".into())),
        ]),
        &CodecLimits::defaults(),
    ));

    let preserved = definition(vec![rule], Vec::new(), true);
    let value = extension_value([
        item(30, Value::text_string("unknown-first".into())),
        item(PAYLOAD_OFFSET, Value::integer(7)),
        item(31, Value::integer(99)),
    ]);
    let validated = extension::validate(&preserved, value, &CodecLimits::defaults())
        .expect("undeclared values are retained under preserve policy");
    let observed: Vec<Tag> = extension::generic_value(&validated)
        .view()
        .children()
        .iter()
        .map(Item::tag)
        .collect();
    assert_eq!(
        observed,
        [
            tag(DISCRIMINATOR_OFFSET),
            tag(30),
            tag(PAYLOAD_OFFSET),
            tag(31)
        ]
    );
    let retained_unknown =
        extension::generic_value(&validated).view().children()[1].with_value(|value| match value {
            kmipkit_ttlv::ValueView::TextString(text) => Some(text.to_owned()),
            _ => None,
        });
    assert_eq!(retained_unknown.as_deref(), Some("unknown-first"));
}

#[test]
fn malformed_schema_builders_reject_incompatible_or_reversed_constraints() {
    assert_eq!(
        extension::scalar(ItemType::Structure)
            .expect_err("Structure requires child rules")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );
    assert_eq!(
        extension::with_minimum_length(
            extension::scalar(ItemType::Integer).expect("Integer is supported"),
            1,
        )
        .expect_err("length constraints require Text String or Byte String")
        .kind(),
        ProtocolErrorKind::InvalidSchema
    );
    assert_eq!(
        extension::with_signed_range(
            extension::scalar(ItemType::Integer).expect("Integer is supported"),
            5,
            4,
        )
        .expect_err("signed ranges cannot be reversed")
        .kind(),
        ProtocolErrorKind::InvalidSchema
    );
    assert_eq!(
        extension::with_unsigned_range(
            extension::scalar(ItemType::Enumeration).expect("Enumeration is supported"),
            5,
            4,
        )
        .expect_err("unsigned ranges cannot be reversed")
        .kind(),
        ProtocolErrorKind::InvalidSchema
    );
    assert_eq!(
        extension::with_allowed_enumeration(
            extension::scalar(ItemType::Integer).expect("Integer is supported"),
            1,
        )
        .expect_err("allowed values require an Enumeration schema")
        .kind(),
        ProtocolErrorKind::InvalidSchema
    );
    assert_eq!(
        extension::with_allowed_bit_mask(
            extension::scalar(ItemType::Enumeration).expect("Enumeration is supported"),
            1,
        )
        .expect_err("bit masks require an Integer schema")
        .kind(),
        ProtocolErrorKind::InvalidSchema
    );
}

#[test]
fn duplicate_child_tags_and_missing_order_endpoints_are_malformed() {
    let child = || {
        extension::required(
            tag(PAYLOAD_OFFSET),
            extension::scalar(ItemType::Integer).expect("Integer is supported"),
        )
        .expect("payload rule is valid")
    };
    assert_eq!(
        extension::structure(vec![child(), child()], Vec::new(), false)
            .expect_err("one Structure cannot declare the same child tag twice")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );

    let edge = extension::extension_order_constraint(tag(PAYLOAD_OFFSET), tag(40))
        .expect("the edge endpoints are distinct");
    assert_eq!(
        extension::structure(vec![child()], vec![edge], false)
            .expect_err("both edge endpoints need declared child rules")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );
}

#[test]
fn configured_ttlv_size_depth_and_element_limits_are_enforced_at_boundaries() {
    let base_definition = definition(
        vec![required_payload(
            extension::scalar(ItemType::Integer).expect("Integer is supported"),
        )],
        Vec::new(),
        false,
    );
    let value = || extension_value([item(PAYLOAD_OFFSET, Value::integer(1))]);

    // Root Structure (8 bytes), discriminator Text String (32 bytes), and
    // Integer payload (16 bytes) produce this exact bounded TTLV size.
    assert!(
        extension::validate(
            &base_definition,
            value(),
            &CodecLimits::new(56, 1, 3).expect("exact limits are valid"),
        )
        .is_ok()
    );
    for limits in [
        CodecLimits::new(55, 1, 3).expect("one byte below exact is valid config"),
        CodecLimits::new(56, 1, 2).expect("one item below exact is valid config"),
    ] {
        let error = extension::validate(&base_definition, value(), &limits)
            .expect_err("the extension exceeds one configured TTLV bound");
        assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
    }

    let nested_schema = extension::structure(
        vec![
            extension::required(
                tag(NESTED_OFFSET),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("nested child rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("nested Structure schema is valid");
    let nested_definition = definition(vec![required_payload(nested_schema)], Vec::new(), false);
    let nested_value = extension_value([item(
        PAYLOAD_OFFSET,
        Value::structure(structure([item(NESTED_OFFSET, Value::integer(1))])),
    )]);
    assert!(
        extension::validate(
            &nested_definition,
            nested_value,
            &CodecLimits::new(1_024, 2, 4).expect("exact nesting/item limits are valid"),
        )
        .is_ok()
    );
    let too_shallow = extension_value([item(
        PAYLOAD_OFFSET,
        Value::structure(structure([item(NESTED_OFFSET, Value::integer(1))])),
    )]);
    assert_eq!(
        extension::validate(
            &nested_definition,
            too_shallow,
            &CodecLimits::new(1_024, 1, 4).expect("a lower depth is a valid config"),
        )
        .expect_err("nested TTLV exceeds the configured Structure depth")
        .kind(),
        ProtocolErrorKind::ResourceLimit
    );
}

fn nested_schema(structures: usize) -> Result<extension::ExtensionSchema, ProtocolError> {
    let mut schema = extension::scalar(ItemType::TextString)?;
    for offset in 0..structures {
        let offset = u32::try_from(offset + 100).expect("test depth fits a u32 tag");
        schema = extension::structure(
            vec![extension::required(tag(offset), schema)?],
            Vec::new(),
            false,
        )?;
    }
    Ok(schema)
}

#[test]
fn schema_nesting_and_discriminator_path_accept_the_limit_and_reject_one_more() {
    assert!(nested_schema(63).is_ok());
    assert_eq!(
        nested_schema(64)
            .expect_err("schema nesting cannot exceed the hard depth")
            .kind(),
        ProtocolErrorKind::ResourceLimit
    );

    let mut path = extension::ttlv_path(tag(100)).expect("the first discriminator tag is valid");
    for offset in 101..164 {
        path = extension::with_child_tag(path, tag(offset))
            .expect("a discriminator path can reach the 64-tag boundary");
    }
    assert_eq!(path.tags().len(), 64);
    assert_eq!(
        extension::with_child_tag(path, tag(164))
            .expect_err("the discriminator path cannot exceed 64 tags")
            .kind(),
        ProtocolErrorKind::ResourceLimit
    );
}

#[test]
fn schema_structure_child_width_accepts_hard_maximum_and_rejects_one_more() {
    let rule_count = 4_096_u32;
    let rules = |count: u32| {
        (0..count)
            .map(|offset| {
                extension::required(
                    tag(1_000 + offset),
                    extension::scalar(ItemType::Integer).expect("Integer is supported"),
                )
                .expect("each wide-schema child rule is valid")
            })
            .collect::<Vec<_>>()
    };
    assert!(extension::structure(rules(rule_count), Vec::new(), true).is_ok());
    assert_eq!(
        extension::structure(rules(rule_count + 1), Vec::new(), true)
            .expect_err("one Structure cannot exceed the hard child width")
            .kind(),
        ProtocolErrorKind::ResourceLimit
    );
}

#[test]
fn per_rule_enumeration_and_order_constraint_counts_stop_at_hard_maximum() {
    let mut enumeration =
        extension::scalar(ItemType::Enumeration).expect("Enumeration is supported");
    for value in 0..4_096_u32 {
        enumeration = extension::with_allowed_enumeration(enumeration, value)
            .expect("an enum rule can reach the hard member limit");
    }
    assert_eq!(
        extension::with_allowed_enumeration(enumeration, 4_096)
            .expect_err("an enum rule cannot exceed the hard member limit")
            .kind(),
        ProtocolErrorKind::ResourceLimit
    );

    let child_count = 4_096_u32;
    let children = (0..child_count)
        .map(|offset| {
            extension::required(
                tag(1_000 + offset),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("order child rule is valid")
        })
        .collect::<Vec<_>>();
    let mut order = (0..(child_count - 1))
        .map(|offset| {
            extension::extension_order_constraint(tag(1_000 + offset), tag(1_001 + offset))
                .expect("adjacent distinct tags form a valid order edge")
        })
        .collect::<Vec<_>>();
    order.push(
        extension::extension_order_constraint(tag(1_000), tag(1_002))
            .expect("a forward skip edge remains acyclic"),
    );
    assert!(extension::structure(children.clone(), order.clone(), true).is_ok());
    order.push(
        extension::extension_order_constraint(tag(1_000), tag(1_003))
            .expect("another forward edge remains acyclic"),
    );
    assert_eq!(
        extension::structure(children, order, true)
            .expect_err("one Structure cannot exceed the hard order-edge member limit")
            .kind(),
        ProtocolErrorKind::ResourceLimit
    );
}

#[test]
fn definition_accounting_counts_reused_nested_schema_nodes_and_constraint_members() {
    let enum_schema = extension::with_allowed_enumeration(
        extension::scalar(ItemType::Enumeration).expect("Enumeration is supported"),
        7,
    )
    .expect("the enum member is valid");
    let schema = extension::structure(
        vec![
            discriminator_rule(),
            extension::required(tag(50), enum_schema.clone())
                .expect("first reused schema child is valid"),
            extension::required(tag(51), enum_schema).expect("second reused schema child is valid"),
        ],
        vec![
            extension::extension_order_constraint(tag(50), tag(51))
                .expect("the order edge is valid"),
        ],
        false,
    )
    .expect("the aggregate schema is valid");
    let identity = extension::extension_identity("example.vendor", "accounting", "1")
        .expect("the accounting identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the accounting definition supports the workspace version");
    let path = extension::ttlv_path(tag(DISCRIMINATOR_OFFSET))
        .expect("the discriminator path is non-empty");
    let discriminator = extension::discriminator(path, Value::text_string(DISCRIMINATOR.into()))
        .expect("the accounting discriminator is valid");
    let definition =
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("the accounting definition is valid");

    let totals = extension::accounting(&definition).expect("schema accounting fits usize");
    assert_eq!(totals.schema_nodes, 4);
    assert_eq!(totals.constraint_members, 3);
}

#[test]
fn repeated_enum_and_ordered_fields_keep_semantics_for_large_input_trees() {
    let enum_schema = extension::with_allowed_enumeration(
        extension::scalar(ItemType::Enumeration).expect("Enumeration is supported"),
        7,
    )
    .expect("the enum value is valid");
    let order = vec![
        extension::extension_order_constraint(tag(60), tag(61)).expect("the order edge is valid"),
    ];
    let definition = definition(
        vec![
            extension::repeated(tag(60), enum_schema).expect("repeated enum rule is valid"),
            extension::repeated(
                tag(61),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("repeated Integer rule is valid"),
        ],
        order,
        false,
    );
    let mut children = (0..4_096)
        .map(|_| item(60, Value::enumeration(7)))
        .collect::<Vec<_>>();
    children.extend((0..4_096).map(|index| item(61, Value::integer(index))));
    assert!(
        extension::validate(
            &definition,
            extension_value(children),
            &CodecLimits::defaults(),
        )
        .is_ok()
    );
}
