use super::*;
use kmipkit_protocol::QueryFunction;

#[test]
fn query_constructor_checks_limits_before_returning_the_batch_item() {
    let limits = CodecLimits::new(1024, 64, 2).expect("small element limits are valid");

    let error = ClientBatchItem::query_with_limits(
        QueryRequest::new([QueryFunction::OPERATIONS]).with_object_groups(["group-a"]),
        &limits,
    )
    .expect_err("Query root, function, and Object Group exceed the element limit");

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
}

#[test]
fn query_constructor_checks_required_structure_depth() {
    let limits = CodecLimits::new(1024, 1, 100).expect("one structure level is valid");

    let error = ClientBatchItem::query_with_limits(
        QueryRequest::new([QueryFunction::OPERATIONS]).with_object_groups(["group-a"]),
        &limits,
    )
    .expect_err("Object Groups adds a nested TTLV Structure");

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
}

#[test]
fn query_payload_limit_preflight_rejects_excess_elements() {
    let batch = ClientBatch::new(ClientBatchItem::query(
        QueryRequest::new([QueryFunction::OPERATIONS]).with_object_groups(["group-a", "group-b"]),
    ));
    let limits = CodecLimits::new(1024, 64, 3).expect("small element limits are valid");

    let error = validate_query_batch_limits(&batch, &limits)
        .expect_err("Query wrappers and children exceed the configured element limit");

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
}

#[test]
fn query_payload_limit_preflight_rejects_object_groups_over_depth_limit() {
    let batch = ClientBatch::new(ClientBatchItem::query(
        QueryRequest::new([QueryFunction::OPERATIONS]).with_object_groups(["group-a"]),
    ));
    let limits = CodecLimits::new(1024, 2, 100).expect("two structure levels are valid");

    let error = validate_query_batch_limits(&batch, &limits)
        .expect_err("Object Groups requires one structure level beyond this limit");

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

#[test]
fn query_text_string_limit_matches_the_ttlv_item_length_field() {
    assert_eq!(
        ensure_query_text_length_is_ttlv_representable(u32::MAX as usize),
        Ok(())
    );
    if usize::BITS > u32::BITS {
        let error = ensure_query_text_length_is_ttlv_representable(
            usize::try_from(u64::from(u32::MAX) + 1)
                .expect("the test runs this boundary on a 64-bit target"),
        )
        .expect_err("TTLV Text String Item Length is limited to an unsigned 32-bit value");
        assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
    }
}

#[test]
fn query_parent_structure_lengths_match_the_ttlv_item_length_field() {
    assert_eq!(
        ensure_query_ttlv_structure_length_is_representable(u32::MAX as usize),
        Ok(())
    );
    if usize::BITS > u32::BITS {
        let error = ensure_query_ttlv_structure_length_is_representable(
            usize::try_from(u64::from(u32::MAX) + 1)
                .expect("the test runs this boundary on a 64-bit target"),
        )
        .expect_err("TTLV Structure Item Length is limited to an unsigned 32-bit value");
        assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
    }
}

#[test]
fn query_with_limits_rejects_an_empty_function_list() {
    let limits = CodecLimits::defaults();
    let error = ClientBatchItem::query_with_limits(QueryRequest::new([]), &limits)
        .expect_err("KMIP Query requires at least one Query Function");

    assert_eq!(error.kind(), ProtocolErrorKind::InvalidInput);
}
