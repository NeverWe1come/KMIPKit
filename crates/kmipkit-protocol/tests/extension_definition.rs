//! Fallible identity, compatibility, discriminator, and definition contracts.
//!
//! Traceability: KMIPKIT-0012-FR-001, FR-002, FR-003, FR-006, and FR-012.

use kmipkit_protocol::{ProtocolError, ProtocolErrorKind, extension};
use kmipkit_ttlv::{ItemType, RawTag, Tag, Value};

const TAG_BASE: u32 = 0x0054_0000;

fn tag(offset: u32) -> Tag {
    RawTag::new(TAG_BASE + offset)
        .expect("definition test tag fits the KMIP tag width")
        .try_checked()
        .expect("definition test tag uses the extension allocation")
}

fn assert_kind<T: std::fmt::Debug>(result: Result<T, ProtocolError>, expected: ProtocolErrorKind) {
    let error = result.expect_err("the invalid definition input must be rejected");
    assert_eq!(error.kind(), expected);
}

fn valid_discriminator(value: Value) -> extension::Discriminator {
    extension::discriminator(
        extension::ttlv_path(tag(1)).expect("the discriminator path is non-empty"),
        value,
    )
    .expect("the discriminator scalar is valid")
}

#[test]
fn vendor_identification_accepts_only_nonempty_ascii_identifier_characters() {
    for accepted in ["a", "Vendor_1", "vendor.name", "A9._"] {
        assert!(
            extension::extension_identity(accepted, "extension", "1").is_ok(),
            "expected vendor identifier {accepted:?} to be accepted"
        );
    }

    for rejected in ["", "vendor-name", "vendor/name", "vendor name", "véndor"] {
        assert_kind(
            extension::extension_identity(rejected, "extension", "1"),
            ProtocolErrorKind::InvalidIdentity,
        );
    }
}

#[test]
fn identity_fields_are_nonempty_and_limited_by_utf8_byte_count() {
    for (vendor, name, version) in [
        ("", "extension", "1"),
        ("vendor", "", "1"),
        ("vendor", "extension", ""),
    ] {
        assert_kind(
            extension::extension_identity(vendor, name, version),
            ProtocolErrorKind::InvalidIdentity,
        );
    }

    let vendor_at_limit = "v".repeat(4_096);
    let name_at_limit = "é".repeat(2_048);
    let version_at_limit = "é".repeat(2_048);
    assert!(
        extension::extension_identity(&vendor_at_limit, &name_at_limit, &version_at_limit).is_ok()
    );

    let name_over_limit = format!("{name_at_limit}x");
    assert_eq!(name_over_limit.len(), 4_097);
    assert_kind(
        extension::extension_identity("vendor", &name_over_limit, "1"),
        ProtocolErrorKind::ResourceLimit,
    );
    let vendor_over_limit = "v".repeat(4_097);
    assert_kind(
        extension::extension_identity(&vendor_over_limit, "name", "1"),
        ProtocolErrorKind::ResourceLimit,
    );
    let version_over_limit = "1".repeat(4_097);
    assert_kind(
        extension::extension_identity("vendor", "name", &version_over_limit),
        ProtocolErrorKind::ResourceLimit,
    );
}

#[test]
fn compatibility_includes_kmip_2_1_and_the_running_kmipkit_version() {
    let current = env!("CARGO_PKG_VERSION");
    assert!(extension::compatibility(2, 1, 2, 1, current, current).is_ok());
    assert!(extension::compatibility(1, 0, 2, 1, "0.0.0", "99.0.0").is_ok());
    assert!(extension::compatibility(2, 1, 9, 9, "0.0.0", "99.0.0").is_ok());

    for args in [
        (2, 2, 3, 0, "0.0.0", "99.0.0"),
        (1, 0, 2, 0, "0.0.0", "99.0.0"),
        (3, 0, 2, 1, "0.0.0", "99.0.0"),
        (2, 1, 2, 1, "not-semver", "99.0.0"),
        (2, 1, 2, 1, "0.0.0", "1.2"),
        (2, 1, 2, 1, "0.0.0.1", "99.0.0"),
        (2, 1, 2, 1, "1.0.0", "0.9.0"),
        (2, 1, 2, 1, "99.0.0", "100.0.0"),
        (2, 1, 2, 1, "0.0.0", "0.0.1"),
    ] {
        assert_kind(
            extension::compatibility(args.0, args.1, args.2, args.3, args.4, args.5),
            ProtocolErrorKind::CompatibilityMismatch,
        );
    }
}

#[test]
fn compatibility_rejects_noncanonical_and_unrepresentable_semver_components() {
    for minimum in [
        "01.0.0",
        "1.00.0",
        "1.0.00",
        "1..0",
        "1.0.18446744073709551616",
        "1.0.0-rc.1",
    ] {
        assert_kind(
            extension::compatibility(2, 1, 2, 1, minimum, "99.0.0"),
            ProtocolErrorKind::CompatibilityMismatch,
        );
    }
}

#[test]
fn discriminator_scalar_size_and_path_depth_obey_their_hard_limits() {
    assert_eq!(
        valid_discriminator(Value::text_string("x".repeat(4_096)))
            .path()
            .tags()
            .len(),
        1
    );
    assert_kind(
        extension::discriminator(
            extension::ttlv_path(tag(1)).expect("the discriminator path is non-empty"),
            Value::text_string("x".repeat(4_097)),
        ),
        ProtocolErrorKind::ResourceLimit,
    );
    assert!(
        extension::discriminator(
            extension::ttlv_path(tag(1)).expect("the byte discriminator path is non-empty"),
            Value::byte_string(vec![0xA5; 4_096]),
        )
        .is_ok()
    );
    assert_kind(
        extension::discriminator(
            extension::ttlv_path(tag(1)).expect("the byte discriminator path is non-empty"),
            Value::byte_string(vec![0xA5; 4_097]),
        ),
        ProtocolErrorKind::ResourceLimit,
    );
    assert_kind(
        extension::discriminator(
            extension::ttlv_path(tag(1)).expect("the discriminator path is non-empty"),
            Value::structure(kmipkit_ttlv::Structure::new()),
        ),
        ProtocolErrorKind::InvalidSchema,
    );

    let mut path = extension::ttlv_path(tag(1)).expect("the path starts with one tag");
    for index in 1..64 {
        path = extension::with_child_tag(path, tag(index)).expect("depth 64 is allowed");
    }
    assert_eq!(path.tags().len(), 64);
    assert_kind(
        extension::with_child_tag(path, tag(64)),
        ProtocolErrorKind::ResourceLimit,
    );
}

#[test]
fn cloned_discriminator_preserves_each_supported_scalar_ttlv_value() {
    let values = [
        Value::integer(-7),
        Value::long_integer(-9),
        Value::big_integer(vec![0x80, 0x01]),
        Value::enumeration(17),
        Value::boolean(true),
        Value::text_string("extension scalar".to_owned()),
        Value::byte_string(vec![0x00, 0xA5]),
        Value::date_time(1_700_000_000),
        Value::interval(2_500),
        Value::date_time_extended(-1_700_000_000),
    ];

    for value in values {
        let original = valid_discriminator(value);
        let cloned = extension::clone_extension_discriminator(&original)
            .expect("a supported scalar discriminator can be copied");

        assert_eq!(cloned.path().tags(), original.path().tags());
        assert!(cloned.has_same_scalar(&original));
    }
}

#[test]
fn scalar_fingerprints_are_type_separated_and_structures_have_no_fingerprint() {
    let values = [
        Value::integer(-7),
        Value::long_integer(-9),
        Value::big_integer(vec![0x80, 0x01]),
        Value::enumeration(17),
        Value::boolean(true),
        Value::text_string("extension scalar".to_owned()),
        Value::byte_string(vec![0x00, 0xA5]),
        Value::date_time(1_700_000_000),
        Value::interval(2_500),
        Value::date_time_extended(-1_700_000_000),
    ];

    let fingerprints = values
        .into_iter()
        .map(|value| {
            let discriminator = valid_discriminator(value);
            let debug = format!("{discriminator:?}");
            assert!(debug.contains("Discriminator"));
            assert!(debug.contains("scalar_value"));
            discriminator
                .scalar_fingerprint()
                .expect("supported scalar types have fingerprints")
        })
        .collect::<Vec<_>>();

    assert_ne!(fingerprints[0], fingerprints[1]);
    assert_eq!(
        fingerprints[5],
        extension::discriminator(
            extension::ttlv_path(tag(1)).expect("the path is non-empty"),
            Value::text_string("extension scalar".to_owned()),
        )
        .expect("the scalar discriminator is valid")
        .scalar_fingerprint()
        .expect("TextString has a scalar fingerprint")
    );

    let structure_item =
        kmipkit_ttlv::Item::new(tag(2), Value::structure(kmipkit_ttlv::Structure::new()))
            .expect("the test Structure uses a valid extension tag");
    assert_eq!(
        structure_item.with_value(|view| extension::scalar_value_fingerprint(&view)),
        None,
        "a Structure is not a discriminator scalar",
    );
}

#[test]
fn nested_discriminator_paths_match_only_the_declared_terminal_schema() {
    let identity =
        extension::extension_identity("example.vendor", "nested", "1").expect("identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("compatibility includes this KMIPKit version");
    let inner_schema = extension::structure(
        vec![
            extension::required(
                tag(2),
                extension::scalar(ItemType::TextString).expect("TextString is supported"),
            )
            .expect("the terminal child rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("the nested Structure schema is valid");
    let schema = extension::structure(
        vec![extension::required(tag(1), inner_schema).expect("the nested rule is valid")],
        Vec::new(),
        false,
    )
    .expect("the root Structure schema is valid");
    let path = extension::with_child_tag(
        extension::ttlv_path(tag(1)).expect("the path starts with one tag"),
        tag(2),
    )
    .expect("the nested path is within the depth limit");
    let discriminator = extension::discriminator(path, Value::text_string("value".to_owned()))
        .expect("the TextString discriminator is valid");

    assert!(
        extension::extension_definition(identity.clone(), compatibility, discriminator, schema)
            .is_ok()
    );

    let missing_path = extension::ttlv_path(tag(3)).expect("the path is non-empty");
    let missing_discriminator = valid_discriminator(Value::text_string("value".to_owned()));
    assert_eq!(missing_discriminator.path().tags(), &[tag(1)]);
    assert_eq!(missing_path.tags(), &[tag(3)]);
    let scalar_schema = extension::structure(
        vec![
            extension::required(
                tag(1),
                extension::scalar(ItemType::TextString).expect("TextString is supported"),
            )
            .expect("the child rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("the root schema is valid");
    assert_kind(
        extension::extension_definition(
            identity,
            compatibility,
            extension::discriminator(missing_path, Value::text_string("value".to_owned()))
                .expect("the discriminator scalar is valid"),
            scalar_schema,
        ),
        ProtocolErrorKind::InvalidSchema,
    );
}

#[test]
fn cloned_definition_preserves_identity_compatibility_discriminator_and_information() {
    let identity =
        extension::extension_identity("example.vendor", "cloned", "1").expect("identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("compatibility includes this KMIPKit version");
    let schema = extension::structure(
        vec![
            extension::required(
                tag(1),
                extension::scalar(ItemType::TextString).expect("TextString is supported"),
            )
            .expect("the discriminator child rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("root schema is valid");
    let discriminator = valid_discriminator(Value::text_string("match".to_owned()));
    let definition =
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("schema and discriminator agree");
    let cloned_without_information = extension::clone_extension_definition(&definition)
        .expect("a definition without optional metadata can be copied");
    assert!(extension::information(&cloned_without_information).is_none());
    let information = extension::extension_information("metadata").expect("name is valid");
    let definition = extension::with_information(definition, information)
        .expect("optional Extension Information can be attached");

    let cloned = extension::clone_extension_definition(&definition)
        .expect("a complete definition can be copied for a binding");

    assert_eq!(cloned.identity_ref(), definition.identity_ref());
    assert_eq!(cloned.compatibility(), definition.compatibility());
    assert!(
        cloned
            .discriminator()
            .has_same_scalar(definition.discriminator())
    );
    assert_eq!(
        extension::information(&cloned),
        extension::information(&definition)
    );
}

#[test]
fn registry_limit_values_allow_defaults_and_raises_but_reject_each_hard_maximum_plus_one() {
    let defaults = [
        256, 16_384, 256, 4_096, 1_048_576, 4_096, 1_048_576, 256, 16_384, 200_000, 1_048_576, 64,
    ];
    let hard = [
        1_024,
        100_000,
        4_096,
        4_096,
        16 * 1024 * 1024,
        4_096,
        16 * 1024 * 1024,
        4_096,
        100_000,
        200_000,
        4_194_304,
        64,
    ];
    let defaults_from_api = extension::defaults();
    let defaults_from_api_values = [
        defaults_from_api.max_definitions(),
        defaults_from_api.max_schema_nodes(),
        defaults_from_api.max_child_rules_per_structure(),
        defaults_from_api.max_text_bytes_per_field(),
        defaults_from_api.max_registry_text_bytes(),
        defaults_from_api.max_discriminator_scalar_bytes(),
        defaults_from_api.max_total_discriminator_scalar_bytes(),
        defaults_from_api.max_constraint_members_per_rule(),
        defaults_from_api.max_total_constraint_members(),
        defaults_from_api.max_payload_index_records(),
        defaults_from_api.max_lookup_comparisons(),
        defaults_from_api.max_depth(),
    ];
    assert_eq!(defaults_from_api_values, defaults);
    assert!(
        extension::with_values(
            defaults[0],
            defaults[1],
            defaults[2],
            defaults[3],
            defaults[4],
            defaults[5],
            defaults[6],
            defaults[7],
            defaults[8],
            defaults[9],
            defaults[10],
            defaults[11],
        )
        .is_ok()
    );
    assert!(extension::with_values(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,).is_ok());
    assert!(
        extension::with_values(
            hard[0], hard[1], hard[2], hard[3], hard[4], hard[5], hard[6], hard[7], hard[8],
            hard[9], hard[10], hard[11],
        )
        .is_ok()
    );

    for index in 0..hard.len() {
        if defaults[index] < hard[index] {
            let mut raised = defaults;
            raised[index] += 1;
            assert!(
                extension::with_values(
                    raised[0], raised[1], raised[2], raised[3], raised[4], raised[5], raised[6],
                    raised[7], raised[8], raised[9], raised[10], raised[11],
                )
                .is_ok(),
                "limit {index} may be raised above its default up to its hard maximum"
            );
        }

        let mut above_hard = hard;
        above_hard[index] += 1;
        assert_kind(
            extension::with_values(
                above_hard[0],
                above_hard[1],
                above_hard[2],
                above_hard[3],
                above_hard[4],
                above_hard[5],
                above_hard[6],
                above_hard[7],
                above_hard[8],
                above_hard[9],
                above_hard[10],
                above_hard[11],
            ),
            ProtocolErrorKind::ResourceLimit,
        );
    }
}

#[test]
fn extension_information_text_fields_use_their_utf8_byte_limits() {
    assert!(extension::extension_information(&"x".repeat(4_096)).is_ok());
    assert_kind(
        extension::extension_information(&"x".repeat(4_097)),
        ProtocolErrorKind::ResourceLimit,
    );

    let information = extension::extension_information("name").expect("name is valid");
    assert!(extension::with_description(information.clone(), &"é".repeat(2_048)).is_ok());
    assert_kind(
        extension::with_description(information, &format!("{}x", "é".repeat(2_048))),
        ProtocolErrorKind::ResourceLimit,
    );
}

#[test]
fn extension_definition_rejects_a_discriminator_schema_type_mismatch() {
    let identity =
        extension::extension_identity("example.vendor", "typed", "1").expect("identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("compatibility includes this KMIPKit version");
    let schema = extension::structure(
        vec![
            extension::required(
                tag(1),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("required discriminator rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("root schema is valid");

    assert_kind(
        extension::extension_definition(
            identity,
            compatibility,
            valid_discriminator(Value::text_string("text".to_owned())),
            schema,
        ),
        ProtocolErrorKind::InvalidSchema,
    );
}
