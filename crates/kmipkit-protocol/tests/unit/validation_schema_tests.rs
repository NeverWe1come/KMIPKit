use kmipkit_ttlv::{Item, RawTag, Structure, Value};

use crate::{NestedStructureSchema, RequestPayloadSchema, ValidationSchema, validate_structure};

const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const VENDOR_EXTENSION: u32 = 0x0054_0001;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const DATA: u32 = 0x0042_00C2;

fn item(tag: u32, value: Value) -> Item {
    let raw_tag = RawTag::new(tag).expect("fixture tag fits the TTLV tag field");
    let checked_tag = raw_tag.try_checked().expect("fixture tag is allocated");
    Item::new(checked_tag, value).expect("fixture item is valid in the generic model")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("test structure stays within generic model limits");
    }
    structure
}

fn create_schema() -> ValidationSchema {
    ValidationSchema::RequestPayload(RequestPayloadSchema::Create)
}

#[test]
fn encrypt_payload_recursively_validates_known_cryptographic_parameter_fields() {
    let parameters = structure([item(
        HASHING_ALGORITHM,
        Value::byte_string(b"invalid-ttlv-type".to_vec()),
    )]);
    let payload = structure([
        item(CRYPTOGRAPHIC_PARAMETERS, Value::structure(parameters)),
        item(DATA, Value::byte_string(b"input".to_vec())),
    ]);

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Encrypt,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.path().to_string() == "Request Payload.Cryptographic Parameters.Hashing Algorithm"
    }));
    assert!(
        report
            .issues()
            .iter()
            .all(|issue| { issue.source().to_string() == "KMIP v2.1 §4.16, Table 59" })
    );
}

#[test]
fn cryptographic_parameters_require_iv_and_gcm_tag_lengths_when_mode_requires_them() {
    let parameters = structure([item(0x0042_0011, Value::enumeration(9))]);

    let report = validate_structure(
        &parameters.view(),
        Some(ValidationSchema::Nested(
            NestedStructureSchema::CryptographicParameters,
        )),
    );

    let paths = report
        .issues()
        .iter()
        .map(|issue| issue.path().to_string())
        .collect::<Vec<_>>();
    assert_eq!(paths, ["IV Length", "Tag Length"]);
    assert!(
        report
            .issues()
            .iter()
            .all(|issue| { issue.source().to_string() == "KMIP v2.1 §4.16, Table 59" })
    );
}

#[test]
fn cryptographic_parameters_preserve_unknown_members_and_future_enumerations() {
    let parameters = structure([
        item(HASHING_ALGORITHM, Value::enumeration(0x8000_0042)),
        item(VENDOR_EXTENSION, Value::byte_string(vec![0xCA, 0xFE])),
    ]);

    let report = validate_structure(
        &parameters.view(),
        Some(ValidationSchema::Nested(
            NestedStructureSchema::CryptographicParameters,
        )),
    );

    assert!(report.is_valid());
    let view = parameters.view();
    let children = view.children();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].tag().raw(), HASHING_ALGORITHM);
    assert_eq!(children[1].tag().raw(), VENDOR_EXTENSION);
}

#[test]
fn derive_key_payload_recursively_validates_derivation_parameters() {
    let derivation_parameters = structure([item(0x0042_0084, Value::integer(9))]);
    let payload = structure([
        item(OBJECT_TYPE, Value::enumeration(2)),
        item(0x0042_0094, Value::text_string("derivation-key".to_owned())),
        item(0x0042_0031, Value::enumeration(2)),
        item(0x0042_0032, Value::structure(derivation_parameters)),
        item(ATTRIBUTES, Value::structure(Structure::new())),
    ]);

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::DeriveKey,
        )),
    );

    assert!(
        report.issues().iter().any(|issue| {
            issue.path().to_string() == "Request Payload.Derivation Parameters.Salt"
        })
    );
    assert!(
        report
            .issues()
            .iter()
            .all(|issue| { issue.source().to_string() == "KMIP v2.1 §7.12, Table 364" })
    );
}

#[test]
fn derive_key_schema_applies_pbkdf2_parameter_requirements_from_outer_method() {
    let payload = structure([
        item(OBJECT_TYPE, Value::enumeration(2)),
        item(0x0042_0094, Value::text_string("derivation-key".to_owned())),
        item(0x0042_0031, Value::enumeration(1)),
        item(0x0042_0032, Value::structure(Structure::new())),
        item(ATTRIBUTES, Value::structure(Structure::new())),
    ]);

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::DeriveKey,
        )),
    );

    let paths = report
        .issues()
        .iter()
        .map(|issue| issue.path().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            "Request Payload.Derivation Parameters.Salt",
            "Request Payload.Derivation Parameters.Iteration Count",
        ]
    );
}

#[test]
fn generic_structure_without_identity_does_not_guess_an_operation_schema() {
    let mut structure = Structure::new();
    structure
        .try_push(item(VENDOR_EXTENSION, Value::byte_string(vec![0xCA, 0xFE])))
        .expect("vendor extension fits the generic structure limit");

    let report = validate_structure(&structure.view(), None);

    assert!(report.is_valid());
    assert_eq!(report.issues(), []);
}

#[test]
fn create_payload_reports_missing_required_fields_with_stable_source_ids() {
    let report = validate_structure(&Structure::new().view(), Some(create_schema()));

    assert!(!report.is_valid());
    let codes = report
        .issues()
        .iter()
        .map(|issue| issue.code().as_str())
        .collect::<Vec<_>>();
    assert_eq!(codes, ["KMIPKIT-0051-R019", "KMIPKIT-0051-R020"]);
    assert_eq!(
        report.issues()[0].path().to_string(),
        "Request Payload.Object Type"
    );
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.8, Table 186"
    );
}

#[test]
fn create_payload_checks_type_and_oasis_field_order_without_sorting_tags() {
    let mut reversed = Structure::new();
    reversed
        .try_push(item(ATTRIBUTES, Value::structure(Structure::new())))
        .expect("attributes fixture is valid");
    reversed
        .try_push(item(OBJECT_TYPE, Value::integer(1)))
        .expect("wrong schema type is still a valid generic TTLV value");

    let report = validate_structure(&reversed.view(), Some(create_schema()));

    assert!(!report.is_valid());
    assert!(
        report
            .issues()
            .iter()
            .any(|issue| issue.code().as_str() == "KMIPKIT-0051-R019")
    );
    assert!(
        report
            .issues()
            .iter()
            .any(|issue| issue.path().to_string() == "Request Payload.Object Type")
    );
}

#[test]
fn create_payload_rejects_wrong_ttlv_type_without_reporting_the_value() {
    let mut structure = Structure::new();
    structure
        .try_push(item(OBJECT_TYPE, Value::integer(1)))
        .expect("wrong schema type is valid in generic TTLV");
    structure
        .try_push(item(ATTRIBUTES, Value::structure(Structure::new())))
        .expect("attributes fixture is valid");

    let report = validate_structure(&structure.view(), Some(create_schema()));
    let issue = report
        .issues()
        .first()
        .expect("the Object Type item has the wrong TTLV type");

    assert_eq!(issue.code().as_str(), "KMIPKIT-0051-R019");
    assert_eq!(issue.path().to_string(), "Request Payload.Object Type");
    assert_eq!(issue.source().to_string(), "KMIP v2.1 §6.1.8, Table 186");
    assert!(!format!("{report:?}").contains("raw-ttlv"));
}

#[test]
fn create_payload_rejects_duplicate_singleton_fields() {
    let mut structure = Structure::new();
    for object_type in [2, 3] {
        structure
            .try_push(item(OBJECT_TYPE, Value::enumeration(object_type)))
            .expect("object type fixture is valid");
    }
    structure
        .try_push(item(ATTRIBUTES, Value::structure(Structure::new())))
        .expect("attributes fixture is valid");

    let report = validate_structure(&structure.view(), Some(create_schema()));

    assert!(!report.is_valid());
    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R019"
            && issue.path().to_string() == "Request Payload.Object Type[1]"
    }));
}

#[test]
fn every_supported_request_payload_has_an_explicit_schema_identity() {
    let schemas = [
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
        RequestPayloadSchema::Recertify,
        RequestPayloadSchema::Recover,
        RequestPayloadSchema::Register,
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

    assert_eq!(schemas.len(), 56);
    for schema in schemas {
        let report = validate_structure(
            &Structure::new().view(),
            Some(ValidationSchema::RequestPayload(schema)),
        );
        assert!(
            report
                .issues()
                .iter()
                .all(|issue| issue.code().as_str() != "KMIPKIT-0051-P002"),
            "the supported schema descriptor exists for {schema:?}"
        );
    }
}

#[test]
fn create_payload_preserves_unknown_vendor_fields_without_rejecting_them() {
    let mut structure = Structure::new();
    structure
        .try_push(item(OBJECT_TYPE, Value::enumeration(2)))
        .expect("object type fixture is valid");
    structure
        .try_push(item(ATTRIBUTES, Value::structure(Structure::new())))
        .expect("attributes fixture is valid");
    structure
        .try_push(item(VENDOR_EXTENSION, Value::byte_string(vec![0xCA, 0xFE])))
        .expect("extension fixture is valid");

    let report = validate_structure(&structure.view(), Some(create_schema()));

    assert!(report.is_valid(), "unknown extension alone is not an error");
    assert_eq!(report.issues(), []);
}

#[test]
fn unsupported_schema_identity_is_reported_without_echoing_identity_data() {
    let report = validate_structure(
        &Structure::new().view(),
        Some(ValidationSchema::Unsupported),
    );

    assert!(!report.is_valid());
    assert_eq!(report.issues().len(), 1);
    assert_eq!(report.issues()[0].code().as_str(), "KMIPKIT-0051-P002");
    assert!(!format!("{report:?}").contains("raw-ttlv"));
}

#[test]
fn repeated_protocol_versions_are_allowed_by_their_operation_schema() {
    let mut structure = Structure::new();
    for major in [1, 2] {
        let mut version = Structure::new();
        version
            .try_push(item(0x0042_0069, Value::integer(major)))
            .expect("version major is a valid integer field");
        version
            .try_push(item(0x0042_006A, Value::integer(0)))
            .expect("version minor is a valid integer field");
        structure
            .try_push(item(0x0042_0068, Value::structure(version)))
            .expect("each Protocol Version is a Structure");
    }

    let report = validate_structure(
        &structure.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::DiscoverVersions,
        )),
    );

    assert!(report.is_valid());
}

#[test]
fn nested_object_types_schema_checks_known_members_and_preserves_vendor_fields() {
    let mut malformed = Structure::new();
    malformed
        .try_push(item(OBJECT_TYPE, Value::integer(7)))
        .expect("a wrong schema type is valid generic TTLV");
    malformed
        .try_push(item(VENDOR_EXTENSION, Value::byte_string(vec![0xCA, 0xFE])))
        .expect("unknown vendor data remains valid generic TTLV");

    let report = validate_structure(
        &malformed.view(),
        Some(ValidationSchema::Nested(NestedStructureSchema::ObjectTypes)),
    );

    assert!(!report.is_valid());
    assert_eq!(report.issues()[0].path().to_string(), "Object Type[0]");
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §7.24, Table 376"
    );
    assert_eq!(
        report.issues().len(),
        1,
        "the unknown vendor child is retained"
    );
}

#[test]
fn nested_object_defaults_enforces_alternative_and_required_attributes() {
    let mut missing_both = Structure::new();
    missing_both
        .try_push(item(ATTRIBUTES, Value::structure(Structure::new())))
        .expect("Attributes is a valid structure");
    let missing_alternative = validate_structure(
        &missing_both.view(),
        Some(ValidationSchema::Nested(
            NestedStructureSchema::ObjectDefaults,
        )),
    );
    assert!(!missing_alternative.is_valid());
    assert_eq!(
        missing_alternative.issues()[0].source().to_string(),
        "KMIP v2.1 §7.22, Table 374"
    );

    let mut valid = Structure::new();
    valid
        .try_push(item(OBJECT_TYPE, Value::enumeration(2)))
        .expect("Object Type is a valid Enumeration");
    valid
        .try_push(item(ATTRIBUTES, Value::structure(Structure::new())))
        .expect("Attributes is a valid structure");
    assert!(
        validate_structure(
            &valid.view(),
            Some(ValidationSchema::Nested(
                NestedStructureSchema::ObjectDefaults,
            )),
        )
        .is_valid()
    );
}

#[test]
fn request_payload_schema_recurses_into_known_nested_payloads() {
    const DEFAULTS_INFORMATION: u32 = 0x0042_0152;
    const OBJECT_DEFAULTS: u32 = 0x0042_0153;

    let mut object_defaults = Structure::new();
    object_defaults
        .try_push(item(OBJECT_TYPE, Value::enumeration(2)))
        .expect("Object Type is a valid Enumeration");
    let mut defaults_information = Structure::new();
    defaults_information
        .try_push(item(OBJECT_DEFAULTS, Value::structure(object_defaults)))
        .expect("Object Defaults is a valid Structure");
    let mut request_payload = Structure::new();
    request_payload
        .try_push(item(
            DEFAULTS_INFORMATION,
            Value::structure(defaults_information),
        ))
        .expect("Defaults Information is a valid Structure");

    let report = validate_structure(
        &request_payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::SetDefaults,
        )),
    );

    assert!(!report.is_valid());
    assert!(report.issues().iter().any(|issue| {
        issue.path().to_string()
            == "Request Payload.Defaults Information.Object Defaults[0].Attributes"
            && issue.source().to_string() == "KMIP v2.1 §7.22, Table 374"
    }));
}

#[test]
fn constraints_schema_recurses_through_known_children_and_keeps_extensions() {
    const CONSTRAINT: u32 = 0x0042_0169;
    const OBJECT_TYPES: u32 = 0x0042_0167;
    const OBJECT_GROUPS: u32 = 0x0042_0166;
    const OBJECT_GROUP: u32 = 0x0042_0056;

    let mut object_types = Structure::new();
    object_types
        .try_push(item(OBJECT_TYPE, Value::enumeration(0xF123_4567)))
        .expect("future Object Type values remain valid Enumeration values");
    let mut object_groups = Structure::new();
    object_groups
        .try_push(item(
            OBJECT_GROUP,
            Value::text_string(String::from("tenant-a")),
        ))
        .expect("Object Group is a Text String");
    let mut attributes = Structure::new();
    attributes
        .try_push(item(VENDOR_EXTENSION, Value::byte_string(vec![0xCA, 0xFE])))
        .expect("vendor attribute remains valid generic TTLV");
    let mut constraint = Structure::new();
    constraint
        .try_push(item(OBJECT_TYPES, Value::structure(object_types)))
        .expect("Object Types is a Structure");
    constraint
        .try_push(item(OBJECT_GROUPS, Value::structure(object_groups)))
        .expect("Object Groups is a Structure");
    constraint
        .try_push(item(ATTRIBUTES, Value::structure(attributes)))
        .expect("Attributes is a Structure");
    constraint
        .try_push(item(VENDOR_EXTENSION, Value::byte_string(vec![0xBA, 0xBE])))
        .expect("unknown Constraint children are preserved");
    let mut constraints = Structure::new();
    constraints
        .try_push(item(CONSTRAINT, Value::structure(constraint)))
        .expect("Constraint is a Structure");

    let report = validate_structure(
        &constraints.view(),
        Some(ValidationSchema::Nested(NestedStructureSchema::Constraints)),
    );

    assert!(
        report.is_valid(),
        "unknown tags and future values stay open"
    );
    assert_eq!(report.issues(), []);
}

#[test]
fn client_initiated_set_endpoint_role_is_explicit_but_deferred() {
    let report = validate_structure(
        &Structure::new().view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::SetEndpointRole,
        )),
    );

    assert!(!report.is_valid());
    assert_eq!(report.issues().len(), 1);
    assert_eq!(report.issues()[0].code().as_str(), "KMIPKIT-0051-P002");
}
