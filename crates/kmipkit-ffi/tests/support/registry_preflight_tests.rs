use super::*;

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
    let status = kmipkit_client_extension_registry_create(
        definitions.as_mut_ptr(),
        definitions.len() as u64,
        1,
        max_schema_nodes,
        4_096,
        4_096,
        16 * 1024 * 1024,
        4_096,
        16 * 1024 * 1024,
        4_096,
        100_000,
        200_000,
        4_194_304,
        64,
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
