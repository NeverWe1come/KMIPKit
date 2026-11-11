//! Fluent cryptographic request builders from KMIPKIT-0102 and KMIP v2.1.
//!
//! Normative sources: KMIP v2.1 §4.16, Table 59; §6.1.11, Table 196;
//! §6.1.17, Table 214; §6.1.24, Table 235; §6.1.32, Table 259;
//! §6.1.33, Table 262; §6.1.55, Table 334; §6.1.56, Table 337; and
//! §§7.3, 7.8, 7.9, 7.14, 7.17, 7.20, 7.38, and 7.39. Traceability:
//! KMIPKIT-0102-FR-001 through KMIPKIT-0102-FR-007 and KMIPKIT-0093-FR-004
//! through KMIPKIT-0093-FR-011.

use kmipkit_protocol::{
    CryptographicParameters, CryptographicParametersBuilder, OperationData, SecretBytes,
    UniqueIdentifier,
};

use crate::FluentBuildError;
use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Structure, Value, ValueView};

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
const DATA_TAG: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE_TAG: u32 = 0x0042_003D;
const CORRELATION_VALUE_TAG: u32 = 0x0042_00D6;
const INIT_INDICATOR_TAG: u32 = 0x0042_00D7;
const FINAL_INDICATOR_TAG: u32 = 0x0042_00D8;
const DIGESTED_DATA_TAG: u32 = 0x0042_0107;
const SIGNATURE_DATA_TAG: u32 = 0x0042_00C3;
const MAC_DATA_TAG: u32 = 0x0042_00C6;
const AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG: u32 = 0x0042_00FE;
const AUTHENTICATED_ENCRYPTION_TAG_TAG: u32 = 0x0042_00FF;
const PENDING_CORRELATION: &[u8] = b"ENCRYPT_PENDING_CORRELATION";
const SECRET_DATA: &[u8] = b"sensitive-crypto-input";

fn assert_operation(item: Result<ClientBatchItem, FluentBuildError>, expected: ClientOperation) {
    let item = item.unwrap_or_else(|error| {
        panic!("{expected:?} builder rejects its typed request: {error:?}");
    });
    match item.request() {
        ClientRequest::Encrypt(_) => assert_eq!(expected, ClientOperation::Encrypt),
        ClientRequest::Decrypt(_) => assert_eq!(expected, ClientOperation::Decrypt),
        ClientRequest::Sign(_) => assert_eq!(expected, ClientOperation::Sign),
        ClientRequest::SignatureVerify(_) => {
            assert_eq!(expected, ClientOperation::SignatureVerify);
        }
        ClientRequest::Mac(_) => assert_eq!(expected, ClientOperation::Mac),
        ClientRequest::MacVerify(_) => assert_eq!(expected, ClientOperation::MacVerify),
        ClientRequest::Hash(_) => assert_eq!(expected, ClientOperation::Hash),
        _ => panic!("builder constructed a different typed request variant"),
    }
}

fn typed_hash_parameters() -> kmipkit_protocol::CryptographicParameters {
    kmipkit_protocol::CryptographicParametersBuilder::new()
        .with_hashing_algorithm(
            kmipkit_protocol::named_values_generated::values::enumeration::hashing_algorithm::SHA_256,
        )
        .build()
        .expect("SHA-256 parameters are a valid caller-selected structure")
}

fn typed_aes_gcm_parameters() -> CryptographicParameters {
    CryptographicParametersBuilder::new()
        .with_cryptographic_algorithm(
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::AES,
        )
        .with_block_cipher_mode(
            kmipkit_protocol::named_values_generated::values::enumeration::block_cipher_mode::GCM,
        )
        .with_iv_length(96)
        .with_tag_length(16)
        .build()
        .expect("the caller-selected AES-GCM parameters are valid")
}

fn typed_signature_parameters() -> CryptographicParameters {
    CryptographicParametersBuilder::new()
        .with_digital_signature_algorithm(
            kmipkit_protocol::named_values_generated::values::enumeration::digital_signature_algorithm::SHA_256_WITH_RSA_ENCRYPTION,
        )
        .with_cryptographic_algorithm(
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::RSA,
        )
        .build()
        .expect("the caller-selected RSA signature parameters are valid")
}

fn typed_hmac_parameters() -> CryptographicParameters {
    CryptographicParametersBuilder::new()
        .with_cryptographic_algorithm(
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::HMAC_SHA256,
        )
        .build()
        .expect("the caller-selected HMAC parameters are valid")
}

fn assert_payload_tags(payload: &Structure, expected: &[u32]) {
    let actual = payload
        .view()
        .children()
        .iter()
        .map(|item| item.tag().raw())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn assert_payload_bytes(payload: &Structure, tag: u32, expected: &[u8]) {
    let actual = payload
        .view()
        .children()
        .iter()
        .find(|item| item.tag().raw() == tag)
        .expect("the expected byte-string field is present")
        .with_value(|value| match value {
            ValueView::ByteString(bytes) => Some(bytes.to_vec()),
            _ => None,
        });
    assert_eq!(actual.as_deref(), Some(expected));
}

fn assert_payload_text(payload: &Structure, tag: u32, expected: &str) {
    let actual = payload
        .view()
        .children()
        .iter()
        .find(|item| item.tag().raw() == tag)
        .expect("the expected text field is present")
        .with_value(|value| match value {
            ValueView::TextString(text) => Some(text.to_owned()),
            _ => None,
        });
    assert_eq!(actual.as_deref(), Some(expected));
}

fn assert_payload_boolean(payload: &Structure, tag: u32, expected: bool) {
    let actual = payload
        .view()
        .children()
        .iter()
        .find(|item| item.tag().raw() == tag)
        .expect("the expected boolean field is present")
        .with_value(|value| match value {
            ValueView::Boolean(value) => Some(*value),
            _ => None,
        });
    assert_eq!(actual, Some(expected));
}

#[test]
fn encrypt_builder_preserves_operation_specific_request_fields() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let encrypt = client
        .request()
        .encrypt()
        .unique_identifier(UniqueIdentifier::from("encrypt-key"))
        .cryptographic_parameters(typed_aes_gcm_parameters())
        .data_value(OperationData::ByteString(SecretBytes::new(
            b"plain".to_vec(),
        )))
        .iv_counter_nonce(b"nonce-12byte".to_vec())
        .authenticated_encryption_additional_data(b"aad".to_vec())
        .init_indicator(true)
        .final_indicator(true)
        .build()
        .expect("caller-selected Encrypt fields pass typed validation")
        .to_ttlv_payload()
        .expect("the typed Encrypt request preserves its fields");
    assert_payload_tags(
        &encrypt,
        &[
            UNIQUE_IDENTIFIER_TAG,
            CRYPTOGRAPHIC_PARAMETERS_TAG,
            DATA_TAG,
            IV_COUNTER_NONCE_TAG,
            INIT_INDICATOR_TAG,
            FINAL_INDICATOR_TAG,
            AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG,
        ],
    );
    assert_payload_text(&encrypt, UNIQUE_IDENTIFIER_TAG, "encrypt-key");
    assert_payload_bytes(&encrypt, DATA_TAG, b"plain");
    assert_payload_bytes(&encrypt, IV_COUNTER_NONCE_TAG, b"nonce-12byte");
    assert_payload_bytes(
        &encrypt,
        AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG,
        b"aad",
    );
    assert_payload_boolean(&encrypt, INIT_INDICATOR_TAG, true);
    assert_payload_boolean(&encrypt, FINAL_INDICATOR_TAG, true);
}

#[test]
fn decrypt_builder_preserves_operation_specific_request_fields() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let decrypt = client
        .request()
        .decrypt()
        .unique_identifier(UniqueIdentifier::from("decrypt-key"))
        .cryptographic_parameters_structure(typed_aes_gcm_parameters().into_ttlv())
        .data(b"ciphertext".to_vec())
        .iv_counter_nonce(b"nonce-12byte".to_vec())
        .authenticated_encryption_additional_data(b"aad".to_vec())
        .authenticated_encryption_tag(b"tag-16-bytes!!!!".to_vec())
        .init_indicator(true)
        .final_indicator(true)
        .build()
        .expect("caller-selected Decrypt fields pass typed validation")
        .to_ttlv_payload()
        .expect("the typed Decrypt request preserves its fields");
    assert_payload_text(&decrypt, UNIQUE_IDENTIFIER_TAG, "decrypt-key");
    assert_payload_bytes(&decrypt, DATA_TAG, b"ciphertext");
    assert_payload_bytes(&decrypt, IV_COUNTER_NONCE_TAG, b"nonce-12byte");
    assert_payload_bytes(
        &decrypt,
        AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG,
        b"aad",
    );
    assert_payload_bytes(
        &decrypt,
        AUTHENTICATED_ENCRYPTION_TAG_TAG,
        b"tag-16-bytes!!!!",
    );
    assert_payload_boolean(&decrypt, INIT_INDICATOR_TAG, true);
    assert_payload_boolean(&decrypt, FINAL_INDICATOR_TAG, true);
}

#[test]
fn sign_builder_preserves_operation_specific_request_fields() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let sign = client
        .request()
        .sign()
        .unique_identifier(UniqueIdentifier::from("sign-key"))
        .cryptographic_parameters(typed_signature_parameters())
        .data(b"message".to_vec())
        .digested_data(b"digest".to_vec())
        .correlation_value(b"sign-correlation".to_vec())
        .init_indicator(false)
        .final_indicator(true)
        .build()
        .expect("caller-selected Sign fields pass typed validation")
        .to_ttlv_payload()
        .expect("the typed Sign request preserves its fields");
    assert_payload_text(&sign, UNIQUE_IDENTIFIER_TAG, "sign-key");
    assert_payload_bytes(&sign, DATA_TAG, b"message");
    assert_payload_bytes(&sign, DIGESTED_DATA_TAG, b"digest");
    assert_payload_bytes(&sign, CORRELATION_VALUE_TAG, b"sign-correlation");
    assert_payload_boolean(&sign, INIT_INDICATOR_TAG, false);
    assert_payload_boolean(&sign, FINAL_INDICATOR_TAG, true);
}

#[test]
fn signature_verify_builder_preserves_operation_specific_request_fields() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let signature_verify = client
        .request()
        .signature_verify()
        .unique_identifier(UniqueIdentifier::from("verify-key"))
        .cryptographic_parameters(typed_signature_parameters())
        .data_value(OperationData::ByteString(SecretBytes::new(
            b"message".to_vec(),
        )))
        .digested_data(b"digest".to_vec())
        .signature_data(b"signature".to_vec())
        .correlation_value(b"verify-correlation".to_vec())
        .init_indicator(false)
        .final_indicator(true)
        .build()
        .expect("caller-selected Signature Verify fields pass typed validation")
        .to_ttlv_payload()
        .expect("the typed Signature Verify request preserves its fields");
    assert_payload_text(&signature_verify, UNIQUE_IDENTIFIER_TAG, "verify-key");
    assert_payload_bytes(&signature_verify, DATA_TAG, b"message");
    assert_payload_bytes(&signature_verify, DIGESTED_DATA_TAG, b"digest");
    assert_payload_bytes(&signature_verify, SIGNATURE_DATA_TAG, b"signature");
    assert_payload_bytes(
        &signature_verify,
        CORRELATION_VALUE_TAG,
        b"verify-correlation",
    );
}

#[test]
fn mac_builder_preserves_operation_specific_request_fields() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let mac = client
        .request()
        .mac()
        .unique_identifier(UniqueIdentifier::from("mac-key"))
        .cryptographic_parameters(typed_hmac_parameters())
        .data(b"mac-message".to_vec())
        .correlation_value(b"mac-correlation".to_vec())
        .init_indicator(false)
        .final_indicator(true)
        .build()
        .expect("caller-selected MAC fields pass typed validation")
        .to_ttlv_payload()
        .expect("the typed MAC request preserves its fields");
    assert_payload_text(&mac, UNIQUE_IDENTIFIER_TAG, "mac-key");
    assert_payload_bytes(&mac, DATA_TAG, b"mac-message");
    assert_payload_bytes(&mac, CORRELATION_VALUE_TAG, b"mac-correlation");
}

#[test]
fn mac_verify_builder_preserves_operation_specific_request_fields() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let mac_verify = client
        .request()
        .mac_verify()
        .unique_identifier(UniqueIdentifier::from("mac-verify-key"))
        .cryptographic_parameters(typed_hmac_parameters())
        .data_value(OperationData::ByteString(SecretBytes::new(
            b"message".to_vec(),
        )))
        .mac_data(b"mac-value".to_vec())
        .build()
        .expect("caller-selected MAC Verify fields pass typed validation")
        .to_ttlv_payload()
        .expect("the typed MAC Verify request preserves its fields");
    assert_payload_text(&mac_verify, UNIQUE_IDENTIFIER_TAG, "mac-verify-key");
    assert_payload_bytes(&mac_verify, DATA_TAG, b"message");
    assert_payload_bytes(&mac_verify, MAC_DATA_TAG, b"mac-value");
}

#[test]
fn hash_builder_preserves_operation_specific_request_fields() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let hash = client
        .request()
        .hash()
        .cryptographic_parameters(typed_hash_parameters())
        .data_value(OperationData::ByteString(SecretBytes::new(
            b"hash-me".to_vec(),
        )))
        .correlation_value(b"hash-correlation".to_vec())
        .init_indicator(false)
        .final_indicator(true)
        .build()
        .expect("caller-selected Hash fields pass typed validation")
        .to_ttlv_payload()
        .expect("the typed Hash request preserves its fields");
    assert_payload_bytes(&hash, DATA_TAG, b"hash-me");
    assert_payload_bytes(&hash, CORRELATION_VALUE_TAG, b"hash-correlation");
    assert_payload_boolean(&hash, INIT_INDICATOR_TAG, false);
    assert_payload_boolean(&hash, FINAL_INDICATOR_TAG, true);
}

#[test]
fn cryptographic_constructors_create_the_existing_typed_request_variants() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .encrypt()
            .unique_identifier(UniqueIdentifier::from("encrypt-key"))
            .data(b"cleartext".to_vec())
            .to_batch_item(),
        ClientOperation::Encrypt,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .decrypt()
            .unique_identifier(UniqueIdentifier::from("decrypt-key"))
            .data(b"ciphertext".to_vec())
            .to_batch_item(),
        ClientOperation::Decrypt,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .sign()
            .unique_identifier(UniqueIdentifier::from("sign-key"))
            .data(b"message".to_vec())
            .to_batch_item(),
        ClientOperation::Sign,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .signature_verify()
            .unique_identifier(UniqueIdentifier::from("verify-key"))
            .data(b"message".to_vec())
            .signature_data(b"signature".to_vec())
            .to_batch_item(),
        ClientOperation::SignatureVerify,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .mac()
            .unique_identifier(UniqueIdentifier::from("mac-key"))
            .data(b"message".to_vec())
            .to_batch_item(),
        ClientOperation::Mac,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .mac_verify()
            .unique_identifier(UniqueIdentifier::from("mac-verify-key"))
            .data(b"message".to_vec())
            .mac_data(b"mac".to_vec())
            .to_batch_item(),
        ClientOperation::MacVerify,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .hash()
            .cryptographic_parameters(typed_hash_parameters())
            .data(b"message".to_vec())
            .to_batch_item(),
        ClientOperation::Hash,
    );
}

#[test]
fn cryptographic_builders_preserve_explicit_ttlv_expert_parameters() {
    let hashing_algorithm_tag = kmipkit_ttlv::RawTag::new(0x0042_0038)
        .expect("Hashing Algorithm has a nonzero KMIP identifier")
        .try_checked()
        .expect("Hashing Algorithm is a valid assigned KMIP tag");
    let extension_tag = kmipkit_ttlv::RawTag::new(0x0054_1234)
        .expect("the vendor extension tag has a nonzero KMIP identifier")
        .try_checked()
        .expect("the vendor extension tag is structurally valid");
    let parameters = test_structure([
        kmipkit_ttlv::Item::new(
            hashing_algorithm_tag,
            kmipkit_ttlv::Value::enumeration(
                kmipkit_protocol::named_values_generated::values::enumeration::hashing_algorithm::SHA_256,
            ),
        )
        .expect("the required hashing parameter is structurally valid"),
        kmipkit_ttlv::Item::new(
            extension_tag,
            kmipkit_ttlv::Value::byte_string(b"vendor-parameter".to_vec()),
        )
        .expect("the explicit extension parameter is structurally valid"),
    ]);
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .hash()
        .cryptographic_parameters_structure(parameters)
        .data(b"message".to_vec())
        .build()
        .expect("the expert Structure passes through typed Hash validation");
    let payload = request
        .to_ttlv_payload()
        .expect("the typed Hash model builds the payload");
    assert_eq!(payload.view().children()[0].tag().raw(), 0x0042_002B);
}

#[test]
fn locally_invalid_cryptographic_requests_are_not_sent() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .hash()
        .data(b"message".to_vec())
        .execute(&CodecLimits::defaults())
        .expect_err("Hash requires caller-selected Cryptographic Parameters");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);

    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .encrypt()
        .data(b"message".to_vec())
        .correlation_value(b"part-1".to_vec())
        .init_indicator(false)
        .authenticated_encryption_additional_data(b"aad".to_vec())
        .execute(&CodecLimits::defaults())
        .expect_err("AEAD additional data is invalid on a non-initial part");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);

    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .encrypt()
        .data(b"plaintext".to_vec())
        .with_options(
            RequestOptions::default()
                .with_protocol_version(kmipkit_protocol::ProtocolVersion::from_raw(1, 4)),
        )
        .execute(&CodecLimits::defaults())
        .expect_err("an unsupported KMIP version remains behind the typed gate");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn cryptographic_builder_expert_parameter_paths_validate() {
    assert_eq!(
        FluentBuildError::MissingHashCryptographicParameters.to_string(),
        "Hash requires caller-selected Cryptographic Parameters"
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    client
        .request()
        .encrypt()
        .cryptographic_parameters_structure(typed_aes_gcm_parameters().into_ttlv())
        .data(b"plaintext".to_vec())
        .build()
        .expect("Encrypt accepts an explicitly typed expert parameter structure");

    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .decrypt()
        .unique_identifier(UniqueIdentifier::from("decrypt-key"))
        .data(b"ciphertext".to_vec())
        .execute(&CodecLimits::defaults())
        .expect_err("Decrypt reaches its typed client route");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 1, "{error:?}");

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    client
        .request()
        .decrypt()
        .cryptographic_parameters(typed_aes_gcm_parameters())
        .data(b"ciphertext".to_vec())
        .build()
        .expect("Decrypt accepts caller-selected cryptographic parameters");

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    client
        .request()
        .decrypt()
        .data(b"multipart-ciphertext".to_vec())
        .correlation_value(b"decrypt-correlation".to_vec())
        .init_indicator(false)
        .build()
        .expect("Decrypt accepts the explicit multipart continuation shape");
}

#[test]
fn decrypt_sign_and_signature_verify_execute_the_typed_route() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .decrypt()
        .unique_identifier(UniqueIdentifier::from("decrypt-key"))
        .data(b"ciphertext".to_vec())
        .execute(&CodecLimits::defaults())
        .expect_err("Decrypt reaches its typed client route");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 1);

    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .sign()
        .unique_identifier(UniqueIdentifier::from("sign-key"))
        .cryptographic_parameters_structure(typed_signature_parameters().into_ttlv())
        .data_value(OperationData::ByteString(SecretBytes::new(
            b"message".to_vec(),
        )))
        .execute(&CodecLimits::defaults())
        .expect_err("Sign reaches its typed client route");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 1);

    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .signature_verify()
        .unique_identifier(UniqueIdentifier::from("verify-key"))
        .cryptographic_parameters_structure(typed_signature_parameters().into_ttlv())
        .data(b"message".to_vec())
        .signature_data(b"signature".to_vec())
        .execute(&CodecLimits::defaults())
        .expect_err("Signature Verify reaches its typed client route");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn mac_mac_verify_and_hash_execute_the_typed_route() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .mac()
        .unique_identifier(UniqueIdentifier::from("mac-key"))
        .cryptographic_parameters_structure(typed_hmac_parameters().into_ttlv())
        .data_value(OperationData::ByteString(SecretBytes::new(
            b"message".to_vec(),
        )))
        .execute(&CodecLimits::defaults())
        .expect_err("MAC reaches its typed client route");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 1);

    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .mac_verify()
        .unique_identifier(UniqueIdentifier::from("mac-verify-key"))
        .cryptographic_parameters_structure(typed_hmac_parameters().into_ttlv())
        .data(b"message".to_vec())
        .mac_data(b"expected-mac".to_vec())
        .correlation_value(b"mac-correlation".to_vec())
        .init_indicator(true)
        .final_indicator(true)
        .execute(&CodecLimits::defaults())
        .expect_err("MAC Verify reaches its typed client route");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 1);

    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .hash()
        .cryptographic_parameters(typed_hash_parameters())
        .data(b"message".to_vec())
        .execute(&CodecLimits::defaults())
        .expect_err("Hash reaches its typed client route");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn cryptographic_builders_accept_explicit_options_and_async_headers() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    client
        .request()
        .decrypt()
        .data(b"ciphertext".to_vec())
        .with_options(RequestOptions::default())
        .with_asynchronous_indicator(1)
        .build()
        .expect("Decrypt accepts exchange-local options and the explicit async header");

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    client
        .request()
        .sign()
        .data(b"message".to_vec())
        .with_options(RequestOptions::default())
        .with_asynchronous_indicator(1)
        .build()
        .expect("Sign accepts exchange-local options and the explicit async header");

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    client
        .request()
        .signature_verify()
        .data(b"message".to_vec())
        .signature_data(b"signature".to_vec())
        .with_options(RequestOptions::default())
        .with_asynchronous_indicator(1)
        .build()
        .expect("Signature Verify accepts exchange-local options and the explicit async header");

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    client
        .request()
        .mac()
        .data(b"message".to_vec())
        .with_options(RequestOptions::default())
        .with_asynchronous_indicator(1)
        .build()
        .expect("MAC accepts exchange-local options and the explicit async header");

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    client
        .request()
        .mac_verify()
        .data(b"message".to_vec())
        .mac_data(b"expected-mac".to_vec())
        .with_options(RequestOptions::default())
        .with_asynchronous_indicator(1)
        .build()
        .expect("MAC Verify accepts exchange-local options and the explicit async header");

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    client
        .request()
        .hash()
        .cryptographic_parameters(typed_hash_parameters())
        .data(b"message".to_vec())
        .with_options(RequestOptions::default())
        .with_asynchronous_indicator(1)
        .build()
        .expect("Hash accepts exchange-local options and the explicit async header");
}

#[test]
fn fluent_cryptographic_requests_redact_moved_secret_data() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .encrypt()
        .data(SECRET_DATA.to_vec())
        .build()
        .expect("Encrypt accepts caller-supplied byte data");
    assert!(!format!("{request:?}").contains("sensitive-crypto-input"));

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .signature_verify()
        .data(SECRET_DATA.to_vec())
        .signature_data(SECRET_DATA.to_vec())
        .build()
        .expect("Signature Verify accepts caller-supplied data");
    assert!(!format!("{request:?}").contains("sensitive-crypto-input"));
}

#[test]
fn encrypt_builder_executes_once_and_preserves_success_failure_and_pending_results() {
    let success_payload = test_structure([
        test_item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("execution-key".to_owned()),
        ),
        test_item(DATA_TAG, Value::byte_string(b"ciphertext".to_vec())),
    ]);
    let success_response = asynchronous_response_bytes(
        ENCRYPT_OPERATION,
        SUCCESS,
        None,
        None,
        Some(success_payload),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: success_response,
        request_write_chunks: Vec::new(),
    });
    let success = client
        .request()
        .encrypt()
        .unique_identifier(UniqueIdentifier::from("execution-key"))
        .data(b"plaintext".to_vec())
        .execute(&CodecLimits::defaults())
        .expect("the existing typed route accepts a valid Encrypt response");
    assert_eq!(success.outcome().operation(), ClientOperation::Encrypt);
    assert_eq!(success.outcome().result().status().raw(), SUCCESS);
    assert_eq!(fake.borrow().exchange_count(), 1);

    let failure_response = asynchronous_response_bytes(
        ENCRYPT_OPERATION,
        OPERATION_FAILED,
        Some(OBJECT_NOT_FOUND),
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: failure_response,
        request_write_chunks: Vec::new(),
    });
    let failure = client
        .request()
        .encrypt()
        .unique_identifier(UniqueIdentifier::from("execution-key"))
        .data(b"plaintext".to_vec())
        .execute(&CodecLimits::defaults())
        .expect("the existing typed route returns KMIP Failure results");
    assert_eq!(failure.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(fake.borrow().exchange_count(), 1);

    let pending_response = asynchronous_response_bytes(
        ENCRYPT_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: pending_response,
        request_write_chunks: Vec::new(),
    });
    let pending = client
        .request()
        .encrypt()
        .unique_identifier(UniqueIdentifier::from("execution-key"))
        .data(b"plaintext".to_vec())
        .with_asynchronous_indicator(1)
        .execute(&CodecLimits::defaults())
        .expect("the existing typed route returns KMIP Pending results");
    let ClientBatchOutcome::Pending(pending_result) = pending.outcome() else {
        panic!("Encrypt Pending remains visible to the caller");
    };
    assert_eq!(
        pending_result.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
