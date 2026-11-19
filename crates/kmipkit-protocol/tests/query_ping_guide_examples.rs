//! Executable examples for constructing the KMIP 2.1 Query and Ping payloads.

use kmipkit_protocol::{PingRequest, QueryFunction, QueryRequest};
use kmipkit_ttlv::ValueView;

#[test]
fn query_example_keeps_requested_functions_and_object_groups() {
    let request = QueryRequest::new([
        QueryFunction::OPERATIONS,
        QueryFunction::OBJECTS,
        QueryFunction::from_raw(0x8000_0042),
    ])
    .with_object_groups(["symmetric-keys", "certificates"]);
    let payload = request
        .to_ttlv_payload()
        .expect("the selected Query values are valid");
    let payload_view = payload.view();
    let fields = payload_view.children();

    assert_eq!(fields.len(), 4);
    assert_eq!(fields[0].tag().raw(), 0x0042_0074);
    assert_eq!(fields[1].tag().raw(), 0x0042_0074);
    assert_eq!(fields[2].tag().raw(), 0x0042_0074);
    assert_eq!(fields[3].tag().raw(), 0x0042_0166);
    fields[3].with_value(|value| match value {
        ValueView::Structure(groups) => {
            assert_eq!(groups.children().len(), 2);
            assert_eq!(groups.children()[0].tag().raw(), 0x0042_0056);
        }
        _ => panic!("Object Groups is a Structure"),
    });
}

#[test]
fn ping_example_encodes_an_empty_operation_payload() {
    let payload = PingRequest::new()
        .to_ttlv_payload()
        .expect("the empty Ping payload is representable");
    assert!(payload.view().children().is_empty());
}
