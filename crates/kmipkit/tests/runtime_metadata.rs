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
    let ready_capabilities = [
        "client_profile_readiness",
        "client_operation_identity",
        "ffi_runtime_metadata",
        "fluent_create_key_pair_parity",
        "fluent_dx_api",
        "fluent_dx_journeys",
        "fluent_set_attribute_parity",
        "fluent_locate_revoke_parity",
        "fluent_named_tag_values",
        "generic_ttlv_model",
        "ttlv_wire_codec",
        "vendor_extension_registry",
        "discover_versions_request_item",
        "usage_allocation_guard",
        "opaque_register_context",
        "server_generated_creation",
        "typed_message_model",
        "typed_async_response_readers",
        "credential_ttlv_parity",
        "credential_type_parity",
        "credential_validation_error_parity",
        "credential_secret_parity",
        "nonce_ttlv_parity",
        "hashed_password_timestamp_sequence",
        "public_binding_surface",
    ];

    for capability in ready_capabilities {
        assert_eq!(
            kmipkit::capability_is_available(capability),
            Ok(true),
            "manifest-ready capability {capability} must be available"
        );
    }
    assert_eq!(
        kmipkit::capability_is_available("unknown_capability"),
        Ok(false)
    );
}

#[test]
fn capability_query_reports_the_opt_in_kmip30_draft_routes() {
    assert_eq!(
        kmipkit::capability_is_available("kmip30_draft_routes"),
        Ok(true)
    );
    assert_eq!(
        kmipkit::capability_is_available("kmip30_credential_type_values"),
        Ok(true)
    );
}

#[test]
fn capability_query_reports_protocol_version_selection() {
    assert_eq!(
        kmipkit::capability_is_available("kmip_protocol_version_selection"),
        Ok(true)
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
