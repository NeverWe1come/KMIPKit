use kmipkit_ttlv::{Item, RawTag, Structure, Value};

use crate::{NestedStructureSchema, RequestPayloadSchema, ValidationSchema, validate_structure};

const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const VENDOR_EXTENSION: u32 = 0x0054_0001;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const DATA: u32 = 0x0042_00C2;
const KEY_WRAPPING_SPECIFICATION: u32 = 0x0042_0047;
const WRAPPING_METHOD: u32 = 0x0042_009E;
const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const MAC_SIGNATURE_KEY_INFORMATION: u32 = 0x0042_004E;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;
const USAGE_LIMITS: u32 = 0x0042_0095;
const USAGE_LIMITS_TOTAL: u32 = 0x0042_0097;
const RIGHT: u32 = 0x0042_0150;
const OBJECTS: u32 = 0x0042_014E;
const OPERATIONS: u32 = 0x0042_014F;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;
const CURRENT_ATTRIBUTE: u32 = 0x0042_013C;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_REQUEST: u32 = 0x0042_0018;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const CERTIFICATE_VALUE: u32 = 0x0042_001E;
const CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const ASYNC_CORRELATION_VALUES: u32 = 0x0042_0176;
const ASYNC_CORRELATION_VALUE: u32 = 0x0042_0006;
const REVOCATION_REASON: u32 = 0x0042_0081;
const REVOCATION_REASON_CODE: u32 = 0x0042_0082;
const PKCS11_INTERFACE: u32 = 0x0042_0159;
const PKCS11_FUNCTION: u32 = 0x0042_015A;
const PKCS11_INPUT_PARAMETERS: u32 = 0x0042_015B;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const OPAQUE_OBJECT: u32 = 0x0042_005B;
const OPAQUE_DATA_TYPE: u32 = 0x0042_0059;

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
    let salt_issue = report
        .issues()
        .iter()
        .find(|issue| issue.path().to_string() == "Request Payload.Derivation Parameters.Salt")
        .expect("malformed Salt is reported at its schema path");
    assert_eq!(
        salt_issue.source().to_string(),
        "KMIP v2.1 §7.12, Table 364"
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
fn derive_key_schema_requires_hashing_algorithm_for_hash_method() {
    let derivation_parameters = structure([item(
        CRYPTOGRAPHIC_PARAMETERS,
        Value::structure(Structure::new()),
    )]);
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

    assert!(report.issues().iter().any(|issue| {
        issue.path().to_string()
            == "Request Payload.Derivation Parameters.Cryptographic Parameters.Hashing Algorithm"
    }));
}

#[test]
fn derivation_parameters_preserve_repeated_data_and_unknown_extensions() {
    let parameters = structure([
        item(0x0042_0030, Value::byte_string(b"first".to_vec())),
        item(0x0042_0030, Value::byte_string(b"second".to_vec())),
        item(VENDOR_EXTENSION, Value::text_string("opaque".to_owned())),
    ]);

    let report = validate_structure(
        &parameters.view(),
        Some(ValidationSchema::Nested(
            NestedStructureSchema::DerivationParameters,
        )),
    );

    assert!(report.is_valid());
    let view = parameters.view();
    assert_eq!(view.children().len(), 3);
    assert_eq!(view.children()[0].tag().raw(), 0x0042_0030);
    assert_eq!(view.children()[1].tag().raw(), 0x0042_0030);
    assert_eq!(view.children()[2].tag().raw(), VENDOR_EXTENSION);
}

#[test]
fn get_payload_recursively_validates_key_wrapping_specification() {
    let key_information = structure([item(
        UNIQUE_IDENTIFIER,
        Value::text_string("wrapping-key".to_owned()),
    )]);
    let wrapping = structure([
        item(
            WRAPPING_METHOD,
            Value::text_string("invalid-type".to_owned()),
        ),
        item(
            ENCRYPTION_KEY_INFORMATION,
            Value::structure(key_information),
        ),
    ]);
    let payload = structure([item(KEY_WRAPPING_SPECIFICATION, Value::structure(wrapping))]);

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(RequestPayloadSchema::Get)),
    );

    let issue = report
        .issues()
        .iter()
        .find(|issue| {
            issue.path().to_string() == "Request Payload.Key Wrapping Specification.Wrapping Method"
        })
        .expect("wrong Wrapping Method TTLV type is reported recursively");
    assert_eq!(issue.source().to_string(), "KMIP v2.1 §7.18, Table 370");
}

#[test]
fn key_wrapping_specification_requires_key_information_but_allows_both_forms() {
    let missing_information = structure([item(WRAPPING_METHOD, Value::enumeration(1))]);
    let missing_report = validate_structure(
        &missing_information.view(),
        Some(ValidationSchema::Nested(
            NestedStructureSchema::KeyWrappingSpecification,
        )),
    );
    assert!(missing_report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R236"
            && issue.path().to_string() == "Encryption Key Information"
    }));

    let encryption_information = structure([item(
        UNIQUE_IDENTIFIER,
        Value::text_string("encryption-key".to_owned()),
    )]);
    let mac_information = structure([item(
        UNIQUE_IDENTIFIER,
        Value::text_string("mac-key".to_owned()),
    )]);
    let wrapping = structure([
        item(WRAPPING_METHOD, Value::enumeration(1)),
        item(
            ENCRYPTION_KEY_INFORMATION,
            Value::structure(encryption_information),
        ),
        item(
            MAC_SIGNATURE_KEY_INFORMATION,
            Value::structure(mac_information),
        ),
        item(ATTRIBUTE_NAME, Value::text_string("Name".to_owned())),
        item(ATTRIBUTE_NAME, Value::text_string("Comment".to_owned())),
    ]);
    let both_report = validate_structure(
        &wrapping.view(),
        Some(ValidationSchema::Nested(
            NestedStructureSchema::KeyWrappingSpecification,
        )),
    );
    assert!(
        both_report.is_valid(),
        "both key-information alternatives are allowed"
    );
}

#[test]
fn key_information_requires_text_string_unique_identifier() {
    let information = structure([item(UNIQUE_IDENTIFIER, Value::enumeration(7))]);
    let report = validate_structure(
        &information.view(),
        Some(ValidationSchema::Nested(
            NestedStructureSchema::EncryptionKeyInformation,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.path().to_string() == "Unique Identifier"
            && issue.source().to_string() == "KMIP v2.1 §3.3, Table 16"
    }));
}

#[test]
fn rights_schema_recursively_validates_usage_limits_and_preserves_empty_rights() {
    let malformed_usage_limits = structure([item(USAGE_LIMITS_TOTAL, Value::integer(2))]);
    let right = structure([item(USAGE_LIMITS, Value::structure(malformed_usage_limits))]);
    let rights = structure([item(RIGHT, Value::structure(right))]);
    let payload = structure([
        item(0x0042_014A, Value::enumeration(1)),
        item(0x0042_014B, Value::byte_string(b"ticket".to_vec())),
        item(0x0042_014D, Value::structure(rights)),
    ]);

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::DelegatedLogin,
        )),
    );

    assert!(
        report.issues().iter().any(|issue| {
            issue.path().to_string()
                == "Request Payload.Rights.Right[0].Usage Limits.Usage Limits Total"
                && issue.source().to_string() == "KMIP v2.1 §7.40, Table 392"
        }),
        "unexpected validation report: {report:?}"
    );
}

#[test]
fn rights_schema_preserves_empty_and_repeated_rights_and_unknown_extensions() {
    let empty_rights = structure([]);
    let empty_payload = structure([
        item(0x0042_014A, Value::enumeration(1)),
        item(0x0042_014B, Value::byte_string(b"ticket".to_vec())),
        item(0x0042_014D, Value::structure(empty_rights)),
    ]);
    let empty_report = validate_structure(
        &empty_payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::DelegatedLogin,
        )),
    );
    assert!(empty_report.is_valid());

    let empty_right = structure([]);
    let populated_right = structure([
        item(
            OPERATIONS,
            Value::structure(structure([item(
                0x0042_005C,
                Value::enumeration(0x8000_0001),
            )])),
        ),
        item(
            OBJECTS,
            Value::structure(structure([item(
                0x0042_0094,
                Value::text_string("object-id".to_owned()),
            )])),
        ),
        item(VENDOR_EXTENSION, Value::byte_string(vec![0xCA, 0xFE])),
    ]);
    let rights = structure([
        item(RIGHT, Value::structure(empty_right)),
        item(RIGHT, Value::structure(populated_right)),
        item(VENDOR_EXTENSION, Value::byte_string(vec![0xBE, 0xEF])),
    ]);
    let payload = structure([
        item(0x0042_014A, Value::enumeration(1)),
        item(0x0042_014B, Value::byte_string(b"ticket".to_vec())),
        item(0x0042_014D, Value::structure(rights)),
    ]);

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::DelegatedLogin,
        )),
    );

    assert!(report.is_valid());
    assert_eq!(payload.view().children().len(), 3);
}

#[test]
fn attribute_wrappers_require_exactly_one_attribute_item() {
    let payload = structure([item(NEW_ATTRIBUTE, Value::structure(structure([])))]);
    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::AddAttribute,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R247"
            && issue.path().to_string() == "Request Payload.New Attribute.Attribute"
            && issue.source().to_string() == "KMIP v2.1 §5.7, Table 163"
    }));
}

#[test]
fn current_attribute_wrapper_rejects_multiple_attribute_items() {
    let current_attribute = structure([
        item(0x0042_00C5, Value::boolean(true)),
        item(HASHING_ALGORITHM, Value::enumeration(1)),
    ]);
    let payload = structure([item(CURRENT_ATTRIBUTE, Value::structure(current_attribute))]);
    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::DeleteAttribute,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R248"
            && issue.path().to_string() == "Request Payload.Current Attribute.Attribute"
            && issue.source().to_string() == "KMIP v2.1 §5.6, Table 162"
    }));
}

#[test]
fn validate_request_recursively_checks_certificate_structure_without_parsing_der() {
    let certificate = structure([
        item(CERTIFICATE_TYPE, Value::integer(1)),
        item(CERTIFICATE_VALUE, Value::byte_string(vec![0x30, 0x00])),
    ]);
    let payload = structure([item(CERTIFICATE, Value::structure(certificate))]);
    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Validate,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R249"
            && issue.path().to_string() == "Request Payload.Certificate[0].Certificate Type"
            && issue.source().to_string() == "KMIP v2.1 §2.1, Table 2"
    }));
}

#[test]
fn reprovision_request_checks_certificate_request_members() {
    let certificate_request = structure([item(
        CERTIFICATE_REQUEST_VALUE,
        Value::byte_string(vec![0x30, 0x00]),
    )]);
    let payload = structure([item(
        CERTIFICATE_REQUEST,
        Value::structure(certificate_request),
    )]);
    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::ReProvision,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R251"
            && issue.path().to_string()
                == "Request Payload.Certificate Request.Certificate Request Type"
            && issue.source().to_string() == "KMIP v2.1 §2.2, Table 3"
    }));
}

#[test]
fn discover_versions_recursively_checks_protocol_version_members() {
    let version = structure([item(PROTOCOL_VERSION_MAJOR, Value::integer(2))]);
    let payload = structure([item(PROTOCOL_VERSION, Value::structure(version))]);
    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::DiscoverVersions,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R256"
            && issue.path().to_string()
                == "Request Payload.Protocol Version[0].Protocol Version Minor"
            && issue.source().to_string() == "KMIP v2.1 §9.16, Table 421"
    }));

    let valid_version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let valid_payload = structure([item(PROTOCOL_VERSION, Value::structure(valid_version))]);
    assert!(
        validate_structure(
            &valid_payload.view(),
            Some(ValidationSchema::RequestPayload(
                RequestPayloadSchema::DiscoverVersions,
            )),
        )
        .is_valid()
    );
}

#[test]
fn logout_recursively_checks_ticket_value() {
    let ticket = structure([item(TICKET_TYPE, Value::enumeration(1))]);
    let payload = structure([item(TICKET, Value::structure(ticket))]);
    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Logout,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R254"
            && issue.path().to_string() == "Request Payload.Ticket.Ticket Value"
            && issue.source().to_string() == "KMIP v2.1 §7.39, Table 391"
    }));

    let valid_ticket = structure([
        item(TICKET_TYPE, Value::enumeration(0x8000_0001)),
        item(TICKET_VALUE, Value::byte_string(vec![0xCA, 0xFE])),
    ]);
    let valid_payload = structure([item(TICKET, Value::structure(valid_ticket))]);
    assert!(
        validate_structure(
            &valid_payload.view(),
            Some(ValidationSchema::RequestPayload(
                RequestPayloadSchema::Logout
            )),
        )
        .is_valid()
    );
}

#[test]
fn query_async_requests_recursively_checks_correlation_value_type() {
    let values = structure([item(ASYNC_CORRELATION_VALUE, Value::integer(7))]);
    let payload = structure([item(ASYNC_CORRELATION_VALUES, Value::structure(values))]);
    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::QueryAsyncRequests,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R259"
            && issue.path().to_string()
                == "Request Payload.Asynchronous Correlation Values.Asynchronous Correlation Value[0]"
            && issue.source().to_string() == "KMIP v2.1 §7.1, Table 352"
    }));
}

#[test]
fn revoke_recursively_checks_revocation_reason_code_type() {
    let reason = structure([item(
        REVOCATION_REASON_CODE,
        Value::text_string("invalid".to_owned()),
    )]);
    let payload = structure([item(REVOCATION_REASON, Value::structure(reason))]);
    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Revoke,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R257"
            && issue.path().to_string()
                == "Request Payload.Revocation Reason.Revocation Reason Code"
            && issue.source().to_string() == "KMIP v2.1 §4.47, Table 123"
    }));
}

#[test]
fn query_request_recursively_checks_object_group_value_type() {
    let groups = structure([item(0x0042_0056, Value::integer(7))]);
    let payload = structure([
        item(0x0042_0074, Value::enumeration(1)),
        item(0x0042_0166, Value::structure(groups)),
    ]);
    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Query,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R196"
            && issue.path().to_string() == "Request Payload.Object Groups.Object Group[0]"
            && issue.source().to_string() == "KMIP v2.1 §7.23, Table 375"
    }));
}

#[test]
fn query_async_requests_recursively_checks_operation_value_type() {
    let operations = structure([item(0x0042_005C, Value::integer(7))]);
    let payload = structure([item(OPERATIONS, Value::structure(operations))]);
    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::QueryAsyncRequests,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R246"
            && issue.path().to_string() == "Request Payload.Operations.Operation[0]"
            && issue.source().to_string() == "KMIP v2.1 §7.25, Table 377"
    }));
}

#[test]
fn pkcs11_request_schema_recursively_checks_interface_structure() {
    let payload = structure([
        item(
            PKCS11_INTERFACE,
            Value::structure(structure([item(PKCS11_INTERFACE, Value::integer(1))])),
        ),
        item(
            PKCS11_FUNCTION,
            Value::structure(structure([item(PKCS11_FUNCTION, Value::enumeration(1))])),
        ),
    ]);

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Pkcs11,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R260"
            && issue.path().to_string() == "Request Payload.PKCS#11 Interface.PKCS#11 Interface"
            && issue.source().to_string() == "KMIP v2.1 §7.28, Table 380"
    }));
}

#[test]
fn pkcs11_request_schema_recursively_checks_input_parameter_structure() {
    let payload = structure([
        item(
            PKCS11_FUNCTION,
            Value::structure(structure([item(PKCS11_FUNCTION, Value::enumeration(1))])),
        ),
        item(
            PKCS11_INPUT_PARAMETERS,
            Value::structure(structure([item(
                PKCS11_INPUT_PARAMETERS,
                Value::enumeration(1),
            )])),
        ),
    ]);

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Pkcs11,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R261"
            && issue.path().to_string()
                == "Request Payload.PKCS#11 Input Parameters.PKCS#11 Input Parameters"
            && issue.source().to_string() == "KMIP v2.1 §7.27, Table 379"
    }));
}

#[test]
fn register_schema_recursively_checks_key_block_fields() {
    let key_block = structure([item(
        KEY_FORMAT_TYPE,
        Value::text_string("invalid-key-format-ttlv-type".to_owned()),
    )]);
    let symmetric_key = structure([item(KEY_BLOCK, Value::structure(key_block))]);
    let payload = structure([
        item(OBJECT_TYPE, Value::enumeration(2)),
        item(ATTRIBUTES, Value::structure(Structure::new())),
        item(SYMMETRIC_KEY, Value::structure(symmetric_key)),
    ]);

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Register,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R262"
            && issue.path().to_string()
                == "Request Payload.Any Object (Section 2).Key Block.Key Format Type"
            && issue.source().to_string() == "KMIP v2.1 §3.1, Table 12"
    }));
    assert!(!format!("{report:?}").contains("invalid-key-format-ttlv-type"));
}

#[test]
fn register_schema_checks_both_required_opaque_object_fields() {
    let opaque_object = structure([item(OPAQUE_DATA_TYPE, Value::enumeration(0x8000_0001))]);
    let payload = structure([
        item(OBJECT_TYPE, Value::enumeration(0x8000_0001)),
        item(ATTRIBUTES, Value::structure(Structure::new())),
        item(OPAQUE_OBJECT, Value::structure(opaque_object)),
    ]);

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Register,
        )),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R270"
            && issue.path().to_string()
                == "Request Payload.Any Object (Section 2).Opaque Data Value"
            && issue.source().to_string() == "KMIP v2.1 §2.3, Table 4"
    }));
}

#[test]
fn key_block_requires_length_when_algorithm_is_present() {
    let key_block = structure([
        item(KEY_FORMAT_TYPE, Value::enumeration(0x8000_0001)),
        item(0x0042_0028, Value::enumeration(0x8000_0002)),
    ]);

    let report = validate_structure(
        &key_block.view(),
        Some(ValidationSchema::Nested(NestedStructureSchema::KeyBlock)),
    );

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R266"
            && issue.path().to_string() == "Cryptographic Length"
            && issue.source().to_string() == "KMIP v2.1 §3.1, Table 12"
    }));
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
