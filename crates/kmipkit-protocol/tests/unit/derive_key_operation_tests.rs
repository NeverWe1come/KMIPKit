//! Source-derived contracts from KMIP 2.1 §6.1.14 Tables 205–207 and §7.12 Table 364.

use crate::async_operation_fixtures::response_message;
use crate::{
    AttributeSet, DerivationMethod, DerivationParameters, DeriveKeyError, DeriveKeyRequest,
    DeriveKeyResponse, ObjectType, ResultReason, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

const OPERATION: u32 = 0x0000_0005;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const DERIVATION_METHOD: u32 = 0x0042_0031;
const DERIVATION_PARAMETERS: u32 = 0x0042_0032;
const ATTRIBUTES: u32 = 0x0042_0125;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const DERIVATION_DATA: u32 = 0x0042_0030;
const ITERATION_COUNT: u32 = 0x0042_003c;
const SALT: u32 = 0x0042_0084;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const RESPONSE_TOO_LARGE: u32 = 2;
const FUTURE_RESULT_REASON: u32 = 0xF123_4567;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

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
        value
            .try_push(child)
            .expect("fixture is within TTLV limits");
    }
    value
}

fn response_item(message: &crate::ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("one response item exists")
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
        DerivationParameters::new()
            .with_derivation_data(crate::SecretBytes::new(b"synthetic-input".to_vec()))
            .with_derivation_data(crate::SecretBytes::new(b"second-input".to_vec()))
            .with_salt(crate::SecretBytes::new(b"synthetic-salt".to_vec()))
            .with_iteration_count(120_000),
        valid_attributes(),
    )
}

fn field_tags(value: &Structure) -> Vec<u32> {
    value
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

#[test]
fn request_encodes_required_table_205_fields_in_order_and_repeats_identifiers() {
    let debug = format!("{:?}", request());
    assert!(!debug.contains("synthetic-input"));
    assert!(!debug.contains("synthetic-salt"));
    let payload = request()
        .into_ttlv_payload()
        .expect("caller-supplied Table 205 fields are valid");
    let view = payload.view();
    let fields = view.children();
    assert_eq!(
        field_tags(&payload),
        [
            OBJECT_TYPE,
            UNIQUE_IDENTIFIER,
            UNIQUE_IDENTIFIER,
            DERIVATION_METHOD,
            DERIVATION_PARAMETERS,
            ATTRIBUTES
        ]
    );
    assert_eq!(
        fields[1].with_value(|value| match value {
            ValueView::TextString(value) => Some(value.to_owned()),
            _ => None,
        }),
        Some("source-one".to_owned())
    );
    assert_eq!(
        fields[2].with_value(|value| match value {
            ValueView::TextString(value) => Some(value.to_owned()),
            _ => None,
        }),
        Some("source-two".to_owned())
    );
    let derivation_parameters = fields[4].with_value(|value| match value {
        ValueView::Structure(value) => Some(
            value
                .children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(
        derivation_parameters,
        Some(vec![
            DERIVATION_DATA,
            DERIVATION_DATA,
            SALT,
            ITERATION_COUNT
        ])
    );
}

#[test]
fn request_rejects_id_placeholder_and_missing_symmetric_key_attributes() {
    let placeholder = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::Enumeration(1)],
        DerivationMethod::from_raw(1),
        DerivationParameters::new(),
        valid_attributes(),
    );
    assert_eq!(
        placeholder.into_ttlv_payload().unwrap_err().kind(),
        crate::ProtocolErrorKind::InvalidValue
    );

    let no_algorithm = AttributeSet::try_new([item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))])
        .expect("length is a valid typed attribute");
    let missing = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(1),
        DerivationParameters::new(),
        no_algorithm,
    );
    assert!(missing.into_ttlv_payload().is_err());
}

#[test]
fn response_preserves_success_identifier_and_server_error_reason() {
    let success_payload = structure([
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string("derived-key".to_owned()),
        ),
        item(UNKNOWN_VENDOR_TAG, Value::integer(17)),
    ]);
    let success = response_message(OPERATION, SUCCESS, None, None, Some(success_payload));
    let response = DeriveKeyResponse::try_from_response_item(response_item(&success))
        .expect("Table 206 success has a Unique Identifier");
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString("derived-key".to_owned()))
    );
    drop(success);
    assert!(response.with_ttlv(|ttlv| {
        ttlv.children().iter().any(|field| {
            field.tag().raw() == 0x0042_007c
                && field.with_value(|value| match value {
                    ValueView::Structure(payload) => payload
                        .children()
                        .iter()
                        .any(|field| field.tag().raw() == UNKNOWN_VENDOR_TAG),
                    _ => false,
                })
        })
    }));

    let failed = response_message(
        OPERATION,
        OPERATION_FAILED,
        Some(FUTURE_RESULT_REASON),
        None,
        None,
    );
    let response = DeriveKeyResponse::try_from_response_item(response_item(&failed))
        .expect("Table 207 failure remains a typed KMIP result");
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(FUTURE_RESULT_REASON)
    );
}

#[test]
fn both_derivation_input_forms_are_sent_and_server_error_is_not_rejected_locally() {
    let payload = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(1),
        DerivationParameters::new()
            .with_derivation_data(crate::SecretBytes::new(b"synthetic-input".to_vec()))
            .with_salt(crate::SecretBytes::new(b"synthetic-salt".to_vec()))
            .with_iteration_count(120_000),
        valid_attributes(),
    )
    .into_ttlv_payload()
    .expect("§7.12 assigns the rejection to the server when both inputs are present");
    assert!(
        payload
            .view()
            .children()
            .iter()
            .any(|field| field.tag().raw() == DERIVATION_PARAMETERS)
    );

    let failed = response_message(
        OPERATION,
        OPERATION_FAILED,
        Some(RESPONSE_TOO_LARGE),
        None,
        None,
    );
    let response = DeriveKeyResponse::try_from_response_item(response_item(&failed))
        .expect("the server error result passes through");
    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    let wrong_operation = response_message(
        OPERATION + 1,
        OPERATION_FAILED,
        Some(RESPONSE_TOO_LARGE),
        None,
        None,
    );
    assert_eq!(
        DeriveKeyResponse::try_from_response_item(response_item(&wrong_operation)).unwrap_err(),
        DeriveKeyError::UnexpectedOperation
    );
}

#[test]
fn method_conditional_fields_are_checked_without_selecting_defaults() {
    let missing_hash_parameters = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(2),
        DerivationParameters::new(),
        valid_attributes(),
    );
    assert!(missing_hash_parameters.into_ttlv_payload().is_err());

    let hash_parameters = structure([item(0x0042_0038, Value::enumeration(6))]);
    let valid_hash = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(2),
        DerivationParameters::new().with_cryptographic_parameters(hash_parameters),
        valid_attributes(),
    );
    assert!(valid_hash.into_ttlv_payload().is_ok());
}
