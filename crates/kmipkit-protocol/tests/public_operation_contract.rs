//! Public Rust API contract tests for KMIP Get and Locate.
//!
//! KMIP payload semantics follow OASIS KMIP Specification v2.1 §6.1.19,
//! Tables 220–222, and §6.1.28, Tables 247–249. These are public-surface and
//! source-derived field-order checks, not official OASIS conformance vectors.

use kmipkit_protocol::{
    AttributeSet, GetRequest, GetResponse, KmipOperationResult, LocateRequest, LocateResponse,
    ObjectType, ResponseBatchItemView, StorageStatusMask, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, ItemType, ValueView};

const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const OFFSET_ITEMS_TAG: u32 = 0x0042_00D4;
const ATTRIBUTES_TAG: u32 = 0x0042_0125;

#[test]
fn get_request_exposes_optional_identifier_and_table_220_order() {
    assert!(GetRequest::new().unique_identifier().is_none());

    let identifier = UniqueIdentifier::TextString(String::from("public-contract-object"));
    let request = GetRequest::new().with_unique_identifier(identifier.clone());
    assert_eq!(request.unique_identifier(), Some(&identifier));

    let payload = request
        .to_ttlv_payload()
        .expect("the Table 220 Unique Identifier request is representable");
    let fields = payload.view().children();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), UNIQUE_IDENTIFIER_TAG);
    assert_eq!(fields[0].item_type(), ItemType::TextString);
    assert!(fields[0].with_value(
        |value| matches!(value, ValueView::TextString(actual) if actual == "public-contract-object")
    ));
}

#[test]
fn locate_request_keeps_empty_attributes_explicit_zero_and_table_247_order() {
    let empty_request = LocateRequest::new(AttributeSet::new());
    assert!(empty_request.attributes().is_empty());
    assert_eq!(empty_request.offset_items(), None);

    let request = empty_request.with_offset_items(0);
    assert!(request.attributes().is_empty());
    assert_eq!(request.offset_items(), Some(0));

    let payload = request
        .to_ttlv_payload()
        .expect("the required empty Attributes Structure is representable");
    let fields = payload.view().children();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [OFFSET_ITEMS_TAG, ATTRIBUTES_TAG]
    );
    assert_eq!(fields[0].item_type(), ItemType::Integer);
    assert!(fields[0].with_value(|value| matches!(value, ValueView::Integer(0))));
    assert_eq!(fields[1].item_type(), ItemType::Structure);
    assert!(fields[1].with_value(|value| matches!(
        value,
        ValueView::Structure(structure) if structure.children().is_empty()
    )));
}

#[test]
fn storage_status_mask_keeps_unknown_raw_bits() {
    for raw in [0x8000_0000, 0x8000_0005] {
        assert_eq!(StorageStatusMask::from_raw(raw).raw(), raw);
    }
}

#[test]
fn get_and_locate_models_export_contract_constructors_and_accessors() {
    assert_owned_get_response_constructor(GetResponse::try_from_response_item);
    let _: fn(&GetResponse) -> &KmipOperationResult = GetResponse::result;
    let _: fn(&GetResponse) -> Option<ObjectType> = GetResponse::object_type;
    let _: for<'a> fn(&'a GetResponse) -> Option<&'a UniqueIdentifier> =
        GetResponse::unique_identifier;
    let _: for<'a> fn(&'a GetResponse) -> Option<&'a Item> = GetResponse::object;

    assert_owned_locate_response_constructor(LocateResponse::try_from_response_item);
    let _: fn(&LocateResponse) -> &KmipOperationResult = LocateResponse::result;
    let _located_items_accessor = LocateResponse::located_items;
    let _: for<'a> fn(&'a LocateResponse) -> &'a [UniqueIdentifier] =
        LocateResponse::unique_identifiers;
}

fn assert_owned_get_response_constructor<'view, Error>(
    _: fn(ResponseBatchItemView<'view>) -> Result<GetResponse, Error>,
) {
}

fn assert_owned_locate_response_constructor<'view, Error>(
    _: fn(ResponseBatchItemView<'view>) -> Result<LocateResponse, Error>,
) {
}
