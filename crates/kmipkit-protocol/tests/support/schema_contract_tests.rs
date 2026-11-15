use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Value};

use super::{
    FieldDescriptor, FieldTag, RequestPayloadSchema, ValidationReport, generated, item_type_bit,
    validate_structure_with_descriptor,
};

const SUPPORTED_SCHEMAS: [RequestPayloadSchema; 56] = [
    RequestPayloadSchema::Activate,
    RequestPayloadSchema::AddAttribute,
    RequestPayloadSchema::AdjustAttribute,
    RequestPayloadSchema::Archive,
    RequestPayloadSchema::Cancel,
    RequestPayloadSchema::Certify,
    RequestPayloadSchema::Check,
    RequestPayloadSchema::Create,
    RequestPayloadSchema::CreateKeyPair,
    RequestPayloadSchema::CreateSplitKey,
    RequestPayloadSchema::Decrypt,
    RequestPayloadSchema::DelegatedLogin,
    RequestPayloadSchema::DeleteAttribute,
    RequestPayloadSchema::DeriveKey,
    RequestPayloadSchema::Destroy,
    RequestPayloadSchema::DiscoverVersions,
    RequestPayloadSchema::Encrypt,
    RequestPayloadSchema::Export,
    RequestPayloadSchema::Get,
    RequestPayloadSchema::GetAttributeList,
    RequestPayloadSchema::GetAttributes,
    RequestPayloadSchema::GetConstraints,
    RequestPayloadSchema::GetUsageAllocation,
    RequestPayloadSchema::Hash,
    RequestPayloadSchema::Import,
    RequestPayloadSchema::Interop,
    RequestPayloadSchema::JoinSplitKey,
    RequestPayloadSchema::Locate,
    RequestPayloadSchema::Log,
    RequestPayloadSchema::Login,
    RequestPayloadSchema::Logout,
    RequestPayloadSchema::Mac,
    RequestPayloadSchema::MacVerify,
    RequestPayloadSchema::ModifyAttribute,
    RequestPayloadSchema::ObtainLease,
    RequestPayloadSchema::Pkcs11,
    RequestPayloadSchema::Ping,
    RequestPayloadSchema::Poll,
    RequestPayloadSchema::Process,
    RequestPayloadSchema::Query,
    RequestPayloadSchema::QueryAsyncRequests,
    RequestPayloadSchema::ReProvision,
    RequestPayloadSchema::Recover,
    RequestPayloadSchema::Register,
    RequestPayloadSchema::Recertify,
    RequestPayloadSchema::Rekey,
    RequestPayloadSchema::RekeyKeyPair,
    RequestPayloadSchema::Revoke,
    RequestPayloadSchema::RngRetrieve,
    RequestPayloadSchema::RngSeed,
    RequestPayloadSchema::SetAttribute,
    RequestPayloadSchema::SetConstraints,
    RequestPayloadSchema::SetDefaults,
    RequestPayloadSchema::Sign,
    RequestPayloadSchema::SignatureVerify,
    RequestPayloadSchema::Validate,
];

const ITEM_TYPES: [ItemType; 11] = [
    ItemType::Structure,
    ItemType::Integer,
    ItemType::LongInteger,
    ItemType::BigInteger,
    ItemType::Enumeration,
    ItemType::Boolean,
    ItemType::TextString,
    ItemType::ByteString,
    ItemType::DateTime,
    ItemType::Interval,
    ItemType::DateTimeExtended,
];

fn value_for(item_type: ItemType) -> Value {
    match item_type {
        ItemType::Structure => Value::structure(Structure::new()),
        ItemType::Integer => Value::integer(7),
        ItemType::LongInteger => Value::long_integer(7),
        ItemType::BigInteger => Value::big_integer(vec![7]),
        ItemType::Enumeration => Value::enumeration(7),
        ItemType::Boolean => Value::boolean(true),
        ItemType::TextString => Value::text_string("schema-fixture".to_owned()),
        ItemType::ByteString => Value::byte_string(vec![7]),
        ItemType::DateTime => Value::date_time(7),
        ItemType::Interval => Value::interval(7),
        ItemType::DateTimeExtended => Value::date_time_extended(7),
        other => panic!("test fixture does not represent new TTLV Item Type {other:?}"),
    }
}

fn item(tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(tag)
            .expect("schema descriptor tag fits the TTLV field")
            .try_checked()
            .expect("schema descriptor tag is allocated"),
        value,
    )
    .expect("schema descriptor fixture is valid generic TTLV")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for item in items {
        structure
            .try_push(item)
            .expect("descriptor contract fixture stays within TTLV limits");
    }
    structure
}

fn exact_tag(field: &FieldDescriptor) -> Option<u32> {
    match field.tag {
        FieldTag::Exact(tag) => Some(tag),
        FieldTag::ManagedObject => None,
    }
}

fn accepted_type(field: &FieldDescriptor) -> ItemType {
    ITEM_TYPES
        .into_iter()
        .find(|item_type| field.types & item_type_bit(*item_type) != 0)
        .expect("each generated field has an allowed TTLV type")
}

fn invalid_type(field: &FieldDescriptor) -> Option<ItemType> {
    ITEM_TYPES
        .into_iter()
        .find(|item_type| field.types & item_type_bit(*item_type) == 0)
}

fn contains_rule(report: &ValidationReport, rule_id: &str) -> bool {
    report
        .issues()
        .iter()
        .any(|issue| issue.code().as_str() == rule_id)
}

fn assert_ordered_fields(
    schema: RequestPayloadSchema,
    descriptor: &super::RequestSchemaDescriptor,
) -> usize {
    let mut pair_count = 0;
    for (first_index, first) in descriptor.fields.iter().enumerate() {
        let Some(first_tag) = exact_tag(first) else {
            continue;
        };
        for second in descriptor.fields.iter().skip(first_index + 1) {
            let Some(second_tag) = exact_tag(second) else {
                continue;
            };
            if first_tag == second_tag {
                continue;
            }
            pair_count += 1;
            let reversed = structure([
                item(second_tag, value_for(accepted_type(second))),
                item(first_tag, value_for(accepted_type(first))),
            ]);
            let report = validate_structure_with_descriptor(&reversed.view(), descriptor);
            assert!(
                contains_rule(&report, first.rule_id),
                "{schema:?} reversed {} after {} reports {}",
                first.name,
                second.name,
                first.rule_id
            );
        }
    }
    pair_count
}

#[test]
fn every_supported_payload_field_enforces_its_descriptor_contract() {
    let mut field_count = 0;
    let mut required_count = 0;
    let mut singleton_count = 0;
    let mut repeated_count = 0;
    let mut ordered_pair_count = 0;
    let mut type_constrained_count = 0;
    let mut dynamic_tag_count = 0;

    for schema in SUPPORTED_SCHEMAS {
        let descriptor =
            generated::descriptor(schema).expect("every supported operation has a schema");
        for field in descriptor.fields {
            field_count += 1;

            if field.min_count > 0 {
                required_count += 1;
                let report =
                    validate_structure_with_descriptor(&Structure::new().view(), &descriptor);
                assert!(
                    contains_rule(&report, field.rule_id),
                    "{schema:?} missing {} reports {}",
                    field.name,
                    field.rule_id
                );
            }

            let Some(tag) = exact_tag(field) else {
                dynamic_tag_count += 1;
                continue;
            };

            if let Some(wrong_type) = invalid_type(field) {
                type_constrained_count += 1;
                let wrong_type = structure([item(tag, value_for(wrong_type))]);
                let report = validate_structure_with_descriptor(&wrong_type.view(), &descriptor);
                assert!(
                    contains_rule(&report, field.rule_id),
                    "{schema:?} wrong type for {} reports {}",
                    field.name,
                    field.rule_id
                );
            }

            if field.max_count == Some(1) {
                singleton_count += 1;
                let duplicate = structure([
                    item(tag, value_for(accepted_type(field))),
                    item(tag, value_for(accepted_type(field))),
                ]);
                let report = validate_structure_with_descriptor(&duplicate.view(), &descriptor);
                assert!(
                    contains_rule(&report, field.rule_id),
                    "{schema:?} duplicate singleton {} reports {}",
                    field.name,
                    field.rule_id
                );
            } else if field.max_count.is_none() {
                repeated_count += 1;
                let repeated = structure([
                    item(tag, value_for(accepted_type(field))),
                    item(tag, value_for(accepted_type(field))),
                ]);
                let report = validate_structure_with_descriptor(&repeated.view(), &descriptor);
                assert!(
                    !contains_rule(&report, field.rule_id),
                    "{schema:?} repeated {} is allowed by the descriptor",
                    field.name
                );
            }
        }

        ordered_pair_count += assert_ordered_fields(schema, &descriptor);
    }

    assert_eq!(field_count, 189, "all operation field rules are visited");
    assert!(required_count > 0);
    assert!(singleton_count > 0);
    assert!(repeated_count > 0);
    assert!(ordered_pair_count > 0);
    assert!(type_constrained_count > 0);
    assert_eq!(
        dynamic_tag_count, 2,
        "Import and Register select object tags dynamically"
    );
}
