use super::{ClientBatchItemResponse, take_single_item_response};
use crate::ClientErrorCategory;
use kmipkit_transport::RequestDeliveryState;

#[test]
fn empty_single_item_response_preserves_response_started_delivery() {
    let error = take_single_item_response(Vec::<ClientBatchItemResponse>::new())
        .expect_err("a single-item operation cannot return an empty response batch");

    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert!(!error.to_string().contains("response items"));
    assert!(!format!("{error:?}").contains("response items"));
}
