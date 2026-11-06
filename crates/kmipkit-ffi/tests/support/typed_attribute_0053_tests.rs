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
