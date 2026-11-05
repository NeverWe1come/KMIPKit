use kmipkit_ttlv::{Item, RawTag, Structure, Value};

use crate::{RequestPayloadSchema, ValidationSchema, validate_structure};

const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const VENDOR_EXTENSION: u32 = 0x0054_0001;

fn item(tag: u32, value: Value) -> Item {
    let raw_tag = RawTag::new(tag).expect("fixture tag fits the TTLV tag field");
    let checked_tag = raw_tag.try_checked().expect("fixture tag is allocated");
    Item::new(checked_tag, value).expect("fixture item is valid in the generic model")
}

fn create_schema() -> ValidationSchema {
    ValidationSchema::RequestPayload(RequestPayloadSchema::Create)
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
