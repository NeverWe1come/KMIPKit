//! Derived KMIP 2.1 Cancel tests; not official conformance vectors.
//!
//! Traceability: `KMIPKIT-0009-FR-001`, `FR-004`, `FR-009`; OASIS KMIP v2.1
//! §§6.1.5 and 11.7, Tables 176–178 and 437–438.

use kmipkit_ttlv::ValueView;

use crate::async_operation_fixtures::{ASYNCHRONOUS_CORRELATION_VALUE, item};
use crate::{CancelRequest, CancellationResult};

#[test]
fn cancel_request_preserves_exact_correlation_bytes_in_table_order() {
    let correlation = [0x00, 0xff, 0x80, b'X'];
    let request = CancelRequest::new(&correlation);
    let payload = request
        .to_ttlv_payload()
        .expect("the Cancel payload uses allocated KMIP 2.1 fields");
    let view = payload.view();
    let fields = view.children();

    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), ASYNCHRONOUS_CORRELATION_VALUE);
    assert_eq!(
        fields[0].with_value(|value| match value {
            ValueView::ByteString(bytes) => Some(bytes.to_vec()),
            _ => None,
        }),
        Some(correlation.to_vec())
    );
    assert!(!format!("{request:?}").contains("255"));
}

#[test]
fn cancellation_result_maps_every_assigned_v21_value_and_preserves_unknowns() {
    let expected = [
        CancellationResult::Canceled,
        CancellationResult::UnableToCancel,
        CancellationResult::Completed,
        CancellationResult::Failed,
        CancellationResult::Unavailable,
    ];
    for (offset, value) in expected.into_iter().enumerate() {
        let raw = u32::try_from(offset + 1).expect("small enum value fits u32");
        assert_eq!(CancellationResult::from_raw(raw), value);
        assert_eq!(CancellationResult::from_raw(raw).raw(), raw);
    }
    assert_eq!(
        CancellationResult::from_raw(0x8123_4567),
        CancellationResult::Unknown(0x8123_4567)
    );
    assert_eq!(CancellationResult::from_raw(0x8123_4567).raw(), 0x8123_4567);
}

#[test]
fn cancel_success_response_exposes_echo_and_raw_cancellation_result() {
    use crate::CancelResponse;
    use crate::async_operation_fixtures::{
        CANCEL, CANCELLATION_RESULT, response_message, structure,
    };
    use kmipkit_ttlv::Value;

    let correlation = [0x00, 0xff, 0x80];
    let payload = structure([
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(correlation.to_vec()),
        ),
        item(CANCELLATION_RESULT, Value::enumeration(3)),
    ]);
    let message = response_message(CANCEL, 0, None, None, Some(payload));
    let item = message
        .batch_items()
        .next()
        .expect("one response item exists");
    let cancel = CancelResponse::try_from_response_item(item)
        .expect("successful Cancel includes the Table 177 fields");

    assert_eq!(cancel.result().status().raw(), 0);
    assert_eq!(
        cancel.with_asynchronous_correlation_value(<[u8]>::to_vec),
        Some(correlation.to_vec())
    );
    assert_eq!(
        cancel.cancellation_result(),
        Some(CancellationResult::Completed)
    );
    assert!(!format!("{cancel:?}").contains("255"));
}

#[test]
fn cancel_rejects_pending_and_malformed_success_payloads() {
    use crate::CancelResponse;
    use crate::async_operation_fixtures::{
        CANCEL, CANCELLATION_RESULT, response_message, structure,
    };
    use kmipkit_ttlv::Value;

    let pending_payload = structure([
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(b"corr".to_vec()),
        ),
        item(CANCELLATION_RESULT, Value::enumeration(1)),
    ]);
    let pending = response_message(CANCEL, 2, None, Some(b"corr"), Some(pending_payload));
    let pending_item = pending
        .batch_items()
        .next()
        .expect("one response item exists");
    assert!(CancelResponse::try_from_response_item(pending_item).is_err());

    let malformed_payload = structure([
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::enumeration(0xdead_beef),
        ),
        item(CANCELLATION_RESULT, Value::enumeration(1)),
    ]);
    let malformed = response_message(CANCEL, 0, None, None, Some(malformed_payload));
    let malformed_item = malformed
        .batch_items()
        .next()
        .expect("one response item exists");
    assert!(CancelResponse::try_from_response_item(malformed_item).is_err());
}
