//! Payload tests for the KMIP 3.0 client operations in KMIPKIT-0050.

use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

use crate::{
    AttributeSet, CreateCredentialObject, CreateCredentialRequest, CreateCredentialResponse,
    CreateGroupRequest, CreateGroupResponse, CreateUserRequest, CreateUserResponse, CredentialType,
    DeactivateRequest, DecapsulateRequest, EncapsulateRequest, ObliterateRequest,
    ObliterateResponse, PasswordCredentialObject, PasswordCredentialObjectFields, SecretText,
    UniqueIdentifier,
};

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const DATA: u32 = 0x0042_00C2;
const ATTRIBUTES: u32 = 0x0042_0125;
const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const PASSWORD_CREDENTIAL: u32 = 0x0042_01A1;
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

#[test]
fn create_credential_uses_table_277_order_and_redacts_credential_material() {
    let fields = PasswordCredentialObjectFields {
        password: Some(SecretText::new("secret-marker".to_owned())),
        ..PasswordCredentialObjectFields::default()
    };
    let password = PasswordCredentialObject::try_new(fields).expect("valid credential object");
    let request = CreateCredentialRequest::new(
        CredentialType::UsernameAndPassword,
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
}

#[test]
fn create_group_and_user_use_required_attributes_and_identifier_responses() {
    let group = CreateGroupRequest::new(AttributeSet::new())
        .into_ttlv_payload()
        .expect("valid group payload");
    assert_eq!(payload_tags(&group), [ATTRIBUTES]);
    let user = CreateUserRequest::new(AttributeSet::new())
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
fn create_credential_response_and_obliterate_follow_their_tables() {
    let response = CreateCredentialResponse::new(UniqueIdentifier::Integer(7))
        .into_ttlv_payload()
        .expect("valid response payload");
    assert_eq!(payload_tags(&response), [UNIQUE_IDENTIFIER]);

    let request = ObliterateRequest::new(UniqueIdentifier::TextString("uid".into()))
        .into_ttlv_payload()
        .expect("valid request payload");
    assert_eq!(payload_tags(&request), [UNIQUE_IDENTIFIER]);
    let response = ObliterateResponse::new()
        .into_ttlv_payload()
        .expect("valid empty response payload");
    assert_eq!(payload_tags(&response), [] as [u32; 0]);
}

#[test]
fn deactivate_preserves_optional_reason_and_date_order() {
    let mut reason = Structure::new();
    reason
        .try_push(item(0x0042_01B9, Value::enumeration(0x8000_0001)))
        .expect("valid reason field");
    let payload = DeactivateRequest::new("uid")
        .with_deactivation_reason(reason)
        .with_deactivation_date(1_700_000_000)
        .into_ttlv_payload()
        .expect("valid deactivate payload");
    assert_eq!(
        payload_tags(&payload),
        [UNIQUE_IDENTIFIER, DEACTIVATION_REASON, DEACTIVATION_DATE]
    );
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

    let decap = DecapsulateRequest::new("priv-key", vec![0x01, 0x02])
        .with_typed_cryptographic_parameters(unknown_parameters())
        .with_attributes(attributes_with_unknown_extension())
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
    assert!(matches!(
        CreateUserResponse::try_from_response_item(group_item),
        Err(crate::Kmip30OperationError::UnexpectedOperation)
    ));

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
    let debug = format!("{encapsulate_response:?}");
    assert!(!debug.contains("shared_secret"));
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
}
