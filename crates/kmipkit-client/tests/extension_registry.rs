//! Client-owned immutable extension registry tests.
//!
//! Traceability: KMIPKIT-0012-FR-001, KMIPKIT-0012-FR-009, and
//! KMIPKIT-0012-SC-002.

use kmipkit_client::extension_registry;
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
    let identity = extension::extension_identity(VENDOR_IDENTIFIER, fixture.name, fixture.version)
        .expect("the shared fixture identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the shared fixture is compatible with this client");

    let mut path = extension::ttlv_path(tag(fixture.discriminator_path[0]))
        .expect("the fixture discriminator path is non-empty");
    for raw_tag in &fixture.discriminator_path[1..] {
        path = extension::with_child_tag(path, tag(*raw_tag))
            .expect("the fixture discriminator path is valid");
    }
    let discriminator = extension::discriminator(
        path,
        kmipkit_ttlv::Value::text_string(fixture.discriminator_value.to_owned()),
    )
    .expect("the fixture discriminator is a text scalar");

    let mut schema =
        extension::scalar(ItemType::TextString).expect("Text String is a supported schema scalar");
    for raw_tag in fixture.discriminator_path.iter().rev() {
        let child = extension::required(tag(*raw_tag), schema)
            .expect("the fixture discriminator child rule is valid");
        schema = extension::structure(vec![child], Vec::new(), false)
            .expect("the fixture discriminator schema is valid");
    }

    let definition =
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("the shared fixture definition is internally consistent");
    let information = extension::extension_information(fixture.name)
        .expect("the shared fixture metadata name is non-empty");
    extension::with_information(definition, information)
        .expect("the shared fixture metadata is valid")
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
    assert!(extension::identity(alpha_definition) == identity(alpha));
    assert!(
        extension_registry::definition_for_identity(&alpha_registry, identity(beta)).is_none(),
        "the first registry does not expose the second registry's definition"
    );

    let beta_definition =
        extension_registry::definition_for_identity(&beta_registry, identity(beta))
            .expect("the second registry contains its own definition");
    assert!(extension::identity(beta_definition) == identity(beta));
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
        assert!(extension::identity(found) == identity(fixture));
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
            assert!(extension::identity(found) == expected);
        }
    }
}
