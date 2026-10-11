//! Source-derived contracts from KMIP 2.1 §6.1.14 Tables 205–207 and §7.12 Table 364.

use crate::async_operation_fixtures::response_message;
use crate::{
    AttributeSet, DerivationMethod, DeriveKeyError, DeriveKeyRequest, DeriveKeyResponse,
    ObjectType, ResultReason, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

const OPERATION: u32 = 0x0000_0005;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const DERIVATION_METHOD: u32 = 0x0042_0051;
const DERIVATION_PARAMETERS: u32 = 0x0042_0050;
const ATTRIBUTES: u32 = 0x0042_0125;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const DERIVATION_DATA: u32 = 0x0042_00c8;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const RESPONSE_TOO_LARGE: u32 = 2;

fn item(raw: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw)
            .expect("fixture tag fits KMIP's 24-bit tag field")
            .try_checked()
            .expect("fixture tag is allocated by the KMIP catalog"),
        value,
    )
    .expect("fixture item has a valid TTLV value")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut value = Structure::new();
    for child in items {
        value.try_push(child).expect("fixture is within TTLV limits");
    }
    value
}

fn response_item(message: &crate::ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message.batch_items().next().expect("one response item exists")
}

fn valid_attributes() -> AttributeSet {
    AttributeSet::try_new([
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
    ])
    .expect("the symmetric-key attributes use catalogued TTLV types")
}

fn request() -> DeriveKeyRequest {
    DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![
            UniqueIdentifier::TextString("source-one".to_owned()),
            UniqueIdentifier::TextString("source-two".to_owned()),
        ],
        DerivationMethod::from_raw(1),
        structure([item(DERIVATION_DATA, Value::byte_string(b"synthetic-input".to_vec()))]),
        valid_attributes(),
    )
}

fn field_tags(value: &Structure) -> Vec<u32> {
    value.view().children().iter().map(|field| field.tag().raw()).collect()
}

#[test]
fn request_encodes_required_table_205_fields_in_order_and_repeats_identifiers() {
    let payload = request()
        .into_ttlv_payload()
        .expect("caller-supplied Table 205 fields are valid");
    let fields = payload.view().children();
    assert_eq!(
        field_tags(&payload),
        [OBJECT_TYPE, UNIQUE_IDENTIFIER, UNIQUE_IDENTIFIER, DERIVATION_METHOD, DERIVATION_PARAMETERS, ATTRIBUTES]
    );
    assert_eq!(
        fields[1].with_value(|value| match value {
            ValueView::TextString(value) => Some(value),
            _ => None,
        }),
        Some("source-one")
    );
    assert_eq!(
        fields[2].with_value(|value| match value {
            ValueView::TextString(value) => Some(value),
            _ => None,
        }),
        Some("source-two")
    );
}

#[test]
fn request_rejects_id_placeholder_and_missing_symmetric_key_attributes() {
    let placeholder = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::Enumeration(1)],
        DerivationMethod::from_raw(1),
        Structure::new(),
        valid_attributes(),
    );
    assert_eq!(placeholder.into_ttlv_payload().unwrap_err().kind(), crate::ProtocolErrorKind::InvalidValue);

    let no_algorithm = AttributeSet::try_new([item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))])
        .expect("length is a valid typed attribute");
    let missing = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(1),
        Structure::new(),
        no_algorithm,
    );
    assert!(missing.into_ttlv_payload().is_err());
}

#[test]
fn response_preserves_success_identifier_and_server_error_reason() {
    let success_payload = structure([item(
        UNIQUE_IDENTIFIER,
        Value::text_string("derived-key".to_owned()),
    )]);
    let success = response_message(OPERATION, SUCCESS, None, None, Some(success_payload));
    let response = DeriveKeyResponse::try_from_response_item(response_item(&success))
        .expect("Table 206 success has a Unique Identifier");
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString("derived-key".to_owned()))
    );

    let failed = response_message(
        OPERATION,
        OPERATION_FAILED,
        Some(RESPONSE_TOO_LARGE),
        None,
        None,
    );
    let response = DeriveKeyResponse::try_from_response_item(response_item(&failed))
        .expect("Table 207 failure remains a typed KMIP result");
    assert_eq!(response.result().reason().map(ResultReason::raw), Some(RESPONSE_TOO_LARGE));
}

#[test]
fn both_derivation_input_forms_are_sent_and_server_error_is_not_rejected_locally() {
    let derivation_parameters = structure([
        item(DERIVATION_DATA, Value::byte_string(b"synthetic-input".to_vec())),
        item(0x0042_0094, Value::text_string("secret-data-id".to_owned())),
    ]);
    let payload = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(1),
        derivation_parameters,
        valid_attributes(),
    )
    .into_ttlv_payload()
    .expect("§7.12 assigns the rejection to the server when both inputs are present");
    assert!(payload.view().children().iter().any(|field| field.tag().raw() == DERIVATION_PARAMETERS));

    let failed = response_message(OPERATION, OPERATION_FAILED, Some(RESPONSE_TOO_LARGE), None, None);
    let response = DeriveKeyResponse::try_from_response_item(response_item(&failed))
        .expect("the server error result passes through");
    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    let _: Option<DeriveKeyError> = None;
}
