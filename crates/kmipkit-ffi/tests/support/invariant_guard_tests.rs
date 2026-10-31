use super::*;

// Public constructors always keep the envelope kind and payload variant consistent. These
// tests use live library allocations with an intentionally inconsistent private envelope to
// verify that a future internal regression fails closed before producing a public result.
fn inconsistent_handle<T>(kind: Kind) -> *mut T {
    let payload = if kind == Kind::Value {
        HandleValue::Structure(Structure::new())
    } else {
        HandleValue::Value(Value::integer(42))
    };
    make_handle(kind, payload)
}

fn assert_rejected_handle_output<T>(call: impl FnOnce(*mut *mut T) -> i32) {
    let mut output = ptr::null_mut();
    assert_eq!(call(&raw mut output), ERROR_INVALID_INPUT);
    assert!(
        output.is_null(),
        "a rejected operation must not produce a handle"
    );
    assert_invalid_input_snapshot();
}

fn assert_rejected_scalar<T: Copy + Eq + std::fmt::Debug>(
    sentinel: T,
    call: impl FnOnce(*mut T) -> i32,
) {
    let mut output = sentinel;
    assert_eq!(call(&raw mut output), ERROR_INVALID_INPUT);
    assert_eq!(
        output, sentinel,
        "a rejected operation must preserve scalar output"
    );
    assert_invalid_input_snapshot();
}

fn assert_invalid_input_snapshot() {
    let (mut status, mut category, mut cause, mut delivery) = (0, 0, 0, u32::MAX);
    assert_eq!(
        kmipkit_last_error_get(
            &raw mut status,
            &raw mut category,
            &raw mut cause,
            &raw mut delivery
        ),
        SUCCESS
    );
    assert_eq!(
        (status, category, cause, delivery),
        (
            ERROR_INVALID_INPUT,
            ERROR_CATEGORY_VALIDATION,
            ERROR_CAUSE_INVALID_INPUT,
            DELIVERY_STATE_NONE
        )
    );
}

#[test]
fn identity_accessors_fail_closed_for_inconsistent_private_envelopes() {
    let identity = inconsistent_handle(Kind::Identity);
    for accessor in [
        kmipkit_extension_identity_vendor_identifier,
        kmipkit_extension_identity_name,
        kmipkit_extension_identity_version,
    ] {
        assert_rejected_handle_output(|out| accessor(identity, out));
    }
    kmipkit_extension_identity_release(identity);
}

#[test]
fn consuming_extension_builders_fail_closed_for_inconsistent_private_envelopes() {
    assert_rejected_handle_output(|out| {
        kmipkit_ttlv_path_with_child_tag(inconsistent_handle(Kind::Path), 0x0042_0001, out)
    });
    for invalid_path in [true, false] {
        let mut path = ptr::null_mut();
        let mut scalar = ptr::null_mut();
        assert_eq!(
            kmipkit_ttlv_path_create(0x0042_0001, &raw mut path),
            SUCCESS
        );
        assert_eq!(kmipkit_ttlv_value_integer(42, &raw mut scalar), SUCCESS);
        if invalid_path {
            kmipkit_ttlv_path_release(path);
            path = inconsistent_handle(Kind::Path);
        } else {
            kmipkit_ttlv_value_release(scalar);
            scalar = inconsistent_handle(Kind::Value);
        }
        assert_rejected_handle_output(|out| {
            kmipkit_extension_discriminator_create(path, scalar, out)
        });
    }
    assert_rejected_handle_output(|out| {
        kmipkit_extension_schema_minimum_length(inconsistent_handle(Kind::Schema), 1, out)
    });
    assert_rejected_handle_output(|out| {
        kmipkit_client_configuration_create(inconsistent_handle(Kind::Registry), out)
    });
    assert_rejected_handle_output(|out| {
        kmipkit_client_request_message_extension_create(
            inconsistent_handle(Kind::Registered),
            0,
            out,
        )
    });
    assert_rejected_handle_output(|out| {
        kmipkit_client_batch_item_with_extension(
            inconsistent_handle(Kind::BatchItem),
            inconsistent_handle(Kind::MessageExtension),
            out,
        )
    });
}

#[test]
fn schema_and_definition_builders_fail_closed_for_inconsistent_private_envelopes() {
    let schema = inconsistent_handle(Kind::Schema);
    assert_rejected_handle_output(|out| {
        kmipkit_extension_child_rule_required(0x0042_0001, schema, out)
    });
    let identity = inconsistent_handle(Kind::Identity);
    let compatibility = inconsistent_handle(Kind::Compatibility);
    let discriminator = inconsistent_handle(Kind::Discriminator);
    assert_rejected_handle_output(|out| {
        kmipkit_extension_definition_create(identity, compatibility, discriminator, schema, out)
    });
    kmipkit_extension_identity_release(identity);
    kmipkit_extension_compatibility_release(compatibility);
    kmipkit_extension_discriminator_release(discriminator);
    kmipkit_extension_schema_release(schema);
}

#[test]
fn configuration_and_registry_queries_fail_closed_for_inconsistent_private_envelopes() {
    let configuration = inconsistent_handle(Kind::Configuration);
    assert_rejected_handle_output(|out| {
        kmipkit_client_configuration_extension_registry(configuration, out)
    });
    kmipkit_client_configuration_release(configuration);
    let registry = make_handle(
        Kind::Registry,
        HandleValue::Registry(Arc::new(
            client_extension::client_extension_registry(Vec::new(), extension::defaults()).unwrap(),
        )),
    );
    let identity = inconsistent_handle(Kind::Identity);
    let structure = inconsistent_handle(Kind::Structure);
    let limits = make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
    );
    assert_rejected_handle_output(|out| {
        kmipkit_client_extension_registry_definition_for_identity(registry, identity, out)
    });
    assert_rejected_handle_output(|out| {
        kmipkit_client_extension_registry_validate(registry, identity, structure, limits, out)
    });
    let vendor = b"example.vendor";
    assert_rejected_handle_output(|out| {
        kmipkit_client_extension_registry_inspect(
            registry,
            vendor.as_ptr(),
            vendor.len() as u64,
            structure,
            out,
            limits,
        )
    });
    let definition = definition_handle();
    assert_rejected_handle_output(|out| {
        kmipkit_extension_definition_validate(definition, structure, out, limits)
    });
    kmipkit_extension_definition_release(definition);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_extension_identity_release(identity);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_codec_limits_release(limits);
}

#[test]
fn batch_accessors_fail_closed_for_inconsistent_private_envelopes() {
    let item = inconsistent_handle(Kind::BatchItem);
    assert_rejected_scalar(u64::MAX, |out| {
        kmipkit_client_batch_item_extension_count(item, out)
    });
    assert_rejected_handle_output(|out| {
        kmipkit_client_batch_item_extension_identity_at(item, 0, out)
    });
    assert_rejected_scalar(u8::MAX, |out| {
        kmipkit_client_batch_item_extension_criticality_indicator_at(item, 0, out)
    });
    kmipkit_client_batch_item_release(item);
}

#[test]
fn information_builders_fail_closed_for_inconsistent_private_envelopes() {
    let information = inconsistent_handle(Kind::Information);
    assert_rejected_handle_output(|out| kmipkit_extension_information_to_ttlv(information, out));
    assert_rejected_handle_output(|out| {
        kmipkit_extension_definition_with_information(
            inconsistent_handle(Kind::Definition),
            information,
            out,
        )
    });
    assert_rejected_handle_output(|out| {
        kmipkit_extension_definition_with_information(definition_handle(), information, out)
    });
    kmipkit_extension_information_release(information);
    assert_rejected_handle_output(|out| {
        kmipkit_extension_information_description(
            inconsistent_handle(Kind::Information),
            ptr::null(),
            0,
            out,
        )
    });
    assert_rejected_handle_output(|out| {
        kmipkit_extension_information_tag(inconsistent_handle(Kind::Information), 0x0042_0001, out)
    });
}

#[test]
fn recognition_and_validated_queries_fail_closed_for_inconsistent_private_envelopes() {
    let recognition = inconsistent_handle(Kind::Recognition);
    assert_rejected_scalar(u32::MAX, |out| {
        kmipkit_extension_recognition_is_recognized(recognition, out)
    });
    assert_rejected_handle_output(|out| {
        kmipkit_extension_recognition_validated_value(recognition, out)
    });
    assert_rejected_handle_output(|out| {
        kmipkit_extension_recognition_generic_value(recognition, out)
    });
    kmipkit_extension_recognition_release(recognition);
    let value = inconsistent_handle(Kind::Validated);
    let path = inconsistent_handle(Kind::Path);
    assert_rejected_handle_output(|out| {
        kmipkit_validated_extension_value_value_at(value, path, out)
    });
    kmipkit_validated_extension_value_release(value);
    kmipkit_ttlv_path_release(path);
}

#[test]
fn tag_accessors_fail_closed_for_inconsistent_private_envelopes() {
    let raw_tag = inconsistent_handle(Kind::RawTag);
    assert_rejected_scalar(u32::MAX, |out| kmipkit_ttlv_raw_tag_value(raw_tag, out));
    assert_rejected_handle_output(|out| kmipkit_ttlv_raw_tag_try_checked(raw_tag, out));
    kmipkit_raw_tag_release(raw_tag);
    let tag = inconsistent_handle(Kind::Tag);
    assert_rejected_scalar(u32::MAX, |out| kmipkit_ttlv_tag_value(tag, out));
    kmipkit_tag_release(tag);
}

#[test]
fn owned_ttlv_builders_fail_closed_for_inconsistent_private_envelopes() {
    let limits = make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
    );
    assert_rejected_handle_output(|out| {
        kmipkit_ttlv_value_structure(inconsistent_handle(Kind::Structure), limits, out)
    });
    let tag = inconsistent_handle(Kind::Tag);
    let mut value = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_value_integer(42, &raw mut value), SUCCESS);
    assert_rejected_handle_output(|out| kmipkit_ttlv_item_create(tag, value, limits, out));
    // Tag rejection precedes ownership transfer, so the caller still owns the value.
    kmipkit_ttlv_value_release(value);
    kmipkit_tag_release(tag);
    let tag = make_handle(
        Kind::Tag,
        HandleValue::Tag(RawTag::new(0x0042_0001).unwrap().try_checked().unwrap()),
    );
    assert_rejected_handle_output(|out| {
        kmipkit_ttlv_item_create(tag, inconsistent_handle(Kind::Value), limits, out)
    });
    kmipkit_tag_release(tag);
    assert_rejected_handle_output(|out| {
        kmipkit_ttlv_structure_with_item(
            inconsistent_handle(Kind::Structure),
            inconsistent_handle(Kind::Item),
            limits,
            out,
        )
    });
    kmipkit_codec_limits_release(limits);
}

#[test]
fn ttlv_structure_and_item_views_fail_closed_for_inconsistent_private_envelopes() {
    let structure_view = inconsistent_handle(Kind::StructureView);
    assert_rejected_scalar(u64::MAX, |out| {
        kmipkit_ttlv_structure_view_item_count(structure_view, out)
    });
    assert_rejected_handle_output(|out| {
        kmipkit_ttlv_structure_view_item_at(structure_view, 0, out)
    });
    kmipkit_ttlv_structure_view_release(structure_view);
    let item_view = inconsistent_handle(Kind::ItemView);
    assert_rejected_handle_output(|out| kmipkit_ttlv_item_view_tag(item_view, out));
    assert_rejected_scalar(u8::MAX, |out| kmipkit_ttlv_item_view_type(item_view, out));
    assert_rejected_handle_output(|out| kmipkit_ttlv_item_view_value(item_view, out));
    kmipkit_ttlv_item_view_release(item_view);
}

#[test]
fn ttlv_value_views_fail_closed_for_inconsistent_private_envelopes() {
    let view = inconsistent_handle(Kind::ValueView);
    assert_rejected_scalar(u8::MAX, |out| kmipkit_ttlv_value_view_type(view, out));
    assert_rejected_handle_output(|out| kmipkit_ttlv_value_view_structure(view, out));
    assert_rejected_scalar(i32::MAX, |out| kmipkit_ttlv_value_view_integer(view, out));
    assert_rejected_scalar(i64::MAX, |out| {
        kmipkit_ttlv_value_view_long_integer(view, out)
    });
    assert_rejected_scalar(u32::MAX, |out| {
        kmipkit_ttlv_value_view_enumeration(view, out)
    });
    assert_rejected_scalar(u8::MAX, |out| kmipkit_ttlv_value_view_boolean(view, out));
    assert_rejected_scalar(i64::MAX, |out| kmipkit_ttlv_value_view_date_time(view, out));
    assert_rejected_scalar(u32::MAX, |out| kmipkit_ttlv_value_view_interval(view, out));
    assert_rejected_scalar(i64::MAX, |out| {
        kmipkit_ttlv_value_view_date_time_extended(view, out)
    });
    assert_rejected_scalar(u64::MAX, |out| {
        kmipkit_ttlv_value_view_byte_length(view, out)
    });
    assert_rejected_scalar(u8::MAX, |out| kmipkit_ttlv_value_view_byte_at(view, 0, out));
    kmipkit_ttlv_value_view_release(view);
}

#[test]
fn unrecognized_internal_status_uses_the_redacted_other_error_category() {
    let unknown_status = i32::MAX;
    assert_eq!(record_ffi_error(unknown_status), unknown_status);
    let (mut status, mut category, mut cause, mut delivery) = (0, 0, 0, u32::MAX);
    assert_eq!(
        kmipkit_last_error_get(
            &raw mut status,
            &raw mut category,
            &raw mut cause,
            &raw mut delivery
        ),
        SUCCESS
    );
    assert_eq!(
        (status, category, cause, delivery),
        (
            unknown_status,
            ERROR_CATEGORY_OTHER,
            ERROR_CAUSE_OTHER,
            DELIVERY_STATE_NONE
        )
    );
}
