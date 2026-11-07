use crate::{
    ProtocolVersion, RequestMessage, ServerDiscoverVersionsRequest, ServerDiscoverVersionsResponse,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

fn wrap_request_item(operation: u32, payload: Structure) -> RequestMessage {
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
                RawTag::new(0x0042_0079).unwrap().try_checked().unwrap(),
                Value::structure(payload),
            )
            .unwrap(),
        )
        .unwrap();

    let mut msg = Structure::new();
    msg.try_push(
        Item::new(
            RawTag::new(0x0042_0077).unwrap().try_checked().unwrap(),
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

    RequestMessage::try_from_ttlv(msg).expect("valid request message fixture")
}

#[test]
fn test_server_discover_versions_request_decoding_with_versions() {
    let mut v1 = Structure::new();
    v1.try_push(
        Item::new(
            RawTag::new(0x0042_006A).unwrap().try_checked().unwrap(),
            Value::integer(2),
        )
        .unwrap(),
    )
    .unwrap();
    v1.try_push(
        Item::new(
            RawTag::new(0x0042_006B).unwrap().try_checked().unwrap(),
            Value::integer(1),
        )
        .unwrap(),
    )
    .unwrap();

    let mut v2 = Structure::new();
    v2.try_push(
        Item::new(
            RawTag::new(0x0042_006A).unwrap().try_checked().unwrap(),
            Value::integer(1),
        )
        .unwrap(),
    )
    .unwrap();
    v2.try_push(
        Item::new(
            RawTag::new(0x0042_006B).unwrap().try_checked().unwrap(),
            Value::integer(4),
        )
        .unwrap(),
    )
    .unwrap();

    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0069).unwrap().try_checked().unwrap(),
                Value::structure(v1),
            )
            .unwrap(),
        )
        .unwrap();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0069).unwrap().try_checked().unwrap(),
                Value::structure(v2),
            )
            .unwrap(),
        )
        .unwrap();

    let msg = wrap_request_item(0x0000_001E, payload);
    let view = msg
        .batch_items()
        .next()
        .expect("valid request batch item view");

    let req = ServerDiscoverVersionsRequest::try_from_request_item(view)
        .expect("valid s2c discover versions");
    assert_eq!(req.protocol_versions().len(), 2);
    assert_eq!(req.protocol_versions()[0], ProtocolVersion::from_raw(2, 1));
    assert_eq!(req.protocol_versions()[1], ProtocolVersion::from_raw(1, 4));

    // Negotiation rule 1: Intersection in client preference order
    let client_supported = [ProtocolVersion::from_raw(2, 1)];
    let resp = ServerDiscoverVersionsResponse::negotiate(&req, &client_supported);
    assert_eq!(resp.protocol_versions(), &[ProtocolVersion::from_raw(2, 1)]);

    // Negotiation rule 2: Empty if no intersection
    let client_supported_v20 = [ProtocolVersion::from_raw(2, 0)];
    let resp_empty = ServerDiscoverVersionsResponse::negotiate(&req, &client_supported_v20);
    assert_eq!(resp_empty.protocol_versions(), []);
}

#[test]
fn test_server_discover_versions_request_decoding_omitted_versions() {
    // Negotiation rule 3: If omitted, respond with all supported client versions
    let payload = Structure::new(); // empty payload

    let msg = wrap_request_item(0x0000_001E, payload);
    let view = msg.batch_items().next().expect("valid view");

    let req = ServerDiscoverVersionsRequest::try_from_request_item(view)
        .expect("valid s2c discover versions");
    assert_eq!(req.protocol_versions(), []);

    let client_supported = [ProtocolVersion::from_raw(2, 1)];
    let resp = ServerDiscoverVersionsResponse::negotiate(&req, &client_supported);
    assert_eq!(resp.protocol_versions(), &[ProtocolVersion::from_raw(2, 1)]);

    let payload = resp.to_ttlv_payload().expect("payload build succeeds");
    assert_eq!(payload.view().children().len(), 1);
    assert_eq!(payload.view().children()[0].tag().raw(), 0x0042_0069);
}

#[test]
fn test_server_discover_versions_constructors_and_errors() {
    use crate::server_to_client::discover_versions::ServerDiscoverVersionsError;

    let req = ServerDiscoverVersionsRequest::new(vec![ProtocolVersion::from_raw(2, 1)]);
    assert_eq!(req.protocol_versions(), &[ProtocolVersion::from_raw(2, 1)]);

    let resp = ServerDiscoverVersionsResponse::new(vec![ProtocolVersion::from_raw(2, 1)]);
    assert_eq!(resp.protocol_versions(), &[ProtocolVersion::from_raw(2, 1)]);

    let err1 = ServerDiscoverVersionsError::UnexpectedOperation;
    assert!(format!("{err1}").contains("Discover Versions"));

    let err2 = ServerDiscoverVersionsError::MissingPayload;
    assert!(format!("{err2}").contains("missing payload"));

    let err3 = ServerDiscoverVersionsError::MalformedProtocolVersion;
    assert!(format!("{err3}").contains("malformed Protocol Version"));

    // Wrong operation rejection
    let msg = wrap_request_item(0x0000_0001, Structure::new());
    let view = msg.batch_items().next().expect("valid view");
    let err = ServerDiscoverVersionsRequest::try_from_request_item(view).unwrap_err();
    assert_eq!(err, ServerDiscoverVersionsError::UnexpectedOperation);
}
