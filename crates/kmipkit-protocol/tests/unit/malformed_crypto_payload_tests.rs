//! Malformed Encrypt and Decrypt payload tests derived from OASIS KMIP
//! Specification v2.1 §6.1.11, Tables 196–198; §6.1.17, Tables 214–216;
//! §4.16; and §7.9. The multipart duplicate cases use the final-request shape
//! from §6.1 (Data, Correlation Value, and Final Indicator=true). Response
//! Data is a Byte String in Tables 197 and 215.
//! Request checks exercise the crate-private validation boundary used before
//! serialization/transmission; they do not define a public request parser.
//!
//! Traceability: `KMIPKIT-REQ-SPEC-6.1.11-001-001`,
//! `KMIPKIT-REQ-SPEC-6.1.17-001-001`, `KMIPKIT-REQ-SPEC-4.16-002`,
//! `KMIPKIT-REQ-SPEC-4.16-003`, `KMIPKIT-REQ-SPEC-6.1-001-002`,
//! `KMIPKIT-ELEM-OP-C2S-DECRYPT`, `KMIPKIT-ELEM-OP-C2S-ENCRYPT`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-4-AUTHENTICATED-ENCRYPTION-TAG`,
//! and `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA`.

use std::fmt::{Debug, Display};

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{DecryptResponse, EncryptResponse, ResponseMessage};
use kmipkit_ttlv::{ItemType, Structure, Value};

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const SUCCESS: u32 = 0;

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const DATA: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE: u32 = 0x0042_003D;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INIT_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const IV_LENGTH: u32 = 0x0042_00CD;
const TAG_LENGTH: u32 = 0x0042_00CE;
const AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA: u32 = 0x0042_00FE;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;

const IDENTIFIER_SENTINEL: &str = "FIELD_VALUE_SECRET_SENTINEL_4931";
const BODY_SENTINEL: &[u8] = b"RAW_KMIP_BODY_SECRET_SENTINEL_7062";

const ENCRYPT_REQUEST_FIELDS: &[u32] = &[
    UNIQUE_IDENTIFIER,
    CRYPTOGRAPHIC_PARAMETERS,
    DATA,
    IV_COUNTER_NONCE,
    CORRELATION_VALUE,
    INIT_INDICATOR,
    FINAL_INDICATOR,
    AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA,
];
const DECRYPT_REQUEST_FIELDS: &[u32] = &[
    UNIQUE_IDENTIFIER,
    CRYPTOGRAPHIC_PARAMETERS,
    DATA,
    IV_COUNTER_NONCE,
    CORRELATION_VALUE,
    INIT_INDICATOR,
    FINAL_INDICATOR,
    AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA,
    AUTHENTICATED_ENCRYPTION_TAG,
];
const ENCRYPT_RESPONSE_FIELDS: &[u32] = &[
    UNIQUE_IDENTIFIER,
    DATA,
    IV_COUNTER_NONCE,
    CORRELATION_VALUE,
    AUTHENTICATED_ENCRYPTION_TAG,
];
const DECRYPT_RESPONSE_FIELDS: &[u32] = &[UNIQUE_IDENTIFIER, DATA, CORRELATION_VALUE];

fn validate_encrypt_request(payload: &Structure) -> Result<(), crate::ProtocolError> {
    crate::encrypt::validate_request_payload(payload).map(|_| ())
}

fn validate_decrypt_request(payload: &Structure) -> Result<(), crate::ProtocolError> {
    crate::decrypt::validate_request_payload(payload).map(|_| ())
}

fn request_value(raw_tag: u32, parameters: &mut Option<Structure>) -> Value {
    match raw_tag {
        UNIQUE_IDENTIFIER => Value::text_string(IDENTIFIER_SENTINEL.to_owned()),
        CRYPTOGRAPHIC_PARAMETERS => {
            Value::structure(parameters.take().unwrap_or_else(Structure::new))
        }
        DATA
        | IV_COUNTER_NONCE
        | CORRELATION_VALUE
        | AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA
        | AUTHENTICATED_ENCRYPTION_TAG => Value::byte_string(BODY_SENTINEL.to_vec()),
        INIT_INDICATOR => Value::boolean(true),
        FINAL_INDICATOR => Value::boolean(false),
        _ => unreachable!("the source-derived request field list uses known tags"),
    }
}

fn request_payload(
    fields: &[u32],
    duplicate_tag: Option<u32>,
    parameters: Option<Structure>,
) -> Structure {
    let mut payload = Structure::new();
    let mut parameters = parameters;
    let include_parameters = parameters.is_some();
    for raw_tag in fields.iter().copied().filter(|raw_tag| {
        matches!(*raw_tag, UNIQUE_IDENTIFIER | DATA)
            || Some(*raw_tag) == duplicate_tag
            || (include_parameters && *raw_tag == CRYPTOGRAPHIC_PARAMETERS)
    }) {
        payload
            .try_push(item(raw_tag, request_value(raw_tag, &mut parameters)))
            .expect("malformed request test payload fits TTLV limits");
        if Some(raw_tag) == duplicate_tag {
            payload
                .try_push(item(raw_tag, request_value(raw_tag, &mut parameters)))
                .expect("duplicate singleton test payload fits TTLV limits");
        }
    }
    payload
}

fn duplicate_multipart_request_payload(duplicate_tag: u32) -> Structure {
    // §6.1 final multipart shape: Data, Correlation Value, and Final=true are
    // present, Init is absent, and only the selected field is repeated.
    assert!(matches!(duplicate_tag, CORRELATION_VALUE | FINAL_INDICATOR));
    let mut payload = Structure::new();
    for raw_tag in [UNIQUE_IDENTIFIER, DATA, CORRELATION_VALUE, FINAL_INDICATOR] {
        payload
            .try_push(item(raw_tag, multipart_request_value(raw_tag)))
            .expect("final multipart request fields fit the test Structure");
        if raw_tag == duplicate_tag {
            payload
                .try_push(item(raw_tag, multipart_request_value(raw_tag)))
                .expect("the selected multipart field fits twice");
        }
    }
    let view = payload.view();
    assert_eq!(view.children().len(), 5);
    for raw_tag in [UNIQUE_IDENTIFIER, DATA, CORRELATION_VALUE, FINAL_INDICATOR] {
        assert_eq!(
            tag_count(&payload, raw_tag),
            if raw_tag == duplicate_tag { 2 } else { 1 },
            "only the selected multipart field is duplicated"
        );
    }
    assert_eq!(tag_count(&payload, INIT_INDICATOR), 0);
    payload
}

fn multipart_request_value(raw_tag: u32) -> Value {
    match raw_tag {
        UNIQUE_IDENTIFIER => Value::text_string(IDENTIFIER_SENTINEL.to_owned()),
        DATA | CORRELATION_VALUE => Value::byte_string(BODY_SENTINEL.to_vec()),
        FINAL_INDICATOR => Value::boolean(true),
        _ => unreachable!("the final multipart request uses four known fields"),
    }
}

fn tag_count(payload: &Structure, raw_tag: u32) -> usize {
    payload
        .view()
        .children()
        .iter()
        .filter(|child| child.tag().raw() == raw_tag)
        .count()
}

fn response_value(raw_tag: u32) -> Value {
    match raw_tag {
        UNIQUE_IDENTIFIER => Value::text_string(IDENTIFIER_SENTINEL.to_owned()),
        DATA | IV_COUNTER_NONCE | CORRELATION_VALUE | AUTHENTICATED_ENCRYPTION_TAG => {
            Value::byte_string(BODY_SENTINEL.to_vec())
        }
        _ => unreachable!("the response field arrays contain Table 197/215 members"),
    }
}

fn response_payload(fields: &[u32], duplicate_tag: u32) -> Structure {
    let mut payload = Structure::new();
    for raw_tag in fields {
        payload
            .try_push(item(*raw_tag, response_value(*raw_tag)))
            .expect("response fields fit the test Structure");
        if *raw_tag == duplicate_tag {
            // The table-derived field list emits every other singleton once.
            payload
                .try_push(item(*raw_tag, response_value(*raw_tag)))
                .expect("one duplicate response field fits the test Structure");
        }
    }
    let view = payload.view();
    let children = view.children();
    assert_eq!(children.len(), fields.len() + 1);
    for raw_tag in fields {
        let matching: Vec<_> = children
            .iter()
            .filter(|child| child.tag().raw() == *raw_tag)
            .collect();
        assert_eq!(
            matching.len(),
            if *raw_tag == duplicate_tag { 2 } else { 1 },
            "only the selected response singleton is duplicated"
        );
        assert!(
            matching
                .iter()
                .all(|child| child.item_type() == response_item_type(*raw_tag))
        );
    }
    payload
}

fn response_item_type(raw_tag: u32) -> ItemType {
    match raw_tag {
        UNIQUE_IDENTIFIER => ItemType::TextString,
        DATA | IV_COUNTER_NONCE | CORRELATION_VALUE | AUTHENTICATED_ENCRYPTION_TAG => {
            ItemType::ByteString
        }
        _ => unreachable!("the response field arrays contain Table 197/215 members"),
    }
}

fn cryptographic_parameters(raw_tag: u32, value: Value) -> Structure {
    structure([item(raw_tag, value)])
}

fn validate_and_assert_redacted<T, E>(result: Result<T, E>)
where
    E: Debug + Display,
{
    let error = result
        .err()
        .expect("the malformed operation payload is rejected");
    let display = error.to_string();
    let debug = format!("{error:?}");
    for message in [display.as_str(), debug.as_str()] {
        assert!(!message.contains(IDENTIFIER_SENTINEL));
        assert!(!message.contains("RAW_KMIP_BODY_SECRET_SENTINEL_7062"));
        assert!(!message.contains("4294967295"));
        assert!(!message.contains("-2147483648"));
        assert!(!message.contains("4951"));
        assert!(!message.contains("9320"));
        assert!(!message.contains("13978"));
    }
}

fn successful_response(operation: u32, payload: Structure) -> ResponseMessage {
    response_message(operation, SUCCESS, None, None, Some(payload))
}

#[test]
fn encrypt_request_rejects_duplicate_table_214_singletons() {
    for raw_tag in ENCRYPT_REQUEST_FIELDS {
        let malformed = if matches!(*raw_tag, CORRELATION_VALUE | FINAL_INDICATOR) {
            duplicate_multipart_request_payload(*raw_tag)
        } else {
            request_payload(ENCRYPT_REQUEST_FIELDS, Some(*raw_tag), None)
        };
        validate_and_assert_redacted(validate_encrypt_request(&malformed));
    }
}

#[test]
fn decrypt_request_rejects_duplicate_table_196_singletons() {
    for raw_tag in DECRYPT_REQUEST_FIELDS {
        let malformed = if matches!(*raw_tag, CORRELATION_VALUE | FINAL_INDICATOR) {
            duplicate_multipart_request_payload(*raw_tag)
        } else {
            request_payload(DECRYPT_REQUEST_FIELDS, Some(*raw_tag), None)
        };
        validate_and_assert_redacted(validate_decrypt_request(&malformed));
    }
}

#[test]
fn encrypt_request_rejects_duplicate_known_cryptographic_parameter_members() {
    for (raw_tag, value) in [
        (BLOCK_CIPHER_MODE, Value::enumeration(u32::MAX)),
        (IV_LENGTH, Value::integer(128)),
        (TAG_LENGTH, Value::integer(16)),
    ] {
        let mut parameters = Structure::new();
        parameters
            .try_push(item(raw_tag, value))
            .expect("known parameter fits the test Structure");
        parameters
            .try_push(item(raw_tag, request_parameter_value(raw_tag)))
            .expect("duplicate known parameter fits the test Structure");
        let malformed = request_payload(ENCRYPT_REQUEST_FIELDS, None, Some(parameters));
        validate_and_assert_redacted(validate_encrypt_request(&malformed));
    }
}

#[test]
fn decrypt_request_rejects_duplicate_known_cryptographic_parameter_members() {
    for (raw_tag, value) in [
        (BLOCK_CIPHER_MODE, Value::enumeration(u32::MAX)),
        (IV_LENGTH, Value::integer(128)),
        (TAG_LENGTH, Value::integer(16)),
    ] {
        let mut parameters = Structure::new();
        parameters
            .try_push(item(raw_tag, value))
            .expect("known parameter fits the test Structure");
        parameters
            .try_push(item(raw_tag, request_parameter_value(raw_tag)))
            .expect("duplicate known parameter fits the test Structure");
        let malformed = request_payload(DECRYPT_REQUEST_FIELDS, None, Some(parameters));
        validate_and_assert_redacted(validate_decrypt_request(&malformed));
    }
}

fn request_parameter_value(raw_tag: u32) -> Value {
    match raw_tag {
        BLOCK_CIPHER_MODE => Value::enumeration(u32::MAX),
        IV_LENGTH => Value::integer(128),
        TAG_LENGTH => Value::integer(16),
        _ => unreachable!("the parameter test uses only the three required members"),
    }
}

#[test]
fn encrypt_request_rejects_wrong_known_cryptographic_parameter_types() {
    for (raw_tag, value) in [
        (BLOCK_CIPHER_MODE, Value::integer(0x1357)),
        (IV_LENGTH, Value::enumeration(0x2468)),
        (TAG_LENGTH, Value::enumeration(0x369A)),
    ] {
        let parameters = cryptographic_parameters(raw_tag, value);
        let malformed = request_payload(ENCRYPT_REQUEST_FIELDS, None, Some(parameters));
        validate_and_assert_redacted(validate_encrypt_request(&malformed));
    }
}

#[test]
fn decrypt_request_rejects_wrong_known_cryptographic_parameter_types() {
    for (raw_tag, value) in [
        (BLOCK_CIPHER_MODE, Value::integer(0x1357)),
        (IV_LENGTH, Value::enumeration(0x2468)),
        (TAG_LENGTH, Value::enumeration(0x369A)),
    ] {
        let parameters = cryptographic_parameters(raw_tag, value);
        let malformed = request_payload(DECRYPT_REQUEST_FIELDS, None, Some(parameters));
        validate_and_assert_redacted(validate_decrypt_request(&malformed));
    }
}

#[test]
fn encrypt_request_rejects_the_decrypt_only_authenticated_encryption_tag() {
    let malformed = structure([
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string(IDENTIFIER_SENTINEL.to_owned()),
        ),
        item(DATA, Value::byte_string(BODY_SENTINEL.to_vec())),
        item(
            AUTHENTICATED_ENCRYPTION_TAG,
            Value::byte_string(BODY_SENTINEL.to_vec()),
        ),
    ]);
    validate_and_assert_redacted(validate_encrypt_request(&malformed));
}

#[test]
fn successful_encrypt_response_requires_unique_identifier() {
    let payload = structure([item(DATA, Value::byte_string(BODY_SENTINEL.to_vec()))]);
    let message = successful_response(ENCRYPT_OPERATION, payload);
    let item = message
        .batch_items()
        .next()
        .expect("the successful response has one batch item");
    validate_and_assert_redacted(EncryptResponse::try_from_response_item(item));
}

#[test]
fn successful_decrypt_response_requires_unique_identifier() {
    let payload = structure([item(DATA, Value::byte_string(BODY_SENTINEL.to_vec()))]);
    let message = successful_response(DECRYPT_OPERATION, payload);
    let item = message
        .batch_items()
        .next()
        .expect("the successful response has one batch item");
    validate_and_assert_redacted(DecryptResponse::try_from_response_item(item));
}

#[test]
fn encrypt_response_rejects_each_duplicate_table_215_singleton() {
    for raw_tag in ENCRYPT_RESPONSE_FIELDS {
        let payload = response_payload(ENCRYPT_RESPONSE_FIELDS, *raw_tag);
        let message = successful_response(ENCRYPT_OPERATION, payload);
        let item = message
            .batch_items()
            .next()
            .expect("the successful response has one batch item");
        validate_and_assert_redacted(EncryptResponse::try_from_response_item(item));
    }
}

#[test]
fn decrypt_response_rejects_each_duplicate_table_197_singleton() {
    for raw_tag in DECRYPT_RESPONSE_FIELDS {
        let payload = response_payload(DECRYPT_RESPONSE_FIELDS, *raw_tag);
        let message = successful_response(DECRYPT_OPERATION, payload);
        let item = message
            .batch_items()
            .next()
            .expect("the successful response has one batch item");
        validate_and_assert_redacted(DecryptResponse::try_from_response_item(item));
    }
}

#[test]
fn encrypt_response_rejects_enumeration_or_integer_data() {
    for value in [Value::enumeration(u32::MAX), Value::integer(i32::MIN)] {
        let payload = structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(IDENTIFIER_SENTINEL.to_owned()),
            ),
            item(DATA, value),
        ]);
        let message = successful_response(ENCRYPT_OPERATION, payload);
        let item = message
            .batch_items()
            .next()
            .expect("the successful response has one batch item");
        validate_and_assert_redacted(EncryptResponse::try_from_response_item(item));
    }
}

#[test]
fn decrypt_response_rejects_enumeration_or_integer_data() {
    for value in [Value::enumeration(u32::MAX), Value::integer(i32::MIN)] {
        let payload = structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(IDENTIFIER_SENTINEL.to_owned()),
            ),
            item(DATA, value),
        ]);
        let message = successful_response(DECRYPT_OPERATION, payload);
        let item = message
            .batch_items()
            .next()
            .expect("the successful response has one batch item");
        validate_and_assert_redacted(DecryptResponse::try_from_response_item(item));
    }
}

#[test]
fn decrypt_response_rejects_encrypt_only_iv_and_authenticated_tag() {
    for forbidden_tag in [IV_COUNTER_NONCE, AUTHENTICATED_ENCRYPTION_TAG] {
        let payload = structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(IDENTIFIER_SENTINEL.to_owned()),
            ),
            item(forbidden_tag, Value::byte_string(BODY_SENTINEL.to_vec())),
        ]);
        let message = successful_response(DECRYPT_OPERATION, payload);
        let item = message
            .batch_items()
            .next()
            .expect("the successful response has one batch item");
        validate_and_assert_redacted(DecryptResponse::try_from_response_item(item));
    }
}
