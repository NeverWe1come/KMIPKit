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
