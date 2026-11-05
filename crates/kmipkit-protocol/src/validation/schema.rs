//! Explicit request-payload schema identities and borrowed Structure checks.

use kmipkit_ttlv::{ItemType, StructureView};

use super::{
    ValidationCode, ValidationIssue, ValidationPath, ValidationReport, ValidationSeverity,
    ValidationSourceRef,
};

#[path = "schemas_generated.rs"]
mod generated;

pub use generated::RequestPayloadSchema;

const TYPE_STRUCTURE: u16 = 1 << 0;
const TYPE_INTEGER: u16 = 1 << 1;
const TYPE_LONG_INTEGER: u16 = 1 << 2;
const TYPE_BIG_INTEGER: u16 = 1 << 3;
const TYPE_ENUMERATION: u16 = 1 << 4;
const TYPE_BOOLEAN: u16 = 1 << 5;
const TYPE_TEXT_STRING: u16 = 1 << 6;
const TYPE_BYTE_STRING: u16 = 1 << 7;
const TYPE_DATE_TIME: u16 = 1 << 8;
const TYPE_INTERVAL: u16 = 1 << 9;
const TYPE_DATE_TIME_EXTENDED: u16 = 1 << 10;

/// Explicitly identifies the schema to use for generic TTLV validation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ValidationSchema {
    /// A KMIP 2.1 client Request Payload schema.
    RequestPayload(RequestPayloadSchema),
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
    rule_id: &'static str,
    name: &'static str,
    tag: FieldTag,
    types: u16,
    min_count: usize,
    max_count: Option<usize>,
}

#[derive(Clone, Copy)]
pub(super) struct RequestSchemaDescriptor {
    source: &'static str,
    fields: &'static [FieldDescriptor],
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

    let descriptor = match schema {
        ValidationSchema::RequestPayload(request_schema) => {
            let Some(descriptor) = generated::descriptor(request_schema) else {
                return unsupported_schema_report();
            };
            descriptor
        }
        ValidationSchema::Unsupported => return unsupported_schema_report(),
    };

    validate_structure_with_descriptor(structure, &descriptor)
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

fn validate_structure_with_descriptor(
    structure: &StructureView<'_>,
    descriptor: &RequestSchemaDescriptor,
) -> ValidationReport {
    let mut report = ValidationReport::default();
    let mut counts = vec![0_usize; descriptor.fields.len()];
    let mut last_known_field = 0_usize;
    let mut has_known_field = false;

    for (child_index, item) in structure.children().iter().enumerate() {
        let Some(field_index) = descriptor
            .fields
            .iter()
            .position(|field| field_matches(field, item.tag().raw()))
        else {
            continue;
        };
        let field = descriptor.fields[field_index];
        let count = &mut counts[field_index];
        *count = count.saturating_add(1);

        if has_known_field && field_index < last_known_field {
            push_field_issue(
                &mut report,
                field,
                descriptor.source,
                ValidationCode::new(field.rule_id),
                field.max_count.is_none().then_some(child_index),
            );
        }
        last_known_field = field_index;
        has_known_field = true;

        if field.max_count.is_some_and(|maximum| *count > maximum) {
            push_field_issue(
                &mut report,
                field,
                descriptor.source,
                ValidationCode::new(field.rule_id),
                Some(child_index),
            );
        }

        if field.types & item_type_bit(item.item_type()) == 0 {
            push_field_issue(
                &mut report,
                field,
                descriptor.source,
                ValidationCode::new(field.rule_id),
                (field.max_count.is_none() || *count > 1).then_some(child_index),
            );
        }
    }

    for (field, count) in descriptor.fields.iter().zip(counts) {
        if count < field.min_count {
            report.push(ValidationIssue::new(
                ValidationCode::new(field.rule_id),
                ValidationPath::root()
                    .field("Request Payload")
                    .field(field.name),
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
    child_index: Option<usize>,
) {
    let path = ValidationPath::root()
        .field("Request Payload")
        .field(field.name);
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
