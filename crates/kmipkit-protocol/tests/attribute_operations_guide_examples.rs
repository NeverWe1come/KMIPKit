use kmipkit_protocol::{
    AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, Attribute, AttributeReference,
    AttributeSet, CurrentAttribute, DeleteAttributeRequest, GetAttributeListRequest,
    GetAttributesRequest, ModifyAttributeRequest, NewAttribute, SetAttributeRequest,
    attribute::typed,
};
use kmipkit_ttlv::{Item, RawTag, Tag, Value};

fn allocated_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the example uses a 24-bit KMIP tag")
        .try_checked()
        .expect("Comment is allocated in KMIP 2.1")
}

fn comment(value: &str) -> Attribute<typed::Comment> {
    Attribute::new(value.to_owned()).expect("Comment is a catalogued Text String")
}

#[test]
fn typed_attribute_examples_retain_generic_extension_items_and_values() {
    let mut attributes = AttributeSet::new();
    attributes
        .try_push_typed(comment("finance"))
        .expect("typed Comment has its catalogued tag and value type");

    let comment_value = attributes
        .typed::<typed::Comment>()
        .next()
        .and_then(|comment| comment.with_value(str::to_owned).ok());
    assert_eq!(comment_value.as_deref(), Some("finance"));

    attributes
        .try_push(
            Item::new(
                allocated_tag(0x0054_1234),
                Value::byte_string(vec![0x01, 0x02]),
            )
            .expect("manufacturer extension Item is structurally valid"),
        )
        .expect("unknown allocated extension Item remains generic");
    assert_eq!(attributes.as_items().len(), 2);
}

#[test]
fn documented_attribute_operation_builders_create_valid_payloads() {
    let object = Some("object-id-17".to_owned());
    let by_tag = AttributeReference::tag(0x0042_00FD);

    let add =
        AddAttributeRequest::new(object.clone(), NewAttribute::from_typed(comment("finance")));
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
        Some(CurrentAttribute::from_typed(comment("legacy"))),
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
        Some(CurrentAttribute::from_typed(comment("legacy"))),
        NewAttribute::from_typed(comment("current")),
    );
    assert!(modify.to_ttlv_payload().is_ok());

    let set = SetAttributeRequest::new(object, NewAttribute::from_typed(comment("current")));
    assert!(set.to_ttlv_payload().is_ok());
}
