//! Derived KMIP 2.1 Poll tests; not official conformance vectors.
//!
//! Traceability: `KMIPKIT-0009-FR-001`–`FR-003`, `FR-009`; OASIS KMIP v2.1
//! §§6.1.38, 8.6, 9.1, and 9.19, Tables 276, 399, 400, and 424.

use kmipkit_ttlv::{Value, ValueView};

use crate::async_operation_fixtures::{
    ASYNCHRONOUS_CORRELATION_VALUE, POLL, item, response_message, structure,
};
use crate::{PollRequest, PollResponse};

#[test]
fn poll_request_preserves_arbitrary_correlation_bytes_in_table_order() {
    let correlation = [0x00, 0xff, b'P', 0x80, 0x01];
    let request = PollRequest::new(&correlation);
    let payload = request
        .to_ttlv_payload()
        .expect("the Poll payload uses allocated KMIP 2.1 fields");

    let fields = payload.view().children();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), ASYNCHRONOUS_CORRELATION_VALUE);
    let actual = fields[0].with_value(|value| match value {
        ValueView::ByteString(bytes) => Some(bytes.to_vec()),
        _ => None,
    });
    assert_eq!(actual, Some(correlation.to_vec()));
    assert_eq!(request.asynchronous_correlation_value(), correlation);
    assert!(!format!("{request:?}").contains("255"));
}

#[test]
fn poll_pending_response_has_no_payload_and_lends_exact_correlation() {
    let correlation = [0x00, 0xff, 0x80, 0x01];
    let response = response_message(POLL, 2, None, Some(&correlation), None);
    let item = response
        .batch_items()
        .next()
        .expect("one response item exists");
    let poll = PollResponse::try_from_response_item(item)
        .expect("§6.1.38 permits Pending without a Poll response payload");

    assert!(poll.is_pending());
    assert_eq!(poll.result().status().raw(), 2);
    assert_eq!(
        poll.with_asynchronous_correlation_value(<[u8]>::to_vec),
        Some(correlation.to_vec())
    );
    assert_eq!(
        poll.with_response_payload(|payload| payload.children().len()),
        None
    );
    assert!(!format!("{poll:?}").contains("255"));
}

#[test]
fn poll_success_keeps_the_original_operation_payload_generic() {
    let payload = structure([
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::enumeration(0xdead_beef),
        ),
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(vec![0x00, 0xff]),
        ),
    ]);
    let response = response_message(POLL, 0, None, None, Some(payload));
    let item = response
        .batch_items()
        .next()
        .expect("one response item exists");
    let poll = PollResponse::try_from_response_item(item)
        .expect("successful Poll retains the original operation payload");

    assert!(!poll.is_pending());
    assert_eq!(poll.result().status().raw(), 0);
    assert_eq!(
        poll.with_response_payload(|payload| payload.children().len()),
        Some(2)
    );
}

#[test]
fn poll_failure_exposes_reason_without_an_operation_payload() {
    let response = response_message(POLL, 1, Some(1), None, None);
    let item = response
        .batch_items()
        .next()
        .expect("one response item exists");
    let poll = PollResponse::try_from_response_item(item)
        .expect("terminal Poll Failure uses the general response shape");

    assert!(!poll.is_pending());
    assert_eq!(poll.result().status().raw(), 1);
    assert_eq!(poll.result().reason().map(|reason| reason.raw()), Some(1));
    assert_eq!(poll.with_response_payload(|_| ()), None);
}
