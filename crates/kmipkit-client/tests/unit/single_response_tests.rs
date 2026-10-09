use super::{ClientBatchResponse, take_single_response};
use kmipkit_transport::RequestDeliveryState;

#[test]
fn single_operation_response_rejects_an_empty_batch_without_losing_delivery_state() {
    let error = take_single_response(ClientBatchResponse { items: Vec::new() })
        .expect_err("a single-operation convenience call must return one result");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_ne!(error.to_string(), "");
}
