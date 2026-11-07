use crate::{
    EndpointRole, ResponseMessage, ResultReason, ResultStatus, SetEndpointRoleRequest,
    SetEndpointRoleResponse,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

#[allow(clippy::too_many_lines)]
fn wrap_response_item(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> ResponseMessage {
    let mut pv = Structure::new();
    pv.try_push(
        Item::new(
            RawTag::new(0x0042_006A).unwrap().try_checked().unwrap(),
            Value::integer(2),
        )
        .unwrap(),
    )
    .unwrap();
    pv.try_push(
        Item::new(
            RawTag::new(0x0042_006B).unwrap().try_checked().unwrap(),
            Value::integer(1),
        )
        .unwrap(),
    )
    .unwrap();

    let mut header = Structure::new();
    header
        .try_push(
            Item::new(
                RawTag::new(0x0042_0069).unwrap().try_checked().unwrap(),
                Value::structure(pv),
            )
            .unwrap(),
        )
        .unwrap();
    header
        .try_push(
            Item::new(
                RawTag::new(0x0042_0092).unwrap().try_checked().unwrap(),
                Value::date_time(1_700_000_000),
            )
            .unwrap(),
        )
        .unwrap();
    header
        .try_push(
            Item::new(
                RawTag::new(0x0042_000D).unwrap().try_checked().unwrap(),
                Value::integer(1),
            )
            .unwrap(),
        )
        .unwrap();

    let mut item_structure = Structure::new();
    item_structure
        .try_push(
            Item::new(
                RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(),
                Value::enumeration(operation),
            )
            .unwrap(),
        )
        .unwrap();
    item_structure
        .try_push(
            Item::new(
                RawTag::new(0x0042_007F).unwrap().try_checked().unwrap(),
                Value::enumeration(status),
            )
            .unwrap(),
        )
        .unwrap();
    if let Some(r) = reason {
        item_structure
            .try_push(
                Item::new(
                    RawTag::new(0x0042_007E).unwrap().try_checked().unwrap(),
                    Value::enumeration(r),
                )
                .unwrap(),
            )
            .unwrap();
    }
    if let Some(p) = payload {
        item_structure
            .try_push(
                Item::new(
                    RawTag::new(0x0042_007C).unwrap().try_checked().unwrap(),
                    Value::structure(p),
                )
                .unwrap(),
            )
            .unwrap();
    }

    let mut msg = Structure::new();
    msg.try_push(
        Item::new(
            RawTag::new(0x0042_007A).unwrap().try_checked().unwrap(),
            Value::structure(header),
        )
        .unwrap(),
    )
    .unwrap();
    msg.try_push(
        Item::new(
            RawTag::new(0x0042_000F).unwrap().try_checked().unwrap(),
            Value::structure(item_structure),
        )
        .unwrap(),
    )
    .unwrap();

    ResponseMessage::try_from_ttlv(msg).expect("valid response message fixture")
}

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
    assert_eq!(
        EndpointRole::from_wire_name("Client"),
        Some(EndpointRole::CLIENT)
    );
    assert_eq!(
        EndpointRole::from_wire_name("Server"),
        Some(EndpointRole::SERVER)
    );
    assert_eq!(EndpointRole::from_wire_name("Unknown"), None);
}

#[test]
fn test_set_endpoint_role_request_encoding() {
    let request =
        SetEndpointRoleRequest::new(EndpointRole::SERVER).with_vendor_identification("AcmeCorp");

    assert_eq!(request.endpoint_role(), EndpointRole::SERVER);
    assert_eq!(request.vendor_identification(), Some("AcmeCorp"));

    let payload = request.to_ttlv_payload().expect("payload build succeeds");
    let binding = payload.view();
    let items = binding.children();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].tag().raw(), 0x0042_0151); // Endpoint Role
    assert_eq!(
        items[0].with_value(|v| match v {
            kmipkit_ttlv::ValueView::Enumeration(e) => Some(*e),
            _ => None,
        }),
        Some(2)
    );
    assert_eq!(items[1].tag().raw(), 0x0042_009D); // Vendor Identification
    assert_eq!(
        items[1].with_value(|v| match v {
            kmipkit_ttlv::ValueView::TextString(s) => Some(s.to_string()),
            _ => None,
        }),
        Some("AcmeCorp".to_string())
    );
}

#[test]
fn test_set_endpoint_role_response_decoding_success() {
    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0151).unwrap().try_checked().unwrap(),
                Value::enumeration(2),
            )
            .unwrap(),
        )
        .unwrap();

    let msg = wrap_response_item(0x0000_0032, 0, None, Some(payload));
    let view = msg.batch_items().next().expect("valid view");
    let response = SetEndpointRoleResponse::try_from_response_item(view).expect("valid response");

    assert_eq!(response.result().status(), ResultStatus::from_raw(0));
    assert_eq!(response.endpoint_role(), Some(EndpointRole::SERVER));
    assert_eq!(response.into_result().unwrap(), EndpointRole::SERVER);
}

#[test]
fn test_set_endpoint_role_response_decoding_failure() {
    let msg = wrap_response_item(0x0000_0032, 1, Some(2), None);
    let view = msg.batch_items().next().expect("valid view");
    let response = SetEndpointRoleResponse::try_from_response_item(view).expect("valid response");

    assert_eq!(response.result().status(), ResultStatus::from_raw(1));
    assert_eq!(response.result().reason(), Some(ResultReason::from_raw(2)));
    assert!(response.into_result().is_err());
}

#[test]
fn test_endpoint_role_helpers_and_formatting() {
    let client = EndpointRole::CLIENT;
    assert!(client.is_known());
    assert!(!client.is_extension());
    assert_eq!(client.wire_name(), "Client");
    assert_eq!(format!("{client:?}"), "EndpointRole(1: Client)");
    assert_eq!(format!("{client}"), "Client");
    assert_eq!(EndpointRole::from_wire_name("Client"), Some(client));

    let server = EndpointRole::SERVER;
    assert!(server.is_known());
    assert!(!server.is_extension());
    assert_eq!(server.wire_name(), "Server");
    assert_eq!(EndpointRole::from_wire_name("Server"), Some(server));

    let ext = EndpointRole::from_raw(0x8000_0001);
    assert!(!ext.is_known());
    assert!(ext.is_extension());
    assert_eq!(ext.wire_name(), "Unknown");
    assert_eq!(EndpointRole::from_wire_name("Invalid"), None);
}

#[test]
fn test_set_endpoint_role_response_builders_and_payloads() {
    let resp = SetEndpointRoleResponse::success(EndpointRole::CLIENT, Some("VendorX".to_string()));
    assert_eq!(resp.endpoint_role(), Some(EndpointRole::CLIENT));
    assert_eq!(resp.vendor_identification(), Some("VendorX"));

    let ttlv = resp.to_ttlv_payload().expect("to_ttlv_payload succeeds");
    let view = ttlv.view();
    assert_eq!(view.children().len(), 2);

    let resp_no_vendor = SetEndpointRoleResponse::success(EndpointRole::SERVER, None);
    assert_eq!(resp_no_vendor.vendor_identification(), None);
    let ttlv_no_vendor = resp_no_vendor
        .to_ttlv_payload()
        .expect("to_ttlv_payload succeeds");
    assert_eq!(ttlv_no_vendor.view().children().len(), 1);
}

#[test]
fn test_set_endpoint_role_request_try_from_request_item() {
    use crate::RequestMessage;

    // Build a valid request message
    let mut header = Structure::new();
    let mut pv = Structure::new();
    pv.try_push(
        Item::new(
            RawTag::new(0x0042_006A).unwrap().try_checked().unwrap(),
            Value::integer(2),
        )
        .unwrap(),
    )
    .unwrap();
    pv.try_push(
        Item::new(
            RawTag::new(0x0042_006B).unwrap().try_checked().unwrap(),
            Value::integer(1),
        )
        .unwrap(),
    )
    .unwrap();
    header
        .try_push(
            Item::new(
                RawTag::new(0x0042_0069).unwrap().try_checked().unwrap(),
                Value::structure(pv),
            )
            .unwrap(),
        )
        .unwrap();
    header
        .try_push(
            Item::new(
                RawTag::new(0x0042_000D).unwrap().try_checked().unwrap(),
                Value::integer(1),
            )
            .unwrap(),
        )
        .unwrap();

    let mut item_s = Structure::new();
    item_s
        .try_push(
            Item::new(
                RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(),
                Value::enumeration(0x32),
            )
            .unwrap(),
        )
        .unwrap();

    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0151).unwrap().try_checked().unwrap(),
                Value::enumeration(1),
            )
            .unwrap(),
        )
        .unwrap();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_009D).unwrap().try_checked().unwrap(),
                Value::text_string("TestVendor".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
    item_s
        .try_push(
            Item::new(
                RawTag::new(0x0042_0079).unwrap().try_checked().unwrap(),
                Value::structure(payload),
            )
            .unwrap(),
        )
        .unwrap();

    let mut root = Structure::new();
    root.try_push(
        Item::new(
            RawTag::new(0x0042_0077).unwrap().try_checked().unwrap(),
            Value::structure(header),
        )
        .unwrap(),
    )
    .unwrap();
    root.try_push(
        Item::new(
            RawTag::new(0x0042_000F).unwrap().try_checked().unwrap(),
            Value::structure(item_s),
        )
        .unwrap(),
    )
    .unwrap();

    let req_msg = RequestMessage::try_from_ttlv(root).expect("valid request message");
    let item_view = req_msg.batch_items().next().expect("batch item exists");
    let req = SetEndpointRoleRequest::try_from_request_item(item_view).expect("parsed request");
    assert_eq!(req.endpoint_role(), EndpointRole::CLIENT);
    assert_eq!(req.vendor_identification(), Some("TestVendor"));
}

#[test]
fn test_set_endpoint_role_error_displays_and_coverage() {
    use crate::set_endpoint_role::SetEndpointRoleError;

    let err1 = SetEndpointRoleError::UnexpectedOperation;
    assert!(format!("{err1}").contains("Set Endpoint Role"));

    let err2 = SetEndpointRoleError::MissingResultStatus;
    assert!(format!("{err2}").contains("missing a result status"));

    let err3 = SetEndpointRoleError::MissingPayload;
    assert!(format!("{err3}").contains("omitted payload"));

    let err4 = SetEndpointRoleError::InvalidPayloadStructure;
    assert!(format!("{err4}").contains("malformed Set Endpoint Role payload"));

    let err5 = SetEndpointRoleError::MissingRequiredField;
    assert!(format!("{err5}").contains("missing required Endpoint Role"));

    let op_res = crate::KmipOperationResult::new(
        ResultStatus::from_raw(1),
        Some(ResultReason::from_raw(1)),
        None,
    )
    .unwrap();
    let err6 = SetEndpointRoleError::ServerRejected(op_res);
    assert!(format!("{err6}").contains("server rejected Set Endpoint Role"));
}
