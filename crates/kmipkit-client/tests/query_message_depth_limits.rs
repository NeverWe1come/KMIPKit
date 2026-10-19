use kmipkit_client::ClientBatchItem;
use kmipkit_protocol::{QueryFunction, QueryRequest};
use kmipkit_ttlv::codec::CodecLimits;

#[test]
fn query_constructor_checks_full_message_depth_without_object_groups() {
    let limits = CodecLimits::new(1024, 2, 100).expect("custom limits are valid");

    let result =
        ClientBatchItem::query_with_limits(QueryRequest::new([QueryFunction::OPERATIONS]), &limits);

    assert!(
        result.is_err(),
        "the Request Message needs three structure levels"
    );
}

#[test]
fn query_constructor_checks_full_message_depth_with_object_groups() {
    let limits = CodecLimits::new(1024, 3, 100).expect("custom limits are valid");

    let result = ClientBatchItem::query_with_limits(
        QueryRequest::new([QueryFunction::OPERATIONS]).with_object_groups(["group-a"]),
        &limits,
    );

    assert!(
        result.is_err(),
        "Object Groups adds a fourth structure level"
    );
}
