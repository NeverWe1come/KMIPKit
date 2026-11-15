use kmipkit_client::ClientBatchItem;
use kmipkit_protocol::{QueryFunction, QueryRequest};
use kmipkit_ttlv::codec::CodecLimits;

#[test]
fn query_with_empty_object_groups_fits_its_exact_ttlv_payload_size() {
    // Request Payload (8) + one Query Function item (16) + empty Object Groups item (8).
    let limits = CodecLimits::new(32, 4, 3).expect("custom limits are valid");

    let result = ClientBatchItem::query_with_limits(
        QueryRequest::new([QueryFunction::OPERATIONS])
            .with_object_groups(std::iter::empty::<&str>()),
        &limits,
    );

    assert!(result.is_ok(), "the 32-byte TTLV payload fits exactly");
}
