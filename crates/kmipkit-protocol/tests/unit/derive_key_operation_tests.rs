//! Source-derived contracts from KMIP 2.1 §6.1.14 Tables 205–207 and §7.12 Table 364.

use std::error::Error as _;

use crate::async_operation_fixtures::response_message;
use crate::{
    AttributeSet, DerivationMethod, DerivationParameters, DeriveKeyError, DeriveKeyRequest,
    DeriveKeyResponse, ObjectType, ResultReason, ResultValidationError, UniqueIdentifier,
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
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002b;
const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const DERIVATION_DATA: u32 = 0x0042_0030;
const INITIALIZATION_VECTOR: u32 = 0x0042_003A;
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
fn request_rejects_repeated_singletons_and_undetermined_dsa_but_allows_explicit_pgp_key() {
    // KMIP Specification v2.1 §§4, 4.22 Table 72, and §6.1.14 Table 205.
    let duplicate_singleton = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(1),
        DerivationParameters::new()
            .with_salt(crate::SecretBytes::new(b"salt".to_vec()))
            .with_iteration_count(120_000),
        AttributeSet::try_new([
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            item(0x0042_00FD, Value::text_string("first".to_owned())),
            item(0x0042_00FD, Value::text_string("second".to_owned())),
        ])
        .expect("the fixture contains catalogued, correctly encoded attributes"),
    );
    assert!(duplicate_singleton.into_ttlv_payload().is_err());

    let parameters = || {
        DerivationParameters::new()
            .with_salt(crate::SecretBytes::new(b"salt".to_vec()))
            .with_iteration_count(120_000)
    };
    let unresolved = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(1),
        parameters(),
        AttributeSet::try_new([
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            item(0x0042_00AE, Value::enumeration(1)),
            item(0x0042_00AE, Value::enumeration(2)),
        ])
        .expect("all fixture attributes have catalogued TTLV types"),
    );
    assert!(unresolved.into_ttlv_payload().is_err());

    let explicit_pgp_key = DeriveKeyRequest::new(
        ObjectType::from_raw(9),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(1),
        parameters(),
        AttributeSet::try_new([
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            item(0x0042_00AE, Value::enumeration(1)),
            item(0x0042_00AE, Value::enumeration(2)),
        ])
        .expect("all fixture attributes have catalogued TTLV types"),
    );
    assert!(explicit_pgp_key.into_ttlv_payload().is_ok());
}

#[test]
fn request_rejects_repeated_dsa_for_symmetric_key_output_context() {
    // KMIP Specification v2.1 §§4, 4.22 Table 72, and §6.1.14 Table 205.
    let attributes = AttributeSet::try_new([
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        item(0x0042_00AE, Value::enumeration(1)),
        item(0x0042_00AE, Value::enumeration(2)),
    ])
    .expect("all fixture attributes have catalogued TTLV types");
    let request = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(1),
        DerivationParameters::new()
            .with_salt(crate::SecretBytes::new(b"salt".to_vec()))
            .with_iteration_count(120_000),
        attributes,
    );

    assert!(request.into_ttlv_payload().is_err());
}

#[test]
fn request_allows_repeated_dsa_for_explicit_pgp_key_output_type() {
    // KMIP Specification v2.1 §§4, 4.22 Table 72, and §6.1.14 Table 205.
    let attributes = AttributeSet::try_new([
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        item(0x0042_00AE, Value::enumeration(1)),
        item(0x0042_00AE, Value::enumeration(2)),
    ])
    .expect("all fixture attributes have catalogued TTLV types");
    let request = DeriveKeyRequest::new(
        ObjectType::from_raw(9),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(1),
        DerivationParameters::new()
            .with_salt(crate::SecretBytes::new(b"salt".to_vec()))
            .with_iteration_count(120_000),
        attributes,
    );

    assert!(request.into_ttlv_payload().is_ok());
}

#[test]
fn secret_data_output_accepts_a_total_cryptographic_length_without_key_algorithm() {
    let attributes = AttributeSet::try_new([item(CRYPTOGRAPHIC_LENGTH, Value::integer(512))])
        .expect("Secret Data length is a valid typed attribute");
    let request = DeriveKeyRequest::new(
        ObjectType::from_raw(7),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(1),
        DerivationParameters::new()
            .with_salt(crate::SecretBytes::new(b"synthetic-salt".to_vec()))
            .with_iteration_count(10_000),
        attributes,
    );

    let payload = request
        .into_ttlv_payload()
        .expect("Secret Data uses the requested total Cryptographic Length");
    assert_eq!(payload.view().children()[0].tag().raw(), OBJECT_TYPE);
    // OASIS KMIP 2.1 §11.34/Table 468 assigns Secret Data Object Type value 7.
    assert!(
        payload.view().children()[0]
            .with_value(|value| matches!(value, ValueView::Enumeration(raw) if *raw == 7))
    );
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
    let secret_object_input = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString(
            "secret-input-object".to_owned(),
        )],
        DerivationMethod::from_raw(1),
        DerivationParameters::new()
            .with_salt(crate::SecretBytes::new(b"synthetic-salt".to_vec()))
            .with_iteration_count(120_000),
        valid_attributes(),
    )
    .into_ttlv_payload()
    .expect("a caller may select a Secret Data input object");
    let fields = secret_object_input.view();
    assert!(fields.children().iter().any(|field| {
        field.tag().raw() == UNIQUE_IDENTIFIER
            && field.with_value(|value| {
                matches!(value, ValueView::TextString(value) if value == "secret-input-object")
            })
    }));
    assert!(fields.children().iter().any(|field| {
        field.tag().raw() == DERIVATION_PARAMETERS
            && field.with_value(|value| match value {
                ValueView::Structure(parameters) => !parameters
                    .children()
                    .iter()
                    .any(|parameter| parameter.tag().raw() == DERIVATION_DATA),
                _ => false,
            })
    }));

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

    let encrypt_parameters = structure([item(BLOCK_CIPHER_MODE, Value::enumeration(1))]);
    let encrypt = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(4),
        DerivationParameters::new().with_cryptographic_parameters(encrypt_parameters),
        valid_attributes(),
    );
    let payload = encrypt
        .into_ttlv_payload()
        .expect("caller-selected ENCRYPT parameters may specify AES-CBC mode");
    assert!(payload.view().children().iter().any(|field| {
        field.tag().raw() == DERIVATION_PARAMETERS
            && field.with_value(|value| match value {
                ValueView::Structure(parameters) => parameters.children().iter().any(|parameter| {
                    parameter.tag().raw() == CRYPTOGRAPHIC_PARAMETERS
                        && parameter.with_value(|value| {
                            match value {
                            ValueView::Structure(parameters) => {
                                parameters.children().len() == 1
                                    && parameters.children()[0].tag().raw() == BLOCK_CIPHER_MODE
                                    && parameters.children()[0].with_value(|value| {
                                        matches!(value, ValueView::Enumeration(mode) if *mode == 1)
                                    })
                            }
                            _ => false,
                        }
                        })
                }),
                _ => false,
            })
    }));
}

fn parameters_with_all_extensions() -> DerivationParameters {
    DerivationParameters::default()
        .with_cryptographic_parameters(structure([item(
            CRYPTOGRAPHIC_PARAMETERS,
            Value::enumeration(9),
        )]))
        .with_initialization_vector(crate::SecretBytes::new(b"synthetic-iv".to_vec()))
        .with_derivation_data(crate::SecretBytes::new(b"synthetic-data".to_vec()))
        .with_salt(crate::SecretBytes::new(b"synthetic-salt".to_vec()))
        .with_iteration_count(77)
        .with_extension(item(UNKNOWN_VENDOR_TAG, Value::integer(1)))
        .expect("the extension tag is an allocated vendor tag")
        .with_extension(item(UNKNOWN_VENDOR_TAG + 1, Value::long_integer(2)))
        .expect("the second extension tag is allocated")
        .with_extension(item(
            UNKNOWN_VENDOR_TAG + 2,
            Value::big_integer(vec![1, 2, 3, 4, 5, 6, 7, 8]),
        ))
        .expect("the big integer extension is a valid TTLV value")
        .with_extension(item(UNKNOWN_VENDOR_TAG + 3, Value::enumeration(3)))
        .expect("the enumeration extension is preserved")
        .with_extension(item(UNKNOWN_VENDOR_TAG + 4, Value::boolean(true)))
        .expect("the boolean extension is preserved")
        .with_extension(item(
            UNKNOWN_VENDOR_TAG + 5,
            Value::text_string("synthetic".to_owned()),
        ))
        .expect("the text extension is preserved")
        .with_extension(item(UNKNOWN_VENDOR_TAG + 6, Value::byte_string(vec![7, 8])))
        .expect("the byte string extension is preserved")
        .with_extension(item(UNKNOWN_VENDOR_TAG + 7, Value::date_time(9)))
        .expect("the Date Time extension is preserved")
        .with_extension(item(UNKNOWN_VENDOR_TAG + 8, Value::interval(10)))
        .expect("the Interval extension is preserved")
        .with_extension(item(UNKNOWN_VENDOR_TAG + 9, Value::date_time_extended(11)))
        .expect("the Date Time Extended extension is preserved")
        .with_extension(item(
            UNKNOWN_VENDOR_TAG + 10,
            Value::structure(structure([item(
                UNKNOWN_VENDOR_TAG + 11,
                Value::integer(12),
            )])),
        ))
        .expect("the nested structure extension is preserved")
}

#[test]
fn derivation_parameter_debug_reports_presence_counts_and_redacts_values() {
    let parameters = parameters_with_all_extensions();
    let debug = format!("{parameters:?}");
    assert!(debug.contains("has_cryptographic_parameters: true"));
    assert!(debug.contains("has_initialization_vector: true"));
    assert!(debug.contains("derivation_data_count: 1"));
    assert!(debug.contains("has_salt: true"));
    assert!(debug.contains("has_iteration_count: true"));
    assert!(debug.contains("extension_count: 11"));
    assert!(!debug.contains("synthetic"));
}

#[test]
fn unknown_method_and_identifier_forms_are_encoded_in_wire_order() {
    let parameters = parameters_with_all_extensions();
    let method = DerivationMethod::from_raw(0xF123_4567);
    assert_eq!(method.raw(), 0xF123_4567);
    let payload = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![
            UniqueIdentifier::TextString("source".to_owned()),
            UniqueIdentifier::Enumeration(0xF123_4568),
            UniqueIdentifier::Integer(-7),
        ],
        method,
        parameters,
        valid_attributes(),
    )
    .into_ttlv_payload()
    .expect("unknown method values and supported identifier forms remain explicit");
    let fields = payload.view();
    assert_eq!(fields.children()[1].tag().raw(), UNIQUE_IDENTIFIER);
    assert_eq!(fields.children()[2].tag().raw(), UNIQUE_IDENTIFIER);
    assert_eq!(fields.children()[3].tag().raw(), UNIQUE_IDENTIFIER);
    assert!(fields.children().iter().any(|field| {
        field.tag().raw() == DERIVATION_METHOD
            && field.with_value(
                |value| matches!(value, ValueView::Enumeration(raw) if *raw == 0xF123_4567),
            )
    }));
    let parameters = fields.children()[5].with_value(|value| match value {
        ValueView::Structure(parameters) => Some(
            parameters
                .children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(
        parameters,
        Some(vec![
            CRYPTOGRAPHIC_PARAMETERS,
            INITIALIZATION_VECTOR,
            DERIVATION_DATA,
            SALT,
            ITERATION_COUNT,
            UNKNOWN_VENDOR_TAG,
            UNKNOWN_VENDOR_TAG + 1,
            UNKNOWN_VENDOR_TAG + 2,
            UNKNOWN_VENDOR_TAG + 3,
            UNKNOWN_VENDOR_TAG + 4,
            UNKNOWN_VENDOR_TAG + 5,
            UNKNOWN_VENDOR_TAG + 6,
            UNKNOWN_VENDOR_TAG + 7,
            UNKNOWN_VENDOR_TAG + 8,
            UNKNOWN_VENDOR_TAG + 9,
            UNKNOWN_VENDOR_TAG + 10,
        ])
    );
}

#[test]
fn output_attribute_and_hash_parameter_rules_cover_invalid_boundaries() {
    let identifier = || UniqueIdentifier::TextString("source".to_owned());
    let request_for = |object_type, attributes, method, parameters| {
        DeriveKeyRequest::new(
            ObjectType::from_raw(object_type),
            vec![identifier()],
            DerivationMethod::from_raw(method),
            parameters,
            attributes,
        )
    };

    let duplicate_length = AttributeSet::try_new([
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(128)),
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
    ])
    .expect("the generic AttributeSet preserves repeated attributes");
    assert!(
        request_for(2, duplicate_length, 99, DerivationParameters::new())
            .into_ttlv_payload()
            .is_err()
    );

    let zero_length = AttributeSet::try_new([
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(0)),
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
    ])
    .expect("zero remains representable for local validation");
    assert!(
        request_for(2, zero_length, 99, DerivationParameters::new())
            .into_ttlv_payload()
            .is_err()
    );
    let unknown_attribute = AttributeSet::try_new([
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        item(UNKNOWN_VENDOR_TAG, Value::integer(9)),
    ])
    .expect("unknown attributes remain structurally represented");
    assert!(
        request_for(2, unknown_attribute, 99, DerivationParameters::new())
            .into_ttlv_payload()
            .is_ok()
    );

    let hash_without_algorithm = structure([item(UNKNOWN_VENDOR_TAG, Value::enumeration(6))]);
    assert!(
        request_for(
            2,
            valid_attributes(),
            2,
            DerivationParameters::new().with_cryptographic_parameters(hash_without_algorithm),
        )
        .into_ttlv_payload()
        .is_err()
    );
    let hash_with_wrong_ttlv_type = structure([item(0x0042_0038, Value::integer(6))]);
    assert!(
        request_for(
            2,
            valid_attributes(),
            2,
            DerivationParameters::new().with_cryptographic_parameters(hash_with_wrong_ttlv_type),
        )
        .into_ttlv_payload()
        .is_err()
    );

    let symmetric_without_algorithm =
        AttributeSet::try_new([item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))])
            .expect("a length-only set can represent Secret Data output");
    assert!(
        request_for(
            2,
            symmetric_without_algorithm,
            99,
            DerivationParameters::new(),
        )
        .into_ttlv_payload()
        .is_err()
    );
    let unknown_object_type_without_algorithm =
        AttributeSet::try_new([item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))])
            .expect("a length-only set is structurally valid");
    assert!(
        request_for(
            0xF123_4567,
            unknown_object_type_without_algorithm,
            99,
            DerivationParameters::new(),
        )
        .into_ttlv_payload()
        .is_ok()
    );
}

#[test]
fn malformed_success_identifiers_and_error_formatting_are_explicit() {
    for invalid_payload in [
        structure([]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::integer(2)),
        ]),
        structure([item(UNIQUE_IDENTIFIER, Value::boolean(true))]),
    ] {
        let message = response_message(OPERATION, SUCCESS, None, None, Some(invalid_payload));
        assert_eq!(
            DeriveKeyResponse::try_from_response_item(response_item(&message)).unwrap_err(),
            DeriveKeyError::MalformedSuccessPayload
        );
    }

    let integer_id = response_message(
        OPERATION,
        SUCCESS,
        None,
        None,
        Some(structure([item(UNIQUE_IDENTIFIER, Value::integer(-4))])),
    );
    let response = DeriveKeyResponse::try_from_response_item(response_item(&integer_id))
        .expect("the response preserves integer Unique Identifier values");
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::Integer(-4))
    );
    assert!(format!("{response:?}").contains("DeriveKeyResponse"));

    let enumeration_id = response_message(
        OPERATION,
        SUCCESS,
        None,
        None,
        Some(structure([item(
            UNIQUE_IDENTIFIER,
            Value::enumeration(0xF123_4568),
        )])),
    );
    let response = DeriveKeyResponse::try_from_response_item(response_item(&enumeration_id))
        .expect("the response preserves enumeration Unique Identifier values");
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::Enumeration(0xF123_4568))
    );

    let failure = ResultValidationError::SuccessForbidsReason;
    let invalid_result = DeriveKeyError::InvalidOperationResult(failure);
    assert_eq!(
        invalid_result.source().unwrap().to_string(),
        failure.to_string()
    );
    assert!(DeriveKeyError::UnexpectedOperation.source().is_none());
    for error in [
        DeriveKeyError::UnexpectedOperation,
        DeriveKeyError::MissingResultStatus,
        invalid_result,
        DeriveKeyError::MissingSuccessPayload,
        DeriveKeyError::MalformedSuccessPayload,
        DeriveKeyError::MalformedResponseItem,
    ] {
        assert_ne!(error.to_string(), "");
    }
}
