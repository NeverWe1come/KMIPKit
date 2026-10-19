use super::*;
use kmipkit_protocol::QueryFunction;

#[test]
fn query_payload_limit_preflight_rejects_excess_elements() {
    let batch = ClientBatch::new(ClientBatchItem::query(
        QueryRequest::new([QueryFunction::OPERATIONS])
            .with_object_groups(["group-a", "group-b"]),
    ));
    let limits = CodecLimits::new(1024, 64, 3).expect("small element limits are valid");

    let error = validate_query_batch_limits(&batch, &limits)
        .expect_err("Query wrappers and children exceed the configured element limit");

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
}

#[test]
fn query_payload_limit_preflight_rejects_excess_encoded_size() {
    let large_group = "g".repeat(512);
    let batch = ClientBatch::new(ClientBatchItem::query(
        QueryRequest::new([QueryFunction::OPERATIONS]).with_object_groups([large_group.as_str()]),
    ));
    let limits = CodecLimits::new(128, 64, 100).expect("small byte limits are valid");

    let error = validate_query_batch_limits(&batch, &limits)
        .expect_err("Query payload exceeds the configured message limit");

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
}

#[test]
fn query_payload_limit_preflight_aggregates_the_complete_batch() {
    let item = || ClientBatchItem::query(QueryRequest::new([QueryFunction::OPERATIONS]));
    let batch = ClientBatch::from_items([item(), item()]);
    let limits = CodecLimits::new(40, 64, 100).expect("small byte limits are valid");

    let error = validate_query_batch_limits(&batch, &limits)
        .expect_err("combined Query payloads exceed the configured message limit");

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
}
