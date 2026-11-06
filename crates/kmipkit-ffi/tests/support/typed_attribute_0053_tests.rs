use super::*;

const ACTIVATION_DATE_TAG: u32 = 0x0042_0001;
const NAME_TAG: u32 = 0x0042_0053;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const VENDOR_TAG: u32 = 0x0054_0001;
const TAG_MISMATCH_STATUS: i32 = 12;
const VALUE_TYPE_MISMATCH_STATUS: i32 = 13;

#[link(name = "kmipkit_typed_attribute_parity_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_0053_typed_attribute_from_item(
        expected_tag: u32,
        item: *mut kmipkit_ttlv_item_t,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_typed_attribute_from_value(
        expected_tag: u32,
        value: *mut kmipkit_ttlv_value_t,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_typed_attribute_value(
        expected_tag: u32,
        item: *mut kmipkit_ttlv_item_t,
        out_value: *mut *mut kmipkit_ttlv_value_t,
    ) -> i32;
    fn kmipkit_0053_attribute_set_try_push_typed(
        attributes: *mut kmipkit_attribute_set_t,
        expected_tag: u32,
        item: *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_attribute_set_typed_count(
        attributes: *mut kmipkit_attribute_set_t,
        expected_tag: u32,
        out_count: *mut u64,
    ) -> i32;
    fn kmipkit_0053_attribute_set_typed_at(
        attributes: *mut kmipkit_attribute_set_t,
        expected_tag: u32,
        occurrence: u64,
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

fn call_compiled_c_value_constructor(
    expected_tag: u32,
    value: *mut kmipkit_ttlv_value_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    // SAFETY: Test-owned value and output slots remain live during the C call.
    unsafe { kmipkit_0053_typed_attribute_from_value(expected_tag, value, out_item) }
}

fn call_compiled_c_value_reader(
    expected_tag: u32,
    item: *mut kmipkit_ttlv_item_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    // SAFETY: Test-owned item and output slots remain live during the C call.
    unsafe { kmipkit_0053_typed_attribute_value(expected_tag, item, out_value) }
}

fn call_compiled_c_set_push_typed(
    attributes: *mut kmipkit_attribute_set_t,
    expected_tag: u32,
    item: *mut kmipkit_ttlv_item_t,
) -> i32 {
    // SAFETY: The test-owned set and Item handles remain live during the C call.
    unsafe { kmipkit_0053_attribute_set_try_push_typed(attributes, expected_tag, item) }
}

fn call_compiled_c_set_typed_count(
    attributes: *mut kmipkit_attribute_set_t,
    expected_tag: u32,
    out_count: *mut u64,
) -> i32 {
    // SAFETY: The test-owned set and count output slot remain live during the C call.
    unsafe { kmipkit_0053_attribute_set_typed_count(attributes, expected_tag, out_count) }
}

fn call_compiled_c_set_typed_at(
    attributes: *mut kmipkit_attribute_set_t,
    expected_tag: u32,
    occurrence: u64,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    // SAFETY: The test-owned set and output slot remain live during the C call.
    unsafe { kmipkit_0053_attribute_set_typed_at(attributes, expected_tag, occurrence, out_item) }
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
        "Structure" if tag == 0x0042_0004 => {
            let mut structure = Structure::new();
            for (child_tag, value) in [
                (0x0054_0001, Value::text_string("first unknown".to_owned())),
                (0x0042_000a, Value::text_string("known child".to_owned())),
                (0x0054_0001, Value::text_string("second unknown".to_owned())),
            ] {
                let child_tag = RawTag::new(child_tag)
                    .expect("Structure child tag fits")
                    .try_checked()
                    .expect("Structure child tag is allocated");
                structure
                    .try_push(Item::new(child_tag, value).expect("Structure child is valid"))
                    .expect("Structure remains within limits");
            }
            Value::structure(structure)
        }
        "Structure" => Value::structure(Structure::new()),
        "TextString" | "UniqueIdentifier" => Value::text_string("attribute-value".to_owned()),
        _ => panic!("catalogued attribute has an unsupported test value kind: {kind}"),
    }
}

fn values_match(actual: ValueView<'_>, expected: ValueView<'_>) -> bool {
    match (actual, expected) {
        (ValueView::Structure(actual), ValueView::Structure(expected)) => {
            actual.children().len() == expected.children().len()
                && actual
                    .children()
                    .iter()
                    .zip(expected.children())
                    .all(|(left, right)| {
                        left.tag() == right.tag()
                            && left.with_value(|value| {
                                right.with_value(|expected| values_match(value, expected))
                            })
                    })
        }
        (ValueView::Integer(actual), ValueView::Integer(expected)) => actual == expected,
        (ValueView::LongInteger(actual), ValueView::LongInteger(expected)) => actual == expected,
        (ValueView::BigInteger(actual), ValueView::BigInteger(expected)) => actual == expected,
        (ValueView::Enumeration(actual), ValueView::Enumeration(expected)) => actual == expected,
        (ValueView::Boolean(actual), ValueView::Boolean(expected)) => actual == expected,
        (ValueView::TextString(actual), ValueView::TextString(expected)) => actual == expected,
        (ValueView::ByteString(actual), ValueView::ByteString(expected)) => actual == expected,
        (ValueView::DateTime(actual), ValueView::DateTime(expected)) => actual == expected,
        (ValueView::Interval(actual), ValueView::Interval(expected)) => actual == expected,
        (ValueView::DateTimeExtended(actual), ValueView::DateTimeExtended(expected)) => {
            actual == expected
        }
        _ => false,
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
            TAG_MISMATCH_STATUS,
        ),
        (
            NAME_TAG,
            NAME_TAG,
            Value::text_string("sensitive name".to_owned()),
            VALUE_TYPE_MISMATCH_STATUS,
        ),
        (
            VENDOR_TAG,
            VENDOR_TAG,
            Value::byte_string(b"vendor".to_vec()),
            ERROR_INVALID_INPUT,
        ),
    ];

    for (expected_tag, item_tag, value, expected_status) in cases {
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

        assert_eq!(
            status, expected_status,
            "expected tag {expected_tag:#010x}, item tag {item_tag:#010x}"
        );
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

#[test]
fn typed_attribute_value_construction_roundtrips_all_sixty_two_definitions() {
    assert_eq!(
        typed_attributes_catalog_generated::ATTRIBUTE_DEFINITIONS.len(),
        62
    );
    for (tag, kind) in typed_attributes_catalog_generated::ATTRIBUTE_DEFINITIONS {
        let source = make_handle(Kind::Value, HandleValue::Value(catalog_value(*tag, kind)));
        let mut item = ptr::null_mut();
        let status = call_compiled_c_value_constructor(*tag, source, &raw mut item);
        assert_eq!(status, SUCCESS, "typed construction {tag:#08x} ({kind})");
        let source_handle =
            reference_handle(source, Kind::Value).expect("caller value stays owned");
        let HandleValue::Value(source_value) = &source_handle.value else {
            panic!("input handle stores the original TTLV value");
        };
        let result = reference_handle(item, Kind::Item).expect("typed Item is owned");
        let HandleValue::Item(result_item) = &result.value else {
            panic!("typed result handle stores an Item");
        };
        assert_eq!(result_item.tag().raw(), *tag);
        assert_eq!(result_item.item_type(), source_value.item_type());
        let mut actual_value_handle = ptr::null_mut();
        assert_eq!(
            call_compiled_c_value_reader(*tag, item, &raw mut actual_value_handle),
            SUCCESS,
            "typed readback {tag:#08x} ({kind})"
        );
        let actual_handle =
            reference_handle(actual_value_handle, Kind::Value).expect("typed Value is owned");
        let HandleValue::Value(actual_value) = &actual_handle.value else {
            panic!("typed value result handle stores a TTLV Value");
        };
        let equal = source_value.with_value(|expected| {
            actual_value.with_value(|actual| values_match(actual, expected))
        });
        assert!(equal);
        kmipkit_ttlv_value_release(actual_value_handle);
        kmipkit_ttlv_item_release(item);
        kmipkit_ttlv_value_release(source);
    }
}

#[test]
fn unique_identifier_preserves_open_enum_and_integer_values_through_c_abi() {
    for value in [Value::enumeration(u32::MAX), Value::integer(-23)] {
        let source = make_handle(Kind::Value, HandleValue::Value(value));
        let mut item = ptr::null_mut();
        assert_eq!(
            call_compiled_c_value_constructor(0x0042_0094, source, &raw mut item),
            SUCCESS
        );
        let mut copied = ptr::null_mut();
        assert_eq!(
            call_compiled_c_value_reader(0x0042_0094, item, &raw mut copied),
            SUCCESS
        );
        let owner = reference_handle(copied, Kind::Value).expect("copy remains caller-owned");
        let HandleValue::Value(value) = &owner.value else {
            panic!("readback is a TTLV value");
        };
        let preserved = value.with_value(|actual| match actual {
            ValueView::Enumeration(raw) => *raw == u32::MAX,
            ValueView::Integer(raw) => *raw == -23,
            _ => false,
        });
        assert!(preserved);
        kmipkit_ttlv_value_release(copied);
        kmipkit_ttlv_item_release(item);
        kmipkit_ttlv_value_release(source);
    }
}

#[test]
fn typed_attribute_debug_and_rejected_value_redact_sensitive_contents() {
    let marker = "typed-attribute-private-marker";
    let source = make_handle(
        Kind::Value,
        HandleValue::Value(Value::text_string(marker.to_owned())),
    );
    let mut typed = ptr::null_mut();
    assert_eq!(
        call_compiled_c_value_constructor(UNIQUE_IDENTIFIER_TAG, source, &raw mut typed),
        SUCCESS
    );
    let item = reference_handle(typed, Kind::Item).expect("typed item is owned");
    let HandleValue::Item(item) = &item.value else {
        panic!("typed result is an Item");
    };
    assert!(!format!("{item:?}").contains(marker));
    let mut copied = ptr::null_mut();
    assert_eq!(
        call_compiled_c_value_reader(UNIQUE_IDENTIFIER_TAG, typed, &raw mut copied),
        SUCCESS
    );
    let value = reference_handle(copied, Kind::Value).expect("value copy is owned");
    let HandleValue::Value(value) = &value.value else {
        panic!("value readback is a TTLV value");
    };
    assert!(!format!("{value:?}").contains(marker));

    let invalid = make_handle(
        Kind::Value,
        HandleValue::Value(Value::text_string(marker.to_owned())),
    );
    let mut rejected = ptr::null_mut();
    assert_eq!(
        call_compiled_c_value_constructor(ACTIVATION_DATE_TAG, invalid, &raw mut rejected),
        VALUE_TYPE_MISMATCH_STATUS
    );
    assert!(rejected.is_null());
    assert!(reference_handle(invalid, Kind::Value).is_ok());
    kmipkit_ttlv_value_release(invalid);
    kmipkit_ttlv_value_release(copied);
    kmipkit_ttlv_item_release(typed);
    kmipkit_ttlv_value_release(source);
}

#[test]
fn application_specific_structure_roundtrip_preserves_unknown_child_order_and_values() {
    let source_value = catalog_value(0x0042_0004, "Structure");
    let source = make_handle(Kind::Value, HandleValue::Value(source_value));
    let mut item = ptr::null_mut();
    assert_eq!(
        call_compiled_c_value_constructor(0x0042_0004, source, &raw mut item),
        SUCCESS
    );
    let mut copied = ptr::null_mut();
    assert_eq!(
        call_compiled_c_value_reader(0x0042_0004, item, &raw mut copied),
        SUCCESS
    );
    let expected_owner = reference_handle(source, Kind::Value).expect("input value remains owned");
    let copied_owner = reference_handle(copied, Kind::Value).expect("readback is owned");
    let (HandleValue::Value(expected), HandleValue::Value(actual)) =
        (&expected_owner.value, &copied_owner.value)
    else {
        panic!("typed Structure construction and readback produce Values");
    };
    assert!(
        expected.with_value(|expected| actual.with_value(|actual| values_match(actual, expected)))
    );
    let children = actual.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(|child| {
                let tag = child.tag().raw();
                let text = child.with_value(|value| match value {
                    ValueView::TextString(text) => Some(text.to_owned()),
                    _ => None,
                });
                (tag, text)
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    });
    assert_eq!(
        children,
        [
            (0x0054_0001, Some("first unknown".to_owned())),
            (0x0042_000a, Some("known child".to_owned())),
            (0x0054_0001, Some("second unknown".to_owned())),
        ]
    );
    kmipkit_ttlv_value_release(copied);
    kmipkit_ttlv_item_release(item);
    kmipkit_ttlv_value_release(source);
}

#[test]
fn typed_attribute_set_supports_typed_insert_read_and_repeated_iteration() {
    let source = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut attributes = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_attribute_set_create(source, &raw mut attributes),
        SUCCESS
    );
    let values = [u32::MAX, u32::MAX - 1];
    let mut items = Vec::new();
    for value in values {
        let item = make_handle(
            Kind::Item,
            HandleValue::Item(
                Item::new(
                    RawTag::new(0x0042_0028)
                        .expect("fixture tag fits")
                        .try_checked()
                        .expect("tag is allocated"),
                    Value::enumeration(value),
                )
                .expect("fixture Item is valid"),
            ),
        );
        assert_eq!(
            call_compiled_c_set_push_typed(attributes, 0x0042_0028, item,),
            SUCCESS
        );
        items.push(item);
    }
    let mut count = 0;
    assert_eq!(
        call_compiled_c_set_typed_count(attributes, 0x0042_0028, &raw mut count),
        SUCCESS
    );
    assert_eq!(count, 2);
    let mut actual = ptr::null_mut();
    assert_eq!(
        call_compiled_c_set_typed_at(attributes, 0x0042_0028, 1, &raw mut actual),
        SUCCESS
    );
    let handle = reference_handle(actual, Kind::Item).expect("typed read is caller-owned");
    let HandleValue::Item(item) = &handle.value else {
        panic!("typed read returns an Item");
    };
    assert!(
        item.with_value(
            |value| matches!(value, ValueView::Enumeration(raw) if *raw == u32::MAX - 1)
        )
    );
    kmipkit_ttlv_item_release(actual);
    for item in items {
        kmipkit_ttlv_item_release(item);
    }
    kmipkit_attribute_set_release(attributes);
    kmipkit_ttlv_structure_release(source);
}
