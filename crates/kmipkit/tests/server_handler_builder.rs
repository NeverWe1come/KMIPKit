use kmipkit::{ServerHandlerBuilder, ServerRequestHandler};

#[test]
fn server_handler_builder_dispatches_configured_and_unconfigured_ops() {
    let mut handler = ServerHandlerBuilder::new()
        .on_notify(|_req| Ok(kmipkit_protocol::ServerNotifyResponse::new()))
        .build();
    let notify_req =
        kmipkit_protocol::ServerNotifyRequest::new("id-123".to_owned(), None, Vec::new());
    assert!(handler.handle_notify(&notify_req).is_ok());

    let query_req = kmipkit_protocol::ServerQueryRequest::new(vec![]);
    let err = handler
        .handle_query(&query_req)
        .expect_err("unconfigured handler returns OperationNotSupported");
    assert_eq!(err.status(), kmipkit_protocol::ResultStatus::from_raw(1));
}
