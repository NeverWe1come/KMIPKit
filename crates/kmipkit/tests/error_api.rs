use kmipkit::{
    ClientError, ClientErrorCategory, KmipOperationResult, RequestDeliveryState, ResultMessage,
    ResultReason, ResultStatus, TransportCauseCategory, TransportError,
};

#[test]
fn facade_exposes_lossless_server_result_without_local_delivery_state() {
    let status = ResultStatus::from_raw(u32::MAX);
    let reason = ResultReason::from_raw(0x8000_0001);
    let message_text = "vendor response \u{1f512}";
    let result = KmipOperationResult::new(
        status,
        Some(reason),
        Some(ResultMessage::new(message_text.to_owned())),
    )
    .expect("unknown status values do not gain result-reason rules");
    let error = ClientError::server_result(result);

    assert_eq!(error.category(), ClientErrorCategory::ServerResult);
    assert_eq!(error.server_status(), Some(status));
    assert_eq!(error.delivery_state(), None);
    let exposed = error
        .server_operation_result()
        .expect("complete result remains available");
    assert_eq!(exposed.reason(), Some(reason));
    assert_eq!(
        exposed.message().map(ResultMessage::as_str),
        Some(message_text)
    );
}

#[test]
fn facade_exposes_transport_delivery_mapping() {
    let transport = TransportError::new(
        RequestDeliveryState::ResponseStarted,
        TransportCauseCategory::Http,
        std::io::Error::other("untrusted body sentinel"),
    );
    let error = ClientError::transport(transport);

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
}
