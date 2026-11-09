//! Borrowed validators for reusable, explicitly identified KMIP structures.

use kmipkit_ttlv::{StructureView, ValueView};

use crate::expected_key_material_item_type;

use super::schema::{
    FieldDescriptor, FieldTag, NestedStructureSchema, RequestSchemaDescriptor, TYPE_BIG_INTEGER,
    TYPE_BOOLEAN, TYPE_BYTE_STRING, TYPE_ENUMERATION, TYPE_INTEGER, TYPE_LONG_INTEGER,
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

macro_rules! optional_singleton_field {
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
    optional_singleton_field!(
        "KMIPKIT-0051-R201",
        "Block Cipher Mode",
        0x0042_0011,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R202",
        "Padding Method",
        0x0042_005F,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R203",
        "Hashing Algorithm",
        0x0042_0038,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R204",
        "Key Role Type",
        0x0042_0083,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R205",
        "Digital Signature Algorithm",
        0x0042_00AE,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R206",
        "Cryptographic Algorithm",
        0x0042_0028,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!("KMIPKIT-0051-R207", "Random IV", 0x0042_00C5, TYPE_BOOLEAN),
    optional_singleton_field!("KMIPKIT-0051-R208", "IV Length", 0x0042_00CD, TYPE_INTEGER),
    optional_singleton_field!("KMIPKIT-0051-R209", "Tag Length", 0x0042_00CE, TYPE_INTEGER),
    optional_singleton_field!(
        "KMIPKIT-0051-R210",
        "Fixed Field Length",
        0x0042_00CF,
        TYPE_INTEGER
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R211",
        "Invocation Field Length",
        0x0042_00D2,
        TYPE_INTEGER
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R212",
        "Counter Length",
        0x0042_00D0,
        TYPE_INTEGER
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R213",
        "Initial Counter Value",
        0x0042_00D1,
        TYPE_INTEGER
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R214",
        "Salt Length",
        0x0042_0100,
        TYPE_INTEGER
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R215",
        "Mask Generator",
        0x0042_0101,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R216",
        "Mask Generator Hashing Algorithm",
        0x0042_0102,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R217",
        "P Source",
        0x0042_0103,
        TYPE_BYTE_STRING
    ),
    optional_singleton_field!(
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

const KEY_WRAPPING_SPECIFICATION_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R229",
        name: "Wrapping Method",
        tag: FieldTag::Exact(0x0042_009E),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R230",
        name: "Encryption Key Information",
        tag: FieldTag::Exact(0x0042_0036),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::EncryptionKeyInformation),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R231",
        name: "MAC/Signature Key Information",
        tag: FieldTag::Exact(0x0042_004E),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::MacSignatureKeyInformation),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R232",
        name: "Attribute Name",
        tag: FieldTag::Exact(0x0042_000A),
        types: TYPE_TEXT_STRING,
        min_count: 0,
        max_count: None,
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R233",
        name: "Encoding Option",
        tag: FieldTag::Exact(0x0042_00A3),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

const KEY_INFORMATION_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R234",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_TEXT_STRING,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R235",
        name: "Cryptographic Parameters",
        tag: FieldTag::Exact(0x0042_002B),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CryptographicParameters),
    },
];

macro_rules! optional_nested_field {
    ($rule:literal, $name:literal, $tag:literal, $schema:expr) => {
        FieldDescriptor {
            rule_id: $rule,
            name: $name,
            tag: FieldTag::Exact($tag),
            types: TYPE_STRUCTURE,
            min_count: 0,
            max_count: Some(1),
            nested_schema: Some($schema),
        }
    };
}

const KEY_VALUE_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R278",
        name: "Key Material",
        tag: FieldTag::Exact(0x0042_0043),
        types: TYPE_BYTE_STRING | TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R279",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
];

const KEY_WRAPPING_DATA_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R271",
        name: "Wrapping Method",
        tag: FieldTag::Exact(0x0042_009E),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    optional_nested_field!(
        "KMIPKIT-0051-R272",
        "Encryption Key Information",
        0x0042_0036,
        NestedStructureSchema::EncryptionKeyInformation
    ),
    optional_nested_field!(
        "KMIPKIT-0051-R273",
        "MAC/Signature Key Information",
        0x0042_004E,
        NestedStructureSchema::MacSignatureKeyInformation
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R274",
        "MAC/Signature",
        0x0042_004D,
        TYPE_BYTE_STRING
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R275",
        "IV/Counter/Nonce",
        0x0042_003D,
        TYPE_BYTE_STRING
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R276",
        "Encoding Option",
        0x0042_00A3,
        TYPE_ENUMERATION
    ),
];

const RIGHTS_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R237",
    name: "Right",
    tag: FieldTag::Exact(0x0042_0150),
    types: TYPE_STRUCTURE,
    min_count: 0,
    max_count: None,
    nested_schema: Some(NestedStructureSchema::Right),
}];

const RIGHT_FIELDS: &[FieldDescriptor] = &[
    optional_nested_field!(
        "KMIPKIT-0051-R238",
        "Usage Limits",
        0x0042_0095,
        NestedStructureSchema::UsageLimits
    ),
    optional_nested_field!(
        "KMIPKIT-0051-R239",
        "Operations",
        0x0042_014F,
        NestedStructureSchema::Operations
    ),
    optional_nested_field!(
        "KMIPKIT-0051-R240",
        "Objects",
        0x0042_014E,
        NestedStructureSchema::Objects
    ),
    optional_nested_field!(
        "KMIPKIT-0051-R241",
        "Object Groups",
        0x0042_0166,
        NestedStructureSchema::ObjectGroups
    ),
];

const USAGE_LIMITS_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R242",
        name: "Usage Limits Total",
        tag: FieldTag::Exact(0x0042_0097),
        types: TYPE_LONG_INTEGER,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R243",
        name: "Usage Limits Count",
        tag: FieldTag::Exact(0x0042_0096),
        types: TYPE_LONG_INTEGER,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R244",
        name: "Usage Limits Unit",
        tag: FieldTag::Exact(0x0042_0098),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
];

const OBJECTS_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R245",
    name: "Unique Identifier",
    tag: FieldTag::Exact(0x0042_0094),
    types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
    min_count: 0,
    max_count: None,
    nested_schema: None,
}];

const OPERATIONS_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R246",
    name: "Operation",
    tag: FieldTag::Exact(0x0042_005C),
    types: TYPE_ENUMERATION,
    min_count: 0,
    max_count: None,
    nested_schema: None,
}];

macro_rules! required_singleton_field {
    ($rule:literal, $name:literal, $tag:literal, $types:expr) => {
        FieldDescriptor {
            rule_id: $rule,
            name: $name,
            tag: FieldTag::Exact($tag),
            types: $types,
            min_count: 1,
            max_count: Some(1),
            nested_schema: None,
        }
    };
}

const CERTIFICATE_FIELDS: &[FieldDescriptor] = &[
    required_singleton_field!(
        "KMIPKIT-0051-R249",
        "Certificate Type",
        0x0042_001D,
        TYPE_ENUMERATION
    ),
    required_singleton_field!(
        "KMIPKIT-0051-R250",
        "Certificate Value",
        0x0042_001E,
        TYPE_BYTE_STRING
    ),
];

const CERTIFICATE_REQUEST_FIELDS: &[FieldDescriptor] = &[
    required_singleton_field!(
        "KMIPKIT-0051-R251",
        "Certificate Request Type",
        0x0042_0019,
        TYPE_ENUMERATION
    ),
    required_singleton_field!(
        "KMIPKIT-0051-R252",
        "Certificate Request Value",
        0x0042_0140,
        TYPE_BYTE_STRING
    ),
];

const ASYNC_CORRELATION_VALUES_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R259",
    name: "Asynchronous Correlation Value",
    tag: FieldTag::Exact(0x0042_0006),
    types: TYPE_BYTE_STRING,
    min_count: 0,
    max_count: None,
    nested_schema: None,
}];

const TICKET_FIELDS: &[FieldDescriptor] = &[
    required_singleton_field!(
        "KMIPKIT-0051-R253",
        "Ticket Type",
        0x0042_014A,
        TYPE_ENUMERATION
    ),
    required_singleton_field!(
        "KMIPKIT-0051-R254",
        "Ticket Value",
        0x0042_014B,
        TYPE_BYTE_STRING
    ),
];

const PROTOCOL_VERSION_FIELDS: &[FieldDescriptor] = &[
    required_singleton_field!(
        "KMIPKIT-0051-R255",
        "Protocol Version Major",
        0x0042_006A,
        TYPE_INTEGER
    ),
    required_singleton_field!(
        "KMIPKIT-0051-R256",
        "Protocol Version Minor",
        0x0042_006B,
        TYPE_INTEGER
    ),
];

const REVOCATION_REASON_FIELDS: &[FieldDescriptor] = &[
    required_singleton_field!(
        "KMIPKIT-0051-R257",
        "Revocation Reason Code",
        0x0042_0082,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R258",
        "Revocation Message",
        0x0042_0080,
        TYPE_TEXT_STRING
    ),
];

const PKCS11_INPUT_PARAMETERS_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R261",
    name: "PKCS#11 Input Parameters",
    tag: FieldTag::Exact(0x0042_015B),
    types: TYPE_BYTE_STRING,
    min_count: 0,
    max_count: Some(1),
    nested_schema: None,
}];

const PKCS11_INTERFACE_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R260",
    name: "PKCS#11 Interface",
    tag: FieldTag::Exact(0x0042_0159),
    types: TYPE_TEXT_STRING,
    min_count: 0,
    max_count: Some(1),
    nested_schema: None,
}];

const KEY_BLOCK_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R262",
        name: "Key Format Type",
        tag: FieldTag::Exact(0x0042_0042),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    optional_singleton_field!(
        "KMIPKIT-0051-R263",
        "Key Compression Type",
        0x0042_0041,
        TYPE_ENUMERATION
    ),
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R264",
        name: "Key Value",
        tag: FieldTag::Exact(0x0042_0045),
        types: TYPE_BYTE_STRING | TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::KeyValue),
    },
    optional_singleton_field!(
        "KMIPKIT-0051-R265",
        "Cryptographic Algorithm",
        0x0042_0028,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R266",
        "Cryptographic Length",
        0x0042_002A,
        TYPE_INTEGER
    ),
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R267",
        name: "Key Wrapping Data",
        tag: FieldTag::Exact(0x0042_0046),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::KeyWrappingData),
    },
];

const KEY_BEARING_OBJECT_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R268",
    name: "Key Block",
    tag: FieldTag::Exact(0x0042_0040),
    types: TYPE_STRUCTURE,
    min_count: 1,
    max_count: Some(1),
    nested_schema: Some(NestedStructureSchema::KeyBlock),
}];

const PGP_KEY_OBJECT_FIELDS: &[FieldDescriptor] = &[
    required_singleton_field!(
        "KMIPKIT-0051-R285",
        "PGP Key Version",
        0x0042_00BE,
        TYPE_INTEGER
    ),
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R268",
        name: "Key Block",
        tag: FieldTag::Exact(0x0042_0040),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::KeyBlock),
    },
];

const SECRET_DATA_OBJECT_FIELDS: &[FieldDescriptor] = &[
    required_singleton_field!(
        "KMIPKIT-0051-R286",
        "Secret Data Type",
        0x0042_0086,
        TYPE_ENUMERATION
    ),
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R268",
        name: "Key Block",
        tag: FieldTag::Exact(0x0042_0040),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::KeyBlock),
    },
];

const SPLIT_KEY_OBJECT_FIELDS: &[FieldDescriptor] = &[
    required_singleton_field!(
        "KMIPKIT-0051-R287",
        "Split Key Parts",
        0x0042_008B,
        TYPE_INTEGER
    ),
    required_singleton_field!(
        "KMIPKIT-0051-R288",
        "Key Part Identifier",
        0x0042_0044,
        TYPE_INTEGER
    ),
    required_singleton_field!(
        "KMIPKIT-0051-R289",
        "Split Key Threshold",
        0x0042_008C,
        TYPE_INTEGER
    ),
    required_singleton_field!(
        "KMIPKIT-0051-R290",
        "Split Key Method",
        0x0042_008A,
        TYPE_ENUMERATION
    ),
    optional_singleton_field!(
        "KMIPKIT-0051-R291",
        "Prime Field Size",
        0x0042_0062,
        TYPE_BIG_INTEGER
    ),
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R268",
        name: "Key Block",
        tag: FieldTag::Exact(0x0042_0040),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::KeyBlock),
    },
];

const OPAQUE_OBJECT_FIELDS: &[FieldDescriptor] = &[
    required_singleton_field!(
        "KMIPKIT-0051-R269",
        "Opaque Data Type",
        0x0042_0059,
        TYPE_ENUMERATION
    ),
    required_singleton_field!(
        "KMIPKIT-0051-R270",
        "Opaque Data Value",
        0x0042_005A,
        TYPE_BYTE_STRING
    ),
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
    let mut report = match schema {
        NestedStructureSchema::SplitKey => validate_split_key_object(structure, prefix),
        _ => {
            let descriptor = descriptor(schema);
            validate_structure_with_descriptor_and_prefix(structure, &descriptor, prefix)
        }
    };

    if matches!(
        schema,
        NestedStructureSchema::CurrentAttribute | NestedStructureSchema::NewAttribute
    ) {
        validate_attribute_wrapper(structure, schema, prefix, &mut report);
    }
    if schema == NestedStructureSchema::ObjectDefaults {
        validate_object_defaults_alternative(structure, prefix, &mut report);
    }
    if schema == NestedStructureSchema::CryptographicParameters {
        validate_cryptographic_parameter_requirements(structure, prefix, &mut report);
    }
    if schema == NestedStructureSchema::KeyBlock {
        report.append(validate_key_block_requirements(structure, prefix));
        report.append(validate_key_format_material_type(structure, prefix));
    }
    if schema == NestedStructureSchema::KeyWrappingSpecification {
        validate_one_key_information_alternative(
            structure,
            prefix,
            &mut report,
            "KMIPKIT-0051-R236",
            "§7.18, Table 370",
        );
    }
    if schema == NestedStructureSchema::KeyWrappingData {
        validate_one_key_information_alternative(
            structure,
            prefix,
            &mut report,
            "KMIPKIT-0051-R277",
            "§3.3, Table 15",
        );
    }

    report
}

fn validate_key_block_requirements(
    structure: &StructureView<'_>,
    prefix: &ValidationPath,
) -> ValidationReport {
    const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
    const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;

    let has_algorithm = has_tag(structure, CRYPTOGRAPHIC_ALGORITHM);
    let has_length = has_tag(structure, CRYPTOGRAPHIC_LENGTH);
    let mut report = ValidationReport::default();

    let (missing_field, rule_id) = match (has_algorithm, has_length) {
        (true, false) => (Some("Cryptographic Length"), "KMIPKIT-0051-R266"),
        (false, true) => (Some("Cryptographic Algorithm"), "KMIPKIT-0051-R265"),
        _ => (None, ""),
    };
    if let Some(field) = missing_field {
        report.push(ValidationIssue::new(
            ValidationCode::new(rule_id),
            prefix.clone().field(field),
            ValidationSeverity::Error,
            ValidationSourceRef::Oasis {
                document: "KMIP v2.1",
                reference: "§3.1, Table 12",
            },
        ));
    }

    report
}

fn validate_key_format_material_type(
    structure: &StructureView<'_>,
    prefix: &ValidationPath,
) -> ValidationReport {
    const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
    const KEY_VALUE: u32 = 0x0042_0045;
    const KEY_MATERIAL: u32 = 0x0042_0043;

    let key_format_type = structure
        .children()
        .iter()
        .find(|item| item.tag().raw() == KEY_FORMAT_TYPE)
        .and_then(|item| {
            item.with_value(|value| match value {
                kmipkit_ttlv::ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            })
        });
    let Some(expected_material_type) = key_format_type.and_then(expected_key_material_item_type)
    else {
        return ValidationReport::default();
    };

    let actual_material_type = structure
        .children()
        .iter()
        .find(|item| item.tag().raw() == KEY_VALUE)
        .and_then(|key_value| {
            key_value.with_value(|value| {
                let kmipkit_ttlv::ValueView::Structure(key_value) = value else {
                    return None;
                };
                key_value
                    .children()
                    .iter()
                    .find(|item| item.tag().raw() == KEY_MATERIAL)
                    .map(kmipkit_ttlv::Item::item_type)
            })
        });

    let Some(actual_material_type) = actual_material_type else {
        return ValidationReport::default();
    };
    if actual_material_type == expected_material_type {
        return ValidationReport::default();
    }

    let mut report = ValidationReport::default();
    report.push(ValidationIssue::new(
        ValidationCode::new("KMIPKIT-0051-R280"),
        prefix.clone().field("Key Value").field("Key Material"),
        ValidationSeverity::Error,
        ValidationSourceRef::Oasis {
            document: "KMIP v2.1",
            reference: "§3.2, Table 13; §11.25, Table 461",
        },
    ));
    report
}

pub(super) fn validate_managed_object(
    structure: &StructureView<'_>,
    object_tag: u32,
    prefix: &ValidationPath,
) -> ValidationReport {
    match object_tag {
        0x0042_0013 => {
            validate_nested_with_prefix(structure, NestedStructureSchema::Certificate, prefix)
        }
        0x0042_0018 => validate_nested_with_prefix(
            structure,
            NestedStructureSchema::CertificateRequest,
            prefix,
        ),
        0x0042_005B => {
            validate_nested_with_prefix(structure, NestedStructureSchema::OpaqueObject, prefix)
        }
        0x0042_00BD => validate_managed_object_fields(
            structure,
            "§2.4, Table 5",
            PGP_KEY_OBJECT_FIELDS,
            prefix,
        ),
        0x0042_0064 => validate_key_bearing_object(structure, "§2.5, Table 6", prefix),
        0x0042_006D => validate_key_bearing_object(structure, "§2.6, Table 7", prefix),
        0x0042_0085 => validate_managed_object_fields(
            structure,
            "§2.7, Table 8",
            SECRET_DATA_OBJECT_FIELDS,
            prefix,
        ),
        0x0042_0089 => validate_split_key_object(structure, prefix),
        0x0042_008F => validate_key_bearing_object(structure, "§2.9, Table 10", prefix),
        _ => ValidationReport::default(),
    }
}

fn validate_key_bearing_object(
    structure: &StructureView<'_>,
    source: &'static str,
    prefix: &ValidationPath,
) -> ValidationReport {
    validate_managed_object_fields(structure, source, KEY_BEARING_OBJECT_FIELDS, prefix)
}

fn validate_managed_object_fields(
    structure: &StructureView<'_>,
    source: &'static str,
    fields: &'static [FieldDescriptor],
    prefix: &ValidationPath,
) -> ValidationReport {
    let descriptor = RequestSchemaDescriptor { source, fields };
    validate_structure_with_descriptor_and_prefix(structure, &descriptor, prefix)
}

fn validate_split_key_object(
    structure: &StructureView<'_>,
    prefix: &ValidationPath,
) -> ValidationReport {
    const SPLIT_KEY_METHOD: u32 = 0x0042_008A;
    const SPLIT_KEY_PARTS: u32 = 0x0042_008B;
    const KEY_PART_IDENTIFIER: u32 = 0x0042_0044;
    const PRIME_FIELD_SIZE: u32 = 0x0042_0062;
    const POLYNOMIAL_SHARING_PRIME_FIELD: u32 = 3;

    let mut report =
        validate_managed_object_fields(structure, "§2.8, Table 9", SPLIT_KEY_OBJECT_FIELDS, prefix);
    let field_integer = |tag: u32| {
        structure
            .children()
            .iter()
            .find(|item| item.tag().raw() == tag)
            .and_then(|item| {
                item.with_value(|value| match value {
                    ValueView::Integer(raw) => Some(*raw),
                    _ => None,
                })
            })
    };
    if let (Some(parts), Some(part_id)) = (
        field_integer(SPLIT_KEY_PARTS),
        field_integer(KEY_PART_IDENTIFIER),
    ) && (part_id < 1 || part_id > parts)
    {
        report.push(ValidationIssue::new(
            ValidationCode::new("KMIPKIT-0051-R293"),
            prefix.clone().field("Key Part Identifier"),
            ValidationSeverity::Error,
            ValidationSourceRef::Oasis {
                document: "KMIP v2.1",
                reference: "§2.8",
            },
        ));
    }
    let uses_prime_field = structure
        .children()
        .iter()
        .find(|item| item.tag().raw() == SPLIT_KEY_METHOD)
        .and_then(|item| {
            item.with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(*raw == POLYNOMIAL_SHARING_PRIME_FIELD),
                _ => None,
            })
        })
        .unwrap_or(false);
    if uses_prime_field
        && !structure
            .children()
            .iter()
            .any(|item| item.tag().raw() == PRIME_FIELD_SIZE)
    {
        report.push(ValidationIssue::new(
            ValidationCode::new("KMIPKIT-0051-R292"),
            prefix.clone().field("Prime Field Size"),
            ValidationSeverity::Error,
            ValidationSourceRef::Oasis {
                document: "KMIP v2.1",
                reference: "§2.8, Table 9",
            },
        ));
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
        NestedStructureSchema::KeyWrappingSpecification => {
            ("§7.18, Table 370", KEY_WRAPPING_SPECIFICATION_FIELDS)
        }
        NestedStructureSchema::EncryptionKeyInformation => {
            ("§3.3, Table 16", KEY_INFORMATION_FIELDS)
        }
        NestedStructureSchema::MacSignatureKeyInformation => {
            ("§3.3, Table 17", KEY_INFORMATION_FIELDS)
        }
        NestedStructureSchema::Rights => ("§7.35, Table 387", RIGHTS_FIELDS),
        NestedStructureSchema::Right => ("§7.34, Table 386", RIGHT_FIELDS),
        NestedStructureSchema::UsageLimits => ("§7.40, Table 392", USAGE_LIMITS_FIELDS),
        NestedStructureSchema::Objects => ("§7.21, Table 373", OBJECTS_FIELDS),
        NestedStructureSchema::Operations => ("§7.25, Table 377", OPERATIONS_FIELDS),
        NestedStructureSchema::CurrentAttribute => ("§5.6, Table 162", &[] as &[FieldDescriptor]),
        NestedStructureSchema::NewAttribute => ("§5.7, Table 163", &[] as &[FieldDescriptor]),
        NestedStructureSchema::Certificate => ("§2.1, Table 2", CERTIFICATE_FIELDS),
        NestedStructureSchema::CertificateRequest => ("§2.2, Table 3", CERTIFICATE_REQUEST_FIELDS),
        NestedStructureSchema::AsynchronousCorrelationValues => {
            ("§7.1, Table 352", ASYNC_CORRELATION_VALUES_FIELDS)
        }
        NestedStructureSchema::Ticket => ("§7.39, Table 391", TICKET_FIELDS),
        NestedStructureSchema::ProtocolVersion => ("§9.16, Table 421", PROTOCOL_VERSION_FIELDS),
        NestedStructureSchema::RevocationReason => ("§4.47, Table 123", REVOCATION_REASON_FIELDS),
        NestedStructureSchema::Pkcs11InputParameters => {
            ("§7.27, Table 379", PKCS11_INPUT_PARAMETERS_FIELDS)
        }
        NestedStructureSchema::Pkcs11Interface => ("§7.28, Table 380", PKCS11_INTERFACE_FIELDS),
        NestedStructureSchema::KeyBlock => ("§3.1, Table 12", KEY_BLOCK_FIELDS),
        NestedStructureSchema::KeyValue => ("§3.2, Table 13", KEY_VALUE_FIELDS),
        NestedStructureSchema::KeyWrappingData => ("§3.3, Table 14", KEY_WRAPPING_DATA_FIELDS),
        NestedStructureSchema::OpaqueObject => ("§2.3, Table 4", OPAQUE_OBJECT_FIELDS),
        NestedStructureSchema::PgpKey => ("§2.4, Table 5", PGP_KEY_OBJECT_FIELDS),
        NestedStructureSchema::SecretData => ("§2.7, Table 8", SECRET_DATA_OBJECT_FIELDS),
        NestedStructureSchema::SplitKey => ("§2.8, Table 9", SPLIT_KEY_OBJECT_FIELDS),
    };
    RequestSchemaDescriptor { source, fields }
}

fn validate_attribute_wrapper(
    structure: &StructureView<'_>,
    schema: NestedStructureSchema,
    prefix: &ValidationPath,
    report: &mut ValidationReport,
) {
    let (rule_id, source) = match schema {
        NestedStructureSchema::CurrentAttribute => ("KMIPKIT-0051-R248", "§5.6, Table 162"),
        NestedStructureSchema::NewAttribute => ("KMIPKIT-0051-R247", "§5.7, Table 163"),
        _ => return,
    };
    if structure.children().len() == 1 {
        return;
    }

    report.push(ValidationIssue::new(
        ValidationCode::new(rule_id),
        prefix.clone().field("Attribute"),
        ValidationSeverity::Error,
        ValidationSourceRef::Oasis {
            document: "KMIP v2.1",
            reference: source,
        },
    ));
}

fn validate_one_key_information_alternative(
    structure: &StructureView<'_>,
    prefix: &ValidationPath,
    report: &mut ValidationReport,
    rule_id: &'static str,
    source: &'static str,
) {
    const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;
    const MAC_SIGNATURE_KEY_INFORMATION: u32 = 0x0042_004E;

    if has_tag(structure, ENCRYPTION_KEY_INFORMATION)
        || has_tag(structure, MAC_SIGNATURE_KEY_INFORMATION)
    {
        return;
    }

    report.push(ValidationIssue::new(
        ValidationCode::new(rule_id),
        prefix.clone().field("Encryption Key Information"),
        ValidationSeverity::Error,
        ValidationSourceRef::Oasis {
            document: "KMIP v2.1",
            reference: source,
        },
    ));
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
