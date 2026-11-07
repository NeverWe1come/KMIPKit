use crate::{
    EndpointRole, ResponseBatchItemView,
    ResultReason, ResultStatus, SetEndpointRoleRequest, SetEndpointRoleResponse,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

#[test]
fn test_endpoint_role_constants_and_raw() {
    assert_eq!(EndpointRole::CLIENT.raw(), 1);
    assert_eq!(EndpointRole::SERVER.raw(), 2);
    assert!(EndpointRole::CLIENT.is_known());
    assert!(EndpointRole::SERVER.is_known());
    assert!(!EndpointRole::CLIENT.is_extension());

    let ext = EndpointRole::from_raw(0x8000_0001);
    assert!(!ext.is_known());
    assert!(ext.is_extension());
    assert_eq!(ext.raw(), 0x8000_0001);

    assert_eq!(EndpointRole::CLIENT.wire_name(), "Client");
    assert_eq!(EndpointRole::SERVER.wire_name(), "Server");
    assert_eq!(EndpointRole::from_wire_name("Client"), Some(EndpointRole::CLIENT));
    assert_eq!(EndpointRole::from_wire_name("Server"), Some(EndpointRole::SERVER));
    assert_eq!(EndpointRole::from_wire_name("Unknown"), None);
}

#[test]
fn test_set_endpoint_role_request_encoding() {
    let request = SetEndpointRoleRequest::new(EndpointRole::SERVER)
        .with_vendor_identification("AcmeCorp");

    assert_eq!(request.endpoint_role(), EndpointRole::SERVER);
    assert_eq!(request.vendor_identification(), Some("AcmeCorp"));

    let payload = request.to_ttlv_payload().expect("payload build succeeds");
    let items = payload.as_items();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].tag().raw(), 0x0042_0151); // Endpoint Role
    assert_eq!(items[0].value().as_enumeration(), Some(2));
    assert_eq!(items[1].tag().raw(), 0x0042_009D); // Vendor Identification
    assert_eq!(items[1].value().as_text_string(), Some("AcmeCorp"));
}

#[test]
fn test_set_endpoint_role_response_decoding_success() {
    let mut payload = Structure::new();
    payload.try_push(Item::new(
        RawTag::new(0x0042_0151).unwrap().try_checked().unwrap(),
        Value::enumeration(2),
    ).unwrap()).unwrap();

    let mut item_structure = Structure::new();
    item_structure.try_push(Item::new(
        RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(), // Operation
        Value::enumeration(0x0000_0032), // Set Endpoint Role
    ).unwrap()).unwrap();
    item_structure.try_push(Item::new(
        RawTag::new(0x0042_0079).unwrap().try_checked().unwrap(), // Result Status
        Value::enumeration(0), // Success
    ).unwrap()).unwrap();
    item_structure.try_push(Item::new(
        RawTag::new(0x0042_007C).unwrap().try_checked().unwrap(), // Response Payload
        Value::structure(payload),
    ).unwrap()).unwrap();

    let item = Item::new(
        RawTag::new(0x0042_000F).unwrap().try_checked().unwrap(), // Response Batch Item
        Value::structure(item_structure),
    ).unwrap();

    let view = ResponseBatchItemView::try_from_item(&item).expect("valid view");
    let response = SetEndpointRoleResponse::try_from_response_item(view).expect("valid response");

    assert_eq!(response.result().status(), ResultStatus::Success);
    assert_eq!(response.endpoint_role(), Some(EndpointRole::SERVER));
    assert_eq!(response.into_result().unwrap(), EndpointRole::SERVER);
}

#[test]
fn test_set_endpoint_role_response_decoding_failure() {
    let mut item_structure = Structure::new();
    item_structure.try_push(Item::new(
        RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(),
        Value::enumeration(0x0000_0032),
    ).unwrap()).unwrap();
    item_structure.try_push(Item::new(
        RawTag::new(0x0042_0079).unwrap().try_checked().unwrap(),
        Value::enumeration(1), // Operation Failed
    ).unwrap()).unwrap();
    item_structure.try_push(Item::new(
        RawTag::new(0x0042_007A).unwrap().try_checked().unwrap(), // Result Reason
        Value::enumeration(2), // Permission Denied
    ).unwrap()).unwrap();

    let item = Item::new(
        RawTag::new(0x0042_000F).unwrap().try_checked().unwrap(),
        Value::structure(item_structure),
    ).unwrap();

    let view = ResponseBatchItemView::try_from_item(&item).expect("valid view");
    let response = SetEndpointRoleResponse::try_from_response_item(view).expect("valid response");

    assert_eq!(response.result().status(), ResultStatus::OperationFailed);
    assert_eq!(response.result().reason(), Some(ResultReason::PermissionDenied));
    assert!(response.into_result().is_err());
}
