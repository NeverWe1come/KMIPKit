use super::*;

#[path = "invariant_guard_tests.rs"]
mod invariant_guard_tests;

#[test]
fn join_split_key_identifier_copy_maps_reservation_failure_to_resource_limit() {
    let result = copy_join_split_key_identifier_with_reservation("identifier", |_, _| Err(()));

    assert_eq!(result, Err(ERROR_RESOURCE_LIMIT));
}

#[test]
fn register_request_create_consumes_every_input_when_attributes_have_a_live_view() {
    let attributes: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut attributes_view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view(attributes, &raw mut attributes_view,),
        SUCCESS
    );

    let object_tag = RawTag::new(0x0042_005B)
        .expect("Opaque Object tag is representable")
        .try_checked()
        .expect("Opaque Object tag is assigned");
    let object: *mut kmipkit_ttlv_item_t = make_handle(
        Kind::Item,
        HandleValue::Item(
            Item::new(object_tag, Value::structure(Structure::new()))
                .expect("empty Opaque Object structure is representable"),
        ),
    );
    let object_witness = clone_handle_owner(object, Kind::Item)
        .expect("object witness clones the live native owner");
    assert_eq!(Arc::strong_count(&object_witness), 2);

    let mut request = ptr::null_mut();
    assert_eq!(
        kmipkit_register_request_create(8, attributes, object, &raw mut request,),
        ERROR_INVALID_INPUT,
        "the live Structure view prevents exclusive payload consumption"
    );
    assert!(request.is_null());
    assert_eq!(
        Arc::strong_count(&object_witness),
        1,
        "the second consumed input is released even when the first unwrap fails"
    );

    kmipkit_ttlv_structure_view_release(attributes_view);
    drop(object_witness);
}

#[test]
fn register_request_create_rejects_a_catalogued_attribute_with_the_wrong_ttlv_type() {
    let cryptographic_length = Item::new(
        RawTag::new(0x0042_002a)
            .expect("Cryptographic Length tag is representable")
            .try_checked()
            .expect("Cryptographic Length is allocated"),
        Value::text_string("not-an-integer".to_owned()),
    )
    .expect("a TextString TTLV item is structurally valid");
    let mut attributes = Structure::new();
    attributes
        .try_push(cryptographic_length)
        .expect("the test attribute fits the Structure");
    let attributes = make_handle(Kind::Structure, HandleValue::Structure(attributes));
    let opaque_object_tag = RawTag::new(0x0042_005b)
        .expect("Opaque Object tag is representable")
        .try_checked()
        .expect("Opaque Object is allocated");
    let object = make_handle(
        Kind::Item,
        HandleValue::Item(
            Item::new(opaque_object_tag, Value::structure(Structure::new()))
                .expect("empty Opaque Object structure is structurally valid"),
        ),
    );
    let mut request = ptr::dangling_mut();

    assert_eq!(
        kmipkit_register_request_create(8, attributes, object, &raw mut request),
        ERROR_INVALID_INPUT
    );
    assert!(request.is_null());
}

#[test]
fn register_request_context_rejects_an_unassigned_value() {
    let attributes = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let opaque_object_tag = RawTag::new(0x0042_005b)
        .expect("Opaque Object tag is representable")
        .try_checked()
        .expect("Opaque Object is allocated");
    let object = make_handle(
        Kind::Item,
        HandleValue::Item(
            Item::new(opaque_object_tag, Value::structure(Structure::new()))
                .expect("empty Opaque Object structure is structurally valid"),
        ),
    );
    let mut request = ptr::null_mut();
    assert_eq!(
        kmipkit_register_request_create(8, attributes, object, &raw mut request),
        SUCCESS
    );

    let mut contextual_request = ptr::dangling_mut();
    assert_eq!(
        kmipkit_register_request_with_opaque_object_context(
            request,
            0,
            &raw mut contextual_request,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(contextual_request.is_null());
}

fn definition_handle() -> *mut kmipkit_extension_definition_t {
    let tag = RawTag::new(0x0042_0001)
        .expect("tag is representable")
        .try_checked()
        .expect("tag is allocated");
    let identity =
        extension::extension_identity("example.vendor", "alpha", "1").expect("identity is valid");
    let compatibility = extension::compatibility(
        2,
        1,
        2,
        1,
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_VERSION"),
    )
    .expect("compatibility includes the current version");
    let path = extension::ttlv_path(tag).expect("path is valid");
    let discriminator = extension::discriminator(path, Value::text_string("marker".to_owned()))
        .expect("discriminator is valid");
    let child = extension::required(
        tag,
        extension::scalar(ItemType::TextString).expect("scalar schema is valid"),
    )
    .expect("child rule is valid");
    let schema =
        extension::structure(vec![child], Vec::new(), true).expect("structure schema is valid");
    let definition =
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("definition is valid");

    make_handle(
        Kind::Definition,
        HandleValue::Definition(Box::new(definition)),
    )
}

fn call_registry_create(
    definitions: &mut [*mut kmipkit_extension_definition_t],
    max_schema_nodes: u64,
) -> (i32, *mut kmipkit_client_extension_registry_t) {
    let mut registry = ptr::null_mut();
    let limits = kmipkit_extension_registry_limits_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_extension_registry_limits_v1_t>())
            .expect("v1 limits structure size fits the ABI field"),
        abi_version: 1,
        max_definitions: 1,
        max_schema_nodes,
        max_child_rules_per_structure: 4_096,
        max_text_bytes_per_field: 4_096,
        max_registry_text_bytes: 16 * 1024 * 1024,
        max_discriminator_scalar_bytes: 4_096,
        max_total_discriminator_scalar_bytes: 16 * 1024 * 1024,
        max_constraint_members_per_rule: 4_096,
        max_total_constraint_members: 100_000,
        max_payload_index_records: 200_000,
        max_lookup_comparisons: 4_194_304,
        max_depth: 64,
    };
    let status = kmipkit_client_extension_registry_create_v1(
        definitions.as_mut_ptr(),
        definitions.len() as u64,
        &raw const limits,
        &raw mut registry,
    );
    (status, registry)
}

#[test]
fn registry_construction_checks_aggregate_limits_before_definition_clones() {
    let definition = definition_handle();
    let mut definitions = [definition];
    REGISTRY_DEFINITION_CLONE_COUNT.with(|count| count.set(0));

    let (status, registry) = call_registry_create(&mut definitions, 1);

    assert_eq!(status, ERROR_RESOURCE_LIMIT);
    assert!(registry.is_null());
    REGISTRY_DEFINITION_CLONE_COUNT.with(|count| assert_eq!(count.get(), 0));

    REGISTRY_DEFINITION_CLONE_COUNT.with(|count| count.set(0));
    let (status, registry) = call_registry_create(&mut definitions, 2);

    assert_eq!(status, SUCCESS);
    assert!(!registry.is_null());
    REGISTRY_DEFINITION_CLONE_COUNT.with(|count| assert_eq!(count.get(), 1));
    kmipkit_client_extension_registry_release(registry);
    kmipkit_extension_definition_release(definition);
}

fn versioned_default_limits() -> kmipkit_extension_registry_limits_v1_t {
    let mut limits = kmipkit_extension_registry_limits_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_extension_registry_limits_v1_t>())
            .expect("v1 limits structure size fits the ABI field"),
        abi_version: 1,
        max_definitions: 0,
        max_schema_nodes: 0,
        max_child_rules_per_structure: 0,
        max_text_bytes_per_field: 0,
        max_registry_text_bytes: 0,
        max_discriminator_scalar_bytes: 0,
        max_total_discriminator_scalar_bytes: 0,
        max_constraint_members_per_rule: 0,
        max_total_constraint_members: 0,
        max_payload_index_records: 0,
        max_lookup_comparisons: 0,
        max_depth: 0,
    };
    assert_eq!(
        kmipkit_extension_registry_limits_v1_default_values(&raw mut limits),
        SUCCESS
    );
    limits
}

#[test]
fn versioned_limits_reject_null_and_incompatible_headers_without_writing_fields() {
    assert_eq!(
        kmipkit_extension_registry_limits_v1_default_values(ptr::null_mut()),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_extension_registry_limits_v1_validate(ptr::null()),
        ERROR_INVALID_INPUT
    );
    let valid_size = versioned_default_limits().struct_size;
    for (struct_size, abi_version) in [(valid_size - 1, 1), (valid_size + 1, 1), (valid_size, 2)] {
        let mut limits = versioned_default_limits();
        limits.struct_size = struct_size;
        limits.abi_version = abi_version;
        limits.max_definitions = u64::MAX;
        assert_eq!(
            kmipkit_extension_registry_limits_v1_default_values(&raw mut limits),
            ERROR_INVALID_INPUT
        );
        assert_eq!(limits.struct_size, struct_size);
        assert_eq!(limits.abi_version, abi_version);
        assert_eq!(limits.max_definitions, u64::MAX);
        assert_eq!(
            kmipkit_extension_registry_limits_v1_validate(&raw const limits),
            ERROR_INVALID_INPUT
        );
    }
}

#[test]
fn versioned_registry_creation_rejects_invalid_limits_and_definition_spans() {
    let definition = definition_handle();
    let mut definitions = [definition];
    let mut limits = versioned_default_limits();
    limits.max_definitions = 1;
    for (count, limits_pointer, expected) in [
        (1, ptr::null(), ERROR_INVALID_INPUT),
        (2, &raw const limits, ERROR_RESOURCE_LIMIT),
    ] {
        let mut registry = definition.cast();
        assert_eq!(
            kmipkit_client_extension_registry_create_v1(
                definitions.as_mut_ptr(),
                count,
                limits_pointer,
                &raw mut registry,
            ),
            expected
        );
        assert!(registry.is_null());
    }
    let mut registry = definition.cast();
    assert_eq!(
        kmipkit_client_extension_registry_create_v1(
            ptr::null_mut(),
            1,
            &raw const limits,
            &raw mut registry,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(registry.is_null());
    limits.max_depth = 65;
    assert_eq!(
        kmipkit_extension_registry_limits_v1_validate(&raw const limits),
        ERROR_RESOURCE_LIMIT
    );
    assert_eq!(
        kmipkit_client_extension_registry_create_v1(
            definitions.as_mut_ptr(),
            1,
            &raw const limits,
            &raw mut registry,
        ),
        ERROR_RESOURCE_LIMIT
    );
    assert!(registry.is_null());
    // All rejected calls borrow the definition, so the caller can still inspect and release it.
    let mut identity = ptr::null_mut();
    assert_eq!(
        kmipkit_extension_definition_identity(definition, &raw mut identity),
        SUCCESS
    );
    kmipkit_extension_identity_release(identity);
    kmipkit_extension_definition_release(definition);
}

#[test]
fn legacy_limits_defaults_preflight_every_output_before_writing_any_value() {
    for missing_index in 0..12 {
        let mut values = [u64::MAX; 12];
        let mut outputs = values.each_mut().map(ptr::from_mut);
        outputs[missing_index] = ptr::null_mut();
        assert_eq!(
            kmipkit_extension_registry_limits_default_values(
                outputs[0],
                outputs[1],
                outputs[2],
                outputs[3],
                outputs[4],
                outputs[5],
                outputs[6],
                outputs[7],
                outputs[8],
                outputs[9],
                outputs[10],
                outputs[11],
            ),
            ERROR_INVALID_INPUT
        );
        assert_eq!(values, [u64::MAX; 12]);
    }
}

#[test]
fn structure_schema_rejects_non_boolean_preservation_flag_without_consuming_children() {
    let mut schema = ptr::null_mut();
    let mut child = ptr::null_mut();
    assert_eq!(kmipkit_extension_schema_scalar(7, &raw mut schema), SUCCESS);
    assert_eq!(
        kmipkit_extension_child_rule_required(0x0042_0001, schema, &raw mut child),
        SUCCESS
    );
    let mut children = [child];
    for invalid_flag in [2, u8::MAX] {
        let mut output = schema;
        assert_eq!(
            kmipkit_extension_schema_structure(
                children.as_mut_ptr(),
                1,
                ptr::null_mut(),
                0,
                invalid_flag,
                &raw mut output,
            ),
            ERROR_INVALID_INPUT
        );
        assert!(output.is_null());
    }
    let mut output = ptr::null_mut();
    assert_eq!(
        kmipkit_extension_schema_structure(
            children.as_mut_ptr(),
            1,
            ptr::null_mut(),
            0,
            1,
            &raw mut output,
        ),
        SUCCESS
    );
    kmipkit_extension_schema_release(output);
    kmipkit_extension_child_rule_release(child);
    kmipkit_extension_schema_release(schema);
}
