use crate::{
    AttributeSet, RequestBatchItemView, ServerNotifyRequest, ServerNotifyResponse,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

#[test]
fn test_server_notify_request_decoding_with_attributes_and_deleted_references() {
    let mut attrs = Structure::new();
    attrs.try_push(Item::new(RawTag::new(0x0042_0057).unwrap().try_checked().unwrap(), Value::enumeration(2)).unwrap()).unwrap(); // Object Type: Symmetric Key

    let mut del_ref = Structure::new();
    del_ref.try_push(Item::new(RawTag::new(0x0042_000A).unwrap().try_checked().unwrap(), Value::enumeration(0x0042_0028)).unwrap()).unwrap(); // Attribute Tag: Cryptographic Algorithm

    let mut payload = Structure::new();
    payload.try_push(Item::new(RawTag::new(0x0042_0094).unwrap().try_checked().unwrap(), Value::text_string("key-123".to_string())).unwrap()).unwrap(); // Unique Identifier
    payload.try_push(Item::new(RawTag::new(0x0042_0125).unwrap().try_checked().unwrap(), Value::structure(attrs)).unwrap()).unwrap(); // Attributes
    payload.try_push(Item::new(RawTag::new(0x0042_0126).unwrap().try_checked().unwrap(), Value::structure(del_ref)).unwrap()).unwrap(); // Deleted Attribute Reference

    let mut item_structure = Structure::new();
    item_structure.try_push(Item::new(RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(), Value::enumeration(0x0000_001B)).unwrap()).unwrap(); // Notify
    item_structure.try_push(Item::new(RawTag::new(0x0042_007F).unwrap().try_checked().unwrap(), Value::structure(payload)).unwrap()).unwrap();

    let item = Item::new(RawTag::new(0x0042_000B).unwrap().try_checked().unwrap(), Value::structure(item_structure)).unwrap();
    let view = RequestBatchItemView::try_from(&item).expect("valid view");

    let req = ServerNotifyRequest::try_from_request_item(view).expect("valid notify request");
    assert_eq!(req.unique_identifier(), "key-123");
    assert!(req.attributes().is_some());
    let attr_items = req.attributes().unwrap().as_items();
    assert_eq!(attr_items.len(), 1);
    assert_eq!(attr_items[0].tag().raw(), 0x0042_0057);

    assert_eq!(req.deleted_attribute_references().len(), 1);

    // Response must be empty payload
    let resp = ServerNotifyResponse::new();
    let resp_payload = resp.to_ttlv_payload().expect("payload build succeeds");
    assert!(resp_payload.as_items().is_empty());
}

#[test]
fn test_server_notify_missing_unique_identifier_rejected() {
    let payload = Structure::new(); // Missing required Unique Identifier

    let mut item_structure = Structure::new();
    item_structure.try_push(Item::new(RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(), Value::enumeration(0x0000_001B)).unwrap()).unwrap();
    item_structure.try_push(Item::new(RawTag::new(0x0042_007F).unwrap().try_checked().unwrap(), Value::structure(payload)).unwrap()).unwrap();

    let item = Item::new(RawTag::new(0x0042_000B).unwrap().try_checked().unwrap(), Value::structure(item_structure)).unwrap();
    let view = RequestBatchItemView::try_from(&item).expect("valid view");

    assert!(ServerNotifyRequest::try_from_request_item(view).is_err());
}
