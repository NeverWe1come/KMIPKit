//! Derived KMIP 2.1 Process tests; not official conformance vectors.
//!
//! Traceability: `KMIPKIT-0009-FR-001`, `FR-005`, `FR-009`; OASIS KMIP v2.1
//! §6.1.39, Tables 278–280. The missing catalog requirement ID for Table 278
//! remains an explicit traceability gate (OD-002).

use kmipkit_ttlv::ValueView;

use crate::async_operation_fixtures::ASYNCHRONOUS_CORRELATION_VALUE;
use crate::{AsynchronousOperationError, ProcessRequest};

#[test]
fn process_request_contains_only_the_required_exact_correlation_field() {
    let correlation = [0x00, 0xff, 0x80, 0x01];
    let request = ProcessRequest::new(&correlation);
    let payload = request
        .to_ttlv_payload()
        .expect("the Process payload uses allocated KMIP 2.1 fields");
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
fn process_success_accepts_the_empty_table_279_payload() {
    use crate::ProcessResponse;
    use crate::async_operation_fixtures::{PROCESS, response_message, structure};

    let message = response_message(PROCESS, 0, None, None, Some(structure([])));
    let item = message
        .batch_items()
        .next()
        .expect("one response item exists");
    let process = ProcessResponse::try_from_response_item(item)
        .expect("Table 279 defines an empty Process response payload");

    assert_eq!(process.result().status().raw(), 0);
    assert_eq!(process.payload_member_count(), Some(0));
    assert!(!process.is_pending());
}

#[test]
fn process_success_rejects_unexpected_payload_members() {
    use crate::ProcessResponse;
    use crate::async_operation_fixtures::{PROCESS, response_message, structure};
    use kmipkit_ttlv::Value;

    let message = response_message(
        PROCESS,
        0,
        None,
        None,
        Some(structure([crate::async_operation_fixtures::item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(b"unexpected".to_vec()),
        )])),
    );
    let item = message
        .batch_items()
        .next()
        .expect("one response item exists");

    assert!(ProcessResponse::try_from_response_item(item).is_err());
}

#[test]
fn process_pending_exposes_a_borrowed_new_correlation_value() {
    use crate::ProcessResponse;
    use crate::async_operation_fixtures::{PROCESS, response_message};

    let correlation = [0x00, 0xff, 0x80];
    // §8.6/Table 399 requires a Response Payload for every non-Failure status;
    // Table 279 defines Process's payload as an empty Structure.
    let message = response_message(
        PROCESS,
        2,
        None,
        Some(&correlation),
        Some(crate::async_operation_fixtures::structure([])),
    );
    let item = message
        .batch_items()
        .next()
        .expect("one response item exists");
    let process = ProcessResponse::try_from_response_item(item)
        .expect("Process follows the general asynchronous response model");

    assert!(process.is_pending());
    assert_eq!(
        process.with_asynchronous_correlation_value(<[u8]>::to_vec),
        Some(correlation.to_vec())
    );
    assert_eq!(process.payload_member_count(), Some(0));
    assert_eq!(
        process.with_response_payload(|payload| payload.children().len()),
        Some(0)
    );
    assert!(!format!("{process:?}").contains("00, 255, 128"));
}

#[test]
fn process_failure_has_no_payload_or_correlation_value() {
    use crate::ProcessResponse;
    use crate::async_operation_fixtures::{PROCESS, response_message};

    let message = response_message(PROCESS, 1, Some(1), None, None);
    let item = message
        .batch_items()
        .next()
        .expect("one response item exists");
    let process = ProcessResponse::try_from_response_item(item)
        .expect("Process failure uses the general Failure response shape");

    assert!(!process.is_pending());
    assert_eq!(process.payload_member_count(), None);
    assert_eq!(
        process.with_response_payload(|payload| payload.children().len()),
        None
    );
    assert_eq!(
        process.with_asynchronous_correlation_value(<[u8]>::len),
        None
    );
    assert!(format!("{process:?}").contains("[REDACTED]"));
}

#[test]
fn process_rejects_a_response_for_a_different_operation() {
    use crate::ProcessResponse;
    use crate::async_operation_fixtures::{POLL, response_message};

    let message = response_message(POLL, 1, Some(1), None, None);
    let item = message
        .batch_items()
        .next()
        .expect("one response item exists");

    assert!(matches!(
        ProcessResponse::try_from_response_item(item),
        Err(AsynchronousOperationError::UnexpectedOperation)
    ));
}
