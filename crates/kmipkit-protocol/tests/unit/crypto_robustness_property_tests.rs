//! Bounded parser and malformed-payload properties for KMIP 2.1 cryptographic
//! operations. Source-derived protocol coverage, not official OASIS cases.
//!
//! References: KMIP Specification v2.1 §6.1.11, Table 196; §6.1.17, Table
//! 214; §7.9, Tables 360–361; §§8.6 and 9.17–9.18; §11.46.
//! Traceability: `KMIPKIT-0019-FR-001`, `KMIPKIT-0019-FR-007`,
//! `KMIPKIT-0019-FR-009`, `KMIPKIT-0019-FR-012`, and
//! `KMIPKIT-REQ-SPEC-6.1-001-002`.

use std::fmt;

use kmipkit_ttlv::codec::{CodecLimits, DecodeErrorKind, decode_with_limits};
use kmipkit_ttlv::{Item, ItemType, Structure, Value, ValueView};
use quickcheck::{Arbitrary, Gen, QuickCheck};

use crate::async_operation_fixtures::item;
use crate::{DecryptRequest, EncryptRequest, OperationData, SecretBytes, UniqueIdentifier};

const PROPERTY_SEED: u64 = 0x4b4d_4950_4b49_5432;
const PROPERTY_RUNS: u64 = 256;
const MAX_OPERATION_DATA_BYTES: usize = 256;
const MAX_TTLV_INPUT_BYTES: usize = 4 * 1024;
const MAX_DECODE_BYTES: usize = 512;

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const DATA: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE: u32 = 0x0042_003D;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INIT_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;

const KEY_IDENTIFIER_SENTINEL: &str = "T042_KEY_IDENTIFIER_SENTINEL";
const SECRET_TEXT_SENTINEL: &str = "T042_SECRET_TEXT_SENTINEL_9D31";
const SECRET_BYTES_SENTINEL: &[u8] = b"T042_SECRET_BYTES_SENTINEL_8A27";

#[derive(Clone)]
struct OperationDataCase {
    variant: u8,
    bytes: Vec<u8>,
    enumeration: u32,
    integer: i32,
}

impl fmt::Debug for OperationDataCase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OperationDataCase([REDACTED])")
    }
}

impl Arbitrary for OperationDataCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        let mut bytes = Vec::<u8>::arbitrary(generator);
        bytes.truncate(MAX_OPERATION_DATA_BYTES);
        Self {
            variant: u8::arbitrary(generator),
            bytes,
            enumeration: u32::arbitrary(generator),
            integer: i32::arbitrary(generator),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(std::iter::empty())
    }
}

fn operation_data(case: &OperationDataCase) -> OperationData {
    match case.variant % 3 {
        0 => OperationData::ByteString(SecretBytes::new(case.bytes.clone())),
        1 => OperationData::Enumeration(case.enumeration),
        _ => OperationData::Integer(case.integer),
    }
}

fn data_item_matches(item: &Item, case: &OperationDataCase) -> bool {
    item.tag().raw() == DATA
        && match case.variant % 3 {
            0 => {
                item.item_type() == ItemType::ByteString
                    && item.with_value(|value| {
                        matches!(value, ValueView::ByteString(actual) if *actual == case.bytes)
                    })
            }
            1 => {
                item.item_type() == ItemType::Enumeration
                    && item.with_value(|value| {
                        matches!(value, ValueView::Enumeration(actual) if *actual == case.enumeration)
                    })
            }
            _ => {
                item.item_type() == ItemType::Integer
                    && item.with_value(|value| {
                        matches!(value, ValueView::Integer(actual) if *actual == case.integer)
                    })
            }
        }
}

fn request_payload_preserves_operation_data(payload: &Structure, case: &OperationDataCase) -> bool {
    let view = payload.view();
    let fields = view.children();
    fields.len() == 2
        && fields[0].tag().raw() == UNIQUE_IDENTIFIER
        && fields[1].tag().raw() == DATA
        && data_item_matches(&fields[1], case)
}

#[allow(clippy::needless_pass_by_value)]
fn encrypt_and_decrypt_preserve_arbitrary_data_variants(case: OperationDataCase) -> bool {
    let encrypt_data = operation_data(&case);
    let encrypt_is_redacted = format!("{encrypt_data:?}") == "OperationData([REDACTED])"
        && encrypt_data.to_string() == "KMIP operation data ([REDACTED])";
    let Ok(encrypt_payload) = EncryptRequest::new(
        Some(UniqueIdentifier::TextString(
            KEY_IDENTIFIER_SENTINEL.to_owned(),
        )),
        Some(encrypt_data),
    )
    .to_ttlv_payload() else {
        return false;
    };

    let decrypt_data = operation_data(&case);
    let decrypt_is_redacted = format!("{decrypt_data:?}") == "OperationData([REDACTED])"
        && decrypt_data.to_string() == "KMIP operation data ([REDACTED])";
    let Ok(decrypt_payload) = DecryptRequest::new(
        Some(UniqueIdentifier::TextString(
            KEY_IDENTIFIER_SENTINEL.to_owned(),
        )),
        Some(decrypt_data),
    )
    .to_ttlv_payload() else {
        return false;
    };

    encrypt_is_redacted
        && decrypt_is_redacted
        && request_payload_preserves_operation_data(&encrypt_payload, &case)
        && request_payload_preserves_operation_data(&decrypt_payload, &case)
}

#[test]
fn encrypt_and_decrypt_preserve_arbitrary_operation_data_variants_and_redact_them() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(64, PROPERTY_SEED))
        .tests(PROPERTY_RUNS)
        .quickcheck(
            encrypt_and_decrypt_preserve_arbitrary_data_variants as fn(OperationDataCase) -> bool,
        );
}

#[derive(Clone)]
struct MalformedPayloadCase {
    variant: u8,
    bytes: Vec<u8>,
}

impl fmt::Debug for MalformedPayloadCase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MalformedPayloadCase([REDACTED])")
    }
}

impl Arbitrary for MalformedPayloadCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        let mut bytes = Vec::<u8>::arbitrary(generator);
        bytes.truncate(MAX_OPERATION_DATA_BYTES);
        Self {
            variant: u8::arbitrary(generator),
            bytes,
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(std::iter::empty())
    }
}

fn secret_byte_string(case: &MalformedPayloadCase) -> Value {
    let mut bytes = SECRET_BYTES_SENTINEL.to_vec();
    bytes.extend_from_slice(&case.bytes);
    Value::byte_string(bytes)
}

fn malformed_crypto_payload(case: &MalformedPayloadCase, encrypt: bool) -> Structure {
    let mut payload = Structure::new();
    payload
        .try_push(item(
            UNIQUE_IDENTIFIER,
            Value::text_string(KEY_IDENTIFIER_SENTINEL.to_owned()),
        ))
        .expect("the fixed key identifier fits the malformed request payload");

    match case.variant % 4 {
        0 => payload
            .try_push(item(
                DATA,
                Value::text_string(SECRET_TEXT_SENTINEL.to_owned()),
            ))
            .expect("the malformed Data member fits the request payload"),
        1 | 3 if !encrypt => {
            payload
                .try_push(item(DATA, secret_byte_string(case)))
                .expect("the first repeated Data member fits the request payload");
            payload
                .try_push(item(DATA, secret_byte_string(case)))
                .expect("the repeated Data member fits the request payload");
        }
        1 => {
            payload
                .try_push(item(DATA, secret_byte_string(case)))
                .expect("the first Data member fits the request payload");
            payload
                .try_push(item(DATA, secret_byte_string(case)))
                .expect("the repeated Data member fits the request payload");
        }
        2 => {
            let parameters = crate::async_operation_fixtures::structure([item(
                BLOCK_CIPHER_MODE,
                Value::text_string(SECRET_TEXT_SENTINEL.to_owned()),
            )]);
            payload
                .try_push(item(CRYPTOGRAPHIC_PARAMETERS, Value::structure(parameters)))
                .expect("the malformed parameters fit the request payload");
            payload
                .try_push(item(DATA, secret_byte_string(case)))
                .expect("Data fits the malformed parameter request payload");
        }
        _ => {
            payload
                .try_push(item(DATA, secret_byte_string(case)))
                .expect("Data fits the malformed Encrypt request payload");
            payload
                .try_push(item(AUTHENTICATED_ENCRYPTION_TAG, secret_byte_string(case)))
                .expect("the Encrypt-forbidden tag fits the request payload");
        }
    }
    payload
}

fn malformed_payload_is_rejected_without_secret_diagnostics(
    case: &MalformedPayloadCase,
    encrypt: bool,
) -> bool {
    let payload = malformed_crypto_payload(case, encrypt);
    let error = if encrypt {
        crate::encrypt::validate_request_payload(&payload).err()
    } else {
        crate::decrypt::validate_request_payload(&payload).err()
    };
    let Some(error) = error else {
        return false;
    };
    let display = error.to_string();
    let debug = format!("{error:?}");
    [display.as_str(), debug.as_str()].iter().all(|message| {
        !message.contains(KEY_IDENTIFIER_SENTINEL)
            && !message.contains(SECRET_TEXT_SENTINEL)
            && !message.contains("T042_SECRET_BYTES_SENTINEL_8A27")
    })
}

#[allow(clippy::needless_pass_by_value)]
fn arbitrary_malformed_crypto_payloads_are_rejected_safely(case: MalformedPayloadCase) -> bool {
    malformed_payload_is_rejected_without_secret_diagnostics(&case, true)
        && malformed_payload_is_rejected_without_secret_diagnostics(&case, false)
}

#[test]
fn arbitrary_malformed_encrypt_and_decrypt_payloads_are_rejected_safely() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(64, PROPERTY_SEED ^ 0x0019))
        .tests(PROPERTY_RUNS)
        .quickcheck(
            arbitrary_malformed_crypto_payloads_are_rejected_safely
                as fn(MalformedPayloadCase) -> bool,
        );
}

fn operation_payload(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut payload = Structure::new();
    for item in items {
        payload
            .try_push(item)
            .expect("the deterministic malformed payload remains within model limits");
    }
    payload
}

fn valid_request_prefix() -> [Item; 2] {
    [
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string(KEY_IDENTIFIER_SENTINEL.to_owned()),
        ),
        item(DATA, Value::byte_string(SECRET_BYTES_SENTINEL.to_vec())),
    ]
}

#[test]
fn deterministic_malformed_top_level_payloads_are_rejected_by_both_validators() {
    let invalid_payloads = [
        operation_payload([
            item(UNIQUE_IDENTIFIER, Value::boolean(true)),
            item(DATA, Value::byte_string(SECRET_BYTES_SENTINEL.to_vec())),
        ]),
        operation_payload([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(KEY_IDENTIFIER_SENTINEL.to_owned()),
            ),
            item(CRYPTOGRAPHIC_PARAMETERS, Value::integer(7)),
            item(DATA, Value::byte_string(SECRET_BYTES_SENTINEL.to_vec())),
        ]),
        operation_payload([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(KEY_IDENTIFIER_SENTINEL.to_owned()),
            ),
            item(DATA, Value::text_string(SECRET_TEXT_SENTINEL.to_owned())),
        ]),
        operation_payload([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(KEY_IDENTIFIER_SENTINEL.to_owned()),
            ),
            item(DATA, Value::byte_string(SECRET_BYTES_SENTINEL.to_vec())),
            item(IV_COUNTER_NONCE, Value::integer(7)),
        ]),
        operation_payload([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(KEY_IDENTIFIER_SENTINEL.to_owned()),
            ),
            item(DATA, Value::byte_string(SECRET_BYTES_SENTINEL.to_vec())),
            item(
                CORRELATION_VALUE,
                Value::text_string(SECRET_TEXT_SENTINEL.to_owned()),
            ),
        ]),
        operation_payload([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(KEY_IDENTIFIER_SENTINEL.to_owned()),
            ),
            item(DATA, Value::byte_string(SECRET_BYTES_SENTINEL.to_vec())),
            item(INIT_INDICATOR, Value::enumeration(1)),
        ]),
        operation_payload([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(KEY_IDENTIFIER_SENTINEL.to_owned()),
            ),
            item(DATA, Value::byte_string(SECRET_BYTES_SENTINEL.to_vec())),
            item(FINAL_INDICATOR, Value::integer(1)),
        ]),
        operation_payload([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(KEY_IDENTIFIER_SENTINEL.to_owned()),
            ),
            item(DATA, Value::byte_string(SECRET_BYTES_SENTINEL.to_vec())),
            item(DATA, Value::byte_string(SECRET_BYTES_SENTINEL.to_vec())),
        ]),
    ];

    for payload in invalid_payloads {
        let encrypt = crate::encrypt::validate_request_payload(&payload)
            .expect_err("malformed Encrypt top-level fields are rejected");
        let decrypt = crate::decrypt::validate_request_payload(&payload)
            .expect_err("malformed Decrypt top-level fields are rejected");

        assert_eq!(encrypt.kind(), crate::ProtocolErrorKind::InvalidValue);
        assert_eq!(decrypt.kind(), crate::ProtocolErrorKind::InvalidValue);
        for error in [&encrypt, &decrypt] {
            let display = error.to_string();
            let debug = format!("{error:?}");
            assert!(
                !display.contains(SECRET_TEXT_SENTINEL)
                    && !display.contains("T042_SECRET_BYTES_SENTINEL_8A27")
                    && !debug.contains(SECRET_TEXT_SENTINEL)
                    && !debug.contains("T042_SECRET_BYTES_SENTINEL_8A27"),
                "malformed request validation must not reveal field values"
            );
        }
    }

    let mut encrypt_tag_fields = valid_request_prefix().into_iter().collect::<Vec<_>>();
    encrypt_tag_fields.push(item(
        AUTHENTICATED_ENCRYPTION_TAG,
        Value::byte_string(vec![0xA5]),
    ));
    let encrypt_tag_payload = operation_payload(encrypt_tag_fields);
    assert!(
        crate::encrypt::validate_request_payload(&encrypt_tag_payload).is_err(),
        "Encrypt rejects its response-only Authenticated Encryption Tag"
    );
    assert!(
        crate::decrypt::validate_request_payload(&encrypt_tag_payload).is_ok(),
        "Decrypt accepts the request Authenticated Encryption Tag"
    );
}

#[derive(Clone)]
struct BoundedTtlvInput(Vec<u8>);

impl fmt::Debug for BoundedTtlvInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BoundedTtlvInput([REDACTED])")
    }
}

impl Arbitrary for BoundedTtlvInput {
    fn arbitrary(generator: &mut Gen) -> Self {
        let mut bytes = Vec::<u8>::arbitrary(generator);
        bytes.truncate(MAX_TTLV_INPUT_BYTES - SECRET_BYTES_SENTINEL.len());
        let mut bounded = SECRET_BYTES_SENTINEL.to_vec();
        bounded.extend(bytes);
        Self(bounded)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(std::iter::empty())
    }
}

#[allow(clippy::needless_pass_by_value)]
fn arbitrary_bounded_ttlv_inputs_decode_without_secret_diagnostics(
    input: BoundedTtlvInput,
) -> bool {
    let Ok(limits) = CodecLimits::new(MAX_DECODE_BYTES, 8, 64) else {
        return false;
    };
    let result = decode_with_limits(&input.0, &limits);
    let byte_limit_is_enforced = input.0.len() <= MAX_DECODE_BYTES
        || matches!(&result, Err(error) if error.kind() == DecodeErrorKind::MessageTooLarge);
    let diagnostic_is_redacted = match &result {
        Err(error) => {
            !format!("{error}").contains("T042_SECRET_BYTES_SENTINEL_8A27")
                && !format!("{error:?}").contains("T042_SECRET_BYTES_SENTINEL_8A27")
        }
        Ok(_) => true,
    };
    byte_limit_is_enforced && diagnostic_is_redacted
}

#[test]
fn arbitrary_bounded_ttlv_inputs_never_bypass_limits_or_expose_input_bytes() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(1024, PROPERTY_SEED ^ 0x0021))
        .tests(PROPERTY_RUNS)
        .quickcheck(
            arbitrary_bounded_ttlv_inputs_decode_without_secret_diagnostics
                as fn(BoundedTtlvInput) -> bool,
        );
}

#[test]
fn decoder_rejects_over_limit_input_before_parsing_or_formatting_it() {
    let limits = CodecLimits::new(64, 8, 64)
        .expect("configured decoder limits are below the maximum structure depth");
    let mut input = SECRET_BYTES_SENTINEL.to_vec();
    input.resize(65, 0xA5);
    let result = decode_with_limits(&input, &limits);
    let safe = match result {
        Err(error) => {
            error.kind() == DecodeErrorKind::MessageTooLarge
                && !format!("{error}").contains("T042_SECRET_BYTES_SENTINEL_8A27")
                && !format!("{error:?}").contains("T042_SECRET_BYTES_SENTINEL_8A27")
        }
        Ok(_) => false,
    };
    assert!(safe, "oversized TTLV input was not rejected safely");
}
