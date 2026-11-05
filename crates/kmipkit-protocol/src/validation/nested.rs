//! Borrowed validators for reusable, explicitly identified KMIP structures.

use kmipkit_ttlv::StructureView;

use super::schema::{
    FieldDescriptor, FieldTag, NestedStructureSchema, RequestSchemaDescriptor, TYPE_ENUMERATION,
    TYPE_STRUCTURE, TYPE_TEXT_STRING, validate_structure_with_descriptor_and_prefix,
};
use super::{
    ValidationCode, ValidationIssue, ValidationPath, ValidationReport, ValidationSeverity,
    ValidationSourceRef,
};

const CONSTRAINTS_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R191",
    name: "Constraint",
    tag: FieldTag::Exact(0x0042_0169),
    types: TYPE_STRUCTURE,
    min_count: 0,
    max_count: None,
    nested_schema: Some(NestedStructureSchema::Constraint),
}];

const CONSTRAINT_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R192",
        name: "Object Types",
        tag: FieldTag::Exact(0x0042_0167),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::ObjectTypes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R193",
        name: "Object Groups",
        tag: FieldTag::Exact(0x0042_0166),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::ObjectGroups),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R194",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
];

const OBJECT_TYPES_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R195",
    name: "Object Type",
    tag: FieldTag::Exact(0x0042_0057),
    types: TYPE_ENUMERATION,
    min_count: 0,
    max_count: None,
    nested_schema: None,
}];

const OBJECT_GROUPS_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R196",
    name: "Object Group",
    tag: FieldTag::Exact(0x0042_0056),
    types: TYPE_TEXT_STRING,
    min_count: 0,
    max_count: None,
    nested_schema: None,
}];

const DEFAULTS_INFORMATION_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R197",
    name: "Object Defaults",
    tag: FieldTag::Exact(0x0042_0153),
    types: TYPE_STRUCTURE,
    min_count: 0,
    max_count: None,
    nested_schema: Some(NestedStructureSchema::ObjectDefaults),
}];

const OBJECT_DEFAULTS_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R198",
        name: "Object Type",
        tag: FieldTag::Exact(0x0042_0057),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R198",
        name: "Object Types",
        tag: FieldTag::Exact(0x0042_0167),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::ObjectTypes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R199",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R200",
        name: "Object Groups",
        tag: FieldTag::Exact(0x0042_0166),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::ObjectGroups),
    },
];

/// Validates a nested KMIP structure without taking ownership of its values.
pub(super) fn validate_nested_structure(
    structure: &StructureView<'_>,
    schema: NestedStructureSchema,
) -> ValidationReport {
    validate_nested_with_prefix(structure, schema, &ValidationPath::root())
}

pub(super) fn validate_nested_with_prefix(
    structure: &StructureView<'_>,
    schema: NestedStructureSchema,
    prefix: &ValidationPath,
) -> ValidationReport {
    let descriptor = descriptor(schema);
    let mut report = validate_structure_with_descriptor_and_prefix(structure, &descriptor, prefix);

    if schema == NestedStructureSchema::ObjectDefaults {
        validate_object_defaults_alternative(structure, prefix, &mut report);
    }

    report
}

fn descriptor(schema: NestedStructureSchema) -> RequestSchemaDescriptor {
    let (source, fields) = match schema {
        NestedStructureSchema::Constraints => ("§7.7, Table 358", CONSTRAINTS_FIELDS),
        NestedStructureSchema::Constraint => ("§7.6, Table 357", CONSTRAINT_FIELDS),
        NestedStructureSchema::ObjectTypes => ("§7.24, Table 376", OBJECT_TYPES_FIELDS),
        NestedStructureSchema::ObjectGroups => ("§7.23, Table 375", OBJECT_GROUPS_FIELDS),
        // Attribute values are tag-selected by §4; their types are already
        // checked by the attribute catalogue where a typed attribute model is
        // used. This generic Structure schema preserves future/vendor tags.
        NestedStructureSchema::Attributes => ("§5.1, Table 157", &[] as &[FieldDescriptor]),
        NestedStructureSchema::DefaultsInformation => {
            ("§7.11, Table 363", DEFAULTS_INFORMATION_FIELDS)
        }
        NestedStructureSchema::ObjectDefaults => ("§7.22, Table 374", OBJECT_DEFAULTS_FIELDS),
    };
    RequestSchemaDescriptor { source, fields }
}

fn validate_object_defaults_alternative(
    structure: &StructureView<'_>,
    prefix: &ValidationPath,
    report: &mut ValidationReport,
) {
    let has_object_type = structure
        .children()
        .iter()
        .any(|item| item.tag().raw() == 0x0042_0057);
    let has_object_types = structure
        .children()
        .iter()
        .any(|item| item.tag().raw() == 0x0042_0167);

    if has_object_type != has_object_types {
        return;
    }

    report.push(ValidationIssue::new(
        ValidationCode::new("KMIPKIT-0051-R198"),
        prefix.clone().field("Object Type"),
        ValidationSeverity::Error,
        ValidationSourceRef::Oasis {
            document: "KMIP v2.1",
            reference: "§7.22, Table 374",
        },
    ));
}
