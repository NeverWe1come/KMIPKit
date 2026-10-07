//! Client-owned immutable extension registry tests.
//!
//! Traceability: KMIPKIT-0012-FR-001, KMIPKIT-0012-FR-009, and
//! KMIPKIT-0012-SC-002.

use kmipkit_client::extension_registry;
use kmipkit_client::{ClientError, ClientErrorCategory};
use kmipkit_protocol::extension;
use kmipkit_ttlv::{Item, ItemType, RawTag, Tag, ValueView};

const VENDOR_IDENTIFIER: &str = "example.vendor";

struct FixtureDefinition {
    name: &'static str,
    version: &'static str,
    discriminator_path: &'static [u32],
    discriminator_value: &'static str,
}

// T010 uses minimal typed Rust schemas with identity and discriminator values
// shared with tests/fixtures/extensions/cases.json. These are not the full
// schemas in that corpus; T008 remains the full cross-adapter parity corpus.
const FIXTURES: &[FixtureDefinition] = &[
    FixtureDefinition {
        name: "alpha",
        version: "1",
        discriminator_path: &[0x42_0001],
        discriminator_value: "alpha-v1",
    },
    FixtureDefinition {
        name: "ambiguous-alpha",
        version: "1",
        discriminator_path: &[0x42_0010, 0x42_0011],
        discriminator_value: "route-alpha",
    },
    FixtureDefinition {
        name: "ambiguous-beta",
        version: "1",
        discriminator_path: &[0x42_0010, 0x42_0012],
        discriminator_value: "route-beta",
    },
];

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the shared extension fixture tag fits the KMIP Tag width")
        .try_checked()
        .expect("the shared extension fixture tag is in the vendor allocation")
}

fn definition(fixture: &FixtureDefinition) -> extension::ExtensionDefinition {
    definition_with(
        fixture.name,
        fixture.version,
        fixture.discriminator_path,
        fixture.discriminator_value,
    )
}

fn definition_with(
    name: &str,
    version: &str,
    discriminator_path: &[u32],
    discriminator_value: &str,
) -> extension::ExtensionDefinition {
    definition_with_fields(
        VENDOR_IDENTIFIER,
        name,
        version,
        name,
        None,
        discriminator_path,
        discriminator_value,
    )
}

fn definition_with_fields(
    vendor_identifier: &str,
    name: &str,
    version: &str,
    information_name: &str,
    description: Option<&str>,
    discriminator_path: &[u32],
    discriminator_value: &str,
) -> extension::ExtensionDefinition {
    let identity = extension::extension_identity(vendor_identifier, name, version)
        .expect("the shared fixture identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the shared fixture is compatible with this client");

    let mut path = extension::ttlv_path(tag(discriminator_path[0]))
        .expect("the fixture discriminator path is non-empty");
    for raw_tag in &discriminator_path[1..] {
        path = extension::with_child_tag(path, tag(*raw_tag))
            .expect("the fixture discriminator path is valid");
    }
    let discriminator = extension::discriminator(
        path,
        kmipkit_ttlv::Value::text_string(discriminator_value.to_owned()),
    )
    .expect("the fixture discriminator is a text scalar");

    let mut schema =
        extension::scalar(ItemType::TextString).expect("Text String is a supported schema scalar");
    for raw_tag in discriminator_path.iter().rev() {
        let child = extension::required(tag(*raw_tag), schema)
            .expect("the fixture discriminator child rule is valid");
        schema = extension::structure(vec![child], Vec::new(), false)
            .expect("the fixture discriminator schema is valid");
    }

    let definition =
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("the shared fixture definition is internally consistent");
    let mut information = extension::extension_information(information_name)
        .expect("the shared fixture metadata name is non-empty");
    if let Some(description) = description {
        information = extension::with_description(information, description)
            .expect("the shared fixture metadata description is valid");
    }
    extension::with_information(definition, information)
        .expect("the shared fixture metadata is valid")
}

fn byte_definition_with(
    name: &str,
    discriminator_value: Vec<u8>,
) -> extension::ExtensionDefinition {
    let identity = extension::extension_identity(VENDOR_IDENTIFIER, name, "1")
        .expect("the byte extension identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the byte extension is compatible with this client");
    let path = extension::ttlv_path(tag(0x42_0001)).expect("the byte discriminator path is valid");
    let discriminator =
        extension::discriminator(path, kmipkit_ttlv::Value::byte_string(discriminator_value))
            .expect("the byte discriminator is within the hard scalar limit");
    let schema = extension::structure(
        vec![
            extension::required(
                tag(0x42_0001),
                extension::scalar(ItemType::ByteString).expect("Byte String is supported"),
            )
            .expect("the byte discriminator rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("the byte extension schema is valid");
    let definition =
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("the byte extension definition is valid");
    let information =
        extension::extension_information(name).expect("the byte extension metadata name is valid");
    extension::with_information(definition, information)
        .expect("the byte extension metadata is valid")
}

fn identity(fixture: &FixtureDefinition) -> extension::ExtensionIdentity {
    extension::extension_identity(VENDOR_IDENTIFIER, fixture.name, fixture.version)
        .expect("the shared fixture identity is valid")
}

fn registry(fixtures: &[&FixtureDefinition]) -> extension_registry::ClientExtensionRegistry {
    extension_registry::client_extension_registry(
        fixtures.iter().map(|fixture| definition(fixture)).collect(),
        extension::defaults(),
    )
    .expect("the selected shared fixture definitions form a valid registry")
}

fn definition_count_as_usize(registry: &extension_registry::ClientExtensionRegistry) -> usize {
    usize::try_from(extension_registry::definition_count(registry))
        .expect("the selected fixture registry count fits the platform address space")
}

fn metadata_names(registry: &extension_registry::ClientExtensionRegistry) -> Vec<String> {
    (0..definition_count_as_usize(registry))
        .map(|index| {
            let definition = extension_registry::definition_at(registry, index)
                .expect("every index below definition_count has a definition");
            let information = extension::information(definition)
                .expect("every shared fixture definition has metadata");
            let information = extension::to_ttlv(information)
                .expect("fixture metadata converts to its Extension Information structure");
            information
                .view()
                .children()
                .first()
                .and_then(|field: &Item| {
                    field.with_value(|value| match value {
                        ValueView::TextString(name) => Some((*name).to_owned()),
                        _ => None,
                    })
                })
                .expect("Extension Name is the required first metadata field")
        })
        .collect()
}

fn ordered_identities(
    registry: &extension_registry::ClientExtensionRegistry,
) -> Vec<extension::ExtensionIdentity> {
    (0..definition_count_as_usize(registry))
        .map(|index| {
            let definition = extension_registry::definition_at(registry, index)
                .expect("every index below definition_count has a definition");
            extension::identity(definition)
        })
        .collect()
}

#[test]
fn client_registries_keep_definition_snapshots_isolated() {
    // KMIPKIT-0012-FR-001: a registry belongs to one client configuration.
    let alpha = &FIXTURES[0];
    let beta = &FIXTURES[2];
    let alpha_registry = registry(&[alpha]);
    let beta_registry = registry(&[beta]);

    assert_eq!(extension_registry::definition_count(&alpha_registry), 1);
    assert_eq!(extension_registry::definition_count(&beta_registry), 1);

    let alpha_definition =
        extension_registry::definition_for_identity(&alpha_registry, identity(alpha))
            .expect("the first registry contains its own definition");
    assert_eq!(extension::identity(alpha_definition), identity(alpha));
    assert!(
        extension_registry::definition_for_identity(&alpha_registry, identity(beta)).is_none(),
        "the first registry does not expose the second registry's definition"
    );

    let beta_definition =
        extension_registry::definition_for_identity(&beta_registry, identity(beta))
            .expect("the second registry contains its own definition");
    assert_eq!(extension::identity(beta_definition), identity(beta));
    assert!(
        extension_registry::definition_for_identity(&beta_registry, identity(alpha)).is_none(),
        "the second registry does not expose the first registry's definition"
    );
}

#[test]
fn registry_read_operations_leave_the_constructed_snapshot_unchanged() {
    let selected = [&FIXTURES[0], &FIXTURES[1]];
    let registry = registry(&selected);
    let initial_count = extension_registry::definition_count(&registry);
    let initial_metadata = metadata_names(&registry);

    for fixture in selected {
        let found = extension_registry::definition_for_identity(&registry, identity(fixture))
            .expect("every registered identity remains addressable");
        assert_eq!(extension::identity(found), identity(fixture));
    }
    assert!(
        extension_registry::definition_for_identity(&registry, identity(&FIXTURES[2])).is_none(),
        "an identity absent at construction remains absent"
    );
    let first_out_of_range_index = usize::try_from(initial_count)
        .expect("the selected fixture registry count fits the platform address space");
    assert!(extension_registry::definition_at(&registry, first_out_of_range_index).is_none());

    assert_eq!(
        extension_registry::definition_count(&registry),
        initial_count
    );
    assert_eq!(metadata_names(&registry), initial_metadata);
}

#[test]
fn definition_and_metadata_views_ignore_registration_order() {
    // KMIPKIT-0012-FR-009 and KMIPKIT-0012-SC-002: registry-local definition
    // and metadata views are deterministic regardless of registration order.
    let forward = registry(&[&FIXTURES[0], &FIXTURES[1], &FIXTURES[2]]);
    let reverse = registry(&[&FIXTURES[2], &FIXTURES[1], &FIXTURES[0]]);

    let forward_metadata = metadata_names(&forward);
    let reverse_metadata = metadata_names(&reverse);
    assert_eq!(forward_metadata, reverse_metadata);
    assert!(
        ordered_identities(&forward) == ordered_identities(&reverse),
        "definition_at returns the same identity order for equivalent registries"
    );

    for fixture in FIXTURES {
        let expected = identity(fixture);
        for registry in [&forward, &reverse] {
            let found = extension_registry::definition_for_identity(registry, identity(fixture))
                .expect("identity lookup is independent of registration order");
            assert_eq!(extension::identity(found), expected);
        }
    }

    // FR-009: these are deterministic metadata list/map views over the local
    // registry. They do not indicate that a remote KMIP server supports them.
    let local_metadata_map = |registry: &extension_registry::ClientExtensionRegistry| {
        (0..definition_count_as_usize(registry))
            .filter_map(|index| extension_registry::definition_at(registry, index))
            .map(|definition| {
                let identity = extension::identity(definition);
                (
                    (
                        identity.vendor_identifier().to_owned(),
                        identity.name().to_owned(),
                        identity.version().to_owned(),
                    ),
                    extension::information(definition)
                        .map(|information| information.name().to_owned()),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    assert_eq!(local_metadata_map(&forward), local_metadata_map(&reverse));
}

#[test]
fn registry_rejects_duplicate_exact_discriminator_keys() {
    let first = definition_with("alpha", "1", &[0x42_0001], "alpha-v1");
    let second = definition_with("renamed-alpha", "1", &[0x42_0001], "alpha-v1");

    let result: Result<_, ClientError> =
        extension_registry::client_extension_registry(vec![first, second], extension::defaults());
    let error = result.expect_err("one vendor cannot register the same exact discriminator twice");

    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
    assert!(matches!(
        error,
        ClientError::Protocol { error, .. }
            if error.kind() == kmipkit_protocol::ProtocolErrorKind::DuplicateKey
    ));
}

#[test]
fn client_configurations_own_isolated_immutable_registries() {
    let alpha = definition_with("alpha", "1", &[0x42_0001], "alpha-v1");
    let beta = definition_with("beta", "1", &[0x42_0002], "beta-v1");
    let alpha_identity = extension::identity(&alpha);
    let beta_identity = extension::identity(&beta);

    let alpha_registry =
        extension_registry::client_extension_registry(vec![alpha], extension::defaults())
            .expect("alpha registry is valid");
    let beta_registry =
        extension_registry::client_extension_registry(vec![beta], extension::defaults())
            .expect("beta registry is valid");
    let alpha_configuration = extension_registry::ClientConfiguration::new(alpha_registry);
    let beta_configuration = extension_registry::ClientConfiguration::new(beta_registry);

    let alpha_registry = alpha_configuration.extension_registry();
    let beta_registry = beta_configuration.extension_registry();
    assert_eq!(extension_registry::definition_count(alpha_registry), 1);
    assert_eq!(extension_registry::definition_count(beta_registry), 1);
    assert!(
        extension_registry::definition_for_identity(alpha_registry, alpha_identity.clone())
            .is_some()
    );
    assert!(
        extension_registry::definition_for_identity(alpha_registry, beta_identity.clone())
            .is_none(),
        "one configuration must not observe another configuration's definitions"
    );
    assert!(extension_registry::definition_for_identity(beta_registry, beta_identity).is_some());
    assert!(
        extension_registry::definition_for_identity(beta_registry, alpha_identity).is_none(),
        "configuration registries stay isolated after attachment"
    );
}

#[test]
fn registry_accepts_distinct_discriminator_values_at_one_path() {
    let alpha = definition_with("alpha", "1", &[0x42_0001], "alpha-v1");
    let beta = definition_with("beta", "1", &[0x42_0001], "beta-v1");
    let registry =
        extension_registry::client_extension_registry(vec![alpha, beta], extension::defaults())
            .expect("one vendor may use distinct exact values at the same path");
    assert_eq!(extension_registry::definition_count(&registry), 2);
}

#[test]
fn registry_rejects_a_definition_that_exceeds_configured_text_limits() {
    let definition = definition(&FIXTURES[0]);
    let limits = extension::with_values(
        256, 16_384, 256, 1, 1_048_576, 4_096, 1_048_576, 256, 16_384, 200_000, 1_048_576, 64,
    )
    .expect("the reduced text limit remains below every hard maximum");

    let result: Result<_, ClientError> =
        extension_registry::client_extension_registry(vec![definition], limits);
    let error = result.expect_err("the fixture identity is longer than the configured text limit");

    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert!(matches!(
        error,
        ClientError::Protocol { error, .. }
            if error.kind() == kmipkit_protocol::ProtocolErrorKind::ResourceLimit
    ));
}

fn limits_with_registry_text_and_discriminator_bytes(
    max_definitions: u64,
    max_text_bytes_per_field: u64,
    max_registry_text_bytes: u64,
    max_discriminator_scalar_bytes: u64,
    max_total_discriminator_scalar_bytes: u64,
) -> extension::ExtensionRegistryLimits {
    let defaults = extension::defaults();
    extension::with_values(
        max_definitions,
        defaults.max_schema_nodes(),
        defaults.max_child_rules_per_structure(),
        max_text_bytes_per_field,
        max_registry_text_bytes,
        max_discriminator_scalar_bytes,
        max_total_discriminator_scalar_bytes,
        defaults.max_constraint_members_per_rule(),
        defaults.max_total_constraint_members(),
        defaults.max_payload_index_records(),
        defaults.max_lookup_comparisons(),
        defaults.max_depth(),
    )
    .expect("test registry limits are within the hard maxima")
}

fn assert_registry_resource_limit(
    result: Result<extension_registry::ClientExtensionRegistry, ClientError>,
) {
    let error = result.expect_err("the configured registry limit rejects this set");
    assert!(matches!(
        error,
        ClientError::Protocol { error, .. }
            if error.kind() == kmipkit_protocol::ProtocolErrorKind::ResourceLimit
    ));
}

#[test]
fn registry_rejects_duplicate_identities_before_exposing_any_registry() {
    let first = definition_with("same-identity", "1", &[0x42_0001], "first");
    let second = definition_with("same-identity", "1", &[0x42_0002], "second");
    let result =
        extension_registry::client_extension_registry(vec![first, second], extension::defaults());
    let error = result.expect_err("duplicate identities cannot form a registry");

    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert!(matches!(
        error,
        ClientError::Protocol { error, .. }
            if error.kind() == kmipkit_protocol::ProtocolErrorKind::DuplicateKey
    ));
}

#[test]
fn registry_returns_no_partial_snapshot_when_a_later_definition_fails() {
    let accepted = definition_with("first-valid", "1", &[0x42_0001], "first-value");
    let rejected = definition_with_fields(
        VENDOR_IDENTIFIER,
        &"n".repeat(33),
        "1",
        "metadata-too-long-for-configured-field-limit",
        None,
        &[0x42_0002],
        "second-value",
    );
    let limits =
        limits_with_registry_text_and_discriminator_bytes(4, 32, 1_048_576, 4_096, 1_048_576);

    let result = extension_registry::client_extension_registry(vec![accepted, rejected], limits);
    let error = result.expect_err(
        "a valid first definition cannot expose a partial registry when a later one exceeds limits",
    );
    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
}

#[test]
fn registry_definition_count_accepts_the_configured_boundary_and_rejects_one_over() {
    let exact = definition_with("exact-count", "1", &[0x42_0001], "exact");
    let limits =
        limits_with_registry_text_and_discriminator_bytes(1, 4_096, 1_048_576, 4_096, 1_048_576);
    let registry = extension_registry::client_extension_registry(vec![exact], limits)
        .expect("one definition exactly fits the configured count");
    assert_eq!(extension_registry::definition_count(&registry), 1);

    let first = definition_with("first-count", "1", &[0x42_0001], "first");
    let second = definition_with("second-count", "1", &[0x42_0002], "second");
    assert_registry_resource_limit(extension_registry::client_extension_registry(
        vec![first, second],
        limits,
    ));
}

#[test]
fn registry_text_and_discriminator_aggregate_limits_accept_exact_and_reject_one_under() {
    let first = definition_with("aggregate-a", "1", &[0x42_0001], "value-a");
    let second = definition_with("aggregate-b", "1", &[0x42_0002], "value-b");
    let first_accounting = extension::accounting(&first).expect("first definition is bounded");
    let second_accounting = extension::accounting(&second).expect("second definition is bounded");
    let total_text = u64::try_from(first_accounting.text_bytes + second_accounting.text_bytes)
        .expect("text byte count fits u64");
    let total_discriminator =
        u64::try_from(first_accounting.discriminator_bytes + second_accounting.discriminator_bytes)
            .expect("discriminator byte count fits u64");

    let exact_limits = limits_with_registry_text_and_discriminator_bytes(
        2,
        4_096,
        total_text,
        7,
        total_discriminator,
    );
    let exact = extension_registry::client_extension_registry(
        vec![
            definition_with("aggregate-a", "1", &[0x42_0001], "value-a"),
            definition_with("aggregate-b", "1", &[0x42_0002], "value-b"),
        ],
        exact_limits,
    );
    assert!(exact.is_ok(), "exact aggregate byte limits are inclusive");

    assert_registry_resource_limit(extension_registry::client_extension_registry(
        vec![
            definition_with("aggregate-a", "1", &[0x42_0001], "value-a"),
            definition_with("aggregate-b", "1", &[0x42_0002], "value-b"),
        ],
        limits_with_registry_text_and_discriminator_bytes(
            2,
            4_096,
            total_text - 1,
            7,
            total_discriminator,
        ),
    ));

    assert_registry_resource_limit(extension_registry::client_extension_registry(
        vec![definition_with(
            "single-scalar-over",
            "1",
            &[0x42_0001],
            "value-a",
        )],
        limits_with_registry_text_and_discriminator_bytes(1, 4_096, 1_048_576, 6, 1_048_576),
    ));
}

#[test]
fn registry_enforces_configured_text_field_limit_on_every_identity_and_metadata_field() {
    let limits =
        || limits_with_registry_text_and_discriminator_bytes(1, 5, 1_048_576, 4_096, 1_048_576);
    let cases = [
        ("vendor", "name", "1", "meta5", None),
        ("vendr", "name66", "1", "meta5", None),
        ("vendr", "name", "versio", "meta5", None),
        ("vendr", "name", "1", "meta66", None),
        ("vendr", "name", "1", "meta5", Some("desc66")),
    ];

    for (vendor, name, version, metadata_name, description) in cases {
        let definition = definition_with_fields(
            vendor,
            name,
            version,
            metadata_name,
            description,
            &[0x42_0001],
            "value",
        );
        assert_registry_resource_limit(extension_registry::client_extension_registry(
            vec![definition],
            limits(),
        ));
    }

    let exact = definition_with_fields(
        "vendr",
        "name5",
        "ver55",
        "meta5",
        Some("desc5"),
        &[0x42_0001],
        "value",
    );
    assert!(extension_registry::client_extension_registry(vec![exact], limits()).is_ok());
}

#[test]
fn registry_count_hard_boundary_accepts_exact_maximum_and_rejects_one_over() {
    let hard_maximum = 1_024_u64;
    let limits = limits_with_registry_text_and_discriminator_bytes(
        hard_maximum,
        4_096,
        16 * 1024 * 1024,
        4_096,
        16 * 1024 * 1024,
    );
    let definitions = |count: usize| {
        (0..count)
            .map(|index| {
                let identity = format!("definition-{index}");
                let discriminator = format!("value-{index}");
                definition_with(&identity, "1", &[0x42_0001], &discriminator)
            })
            .collect::<Vec<_>>()
    };

    let exact = extension_registry::client_extension_registry(
        definitions(usize::try_from(hard_maximum).expect("hard limit fits usize")),
        limits,
    )
    .expect("the registry accepts exactly its hard definition maximum");
    assert_eq!(extension_registry::definition_count(&exact), hard_maximum);

    let over = extension_registry::client_extension_registry(
        definitions(usize::try_from(hard_maximum + 1).expect("hard limit plus one fits usize")),
        limits,
    );
    assert_registry_resource_limit(over);
}

#[test]
fn registry_count_default_boundary_accepts_exact_default_and_rejects_one_over() {
    let limits = extension::defaults();
    let count = usize::try_from(limits.max_definitions()).expect("default count fits usize");
    let definitions = |count: usize| {
        (0..count)
            .map(|index| {
                definition_with(
                    &format!("default-{index}"),
                    "1",
                    &[0x42_0001],
                    &format!("value-{index}"),
                )
            })
            .collect::<Vec<_>>()
    };

    let exact = extension_registry::client_extension_registry(definitions(count), limits)
        .expect("the default definition maximum is inclusive");
    assert_eq!(
        extension_registry::definition_count(&exact),
        limits.max_definitions()
    );
    assert_registry_resource_limit(extension_registry::client_extension_registry(
        definitions(count + 1),
        limits,
    ));
}

#[test]
fn registry_aggregates_binary_discriminator_bytes_at_exact_and_over_limit_boundaries() {
    let exact = extension_registry::client_extension_registry(
        vec![
            byte_definition_with("bytes-a", vec![0xA1; 4]),
            byte_definition_with("bytes-b", vec![0xB2; 4]),
        ],
        limits_with_registry_text_and_discriminator_bytes(2, 4_096, 1_048_576, 4, 8),
    );
    assert!(
        exact.is_ok(),
        "the exact aggregate binary scalar limit is inclusive"
    );

    assert_registry_resource_limit(extension_registry::client_extension_registry(
        vec![
            byte_definition_with("bytes-a", vec![0xA1; 4]),
            byte_definition_with("bytes-b", vec![0xB2; 4]),
        ],
        limits_with_registry_text_and_discriminator_bytes(2, 4_096, 1_048_576, 4, 7),
    ));

    assert_registry_resource_limit(extension_registry::client_extension_registry(
        vec![byte_definition_with("scalar-over", vec![0xA1; 5])],
        limits_with_registry_text_and_discriminator_bytes(1, 4_096, 1_048_576, 4, 1_048_576),
    ));
}

#[test]
fn registry_rejects_text_totals_over_the_hard_aggregate_limit() {
    const HARD_TEXT_BYTES: usize = 16 * 1024 * 1024;
    const TEXT_BYTES_PER_DEFINITION: usize = 5 * 4_096;
    let count = HARD_TEXT_BYTES / TEXT_BYTES_PER_DEFINITION + 1;
    let vendor = "v".repeat(4_096);
    let version = "1".repeat(4_096);
    let information_name = "i".repeat(4_096);
    let description = "d".repeat(4_096);
    let definitions = (0..count)
        .map(|index| {
            let name = format!("n{index:04}") + &"x".repeat(4_091);
            let discriminator = format!("value-{index}");
            definition_with_fields(
                &vendor,
                &name,
                &version,
                &information_name,
                Some(&description),
                &[0x42_0001],
                &discriminator,
            )
        })
        .collect();
    let limits = limits_with_registry_text_and_discriminator_bytes(
        1_024,
        4_096,
        u64::try_from(HARD_TEXT_BYTES).expect("hard text maximum fits u64"),
        4_096,
        16 * 1024 * 1024,
    );

    assert_registry_resource_limit(extension_registry::client_extension_registry(
        definitions,
        limits,
    ));
}
