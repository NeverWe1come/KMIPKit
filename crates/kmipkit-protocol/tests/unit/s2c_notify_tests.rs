use crate::{RequestMessage, ServerNotifyRequest, ServerNotifyResponse};
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
fn test_server_notify_request_decoding_with_attributes_and_deleted_references() {
    let mut attrs = Structure::new();
    attrs
        .try_push(
            Item::new(
                RawTag::new(0x0042_0057).unwrap().try_checked().unwrap(),
                Value::enumeration(2),
            )
            .unwrap(),
        )
        .unwrap(); // Object Type: Symmetric Key

    let mut del_ref = Structure::new();
    del_ref
        .try_push(
            Item::new(
                RawTag::new(0x0042_000A).unwrap().try_checked().unwrap(),
                Value::enumeration(0x0042_0028),
            )
            .unwrap(),
        )
        .unwrap(); // Attribute Tag: Cryptographic Algorithm

    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0094).unwrap().try_checked().unwrap(),
                Value::text_string("key-123".to_string()),
            )
            .unwrap(),
        )
        .unwrap(); // Unique Identifier
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0125).unwrap().try_checked().unwrap(),
                Value::structure(attrs),
            )
            .unwrap(),
        )
        .unwrap(); // Attributes
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0126).unwrap().try_checked().unwrap(),
                Value::structure(del_ref),
            )
            .unwrap(),
        )
        .unwrap(); // Deleted Attribute Reference

    let msg = wrap_request_item(0x0000_001B, payload);
    let view = msg.batch_items().next().expect("valid view");

    let req = ServerNotifyRequest::try_from_request_item(view).expect("valid notify request");
    assert_eq!(req.unique_identifier(), "key-123");
    assert!(req.attributes().is_some());
    let binding = req.attributes().unwrap().view();
    let attr_items = binding.children();
    assert_eq!(attr_items.len(), 1);
    assert_eq!(attr_items[0].tag().raw(), 0x0042_0057);

    assert_eq!(req.deleted_attribute_references().len(), 1);

    // Response must be empty payload
    let resp = ServerNotifyResponse::new();
    let resp_payload = resp.to_ttlv_payload().expect("payload build succeeds");
    assert!(resp_payload.view().children().is_empty());
}

#[test]
fn test_server_notify_missing_unique_identifier_rejected() {
    let payload = Structure::new(); // Missing required Unique Identifier

    let msg = wrap_request_item(0x0000_001B, payload);
    let view = msg.batch_items().next().expect("valid view");

    assert!(ServerNotifyRequest::try_from_request_item(view).is_err());
}

#[test]
fn test_server_notify_constructors_and_errors() {
    use crate::server_to_client::notify::ServerNotifyError;

    let req = ServerNotifyRequest::new("id-123".to_string(), None, Vec::new());
    assert_eq!(req.unique_identifier(), "id-123");
    assert!(req.attributes().is_none());
    assert!(req.deleted_attribute_references().is_empty());

    let resp = ServerNotifyResponse::default();
    assert!(resp.to_ttlv_payload().unwrap().view().children().is_empty());

    let err1 = ServerNotifyError::UnexpectedOperation;
    assert!(format!("{err1}").contains("Notify"));

    let err2 = ServerNotifyError::MissingPayload;
    assert!(format!("{err2}").contains("missing payload"));

    let err3 = ServerNotifyError::MissingUniqueIdentifier;
    assert!(format!("{err3}").contains("Unique Identifier"));

    let err4 = ServerNotifyError::MalformedPayload;
    assert!(format!("{err4}").contains("malformed Notify payload"));

    let msg = wrap_request_item(0x0000_0001, Structure::new());
    let view = msg.batch_items().next().expect("valid view");
    let err = ServerNotifyRequest::try_from_request_item(view).unwrap_err();
    assert_eq!(err, ServerNotifyError::UnexpectedOperation);
}

#[test]
fn test_server_notify_wildcards() {
    // Mismatched types for UID, ATTRIBUTES, DELETED_ATTRIBUTE_REFERENCE, plus unknown tag
    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0094).unwrap().try_checked().unwrap(),
                Value::integer(42),
            )
            .unwrap(),
        )
        .unwrap(); // not string
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0125).unwrap().try_checked().unwrap(),
                Value::integer(43),
            )
            .unwrap(),
        )
        .unwrap(); // not structure
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_014D).unwrap().try_checked().unwrap(),
                Value::integer(44),
            )
            .unwrap(),
        )
        .unwrap(); // not structure
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0001).unwrap().try_checked().unwrap(),
                Value::integer(45),
            )
            .unwrap(),
        )
        .unwrap(); // known tag, unexpected field

    let msg = wrap_request_item(0x0000_001B, payload);
    let view = msg.batch_items().next().expect("valid view");
    let res = ServerNotifyRequest::try_from_request_item(view);
    assert!(res.is_err());
}
