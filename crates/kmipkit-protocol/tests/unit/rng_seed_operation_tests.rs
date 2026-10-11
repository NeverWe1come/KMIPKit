//! Source-derived RNG Seed contracts from OASIS KMIP v2.1 §6.1.50,
//! Tables 319–321. These tests do not claim an official Test Case pass.
//!
//! Traceability: FR-001–FR-008, `KMIPKIT-REQ-SPEC-6.1.50-001`,
//! `KMIPKIT-ELEM-OP-C2S-RNG-SEED`,
//! `KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-SEED-00000026`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-10-DATA-LENGTH`, and
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA`.

use crate::async_operation_fixtures::response_message;
use crate::cryptographic_operation_test_support::structure_contains_tag;
use crate::operation_test_support::item;
use crate::query_ping_fixtures::{
    response_item as response_item_with_text, response_message as response_message_with_text,
};
use crate::{RngSeedError, RngSeedRequest, RngSeedResponse, SecretBytes};
use kmipkit_ttlv::{Structure, Value, ValueView};
use quickcheck::{Arbitrary, Gen, QuickCheck};

const RNG_SEED_OPERATION: u32 = 0x0000_0026;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const PENDING: u32 = 2;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const DATA: u32 = 0x0042_00C2;
const DATA_LENGTH: u32 = 0x0042_00D5;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const PROPERTY_SEED: u64 = 0x4b4d_4950_4b49_5438;

fn payload(items: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    let mut payload = Structure::new();
    for (raw_tag, value) in items {
        payload
            .try_push(item(raw_tag, value))
            .expect("the source-derived table fixture fits TTLV structure limits");
    }
    payload
}

fn response_item(message: &crate::ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("the fixture contains one validated RNG Seed response item")
}

#[derive(Clone, Debug)]
struct SeedDataCase(Vec<u8>);

impl Arbitrary for SeedDataCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        Self(Vec::<u8>::arbitrary(generator))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(Self))
    }
}

#[allow(clippy::needless_pass_by_value)]
fn request_preserves_arbitrary_table_319_seed_bytes(case: SeedDataCase) -> bool {
    let expected = case.0.clone();
    let Ok(payload) = RngSeedRequest::new(SecretBytes::new(case.0)).to_ttlv_payload() else {
        return false;
    };
    let view = payload.view();
    let fields = view.children();
    fields.len() == 1
        && fields[0].tag().raw() == DATA
        && fields[0].with_value(
            |value| matches!(value, ValueView::ByteString(actual) if *actual == expected),
        )
}

#[test]
fn request_encodes_required_table_319_data_and_redacts_secret_bytes() {
    let secret = b"rng-seed-private".to_vec();
    let request = RngSeedRequest::new(SecretBytes::new(secret.clone()));
    let debug = format!("{request:?}");
    let payload = request
        .to_ttlv_payload()
        .expect("Table 319 contains one required Data Byte String");
    let view = payload.view();
    let fields = view.children();

    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), DATA);
    assert!(
        fields[0]
            .with_value(|value| matches!(value, ValueView::ByteString(actual) if actual == secret))
    );
    assert!(!debug.contains("rng-seed-private"));
    assert!(!debug.contains("private"));
}

#[test]
fn request_preserves_arbitrary_seed_bytes_including_empty_and_non_utf8() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(32, PROPERTY_SEED))
        .tests(256)
        .quickcheck(request_preserves_arbitrary_table_319_seed_bytes as fn(SeedDataCase) -> bool);
}

#[test]
fn successful_response_preserves_zero_data_length_and_unknown_fields() {
    let message = response_message(
        RNG_SEED_OPERATION,
        SUCCESS,
        None,
        None,
        Some(payload([
            (DATA_LENGTH, Value::integer(0)),
            (
                UNKNOWN_VENDOR_TAG,
                Value::text_string("extension".to_owned()),
            ),
        ])),
    );
    let response = RngSeedResponse::try_from_response_item(response_item(&message))
        .expect("§6.1.50 and KMIPKIT-REQ-SPEC-6.1.50-001 accept zero bytes used");

    assert_eq!(response.result().status().raw(), SUCCESS);
    assert_eq!(response.data_length(), Some(0));
    drop(message);
    assert!(response.with_ttlv(|ttlv| structure_contains_tag(&ttlv, UNKNOWN_VENDOR_TAG)));
}

#[derive(Clone, Debug)]
struct DataLengthCase(i32);

impl Arbitrary for DataLengthCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        Self(i32::arbitrary(generator))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(Self))
    }
}

#[allow(clippy::needless_pass_by_value)]
fn success_preserves_all_integer_data_lengths(case: DataLengthCase) -> bool {
    let message = response_message(
        RNG_SEED_OPERATION,
        SUCCESS,
        None,
        None,
        Some(payload([(DATA_LENGTH, Value::integer(case.0))])),
    );
    RngSeedResponse::try_from_response_item(response_item(&message))
        .is_ok_and(|response| response.data_length() == Some(case.0))
}

#[test]
fn response_roundtrips_all_signed_integer_data_lengths() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(32, PROPERTY_SEED))
        .tests(256)
        .quickcheck(success_preserves_all_integer_data_lengths as fn(DataLengthCase) -> bool);
}

#[test]
fn successful_response_rejects_missing_wrong_type_and_duplicate_data_length() {
    let cases = [
        Structure::new(),
        payload([(DATA_LENGTH, Value::byte_string(vec![0]))]),
        payload([
            (DATA_LENGTH, Value::integer(0)),
            (DATA_LENGTH, Value::integer(1)),
        ]),
    ];

    for payload in cases {
        let message = response_message(RNG_SEED_OPERATION, SUCCESS, None, None, Some(payload));
        assert!(RngSeedResponse::try_from_response_item(response_item(&message)).is_err());
    }
}

#[test]
fn failure_response_preserves_common_result_without_seed_count() {
    let message = response_message_with_text(
        RNG_SEED_OPERATION,
        OPERATION_FAILED,
        Some(GENERAL_FAILURE),
        Some("seed request refused"),
        None,
    );
    let response = RngSeedResponse::try_from_response_item(response_item_with_text(&message))
        .expect("Table 321 failure is represented by the common result");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(crate::ResultReason::raw),
        Some(GENERAL_FAILURE)
    );
    assert_eq!(
        response
            .result()
            .message()
            .map(crate::ResultMessage::as_str),
        Some("seed request refused")
    );
    assert!(response.data_length().is_none());
    assert!(!format!("{response:?}").contains("seed request refused"));
}

#[test]
fn pending_response_is_explicit_and_preserves_correlation_without_polling() {
    let correlation = [0x00, 0x80, 0xff];
    let message = response_message(
        RNG_SEED_OPERATION,
        PENDING,
        None,
        Some(&correlation),
        Some(Structure::new()),
    );
    let response = RngSeedResponse::try_from_pending_response_item(response_item(&message))
        .expect("caller-controlled async handling retains the Pending response");

    assert_eq!(response.result().status().raw(), PENDING);
    assert!(response.data_length().is_none());
    assert!(
        response.with_ttlv(|ttlv| structure_contains_tag(&ttlv, ASYNCHRONOUS_CORRELATION_VALUE))
    );
    assert!(RngSeedResponse::try_from_response_item(response_item(&message)).is_err());
}

#[test]
fn response_debug_and_conversion_errors_do_not_reveal_seed_contents() {
    let seed = [0xA5, 0x5A];
    let message = response_message(
        RNG_SEED_OPERATION,
        SUCCESS,
        None,
        None,
        Some(payload([(
            DATA_LENGTH,
            Value::integer(i32::try_from(seed.len()).expect("test sentinel length fits i32")),
        )])),
    );
    let response = RngSeedResponse::try_from_response_item(response_item(&message))
        .expect("the Table 320 length is a valid Integer");
    assert!(!format!("{response:?}").contains("165"));
    assert!(!format!("{response:?}").contains("90"));

    let error = RngSeedError::new(
        "RNG Seed",
        crate::CryptographicOperationErrorKind::MalformedPayload,
    );
    assert!(!error.to_string().contains("A5"));
    assert!(!error.to_string().contains("5A"));
}
