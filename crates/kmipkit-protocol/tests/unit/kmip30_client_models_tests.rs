//! Payload tests for the KMIP 3.0 client operations in KMIPKIT-0050.

use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

use crate::{
    AttributeSet, Certificate, CreateCredentialObject, CreateCredentialRequest,
    CreateCredentialResponse, CreateGroupRequest, CreateGroupResponse, CreateUserRequest,
    CreateUserResponse, CredentialType, DeactivateRequest, DecapsulateRequest,
    DeviceCredentialObject, DeviceCredentialObjectFields, EncapsulateRequest,
    HashedPasswordCredentialObject, HashedPasswordCredentialObjectFields, ObliterateRequest,
    ObliterateResponse, OneTimePasswordCredentialObject, OneTimePasswordCredentialObjectFields,
    PasswordCredentialObject, PasswordCredentialObjectFields, SecretBytes, SecretText,
    UniqueIdentifier,
};

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const DATA: u32 = 0x0042_00C2;
const ATTRIBUTES: u32 = 0x0042_0125;
const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CERTIFICATE: u32 = 0x0042_0013;
const PASSWORD_CREDENTIAL: u32 = 0x0042_01A1;
const DEVICE_CREDENTIAL: u32 = 0x0042_01A6;
const OTP_CREDENTIAL: u32 = 0x0042_01A7;
const HASHED_PASSWORD_CREDENTIAL: u32 = 0x0042_01AF;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const DEACTIVATION_REASON: u32 = 0x0042_01B8;
const DEACTIVATION_DATE: u32 = 0x0042_002F;
const OPERATION: u32 = 0x0042_005C;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_STATUS: u32 = 0x0042_007F;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("test tag is valid")
            .try_checked()
            .expect("test tag is checked"),
        value,
    )
    .expect("test item is valid")
}

fn payload_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure.try_push(child).expect("test structure is valid");
    }
    structure
}

fn successful_response(operation: u32, payload: Structure) -> crate::ResponseMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(3)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(0)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(TIME_STAMP, Value::date_time(1_700_000_000)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(OPERATION, Value::enumeration(operation)),
        item(RESULT_STATUS, Value::enumeration(0)),
        item(RESPONSE_PAYLOAD, Value::structure(payload)),
    ]);
    let message = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ]);
    crate::ResponseMessage::try_from_ttlv(message).expect("response fixture is valid")
}

fn assert_kem_success_payload_error(operation: u32, payload: Structure) {
    let message = successful_response(operation, payload);
    let item = message
        .batch_items()
        .next()
        .expect("one response batch item");
    let result = match operation {
        0x41 => crate::EncapsulateResponse::try_from_response_item(item).map(|_| ()),
        0x42 => crate::DecapsulateResponse::try_from_response_item(item).map(|_| ()),
        _ => panic!("test operation is a KEM operation"),
    };
    assert_eq!(
        result,
        Err(crate::Kmip30OperationError::MalformedSuccessPayload)
    );
}

fn pending_response(operation: u32) -> crate::ResponseMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(3)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(0)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(TIME_STAMP, Value::date_time(1_700_000_000)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(OPERATION, Value::enumeration(operation)),
        item(RESULT_STATUS, Value::enumeration(2)),
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(vec![0x01]),
        ),
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);
    let message = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ]);
    crate::ResponseMessage::try_from_ttlv(message).expect("pending response fixture is valid")
}

fn unknown_parameters() -> crate::CryptographicParameters {
    let mut structure = Structure::new();
    structure
        .try_push(item(0x0054_1234, Value::enumeration(0xDEAD)))
        .expect("test structure is valid");
    crate::CryptographicParameters::try_from_ttlv(structure)
        .expect("unknown parameter is preserved")
}

fn attributes_with_unknown_extension() -> AttributeSet {
    AttributeSet::try_new([item(0x0054_1234, Value::enumeration(0xC0DE))])
        .expect("allocated extension attribute is preserved")
}

fn depth_limited_extension_attribute() -> Item {
    let mut nested = Structure::new();
    for _ in 0..63 {
        let child = item(0x0054_1235, Value::structure(nested));
        let mut parent = Structure::new();
        parent
            .try_push(child)
            .expect("child remains under TTLV depth limit");
        nested = parent;
    }
    item(0x0054_1234, Value::structure(nested))
}

fn assert_credential_alternative_type(
    object: CreateCredentialObject,
    expected_type: CredentialType,
    expected_object_tag: u32,
    expected_debug: &str,
) {
    assert_eq!(format!("{object:?}"), expected_debug);
    let request = CreateCredentialRequest::new(AttributeSet::new(), object);
    let payload = request
        .into_ttlv_payload()
        .expect("valid credential payload");
    let tags = payload_tags(&payload);
    assert_eq!(tags.len(), 3);
    assert_eq!(tags, [CREDENTIAL_TYPE, ATTRIBUTES, expected_object_tag]);
    let encoded_type = payload.view().children()[0].with_value(|value| match value {
        ValueView::Enumeration(value) => *value,
        _ => panic!("Credential Type is encoded as an Enumeration"),
    });
    assert_eq!(encoded_type, expected_type.raw());
}

#[test]
fn create_credential_uses_table_277_order_and_redacts_credential_material() {
    let fields = PasswordCredentialObjectFields {
        password: Some(SecretText::new("secret-marker".to_owned())),
        ..PasswordCredentialObjectFields::default()
    };
    let password = PasswordCredentialObject::try_new(fields).expect("valid credential object");
    let request = CreateCredentialRequest::new(
        AttributeSet::new(),
        CreateCredentialObject::PasswordCredential(password),
    );

    let debug = format!("{request:?}");
    assert!(!debug.contains("secret-marker"));
    let payload = request.into_ttlv_payload().expect("valid request payload");
    assert_eq!(
        payload_tags(&payload),
        [CREDENTIAL_TYPE, ATTRIBUTES, PASSWORD_CREDENTIAL]
    );
    let encoded_credential_type = payload.view().children()[0].with_value(|value| match value {
        ValueView::Enumeration(value) => *value,
        _ => panic!("Credential Type is encoded as an Enumeration"),
    });
    assert_eq!(
        encoded_credential_type,
        CredentialType::UsernameAndPassword.raw()
    );
}

#[test]
fn create_credential_derives_all_types_from_its_closed_object_alternative() {
    assert_credential_alternative_type(
        CreateCredentialObject::Certificate(
            Certificate::try_new(1, vec![0x30, 0x00]).expect("valid certificate object"),
        ),
        CredentialType::Attestation,
        CERTIFICATE,
        "Certificate",
    );
    assert_credential_alternative_type(
        CreateCredentialObject::PasswordCredential(
            PasswordCredentialObject::try_new(PasswordCredentialObjectFields::default())
                .expect("valid password credential"),
        ),
        CredentialType::UsernameAndPassword,
        PASSWORD_CREDENTIAL,
        "PasswordCredential",
    );
    assert_credential_alternative_type(
        CreateCredentialObject::DeviceCredential(
            DeviceCredentialObject::try_new(DeviceCredentialObjectFields {
                device_identifier: Some(SecretText::new("device-id".to_owned())),
                ..DeviceCredentialObjectFields::default()
            })
            .expect("valid device credential"),
        ),
        CredentialType::Device,
        DEVICE_CREDENTIAL,
        "DeviceCredential",
    );
    assert_credential_alternative_type(
        CreateCredentialObject::HashedPasswordCredential(
            HashedPasswordCredentialObject::try_new(HashedPasswordCredentialObjectFields {
                hashed_username_password: Some(SecretBytes::new(vec![0xA1])),
                hashed_password_username: Some(SecretBytes::new(vec![0xB2])),
                hashing_algorithm: None,
            })
            .expect("valid hashed password credential"),
        ),
        CredentialType::HashedPassword,
        HASHED_PASSWORD_CREDENTIAL,
        "HashedPasswordCredential",
    );
    assert_credential_alternative_type(
        CreateCredentialObject::OneTimePasswordCredential(
            OneTimePasswordCredentialObject::try_new(OneTimePasswordCredentialObjectFields {
                otp_algorithm: Some(1),
                ..OneTimePasswordCredentialObjectFields::default()
            })
            .expect("valid one-time password credential"),
        ),
        CredentialType::OneTimePassword,
        OTP_CREDENTIAL,
        "OneTimePasswordCredential",
    );
}

#[test]
fn create_group_and_user_use_required_attributes_and_identifier_responses() {
    let group_request = CreateGroupRequest::new(AttributeSet::new());
    assert!(format!("{group_request:?}").contains("attributes_count: 0"));
    let group = group_request
        .into_ttlv_payload()
        .expect("valid group payload");
    assert_eq!(payload_tags(&group), [ATTRIBUTES]);
    let user_request = CreateUserRequest::new(AttributeSet::new());
    assert!(format!("{user_request:?}").contains("attributes_count: 0"));
    let user = user_request
        .into_ttlv_payload()
        .expect("valid user payload");
    assert_eq!(payload_tags(&user), [ATTRIBUTES]);

    let group_response = CreateGroupResponse::new(UniqueIdentifier::TextString("g".into()))
        .into_ttlv_payload()
        .expect("valid group response");
    assert_eq!(payload_tags(&group_response), [UNIQUE_IDENTIFIER]);
    let user_response = CreateUserResponse::new(UniqueIdentifier::Enumeration(0x8000_0001))
        .into_ttlv_payload()
        .expect("valid user response");
    assert_eq!(payload_tags(&user_response), [UNIQUE_IDENTIFIER]);
}

#[test]
fn create_group_rejects_an_attribute_that_exceeds_the_local_structure_depth_limit() {
    let attributes = AttributeSet::try_new([depth_limited_extension_attribute()])
        .expect("unknown extension remains a generic attribute");
    let error = CreateGroupRequest::new(attributes)
        .into_ttlv_payload()
        .expect_err("embedding the deep attribute exceeds the local TTLV depth limit");
    assert!(!error.is_empty());
}

#[test]
fn create_credential_response_and_obliterate_follow_their_tables() {
    let response = CreateCredentialResponse::new(UniqueIdentifier::Integer(7))
        .into_ttlv_payload()
        .expect("valid response payload");
    assert_eq!(payload_tags(&response), [UNIQUE_IDENTIFIER]);

    let request = ObliterateRequest::new(UniqueIdentifier::TextString("uid".into()))
        .into_ttlv_payload()
        .expect("valid request payload");
    assert_eq!(payload_tags(&request), [UNIQUE_IDENTIFIER]);
    let request = ObliterateRequest::new(UniqueIdentifier::TextString("uid".into()));
    assert_eq!(
        request.unique_identifier(),
        &UniqueIdentifier::TextString("uid".into())
    );
    let debug = format!("{request:?}");
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("uid"));
    let response = ObliterateResponse::new()
        .into_ttlv_payload()
        .expect("valid empty response payload");
    assert_eq!(payload_tags(&response), [] as [u32; 0]);
}

#[test]
fn obliterate_response_requires_an_empty_success_payload_and_lends_original_ttlv() {
    let success = successful_response(0x3D, Structure::new());
    let response = ObliterateResponse::try_from_response_item(
        success
            .batch_items()
            .next()
            .expect("one response batch item"),
    )
    .expect("empty success payload is valid");
    assert_eq!(response.result().status().raw(), 0);
    assert_eq!(response.with_ttlv(|item| item.children().len()), Some(3));

    let pending = pending_response(0x3D);
    let response = ObliterateResponse::try_from_response_item(
        pending
            .batch_items()
            .next()
            .expect("one response batch item"),
    )
    .expect("Pending has no success payload requirements");
    assert_eq!(response.result().status().raw(), 2);
    assert_eq!(response.with_ttlv(|item| item.children().len()), None);
    assert_eq!(
        ObliterateResponse::default().result().status().raw(),
        0,
        "Default is a successful empty response"
    );

    let malformed = successful_response(0x3D, structure([item(DATA, Value::byte_string(vec![1]))]));
    assert!(matches!(
        ObliterateResponse::try_from_response_item(
            malformed
                .batch_items()
                .next()
                .expect("one response batch item")
        ),
        Err(crate::Kmip30OperationError::MalformedSuccessPayload)
    ));
}

#[test]
fn deactivate_preserves_optional_reason_and_date_order() {
    let mut reason = Structure::new();
    reason
        .try_push(item(0x0042_01B9, Value::enumeration(0x8000_0001)))
        .expect("valid reason field");
    let request = DeactivateRequest::new("uid")
        .with_deactivation_reason(reason)
        .with_deactivation_date(1_700_000_000);
    let debug = format!("{request:?}");
    assert!(debug.contains("has_deactivation_reason: true"));
    assert!(debug.contains("has_deactivation_date: true"));
    let payload = request
        .into_ttlv_payload()
        .expect("valid deactivate payload");
    assert_eq!(
        payload_tags(&payload),
        [UNIQUE_IDENTIFIER, DEACTIVATION_REASON, DEACTIVATION_DATE]
    );
}

#[test]
fn deactivate_response_preserves_success_and_pending_results() {
    let message = successful_response(
        0x40,
        structure([item(
            UNIQUE_IDENTIFIER,
            Value::text_string("deactivated-object".to_owned()),
        )]),
    );
    let response = crate::DeactivateResponse::try_from_response_item(
        message
            .batch_items()
            .next()
            .expect("one response batch item"),
    )
    .expect("Deactivate response is valid");
    assert_eq!(response.result().status().raw(), 0);
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "deactivated-object".to_owned()
        ))
    );
    assert_eq!(response.with_ttlv(|item| item.children().len()), Some(3));
    assert_eq!(
        payload_tags(
            &response
                .into_ttlv_payload()
                .expect("valid response payload")
        ),
        [UNIQUE_IDENTIFIER]
    );

    let message = pending_response(0x40);
    let response = crate::DeactivateResponse::try_from_response_item(
        message
            .batch_items()
            .next()
            .expect("one response batch item"),
    )
    .expect("Pending is a represented KMIP outcome");
    assert_eq!(response.result().status().raw(), 2);
    assert!(response.unique_identifier().is_none());
    assert_eq!(response.with_ttlv(|item| item.children().len()), None);
}

#[test]
fn kem_requests_emit_optional_members_in_table_order_and_preserve_unknown_parameters() {
    let encap = EncapsulateRequest::new("pub-key")
        .with_typed_cryptographic_parameters(unknown_parameters())
        .with_attributes(attributes_with_unknown_extension())
        .into_ttlv_payload()
        .expect("valid encapsulate payload");
    assert_eq!(
        payload_tags(&encap),
        [UNIQUE_IDENTIFIER, CRYPTOGRAPHIC_PARAMETERS, ATTRIBUTES]
    );
    let parameters = encap.view().children()[1].with_value(|value| match value {
        ValueView::Structure(value) => value.children()[0].tag().raw(),
        _ => 0,
    });
    assert_eq!(parameters, 0x0054_1234);
    let attributes = encap.view().children()[2].with_value(|value| match value {
        ValueView::Structure(value) => value.children()[0].tag().raw(),
        _ => 0,
    });
    assert_eq!(attributes, 0x0054_1234);

    let request = EncapsulateRequest::new("pub-key")
        .with_typed_cryptographic_parameters(unknown_parameters())
        .with_attributes(attributes_with_unknown_extension());
    let debug = format!("{request:?}");
    assert!(debug.contains("has_cryptographic_parameters: true"));
    assert!(debug.contains("has_attributes: true"));
    assert_eq!(
        request.unique_identifier(),
        &UniqueIdentifier::TextString("pub-key".into())
    );

    let decap = DecapsulateRequest::new("priv-key", vec![0x01, 0x02])
        .with_typed_cryptographic_parameters(unknown_parameters())
        .with_attributes(attributes_with_unknown_extension());
    assert_eq!(
        decap.unique_identifier(),
        &UniqueIdentifier::TextString("priv-key".into())
    );
    assert_eq!(decap.ciphertext(), &[0x01, 0x02]);
    let debug = format!("{decap:?}");
    assert!(debug.contains("ciphertext_length: 2"));
    assert!(debug.contains("has_cryptographic_parameters: true"));
    assert!(debug.contains("has_attributes: true"));
    let decap = decap
        .into_ttlv_payload()
        .expect("valid decapsulate payload");
    assert_eq!(
        payload_tags(&decap),
        [
            UNIQUE_IDENTIFIER,
            CRYPTOGRAPHIC_PARAMETERS,
            DATA,
            ATTRIBUTES
        ]
    );
}

#[test]
fn kem_requests_reject_depth_excess_and_invalid_known_cryptographic_parameters() {
    let attributes = AttributeSet::try_new([depth_limited_extension_attribute()])
        .expect("unknown extension remains a generic attribute");
    assert!(
        EncapsulateRequest::new("key")
            .with_attributes(attributes)
            .into_ttlv_payload()
            .is_err()
    );

    let parameters = structure([item(
        CRYPTOGRAPHIC_ALGORITHM,
        Value::byte_string(vec![0x01]),
    )]);
    assert!(
        EncapsulateRequest::new("key")
            .with_cryptographic_parameters(parameters)
            .into_ttlv_payload()
            .is_err()
    );
}

#[test]
fn response_parsers_extract_success_fields_and_reject_another_operation() {
    let group_payload = structure([item(
        UNIQUE_IDENTIFIER,
        Value::text_string("created-group".to_owned()),
    )]);
    let group_message = successful_response(0x3C, group_payload);
    let group_item = group_message
        .batch_items()
        .next()
        .expect("one response batch item");
    let group_response = CreateGroupResponse::try_from_response_item(group_item)
        .expect("Create Group response is valid");
    assert_eq!(group_response.result().status().raw(), 0);
    assert_eq!(
        group_response.unique_identifier(),
        Some(&UniqueIdentifier::TextString("created-group".to_owned()))
    );
    assert_eq!(
        group_response.with_ttlv(|item| item.children().len()),
        Some(3)
    );
    assert!(matches!(
        CreateUserResponse::try_from_response_item(group_item),
        Err(crate::Kmip30OperationError::UnexpectedOperation)
    ));

    let credential_message = successful_response(
        0x3F,
        structure([item(UNIQUE_IDENTIFIER, Value::integer(7))]),
    );
    let credential_response = CreateCredentialResponse::try_from_response_item(
        credential_message
            .batch_items()
            .next()
            .expect("one response batch item"),
    )
    .expect("Create Credential response is valid");
    assert_eq!(credential_response.result().status().raw(), 0);
    assert_eq!(
        credential_response.unique_identifier(),
        Some(&UniqueIdentifier::Integer(7))
    );
    assert_eq!(
        credential_response.with_ttlv(|item| item.children().len()),
        Some(3)
    );

    let encapsulate_payload = structure([
        item(UNIQUE_IDENTIFIER, Value::enumeration(0x8000_0001)),
        item(DATA, Value::byte_string(vec![0xAA, 0xBB])),
    ]);
    let encapsulate_message = successful_response(0x41, encapsulate_payload);
    let encapsulate_item = encapsulate_message
        .batch_items()
        .next()
        .expect("one response batch item");
    let encapsulate_response = crate::EncapsulateResponse::try_from_response_item(encapsulate_item)
        .expect("Encapsulate response is valid");
    assert_eq!(encapsulate_response.result().status().raw(), 0);
    assert_eq!(encapsulate_response.data(), Some(&[0xAA, 0xBB][..]));
    assert_eq!(
        encapsulate_response.unique_identifier(),
        Some(&UniqueIdentifier::Enumeration(0x8000_0001))
    );
    assert_eq!(
        encapsulate_response.with_ttlv(|item| item.children().len()),
        Some(3)
    );
    let debug = format!("{encapsulate_response:?}");
    assert!(!debug.contains("shared_secret"));
    assert_eq!(
        payload_tags(
            &encapsulate_response
                .into_ttlv_payload()
                .expect("valid Encapsulate payload")
        ),
        [UNIQUE_IDENTIFIER, DATA]
    );

    let integer_identifier = successful_response(
        0x41,
        structure([
            item(UNIQUE_IDENTIFIER, Value::integer(11)),
            item(DATA, Value::byte_string(vec![0xCC])),
        ]),
    );
    let integer_response = crate::EncapsulateResponse::try_from_response_item(
        integer_identifier
            .batch_items()
            .next()
            .expect("one response batch item"),
    )
    .expect("Integer Unique Identifier is preserved");
    assert_eq!(
        integer_response.unique_identifier(),
        Some(&UniqueIdentifier::Integer(11))
    );
}

#[test]
fn response_parser_keeps_pending_result_without_success_payload() {
    let message = pending_response(0x3C);
    let response = CreateGroupResponse::try_from_response_item(
        message
            .batch_items()
            .next()
            .expect("one response batch item"),
    )
    .expect("Operation Pending is a represented KMIP outcome");
    assert_eq!(response.result().status().raw(), 2);
    assert!(response.unique_identifier().is_none());
}

#[test]
fn decapsulate_response_rejects_data_and_retains_unknown_extensions() {
    let invalid_payload = structure([
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string("secret-object".to_owned()),
        ),
        item(
            DATA,
            Value::byte_string(vec![0x53, 0x45, 0x43, 0x52, 0x45, 0x54]),
        ),
    ]);
    let invalid_message = successful_response(0x42, invalid_payload);
    let invalid_item = invalid_message
        .batch_items()
        .next()
        .expect("one response batch item");
    let error = crate::DecapsulateResponse::try_from_response_item(invalid_item)
        .expect_err("Table 297 does not return Data or a shared secret");
    assert_eq!(error, crate::Kmip30OperationError::MalformedSuccessPayload);
    assert!(!error.to_string().contains("SECRET"));

    let extension_tag = 0x0054_1234;
    let valid_payload = structure([
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string("secret-object".to_owned()),
        ),
        item(extension_tag, Value::byte_string(vec![0xA5])),
    ]);
    let valid_message = successful_response(0x42, valid_payload);
    let valid_item = valid_message
        .batch_items()
        .next()
        .expect("one response batch item");
    let response = crate::DecapsulateResponse::try_from_response_item(valid_item)
        .expect("unknown extension is retained while standard fields are validated");
    assert_eq!(response.result().status().raw(), 0);
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString("secret-object".to_owned()))
    );
    assert_eq!(response.with_ttlv(|item| item.children().len()), Some(3));
    let extension_is_retained = response
        .with_ttlv(|item| {
            item.children().iter().any(|field| {
                field.tag().raw() == RESPONSE_PAYLOAD
                    && field.with_value(|value| match value {
                        ValueView::Structure(payload) => payload
                            .children()
                            .iter()
                            .any(|child| child.tag().raw() == extension_tag),
                        _ => false,
                    })
            })
        })
        .unwrap_or(false);
    assert!(extension_is_retained);
    assert_eq!(
        payload_tags(
            &response
                .into_ttlv_payload()
                .expect("valid Decapsulate response payload")
        ),
        [UNIQUE_IDENTIFIER]
    );
}

#[test]
fn kem_success_response_parsers_reject_duplicate_missing_and_mistyped_fields() {
    let encapsulate_cases = [
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
            item(DATA, Value::byte_string(vec![1])),
        ]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::byte_string(vec![1])),
            item(DATA, Value::byte_string(vec![1])),
        ]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("key".to_owned())),
            item(DATA, Value::byte_string(vec![1])),
            item(DATA, Value::byte_string(vec![2])),
        ]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("key".to_owned())),
            item(DATA, Value::enumeration(1)),
        ]),
        structure([item(DATA, Value::byte_string(vec![1]))]),
        structure([item(
            UNIQUE_IDENTIFIER,
            Value::text_string("key".to_owned()),
        )]),
    ];
    for payload in encapsulate_cases {
        assert_kem_success_payload_error(0x41, payload);
    }

    let decapsulate_cases = [
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
        ]),
        structure([item(UNIQUE_IDENTIFIER, Value::byte_string(vec![1]))]),
        structure([item(DATA, Value::byte_string(vec![0x53, 0x45, 0x43]))]),
        Structure::new(),
    ];
    for payload in decapsulate_cases {
        assert_kem_success_payload_error(0x42, payload);
    }
}
