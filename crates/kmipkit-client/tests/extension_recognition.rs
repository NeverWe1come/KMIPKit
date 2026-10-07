//! Registered extension recognition and inspection behavior.
//!
//! Traceability: KMIPKIT-0012-FR-006, KMIPKIT-0012-FR-007,
//! KMIPKIT-0012-FR-012, and KMIP 2.1 §9.13, Table 418.

use kmipkit_client::extension_registry::{self, ClientExtensionRegistry, ExtensionRecognition};
use kmipkit_client::{ClientError, ClientErrorCategory};
use kmipkit_protocol::extension;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value};

const VENDOR: &str = "example.vendor";
const FIRST_PATH_TAG: u32 = 0x42_0001;
const SECOND_PATH_TAG: u32 = 0x42_0002;
const UNKNOWN_TAG: u32 = 0x42_0003;
const REQUIRED_EXTRA_TAG: u32 = 0x42_0004;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the test tag fits in the KMIP Tag width")
        .try_checked()
        .expect("the test tag uses the vendor allocation")
}

fn definition(
    name: &str,
    discriminator_path: &[u32],
    discriminator_value: &str,
    require_extra: bool,
) -> extension::ExtensionDefinition {
    let identity = extension::extension_identity(VENDOR, name, "1")
        .expect("the test extension identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the test extension supports this client");
    let mut path = extension::ttlv_path(tag(discriminator_path[0]))
        .expect("the discriminator path is non-empty");
    for raw_tag in &discriminator_path[1..] {
        path = extension::with_child_tag(path, tag(*raw_tag))
            .expect("the discriminator path is valid");
    }
    let discriminator =
        extension::discriminator(path, Value::text_string(discriminator_value.to_owned()))
            .expect("the discriminator is a valid scalar");

    let mut schema = extension::scalar(ItemType::TextString)
        .expect("Text String is supported by the extension schema");
    for raw_tag in discriminator_path.iter().rev() {
        let mut children = vec![
            extension::required(tag(*raw_tag), schema)
                .expect("the discriminator schema child is valid"),
        ];
        if require_extra && *raw_tag == discriminator_path[0] {
            children.push(
                extension::required(
                    tag(REQUIRED_EXTRA_TAG),
                    extension::scalar(ItemType::ByteString)
                        .expect("Byte String is supported by the extension schema"),
                )
                .expect("the required extra schema child is valid"),
            );
        }
        schema = extension::structure(children, Vec::new(), true)
            .expect("the extension Structure schema is valid");
    }
    let definition =
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("the test extension definition is valid");
    let information =
        extension::extension_information(name).expect("the extension information name is valid");
    extension::with_information(definition, information).expect("the extension metadata is valid")
}

fn registry(definitions: Vec<extension::ExtensionDefinition>) -> ClientExtensionRegistry {
    extension_registry::client_extension_registry(definitions, extension::defaults())
        .expect("the test definitions form a valid immutable registry")
}

fn registry_limits(
    max_index_records: u64,
    max_lookup_comparisons: u64,
) -> extension::ExtensionRegistryLimits {
    let defaults = extension::defaults();
    extension::with_values(
        defaults.max_definitions(),
        defaults.max_schema_nodes(),
        defaults.max_child_rules_per_structure(),
        defaults.max_text_bytes_per_field(),
        defaults.max_registry_text_bytes(),
        defaults.max_discriminator_scalar_bytes(),
        defaults.max_total_discriminator_scalar_bytes(),
        defaults.max_constraint_members_per_rule(),
        defaults.max_total_constraint_members(),
        max_index_records,
        max_lookup_comparisons,
        defaults.max_depth(),
    )
    .expect("the requested registry work budgets are below their hard maxima")
}

fn registry_with_max_depth(
    definitions: Vec<extension::ExtensionDefinition>,
    max_depth: u64,
) -> ClientExtensionRegistry {
    let defaults = extension::defaults();
    let limits = extension::with_values(
        defaults.max_definitions(),
        defaults.max_schema_nodes(),
        defaults.max_child_rules_per_structure(),
        defaults.max_text_bytes_per_field(),
        defaults.max_registry_text_bytes(),
        defaults.max_discriminator_scalar_bytes(),
        defaults.max_total_discriminator_scalar_bytes(),
        defaults.max_constraint_members_per_rule(),
        defaults.max_total_constraint_members(),
        defaults.max_payload_index_records(),
        defaults.max_lookup_comparisons(),
        max_depth,
    )
    .expect("the requested registry depth is within its hard maximum");
    extension_registry::client_extension_registry(definitions, limits)
        .expect("schema and discriminator depth fit the registry limit")
}

fn payload(path: &[u32], value: Value) -> Structure {
    let mut nested = Structure::new();
    nested
        .try_push(
            Item::new(tag(*path.last().expect("path is non-empty")), value)
                .expect("leaf item is valid"),
        )
        .expect("leaf fits the payload Structure");

    for raw_tag in path[..path.len() - 1].iter().rev() {
        let mut parent = Structure::new();
        parent
            .try_push(
                Item::new(tag(*raw_tag), Value::structure(nested))
                    .expect("nested Structure item is valid"),
            )
            .expect("nested child fits the payload Structure");
        nested = parent;
    }
    nested
}

fn inspect(
    registry: &ClientExtensionRegistry,
    value: Structure,
    limits: &CodecLimits,
) -> ExtensionRecognition {
    extension_registry::inspect(registry, VENDOR, value, limits)
        .expect("the generic payload is within configured resource limits")
}

fn assert_unrecognized(recognition: &ExtensionRecognition) {
    assert!(!extension_registry::is_recognized(recognition));
    assert!(extension_registry::validated_value(recognition).is_none());
}

#[test]
fn exact_discriminator_hit_is_fully_schema_validated_and_keeps_generic_payload() {
    let registry = registry(vec![definition(
        "alpha",
        &[FIRST_PATH_TAG],
        "alpha-v1",
        false,
    )]);
    let value = payload(&[FIRST_PATH_TAG], Value::text_string("alpha-v1".to_owned()));

    let recognition = inspect(&registry, value, &CodecLimits::defaults());

    assert!(extension_registry::is_recognized(&recognition));
    let validated = extension_registry::validated_value(&recognition)
        .expect("one exact match that passes the complete schema yields a typed value");
    assert_eq!(
        extension::validated_extension_value_identity(validated).name(),
        "alpha"
    );
    assert_eq!(
        extension_registry::generic_value(&recognition)
            .view()
            .children()[0]
            .tag(),
        tag(FIRST_PATH_TAG),
        "inspection preserves the original generic subtree"
    );
}

#[test]
fn missing_repeated_and_wrong_type_discriminator_paths_are_unrecognized() {
    let registry = registry(vec![definition(
        "alpha",
        &[FIRST_PATH_TAG],
        "alpha-v1",
        false,
    )]);
    let missing = Structure::new();
    let mut repeated = payload(&[FIRST_PATH_TAG], Value::text_string("alpha-v1".to_owned()));
    repeated
        .try_push(
            Item::new(
                tag(FIRST_PATH_TAG),
                Value::text_string("alpha-v1".to_owned()),
            )
            .expect("the repeated discriminator item is valid"),
        )
        .expect("the repeated discriminator fits the payload");
    let wrong_type = payload(&[FIRST_PATH_TAG], Value::integer(7));

    for value in [missing, repeated, wrong_type] {
        assert_unrecognized(&inspect(&registry, value, &CodecLimits::defaults()));
    }
}

#[test]
fn registry_depth_limit_does_not_restrict_preserved_unknown_payload_subtrees() {
    let registry = registry_with_max_depth(
        vec![definition("alpha", &[FIRST_PATH_TAG], "alpha-v1", false)],
        2,
    );
    let mut deepest = Structure::new();
    deepest
        .try_push(
            Item::new(tag(UNKNOWN_TAG), Value::text_string("preserved".to_owned()))
                .expect("the deepest preserved child is valid"),
        )
        .expect("the deepest preserved child fits");
    let mut middle = Structure::new();
    middle
        .try_push(
            Item::new(tag(UNKNOWN_TAG), Value::structure(deepest))
                .expect("the middle preserved Structure is valid"),
        )
        .expect("the middle preserved Structure fits");
    let mut unknown = Structure::new();
    unknown
        .try_push(
            Item::new(tag(UNKNOWN_TAG), Value::structure(middle))
                .expect("the preserved Structure is valid"),
        )
        .expect("the preserved Structure fits");
    let mut value = payload(&[FIRST_PATH_TAG], Value::text_string("alpha-v1".to_owned()));
    value
        .try_push(
            Item::new(tag(UNKNOWN_TAG), Value::structure(unknown))
                .expect("the unknown nested value is valid"),
        )
        .expect("the unknown nested value fits beside the discriminator");
    let codec_limits = CodecLimits::new(
        CodecLimits::DEFAULT_MAX_MESSAGE_BYTES,
        4,
        CodecLimits::DEFAULT_MAX_ELEMENTS,
    )
    .expect("the payload nesting depth fits the configured TTLV limit");

    let recognition = extension_registry::inspect(&registry, VENDOR, value, &codec_limits)
        .expect("registry depth applies to its schemas and paths, not generic payload depth");

    assert!(extension_registry::is_recognized(&recognition));
    assert_eq!(
        extension_registry::generic_value(&recognition)
            .view()
            .children()
            .len(),
        2,
        "the deeper undeclared subtree remains available unchanged"
    );
}

#[test]
fn absent_repeated_and_wrong_type_nested_path_steps_are_non_matches() {
    let path = [FIRST_PATH_TAG, SECOND_PATH_TAG];
    let registry = registry(vec![definition("nested", &path, "nested-v1", false)]);

    let missing = payload(&[FIRST_PATH_TAG], Value::structure(Structure::new()));
    let mut repeated_leaf = Structure::new();
    for _ in 0..2 {
        repeated_leaf
            .try_push(
                Item::new(
                    tag(SECOND_PATH_TAG),
                    Value::text_string("nested-v1".to_owned()),
                )
                .expect("the repeated nested path item is valid"),
            )
            .expect("the repeated child fits the nested Structure");
    }
    let repeated = payload(&[FIRST_PATH_TAG], Value::structure(repeated_leaf));
    let wrong_type = payload(&[FIRST_PATH_TAG], Value::integer(3));

    for value in [missing, repeated, wrong_type] {
        assert_unrecognized(&inspect(&registry, value, &CodecLimits::defaults()));
    }
}

#[test]
fn schema_failure_after_unique_discriminator_hit_has_no_partial_typed_value() {
    let registry = registry(vec![definition(
        "alpha",
        &[FIRST_PATH_TAG],
        "alpha-v1",
        true,
    )]);
    let value = payload(&[FIRST_PATH_TAG], Value::text_string("alpha-v1".to_owned()));

    let recognition = inspect(&registry, value, &CodecLimits::defaults());

    assert_unrecognized(&recognition);
    assert_eq!(
        extension_registry::generic_value(&recognition)
            .view()
            .children()
            .len(),
        1,
        "a failed typed projection still retains the unmodified generic value"
    );
}

#[test]
fn ambiguous_exact_matches_are_unrecognized_independent_of_registration_order() {
    let forward = inspect(
        &registry(vec![
            definition("alpha", &[FIRST_PATH_TAG], "alpha-v1", false),
            definition("beta", &[SECOND_PATH_TAG], "beta-v1", false),
        ]),
        ambiguous_payload(),
        &CodecLimits::defaults(),
    );
    let reverse = inspect(
        &registry(vec![
            definition("beta", &[SECOND_PATH_TAG], "beta-v1", false),
            definition("alpha", &[FIRST_PATH_TAG], "alpha-v1", false),
        ]),
        ambiguous_payload(),
        &CodecLimits::defaults(),
    );

    assert_unrecognized(&forward);
    assert_unrecognized(&reverse);
}

fn ambiguous_payload() -> Structure {
    let mut value = payload(&[FIRST_PATH_TAG], Value::text_string("alpha-v1".to_owned()));
    value
        .try_push(
            Item::new(
                tag(SECOND_PATH_TAG),
                Value::text_string("beta-v1".to_owned()),
            )
            .expect("the second matching discriminator is valid"),
        )
        .expect("the second discriminator fits the payload");
    value
}

#[test]
fn duplicate_exact_discriminator_keys_are_rejected_during_registry_construction() {
    let result = extension_registry::client_extension_registry(
        vec![
            definition("alpha", &[FIRST_PATH_TAG], "same-v1", false),
            definition("beta", &[FIRST_PATH_TAG], "same-v1", false),
        ],
        extension::defaults(),
    );

    assert!(
        result.is_err(),
        "one vendor cannot register duplicate exact keys"
    );
}

#[test]
fn shared_payload_index_and_comparison_limits_have_exact_success_boundaries() {
    let exact_definition = definition("alpha", &[FIRST_PATH_TAG], "alpha-v1", false);
    let exact_value = payload(&[FIRST_PATH_TAG], Value::text_string("alpha-v1".to_owned()));
    let exact_registry = extension_registry::client_extension_registry(
        vec![exact_definition],
        registry_limits(2, 2),
    )
    .expect("the exact two-record index and two tag-comparison budget are valid");
    let recognized = extension_registry::inspect(
        &exact_registry,
        VENDOR,
        exact_value,
        &CodecLimits::defaults(),
    )
    .expect("one Structure plus one child fits the exact index budget");
    assert!(extension_registry::is_recognized(&recognized));

    let under_index_registry = extension_registry::client_extension_registry(
        vec![definition("alpha", &[FIRST_PATH_TAG], "alpha-v1", false)],
        registry_limits(1, 2),
    )
    .expect("the lowered index limit remains a valid registry configuration");
    let index_error = extension_registry::inspect(
        &under_index_registry,
        VENDOR,
        payload(&[FIRST_PATH_TAG], Value::text_string("alpha-v1".to_owned())),
        &CodecLimits::defaults(),
    )
    .expect_err("the second required payload-index record exceeds the exact boundary");
    assert_eq!(index_error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        index_error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
    assert!(matches!(
        index_error,
        ClientError::Protocol { ref error, .. }
            if error.kind() == kmipkit_protocol::ProtocolErrorKind::ResourceLimit
    ));

    let under_comparison_registry = extension_registry::client_extension_registry(
        vec![definition("alpha", &[FIRST_PATH_TAG], "alpha-v1", false)],
        registry_limits(2, 1),
    )
    .expect("the lowered comparison limit remains a valid registry configuration");
    let comparison_error = extension_registry::inspect(
        &under_comparison_registry,
        VENDOR,
        payload(&[FIRST_PATH_TAG], Value::text_string("alpha-v1".to_owned())),
        &CodecLimits::defaults(),
    )
    .expect_err("the second binary-search tag comparison exceeds the exact boundary");
    assert_eq!(comparison_error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        comparison_error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
    assert!(matches!(
        comparison_error,
        ClientError::Protocol { ref error, .. }
            if error.kind() == kmipkit_protocol::ProtocolErrorKind::ResourceLimit
    ));
}

#[test]
fn wide_payload_with_last_child_match_stays_within_tag_comparison_budget() {
    let defaults = extension::defaults();
    let limits = extension::with_values(
        defaults.max_definitions(),
        defaults.max_schema_nodes(),
        defaults.max_child_rules_per_structure(),
        defaults.max_text_bytes_per_field(),
        defaults.max_registry_text_bytes(),
        defaults.max_discriminator_scalar_bytes(),
        defaults.max_total_discriminator_scalar_bytes(),
        defaults.max_constraint_members_per_rule(),
        defaults.max_total_constraint_members(),
        defaults.max_payload_index_records(),
        35,
        defaults.max_depth(),
    )
    .expect("35 comparisons stays under the hard lookup maximum");
    let registry = extension_registry::client_extension_registry(
        vec![definition("alpha", &[FIRST_PATH_TAG], "alpha-v1", false)],
        limits,
    )
    .expect("the single-definition registry is valid");
    let mut value = Structure::new();
    for _ in 0..99_998 {
        value
            .try_push(
                Item::new(tag(UNKNOWN_TAG), Value::text_string("x".to_owned()))
                    .expect("an unknown generic field is a valid Item"),
            )
            .expect("the wide payload remains a valid Structure");
    }
    value
        .try_push(
            Item::new(
                tag(FIRST_PATH_TAG),
                Value::text_string("alpha-v1".to_owned()),
            )
            .expect("the last discriminator item is valid"),
        )
        .expect("the last discriminator fits the wide payload");
    let codec_limits = CodecLimits::new(
        CodecLimits::DEFAULT_MAX_MESSAGE_BYTES,
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        CodecLimits::DEFAULT_MAX_ELEMENTS,
    )
    .expect("the model depth remains within its hard maximum");

    let recognition = extension_registry::inspect(&registry, VENDOR, value, &codec_limits)
        .expect("the 100,000-item lookup, including its root, stays within budget");

    assert!(extension_registry::is_recognized(&recognition));
}

#[test]
fn repeated_tag_search_stays_within_the_thirty_five_comparison_step_bound() {
    let make_registry = |maximum_comparisons| {
        extension_registry::client_extension_registry(
            vec![definition("alpha", &[FIRST_PATH_TAG], "alpha-v1", false)],
            registry_limits(200_000, maximum_comparisons),
        )
        .expect("the configured comparison budget is within its hard maximum")
    };
    let mut value = Structure::new();
    for _ in 0..99_999 {
        value
            .try_push(
                Item::new(
                    tag(FIRST_PATH_TAG),
                    Value::text_string("alpha-v1".to_owned()),
                )
                .expect("the repeated discriminator item is valid"),
            )
            .expect("the wide payload remains within the model depth bound");
    }
    let codec_limits = CodecLimits::new(
        CodecLimits::DEFAULT_MAX_MESSAGE_BYTES,
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        CodecLimits::DEFAULT_MAX_ELEMENTS,
    )
    .expect("the raised element limit remains within TTLV bounds");

    let exact = extension_registry::inspect(&make_registry(34), VENDOR, value, &codec_limits)
        .expect("the observed worst case uses 34 comparisons, below the hard bound of 35");
    assert_unrecognized(&exact);

    let mut value = Structure::new();
    for _ in 0..99_999 {
        value
            .try_push(
                Item::new(
                    tag(FIRST_PATH_TAG),
                    Value::text_string("alpha-v1".to_owned()),
                )
                .expect("the repeated discriminator item is valid"),
            )
            .expect("the wide payload remains within the model depth bound");
    }
    let below_bound = extension_registry::inspect(&make_registry(33), VENDOR, value, &codec_limits)
        .expect_err("the thirty-fourth tag comparison exceeds the lowered budget");
    assert!(matches!(
        below_bound,
        ClientError::Protocol { ref error, .. }
            if error.kind() == kmipkit_protocol::ProtocolErrorKind::ResourceLimit
    ));
}

#[test]
fn raised_codec_limit_cannot_raise_the_hard_payload_item_cap() {
    let registry = registry(vec![definition(
        "alpha",
        &[FIRST_PATH_TAG],
        "alpha-v1",
        false,
    )]);
    let mut value = Structure::new();
    for _ in 0..100_000 {
        value
            .try_push(
                Item::new(tag(UNKNOWN_TAG), Value::text_string("x".to_owned()))
                    .expect("the generic payload item is valid"),
            )
            .expect("the payload remains within the TTLV model depth bound");
    }
    let codec_limits = CodecLimits::new(
        CodecLimits::DEFAULT_MAX_MESSAGE_BYTES,
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        100_001,
    )
    .expect("the test explicitly raises the configurable element limit");

    let error = extension_registry::inspect(&registry, VENDOR, value, &codec_limits)
        .expect_err("the fixed recognition cap counts the root and rejects total item 100,001");
    assert!(matches!(
        error,
        ClientError::Protocol { ref error, .. }
            if error.kind() == kmipkit_protocol::ProtocolErrorKind::ResourceLimit
    ));
}
