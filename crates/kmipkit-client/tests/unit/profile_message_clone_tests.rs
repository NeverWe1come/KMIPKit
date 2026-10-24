use super::*;

#[test]
fn profiled_request_copy_rejects_the_configured_size_limit() {
    let limits =
        CodecLimits::new(0, 64, 100_000).expect("zero message bytes is a valid per-call limit");
    let request = request_message_for_test(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
        &CodecLimits::defaults(),
    )
    .expect("Discover Versions creates a valid request message");

    let error = clone_request_message(&request, &limits)
        .expect_err("profile validation context must respect the configured encoded-size limit");

    assert_eq!(error.kind(), ProtocolErrorKind::InvalidValue);
    assert_eq!(
        error.cause_category(),
        ProtocolCauseCategory::InvalidEncoding
    );
}
