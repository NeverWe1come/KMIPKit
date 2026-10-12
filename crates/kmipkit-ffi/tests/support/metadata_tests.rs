use super::*;

const ERROR_NONE: u32 = 0;
const ERROR_CATEGORY_VALIDATION: u32 = 1;
const ERROR_CAUSE_INVALID_INPUT: u32 = 1;
const DELIVERY_NONE: u32 = 0;

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
            "ffi_runtime_metadata",
            "generic_ttlv_model",
            "ttlv_wire_codec",
            "vendor_extension_registry",
        ]
    );
    for capability in crate::public_api_generated::C_AVAILABLE_CAPABILITIES {
        assert!(
            matches!(kmipkit::capability_is_available(capability), Ok(true)),
            "Rust facade omitted ready manifest capability {capability}"
        );
    }
    for unavailable in [
        "public_binding_surface",
        "rust_transport_exchange",
    ] {
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
