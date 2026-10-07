use super::{
    ClientConfiguration, candidate_matches_discriminator, client_extension_registry,
    client_request_message_extension, discriminator_candidates, inspect, is_recognized,
    validate_extension_value,
};
use crate::extension_registry_test_support::index_compilation_attempts;
use kmipkit_protocol::extension;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::extension_fixtures as generated_fixtures;

fn vendor_tag() -> Tag {
    RawTag::new(0x42_0001)
        .expect("the test tag fits in the KMIP Tag width")
        .try_checked()
        .expect("the test tag uses the vendor allocation")
}

fn fixture_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the generated fixture tag fits in the KMIP Tag width")
        .try_checked()
        .expect("the generated fixture tag uses an allocated KMIP range")
}

fn fixture_schema(schema: &generated_fixtures::Schema) -> extension::ExtensionSchema {
    if schema.kind == ItemType::Structure {
        let children = schema
            .children
            .iter()
            .map(|rule| {
                extension::required(fixture_tag(rule.tag), fixture_schema(&rule.schema))
                    .expect("the generated required schema child is valid")
            })
            .collect();
        extension::structure(children, Vec::new(), true)
            .expect("the generated Structure schema is valid")
    } else {
        let scalar = extension::scalar(schema.kind)
            .expect("the generated scalar schema has a scalar Item Type");
        if schema.has_range {
            extension::with_signed_range(scalar, schema.minimum, schema.maximum)
                .expect("the generated signed range is valid")
        } else {
            scalar
        }
    }
}

fn fixture_scalar(item_type: ItemType, text: &str, number: i64) -> Value {
    match item_type {
        ItemType::TextString => Value::text_string(text.to_owned()),
        ItemType::Enumeration => Value::enumeration(
            u32::try_from(number).expect("the generated Enumeration fixture fits in u32"),
        ),
        ItemType::LongInteger => Value::long_integer(number),
        _ => panic!("the generated fixture uses an unsupported scalar Item Type"),
    }
}

fn fixture_value(item: &generated_fixtures::Item) -> Value {
    if item.kind == ItemType::Structure {
        Value::structure(fixture_payload(item.children))
    } else {
        fixture_scalar(item.kind, item.text, item.number)
    }
}

pub(crate) fn fixture_payload(items: &[generated_fixtures::Item]) -> Structure {
    let mut structure = Structure::new();
    for item in items {
        structure
            .try_push(
                Item::new(fixture_tag(item.tag), fixture_value(item))
                    .expect("the generated fixture Item is valid"),
            )
            .expect("the generated fixture fits in a TTLV Structure");
    }
    structure
}

pub(crate) fn fixture_definition(
    fixture: &generated_fixtures::Definition,
) -> extension::ExtensionDefinition {
    let identity = extension::extension_identity(fixture.vendor, fixture.name, fixture.version)
        .expect("the generated extension identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the generated compatibility range includes KMIP 2.1");
    let mut tags = fixture.path.iter().copied();
    let first = fixture_tag(tags.next().expect("discriminator path is not empty"));
    let mut path = extension::ttlv_path(first).expect("the generated discriminator path is valid");
    for raw_tag in tags {
        path = extension::with_child_tag(path, fixture_tag(raw_tag))
            .expect("the generated discriminator path remains bounded");
    }
    let discriminator = extension::discriminator(
        path,
        fixture_scalar(
            fixture.discriminator_type,
            fixture.discriminator_text,
            fixture.discriminator_number,
        ),
    )
    .expect("the generated discriminator is a scalar");
    extension::extension_definition(
        identity,
        compatibility,
        discriminator,
        fixture_schema(&fixture.schema),
    )
    .expect("the generated definition is valid")
}

pub(crate) fn shared_fixture_configuration(reversed: bool) -> ClientConfiguration {
    let definitions = (0..generated_fixtures::DEFINITIONS.len())
        .map(|offset| {
            let index = if reversed {
                generated_fixtures::DEFINITIONS.len() - offset - 1
            } else {
                offset
            };
            fixture_definition(&generated_fixtures::DEFINITIONS[index])
        })
        .collect();
    ClientConfiguration::new(
        client_extension_registry(definitions, extension::defaults())
            .expect("shared fixture definitions produce a valid registry"),
    )
}

fn assert_fixture_structure(
    actual: &Structure,
    expected: &[generated_fixtures::Item],
    fixture_id: &str,
) {
    let view = actual.view();
    assert_fixture_view(&view, expected, fixture_id);
}

fn assert_fixture_view(
    actual: &StructureView<'_>,
    expected: &[generated_fixtures::Item],
    fixture_id: &str,
) {
    let children = actual.children();
    assert_eq!(children.len(), expected.len(), "{fixture_id} item count");
    for (index, (actual, expected)) in children.iter().zip(expected).enumerate() {
        assert_eq!(
            actual.tag().raw(),
            expected.tag,
            "{fixture_id} item {index} tag"
        );
        assert_eq!(
            actual.item_type(),
            expected.kind,
            "{fixture_id} item {index} type"
        );
        actual.with_value(|value| match value {
            ValueView::Structure(nested) if expected.kind == ItemType::Structure => {
                assert_fixture_view(&nested, expected.children, fixture_id);
            }
            ValueView::TextString(text) if expected.kind == ItemType::TextString => {
                assert_eq!(text, expected.text, "{fixture_id} item {index} text");
            }
            ValueView::Enumeration(number) if expected.kind == ItemType::Enumeration => {
                assert_eq!(
                    i64::from(*number),
                    expected.number,
                    "{fixture_id} item {index} enumeration"
                );
            }
            ValueView::LongInteger(number) if expected.kind == ItemType::LongInteger => {
                assert_eq!(
                    *number, expected.number,
                    "{fixture_id} item {index} integer"
                );
            }
            _ => {
                panic!("generated fixture value type does not match its Item Type in {fixture_id}")
            }
        });
    }
}

fn definition(name: &str, discriminator_value: &str) -> extension::ExtensionDefinition {
    let identity = extension::extension_identity("example.vendor", name, "1")
        .expect("the test extension identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the test compatibility range includes this client");
    let path = extension::ttlv_path(vendor_tag()).expect("the test path is valid");
    let discriminator =
        extension::discriminator(path, Value::text_string(discriminator_value.to_owned()))
            .expect("the test discriminator is a valid scalar");
    let child_schema =
        extension::scalar(ItemType::TextString).expect("Text String is a scalar schema");
    let child = extension::required(vendor_tag(), child_schema)
        .expect("the test discriminator child is valid");
    let schema = extension::structure(vec![child], Vec::new(), false)
        .expect("the test extension schema is valid");

    extension::extension_definition(identity, compatibility, discriminator, schema)
        .expect("the test definition is valid")
}

fn discriminator_payload(value: Value) -> Structure {
    let mut payload = Structure::new();
    payload
        .try_push(Item::new(vendor_tag(), value).expect("the discriminator payload item is valid"))
        .expect("the discriminator payload fits in a Structure");
    payload
}

fn limits_with_overrides(overrides: [u64; 12]) -> extension::ExtensionRegistryLimits {
    extension::with_values(
        overrides[0],
        overrides[1],
        overrides[2],
        overrides[3],
        overrides[4],
        overrides[5],
        overrides[6],
        overrides[7],
        overrides[8],
        overrides[9],
        overrides[10],
        overrides[11],
    )
    .expect("the test limit overrides remain under every hard maximum")
}

fn limits_with_override(index: usize, value: u64) -> extension::ExtensionRegistryLimits {
    let defaults = extension::defaults();
    let mut overrides = [
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
        defaults.max_depth(),
    ];
    overrides[index] = value;
    limits_with_overrides(overrides)
}

#[test]
fn registry_limits_reject_before_compiling_or_reserving_indexes() {
    let invalid_cases = [
        (
            vec![definition("count", "value")],
            limits_with_override(0, 0),
        ),
        (
            vec![definition("name-too-long", "value")],
            limits_with_override(3, 4),
        ),
        (
            vec![definition("aggregate-text", "value")],
            limits_with_override(4, 1),
        ),
        (
            vec![definition("scalar", "too-long")],
            limits_with_override(5, 3),
        ),
        (
            vec![definition("aggregate-scalar", "value")],
            limits_with_override(6, 3),
        ),
        (
            vec![definition("schema-node", "value")],
            limits_with_override(1, 1),
        ),
    ];

    let initial = index_compilation_attempts();
    for (definitions, limits) in invalid_cases {
        assert!(
            client_extension_registry(definitions, limits).is_err(),
            "each over-limit registry is rejected"
        );
        assert_eq!(
            index_compilation_attempts(),
            initial,
            "rejected registry limits must be checked before index reservations"
        );
    }

    client_extension_registry(vec![definition("valid", "value")], extension::defaults())
        .expect("a valid registry reaches index compilation");
    assert_eq!(index_compilation_attempts(), initial + 1);
}

#[test]
fn registered_request_values_retain_their_source_registry_identity() {
    let registry_a =
        client_extension_registry(vec![definition("alpha", "alpha-v1")], extension::defaults())
            .expect("registry A is valid");
    let registry_b =
        client_extension_registry(vec![definition("alpha", "alpha-v1")], extension::defaults())
            .expect("registry B is independently valid");
    let configuration_a = ClientConfiguration::new(registry_a);
    let configuration_b = ClientConfiguration::new(registry_b);
    let identity = extension::extension_identity("example.vendor", "alpha", "1")
        .expect("the registered identity is valid");
    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(vendor_tag(), Value::text_string("alpha-v1".to_owned()))
                .expect("the discriminator item is valid"),
        )
        .expect("the payload fits in the TTLV structure");
    let registered = validate_extension_value(
        configuration_a.extension_registry(),
        identity,
        payload,
        &CodecLimits::defaults(),
    )
    .expect("registry A validates its registered payload");
    assert!(format!("{configuration_a:?}").contains("definition_count: 1"));
    assert_eq!(
        format!("{registered:?}"),
        "RegisteredExtensionValue([REDACTED])"
    );
    let request_extension = client_request_message_extension(registered, true)
        .expect("the request-use criticality is explicit");

    assert!(request_extension.is_owned_by(&configuration_a));
    assert!(
        !request_extension.is_owned_by(&configuration_b),
        "a request extension sealed by registry A must not be accepted by configuration B"
    );
}

#[test]
fn duplicate_identity_and_exact_discriminator_keys_are_rejected() {
    let duplicate_identity = client_extension_registry(
        vec![
            definition("same-identity", "first-value"),
            definition("same-identity", "second-value"),
        ],
        extension::defaults(),
    );
    assert!(duplicate_identity.is_err());

    let duplicate_discriminator = client_extension_registry(
        vec![
            definition("first-identity", "same-value"),
            definition("second-identity", "same-value"),
        ],
        extension::defaults(),
    );
    assert!(duplicate_discriminator.is_err());
}

#[test]
fn inspection_preserves_unrecognized_vendor_and_scalar_mismatches() {
    let registry =
        client_extension_registry(vec![definition("alpha", "alpha-v1")], extension::defaults())
            .expect("the discriminator registry is valid");

    let wrong_vendor = inspect(
        &registry,
        "different.vendor",
        discriminator_payload(Value::text_string("alpha-v1".to_owned())),
        &CodecLimits::defaults(),
    )
    .expect("an unregistered vendor remains generic");
    let wrong_scalar_type = inspect(
        &registry,
        "example.vendor",
        discriminator_payload(Value::integer(17)),
        &CodecLimits::defaults(),
    )
    .expect("a discriminator with another item type remains generic");

    assert!(!is_recognized(&wrong_vendor));
    assert!(!is_recognized(&wrong_scalar_type));
}

#[test]
fn inspection_rejects_vendor_payload_index_and_comparison_limit_overruns() {
    let bounded_vendor_registry = client_extension_registry(
        vec![definition("alpha", "alpha-v1")],
        limits_with_override(3, 14),
    )
    .expect("the definition's vendor identifier is exactly within the text limit");
    let long_vendor = inspect(
        &bounded_vendor_registry,
        "example.vendor-too-long",
        Structure::new(),
        &CodecLimits::defaults(),
    )
    .expect_err("inspection rejects a vendor identifier beyond the configured limit");
    assert_eq!(
        long_vendor.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );

    let no_index_records = client_extension_registry(
        vec![definition("alpha", "alpha-v1")],
        limits_with_override(9, 0),
    )
    .expect("zero payload index records may be configured");
    let index_limit = inspect(
        &no_index_records,
        "example.vendor",
        discriminator_payload(Value::text_string("alpha-v1".to_owned())),
        &CodecLimits::defaults(),
    )
    .expect_err("the structure record exceeds a zero-record index budget");
    assert_eq!(
        index_limit.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );

    let no_comparisons = client_extension_registry(
        vec![definition("alpha", "alpha-v1")],
        limits_with_override(10, 0),
    )
    .expect("zero lookup comparisons may be configured");
    let comparison_limit = inspect(
        &no_comparisons,
        "example.vendor",
        discriminator_payload(Value::text_string("alpha-v1".to_owned())),
        &CodecLimits::defaults(),
    )
    .expect_err("matching a discriminator requires a bounded tag comparison");
    assert_eq!(
        comparison_limit.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
}

#[test]
fn internal_registry_error_messages_remain_static_and_safe() {
    assert_eq!(
        super::RegistryConstructionFailure.to_string(),
        "extension registry construction failed"
    );
    assert_eq!(
        super::UnregisteredExtension.to_string(),
        "extension identity is not registered"
    );
}

#[test]
fn discriminator_index_selects_by_exact_scalar_after_fingerprinting() {
    let registry = client_extension_registry(
        vec![
            definition("alpha", "alpha-v1"),
            definition("beta", "beta-v1"),
        ],
        extension::defaults(),
    )
    .expect("distinct scalars may share a path");
    let observed = Item::new(vendor_tag(), Value::text_string("beta-v1".to_owned()))
        .expect("the observed discriminator is valid TTLV");

    let matches = observed.with_value(|value| {
        let fingerprint =
            extension::scalar_value_fingerprint(&value).expect("the observed value is a scalar");
        let candidates =
            discriminator_candidates(&registry, "example.vendor", &[vendor_tag()], fingerprint)
                .expect("the exact path has a candidate bucket");
        candidates
            .iter()
            .filter_map(|index| registry.definitions.get(*index))
            .filter(|candidate| candidate.discriminator().matches_value_ref(&value))
            .map(|candidate| extension::identity(candidate).name().to_owned())
            .collect::<Vec<_>>()
    });

    assert_eq!(matches, ["beta"]);
}

#[test]
fn candidate_match_checks_the_full_vendor_path_and_scalar_tuple() {
    let registry = client_extension_registry(
        vec![
            definition("alpha", "alpha-v1"),
            definition("beta", "beta-v1"),
        ],
        extension::defaults(),
    )
    .expect("distinct scalars may share a path");
    let observed = Item::new(vendor_tag(), Value::text_string("beta-v1".to_owned()))
        .expect("the observed discriminator is valid TTLV");
    let wrong_type =
        Item::new(vendor_tag(), Value::integer(7)).expect("the observed integer is valid TTLV");
    let wrong_value = Item::new(vendor_tag(), Value::text_string("alpha-v1".to_owned()))
        .expect("the alternate observed discriminator is valid TTLV");
    let wrong_path = RawTag::new(0x42_0002)
        .expect("the alternate test tag fits in the KMIP Tag width")
        .try_checked()
        .expect("the alternate test tag uses the vendor allocation");

    let outcomes = observed.with_value(|value| {
        let exact_match = candidate_matches_discriminator(
            &registry,
            1,
            "example.vendor",
            &[vendor_tag()],
            &value,
        );
        let wrong_vendor = candidate_matches_discriminator(
            &registry,
            1,
            "different.vendor",
            &[vendor_tag()],
            &value,
        );
        let wrong_path =
            candidate_matches_discriminator(&registry, 1, "example.vendor", &[wrong_path], &value);
        let wrong_scalar_value = wrong_value.with_value(|wrong_value| {
            candidate_matches_discriminator(
                &registry,
                1,
                "example.vendor",
                &[vendor_tag()],
                &wrong_value,
            )
        });
        let out_of_range = candidate_matches_discriminator(
            &registry,
            usize::MAX,
            "example.vendor",
            &[vendor_tag()],
            &value,
        );
        let wrong_item_type = wrong_type.with_value(|wrong_value| {
            candidate_matches_discriminator(
                &registry,
                1,
                "example.vendor",
                &[vendor_tag()],
                &wrong_value,
            )
        });

        (
            exact_match,
            wrong_vendor,
            wrong_path,
            wrong_scalar_value,
            wrong_item_type,
            out_of_range,
        )
    });

    assert_eq!(outcomes, (true, false, false, false, false, false));
}

#[test]
fn every_shared_inbound_fixture_matches_internal_outcome_ids_and_preserved_ttlv_in_both_orders() {
    for reversed in [false, true] {
        let definitions = (0..generated_fixtures::DEFINITIONS.len())
            .map(|offset| {
                let index = if reversed {
                    generated_fixtures::DEFINITIONS.len() - offset - 1
                } else {
                    offset
                };
                fixture_definition(&generated_fixtures::DEFINITIONS[index])
            })
            .collect();
        let registry = client_extension_registry(definitions, extension::defaults())
            .expect("the shared fixture definitions produce a valid registry");

        for fixture in generated_fixtures::CASES {
            assert!(
                !fixture.critical,
                "{} inbound criticality fixture",
                fixture.id
            );
            let details = super::inspect_with_details(
                &registry,
                fixture.vendor,
                fixture_payload(fixture.payload),
                &CodecLimits::defaults(),
            )
            .expect("the shared inbound fixture is within all inspection limits");
            let expected_outcome = match fixture.outcome {
                "recognized" => super::InspectionOutcome::Recognized,
                "unrecognized.schema_invalid" => super::InspectionOutcome::SchemaInvalid,
                "unrecognized.ambiguous" => super::InspectionOutcome::Ambiguous,
                "unrecognized.no_match" => super::InspectionOutcome::NoMatch,
                _ => panic!("unsupported generated inbound outcome code"),
            };
            assert_eq!(details.outcome, expected_outcome, "{} outcome", fixture.id);
            assert_eq!(
                details.matched_count,
                fixture.matched_ids.len(),
                "{} matched count",
                fixture.id
            );
            assert_eq!(
                details.recognition.is_recognized(),
                fixture.typed,
                "{} recognized",
                fixture.id
            );
            assert_eq!(
                details.recognition.validated_value().is_some(),
                fixture.typed,
                "{} typed value",
                fixture.id
            );

            let mut matched_ids = details.matched_definition_indices[..details.matched_count]
                .iter()
                .flatten()
                .map(|index| {
                    let definition = registry
                        .definitions
                        .get(*index)
                        .expect("internal match index names a registered definition");
                    let identity = extension::identity(definition);
                    generated_fixtures::DEFINITIONS
                        .iter()
                        .find(|candidate| {
                            candidate.vendor == identity.vendor_identifier()
                                && candidate.name == identity.name()
                                && candidate.version == identity.version()
                        })
                        .expect("registered identity maps to one generated fixture definition")
                        .id
                })
                .collect::<Vec<_>>();
            matched_ids.sort_unstable();
            assert_eq!(
                matched_ids, fixture.matched_ids,
                "{} matched IDs",
                fixture.id
            );
            assert_fixture_structure(
                super::generic_value(&details.recognition),
                fixture.payload,
                fixture.id,
            );
        }
    }
}
