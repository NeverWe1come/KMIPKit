#[test]
fn public_version_matches_the_synchronized_package_version() {
    assert_eq!(kmipkit::VERSION, env!("CARGO_PKG_VERSION"));
    assert_eq!(kmipkit::VERSION, "1.0.0");
}

#[test]
fn capability_query_matches_manifest_ready_capabilities() {
    assert!(kmipkit::capability_is_available("ffi_runtime_metadata"));
    assert!(kmipkit::capability_is_available("generic_ttlv_model"));
    assert!(kmipkit::capability_is_available("ttlv_wire_codec"));
    assert!(!kmipkit::capability_is_available("public_binding_surface"));
    assert!(!kmipkit::capability_is_available("unknown_capability"));
}
