//! Explicit request-payload schema identities and borrowed Structure checks.

use kmipkit_ttlv::{ItemType, StructureView};

use super::{
    ValidationCode, ValidationIssue, ValidationPath, ValidationReport, ValidationSeverity,
    ValidationSourceRef,
};

#[path = "schemas_generated.rs"]
mod generated;

pub use generated::RequestPayloadSchema;

pub(super) const TYPE_STRUCTURE: u16 = 1 << 0;
pub(super) const TYPE_INTEGER: u16 = 1 << 1;
pub(super) const TYPE_LONG_INTEGER: u16 = 1 << 2;
pub(super) const TYPE_BIG_INTEGER: u16 = 1 << 3;
pub(super) const TYPE_ENUMERATION: u16 = 1 << 4;
pub(super) const TYPE_BOOLEAN: u16 = 1 << 5;
pub(super) const TYPE_TEXT_STRING: u16 = 1 << 6;
pub(super) const TYPE_BYTE_STRING: u16 = 1 << 7;
pub(super) const TYPE_DATE_TIME: u16 = 1 << 8;
pub(super) const TYPE_INTERVAL: u16 = 1 << 9;
pub(super) const TYPE_DATE_TIME_EXTENDED: u16 = 1 << 10;

/// Explicitly identifies the schema to use for generic TTLV validation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ValidationSchema {
    /// A KMIP 2.1 client Request Payload schema.
    RequestPayload(RequestPayloadSchema),
    /// A reusable nested KMIP 2.1 Structure schema.
    Nested(NestedStructureSchema),
    /// A known but unsupported or future schema identity.
    Unsupported,
}

#[derive(Clone, Copy)]
pub(super) enum FieldTag {
    Exact(u32),
    ManagedObject,
}

#[derive(Clone, Copy)]
pub(super) struct FieldDescriptor {
    pub(super) rule_id: &'static str,
    pub(super) name: &'static str,
    pub(super) tag: FieldTag,
    pub(super) types: u16,
    pub(super) min_count: usize,
    pub(super) max_count: Option<usize>,
    pub(super) nested_schema: Option<NestedStructureSchema>,
}

/// Explicit identity for a reusable nested KMIP 2.1 Structure schema.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NestedStructureSchema {
    /// The §7.7 Constraints Structure and its known children.
    Constraints,
    /// One §7.6 Constraint structure.
    Constraint,
    /// The §7.24 Object Types Structure.
    ObjectTypes,
    /// The §7.23 Object Groups Structure.
    ObjectGroups,
    /// The §5.1 Attributes Structure with tag-selected attribute values.
    Attributes,
    /// The §7.11 Defaults Information Structure.
    DefaultsInformation,
    /// One §7.22 Object Defaults structure.
    ObjectDefaults,
    /// The §4.16 Cryptographic Parameters attribute structure.
    CryptographicParameters,
    /// The §7.12 Derivation Parameters structure.
    DerivationParameters,
    /// The §7.18 Key Wrapping Specification structure.
    KeyWrappingSpecification,
    /// The §3.3 Encryption Key Information structure.
    EncryptionKeyInformation,
    /// The §3.3 MAC/Signature Key Information structure.
    MacSignatureKeyInformation,
    /// The §7.35 Rights structure containing repeatable Right structures.
    Rights,
    /// One §7.34 Right structure.
    Right,
    /// The §7.40 Usage Limits structure.
    UsageLimits,
    /// The §7.21 Objects structure containing Unique Identifiers.
    Objects,
    /// The §7.25 Operations structure containing Operation enumerations.
    Operations,
    /// The §5.6 Current Attribute wrapper around one attribute Item.
    CurrentAttribute,
    /// The §5.7 New Attribute wrapper around one attribute Item.
    NewAttribute,
    /// The §2.1 Certificate object structure.
    Certificate,
    /// The §2.2 Certificate Request object structure.
    CertificateRequest,
    /// The §7.1 Asynchronous Correlation Values structure.
    AsynchronousCorrelationValues,
    /// The §7.39 Ticket structure.
    Ticket,
    /// The §9.16 Protocol Version structure.
    ProtocolVersion,
    /// The §4.47 Revocation Reason structure.
    RevocationReason,
    /// The §7.27 PKCS#11 Input Parameters structure.
    Pkcs11InputParameters,
    /// The §7.28 PKCS#11 Interface structure.
    Pkcs11Interface,
    /// The §3.1 Key Block structure shared by managed cryptographic objects.
    KeyBlock,
    /// The §3.2 Key Value structure containing Key Material and optional Attributes.
    KeyValue,
    /// The §3.3 Key Wrapping Data structure.
    KeyWrappingData,
    /// The §2.3 Opaque Object structure.
    OpaqueObject,
    /// The §2.4 PGP Key managed-object structure.
    PgpKey,
    /// The §2.7 Secret Data managed-object structure.
    SecretData,
    /// The §2.8 Split Key managed-object structure and conditional fields.
    SplitKey,
}

#[derive(Clone, Copy)]
pub(super) struct RequestSchemaDescriptor {
    pub(super) source: &'static str,
    pub(super) fields: &'static [FieldDescriptor],
}

/// Validates a borrowed Structure against an explicitly selected schema.
///
/// With `None`, validation is limited to the generic TTLV model already
/// enforced by `kmipkit-ttlv`. Unknown fields in a known schema are retained
/// and are not rejected solely because the selected descriptor does not
/// recognize them.
#[must_use]
pub fn validate_structure(
    structure: &StructureView<'_>,
    schema: Option<ValidationSchema>,
) -> ValidationReport {
    let Some(schema) = schema else {
        return ValidationReport::default();
    };

    let request_payload_schema = match schema {
        ValidationSchema::RequestPayload(request_schema) => {
            let Some(descriptor) = generated::descriptor(request_schema) else {
                return unsupported_schema_report();
            };
            (request_schema, descriptor)
        }
        ValidationSchema::Nested(nested_schema) => {
            return super::nested::validate_nested_structure(structure, nested_schema);
        }
        ValidationSchema::Unsupported => return unsupported_schema_report(),
    };

    let (request_schema, descriptor) = request_payload_schema;
    let mut report = validate_structure_with_descriptor(structure, &descriptor);
    if request_schema == RequestPayloadSchema::DeriveKey {
        report.append(super::nested::validate_derive_key_conditions(structure));
    }
    report
}

fn unsupported_schema_report() -> ValidationReport {
    let mut report = ValidationReport::default();
    report.push(ValidationIssue::new(
        ValidationCode::new("KMIPKIT-0051-P002"),
        ValidationPath::root(),
        ValidationSeverity::Error,
        ValidationSourceRef::ProjectRule("KMIPKIT-0051-P002"),
    ));
    report
}

pub(super) fn validate_structure_with_descriptor(
    structure: &StructureView<'_>,
    descriptor: &RequestSchemaDescriptor,
) -> ValidationReport {
    validate_structure_with_descriptor_and_prefix(
        structure,
        descriptor,
        &ValidationPath::root().field("Request Payload"),
    )
}

pub(super) fn validate_structure_with_descriptor_and_prefix(
    structure: &StructureView<'_>,
    descriptor: &RequestSchemaDescriptor,
    prefix: &ValidationPath,
) -> ValidationReport {
    let mut report = ValidationReport::default();
    let mut state = FieldCounts::new(descriptor.fields.len());

    for (child_index, item) in structure.children().iter().enumerate() {
        let Some(field_index) = descriptor
            .fields
            .iter()
            .position(|field| field_matches(field, item.tag().raw()))
        else {
            continue;
        };
        validate_known_child(
            &mut report,
            descriptor,
            &mut state,
            field_index,
            child_index,
            item.item_type(),
            prefix,
        );
        if let ItemType::Structure = item.item_type() {
            let mut child_prefix = prefix.clone().field(descriptor.fields[field_index].name);
            if descriptor.fields[field_index].max_count.is_none() {
                child_prefix = child_prefix.index(child_index);
            }
            let child_tag = item.tag().raw();
            let nested_report = item.with_value(|value| match value {
                kmipkit_ttlv::ValueView::Structure(nested) => {
                    if matches!(descriptor.fields[field_index].tag, FieldTag::ManagedObject) {
                        super::nested::validate_managed_object(&nested, child_tag, &child_prefix)
                    } else if let Some(nested_schema) = descriptor.fields[field_index].nested_schema
                    {
                        super::nested::validate_nested_with_prefix(
                            &nested,
                            nested_schema,
                            &child_prefix,
                        )
                    } else {
                        ValidationReport::default()
                    }
                }
                _ => ValidationReport::default(),
            });
            report.append(nested_report);
        }
    }

    for (field, count) in descriptor.fields.iter().zip(state.counts) {
        if count < field.min_count {
            report.push(ValidationIssue::new(
                ValidationCode::new(field.rule_id),
                prefix.clone().field(field.name),
                ValidationSeverity::Error,
                ValidationSourceRef::Oasis {
                    document: "KMIP v2.1",
                    reference: descriptor.source,
                },
            ));
        }
    }

    report
}

struct FieldCounts {
    counts: Vec<usize>,
    last_known_field: usize,
    has_known_field: bool,
}

impl FieldCounts {
    fn new(field_count: usize) -> Self {
        Self {
            counts: vec![0; field_count],
            last_known_field: 0,
            has_known_field: false,
        }
    }
}

fn validate_known_child(
    report: &mut ValidationReport,
    descriptor: &RequestSchemaDescriptor,
    state: &mut FieldCounts,
    field_index: usize,
    child_index: usize,
    item_type: ItemType,
    prefix: &ValidationPath,
) {
    let field = descriptor.fields[field_index];
    let count = state.counts[field_index].saturating_add(1);
    state.counts[field_index] = count;

    if state.has_known_field && field_index < state.last_known_field {
        push_field_issue(
            report,
            field,
            descriptor.source,
            ValidationCode::new(field.rule_id),
            prefix,
            field.max_count.is_none().then_some(child_index),
        );
    }
    state.last_known_field = field_index;
    state.has_known_field = true;

    if field.max_count.is_some_and(|maximum| count > maximum) {
        push_field_issue(
            report,
            field,
            descriptor.source,
            ValidationCode::new(field.rule_id),
            prefix,
            Some(child_index),
        );
    }

    if field.types & item_type_bit(item_type) == 0 {
        push_field_issue(
            report,
            field,
            descriptor.source,
            ValidationCode::new(field.rule_id),
            prefix,
            (field.max_count.is_none() || count > 1).then_some(child_index),
        );
    }
}

fn field_matches(field: &FieldDescriptor, tag: u32) -> bool {
    match field.tag {
        FieldTag::Exact(expected) => tag == expected,
        FieldTag::ManagedObject => managed_object_tag(tag),
    }
}

fn managed_object_tag(tag: u32) -> bool {
    matches!(
        tag,
        0x0042_0013 // Certificate
            | 0x0042_0018 // Certificate Request
            | 0x0042_005B // Opaque Object
            | 0x0042_00BD // PGP Key
            | 0x0042_0064 // Private Key
            | 0x0042_006D // Public Key
            | 0x0042_0085 // Secret Data
            | 0x0042_0089 // Split Key
            | 0x0042_008F // Symmetric Key
    )
}

fn item_type_bit(item_type: ItemType) -> u16 {
    match item_type {
        ItemType::Structure => TYPE_STRUCTURE,
        ItemType::Integer => TYPE_INTEGER,
        ItemType::LongInteger => TYPE_LONG_INTEGER,
        ItemType::BigInteger => TYPE_BIG_INTEGER,
        ItemType::Enumeration => TYPE_ENUMERATION,
        ItemType::Boolean => TYPE_BOOLEAN,
        ItemType::TextString => TYPE_TEXT_STRING,
        ItemType::ByteString => TYPE_BYTE_STRING,
        ItemType::DateTime => TYPE_DATE_TIME,
        ItemType::Interval => TYPE_INTERVAL,
        ItemType::DateTimeExtended => TYPE_DATE_TIME_EXTENDED,
        _ => 0,
    }
}

fn push_field_issue(
    report: &mut ValidationReport,
    field: FieldDescriptor,
    source: &'static str,
    code: ValidationCode,
    prefix: &ValidationPath,
    child_index: Option<usize>,
) {
    let path = prefix.clone().field(field.name);
    let path = match child_index {
        Some(index) => path.index(index),
        None => path,
    };
    report.push(ValidationIssue::new(
        code,
        path,
        ValidationSeverity::Error,
        ValidationSourceRef::Oasis {
            document: "KMIP v2.1",
            reference: source,
        },
    ));
}

#[cfg(test)]
#[path = "../../tests/support/schema_contract_tests.rs"]
mod schema_contract_tests;
