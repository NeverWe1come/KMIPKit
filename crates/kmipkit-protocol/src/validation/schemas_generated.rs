//! Generated operation Request Payload schema identities and descriptors.
//! Regenerate with `python tools/validation_schema/generate.py`.

use super::{FieldDescriptor, FieldTag, NestedStructureSchema, RequestSchemaDescriptor};

macro_rules! field_tag {
    (managed) => {
        FieldTag::ManagedObject
    };
    ($tag:literal) => {
        FieldTag::Exact($tag)
    };
}

macro_rules! field_max {
    (many) => {
        None
    };
    ($max:literal) => {
        Some($max)
    };
}

macro_rules! field_nested {
    (none) => {
        None
    };
    ($nested:ident) => {
        Some(NestedStructureSchema::$nested)
    };
}

macro_rules! field {
    ($rule:ident $name:literal $tag:tt $types:literal $min:literal $max:tt $nested:tt) => {
        FieldDescriptor {
            rule_id: concat!("KMIPKIT-0051-", stringify!($rule)),
            name: $name,
            tag: field_tag!($tag),
            types: $types,
            min_count: $min,
            max_count: field_max!($max),
            nested_schema: field_nested!($nested),
        }
    };
}

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

static ACTIVATE_FIELDS: &[FieldDescriptor] =
    &[field!(R001 "Unique Identifier" 0x0042_0094 0x052 0 1 none)];

static ADDATTRIBUTE_FIELDS: &[FieldDescriptor] = &[
    field!(R002 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R003 "New Attribute" 0x0042_013d 0x001 1 1 NewAttribute),
];

static ADJUSTATTRIBUTE_FIELDS: &[FieldDescriptor] = &[
    field!(R004 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R005 "Attribute Reference" 0x0042_013b 0x010 1 1 none),
    field!(R006 "Adjustment Type" 0x0042_0158 0x010 1 1 none),
    field!(R007 "Adjustment Value" 0x0042_0162 0x7ff 0 1 none),
];

static ARCHIVE_FIELDS: &[FieldDescriptor] =
    &[field!(R008 "Unique Identifier" 0x0042_0094 0x052 0 1 none)];

static CANCEL_FIELDS: &[FieldDescriptor] =
    &[field!(R009 "Asynchronous Correlation Value" 0x0042_0006 0x080 1 1 none)];

static CERTIFY_FIELDS: &[FieldDescriptor] = &[
    field!(R010 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R011 "Certificate Request Type" 0x0042_0019 0x010 0 1 none),
    field!(R012 "Certificate Request Value" 0x0042_0140 0x080 0 1 none),
    field!(R013 "Attributes" 0x0042_0125 0x001 0 1 Attributes),
    field!(R014 "Protection Storage Masks" 0x0042_015f 0x001 0 1 none),
];

static CHECK_FIELDS: &[FieldDescriptor] = &[
    field!(R015 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R016 "Usage Limits Count" 0x0042_0096 0x006 0 1 none),
    field!(R017 "Cryptographic Usage Mask" 0x0042_002c 0x002 0 1 none),
    field!(R018 "Lease Time" 0x0042_0049 0x300 0 1 none),
];

static CREATE_FIELDS: &[FieldDescriptor] = &[
    field!(R019 "Object Type" 0x0042_0057 0x010 1 1 none),
    field!(R020 "Attributes" 0x0042_0125 0x001 1 1 Attributes),
    field!(R021 "Protection Storage Masks" 0x0042_015f 0x001 0 1 none),
];

static CREATEKEYPAIR_FIELDS: &[FieldDescriptor] = &[
    field!(R022 "Common Attributes" 0x0042_0126 0x001 0 1 Attributes),
    field!(R023 "Private Key Attributes" 0x0042_0127 0x001 0 1 Attributes),
    field!(R024 "Public Key Attributes" 0x0042_0128 0x001 0 1 Attributes),
    field!(R025 "Common Protection Storage Masks" 0x0042_0163 0x001 0 1 none),
    field!(R026 "Private Protection Storage Masks" 0x0042_0164 0x001 0 1 none),
    field!(R027 "Public Protection Storage Masks" 0x0042_0165 0x001 0 1 none),
];

static CREATESPLITKEY_FIELDS: &[FieldDescriptor] = &[
    field!(R028 "Object Type" 0x0042_0057 0x010 1 1 none),
    field!(R029 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R030 "Split Key Parts" 0x0042_008b 0x002 1 1 none),
    field!(R031 "Split Key Threshold" 0x0042_008c 0x002 1 1 none),
    field!(R032 "Split Key Method" 0x0042_008a 0x010 1 1 none),
    field!(R033 "Prime Field Size" 0x0042_0062 0x00a 0 1 none),
    field!(R034 "Attributes" 0x0042_0125 0x001 1 1 Attributes),
    field!(R035 "Protection Storage Masks" 0x0042_015f 0x001 0 1 none),
];

static DECRYPT_FIELDS: &[FieldDescriptor] = &[
    field!(R036 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R037 "Cryptographic Parameters" 0x0042_002b 0x001 0 1 CryptographicParameters),
    field!(R038 "Data" 0x0042_00c2 0x092 0 1 none),
    field!(R039 "IV" 0x0042_003d 0x080 0 1 none),
    field!(R040 "Correlation Value" 0x0042_00d6 0x080 0 1 none),
    field!(R041 "Init Indicator" 0x0042_00d7 0x020 0 1 none),
    field!(R042 "Final Indicator" 0x0042_00d8 0x020 0 1 none),
    field!(R043 "Authenticated Encryption Additional Data" 0x0042_00fe 0x080 0 1 none),
    field!(R044 "Authenticated Encryption Tag" 0x0042_00ff 0x080 0 1 none),
];

static DELEGATEDLOGIN_FIELDS: &[FieldDescriptor] = &[
    field!(R045 "Lease Time" 0x0042_0049 0x300 0 1 none),
    field!(R046 "Request Count" 0x0042_014c 0x002 0 1 none),
    field!(R047 "Usage Limits" 0x0042_0095 0x001 0 1 UsageLimits),
    field!(R048 "Rights" 0x0042_014d 0x001 1 1 Rights),
];

static DELETEATTRIBUTE_FIELDS: &[FieldDescriptor] = &[
    field!(R049 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R050 "Current Attribute" 0x0042_013c 0x001 0 1 CurrentAttribute),
    field!(R051 "Attribute Reference" 0x0042_013b 0x010 0 1 none),
];

static DERIVEKEY_FIELDS: &[FieldDescriptor] = &[
    field!(R052 "Object Type" 0x0042_0057 0x010 1 1 none),
    field!(R053 "Unique Identifier" 0x0042_0094 0x052 1 many none),
    field!(R054 "Derivation Method" 0x0042_0031 0x010 1 1 none),
    field!(R055 "Derivation Parameters" 0x0042_0032 0x001 1 1 DerivationParameters),
    field!(R056 "Attributes" 0x0042_0125 0x001 1 1 Attributes),
];

static DESTROY_FIELDS: &[FieldDescriptor] =
    &[field!(R057 "Unique Identifier" 0x0042_0094 0x052 0 1 none)];

static DISCOVERVERSIONS_FIELDS: &[FieldDescriptor] =
    &[field!(R058 "Protocol Version" 0x0042_0069 0x001 0 many ProtocolVersion)];

static ENCRYPT_FIELDS: &[FieldDescriptor] = &[
    field!(R059 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R060 "Cryptographic Parameters" 0x0042_002b 0x001 0 1 CryptographicParameters),
    field!(R061 "Data" 0x0042_00c2 0x092 0 1 none),
    field!(R062 "IV" 0x0042_003d 0x080 0 1 none),
    field!(R063 "Correlation Value" 0x0042_00d6 0x080 0 1 none),
    field!(R064 "Init Indicator" 0x0042_00d7 0x020 0 1 none),
    field!(R065 "Final Indicator" 0x0042_00d8 0x020 0 1 none),
    field!(R066 "Authenticated Encryption Additional Data" 0x0042_00fe 0x080 0 1 none),
];

static EXPORT_FIELDS: &[FieldDescriptor] = &[
    field!(R067 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R068 "Key Format Type" 0x0042_0042 0x010 0 1 none),
    field!(R069 "Key Wrap Type" 0x0042_00f8 0x010 0 1 none),
    field!(R070 "Key Compression Type" 0x0042_0041 0x010 0 1 none),
    field!(R071 "Key Wrapping Specification" 0x0042_0047 0x001 0 1 KeyWrappingSpecification),
];

static GET_FIELDS: &[FieldDescriptor] = &[
    field!(R072 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R073 "Key Format Type" 0x0042_0042 0x010 0 1 none),
    field!(R074 "Key Wrap Type" 0x0042_00f8 0x010 0 1 none),
    field!(R075 "Key Compression Type" 0x0042_0041 0x010 0 1 none),
    field!(R076 "Key Wrapping Specification" 0x0042_0047 0x001 0 1 KeyWrappingSpecification),
];

static GETATTRIBUTELIST_FIELDS: &[FieldDescriptor] =
    &[field!(R079 "Unique Identifier" 0x0042_0094 0x052 0 1 none)];

static GETATTRIBUTES_FIELDS: &[FieldDescriptor] = &[
    field!(R077 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R078 "Attribute Reference" 0x0042_013b 0x010 0 many none),
];

static GETCONSTRAINTS_FIELDS: &[FieldDescriptor] = &[];

static GETUSAGEALLOCATION_FIELDS: &[FieldDescriptor] = &[
    field!(R080 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R081 "Usage Limits Count" 0x0042_0096 0x006 1 1 none),
];

static HASH_FIELDS: &[FieldDescriptor] = &[
    field!(R082 "Cryptographic Parameters" 0x0042_002b 0x001 1 1 CryptographicParameters),
    field!(R083 "Data" 0x0042_00c2 0x092 0 1 none),
    field!(R084 "Correlation Value" 0x0042_00d6 0x080 0 1 none),
    field!(R085 "Init Indicator" 0x0042_00d7 0x020 0 1 none),
    field!(R086 "Final Indicator" 0x0042_00d8 0x020 0 1 none),
];

static IMPORT_FIELDS: &[FieldDescriptor] = &[
    field!(R087 "Unique Identifier" 0x0042_0094 0x052 1 1 none),
    field!(R088 "Object Type" 0x0042_0057 0x010 1 1 none),
    field!(R089 "Replace Existing" 0x0042_0124 0x020 0 1 none),
    field!(R090 "Key Wrap Type" 0x0042_00f8 0x010 0 1 none),
    field!(R091 "Attributes" 0x0042_0125 0x001 1 1 Attributes),
    field!(R092 "Any Object (Section 2)" managed 0x001 1 1 none),
];

static INTEROP_FIELDS: &[FieldDescriptor] = &[
    field!(R093 "Interop Function" 0x0042_0160 0x010 1 1 none),
    field!(R094 "Interop Identifier" 0x0042_0161 0x040 1 1 none),
];

static JOINSPLITKEY_FIELDS: &[FieldDescriptor] = &[
    field!(R095 "Object Type" 0x0042_0057 0x010 1 1 none),
    field!(R096 "Unique Identifier" 0x0042_0094 0x052 1 many none),
    field!(R097 "Secret Data Type" 0x0042_0086 0x010 0 1 none),
    field!(R098 "Attributes" 0x0042_0125 0x001 0 1 Attributes),
    field!(R099 "Protection Storage Masks" 0x0042_015f 0x001 0 1 none),
];

static LOCATE_FIELDS: &[FieldDescriptor] = &[
    field!(R100 "Maximum Items" 0x0042_004f 0x002 0 1 none),
    field!(R101 "Offset Items" 0x0042_00d4 0x002 0 1 none),
    field!(R102 "Storage Status Mask" 0x0042_008e 0x002 0 1 none),
    field!(R103 "Object Group Member" 0x0042_00ac 0x010 0 1 none),
    field!(R104 "Attributes" 0x0042_0125 0x001 1 1 Attributes),
];

static LOG_FIELDS: &[FieldDescriptor] = &[field!(R105 "Log Message" 0x0042_0141 0x040 1 1 none)];

static LOGIN_FIELDS: &[FieldDescriptor] = &[
    field!(R106 "Lease Time" 0x0042_0049 0x300 0 1 none),
    field!(R107 "Request Count" 0x0042_014c 0x002 0 1 none),
    field!(R108 "Usage Limits" 0x0042_0095 0x001 0 1 UsageLimits),
];

static LOGOUT_FIELDS: &[FieldDescriptor] = &[field!(R109 "Ticket" 0x0042_0149 0x091 1 1 Ticket)];

static MAC_FIELDS: &[FieldDescriptor] = &[
    field!(R110 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R111 "Cryptographic Parameters" 0x0042_002b 0x001 0 1 CryptographicParameters),
    field!(R112 "Data" 0x0042_00c2 0x092 0 1 none),
    field!(R113 "Correlation Value" 0x0042_00d6 0x080 0 1 none),
    field!(R114 "Init Indicator" 0x0042_00d7 0x020 0 1 none),
    field!(R115 "Final Indicator" 0x0042_00d8 0x020 0 1 none),
];

static MACVERIFY_FIELDS: &[FieldDescriptor] = &[
    field!(R116 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R117 "Cryptographic Parameters" 0x0042_002b 0x001 0 1 CryptographicParameters),
    field!(R118 "Data" 0x0042_00c2 0x092 0 1 none),
    field!(R119 "MAC Data" 0x0042_00c6 0x080 0 1 none),
    field!(R120 "Correlation Value" 0x0042_00d6 0x080 0 1 none),
    field!(R121 "Init Indicator" 0x0042_00d7 0x020 0 1 none),
    field!(R122 "Final Indicator" 0x0042_00d8 0x020 0 1 none),
];

static MODIFYATTRIBUTE_FIELDS: &[FieldDescriptor] = &[
    field!(R123 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R124 "Current Attribute" 0x0042_013c 0x001 0 1 CurrentAttribute),
    field!(R125 "New Attribute" 0x0042_013d 0x001 1 1 NewAttribute),
];

static OBTAINLEASE_FIELDS: &[FieldDescriptor] =
    &[field!(R126 "Unique Identifier" 0x0042_0094 0x052 0 1 none)];

static PING_FIELDS: &[FieldDescriptor] = &[];

static PKCS11_FIELDS: &[FieldDescriptor] = &[
    field!(R127 "PKCS#11 Interface" 0x0042_0159 0x001 0 1 Pkcs11Interface),
    field!(R128 "PKCS#11 Function" 0x0042_015a 0x012 1 1 none),
    field!(R129 "Correlation Value" 0x0042_00d6 0x080 0 1 none),
    field!(R130 "PKCS#11 Input Parameters" 0x0042_015b 0x001 0 1 Pkcs11InputParameters),
];

static POLL_FIELDS: &[FieldDescriptor] =
    &[field!(R131 "Asynchronous Correlation Value" 0x0042_0006 0x080 1 1 none)];

static PROCESS_FIELDS: &[FieldDescriptor] =
    &[field!(R132 "Asynchronous Correlation Value" 0x0042_0006 0x080 1 1 none)];

static QUERY_FIELDS: &[FieldDescriptor] = &[
    field!(R133 "Query Function" 0x0042_0074 0x010 1 many none),
    field!(R134 "Object Groups" 0x0042_0166 0x041 0 1 ObjectGroups),
];

static QUERYASYNCREQUESTS_FIELDS: &[FieldDescriptor] = &[
    field!(R135 "Asynchronous Correlation Values" 0x0042_0176 0x001 0 1 AsynchronousCorrelationValues),
    field!(R136 "Operations" 0x0042_014f 0x011 0 1 Operations),
];

static REPROVISION_FIELDS: &[FieldDescriptor] = &[
    field!(R164 "Certificate Request" 0x0042_0018 0x001 0 1 CertificateRequest),
    field!(R165 "Certificate" 0x0042_0013 0x001 0 1 Certificate),
];

static RECERTIFY_FIELDS: &[FieldDescriptor] = &[
    field!(R145 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R146 "Certificate Request Unique Identifier" 0x0042_0139 0x052 0 1 none),
    field!(R147 "Certificate Request Type" 0x0042_0019 0x010 0 1 none),
    field!(R148 "Certificate Request Value" 0x0042_0140 0x080 0 1 none),
    field!(R149 "Offset" 0x0042_0058 0x200 0 1 none),
    field!(R150 "Attributes" 0x0042_0125 0x001 0 1 Attributes),
    field!(R151 "Protection Storage Masks" 0x0042_015f 0x001 0 1 none),
];

static RECOVER_FIELDS: &[FieldDescriptor] =
    &[field!(R137 "Unique Identifier" 0x0042_0094 0x052 0 1 none)];

static REGISTER_FIELDS: &[FieldDescriptor] = &[
    field!(R138 "Object Type" 0x0042_0057 0x010 1 1 none),
    field!(R139 "Attributes" 0x0042_0125 0x001 1 1 Attributes),
    field!(R140 "Any Object (Section 2)" managed 0x001 1 1 none),
    field!(R141 "Protection Storage Masks" 0x0042_015f 0x001 0 1 none),
];

static REKEY_FIELDS: &[FieldDescriptor] = &[
    field!(R152 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R153 "Offset" 0x0042_0058 0x200 0 1 none),
    field!(R154 "Attributes" 0x0042_0125 0x001 0 1 Attributes),
    field!(R155 "Protection Storage Masks" 0x0042_015f 0x001 0 1 none),
];

static REKEYKEYPAIR_FIELDS: &[FieldDescriptor] = &[
    field!(R156 "Private Key Unique Identifier" 0x0042_0066 0x052 0 1 none),
    field!(R157 "Offset" 0x0042_0058 0x200 0 1 none),
    field!(R158 "Common Attributes" 0x0042_0126 0x001 0 1 Attributes),
    field!(R159 "Private Key Attributes" 0x0042_0127 0x001 0 1 Attributes),
    field!(R160 "Public Key Attributes" 0x0042_0128 0x001 0 1 Attributes),
    field!(R161 "Common Protection Storage Masks" 0x0042_0163 0x001 0 1 none),
    field!(R162 "Private Protection Storage Masks" 0x0042_0164 0x001 0 1 none),
    field!(R163 "Public Protection Storage Masks" 0x0042_0165 0x001 0 1 none),
];

static REVOKE_FIELDS: &[FieldDescriptor] = &[
    field!(R142 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R143 "Revocation Reason" 0x0042_0081 0x011 1 1 RevocationReason),
    field!(R144 "Compromise Occurrence Date" 0x0042_0021 0x100 0 1 none),
];

static RNGRETRIEVE_FIELDS: &[FieldDescriptor] =
    &[field!(R166 "Data Length" 0x0042_00c4 0x002 1 1 none)];

static RNGSEED_FIELDS: &[FieldDescriptor] = &[field!(R167 "Data" 0x0042_00c2 0x092 1 1 none)];

static SETATTRIBUTE_FIELDS: &[FieldDescriptor] = &[
    field!(R168 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R169 "New Attribute" 0x0042_013d 0x001 1 1 NewAttribute),
];

static SETCONSTRAINTS_FIELDS: &[FieldDescriptor] =
    &[field!(R170 "Constraints" 0x0042_0168 0x001 1 1 Constraints)];

static SETDEFAULTS_FIELDS: &[FieldDescriptor] =
    &[field!(R171 "Defaults Information" 0x0042_0152 0x001 0 1 DefaultsInformation)];

static SIGN_FIELDS: &[FieldDescriptor] = &[
    field!(R172 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R173 "Cryptographic Parameters" 0x0042_002b 0x001 0 1 CryptographicParameters),
    field!(R174 "Data" 0x0042_00c2 0x092 0 1 none),
    field!(R175 "Digested Data" 0x0042_0107 0x080 0 1 none),
    field!(R176 "Correlation Value" 0x0042_00d6 0x080 0 1 none),
    field!(R177 "Init Indicator" 0x0042_00d7 0x020 0 1 none),
    field!(R178 "Final Indicator" 0x0042_00d8 0x020 0 1 none),
];

static SIGNATUREVERIFY_FIELDS: &[FieldDescriptor] = &[
    field!(R179 "Unique Identifier" 0x0042_0094 0x052 0 1 none),
    field!(R180 "Cryptographic Parameters" 0x0042_002b 0x001 0 1 CryptographicParameters),
    field!(R181 "Data" 0x0042_00c2 0x092 0 1 none),
    field!(R182 "Digested Data" 0x0042_0107 0x080 0 1 none),
    field!(R183 "Signature Data" 0x0042_00c3 0x080 0 1 none),
    field!(R184 "Correlation Value" 0x0042_00d6 0x080 0 1 none),
    field!(R185 "Init Indicator" 0x0042_00d7 0x020 0 1 none),
    field!(R186 "Final Indicator" 0x0042_00d8 0x020 0 1 none),
];

static VALIDATE_FIELDS: &[FieldDescriptor] = &[
    field!(R187 "Certificate" 0x0042_0013 0x001 0 many Certificate),
    field!(R188 "Unique Identifier" 0x0042_0094 0x052 0 many none),
    field!(R189 "Validity Date" 0x0042_009a 0x100 0 1 none),
];
