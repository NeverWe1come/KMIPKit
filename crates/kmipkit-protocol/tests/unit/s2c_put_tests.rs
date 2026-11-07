use crate::{
    PutFunction, RequestBatchItemView, ServerPutRequest, ServerPutResponse,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

#[test]
fn test_put_function_values() {
    assert_eq!(PutFunction::NEW.raw(), 1);
    assert_eq!(PutFunction::REPLACE.raw(), 2);
    assert!(PutFunction::NEW.is_known());
    assert!(PutFunction::REPLACE.is_known());
    assert_eq!(PutFunction::NEW.wire_name(), "New");
    assert_eq!(PutFunction::REPLACE.wire_name(), "Replace");
}

#[test]
fn test_server_put_request_decoding_new() {
    let mut key_block = Structure::new();
    key_block.try_push(Item::new(RawTag::new(0x0042_0042).unwrap().try_checked().unwrap(), Value::enumeration(1)).unwrap()).unwrap(); // Raw Key Format

    let mut sym_key = Structure::new();
    sym_key.try_push(Item::new(RawTag::new(0x0042_0040).unwrap().try_checked().unwrap(), Value::structure(key_block)).unwrap()).unwrap(); // Key Block

    let mut payload = Structure::new();
    payload.try_push(Item::new(RawTag::new(0x0042_0070).unwrap().try_checked().unwrap(), Value::enumeration(1)).unwrap()).unwrap(); // Put Function: New
    payload.try_push(Item::new(RawTag::new(0x0042_0094).unwrap().try_checked().unwrap(), Value::text_string("key-uuid-1".to_string())).unwrap()).unwrap(); // Unique Identifier
    payload.try_push(Item::new(RawTag::new(0x0042_008F).unwrap().try_checked().unwrap(), Value::structure(sym_key)).unwrap()).unwrap(); // Symmetric Key

    let mut item_structure = Structure::new();
    item_structure.try_push(Item::new(RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(), Value::enumeration(0x0000_001C)).unwrap()).unwrap(); // Put
    item_structure.try_push(Item::new(RawTag::new(0x0042_007F).unwrap().try_checked().unwrap(), Value::structure(payload)).unwrap()).unwrap();

    let item = Item::new(RawTag::new(0x0042_000B).unwrap().try_checked().unwrap(), Value::structure(item_structure)).unwrap();
    let view = RequestBatchItemView::try_from(&item).expect("valid view");

    let req = ServerPutRequest::try_from_request_item(view).expect("valid put request");
    assert_eq!(req.put_function(), PutFunction::NEW);
    assert_eq!(req.unique_identifier(), "key-uuid-1");
    assert_eq!(req.replaced_unique_identifier(), None);
    assert_eq!(req.object_item().tag().raw(), 0x0042_008F);

    // effective_put_function with New always returns New
    assert_eq!(req.effective_put_function(false), PutFunction::NEW);
    assert_eq!(req.effective_put_function(true), PutFunction::NEW);

    let resp = ServerPutResponse::new();
    let resp_payload = resp.to_ttlv_payload().expect("empty payload succeeds");
    assert!(resp_payload.as_items().is_empty());
}

#[test]
fn test_server_put_request_decoding_replace_and_effective_put_function() {
    let mut key_block = Structure::new();
    key_block.try_push(Item::new(RawTag::new(0x0042_0042).unwrap().try_checked().unwrap(), Value::enumeration(1)).unwrap()).unwrap();

    let mut sym_key = Structure::new();
    sym_key.try_push(Item::new(RawTag::new(0x0042_0040).unwrap().try_checked().unwrap(), Value::structure(key_block)).unwrap()).unwrap();

    let mut payload = Structure::new();
    payload.try_push(Item::new(RawTag::new(0x0042_0070).unwrap().try_checked().unwrap(), Value::enumeration(2)).unwrap()).unwrap(); // Put Function: Replace
    payload.try_push(Item::new(RawTag::new(0x0042_0094).unwrap().try_checked().unwrap(), Value::text_string("key-uuid-2".to_string())).unwrap()).unwrap();
    payload.try_push(Item::new(RawTag::new(0x0042_0076).unwrap().try_checked().unwrap(), Value::text_string("key-uuid-1".to_string())).unwrap()).unwrap(); // Replaced Unique Identifier
    payload.try_push(Item::new(RawTag::new(0x0042_008F).unwrap().try_checked().unwrap(), Value::structure(sym_key)).unwrap()).unwrap();

    let mut item_structure = Structure::new();
    item_structure.try_push(Item::new(RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(), Value::enumeration(0x0000_001C)).unwrap()).unwrap();
    item_structure.try_push(Item::new(RawTag::new(0x0042_007F).unwrap().try_checked().unwrap(), Value::structure(payload)).unwrap()).unwrap();

    let item = Item::new(RawTag::new(0x0042_000B).unwrap().try_checked().unwrap(), Value::structure(item_structure)).unwrap();
    let view = RequestBatchItemView::try_from(&item).expect("valid view");

    let req = ServerPutRequest::try_from_request_item(view).expect("valid put request");
    assert_eq!(req.put_function(), PutFunction::REPLACE);
    assert_eq!(req.replaced_unique_identifier(), Some("key-uuid-1"));

    // Normative rule §6.2.3: If replaced object does NOT exist, treat as New
    assert_eq!(req.effective_put_function(false), PutFunction::NEW);
    // If replaced object DOES exist, treat as Replace
    assert_eq!(req.effective_put_function(true), PutFunction::REPLACE);
}

#[test]
fn test_server_put_replace_without_replaced_uid_rejected() {
    let mut payload = Structure::new();
    payload.try_push(Item::new(RawTag::new(0x0042_0070).unwrap().try_checked().unwrap(), Value::enumeration(2)).unwrap()).unwrap(); // Replace
    payload.try_push(Item::new(RawTag::new(0x0042_0094).unwrap().try_checked().unwrap(), Value::text_string("key-uuid-2".to_string())).unwrap()).unwrap();
    // Missing Replaced Unique Identifier!
    let mut sym_key = Structure::new();
    payload.try_push(Item::new(RawTag::new(0x0042_008F).unwrap().try_checked().unwrap(), Value::structure(sym_key)).unwrap()).unwrap();

    let mut item_structure = Structure::new();
    item_structure.try_push(Item::new(RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(), Value::enumeration(0x0000_001C)).unwrap()).unwrap();
    item_structure.try_push(Item::new(RawTag::new(0x0042_007F).unwrap().try_checked().unwrap(), Value::structure(payload)).unwrap()).unwrap();

    let item = Item::new(RawTag::new(0x0042_000B).unwrap().try_checked().unwrap(), Value::structure(item_structure)).unwrap();
    let view = RequestBatchItemView::try_from(&item).expect("valid view");

    assert!(ServerPutRequest::try_from_request_item(view).is_err());
}
