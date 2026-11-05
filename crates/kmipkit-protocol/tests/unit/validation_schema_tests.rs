use kmipkit_ttlv::{Item, RawTag, Structure, Value};

use crate::{
    RequestPayloadSchema, ValidationSchema, validate_structure,
};

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

    let report = validate_structure(structure.view(), None);

    assert!(report.is_valid());
    assert!(report.issues().is_empty());
}

#[test]
fn create_payload_reports_missing_required_fields_with_stable_source_ids() {
    let report = validate_structure(Structure::new().view(), Some(create_schema()));

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

    let report = validate_structure(reversed.view(), Some(create_schema()));

    assert!(!report.is_valid());
    assert!(report
        .issues()
        .iter()
        .any(|issue| issue.code().as_str() == "KMIPKIT-0051-R019"));
    assert!(report
        .issues()
        .iter()
        .any(|issue| issue.path().to_string() == "Request Payload.Object Type"));
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

    let report = validate_structure(structure.view(), Some(create_schema()));

    assert!(report.is_valid(), "unknown extension alone is not an error");
    assert!(report.issues().is_empty());
}

#[test]
fn unsupported_schema_identity_is_reported_without_echoing_identity_data() {
    let report = validate_structure(
        Structure::new().view(),
        Some(ValidationSchema::Unsupported("vendor secret schema name")),
    );

    assert!(!report.is_valid());
    assert_eq!(report.issues().len(), 1);
    assert_eq!(
        report.issues()[0].code().as_str(),
        "KMIPKIT-0051-P002"
    );
    assert!(!format!("{report:?}").contains("vendor secret schema name"));
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
        structure.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::DiscoverVersions,
        )),
    );

    assert!(report.is_valid());
}
