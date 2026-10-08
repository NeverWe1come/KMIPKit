use super::*;

fn byte_string_data(value: &Value) -> *const u8 {
    value
        .with_value(|value| {
            Ok::<*const u8, i32>(match value {
                ValueView::ByteString(bytes) => bytes.as_ptr(),
                _ => ptr::null(),
            })
        })
        .unwrap()
}

fn value_view_byte_string_data(value: &HandleValue) -> *const u8 {
    let HandleValue::ValueView(value) = value else {
        return ptr::null();
    };
    value
        .with_value(|value| {
            Ok::<*const u8, i32>(match value {
                ValueView::ByteString(bytes) => bytes.as_ptr(),
                _ => ptr::null(),
            })
        })
        .unwrap()
}

fn checked_tag(raw: u32) -> Tag {
    RawTag::new(raw).unwrap().try_checked().unwrap()
}

fn valid_extension_definition() -> extension::ExtensionDefinition {
    let discriminator_tag = checked_tag(0x0042_0001);
    let payload_tag = checked_tag(0x0042_0002);
    let identity = extension::extension_identity("example.vendor", "view-test", "1").unwrap();
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0").unwrap();
    let discriminator_path = extension::ttlv_path(discriminator_tag).unwrap();
    let discriminator = extension::discriminator(
        discriminator_path,
        Value::text_string("view-test-v1".to_owned()),
    )
    .unwrap();
    let schema = extension::structure(
        vec![
            extension::required(
                discriminator_tag,
                extension::scalar(ItemType::TextString).unwrap(),
            )
            .unwrap(),
            extension::required(
                payload_tag,
                extension::scalar(ItemType::ByteString).unwrap(),
            )
            .unwrap(),
        ],
        Vec::new(),
        false,
    )
    .unwrap();
    extension::extension_definition(identity, compatibility, discriminator, schema).unwrap()
}

fn valid_extension_payload(bytes: Vec<u8>) -> Structure {
    let mut structure = Structure::new();
    structure
        .try_push(
            Item::new(
                checked_tag(0x0042_0001),
                Value::text_string("view-test-v1".to_owned()),
            )
            .unwrap(),
        )
        .unwrap();
    structure
        .try_push(Item::new(checked_tag(0x0042_0002), Value::byte_string(bytes)).unwrap())
        .unwrap();
    structure
}

fn recognized_extension_payload() -> (*const u8, *mut kmipkit_extension_recognition_t) {
    let payload = valid_extension_payload(vec![0xA5, 0x5A, 0xC3, 0x3C]);
    let source_data = payload.view().children()[1].with_value(|value| match value {
        ValueView::ByteString(bytes) => bytes.as_ptr(),
        _ => ptr::null(),
    });
    let registry = client_extension::client_extension_registry(
        vec![valid_extension_definition()],
        extension::defaults(),
    )
    .unwrap();
    let recognition = client_extension::inspect(
        &registry,
        "example.vendor",
        payload,
        &CodecLimits::defaults(),
    )
    .unwrap();
    assert!(client_extension::is_recognized(&recognition));
    (
        source_data,
        make_handle::<kmipkit_extension_recognition_t>(
            Kind::Recognition,
            HandleValue::Recognition(recognition),
        )
        .cast::<kmipkit_extension_recognition_t>(),
    )
}

#[test]
fn value_view_retains_the_existing_payload_allocation() {
    let source_value = Value::byte_string(vec![0x11, 0x22, 0x33, 0x44]);
    let source_data = byte_string_data(&source_value);
    let source = make_handle(Kind::Value, HandleValue::Value(source_value));
    let mut view = ptr::null_mut();

    assert_eq!(kmipkit_ttlv_value_view(source, &raw mut view), SUCCESS);
    let view_handle = reference_handle(&view, Kind::ValueView).unwrap();
    let view_data = value_view_byte_string_data(&view_handle.value);
    let shares_payload = view_data == source_data;

    kmipkit_ttlv_value_release(source);
    let mut length = 0;
    let mut last_byte = 0;
    assert_eq!(
        kmipkit_ttlv_value_view_byte_length(view, &raw mut length),
        SUCCESS
    );
    assert_eq!(length, 4);
    assert_eq!(
        kmipkit_ttlv_value_view_byte_at(view, 3, &raw mut last_byte),
        SUCCESS
    );
    kmipkit_ttlv_value_view_release(view);

    assert!(shares_payload);
    assert_eq!(last_byte, 0x44);
}

#[test]
fn structure_item_and_value_views_retain_the_original_payload_allocation() {
    let tag = RawTag::new(0x0042_0001)
        .and_then(|raw| raw.try_checked())
        .unwrap();
    let source_value = Value::byte_string(vec![0xA1, 0xB2, 0xC3]);
    let source_data = byte_string_data(&source_value);
    let item = Item::new(tag, source_value).unwrap();
    let mut source_structure = Structure::new();
    source_structure.try_push(item).unwrap();
    let source = make_handle(Kind::Structure, HandleValue::Structure(source_structure));
    let mut structure_view = ptr::null_mut();
    let mut item_view = ptr::null_mut();
    let mut value_view = ptr::null_mut();

    assert_eq!(
        kmipkit_ttlv_structure_view(source, &raw mut structure_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_structure_view_item_at(structure_view, 0, &raw mut item_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_item_view_value(item_view, &raw mut value_view),
        SUCCESS
    );
    let value_handle = reference_handle(&value_view, Kind::ValueView).unwrap();
    let view_data = value_view_byte_string_data(&value_handle.value);
    let shares_payload = view_data == source_data;

    kmipkit_ttlv_structure_release(source);
    kmipkit_ttlv_item_view_release(item_view);
    kmipkit_ttlv_structure_view_release(structure_view);
    let mut last_byte = 0;
    assert_eq!(
        kmipkit_ttlv_value_view_byte_at(value_view, 2, &raw mut last_byte),
        SUCCESS
    );
    kmipkit_ttlv_value_view_release(value_view);

    assert!(shares_payload);
    assert_eq!(last_byte, 0xC3);
}

#[test]
fn value_structure_descendants_retain_the_original_payload_allocation() {
    let tag = RawTag::new(0x0042_0001)
        .and_then(|raw| raw.try_checked())
        .unwrap();
    let source_value = Value::byte_string(vec![0xE1, 0xE2]);
    let source_data = byte_string_data(&source_value);
    let mut nested = Structure::new();
    nested
        .try_push(Item::new(tag, source_value).unwrap())
        .unwrap();
    let source = make_handle(Kind::Value, HandleValue::Value(Value::structure(nested)));
    let mut value_view = ptr::null_mut();
    let mut structure_view = ptr::null_mut();
    let mut item_view = ptr::null_mut();
    let mut leaf_view = ptr::null_mut();

    assert_eq!(
        kmipkit_ttlv_value_view(source, &raw mut value_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_value_view_structure(value_view, &raw mut structure_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_structure_view_item_at(structure_view, 0, &raw mut item_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_item_view_value(item_view, &raw mut leaf_view),
        SUCCESS
    );
    let leaf_handle = reference_handle(&leaf_view, Kind::ValueView).unwrap();
    let leaf_data = value_view_byte_string_data(&leaf_handle.value);
    let shares_payload = leaf_data == source_data;

    kmipkit_ttlv_value_release(source);
    kmipkit_ttlv_value_view_release(value_view);
    kmipkit_ttlv_structure_view_release(structure_view);
    kmipkit_ttlv_item_view_release(item_view);
    let mut last_byte = 0;
    assert_eq!(
        kmipkit_ttlv_value_view_byte_at(leaf_view, 1, &raw mut last_byte),
        SUCCESS
    );
    kmipkit_ttlv_value_view_release(leaf_view);

    assert!(shares_payload);
    assert_eq!(last_byte, 0xE2);
}

#[test]
fn recognition_generic_view_retains_an_unrecognized_payload() {
    let source_value = Value::byte_string(vec![0x51, 0x52, 0x53]);
    let source_data = byte_string_data(&source_value);
    let mut structure = Structure::new();
    structure
        .try_push(Item::new(checked_tag(0x0042_0001), source_value).unwrap())
        .unwrap();
    let registry =
        client_extension::client_extension_registry(Vec::new(), extension::defaults()).unwrap();
    let recognition = client_extension::inspect(
        &registry,
        "example.vendor",
        structure,
        &CodecLimits::defaults(),
    )
    .unwrap();
    let source = make_handle(Kind::Recognition, HandleValue::Recognition(recognition));
    let mut generic_view = ptr::null_mut();
    let mut item_view = ptr::null_mut();
    let mut value_view = ptr::null_mut();

    assert_eq!(
        kmipkit_extension_recognition_generic_value(source, &raw mut generic_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_structure_view_item_at(generic_view, 0, &raw mut item_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_item_view_value(item_view, &raw mut value_view),
        SUCCESS
    );
    let value_handle = reference_handle(&value_view, Kind::ValueView).unwrap();
    let shares_payload = value_view_byte_string_data(&value_handle.value) == source_data;

    kmipkit_extension_recognition_release(source);
    kmipkit_ttlv_structure_view_release(generic_view);
    kmipkit_ttlv_item_view_release(item_view);
    let mut last_byte = 0;
    assert_eq!(
        kmipkit_ttlv_value_view_byte_at(value_view, 2, &raw mut last_byte),
        SUCCESS
    );
    kmipkit_ttlv_value_view_release(value_view);

    assert!(shares_payload);
    assert_eq!(last_byte, 0x53);
}

#[test]
fn validated_generic_views_retain_the_original_payload() {
    let (source_data, source) = recognized_extension_payload();
    let mut validated = ptr::null_mut();
    let mut generic_view = ptr::null_mut();
    let mut generic_item = ptr::null_mut();
    let mut generic_value = ptr::null_mut();
    let mut recognition_generic = ptr::null_mut();
    let mut recognition_item = ptr::null_mut();
    let mut recognition_value = ptr::null_mut();

    assert_eq!(
        kmipkit_extension_recognition_validated_value(source, &raw mut validated),
        SUCCESS
    );
    assert_eq!(
        kmipkit_extension_recognition_generic_value(source, &raw mut recognition_generic),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_structure_view_item_at(recognition_generic, 1, &raw mut recognition_item),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_item_view_value(recognition_item, &raw mut recognition_value),
        SUCCESS
    );
    assert_eq!(
        kmipkit_validated_extension_value_generic_value(validated, &raw mut generic_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_structure_view_item_at(generic_view, 1, &raw mut generic_item),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_item_view_value(generic_item, &raw mut generic_value),
        SUCCESS
    );
    let recognition_handle = reference_handle(&recognition_value, Kind::ValueView).unwrap();
    let generic_handle = reference_handle(&generic_value, Kind::ValueView).unwrap();
    let recognition_shares = value_view_byte_string_data(&recognition_handle.value) == source_data;
    let generic_shares = value_view_byte_string_data(&generic_handle.value) == source_data;

    kmipkit_extension_recognition_release(source);
    kmipkit_validated_extension_value_release(validated);
    kmipkit_ttlv_structure_view_release(recognition_generic);
    kmipkit_ttlv_item_view_release(recognition_item);
    kmipkit_ttlv_structure_view_release(generic_view);
    kmipkit_ttlv_item_view_release(generic_item);
    let mut generic_last_byte = 0;
    let mut recognition_last_byte = 0;
    assert_eq!(
        kmipkit_ttlv_value_view_byte_at(recognition_value, 3, &raw mut recognition_last_byte),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_value_view_byte_at(generic_value, 3, &raw mut generic_last_byte),
        SUCCESS
    );
    kmipkit_ttlv_value_view_release(recognition_value);
    kmipkit_ttlv_value_view_release(generic_value);

    assert!(recognition_shares);
    assert!(generic_shares);
    assert_eq!(recognition_last_byte, 0x3C);
    assert_eq!(generic_last_byte, 0x3C);
}

#[test]
fn validated_tag_path_views_retain_the_original_payload() {
    let (source_data, source) = recognized_extension_payload();
    let mut validated = ptr::null_mut();
    let path = make_handle(
        Kind::Path,
        HandleValue::Path(extension::ttlv_path(checked_tag(0x0042_0002)).unwrap()),
    );
    let bad_nested_path = make_handle(
        Kind::Path,
        HandleValue::Path(
            extension::with_child_tag(
                extension::ttlv_path(checked_tag(0x0042_0002)).unwrap(),
                checked_tag(0x0042_0003),
            )
            .unwrap(),
        ),
    );
    assert_eq!(
        kmipkit_extension_recognition_validated_value(source, &raw mut validated),
        SUCCESS
    );
    let mut path_value = ptr::null_mut();
    let mut bad_path_value = ptr::null_mut();
    assert_eq!(
        kmipkit_validated_extension_value_value_at(validated, path, &raw mut path_value),
        SUCCESS
    );
    let bad_path_status = kmipkit_validated_extension_value_value_at(
        validated,
        bad_nested_path,
        &raw mut bad_path_value,
    );
    let path_handle = reference_handle(&path_value, Kind::ValueView).unwrap();
    let path_shares = value_view_byte_string_data(&path_handle.value) == source_data;

    kmipkit_extension_recognition_release(source);
    kmipkit_validated_extension_value_release(validated);
    kmipkit_ttlv_path_release(path);
    kmipkit_ttlv_path_release(bad_nested_path);
    let mut path_last_byte = 0;
    assert_eq!(
        kmipkit_ttlv_value_view_byte_at(path_value, 3, &raw mut path_last_byte),
        SUCCESS
    );
    kmipkit_ttlv_value_view_release(path_value);

    assert!(path_shares);
    assert_eq!(bad_path_status, ERROR_INVALID_INPUT);
    assert!(bad_path_value.is_null());
    assert_eq!(path_last_byte, 0x3C);
}

#[test]
fn structure_view_rejects_an_out_of_range_item_index() {
    let source = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut structure_view = ptr::null_mut();
    let mut item_view = ptr::null_mut();

    assert_eq!(
        kmipkit_ttlv_structure_view(source, &raw mut structure_view),
        SUCCESS
    );
    let status = kmipkit_ttlv_structure_view_item_at(structure_view, 0, &raw mut item_view);

    kmipkit_ttlv_structure_view_release(structure_view);
    kmipkit_ttlv_structure_release(source);

    assert_eq!(status, ERROR_INVALID_INPUT);
    assert!(item_view.is_null());
}

#[test]
fn repeated_tags_keep_index_selected_views_on_the_same_item() {
    let tag = checked_tag(0x0042_0002);
    let mut structure = Structure::new();
    structure
        .try_push(Item::new(tag, Value::byte_string(vec![0x11])).unwrap())
        .unwrap();
    structure
        .try_push(Item::new(tag, Value::byte_string(vec![0x22])).unwrap())
        .unwrap();
    let source_data = structure.view().children()[1].with_value(|value| match value {
        ValueView::ByteString(bytes) => bytes.as_ptr(),
        _ => ptr::null(),
    });
    let source = make_handle(Kind::Structure, HandleValue::Structure(structure));
    let mut structure_view = ptr::null_mut();
    let mut item_view = ptr::null_mut();
    let mut value_view = ptr::null_mut();

    assert_eq!(
        kmipkit_ttlv_structure_view(source, &raw mut structure_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_structure_view_item_at(structure_view, 1, &raw mut item_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_item_view_value(item_view, &raw mut value_view),
        SUCCESS
    );
    let value = reference_handle(&value_view, Kind::ValueView).unwrap();
    let shares_second_payload = value_view_byte_string_data(&value.value) == source_data;

    kmipkit_ttlv_structure_release(source);
    kmipkit_ttlv_structure_view_release(structure_view);
    kmipkit_ttlv_item_view_release(item_view);
    let mut last_byte = 0;
    assert_eq!(
        kmipkit_ttlv_value_view_byte_at(value_view, 0, &raw mut last_byte),
        SUCCESS
    );
    kmipkit_ttlv_value_view_release(value_view);

    assert!(shares_second_payload);
    assert_eq!(last_byte, 0x22);
}

#[test]
fn view_path_accepts_exactly_sixty_four_ttlv_levels() {
    let source = make_handle::<kmipkit_ttlv_structure_t>(
        Kind::Structure,
        HandleValue::Structure(Structure::new()),
    );
    let owner = clone_handle_owner(source, Kind::Structure).unwrap();
    let mut view = TtlvView::new(owner, ViewRoot::Structure, ViewTarget::Structure);

    for index in 0..64 {
        view = view
            .append(ViewStep::Item(index), ViewTarget::Item)
            .unwrap();
        view = view.append(ViewStep::Value, ViewTarget::Value).unwrap();
    }
    let beyond_limit = view.append(ViewStep::Item(64), ViewTarget::Item).err();
    kmipkit_ttlv_structure_release(source);

    assert_eq!(view.steps.len(), MAX_VIEW_PATH_STEPS);
    assert_eq!(beyond_limit, Some(ERROR_RESOURCE_LIMIT));
}

#[test]
fn wrong_item_type_accessors_return_invalid_input() {
    let byte_value = make_handle(
        Kind::Value,
        HandleValue::Value(Value::byte_string(vec![0xA5])),
    );
    let mut byte_view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_value_view(byte_value, &raw mut byte_view),
        SUCCESS
    );
    let mut integer = 0;
    assert_eq!(
        kmipkit_ttlv_value_view_integer(byte_view, &raw mut integer),
        ERROR_INVALID_INPUT
    );

    let scalar_value = make_handle(Kind::Value, HandleValue::Value(Value::integer(7)));
    let mut scalar_view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_value_view(scalar_value, &raw mut scalar_view),
        SUCCESS
    );
    let mut byte_length = 0;
    let mut structure_view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_value_view_byte_length(scalar_view, &raw mut byte_length),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_ttlv_value_view_structure(scalar_view, &raw mut structure_view),
        ERROR_INVALID_INPUT
    );
    assert!(structure_view.is_null());

    kmipkit_ttlv_value_view_release(byte_view);
    kmipkit_ttlv_value_release(byte_value);
    kmipkit_ttlv_value_view_release(scalar_view);
    kmipkit_ttlv_value_release(scalar_value);
}
