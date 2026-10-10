//! Redaction characterization for KMIP v2.1 Encrypt and Decrypt values from
//! §6.1.11 Tables 196–197 (Decrypt), §6.1.17 Tables 214–215 (Encrypt), and
//! §§7.3, 7.4, 7.8, and 7.9. Traceability: KMIPKIT-0019-FR-009 and SC-004.

use std::error::Error;

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{
    DecryptRequest, DecryptResponse, EncryptRequest, EncryptResponse, OperationData, SecretBytes,
    UniqueIdentifier,
};
use kmipkit_ttlv::Value;

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const SUCCESS: u32 = 0;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const DATA: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE: u32 = 0x0042_003D;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;

const ENCRYPT_REQUEST_DATA: &str = "KMIP_T036_ENCRYPT_REQUEST_DATA_8A71";
const DECRYPT_REQUEST_DATA: &str = "KMIP_T036_DECRYPT_REQUEST_DATA_3C29";
const ENCRYPT_REQUEST_IV: &str = "KMIP_T036_ENCRYPT_REQUEST_IV_47D2";
const DECRYPT_REQUEST_IV: &str = "KMIP_T036_DECRYPT_REQUEST_IV_90B6";
const ENCRYPT_REQUEST_CORRELATION: &str = "KMIP_T036_ENCRYPT_REQUEST_CORRELATION_5F11";
const DECRYPT_REQUEST_CORRELATION: &str = "KMIP_T036_DECRYPT_REQUEST_CORRELATION_62A8";
const ENCRYPT_REQUEST_AAD: &str = "KMIP_T036_ENCRYPT_REQUEST_AAD_1DB4";
const DECRYPT_REQUEST_AAD: &str = "KMIP_T036_DECRYPT_REQUEST_AAD_83E5";
const DECRYPT_REQUEST_TAG: &str = "KMIP_T036_DECRYPT_REQUEST_TAG_A028";
const ENCRYPT_RESPONSE_DATA: &str = "KMIP_T036_ENCRYPT_RESPONSE_DATA_760C";
const DECRYPT_RESPONSE_DATA: &str = "KMIP_T036_DECRYPT_RESPONSE_DATA_2C6A";
const ENCRYPT_RESPONSE_IV: &str = "KMIP_T036_ENCRYPT_RESPONSE_IV_B199";
const ENCRYPT_RESPONSE_CORRELATION: &str = "KMIP_T036_ENCRYPT_RESPONSE_CORRELATION_4E02";
const DECRYPT_RESPONSE_CORRELATION: &str = "KMIP_T036_DECRYPT_RESPONSE_CORRELATION_71D0";
const ENCRYPT_RESPONSE_TAG: &str = "KMIP_T036_ENCRYPT_RESPONSE_TAG_5AA3";
const MALFORMED_RESPONSE_SENTINEL: &str = "KMIP_T036_MALFORMED_RESPONSE_89EF";

fn assert_absent(formatted: &str, sentinels: &[&str], context: &str) {
    for sentinel in sentinels {
        assert!(
            !formatted.contains(sentinel),
            "{context} must redact values"
        );
    }
}

fn assert_secret_redacted(secret: &SecretBytes, expected: &str) {
    assert!(
        secret.with_bytes(|bytes| bytes == expected.as_bytes()),
        "the typed response retains the sentinel bytes"
    );
    let debug = format!("{secret:?}");
    let display = secret.to_string();
    assert_absent(&debug, &[expected], "SecretBytes Debug");
    assert_absent(&display, &[expected], "SecretBytes Display");
}

fn success_response(operation: u32, payload: kmipkit_ttlv::Structure) -> crate::ResponseMessage {
    response_message(operation, SUCCESS, None, None, Some(payload))
}

fn only_batch_item(message: &crate::ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("the redaction fixture contains one response item")
}

fn format_error_chain(error: &(dyn Error + 'static)) -> String {
    let mut formatted = String::new();
    let mut source = Some(error);
    while let Some(cause) = source {
        formatted.push_str(&cause.to_string());
        source = cause.source();
    }
    formatted
}

#[test]
fn encrypt_and_decrypt_request_debug_redacts_each_secret_request_member() {
    let encrypt = EncryptRequest::new(
        Some(UniqueIdentifier::TextString("t036-encrypt-key".to_owned())),
        Some(OperationData::ByteString(SecretBytes::new(
            ENCRYPT_REQUEST_DATA.as_bytes().to_vec(),
        ))),
    )
    .with_iv_counter_nonce(SecretBytes::new(ENCRYPT_REQUEST_IV.as_bytes().to_vec()))
    .with_correlation_value(SecretBytes::new(
        ENCRYPT_REQUEST_CORRELATION.as_bytes().to_vec(),
    ))
    .with_authenticated_encryption_additional_data(SecretBytes::new(
        ENCRYPT_REQUEST_AAD.as_bytes().to_vec(),
    ));
    let encrypt_debug = format!("{encrypt:?}");
    assert_absent(
        &encrypt_debug,
        &[
            ENCRYPT_REQUEST_DATA,
            ENCRYPT_REQUEST_IV,
            ENCRYPT_REQUEST_CORRELATION,
            ENCRYPT_REQUEST_AAD,
        ],
        "Encrypt request Debug",
    );

    let decrypt = DecryptRequest::new(
        Some(UniqueIdentifier::TextString("t036-decrypt-key".to_owned())),
        Some(OperationData::ByteString(SecretBytes::new(
            DECRYPT_REQUEST_DATA.as_bytes().to_vec(),
        ))),
    )
    .with_iv_counter_nonce(SecretBytes::new(DECRYPT_REQUEST_IV.as_bytes().to_vec()))
    .with_correlation_value(SecretBytes::new(
        DECRYPT_REQUEST_CORRELATION.as_bytes().to_vec(),
    ))
    .with_authenticated_encryption_additional_data(SecretBytes::new(
        DECRYPT_REQUEST_AAD.as_bytes().to_vec(),
    ))
    .with_authenticated_encryption_tag(SecretBytes::new(DECRYPT_REQUEST_TAG.as_bytes().to_vec()));
    let decrypt_debug = format!("{decrypt:?}");
    assert_absent(
        &decrypt_debug,
        &[
            DECRYPT_REQUEST_DATA,
            DECRYPT_REQUEST_IV,
            DECRYPT_REQUEST_CORRELATION,
            DECRYPT_REQUEST_AAD,
            DECRYPT_REQUEST_TAG,
        ],
        "Decrypt request Debug",
    );
}

#[test]
fn encrypt_and_decrypt_response_debug_and_secret_display_redact_output_members() {
    let encrypt_message = success_response(
        ENCRYPT_OPERATION,
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string("t036-encrypt-output".to_owned()),
            ),
            item(
                DATA,
                Value::byte_string(ENCRYPT_RESPONSE_DATA.as_bytes().to_vec()),
            ),
            item(
                IV_COUNTER_NONCE,
                Value::byte_string(ENCRYPT_RESPONSE_IV.as_bytes().to_vec()),
            ),
            item(
                CORRELATION_VALUE,
                Value::byte_string(ENCRYPT_RESPONSE_CORRELATION.as_bytes().to_vec()),
            ),
            item(
                AUTHENTICATED_ENCRYPTION_TAG,
                Value::byte_string(ENCRYPT_RESPONSE_TAG.as_bytes().to_vec()),
            ),
        ]),
    );
    let encrypt = EncryptResponse::try_from_response_item(only_batch_item(&encrypt_message))
        .expect("the Encrypt response fixture is well formed");
    let encrypt_debug = format!("{encrypt:?}");
    assert_absent(
        &encrypt_debug,
        &[
            ENCRYPT_RESPONSE_DATA,
            ENCRYPT_RESPONSE_IV,
            ENCRYPT_RESPONSE_CORRELATION,
            ENCRYPT_RESPONSE_TAG,
        ],
        "Encrypt response Debug",
    );
    assert_secret_redacted(
        encrypt.data().expect("Encrypt response Data is present"),
        ENCRYPT_RESPONSE_DATA,
    );
    assert_secret_redacted(
        encrypt
            .iv_counter_nonce()
            .expect("Encrypt response IV is present"),
        ENCRYPT_RESPONSE_IV,
    );
    assert_secret_redacted(
        encrypt
            .correlation_value()
            .expect("Encrypt response Correlation Value is present"),
        ENCRYPT_RESPONSE_CORRELATION,
    );
    assert_secret_redacted(
        encrypt
            .authenticated_encryption_tag()
            .expect("Encrypt response AEAD Tag is present"),
        ENCRYPT_RESPONSE_TAG,
    );

    let decrypt_message = success_response(
        DECRYPT_OPERATION,
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string("t036-decrypt-output".to_owned()),
            ),
            item(
                DATA,
                Value::byte_string(DECRYPT_RESPONSE_DATA.as_bytes().to_vec()),
            ),
            item(
                CORRELATION_VALUE,
                Value::byte_string(DECRYPT_RESPONSE_CORRELATION.as_bytes().to_vec()),
            ),
        ]),
    );
    let decrypt = DecryptResponse::try_from_response_item(only_batch_item(&decrypt_message))
        .expect("the Decrypt response fixture is well formed");
    let decrypt_debug = format!("{decrypt:?}");
    assert_absent(
        &decrypt_debug,
        &[DECRYPT_RESPONSE_DATA, DECRYPT_RESPONSE_CORRELATION],
        "Decrypt response Debug",
    );
    assert_secret_redacted(
        decrypt.data().expect("Decrypt response Data is present"),
        DECRYPT_RESPONSE_DATA,
    );
    assert_secret_redacted(
        decrypt
            .correlation_value()
            .expect("Decrypt response Correlation Value is present"),
        DECRYPT_RESPONSE_CORRELATION,
    );
}

#[test]
fn malformed_encrypt_and_decrypt_response_errors_do_not_echo_payload_values() {
    let cases = [
        (ENCRYPT_OPERATION, MALFORMED_RESPONSE_SENTINEL),
        (DECRYPT_OPERATION, MALFORMED_RESPONSE_SENTINEL),
    ];
    for (operation, sentinel) in cases {
        let message = success_response(
            operation,
            structure([item(DATA, Value::text_string(sentinel.to_owned()))]),
        );
        let error: Box<dyn Error> = if operation == ENCRYPT_OPERATION {
            Box::new(
                EncryptResponse::try_from_response_item(only_batch_item(&message))
                    .expect_err("wrong-type success Data is rejected"),
            )
        } else {
            Box::new(
                DecryptResponse::try_from_response_item(only_batch_item(&message))
                    .expect_err("wrong-type success Data is rejected"),
            )
        };
        let debug = format!("{error:?}");
        let display = error.to_string();
        let chain = format_error_chain(error.as_ref());
        assert_absent(&debug, &[sentinel], "operation error Debug");
        assert_absent(&display, &[sentinel], "operation error Display");
        assert_absent(&chain, &[sentinel], "operation error source chain");
    }
}
