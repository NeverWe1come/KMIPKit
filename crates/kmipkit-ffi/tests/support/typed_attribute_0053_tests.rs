use super::*;

const ACTIVATION_DATE_TAG: u32 = 0x0042_0001;
const NAME_TAG: u32 = 0x0042_000a;
const VENDOR_TAG: u32 = 0x0054_0001;

#[link(name = "kmipkit_typed_attribute_parity_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_0053_typed_attribute_from_item(
        expected_tag: u32,
        item: *mut kmipkit_ttlv_item_t,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
}

fn call_compiled_c_consumer(
    expected_tag: u32,
    item: *mut kmipkit_ttlv_item_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    // SAFETY: Test-owned live opaque handles are passed to a compiled C consumer
    // that forwards them to the public ABI without dereferencing their values.
    unsafe { kmipkit_0053_typed_attribute_from_item(expected_tag, item, out_item) }
}

fn catalog_value(tag: u32, kind: &str) -> Value {
    match kind {
        "BigInteger" => Value::big_integer(vec![1]),
        "Boolean" => Value::boolean(true),
        "ByteString" => Value::byte_string(vec![1]),
        "DateTime" => Value::date_time(1_700_000_000),
        "DateTimeExtended" => Value::date_time_extended(1_700_000_000),
        "Enumeration" => Value::enumeration(u32::MAX),
        "Integer" => Value::integer(7),
        "Interval" => Value::interval(7),
        "Structure" if tag == 0x0042_0008 => {
            let mut structure = Structure::new();
            for (child_tag, value) in [
                (
                    0x0042_009d,
                    Value::text_string("KMIPKit.TestVendor_1".to_owned()),
                ),
                (
                    0x0042_000a,
                    Value::text_string("Opaque.Attribute".to_owned()),
                ),
                (0x0042_000b, Value::byte_string(vec![1])),
            ] {
                let child_tag = RawTag::new(child_tag)
                    .expect("vendor child tag fits")
                    .try_checked()
                    .expect("vendor child tag is assigned");
                structure
                    .try_push(Item::new(child_tag, value).expect("vendor child is valid"))
                    .expect("vendor structure remains within limits");
            }
            Value::structure(structure)
        }
        "Structure" => Value::structure(Structure::new()),
        "TextString" | "UniqueIdentifier" => Value::text_string("attribute-value".to_owned()),
        _ => panic!("catalogued attribute has an unsupported test value kind: {kind}"),
    }
}

#[test]
fn compiled_c_consumer_accepts_all_sixty_two_catalogued_attribute_types() {
    assert_eq!(
        typed_attributes_catalog_generated::ATTRIBUTE_DEFINITIONS.len(),
        62
    );
    for (tag, kind) in typed_attributes_catalog_generated::ATTRIBUTE_DEFINITIONS {
        let source_item = Item::new(
            RawTag::new(*tag)
                .expect("catalog tag fits")
                .try_checked()
                .expect("catalog tag is assigned"),
            catalog_value(*tag, kind),
        )
        .expect("catalog item is valid TTLV");
        let source = make_handle(Kind::Item, HandleValue::Item(source_item));
        let mut converted = ptr::null_mut();

        let status = call_compiled_c_consumer(*tag, source, &raw mut converted);

        assert_eq!(status, SUCCESS, "attribute tag {tag:#08x} ({kind})");
        let input = reference_handle(source, Kind::Item).expect("source owner remains valid");
        let output = reference_handle(converted, Kind::Item).expect("converted Item is owned");
        let HandleValue::Item(input_item) = &input.value else {
            panic!("source handle contains Item");
        };
        let HandleValue::Item(output_item) = &output.value else {
            panic!("converted handle contains Item");
        };
        assert_eq!(output_item.tag().raw(), *tag);
        assert_eq!(output_item.item_type(), input_item.item_type());
        kmipkit_ttlv_item_release(converted);
        kmipkit_ttlv_item_release(source);
    }
}

#[test]
fn typed_attribute_from_item_preserves_valid_catalogued_item_and_input_owner() {
    let source = make_handle(
        Kind::Item,
        HandleValue::Item(
            Item::new(
                RawTag::new(ACTIVATION_DATE_TAG)
                    .expect("fixture tag fits")
                    .try_checked()
                    .expect("fixture tag is allocated"),
                Value::date_time(1_700_000_000),
            )
            .expect("fixture item is valid"),
        ),
    );
    let mut typed = ptr::null_mut();

    let status = call_compiled_c_consumer(ACTIVATION_DATE_TAG, source, &raw mut typed);

    assert_eq!(status, SUCCESS);
    assert!(!typed.is_null());
    let input = reference_handle(source, Kind::Item).expect("input owner remains valid");
    let output = reference_handle(typed, Kind::Item).expect("typed owner is valid");
    let HandleValue::Item(input_item) = &input.value else {
        panic!("source handle stores the original item");
    };
    let HandleValue::Item(output_item) = &output.value else {
        panic!("result handle stores the typed item");
    };
    assert_eq!(input_item.tag().raw(), ACTIVATION_DATE_TAG);
    assert_eq!(output_item.tag().raw(), ACTIVATION_DATE_TAG);
    assert!(output_item.with_value(|value| matches!(
        value,
        ValueView::DateTime(timestamp) if *timestamp == 1_700_000_000
    )));

    kmipkit_ttlv_item_release(typed);
    kmipkit_ttlv_item_release(source);
}

#[test]
fn typed_attribute_from_item_rejects_mismatched_tag_wrong_type_and_unknown_tag() {
    let cases = [
        (
            NAME_TAG,
            ACTIVATION_DATE_TAG,
            Value::date_time(1_700_000_000),
        ),
        (
            NAME_TAG,
            NAME_TAG,
            Value::text_string("sensitive name".to_owned()),
        ),
        (
            VENDOR_TAG,
            VENDOR_TAG,
            Value::byte_string(b"vendor".to_vec()),
        ),
    ];

    for (expected_tag, item_tag, value) in cases {
        let source = make_handle(
            Kind::Item,
            HandleValue::Item(
                Item::new(
                    RawTag::new(item_tag)
                        .expect("fixture tag fits")
                        .try_checked()
                        .expect("fixture tag is allocated"),
                    value,
                )
                .expect("fixture item is valid"),
            ),
        );
        let mut typed = ptr::null_mut();

        let status = call_compiled_c_consumer(expected_tag, source, &raw mut typed);

        assert_eq!(status, ERROR_INVALID_INPUT);
        assert!(typed.is_null());
        assert!(reference_handle(source, Kind::Item).is_ok());
        kmipkit_ttlv_item_release(source);
    }
}

#[test]
fn typed_attribute_from_item_preflights_null_output_without_consuming_input() {
    let source = make_handle(
        Kind::Item,
        HandleValue::Item(
            Item::new(
                RawTag::new(ACTIVATION_DATE_TAG)
                    .expect("fixture tag fits")
                    .try_checked()
                    .expect("fixture tag is allocated"),
                Value::date_time(1_700_000_000),
            )
            .expect("fixture item is valid"),
        ),
    );

    let status = call_compiled_c_consumer(ACTIVATION_DATE_TAG, source, ptr::null_mut());

    assert_eq!(status, ERROR_INVALID_INPUT);
    assert!(reference_handle(source, Kind::Item).is_ok());
    kmipkit_ttlv_item_release(source);
}
