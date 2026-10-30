#[test]
fn public_version_matches_the_synchronized_package_version() {
    assert_eq!(kmipkit::VERSION, env!("CARGO_PKG_VERSION"));
    assert_eq!(kmipkit::VERSION, "1.0.0");
    let runtime_version = kmipkit::runtime_version();
    assert_eq!(runtime_version.product_version, kmipkit::VERSION);
    assert_eq!(runtime_version.abi_version, 1);
}

#[test]
fn capability_query_matches_manifest_ready_capabilities() {
    assert_eq!(
        kmipkit::capability_is_available("ffi_runtime_metadata"),
        Ok(true)
    );
    assert_eq!(
        kmipkit::capability_is_available("generic_ttlv_model"),
        Ok(true)
    );
    assert_eq!(
        kmipkit::capability_is_available("ttlv_wire_codec"),
        Ok(true)
    );
    assert_eq!(
        kmipkit::capability_is_available("vendor_extension_registry"),
        Ok(true)
    );
    assert_eq!(
        kmipkit::capability_is_available("discover_versions_request_item"),
        Ok(true)
    );
    assert_eq!(
        kmipkit::capability_is_available("usage_allocation_guard"),
        Ok(true)
    );
    assert_eq!(
        kmipkit::capability_is_available("opaque_register_context"),
        Ok(true)
    );
    assert_eq!(
        kmipkit::capability_is_available("server_generated_creation"),
        Ok(true)
    );
    assert_eq!(
        kmipkit::capability_is_available("typed_message_model"),
        Ok(true)
    );
    assert_eq!(
        kmipkit::capability_is_available("public_binding_surface"),
        Ok(false)
    );
    assert_eq!(
        kmipkit::capability_is_available("unknown_capability"),
        Ok(false)
    );
}

#[test]
fn capability_query_rejects_ids_over_128_utf8_bytes_as_a_resource_limit() {
    assert_eq!(
        kmipkit::capability_is_available(&"x".repeat(128)),
        Ok(false)
    );

    let error = kmipkit::capability_is_available(&"x".repeat(129))
        .expect_err("an overlong capability identifier is a resource-limit error");
    assert_eq!(error, kmipkit::RuntimeMetadataError::ResourceLimit);
    assert_eq!(
        error.to_string(),
        "capability identifier exceeds the 128-byte limit"
    );

    let multibyte = "🔐".repeat(33);
    assert_eq!(multibyte.chars().count(), 33);
    assert!(multibyte.len() > 128);
    assert!(kmipkit::capability_is_available(&multibyte).is_err());
}
