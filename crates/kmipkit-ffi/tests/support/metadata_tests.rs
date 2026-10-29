use super::*;
use std::collections::BTreeSet;

const ERROR_NONE: u32 = 0;
const ERROR_CATEGORY_VALIDATION: u32 = 1;
const ERROR_CAUSE_INVALID_INPUT: u32 = 1;
const DELIVERY_NONE: u32 = 0;

fn starts_inside_shared_panic_boundary(body: &str) -> bool {
    body.trim_start().starts_with("ffi_entrypoint(||")
}

fn macro_generated_c_exports(source: &str, macro_name: &str) -> Vec<(String, bool)> {
    let definition_marker = format!("macro_rules! {macro_name}");
    let invocation_marker = format!("{macro_name}!(");
    let macro_body = source
        .split_once(&definition_marker)
        .map_or("", |(_, definition)| {
            definition.split("};").next().unwrap_or("")
        });
    let starts_inside_panic_boundary = macro_body
        .split_once("pub extern \"C\" fn $name")
        .and_then(|(_, signature)| signature.split_once("-> i32 {"))
        .is_some_and(|(_, body)| {
            macro_body.contains("#[unsafe(no_mangle)]") && starts_inside_shared_panic_boundary(body)
        });

    source
        .split(&invocation_marker)
        .skip(1)
        .filter_map(|invocation| {
            let symbol = invocation
                .trim_start()
                .split(|character: char| character.is_whitespace() || character == ',')
                .next()?;
            (!symbol.is_empty()).then(|| (symbol.to_owned(), starts_inside_panic_boundary))
        })
        .collect()
}

#[test]
fn version_query_matches_the_package_version_and_stable_abi() {
    let mut major = u32::MAX;
    let mut minor = u32::MAX;
    let mut patch = u32::MAX;
    let mut abi = u32::MAX;

    assert_eq!(
        kmipkit_version_get(&raw mut major, &raw mut minor, &raw mut patch, &raw mut abi,),
        SUCCESS
    );
    let package_version = env!("CARGO_PKG_VERSION");
    let core = package_version.split(['-', '+']).next().unwrap_or("");
    let components: Vec<u32> = core
        .split('.')
        .map(|component| component.parse().expect("Cargo package version is numeric"))
        .collect();
    assert_eq!(
        [major, minor, patch],
        [components[0], components[1], components[2]]
    );
    assert_eq!(abi, kmipkit::ABI_VERSION);
    assert_eq!(abi, 1);
}

#[test]
fn capability_query_only_reports_c_mapped_nonblocked_capabilities() {
    assert_eq!(
        crate::public_api_generated::C_AVAILABLE_CAPABILITIES,
        &[
            "api_level_discovery",
            "client_asynchronous_operations",
            "client_batch_model",
            "client_execution",
            "crypto_response_ttlv",
            "discover_versions_request_item",
            "ffi_runtime_metadata",
            "generic_ttlv_model",
            "opaque_register_context",
            "production_transport",
            "server_generated_creation",
            "ttlv_wire_codec",
            "usage_allocation_guard",
            "vendor_extension_registry",
        ]
    );
    for capability in crate::public_api_generated::C_AVAILABLE_CAPABILITIES {
        assert!(
            matches!(kmipkit::capability_is_available(capability), Ok(true)),
            "Rust facade omitted ready manifest capability {capability}"
        );
    }
    for unavailable in ["public_binding_surface", "rust_transport_exchange"] {
        assert!(
            matches!(kmipkit::capability_is_available(unavailable), Ok(false)),
            "Rust facade advertised a capability outside its mapped surface: {unavailable}"
        );
    }

    let mut available = u8::MAX;
    let generic_ttlv = b"generic_ttlv_model";
    assert_eq!(
        kmipkit_capability_is_available(
            generic_ttlv.as_ptr(),
            generic_ttlv.len() as u64,
            &raw mut available,
        ),
        SUCCESS
    );
    assert_eq!(available, 1);

    let blocked = b"public_binding_surface";
    assert_eq!(
        kmipkit_capability_is_available(blocked.as_ptr(), blocked.len() as u64, &raw mut available,),
        SUCCESS
    );
    assert_eq!(available, 0);

    let unknown = b"not-a-manifest-capability";
    assert_eq!(
        kmipkit_capability_is_available(unknown.as_ptr(), unknown.len() as u64, &raw mut available,),
        SUCCESS
    );
    assert_eq!(available, 0);
}

#[test]
fn oversized_capability_query_records_resource_limit_without_request_delivery() {
    let identifier = [b'x'; 129];
    let mut available = u8::MAX;

    assert_eq!(
        kmipkit_capability_is_available(
            identifier.as_ptr(),
            identifier.len() as u64,
            &raw mut available,
        ),
        ERROR_RESOURCE_LIMIT
    );
    assert_eq!(
        read_last_error(),
        (
            ERROR_RESOURCE_LIMIT,
            ERROR_CATEGORY_VALIDATION,
            ERROR_CAUSE_OTHER,
            DELIVERY_STATE_NONE,
        )
    );
    assert_eq!(available, u8::MAX);
}

#[test]
fn capability_query_rejects_invalid_utf8_without_changing_the_output() {
    let identifier = [u8::MAX];
    let mut available = u8::MAX;

    assert_eq!(
        kmipkit_capability_is_available(
            identifier.as_ptr(),
            identifier.len() as u64,
            &raw mut available,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        read_last_error(),
        (
            ERROR_INVALID_INPUT,
            ERROR_CATEGORY_VALIDATION,
            ERROR_CAUSE_INVALID_INPUT,
            DELIVERY_STATE_NONE,
        )
    );
    assert_eq!(available, u8::MAX);
}

#[test]
fn last_error_is_redacted_thread_local_and_persists_until_explicitly_cleared() {
    kmipkit_last_error_clear();
    assert_eq!(
        read_last_error(),
        (SUCCESS, ERROR_NONE, ERROR_NONE, DELIVERY_NONE)
    );

    assert_eq!(
        kmipkit_ttlv_structure_create(std::ptr::null_mut()),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        read_last_error(),
        (
            ERROR_INVALID_INPUT,
            ERROR_CATEGORY_VALIDATION,
            ERROR_CAUSE_INVALID_INPUT,
            DELIVERY_NONE
        )
    );

    let child = std::thread::spawn(read_last_error);
    assert_eq!(
        child.join().expect("the thread-local query does not panic"),
        (SUCCESS, ERROR_NONE, ERROR_NONE, DELIVERY_NONE)
    );

    let mut structure = std::ptr::null_mut();
    assert_eq!(kmipkit_ttlv_structure_create(&raw mut structure), SUCCESS);
    kmipkit_ttlv_structure_release(structure);
    assert_eq!(
        read_last_error(),
        (
            ERROR_INVALID_INPUT,
            ERROR_CATEGORY_VALIDATION,
            ERROR_CAUSE_INVALID_INPUT,
            DELIVERY_NONE
        )
    );

    kmipkit_last_error_clear();
    assert_eq!(
        read_last_error(),
        (SUCCESS, ERROR_NONE, ERROR_NONE, DELIVERY_NONE)
    );
}

fn read_last_error() -> (i32, u32, u32, u32) {
    let mut status = i32::MAX;
    let mut category = u32::MAX;
    let mut cause = u32::MAX;
    let mut delivery = u32::MAX;
    assert_eq!(
        kmipkit_last_error_get(
            &raw mut status,
            &raw mut category,
            &raw mut cause,
            &raw mut delivery,
        ),
        SUCCESS
    );
    (status, category, cause, delivery)
}

#[test]
fn last_error_get_rejects_null_outputs_without_overwriting_the_snapshot() {
    kmipkit_last_error_clear();
    let mut available = 0;
    assert_eq!(
        kmipkit_capability_is_available(
            std::ptr::null(),
            MAX_CAPABILITY_ID_BYTES + 1,
            &raw mut available,
        ),
        ERROR_RESOURCE_LIMIT
    );
    assert_eq!(
        kmipkit_last_error_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ),
        ERROR_INVALID_INPUT
    );
    let mut status = i32::MAX;
    let mut category = u32::MAX;
    let mut cause = u32::MAX;
    let mut delivery = u32::MAX;
    assert_eq!(
        kmipkit_last_error_get(
            &raw mut status,
            &raw mut category,
            &raw mut cause,
            &raw mut delivery,
        ),
        SUCCESS
    );
    assert_eq!(
        (status, category, cause, delivery),
        (
            ERROR_RESOURCE_LIMIT,
            ERROR_CATEGORY_VALIDATION,
            ERROR_CAUSE_OTHER,
            DELIVERY_NONE
        )
    );
}

#[test]
fn versioned_limits_validator_records_its_error_in_the_shared_snapshot() {
    kmipkit_last_error_clear();

    assert_eq!(
        kmipkit_extension_registry_limits_v1_validate(std::ptr::null()),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        read_last_error(),
        (
            ERROR_INVALID_INPUT,
            ERROR_CATEGORY_VALIDATION,
            ERROR_CAUSE_INVALID_INPUT,
            DELIVERY_NONE
        )
    );
}

#[test]
fn legacy_limits_validator_records_its_error_in_the_shared_snapshot() {
    kmipkit_last_error_clear();

    assert_eq!(
        kmipkit_extension_registry_limits_validate(
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ),
        ERROR_RESOURCE_LIMIT
    );
    assert_eq!(
        read_last_error(),
        (
            ERROR_RESOURCE_LIMIT,
            ERROR_CATEGORY_VALIDATION,
            ERROR_CAUSE_OTHER,
            DELIVERY_NONE
        )
    );
}

#[test]
fn panic_boundary_records_internal_error_for_void_exports() {
    kmipkit_last_error_clear();

    ffi_entrypoint(|| -> () { std::panic::panic_any(()) });

    assert_eq!(
        read_last_error(),
        (
            ERROR_INTERNAL,
            ERROR_CATEGORY_OTHER,
            ERROR_CAUSE_OTHER,
            DELIVERY_NONE
        )
    );
}

#[test]
fn every_exported_c_function_starts_inside_the_shared_panic_boundary() {
    const BLOCKED_C_SYMBOLS: [&str; 1] = ["kmipkit_validated_extension_value_value_at"];
    const INTERNAL_C_SYMBOLS: [&str; 6] = [
        "kmipkit_client_batch_item_response_metadata_v1",
        "kmipkit_client_batch_item_response_release",
        "kmipkit_client_execute_with_options_and_error_ttlv_v1",
        "kmipkit_last_error_clear",
        "kmipkit_transport_config_create_v1",
        "kmipkit_transport_config_release",
    ];
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../specification/api/public-api.json"
    ))
    .expect("the approved public API manifest is valid JSON");
    let declarations = manifest["publicDeclarations"]
        .as_array()
        .expect("the approved manifest lists public declarations");
    let manifest_symbols: BTreeSet<&str> = declarations
        .iter()
        .filter(|declaration| {
            declaration["packageId"] == "c-kmipkit"
                && declaration["kind"] == "c-symbol"
                && declaration["declarationState"] != "blocked"
        })
        .filter_map(|declaration| declaration["exportName"].as_str())
        .collect();
    let blocked_symbols: BTreeSet<&str> = declarations
        .iter()
        .filter(|declaration| {
            declaration["packageId"] == "c-kmipkit"
                && declaration["kind"] == "c-symbol"
                && declaration["declarationState"] == "blocked"
        })
        .filter_map(|declaration| declaration["exportName"].as_str())
        .collect();
    assert_eq!(
        blocked_symbols,
        BLOCKED_C_SYMBOLS.into_iter().collect(),
        "review blocked C symbols when their manifest state changes"
    );

    let sources = [
        include_str!("../../src/extension_registry.rs"),
        include_str!("../../src/extension_registry/client_abi.rs"),
    ];
    let mut exports: Vec<(String, bool)> = sources
        .into_iter()
        .flat_map(|source| source.split("#[unsafe(no_mangle)]"))
        .filter_map(|declaration| {
            let function_start = declaration
                .find("pub unsafe extern \"C\" fn")
                .or_else(|| declaration.find("pub extern \"C\" fn"))?;
            let function = &declaration[function_start..];
            let symbol = function
                .split("extern \"C\" fn ")
                .nth(1)?
                .split('(')
                .next()?
                .to_owned();
            let body_start = function.find('{')?;
            Some((
                symbol,
                starts_inside_shared_panic_boundary(&function[body_start + 1..]),
            ))
        })
        .filter(|(symbol, _)| symbol != "$name")
        .collect();
    exports.extend(macro_generated_c_exports(
        sources[0],
        "lifecycle_batch_item_constructor",
    ));
    exports.extend(macro_generated_c_exports(
        sources[1],
        "lifecycle_response_identifier",
    ));

    let source_symbols: BTreeSet<&str> =
        exports.iter().map(|(symbol, _)| symbol.as_str()).collect();
    let expected_symbols: BTreeSet<&str> = manifest_symbols
        .into_iter()
        .chain(blocked_symbols)
        .chain(INTERNAL_C_SYMBOLS)
        .collect();
    assert_eq!(
        source_symbols, expected_symbols,
        "Rust C exports must match approved manifest symbols and the explicit blocked/internal lists"
    );
    assert_eq!(
        exports.len(),
        source_symbols.len(),
        "duplicate C export symbol"
    );
    assert!(
        exports.iter().all(|(_, uses_boundary)| *uses_boundary),
        "every exported C function must enter the shared panic boundary first"
    );
}

#[test]
fn panic_boundary_returns_a_redacted_internal_error_snapshot() {
    kmipkit_last_error_clear();

    let status = ffi_entrypoint(|| -> i32 { panic!("synthetic panic payload") });

    assert_eq!(status, ERROR_INTERNAL);
    assert_eq!(
        read_last_error(),
        (
            ERROR_INTERNAL,
            ERROR_CATEGORY_OTHER,
            ERROR_CAUSE_OTHER,
            DELIVERY_NONE
        )
    );
}
