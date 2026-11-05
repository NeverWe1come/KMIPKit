//! Generated operation Request Payload schema identities and descriptors.
//! Regenerate with `python tools/validation_schema/generate.py`.

use super::{FieldDescriptor, FieldTag, NestedStructureSchema, RequestSchemaDescriptor};
use super::{
    TYPE_BIG_INTEGER, TYPE_BOOLEAN, TYPE_BYTE_STRING, TYPE_DATE_TIME, TYPE_DATE_TIME_EXTENDED,
    TYPE_ENUMERATION, TYPE_INTEGER, TYPE_INTERVAL, TYPE_LONG_INTEGER, TYPE_STRUCTURE,
    TYPE_TEXT_STRING,
};

/// Explicit identity for a supported KMIP v2.1 client Request Payload.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RequestPayloadSchema {
    /// The `Activate` Request Payload schema.
    Activate,
    /// The `AddAttribute` Request Payload schema.
    AddAttribute,
    /// The `AdjustAttribute` Request Payload schema.
    AdjustAttribute,
    /// The `Archive` Request Payload schema.
    Archive,
    /// The `Cancel` Request Payload schema.
    Cancel,
    /// The `Certify` Request Payload schema.
    Certify,
    /// The `Check` Request Payload schema.
    Check,
    /// The `Create` Request Payload schema.
    Create,
    /// The `CreateKeyPair` Request Payload schema.
    CreateKeyPair,
    /// The `CreateSplitKey` Request Payload schema.
    CreateSplitKey,
    /// The `Decrypt` Request Payload schema.
    Decrypt,
    /// The `DelegatedLogin` Request Payload schema.
    DelegatedLogin,
    /// The `DeleteAttribute` Request Payload schema.
    DeleteAttribute,
    /// The `DeriveKey` Request Payload schema.
    DeriveKey,
    /// The `Destroy` Request Payload schema.
    Destroy,
    /// The `DiscoverVersions` Request Payload schema.
    DiscoverVersions,
    /// The `Encrypt` Request Payload schema.
    Encrypt,
    /// The `Export` Request Payload schema.
    Export,
    /// The `Get` Request Payload schema.
    Get,
    /// The `GetAttributeList` Request Payload schema.
    GetAttributeList,
    /// The `GetAttributes` Request Payload schema.
    GetAttributes,
    /// The `GetConstraints` Request Payload schema.
    GetConstraints,
    /// The `GetUsageAllocation` Request Payload schema.
    GetUsageAllocation,
    /// The `Hash` Request Payload schema.
    Hash,
    /// The `Import` Request Payload schema.
    Import,
    /// The `Interop` Request Payload schema.
    Interop,
    /// The `JoinSplitKey` Request Payload schema.
    JoinSplitKey,
    /// The `Locate` Request Payload schema.
    Locate,
    /// The `Log` Request Payload schema.
    Log,
    /// The `Login` Request Payload schema.
    Login,
    /// The `Logout` Request Payload schema.
    Logout,
    /// The `Mac` Request Payload schema.
    Mac,
    /// The `MacVerify` Request Payload schema.
    MacVerify,
    /// The `ModifyAttribute` Request Payload schema.
    ModifyAttribute,
    /// The `ObtainLease` Request Payload schema.
    ObtainLease,
    /// The `Ping` Request Payload schema.
    Ping,
    /// The `Pkcs11` Request Payload schema.
    Pkcs11,
    /// The `Poll` Request Payload schema.
    Poll,
    /// The `Process` Request Payload schema.
    Process,
    /// The `Query` Request Payload schema.
    Query,
    /// The `QueryAsyncRequests` Request Payload schema.
    QueryAsyncRequests,
    /// The `ReProvision` Request Payload schema.
    ReProvision,
    /// The `Recertify` Request Payload schema.
    Recertify,
    /// The `Recover` Request Payload schema.
    Recover,
    /// The `Register` Request Payload schema.
    Register,
    /// The `Rekey` Request Payload schema.
    Rekey,
    /// The `RekeyKeyPair` Request Payload schema.
    RekeyKeyPair,
    /// The `Revoke` Request Payload schema.
    Revoke,
    /// The `RngRetrieve` Request Payload schema.
    RngRetrieve,
    /// The `RngSeed` Request Payload schema.
    RngSeed,
    /// The `SetAttribute` Request Payload schema.
    SetAttribute,
    /// The `SetConstraints` Request Payload schema.
    SetConstraints,
    /// The `SetDefaults` Request Payload schema.
    SetDefaults,
    /// Client-initiated operation deferred from `KMIPKit` 1.0 to 1.1.
    SetEndpointRole,
    /// The `Sign` Request Payload schema.
    Sign,
    /// The `SignatureVerify` Request Payload schema.
    SignatureVerify,
    /// The `Validate` Request Payload schema.
    Validate,
}

pub(super) fn descriptor(schema: RequestPayloadSchema) -> Option<RequestSchemaDescriptor> {
    match schema {
        RequestPayloadSchema::Activate
        | RequestPayloadSchema::AddAttribute
        | RequestPayloadSchema::AdjustAttribute
        | RequestPayloadSchema::Archive
        | RequestPayloadSchema::Cancel
        | RequestPayloadSchema::Certify
        | RequestPayloadSchema::Check
        | RequestPayloadSchema::Create
        | RequestPayloadSchema::CreateKeyPair
        | RequestPayloadSchema::CreateSplitKey
        | RequestPayloadSchema::Decrypt
        | RequestPayloadSchema::DelegatedLogin => descriptor_group_0(schema),
        RequestPayloadSchema::DeleteAttribute
        | RequestPayloadSchema::DeriveKey
        | RequestPayloadSchema::Destroy
        | RequestPayloadSchema::DiscoverVersions
        | RequestPayloadSchema::Encrypt
        | RequestPayloadSchema::Export
        | RequestPayloadSchema::Get
        | RequestPayloadSchema::GetAttributeList
        | RequestPayloadSchema::GetAttributes
        | RequestPayloadSchema::GetConstraints
        | RequestPayloadSchema::GetUsageAllocation
        | RequestPayloadSchema::Hash => descriptor_group_1(schema),
        RequestPayloadSchema::Import
        | RequestPayloadSchema::Interop
        | RequestPayloadSchema::JoinSplitKey
        | RequestPayloadSchema::Locate
        | RequestPayloadSchema::Log
        | RequestPayloadSchema::Login
        | RequestPayloadSchema::Logout
        | RequestPayloadSchema::Mac
        | RequestPayloadSchema::MacVerify
        | RequestPayloadSchema::ModifyAttribute
        | RequestPayloadSchema::ObtainLease
        | RequestPayloadSchema::Ping => descriptor_group_2(schema),
        RequestPayloadSchema::Pkcs11
        | RequestPayloadSchema::Poll
        | RequestPayloadSchema::Process
        | RequestPayloadSchema::Query
        | RequestPayloadSchema::QueryAsyncRequests
        | RequestPayloadSchema::ReProvision
        | RequestPayloadSchema::Recertify
        | RequestPayloadSchema::Recover
        | RequestPayloadSchema::Register
        | RequestPayloadSchema::Rekey
        | RequestPayloadSchema::RekeyKeyPair
        | RequestPayloadSchema::Revoke => descriptor_group_3(schema),
        RequestPayloadSchema::RngRetrieve
        | RequestPayloadSchema::RngSeed
        | RequestPayloadSchema::SetAttribute
        | RequestPayloadSchema::SetConstraints
        | RequestPayloadSchema::SetDefaults
        | RequestPayloadSchema::Sign
        | RequestPayloadSchema::SignatureVerify
        | RequestPayloadSchema::Validate => descriptor_group_4(schema),
        RequestPayloadSchema::SetEndpointRole => None,
    }
}

fn descriptor_group_0(schema: RequestPayloadSchema) -> Option<RequestSchemaDescriptor> {
    match schema {
        RequestPayloadSchema::Activate => Some(RequestSchemaDescriptor {
            source: "§6.1.1, Table 164",
            fields: ACTIVATE_FIELDS,
        }),
        RequestPayloadSchema::AddAttribute => Some(RequestSchemaDescriptor {
            source: "§6.1.2, Table 167",
            fields: ADDATTRIBUTE_FIELDS,
        }),
        RequestPayloadSchema::AdjustAttribute => Some(RequestSchemaDescriptor {
            source: "§6.1.3, Table 170",
            fields: ADJUSTATTRIBUTE_FIELDS,
        }),
        RequestPayloadSchema::Archive => Some(RequestSchemaDescriptor {
            source: "§6.1.4, Table 173",
            fields: ARCHIVE_FIELDS,
        }),
        RequestPayloadSchema::Cancel => Some(RequestSchemaDescriptor {
            source: "§6.1.5, Table 176",
            fields: CANCEL_FIELDS,
        }),
        RequestPayloadSchema::Certify => Some(RequestSchemaDescriptor {
            source: "§6.1.6, Table 179",
            fields: CERTIFY_FIELDS,
        }),
        RequestPayloadSchema::Check => Some(RequestSchemaDescriptor {
            source: "§6.1.7, Table 183",
            fields: CHECK_FIELDS,
        }),
        RequestPayloadSchema::Create => Some(RequestSchemaDescriptor {
            source: "§6.1.8, Table 186",
            fields: CREATE_FIELDS,
        }),
        RequestPayloadSchema::CreateKeyPair => Some(RequestSchemaDescriptor {
            source: "§6.1.9, Table 189",
            fields: CREATEKEYPAIR_FIELDS,
        }),
        RequestPayloadSchema::CreateSplitKey => Some(RequestSchemaDescriptor {
            source: "§6.1.10, Table 193",
            fields: CREATESPLITKEY_FIELDS,
        }),
        RequestPayloadSchema::Decrypt => Some(RequestSchemaDescriptor {
            source: "§6.1.11, Table 196",
            fields: DECRYPT_FIELDS,
        }),
        RequestPayloadSchema::DelegatedLogin => Some(RequestSchemaDescriptor {
            source: "§6.1.12, Table 199",
            fields: DELEGATEDLOGIN_FIELDS,
        }),
        _ => None,
    }
}

fn descriptor_group_1(schema: RequestPayloadSchema) -> Option<RequestSchemaDescriptor> {
    match schema {
        RequestPayloadSchema::DeleteAttribute => Some(RequestSchemaDescriptor {
            source: "§6.1.13, Table 202",
            fields: DELETEATTRIBUTE_FIELDS,
        }),
        RequestPayloadSchema::DeriveKey => Some(RequestSchemaDescriptor {
            source: "§6.1.14, Table 205",
            fields: DERIVEKEY_FIELDS,
        }),
        RequestPayloadSchema::Destroy => Some(RequestSchemaDescriptor {
            source: "§6.1.15, Table 208",
            fields: DESTROY_FIELDS,
        }),
        RequestPayloadSchema::DiscoverVersions => Some(RequestSchemaDescriptor {
            source: "§6.1.16, Table 211",
            fields: DISCOVERVERSIONS_FIELDS,
        }),
        RequestPayloadSchema::Encrypt => Some(RequestSchemaDescriptor {
            source: "§6.1.17, Table 214",
            fields: ENCRYPT_FIELDS,
        }),
        RequestPayloadSchema::Export => Some(RequestSchemaDescriptor {
            source: "§6.1.18, Table 217",
            fields: EXPORT_FIELDS,
        }),
        RequestPayloadSchema::Get => Some(RequestSchemaDescriptor {
            source: "§6.1.19, Table 220",
            fields: GET_FIELDS,
        }),
        RequestPayloadSchema::GetAttributeList => Some(RequestSchemaDescriptor {
            source: "§6.1.21, Table 226",
            fields: GETATTRIBUTELIST_FIELDS,
        }),
        RequestPayloadSchema::GetAttributes => Some(RequestSchemaDescriptor {
            source: "§6.1.20, Table 223",
            fields: GETATTRIBUTES_FIELDS,
        }),
        RequestPayloadSchema::GetConstraints => Some(RequestSchemaDescriptor {
            source: "§6.1.22, Table 229",
            fields: GETCONSTRAINTS_FIELDS,
        }),
        RequestPayloadSchema::GetUsageAllocation => Some(RequestSchemaDescriptor {
            source: "§6.1.23, Table 232",
            fields: GETUSAGEALLOCATION_FIELDS,
        }),
        RequestPayloadSchema::Hash => Some(RequestSchemaDescriptor {
            source: "§6.1.24, Table 235",
            fields: HASH_FIELDS,
        }),
        _ => None,
    }
}

fn descriptor_group_2(schema: RequestPayloadSchema) -> Option<RequestSchemaDescriptor> {
    match schema {
        RequestPayloadSchema::Import => Some(RequestSchemaDescriptor {
            source: "§6.1.25, Table 238",
            fields: IMPORT_FIELDS,
        }),
        RequestPayloadSchema::Interop => Some(RequestSchemaDescriptor {
            source: "§6.1.26, Table 241",
            fields: INTEROP_FIELDS,
        }),
        RequestPayloadSchema::JoinSplitKey => Some(RequestSchemaDescriptor {
            source: "§6.1.27, Table 244",
            fields: JOINSPLITKEY_FIELDS,
        }),
        RequestPayloadSchema::Locate => Some(RequestSchemaDescriptor {
            source: "§6.1.28, Table 247",
            fields: LOCATE_FIELDS,
        }),
        RequestPayloadSchema::Log => Some(RequestSchemaDescriptor {
            source: "§6.1.29, Table 250",
            fields: LOG_FIELDS,
        }),
        RequestPayloadSchema::Login => Some(RequestSchemaDescriptor {
            source: "§6.1.30, Table 253",
            fields: LOGIN_FIELDS,
        }),
        RequestPayloadSchema::Logout => Some(RequestSchemaDescriptor {
            source: "§6.1.31, Table 256",
            fields: LOGOUT_FIELDS,
        }),
        RequestPayloadSchema::Mac => Some(RequestSchemaDescriptor {
            source: "§6.1.32, Table 259",
            fields: MAC_FIELDS,
        }),
        RequestPayloadSchema::MacVerify => Some(RequestSchemaDescriptor {
            source: "§6.1.33, Table 262",
            fields: MACVERIFY_FIELDS,
        }),
        RequestPayloadSchema::ModifyAttribute => Some(RequestSchemaDescriptor {
            source: "§6.1.34, Table 265",
            fields: MODIFYATTRIBUTE_FIELDS,
        }),
        RequestPayloadSchema::ObtainLease => Some(RequestSchemaDescriptor {
            source: "§6.1.35, Table 268",
            fields: OBTAINLEASE_FIELDS,
        }),
        RequestPayloadSchema::Ping => Some(RequestSchemaDescriptor {
            source: "§6.1.36, Table 271",
            fields: PING_FIELDS,
        }),
        _ => None,
    }
}

fn descriptor_group_3(schema: RequestPayloadSchema) -> Option<RequestSchemaDescriptor> {
    match schema {
        RequestPayloadSchema::Pkcs11 => Some(RequestSchemaDescriptor {
            source: "§6.1.37, Table 273",
            fields: PKCS11_FIELDS,
        }),
        RequestPayloadSchema::Poll => Some(RequestSchemaDescriptor {
            source: "§6.1.38, Table 276",
            fields: POLL_FIELDS,
        }),
        RequestPayloadSchema::Process => Some(RequestSchemaDescriptor {
            source: "§6.1.39, Table 278",
            fields: PROCESS_FIELDS,
        }),
        RequestPayloadSchema::Query => Some(RequestSchemaDescriptor {
            source: "§6.1.40, Table 282",
            fields: QUERY_FIELDS,
        }),
        RequestPayloadSchema::QueryAsyncRequests => Some(RequestSchemaDescriptor {
            source: "§6.1.41, Table 285",
            fields: QUERYASYNCREQUESTS_FIELDS,
        }),
        RequestPayloadSchema::ReProvision => Some(RequestSchemaDescriptor {
            source: "§6.1.48, Table 313",
            fields: REPROVISION_FIELDS,
        }),
        RequestPayloadSchema::Recertify => Some(RequestSchemaDescriptor {
            source: "§6.1.45, Table 300",
            fields: RECERTIFY_FIELDS,
        }),
        RequestPayloadSchema::Recover => Some(RequestSchemaDescriptor {
            source: "§6.1.42, Table 288",
            fields: RECOVER_FIELDS,
        }),
        RequestPayloadSchema::Register => Some(RequestSchemaDescriptor {
            source: "§6.1.43, Table 291",
            fields: REGISTER_FIELDS,
        }),
        RequestPayloadSchema::Rekey => Some(RequestSchemaDescriptor {
            source: "§6.1.46, Table 305",
            fields: REKEY_FIELDS,
        }),
        RequestPayloadSchema::RekeyKeyPair => Some(RequestSchemaDescriptor {
            source: "§6.1.47, Table 310",
            fields: REKEYKEYPAIR_FIELDS,
        }),
        RequestPayloadSchema::Revoke => Some(RequestSchemaDescriptor {
            source: "§6.1.44, Table 295",
            fields: REVOKE_FIELDS,
        }),
        _ => None,
    }
}

fn descriptor_group_4(schema: RequestPayloadSchema) -> Option<RequestSchemaDescriptor> {
    match schema {
        RequestPayloadSchema::RngRetrieve => Some(RequestSchemaDescriptor {
            source: "§6.1.49, Table 316",
            fields: RNGRETRIEVE_FIELDS,
        }),
        RequestPayloadSchema::RngSeed => Some(RequestSchemaDescriptor {
            source: "§6.1.50, Table 319",
            fields: RNGSEED_FIELDS,
        }),
        RequestPayloadSchema::SetAttribute => Some(RequestSchemaDescriptor {
            source: "§6.1.51, Table 322",
            fields: SETATTRIBUTE_FIELDS,
        }),
        RequestPayloadSchema::SetConstraints => Some(RequestSchemaDescriptor {
            source: "§6.1.52, Table 325",
            fields: SETCONSTRAINTS_FIELDS,
        }),
        RequestPayloadSchema::SetDefaults => Some(RequestSchemaDescriptor {
            source: "§6.1.53, Table 328",
            fields: SETDEFAULTS_FIELDS,
        }),
        RequestPayloadSchema::Sign => Some(RequestSchemaDescriptor {
            source: "§6.1.55, Table 334",
            fields: SIGN_FIELDS,
        }),
        RequestPayloadSchema::SignatureVerify => Some(RequestSchemaDescriptor {
            source: "§6.1.56, Table 337",
            fields: SIGNATUREVERIFY_FIELDS,
        }),
        RequestPayloadSchema::Validate => Some(RequestSchemaDescriptor {
            source: "§6.1.57, Table 340",
            fields: VALIDATE_FIELDS,
        }),
        _ => None,
    }
}

static ACTIVATE_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R001",
    name: "Unique Identifier",
    tag: FieldTag::Exact(0x0042_0094),
    types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
    min_count: 0,
    max_count: Some(1),
    nested_schema: None,
}];

static ADDATTRIBUTE_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R002",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R003",
        name: "New Attribute",
        tag: FieldTag::Exact(0x0042_013d),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::NewAttribute),
    },
];

static ADJUSTATTRIBUTE_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R004",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R005",
        name: "Attribute Reference",
        tag: FieldTag::Exact(0x0042_013b),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R006",
        name: "Adjustment Type",
        tag: FieldTag::Exact(0x0042_0158),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R007",
        name: "Adjustment Value",
        tag: FieldTag::Exact(0x0042_0162),
        types: TYPE_STRUCTURE
            | TYPE_INTEGER
            | TYPE_LONG_INTEGER
            | TYPE_BIG_INTEGER
            | TYPE_ENUMERATION
            | TYPE_BOOLEAN
            | TYPE_TEXT_STRING
            | TYPE_BYTE_STRING
            | TYPE_DATE_TIME_EXTENDED
            | TYPE_DATE_TIME
            | TYPE_INTERVAL,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static ARCHIVE_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R008",
    name: "Unique Identifier",
    tag: FieldTag::Exact(0x0042_0094),
    types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
    min_count: 0,
    max_count: Some(1),
    nested_schema: None,
}];

static CANCEL_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R009",
    name: "Asynchronous Correlation Value",
    tag: FieldTag::Exact(0x0042_0006),
    types: TYPE_BYTE_STRING,
    min_count: 1,
    max_count: Some(1),
    nested_schema: None,
}];

static CERTIFY_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R010",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R011",
        name: "Certificate Request Type",
        tag: FieldTag::Exact(0x0042_0019),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R012",
        name: "Certificate Request Value",
        tag: FieldTag::Exact(0x0042_0140),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R013",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R014",
        name: "Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_015f),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static CHECK_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R015",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R016",
        name: "Usage Limits Count",
        tag: FieldTag::Exact(0x0042_0096),
        types: TYPE_INTEGER | TYPE_LONG_INTEGER,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R017",
        name: "Cryptographic Usage Mask",
        tag: FieldTag::Exact(0x0042_002c),
        types: TYPE_INTEGER,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R018",
        name: "Lease Time",
        tag: FieldTag::Exact(0x0042_0049),
        types: TYPE_DATE_TIME | TYPE_INTERVAL,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static CREATE_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R019",
        name: "Object Type",
        tag: FieldTag::Exact(0x0042_0057),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R020",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R021",
        name: "Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_015f),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static CREATEKEYPAIR_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R022",
        name: "Common Attributes",
        tag: FieldTag::Exact(0x0042_0126),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R023",
        name: "Private Key Attributes",
        tag: FieldTag::Exact(0x0042_0127),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R024",
        name: "Public Key Attributes",
        tag: FieldTag::Exact(0x0042_0128),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R025",
        name: "Common Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_0163),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R026",
        name: "Private Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_0164),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R027",
        name: "Public Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_0165),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static CREATESPLITKEY_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R028",
        name: "Object Type",
        tag: FieldTag::Exact(0x0042_0057),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R029",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R030",
        name: "Split Key Parts",
        tag: FieldTag::Exact(0x0042_008b),
        types: TYPE_INTEGER,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R031",
        name: "Split Key Threshold",
        tag: FieldTag::Exact(0x0042_008c),
        types: TYPE_INTEGER,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R032",
        name: "Split Key Method",
        tag: FieldTag::Exact(0x0042_008a),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R033",
        name: "Prime Field Size",
        tag: FieldTag::Exact(0x0042_0062),
        types: TYPE_INTEGER | TYPE_BIG_INTEGER,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R034",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R035",
        name: "Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_015f),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static DECRYPT_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R036",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R037",
        name: "Cryptographic Parameters",
        tag: FieldTag::Exact(0x0042_002b),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CryptographicParameters),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R038",
        name: "Data",
        tag: FieldTag::Exact(0x0042_00c2),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R039",
        name: "IV",
        tag: FieldTag::Exact(0x0042_003d),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R040",
        name: "Correlation Value",
        tag: FieldTag::Exact(0x0042_00d6),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R041",
        name: "Init Indicator",
        tag: FieldTag::Exact(0x0042_00d7),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R042",
        name: "Final Indicator",
        tag: FieldTag::Exact(0x0042_00d8),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R043",
        name: "Authenticated Encryption Additional Data",
        tag: FieldTag::Exact(0x0042_00fe),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R044",
        name: "Authenticated Encryption Tag",
        tag: FieldTag::Exact(0x0042_00ff),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static DELEGATEDLOGIN_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R045",
        name: "Lease Time",
        tag: FieldTag::Exact(0x0042_0049),
        types: TYPE_DATE_TIME | TYPE_INTERVAL,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R046",
        name: "Request Count",
        tag: FieldTag::Exact(0x0042_014c),
        types: TYPE_INTEGER,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R047",
        name: "Usage Limits",
        tag: FieldTag::Exact(0x0042_0095),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::UsageLimits),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R048",
        name: "Rights",
        tag: FieldTag::Exact(0x0042_014d),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Rights),
    },
];

static DELETEATTRIBUTE_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R049",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R050",
        name: "Current Attribute",
        tag: FieldTag::Exact(0x0042_013c),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CurrentAttribute),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R051",
        name: "Attribute Reference",
        tag: FieldTag::Exact(0x0042_013b),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static DERIVEKEY_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R052",
        name: "Object Type",
        tag: FieldTag::Exact(0x0042_0057),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R053",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 1,
        max_count: None,
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R054",
        name: "Derivation Method",
        tag: FieldTag::Exact(0x0042_0031),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R055",
        name: "Derivation Parameters",
        tag: FieldTag::Exact(0x0042_0032),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::DerivationParameters),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R056",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
];

static DESTROY_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R057",
    name: "Unique Identifier",
    tag: FieldTag::Exact(0x0042_0094),
    types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
    min_count: 0,
    max_count: Some(1),
    nested_schema: None,
}];

static DISCOVERVERSIONS_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R058",
    name: "Protocol Version",
    tag: FieldTag::Exact(0x0042_0069),
    types: TYPE_STRUCTURE,
    min_count: 0,
    max_count: None,
    nested_schema: Some(NestedStructureSchema::ProtocolVersion),
}];

static ENCRYPT_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R059",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R060",
        name: "Cryptographic Parameters",
        tag: FieldTag::Exact(0x0042_002b),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CryptographicParameters),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R061",
        name: "Data",
        tag: FieldTag::Exact(0x0042_00c2),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R062",
        name: "IV",
        tag: FieldTag::Exact(0x0042_003d),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R063",
        name: "Correlation Value",
        tag: FieldTag::Exact(0x0042_00d6),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R064",
        name: "Init Indicator",
        tag: FieldTag::Exact(0x0042_00d7),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R065",
        name: "Final Indicator",
        tag: FieldTag::Exact(0x0042_00d8),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R066",
        name: "Authenticated Encryption Additional Data",
        tag: FieldTag::Exact(0x0042_00fe),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static EXPORT_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R067",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R068",
        name: "Key Format Type",
        tag: FieldTag::Exact(0x0042_0042),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R069",
        name: "Key Wrap Type",
        tag: FieldTag::Exact(0x0042_00f8),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R070",
        name: "Key Compression Type",
        tag: FieldTag::Exact(0x0042_0041),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R071",
        name: "Key Wrapping Specification",
        tag: FieldTag::Exact(0x0042_0047),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::KeyWrappingSpecification),
    },
];

static GET_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R072",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R073",
        name: "Key Format Type",
        tag: FieldTag::Exact(0x0042_0042),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R074",
        name: "Key Wrap Type",
        tag: FieldTag::Exact(0x0042_00f8),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R075",
        name: "Key Compression Type",
        tag: FieldTag::Exact(0x0042_0041),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R076",
        name: "Key Wrapping Specification",
        tag: FieldTag::Exact(0x0042_0047),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::KeyWrappingSpecification),
    },
];

static GETATTRIBUTELIST_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R079",
    name: "Unique Identifier",
    tag: FieldTag::Exact(0x0042_0094),
    types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
    min_count: 0,
    max_count: Some(1),
    nested_schema: None,
}];

static GETATTRIBUTES_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R077",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R078",
        name: "Attribute Reference",
        tag: FieldTag::Exact(0x0042_013b),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: None,
        nested_schema: None,
    },
];

static GETCONSTRAINTS_FIELDS: &[FieldDescriptor] = &[];

static GETUSAGEALLOCATION_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R080",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R081",
        name: "Usage Limits Count",
        tag: FieldTag::Exact(0x0042_0096),
        types: TYPE_INTEGER | TYPE_LONG_INTEGER,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
];

static HASH_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R082",
        name: "Cryptographic Parameters",
        tag: FieldTag::Exact(0x0042_002b),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CryptographicParameters),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R083",
        name: "Data",
        tag: FieldTag::Exact(0x0042_00c2),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R084",
        name: "Correlation Value",
        tag: FieldTag::Exact(0x0042_00d6),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R085",
        name: "Init Indicator",
        tag: FieldTag::Exact(0x0042_00d7),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R086",
        name: "Final Indicator",
        tag: FieldTag::Exact(0x0042_00d8),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static IMPORT_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R087",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R088",
        name: "Object Type",
        tag: FieldTag::Exact(0x0042_0057),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R089",
        name: "Replace Existing",
        tag: FieldTag::Exact(0x0042_0124),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R090",
        name: "Key Wrap Type",
        tag: FieldTag::Exact(0x0042_00f8),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R091",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R092",
        name: "Any Object (Section 2)",
        tag: FieldTag::ManagedObject,
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
];

static INTEROP_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R093",
        name: "Interop Function",
        tag: FieldTag::Exact(0x0042_0160),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R094",
        name: "Interop Identifier",
        tag: FieldTag::Exact(0x0042_0161),
        types: TYPE_TEXT_STRING,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
];

static JOINSPLITKEY_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R095",
        name: "Object Type",
        tag: FieldTag::Exact(0x0042_0057),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R096",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 1,
        max_count: None,
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R097",
        name: "Secret Data Type",
        tag: FieldTag::Exact(0x0042_0086),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R098",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R099",
        name: "Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_015f),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static LOCATE_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R100",
        name: "Maximum Items",
        tag: FieldTag::Exact(0x0042_004f),
        types: TYPE_INTEGER,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R101",
        name: "Offset Items",
        tag: FieldTag::Exact(0x0042_00d4),
        types: TYPE_INTEGER,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R102",
        name: "Storage Status Mask",
        tag: FieldTag::Exact(0x0042_008e),
        types: TYPE_INTEGER,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R103",
        name: "Object Group Member",
        tag: FieldTag::Exact(0x0042_00ac),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R104",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
];

static LOG_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R105",
    name: "Log Message",
    tag: FieldTag::Exact(0x0042_0141),
    types: TYPE_TEXT_STRING,
    min_count: 1,
    max_count: Some(1),
    nested_schema: None,
}];

static LOGIN_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R106",
        name: "Lease Time",
        tag: FieldTag::Exact(0x0042_0049),
        types: TYPE_DATE_TIME | TYPE_INTERVAL,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R107",
        name: "Request Count",
        tag: FieldTag::Exact(0x0042_014c),
        types: TYPE_INTEGER,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R108",
        name: "Usage Limits",
        tag: FieldTag::Exact(0x0042_0095),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::UsageLimits),
    },
];

static LOGOUT_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R109",
    name: "Ticket",
    tag: FieldTag::Exact(0x0042_0149),
    types: TYPE_STRUCTURE | TYPE_ENUMERATION | TYPE_BYTE_STRING,
    min_count: 1,
    max_count: Some(1),
    nested_schema: Some(NestedStructureSchema::Ticket),
}];

static MAC_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R110",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R111",
        name: "Cryptographic Parameters",
        tag: FieldTag::Exact(0x0042_002b),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CryptographicParameters),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R112",
        name: "Data",
        tag: FieldTag::Exact(0x0042_00c2),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R113",
        name: "Correlation Value",
        tag: FieldTag::Exact(0x0042_00d6),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R114",
        name: "Init Indicator",
        tag: FieldTag::Exact(0x0042_00d7),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R115",
        name: "Final Indicator",
        tag: FieldTag::Exact(0x0042_00d8),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static MACVERIFY_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R116",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R117",
        name: "Cryptographic Parameters",
        tag: FieldTag::Exact(0x0042_002b),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CryptographicParameters),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R118",
        name: "Data",
        tag: FieldTag::Exact(0x0042_00c2),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R119",
        name: "MAC Data",
        tag: FieldTag::Exact(0x0042_00c6),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R120",
        name: "Correlation Value",
        tag: FieldTag::Exact(0x0042_00d6),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R121",
        name: "Init Indicator",
        tag: FieldTag::Exact(0x0042_00d7),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R122",
        name: "Final Indicator",
        tag: FieldTag::Exact(0x0042_00d8),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static MODIFYATTRIBUTE_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R123",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R124",
        name: "Current Attribute",
        tag: FieldTag::Exact(0x0042_013c),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CurrentAttribute),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R125",
        name: "New Attribute",
        tag: FieldTag::Exact(0x0042_013d),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::NewAttribute),
    },
];

static OBTAINLEASE_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R126",
    name: "Unique Identifier",
    tag: FieldTag::Exact(0x0042_0094),
    types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
    min_count: 0,
    max_count: Some(1),
    nested_schema: None,
}];

static PING_FIELDS: &[FieldDescriptor] = &[];

static PKCS11_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R127",
        name: "PKCS#11 Interface",
        tag: FieldTag::Exact(0x0042_0159),
        types: TYPE_STRUCTURE | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R128",
        name: "PKCS#11 Function",
        tag: FieldTag::Exact(0x0042_015a),
        types: TYPE_INTEGER | TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R129",
        name: "Correlation Value",
        tag: FieldTag::Exact(0x0042_00d6),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R130",
        name: "PKCS#11 Input Parameters",
        tag: FieldTag::Exact(0x0042_015b),
        types: TYPE_STRUCTURE | TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static POLL_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R131",
    name: "Asynchronous Correlation Value",
    tag: FieldTag::Exact(0x0042_0006),
    types: TYPE_BYTE_STRING,
    min_count: 1,
    max_count: Some(1),
    nested_schema: None,
}];

static PROCESS_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R132",
    name: "Asynchronous Correlation Value",
    tag: FieldTag::Exact(0x0042_0006),
    types: TYPE_BYTE_STRING,
    min_count: 1,
    max_count: Some(1),
    nested_schema: None,
}];

static QUERY_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R133",
        name: "Query Function",
        tag: FieldTag::Exact(0x0042_0074),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: None,
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R134",
        name: "Object Groups",
        tag: FieldTag::Exact(0x0042_0166),
        types: TYPE_STRUCTURE | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::ObjectGroups),
    },
];

static QUERYASYNCREQUESTS_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R135",
        name: "Asynchronous Correlation Values",
        tag: FieldTag::Exact(0x0042_0176),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::AsynchronousCorrelationValues),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R136",
        name: "Operations",
        tag: FieldTag::Exact(0x0042_014f),
        types: TYPE_STRUCTURE | TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Operations),
    },
];

static REPROVISION_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R164",
        name: "Certificate Request",
        tag: FieldTag::Exact(0x0042_0018),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CertificateRequest),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R165",
        name: "Certificate",
        tag: FieldTag::Exact(0x0042_0013),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Certificate),
    },
];

static RECERTIFY_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R145",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R146",
        name: "Certificate Request Unique Identifier",
        tag: FieldTag::Exact(0x0042_0139),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R147",
        name: "Certificate Request Type",
        tag: FieldTag::Exact(0x0042_0019),
        types: TYPE_ENUMERATION,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R148",
        name: "Certificate Request Value",
        tag: FieldTag::Exact(0x0042_0140),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R149",
        name: "Offset",
        tag: FieldTag::Exact(0x0042_0058),
        types: TYPE_INTERVAL,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R150",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R151",
        name: "Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_015f),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static RECOVER_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R137",
    name: "Unique Identifier",
    tag: FieldTag::Exact(0x0042_0094),
    types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
    min_count: 0,
    max_count: Some(1),
    nested_schema: None,
}];

static REGISTER_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R138",
        name: "Object Type",
        tag: FieldTag::Exact(0x0042_0057),
        types: TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R139",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R140",
        name: "Any Object (Section 2)",
        tag: FieldTag::ManagedObject,
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R141",
        name: "Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_015f),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static REKEY_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R152",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R153",
        name: "Offset",
        tag: FieldTag::Exact(0x0042_0058),
        types: TYPE_INTERVAL,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R154",
        name: "Attributes",
        tag: FieldTag::Exact(0x0042_0125),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R155",
        name: "Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_015f),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static REKEYKEYPAIR_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R156",
        name: "Private Key Unique Identifier",
        tag: FieldTag::Exact(0x0042_0066),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R157",
        name: "Offset",
        tag: FieldTag::Exact(0x0042_0058),
        types: TYPE_INTERVAL,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R158",
        name: "Common Attributes",
        tag: FieldTag::Exact(0x0042_0126),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R159",
        name: "Private Key Attributes",
        tag: FieldTag::Exact(0x0042_0127),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R160",
        name: "Public Key Attributes",
        tag: FieldTag::Exact(0x0042_0128),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::Attributes),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R161",
        name: "Common Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_0163),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R162",
        name: "Private Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_0164),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R163",
        name: "Public Protection Storage Masks",
        tag: FieldTag::Exact(0x0042_0165),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static REVOKE_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R142",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R143",
        name: "Revocation Reason",
        tag: FieldTag::Exact(0x0042_0081),
        types: TYPE_STRUCTURE | TYPE_ENUMERATION,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::RevocationReason),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R144",
        name: "Compromise Occurrence Date",
        tag: FieldTag::Exact(0x0042_0021),
        types: TYPE_DATE_TIME,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static RNGRETRIEVE_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R166",
    name: "Data Length",
    tag: FieldTag::Exact(0x0042_00c4),
    types: TYPE_INTEGER,
    min_count: 1,
    max_count: Some(1),
    nested_schema: None,
}];

static RNGSEED_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R167",
    name: "Data",
    tag: FieldTag::Exact(0x0042_00c2),
    types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_BYTE_STRING,
    min_count: 1,
    max_count: Some(1),
    nested_schema: None,
}];

static SETATTRIBUTE_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R168",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R169",
        name: "New Attribute",
        tag: FieldTag::Exact(0x0042_013d),
        types: TYPE_STRUCTURE,
        min_count: 1,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::NewAttribute),
    },
];

static SETCONSTRAINTS_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R170",
    name: "Constraints",
    tag: FieldTag::Exact(0x0042_0168),
    types: TYPE_STRUCTURE,
    min_count: 1,
    max_count: Some(1),
    nested_schema: Some(NestedStructureSchema::Constraints),
}];

static SETDEFAULTS_FIELDS: &[FieldDescriptor] = &[FieldDescriptor {
    rule_id: "KMIPKIT-0051-R171",
    name: "Defaults Information",
    tag: FieldTag::Exact(0x0042_0152),
    types: TYPE_STRUCTURE,
    min_count: 0,
    max_count: Some(1),
    nested_schema: Some(NestedStructureSchema::DefaultsInformation),
}];

static SIGN_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R172",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R173",
        name: "Cryptographic Parameters",
        tag: FieldTag::Exact(0x0042_002b),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CryptographicParameters),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R174",
        name: "Data",
        tag: FieldTag::Exact(0x0042_00c2),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R175",
        name: "Digested Data",
        tag: FieldTag::Exact(0x0042_0107),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R176",
        name: "Correlation Value",
        tag: FieldTag::Exact(0x0042_00d6),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R177",
        name: "Init Indicator",
        tag: FieldTag::Exact(0x0042_00d7),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R178",
        name: "Final Indicator",
        tag: FieldTag::Exact(0x0042_00d8),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static SIGNATUREVERIFY_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R179",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R180",
        name: "Cryptographic Parameters",
        tag: FieldTag::Exact(0x0042_002b),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: Some(1),
        nested_schema: Some(NestedStructureSchema::CryptographicParameters),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R181",
        name: "Data",
        tag: FieldTag::Exact(0x0042_00c2),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R182",
        name: "Digested Data",
        tag: FieldTag::Exact(0x0042_0107),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R183",
        name: "Signature Data",
        tag: FieldTag::Exact(0x0042_00c3),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R184",
        name: "Correlation Value",
        tag: FieldTag::Exact(0x0042_00d6),
        types: TYPE_BYTE_STRING,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R185",
        name: "Init Indicator",
        tag: FieldTag::Exact(0x0042_00d7),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R186",
        name: "Final Indicator",
        tag: FieldTag::Exact(0x0042_00d8),
        types: TYPE_BOOLEAN,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];

static VALIDATE_FIELDS: &[FieldDescriptor] = &[
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R187",
        name: "Certificate",
        tag: FieldTag::Exact(0x0042_0013),
        types: TYPE_STRUCTURE,
        min_count: 0,
        max_count: None,
        nested_schema: Some(NestedStructureSchema::Certificate),
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R188",
        name: "Unique Identifier",
        tag: FieldTag::Exact(0x0042_0094),
        types: TYPE_INTEGER | TYPE_ENUMERATION | TYPE_TEXT_STRING,
        min_count: 0,
        max_count: None,
        nested_schema: None,
    },
    FieldDescriptor {
        rule_id: "KMIPKIT-0051-R189",
        name: "Validity Date",
        tag: FieldTag::Exact(0x0042_009a),
        types: TYPE_DATE_TIME,
        min_count: 0,
        max_count: Some(1),
        nested_schema: None,
    },
];
