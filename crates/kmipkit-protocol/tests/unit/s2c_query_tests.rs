use crate::{
    QueryFunction, RequestBatchItemView, ServerCapabilities, ServerQueryRequest, ServerQueryResponse,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

#[test]
fn test_server_query_request_decoding() {
    let mut payload = Structure::new();
    payload.try_push(Item::new(RawTag::new(0x0042_0071).unwrap().try_checked().unwrap(), Value::enumeration(1)).unwrap()).unwrap(); // Query Operations (1)
    payload.try_push(Item::new(RawTag::new(0x0042_0071).unwrap().try_checked().unwrap(), Value::enumeration(2)).unwrap()).unwrap(); // Query Objects (2)

    let mut item_structure = Structure::new();
    item_structure.try_push(Item::new(RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(), Value::enumeration(0x0000_0018)).unwrap()).unwrap(); // Query
    item_structure.try_push(Item::new(RawTag::new(0x0042_007F).unwrap().try_checked().unwrap(), Value::structure(payload)).unwrap()).unwrap();

    let item = Item::new(RawTag::new(0x0042_000B).unwrap().try_checked().unwrap(), Value::structure(item_structure)).unwrap();
    let view = RequestBatchItemView::try_from(&item).expect("valid view");

    let req = ServerQueryRequest::try_from_request_item(view).expect("valid query request");
    assert_eq!(req.query_functions().len(), 2);
    assert_eq!(req.query_functions()[0], QueryFunction::OPERATIONS);
    assert_eq!(req.query_functions()[1], QueryFunction::OBJECTS);
}

#[test]
fn test_server_query_response_extension_map_supersedes_extension_list() {
    // Normative rule §6.2.4: If both Extension List (5) and Extension Map (6) are requested,
    // only Extension Map is returned.
    let mut payload = Structure::new();
    payload.try_push(Item::new(RawTag::new(0x0042_0071).unwrap().try_checked().unwrap(), Value::enumeration(5)).unwrap()).unwrap(); // Extension List
    payload.try_push(Item::new(RawTag::new(0x0042_0071).unwrap().try_checked().unwrap(), Value::enumeration(6)).unwrap()).unwrap(); // Extension Map

    let mut item_structure = Structure::new();
    item_structure.try_push(Item::new(RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(), Value::enumeration(0x0000_0018)).unwrap()).unwrap();
    item_structure.try_push(Item::new(RawTag::new(0x0042_007F).unwrap().try_checked().unwrap(), Value::structure(payload)).unwrap()).unwrap();

    let item = Item::new(RawTag::new(0x0042_000B).unwrap().try_checked().unwrap(), Value::structure(item_structure)).unwrap();
    let view = RequestBatchItemView::try_from(&item).expect("valid view");

    let req = ServerQueryRequest::try_from_request_item(view).expect("valid query request");

    let mut capabilities = ServerCapabilities::default();
    capabilities.set_vendor_identification("KMIPKit");

    let resp = ServerQueryResponse::respond(&req, &capabilities);
    let payload_structure = resp.to_ttlv_payload().expect("payload build succeeds");

    // Must NOT contain Extension List (tag 0x0042_00A3) if Extension Map (0x0042_00A4) is present or requested
    for item in payload_structure.as_items() {
        assert_ne!(item.tag().raw(), 0x0042_00A3, "Extension List must not be present when Extension Map is requested");
    }
}
