//! Borrowed validators for reusable, explicitly identified KMIP structures.

use kmipkit_ttlv::StructureView;

use super::schema::{
    FieldDescriptor, FieldTag, NestedStructureSchema, RequestSchemaDescriptor, TYPE_BOOLEAN,
    TYPE_BYTE_STRING, TYPE_ENUMERATION, TYPE_INTEGER, TYPE_STRUCTURE, TYPE_TEXT_STRING,
    validate_structure_with_descriptor_and_prefix,
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

macro_rules! optional_crypto_field {
    ($rule:literal, $name:literal, $tag:literal, $types:expr) => {
        FieldDescriptor {
            rule_id: $rule,
            name: $name,
            tag: FieldTag::Exact($tag),
            types: $types,
            min_count: 0,
            max_count: Some(1),
            nested_schema: None,
        }
    };
}

const CRYPTOGRAPHIC_PARAMETERS_FIELDS: &[FieldDescriptor] = &[
    optional_crypto_field!(
        "KMIPKIT-0051-R201",
        "Block Cipher Mode",
        0x0042_0011,
        TYPE_ENUMERATION
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R202",
        "Padding Method",
        0x0042_005F,
        TYPE_ENUMERATION
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R203",
        "Hashing Algorithm",
        0x0042_0038,
        TYPE_ENUMERATION
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R204",
        "Key Role Type",
        0x0042_0083,
        TYPE_ENUMERATION
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R205",
        "Digital Signature Algorithm",
        0x0042_00AE,
        TYPE_ENUMERATION
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R206",
        "Cryptographic Algorithm",
        0x0042_0028,
        TYPE_ENUMERATION
    ),
    optional_crypto_field!("KMIPKIT-0051-R207", "Random IV", 0x0042_00C5, TYPE_BOOLEAN),
    optional_crypto_field!("KMIPKIT-0051-R208", "IV Length", 0x0042_00CD, TYPE_INTEGER),
    optional_crypto_field!("KMIPKIT-0051-R209", "Tag Length", 0x0042_00CE, TYPE_INTEGER),
    optional_crypto_field!(
        "KMIPKIT-0051-R210",
        "Fixed Field Length",
        0x0042_00CF,
        TYPE_INTEGER
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R211",
        "Invocation Field Length",
        0x0042_00D2,
        TYPE_INTEGER
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R212",
        "Counter Length",
        0x0042_00D0,
        TYPE_INTEGER
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R213",
        "Initial Counter Value",
        0x0042_00D1,
        TYPE_INTEGER
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R214",
        "Salt Length",
        0x0042_0100,
        TYPE_INTEGER
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R215",
        "Mask Generator",
        0x0042_0101,
        TYPE_ENUMERATION
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R216",
        "Mask Generator Hashing Algorithm",
        0x0042_0102,
        TYPE_ENUMERATION
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R217",
        "P Source",
        0x0042_0103,
        TYPE_BYTE_STRING
    ),
    optional_crypto_field!(
        "KMIPKIT-0051-R218",
        "Trailer Field",
        0x0042_0104,
        TYPE_INTEGER
    ),
];

const DERIVATION_PARAMETERS_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R221",
        name: "Cryptographic Parameters",
        tag: FieldTag::Exact(0x0042_002B),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CryptographicParameters),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R222",
        name: "Initialization Vector",
        tag: FieldTag::Exact(0x0042_003A),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R223",
        name: "Derivation Data",
        tag: FieldTag::Exact(0x0042_0030),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: None,
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R224",
        name: "Salt",
        tag: FieldTag::Exact(0x0042_0084),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R225",
        name: "Iteration Count",
        tag: FieldTag::Exact(0x0042_003C),
        types: TYPE_INTEGER,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const IV_LENGTH: u32 = 0x0042_00CD;
const TAG_LENGTH: u32 = 0x0042_00CE;
const CTR_MODE: u32 = 6;
const CCM_MODE: u32 = 8;
const GCM_MODE: u32 = 9;

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
    if schema == NestedStructureSchema::CryptographicParameters {
        validate_cryptographic_parameter_requirements(structure, prefix, &mut report);
    }

    report
}

pub(super) fn validate_derive_key_conditions(
    request_payload: &StructureView<'_>,
) -> ValidationReport {
    const DERIVATION_METHOD: u32 = 0x0042_0031;
    const DERIVATION_PARAMETERS: u32 = 0x0042_0032;
    const PBKDF2: u32 = 1;
    const HASH: u32 = 2;
    const SALT: u32 = 0x0042_0084;
    const ITERATION_COUNT: u32 = 0x0042_003C;
    const HASHING_ALGORITHM: u32 = 0x0042_0038;
    const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;

    let method = request_payload
        .children()
        .iter()
        .find(|item| item.tag().raw() == DERIVATION_METHOD)
        .and_then(|item| {
            item.with_value(|value| match value {
                kmipkit_ttlv::ValueView::Enumeration(method) => Some(*method),
                _ => None,
            })
        });
    let parameters = request_payload
        .children()
        .iter()
        .find(|item| item.tag().raw() == DERIVATION_PARAMETERS);
    let Some(parameters) = parameters else {
        return ValidationReport::default();
    };

    let mut report = ValidationReport::default();
    parameters.with_value(|value| {
        let kmipkit_ttlv::ValueView::Structure(parameters) = value else {
            return;
        };
        let prefix = ValidationPath::root()
            .field("Request Payload")
            .field("Derivation Parameters");

        if method == Some(PBKDF2) {
            if !has_tag(&parameters, SALT) {
                report.push(ValidationIssue::new(
                    ValidationCode::new("KMIPKIT-0051-R226"),
                    prefix.clone().field("Salt"),
                    ValidationSeverity::Error,
                    ValidationSourceRef::Oasis {
                        document: "KMIP v2.1",
                        reference: "§7.12, Table 364",
                    },
                ));
            }
            if !has_tag(&parameters, ITERATION_COUNT) {
                report.push(ValidationIssue::new(
                    ValidationCode::new("KMIPKIT-0051-R227"),
                    prefix.clone().field("Iteration Count"),
                    ValidationSeverity::Error,
                    ValidationSourceRef::Oasis {
                        document: "KMIP v2.1",
                        reference: "§7.12, Table 364",
                    },
                ));
            }
        }

        if method == Some(HASH) {
            let has_hashing_algorithm = parameters
                .children()
                .iter()
                .find(|field| field.tag().raw() == CRYPTOGRAPHIC_PARAMETERS)
                .is_some_and(|field| {
                    field.with_value(|value| match value {
                        kmipkit_ttlv::ValueView::Structure(parameters) => {
                            has_tag(&parameters, HASHING_ALGORITHM)
                        }
                        _ => true,
                    })
                });
            if !has_hashing_algorithm {
                report.push(ValidationIssue::new(
                    ValidationCode::new("KMIPKIT-0051-R228"),
                    prefix
                        .field("Cryptographic Parameters")
                        .field("Hashing Algorithm"),
                    ValidationSeverity::Error,
                    ValidationSourceRef::Oasis {
                        document: "KMIP v2.1",
                        reference: "§6.1.14, Table 205; §7.12, Table 364",
                    },
                ));
            }
        }
    });
    report
}

fn has_tag(structure: &StructureView<'_>, tag: u32) -> bool {
    structure
        .children()
        .iter()
        .any(|item| item.tag().raw() == tag)
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
        NestedStructureSchema::CryptographicParameters => {
            ("§4.16, Table 59", CRYPTOGRAPHIC_PARAMETERS_FIELDS)
        }
        NestedStructureSchema::DerivationParameters => {
            ("§7.12, Table 364", DERIVATION_PARAMETERS_FIELDS)
        }
    };
    RequestSchemaDescriptor { source, fields }
}

fn validate_cryptographic_parameter_requirements(
    structure: &StructureView<'_>,
    prefix: &ValidationPath,
    report: &mut ValidationReport,
) {
    let mut variable_iv_length_required = false;
    let mut tag_length_required = false;
    let mut has_iv_length = false;
    let mut has_tag_length = false;

    for field in structure.children() {
        match field.tag().raw() {
            BLOCK_CIPHER_MODE => {
                let mode = field.with_value(|value| match value {
                    kmipkit_ttlv::ValueView::Enumeration(raw) => Some(*raw),
                    _ => None,
                });
                match mode {
                    Some(CTR_MODE | CCM_MODE) => variable_iv_length_required = true,
                    Some(GCM_MODE) => {
                        variable_iv_length_required = true;
                        tag_length_required = true;
                    }
                    Some(_) | None => {}
                }
            }
            IV_LENGTH => has_iv_length = true,
            TAG_LENGTH => has_tag_length = true,
            _ => {}
        }
    }

    if variable_iv_length_required && !has_iv_length {
        report.push(ValidationIssue::new(
            ValidationCode::new("KMIPKIT-0051-R219"),
            prefix.clone().field("IV Length"),
            ValidationSeverity::Error,
            ValidationSourceRef::Oasis {
                document: "KMIP v2.1",
                reference: "§4.16, Table 59",
            },
        ));
    }
    if tag_length_required && !has_tag_length {
        report.push(ValidationIssue::new(
            ValidationCode::new("KMIPKIT-0051-R220"),
            prefix.clone().field("Tag Length"),
            ValidationSeverity::Error,
            ValidationSourceRef::Oasis {
                document: "KMIP v2.1",
                reference: "§4.16, Table 59",
            },
        ));
    }
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
