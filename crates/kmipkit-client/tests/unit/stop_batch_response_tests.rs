//! Stop batch response cardinality tests derived from OASIS KMIP v2.1 §§8.3,
//! 8.6, 9.6, 9.21, and 11.5/Table 434, and the KMIPKIT-0092
//! `test-batch-stop-on-first-failure` project scenario. OASIS does not
//! explicitly require omission of unprocessed response items; the partial
//! response rule is `KMIPKit` policy under KMIPKIT-0007-FR-018. These are
//! deterministic fake-transport tests, not official OASIS vectors.
//!
//! Traceability: KMIPKIT-0007-FR-006, FR-018, SC-010.

use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::codec::CodecLimits;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatch, ClientBatchItem, ClientRequest};
use crate::execute_test_support::{ResponseItemFixture, response_bytes};

const FIRST_ID: &[u8] = b"stop-first-item";
const SECOND_ID: &[u8] = b"stop-second-item";
const THIRD_ID: &[u8] = b"stop-third-item";
const BATCH_ERROR_CONTINUATION_CONTINUE: u32 = 1;
const BATCH_ERROR_CONTINUATION_STOP: u32 = 2;
const BATCH_ERROR_CONTINUATION_UNDO: u32 = 3;
const OPERATION_FAILED: u32 = 1;
const PING_OPERATION: u32 = 0x0000_003B;

fn request_batch(ids: &[&[u8]], continuation: Option<u32>) -> ClientBatch {
    let items = ids.iter().map(|id| {
        ClientBatchItem::new(ClientRequest::discover_versions())
            .with_unique_batch_item_id(id.to_vec())
    });
    let batch = ClientBatch::from_items(items);
    match continuation {
        Some(value) => batch.with_batch_error_continuation_option(value),
        None => batch,
    }
}

fn execute_response(
    ids: &[&[u8]],
    continuation: Option<u32>,
    fixtures: &[ResponseItemFixture],
) -> Result<crate::execute::ClientBatchResponse, crate::ClientError> {
    let response = response_bytes((2, 1), fixtures);
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    client.execute(request_batch(ids, continuation), &CodecLimits::defaults())
}

#[test]
fn stop_batch_accepts_failure_response_with_omitted_later_item() {
    let response = execute_response(
        &[FIRST_ID, SECOND_ID],
        None,
        &[ResponseItemFixture::failure(Some(FIRST_ID))],
    )
    .expect("effective Stop accepts the returned failing item without the later result");

    assert_eq!(response.len(), 1);
    assert!(!response.is_empty());
    let first = response
        .get(0)
        .expect("the returned result is indexed first");
    assert_eq!(first.unique_batch_item_id(), Some(FIRST_ID));
    assert_eq!(first.outcome().result().status().raw(), OPERATION_FAILED);
    assert!(
        response.get(1).is_none(),
        "no result is synthesized for the omitted item"
    );
    assert_eq!(response.iter().count(), 1);
}

#[test]
fn stop_batch_orders_returned_subset_by_request_identity() {
    let response = execute_response(
        &[FIRST_ID, SECOND_ID, THIRD_ID],
        Some(BATCH_ERROR_CONTINUATION_STOP),
        &[
            ResponseItemFixture::failure(Some(THIRD_ID)),
            ResponseItemFixture::success(Some(FIRST_ID)),
        ],
    )
    .expect("a Stop failure permits the returned subset");

    assert_eq!(response.len(), 2);
    assert_eq!(
        response.get(0).and_then(|item| item.unique_batch_item_id()),
        Some(FIRST_ID)
    );
    assert_eq!(
        response.get(1).and_then(|item| item.unique_batch_item_id()),
        Some(THIRD_ID)
    );
}

#[test]
fn stop_batch_rejects_empty_partial_response() {
    assert!(execute_response(&[FIRST_ID, SECOND_ID], None, &[]).is_err());
}

#[test]
fn stop_batch_rejects_partial_response_without_failure() {
    assert!(
        execute_response(
            &[FIRST_ID, SECOND_ID],
            None,
            &[ResponseItemFixture::success(Some(FIRST_ID))],
        )
        .is_err()
    );
}

#[test]
fn stop_batch_rejects_partial_continue_or_undo_response() {
    let failure = [ResponseItemFixture::failure(Some(FIRST_ID))];
    for continuation in [
        BATCH_ERROR_CONTINUATION_CONTINUE,
        BATCH_ERROR_CONTINUATION_UNDO,
    ] {
        assert!(execute_response(&[FIRST_ID, SECOND_ID], Some(continuation), &failure,).is_err());
    }
}

#[test]
fn stop_batch_rejects_excess_response_items() {
    assert!(
        execute_response(
            &[FIRST_ID, SECOND_ID],
            Some(BATCH_ERROR_CONTINUATION_STOP),
            &[
                ResponseItemFixture::failure(Some(FIRST_ID)),
                ResponseItemFixture::success(Some(SECOND_ID)),
                ResponseItemFixture::success(Some(THIRD_ID)),
            ],
        )
        .is_err()
    );
}

#[test]
fn stop_batch_rejects_duplicate_unknown_or_mismatched_items() {
    let duplicate = [
        ResponseItemFixture::failure(Some(FIRST_ID)),
        ResponseItemFixture::success(Some(FIRST_ID)),
    ];
    assert!(
        execute_response(
            &[FIRST_ID, SECOND_ID],
            Some(BATCH_ERROR_CONTINUATION_STOP),
            &duplicate,
        )
        .is_err()
    );

    assert!(
        execute_response(
            &[FIRST_ID, SECOND_ID],
            Some(BATCH_ERROR_CONTINUATION_STOP),
            &[ResponseItemFixture::failure(Some(THIRD_ID))],
        )
        .is_err()
    );

    let mismatched = [ResponseItemFixture {
        operation: PING_OPERATION,
        ..ResponseItemFixture::failure(Some(FIRST_ID))
    }];
    assert!(
        execute_response(
            &[FIRST_ID, SECOND_ID],
            Some(BATCH_ERROR_CONTINUATION_STOP),
            &mismatched,
        )
        .is_err()
    );
}
