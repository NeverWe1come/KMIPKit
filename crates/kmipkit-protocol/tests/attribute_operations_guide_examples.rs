use kmipkit_protocol::{
    AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, AttributeReference,
    CurrentAttribute, DeleteAttributeRequest, GetAttributeListRequest, GetAttributesRequest,
    ModifyAttributeRequest, NewAttribute, SetAttributeRequest,
};
use kmipkit_ttlv::{Item, RawTag, Tag, Value};

fn allocated_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the example uses a 24-bit KMIP tag")
        .try_checked()
        .expect("Comment is allocated in KMIP 2.1")
}

fn comment(value: &str) -> Item {
    Item::new(
        allocated_tag(0x0042_00FD),
        Value::text_string(value.to_owned()),
    )
    .expect("the example Item is structurally valid")
}

#[test]
fn documented_attribute_operation_builders_create_valid_payloads() {
    let object = Some("object-id-17".to_owned());
    let by_tag = AttributeReference::tag(0x0042_00FD);

    let add = AddAttributeRequest::new(object.clone(), NewAttribute::new(comment("finance")));
    assert!(add.to_ttlv_payload().is_ok());

    let adjust = AdjustAttributeRequest::new(
        object.clone(),
        by_tag.clone(),
        AdjustmentType::INCREMENT,
        Some(Value::integer(1)),
    );
    assert!(adjust.to_ttlv_payload().is_ok());

    let delete = DeleteAttributeRequest::new(
        object.clone(),
        Some(CurrentAttribute::new(comment("legacy"))),
        None,
    );
    assert!(delete.to_ttlv_payload().is_ok());

    let get_attributes = GetAttributesRequest::try_new(
        object.clone(),
        [
            by_tag,
            AttributeReference::name("example.org", "CostCenter"),
        ],
    )
    .expect("the two references are distinct");
    assert!(get_attributes.to_ttlv_payload().is_ok());

    let get_attribute_list = GetAttributeListRequest::new(object.clone());
    assert!(get_attribute_list.to_ttlv_payload().is_ok());

    let modify = ModifyAttributeRequest::new(
        object.clone(),
        Some(CurrentAttribute::new(comment("legacy"))),
        NewAttribute::new(comment("current")),
    );
    assert!(modify.to_ttlv_payload().is_ok());

    let set = SetAttributeRequest::new(object, NewAttribute::new(comment("current")));
    assert!(set.to_ttlv_payload().is_ok());
}
