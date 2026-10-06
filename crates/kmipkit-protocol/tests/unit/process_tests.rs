//! Derived KMIP 2.1 Process tests; not official conformance vectors.
//!
//! Traceability: `KMIPKIT-0009-FR-001`, `FR-005`, `FR-009`; OASIS KMIP v2.1
//! §6.1.39, Tables 278–280. The missing catalog requirement ID for Table 278
//! remains an explicit traceability gate (OD-002).

use kmipkit_ttlv::ValueView;

use crate::ProcessRequest;
use crate::async_operation_fixtures::ASYNCHRONOUS_CORRELATION_VALUE;

#[test]
fn process_request_contains_only_the_required_exact_correlation_field() {
    let correlation = [0x00, 0xff, 0x80, 0x01];
    let request = ProcessRequest::new(&correlation);
    let payload = request
        .to_ttlv_payload()
        .expect("the Process payload uses allocated KMIP 2.1 fields");
    let fields = payload.view().children();

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
fn process_pending_exposes_a_borrowed_new_correlation_value() {
    use crate::ProcessResponse;
    use crate::async_operation_fixtures::{PROCESS, response_message};

    let correlation = [0x00, 0xff, 0x80];
    let message = response_message(PROCESS, 2, None, Some(&correlation), None);
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
}
