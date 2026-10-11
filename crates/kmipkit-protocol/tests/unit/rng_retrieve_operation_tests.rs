//! Source-derived RNG Retrieve contracts from OASIS KMIP v2.1 §6.1.49,
//! Tables 316–318. These tests do not claim an official Test Case pass.
//!
//! Traceability: FR-001–FR-008, `KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE`,
//! `KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-RETRIEVE-00000025`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-10-DATA-LENGTH`, and
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA`.

use crate::async_operation_fixtures::response_message;
use crate::cryptographic_operation_test_support::{output_bytes, structure_contains_tag};
use crate::operation_test_support::item;
use crate::{ResultMessage, RngRetrieveError, RngRetrieveRequest, RngRetrieveResponse};
use kmipkit_ttlv::{Structure, Value, ValueView};
use quickcheck::{Arbitrary, Gen, QuickCheck};
use std::error::Error;

const RNG_RETRIEVE_OPERATION: u32 = 0x0000_0025;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const PENDING: u32 = 2;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const DATA_LENGTH: u32 = 0x0042_00D5;
const DATA: u32 = 0x0042_00C2;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const PROPERTY_SEED: u64 = 0x4b4d_4950_4b49_5437;

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
        .expect("the fixture contains one validated RNG Retrieve response item")
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
fn request_roundtrips_table_316_data_length(case: DataLengthCase) -> bool {
    let Ok(payload) = RngRetrieveRequest::new(case.0).to_ttlv_payload() else {
        return false;
    };
    let fields = payload.view().children();
    fields.len() == 1
        && fields[0].tag().raw() == DATA_LENGTH
        && fields[0].with_value(|value| matches!(value, ValueView::Integer(actual) if *actual == case.0))
}

#[test]
fn request_encodes_required_table_316_data_length_as_integer() {
    let request = RngRetrieveRequest::new(48);
    let payload = request
        .to_ttlv_payload()
        .expect("Table 316 contains one required Data Length Integer");
    let fields = payload.view().children();

    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), DATA_LENGTH);
    assert!(fields[0].with_value(|value| matches!(value, ValueView::Integer(48))));
}

#[test]
fn request_preserves_all_caller_supplied_integer_values() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(32, PROPERTY_SEED))
        .tests(256)
        .quickcheck(request_roundtrips_table_316_data_length as fn(DataLengthCase) -> bool);
}

#[test]
fn successful_response_requires_and_exposes_secret_data_and_unknown_fields() {
    let output = [0x00, 0x7f, 0x80, 0xff];
    let message = response_message(
        RNG_RETRIEVE_OPERATION,
        SUCCESS,
        None,
        None,
        Some(payload([
            (DATA, Value::byte_string(output.to_vec())),
            (UNKNOWN_VENDOR_TAG, Value::integer(-7)),
        ])),
    );
    let response = RngRetrieveResponse::try_from_response_item(response_item(&message))
        .expect("Table 317 requires returned Data and preserves unknown members");

    assert_eq!(response.result().status().raw(), SUCCESS);
    output_bytes(response.data(), &output);
    drop(message);
    assert!(response.with_ttlv(|ttlv| structure_contains_tag(&ttlv, UNKNOWN_VENDOR_TAG)));
    assert!(!format!("{response:?}").contains("128"));
}

#[test]
fn successful_response_rejects_missing_wrong_type_and_duplicate_data() {
    let cases = [
        Structure::new(),
        payload([(DATA, Value::integer(7))]),
        payload([
            (DATA, Value::byte_string(vec![1])),
            (DATA, Value::byte_string(vec![2])),
        ]),
    ];

    for payload in cases {
        let message = response_message(
            RNG_RETRIEVE_OPERATION,
            SUCCESS,
            None,
            None,
            Some(payload),
        );
        let error = RngRetrieveResponse::try_from_response_item(response_item(&message))
            .expect_err("malformed Table 317 payloads are rejected");
        assert_eq!(error, RngRetrieveError::new("RNG Retrieve", crate::CryptographicOperationErrorKind::MalformedPayload));
        assert!(!error.to_string().contains("7"));
    }
}

#[test]
fn failure_response_preserves_common_result_without_success_data() {
    let message = response_message(
        RNG_RETRIEVE_OPERATION,
        OPERATION_FAILED,
        Some(GENERAL_FAILURE),
        Some(b"RNG request denied"),
        None,
    );
    let response = RngRetrieveResponse::try_from_response_item(response_item(&message))
        .expect("Table 318 failure is represented by the common result");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(response.result().reason().map(crate::ResultReason::raw), Some(GENERAL_FAILURE));
    assert_eq!(response.result().message().map(ResultMessage::as_str), Some("RNG request denied"));
    assert!(response.data().is_none());
}

#[test]
fn pending_response_is_explicit_and_preserves_correlation_without_polling() {
    let correlation = [0x00, 0x80, 0xff];
    let message = response_message(
        RNG_RETRIEVE_OPERATION,
        PENDING,
        None,
        Some(&correlation),
        None,
    );
    let response = RngRetrieveResponse::try_from_pending_response_item(response_item(&message))
        .expect("caller-controlled async handling retains the Pending response");

    assert_eq!(response.result().status().raw(), PENDING);
    assert!(response.data().is_none());
    assert!(response.with_ttlv(|ttlv| structure_contains_tag(
        &ttlv,
        ASYNCHRONOUS_CORRELATION_VALUE
    )));
    assert!(RngRetrieveResponse::try_from_response_item(response_item(&message)).is_err());
}

#[test]
fn response_conversion_errors_are_sanitized_and_have_typed_sources() {
    let error = RngRetrieveError::new(
        "RNG Retrieve",
        crate::CryptographicOperationErrorKind::MalformedPayload,
    );
    assert!(!error.to_string().contains("sensitive"));
    assert!(error.source().is_none());
}
