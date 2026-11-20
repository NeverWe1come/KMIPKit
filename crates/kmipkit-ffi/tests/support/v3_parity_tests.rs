use super::*;
use crate::{
    KMIPKIT_UNIQUE_IDENTIFIER_KIND_IDENTIFIER, KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
    kmipkit_s2c_capabilities_v1_t, kmipkit_s2c_operation_result_v1_t,
    kmipkit_server_request_handler_v1_t, kmipkit_server_request_handler_v2_t,
    kmipkit_server_request_handler_v3_t,
};
use std::ptr;

fn abi_u32(value: usize) -> u32 {
    u32::try_from(value).expect("test ABI values fit in u32")
}

#[test]
fn test_unique_identifier_v1_identifier_roundtrip() {
    let text = b"test-identifier-uid-42";
    let input = kmipkit_unique_identifier_v1_t {
        struct_size: abi_u32(std::mem::size_of::<kmipkit_unique_identifier_v1_t>()),
        abi_version: 1,
        kind: KMIPKIT_UNIQUE_IDENTIFIER_KIND_IDENTIFIER,
        reserved: 0,
        text_data: text.as_ptr(),
        text_length: text.len() as u64,
        enumeration_value: 0,
        integer_value: 0,
    };

    let parsed = copy_unique_identifier(&raw const input)
        .expect("valid identifier")
        .expect("some identifier");
    assert!(
        matches!(parsed, kmipkit_protocol::UniqueIdentifier::Identifier(ref s) if s == "test-identifier-uid-42")
    );

    let mut output: kmipkit_unique_identifier_v1_t = unsafe { std::mem::zeroed() };
    output.struct_size = abi_u32(std::mem::size_of::<kmipkit_unique_identifier_v1_t>());
    output.abi_version = 1;
    client_abi::write_unique_identifier_v1(&raw mut output, Some(&parsed)).expect("write succeeds");
    assert_eq!(output.kind, KMIPKIT_UNIQUE_IDENTIFIER_KIND_IDENTIFIER);
    assert_eq!(output.text_length, text.len() as u64);
    let output_slice = unsafe {
        std::slice::from_raw_parts(
            output.text_data,
            usize::try_from(output.text_length).expect("test ABI length fits in usize"),
        )
    };
    assert_eq!(output_slice, text);
}

#[test]
fn test_ttlv_v3_primitives_constructors_and_views() {
    let mut limits_handle = ptr::null_mut();
    assert_eq!(
        kmipkit_codec_limits_defaults(&raw mut limits_handle),
        SUCCESS
    );
    assert!(!limits_handle.is_null());

    let payload = b"ref-uid-100";
    let mut val_id = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_value_identifier(
            limits_handle,
            payload.as_ptr(),
            payload.len() as u64,
            &raw mut val_id
        ),
        SUCCESS
    );
    assert!(!val_id.is_null());

    let mut val_ref = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_value_reference(
            limits_handle,
            payload.as_ptr(),
            payload.len() as u64,
            &raw mut val_ref
        ),
        SUCCESS
    );
    assert!(!val_ref.is_null());

    let mut val_name_ref = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_value_name_reference(
            limits_handle,
            payload.as_ptr(),
            payload.len() as u64,
            &raw mut val_name_ref
        ),
        SUCCESS
    );
    assert!(!val_name_ref.is_null());

    kmipkit_ttlv_value_release(val_id);
    kmipkit_ttlv_value_release(val_ref);
    kmipkit_ttlv_value_release(val_name_ref);
    kmipkit_codec_limits_release(limits_handle);
}

#[test]
fn test_version_gate_ffi() {
    // KMIP 3.0 operation in 3.0 vs 2.1
    assert_eq!(kmipkit_version_gate_validate_operation(3, 0, 60), SUCCESS);
    assert_eq!(
        kmipkit_version_gate_validate_operation(2, 1, 60),
        ERROR_INVALID_INPUT
    );

    // KMIP 3.0 tag in 3.0 vs 2.1
    assert_eq!(kmipkit_version_gate_validate_tag(3, 0, 0x42_01BF), SUCCESS);
    assert_eq!(
        kmipkit_version_gate_validate_tag(2, 1, 0x42_01BF),
        ERROR_INVALID_INPUT
    );
}

#[test]
fn test_nist_key_lifecycle_ffi() {
    let mut can_orig = 0u8;
    let mut can_proc = 0u8;

    // Active
    assert_eq!(
        client_abi::kmipkit_nist_key_lifecycle_can_originate(2, &raw mut can_orig),
        SUCCESS
    );
    assert_eq!(
        client_abi::kmipkit_nist_key_lifecycle_can_process(2, &raw mut can_proc),
        SUCCESS
    );
    assert_eq!(can_orig, 1);
    assert_eq!(can_proc, 1);

    // Deactivated
    assert_eq!(
        client_abi::kmipkit_nist_key_lifecycle_can_originate(3, &raw mut can_orig),
        SUCCESS
    );
    assert_eq!(
        client_abi::kmipkit_nist_key_lifecycle_can_process(3, &raw mut can_proc),
        SUCCESS
    );
    assert_eq!(can_orig, 0);
    assert_eq!(can_proc, 1);

    // Destroyed
    assert_eq!(
        client_abi::kmipkit_nist_key_lifecycle_can_originate(5, &raw mut can_orig),
        SUCCESS
    );
    assert_eq!(
        client_abi::kmipkit_nist_key_lifecycle_can_process(5, &raw mut can_proc),
        SUCCESS
    );
    assert_eq!(can_orig, 0);
    assert_eq!(can_proc, 0);

    // Invalid enum
    assert_eq!(
        client_abi::kmipkit_nist_key_lifecycle_can_originate(999, &raw mut can_orig),
        ERROR_INVALID_INPUT
    );
}

#[test]
fn test_lease_context_ffi() {
    let uid_text = b"lease-key-test";
    let uid = kmipkit_unique_identifier_v1_t {
        struct_size: abi_u32(std::mem::size_of::<kmipkit_unique_identifier_v1_t>()),
        abi_version: 1,
        kind: KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: uid_text.as_ptr(),
        text_length: uid_text.len() as u64,
        enumeration_value: 0,
        integer_value: 0,
    };

    let mut lease = ptr::null_mut();
    assert_eq!(
        kmipkit_lease_context_create(&raw const uid, 100, 50, &raw mut lease),
        SUCCESS
    );
    assert!(!lease.is_null());

    let mut exp = 0i64;
    assert_eq!(
        kmipkit_lease_context_expires_at(lease, &raw mut exp),
        SUCCESS
    );
    assert_eq!(exp, 150);

    let mut is_exp = 1u8;
    assert_eq!(
        kmipkit_lease_context_is_expired(lease, 140, &raw mut is_exp),
        SUCCESS
    );
    assert_eq!(is_exp, 0);
    assert_eq!(kmipkit_lease_context_check_valid(lease, 140), SUCCESS);

    assert_eq!(
        kmipkit_lease_context_is_expired(lease, 151, &raw mut is_exp),
        SUCCESS
    );
    assert_eq!(is_exp, 1);
    assert_eq!(
        kmipkit_lease_context_check_valid(lease, 151),
        ERROR_INVALID_INPUT
    );

    let mut out_uid: kmipkit_unique_identifier_v1_t = unsafe { std::mem::zeroed() };
    out_uid.struct_size = abi_u32(std::mem::size_of::<kmipkit_unique_identifier_v1_t>());
    out_uid.abi_version = 1;
    assert_eq!(
        kmipkit_lease_context_unique_identifier(lease, &raw mut out_uid),
        SUCCESS
    );
    assert_eq!(out_uid.kind, KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING);
    assert_eq!(out_uid.text_length, uid_text.len() as u64);

    kmipkit_lease_context_release(lease);
}

#[test]
fn test_cryptographic_parameters_v2_ffi() {
    let mut fields: kmipkit_cryptographic_parameters_fields_v2_t = unsafe { std::mem::zeroed() };
    fields.struct_size = abi_u32(std::mem::size_of::<
        kmipkit_cryptographic_parameters_fields_v2_t,
    >());
    fields.abi_version = 2;
    fields.kem_algorithm_present = 1;
    fields.kem_algorithm = 2; // DHKEM
    fields.deterministic_present = 1;
    fields.deterministic = 1;

    let mut params = ptr::null_mut();
    assert_eq!(
        unsafe {
            cryptographic_parameters::kmipkit_cryptographic_parameters_from_fields_v2(
                &raw const fields,
                &raw mut params,
            )
        },
        SUCCESS
    );
    assert!(!params.is_null());

    let mut out_fields: kmipkit_cryptographic_parameters_fields_v2_t =
        unsafe { std::mem::zeroed() };
    out_fields.struct_size = abi_u32(std::mem::size_of::<
        kmipkit_cryptographic_parameters_fields_v2_t,
    >());
    out_fields.abi_version = 2;
    assert_eq!(
        unsafe {
            cryptographic_parameters::kmipkit_cryptographic_parameters_read_fields_v2(
                params,
                &raw mut out_fields,
            )
        },
        SUCCESS
    );
    assert_eq!(out_fields.kem_algorithm_present, 1);
    assert_eq!(out_fields.kem_algorithm, 2);
    assert_eq!(out_fields.deterministic_present, 1);
    assert_eq!(out_fields.deterministic, 1);

    cryptographic_parameters::kmipkit_cryptographic_parameters_release(params);
}

#[test]
fn test_s2c_response_builder_ffi() {
    let mut limits = ptr::null_mut();
    assert_eq!(kmipkit_codec_limits_defaults(&raw mut limits), SUCCESS);

    // Build a batch item structure with Operation, ResultStatus, ResponsePayload
    let mut batch_item = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_structure_create(&raw mut batch_item), SUCCESS);

    let mut raw_tag = ptr::null_mut();
    let mut tag = ptr::null_mut();
    let mut val = ptr::null_mut();
    let mut item = ptr::null_mut();
    let mut s1 = ptr::null_mut();

    // 1. Operation = 1
    assert_eq!(
        kmipkit_ttlv_raw_tag_create(0x42_005C, &raw mut raw_tag),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &raw mut tag),
        SUCCESS
    );
    kmipkit_raw_tag_release(raw_tag);
    assert_eq!(kmipkit_ttlv_value_enumeration(1, &raw mut val), SUCCESS);
    assert_eq!(
        kmipkit_ttlv_item_create(tag, val, limits, &raw mut item),
        SUCCESS
    );
    kmipkit_tag_release(tag);
    assert_eq!(
        kmipkit_ttlv_structure_with_item(batch_item, item, limits, &raw mut s1),
        SUCCESS
    );
    kmipkit_ttlv_structure_release(batch_item);
    kmipkit_ttlv_item_release(item);

    // 2. ResultStatus = 0
    assert_eq!(
        kmipkit_ttlv_raw_tag_create(0x42_007F, &raw mut raw_tag),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &raw mut tag),
        SUCCESS
    );
    kmipkit_raw_tag_release(raw_tag);
    assert_eq!(kmipkit_ttlv_value_enumeration(0, &raw mut val), SUCCESS);
    assert_eq!(
        kmipkit_ttlv_item_create(tag, val, limits, &raw mut item),
        SUCCESS
    );
    kmipkit_tag_release(tag);
    let mut s2 = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_with_item(s1, item, limits, &raw mut s2),
        SUCCESS
    );
    kmipkit_ttlv_structure_release(s1);
    kmipkit_ttlv_item_release(item);

    // 3. ResponsePayload = empty structure
    assert_eq!(
        kmipkit_ttlv_raw_tag_create(0x42_007C, &raw mut raw_tag),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &raw mut tag),
        SUCCESS
    );
    kmipkit_raw_tag_release(raw_tag);
    let mut empty_payload = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_create(&raw mut empty_payload),
        SUCCESS
    );
    assert_eq!(
        kmipkit_ttlv_value_structure(empty_payload, limits, &raw mut val),
        SUCCESS
    );
    kmipkit_ttlv_structure_release(empty_payload);
    assert_eq!(
        kmipkit_ttlv_item_create(tag, val, limits, &raw mut item),
        SUCCESS
    );
    kmipkit_tag_release(tag);
    let mut s3 = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_with_item(s2, item, limits, &raw mut s3),
        SUCCESS
    );
    kmipkit_ttlv_structure_release(s2);
    kmipkit_ttlv_item_release(item);

    let items = [s3];
    let mut resp = ptr::null_mut();
    assert_eq!(
        kmipkit_s2c_build_response_ttlv(2, 1, items.as_ptr(), items.len() as u64, &raw mut resp,),
        SUCCESS
    );
    assert!(!resp.is_null());
    kmipkit_ttlv_structure_release(resp);
    kmipkit_ttlv_structure_release(s3);
    kmipkit_codec_limits_release(limits);
}

#[test]
fn test_set_endpoint_role_item_ffi() {
    let mut item = ptr::null_mut();
    let vendor = b"KMIPKit-Vendor";
    assert_eq!(
        client_abi::kmipkit_client_batch_item_set_endpoint_role(
            1, // Client
            vendor.as_ptr(),
            vendor.len() as u64,
            &raw mut item,
        ),
        SUCCESS
    );
    assert!(!item.is_null());
    kmipkit_client_batch_item_release(item);
}

#[test]
fn test_s2c_listener_dispatcher_and_helpers_ffi() {
    let mut intro_maj = 0;
    let mut intro_min = 0;
    let mut depr_maj = 0;
    let mut depr_min = 0;
    let mut rem_maj = 0;
    let mut rem_min = 0;
    assert_eq!(
        client_abi::kmipkit_version_gate_operation_span(
            0x0000_001E,
            &raw mut intro_maj,
            &raw mut intro_min,
            &raw mut depr_maj,
            &raw mut depr_min,
            &raw mut rem_maj,
            &raw mut rem_min,
        ),
        SUCCESS
    );
    assert_eq!(intro_maj, 2);
    assert_eq!(intro_min, 0);

    let supported = [((2u32) << 16) | 1, ((2u32) << 16)];
    let mut neg_maj = 0;
    let mut neg_min = 0;
    assert_eq!(
        client_abi::kmipkit_s2c_discover_versions_negotiate(
            2,
            1,
            supported.as_ptr(),
            abi_u32(supported.len()),
            &raw mut neg_maj,
            &raw mut neg_min,
        ),
        SUCCESS
    );
    assert_eq!(neg_maj, 2);
    assert_eq!(neg_min, 1);

    let mut eff = 0;
    assert_eq!(
        client_abi::kmipkit_s2c_put_effective_function(2, 0, &raw mut eff,),
        SUCCESS
    );
    assert_eq!(eff, 1);

    assert_eq!(
        client_abi::kmipkit_s2c_put_effective_function(2, 1, &raw mut eff,),
        SUCCESS
    );
    assert_eq!(eff, 2);

    let mut query_ttlv = ptr::null_mut();
    let vendor = b"TestVendor";
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_ttlv(
            0x01 | 0x02 | 0x04,
            vendor.as_ptr(),
            vendor.len() as u64,
            &raw mut query_ttlv,
        ),
        SUCCESS
    );
    assert!(!query_ttlv.is_null());
    kmipkit_ttlv_structure_release(query_ttlv);

    let mut dispatcher = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_create(&raw mut dispatcher),
        SUCCESS
    );
    assert!(!dispatcher.is_null());
    client_abi::kmipkit_server_request_dispatcher_release(dispatcher);
}

fn ttlv_test_item(tag: u32, item_type: u8, value: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + value.len().div_ceil(8) * 8);
    encoded.extend_from_slice(&tag.to_be_bytes()[1..]);
    encoded.push(item_type);
    encoded.extend_from_slice(
        &u32::try_from(value.len())
            .expect("test TTLV item length fits")
            .to_be_bytes(),
    );
    encoded.extend_from_slice(value);
    encoded.resize(8 + value.len().div_ceil(8) * 8, 0);
    encoded
}

static PUT_CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static QUERY_CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static PUT_V1_CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static QUERY_V1_CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static PUT_V3_CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static QUERY_V3_CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

static FAIL_MSG: &[u8] = b"Custom Failure From C Handler";

unsafe extern "C" fn test_put_callback_v3(
    _user_data: *mut core::ffi::c_void,
    _item_view: *const kmipkit_server_request_item_view_v1_t,
    put_function: u32,
    _unique_identifier_data: *const u8,
    _unique_identifier_length: u64,
    _replaced_unique_identifier_data: *const u8,
    _replaced_unique_identifier_length: u64,
    _object_ttlv_data: *const u8,
    _object_ttlv_length: u64,
    _attributes_ttlv_data: *const u8,
    _attributes_ttlv_length: u64,
    out_result: *mut kmipkit_s2c_operation_result_v1_t,
) -> u32 {
    assert_eq!(put_function, 2);
    assert!(!out_result.is_null());
    PUT_V3_CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
    0
}

unsafe extern "C" fn test_put_callback_v3_fail(
    _user_data: *mut core::ffi::c_void,
    _item_view: *const kmipkit_server_request_item_view_v1_t,
    _put_function: u32,
    _unique_identifier_data: *const u8,
    _unique_identifier_length: u64,
    _replaced_unique_identifier_data: *const u8,
    _replaced_unique_identifier_length: u64,
    _object_ttlv_data: *const u8,
    _object_ttlv_length: u64,
    _attributes_ttlv_data: *const u8,
    _attributes_ttlv_length: u64,
    out_result: *mut kmipkit_s2c_operation_result_v1_t,
) -> u32 {
    assert!(!out_result.is_null());
    let res = unsafe { &mut *out_result };
    res.result_status = 1; // OperationFailed
    res.has_result_reason = 1;
    res.result_reason = 0x01; // ItemNotFound
    res.result_message_data = FAIL_MSG.as_ptr();
    res.result_message_length = FAIL_MSG.len() as u64;
    1
}

unsafe extern "C" fn test_query_callback_v3(
    _user_data: *mut core::ffi::c_void,
    _item_view: *const kmipkit_server_request_item_view_v1_t,
    query_function_flags: u32,
    out_capabilities: *mut kmipkit_s2c_capabilities_v1_t,
    out_result: *mut kmipkit_s2c_operation_result_v1_t,
) -> u32 {
    assert!(query_function_flags & 0x01 != 0);
    assert!(!out_capabilities.is_null());
    assert!(!out_result.is_null());
    let caps = unsafe { &mut *out_capabilities };
    caps.operations = CUSTOM_QUERY_OPS.as_ptr();
    caps.operations_count = abi_u32(CUSTOM_QUERY_OPS.len());
    caps.object_types = CUSTOM_QUERY_OTS.as_ptr();
    caps.object_types_count = abi_u32(CUSTOM_QUERY_OTS.len());
    caps.vendor_identification_data = CUSTOM_QUERY_VENDOR.as_ptr();
    caps.vendor_identification_length = CUSTOM_QUERY_VENDOR.len() as u64;

    QUERY_V3_CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
    0
}

unsafe extern "C" fn test_put_callback(
    _user_data: *mut core::ffi::c_void,
    _item_view: *const kmipkit_server_request_item_view_v1_t,
    put_function: u32,
    unique_identifier_data: *const u8,
    unique_identifier_length: u64,
    replaced_unique_identifier_data: *const u8,
    replaced_unique_identifier_length: u64,
    object_ttlv_data: *const u8,
    object_ttlv_length: u64,
    attributes_ttlv_data: *const u8,
    attributes_ttlv_length: u64,
) -> u32 {
    assert_eq!(put_function, 2); // Replace
    let uid = unsafe {
        std::slice::from_raw_parts(
            unique_identifier_data,
            usize::try_from(unique_identifier_length).expect("test ABI length fits in usize"),
        )
    };
    assert_eq!(uid, b"sym-key-uid");
    let rep_uid = unsafe {
        std::slice::from_raw_parts(
            replaced_unique_identifier_data,
            usize::try_from(replaced_unique_identifier_length)
                .expect("test ABI length fits in usize"),
        )
    };
    assert_eq!(rep_uid, b"replaced-sym-uid");
    assert!(!object_ttlv_data.is_null());
    assert!(object_ttlv_length > 0);
    assert!(!attributes_ttlv_data.is_null());
    assert!(attributes_ttlv_length > 0);

    PUT_CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
    0
}

unsafe extern "C" fn test_put_callback_v1(
    _user_data: *mut core::ffi::c_void,
    _item_view: *const kmipkit_server_request_item_view_v1_t,
    unique_identifier_data: *const u8,
    unique_identifier_length: u64,
    object_ttlv_data: *const u8,
    object_ttlv_length: u64,
) -> u32 {
    let uid = unsafe {
        std::slice::from_raw_parts(
            unique_identifier_data,
            usize::try_from(unique_identifier_length).expect("test ABI length fits in usize"),
        )
    };
    assert_eq!(uid, b"sym-key-uid");
    assert!(!object_ttlv_data.is_null());
    assert!(object_ttlv_length > 0);

    PUT_V1_CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
    0
}

static CUSTOM_QUERY_OPS: [u32; 3] = [0x0000_001E, 0x0000_001B, 0x0000_001C];
static CUSTOM_QUERY_OTS: [u32; 2] = [1, 2];
static CUSTOM_QUERY_VENDOR: &[u8] = b"CustomVendorFromC";

unsafe extern "C" fn test_query_callback(
    _user_data: *mut core::ffi::c_void,
    _item_view: *const kmipkit_server_request_item_view_v1_t,
    query_function_flags: u32,
    out_capabilities: *mut kmipkit_s2c_capabilities_v1_t,
) -> u32 {
    assert!(query_function_flags & 0x01 != 0); // Operations
    assert!(query_function_flags & 0x04 != 0); // ServerInformation
    assert!(!out_capabilities.is_null());
    let caps = unsafe { &mut *out_capabilities };
    caps.operations = CUSTOM_QUERY_OPS.as_ptr();
    caps.operations_count = abi_u32(CUSTOM_QUERY_OPS.len());
    caps.object_types = CUSTOM_QUERY_OTS.as_ptr();
    caps.object_types_count = abi_u32(CUSTOM_QUERY_OTS.len());
    caps.vendor_identification_data = CUSTOM_QUERY_VENDOR.as_ptr();
    caps.vendor_identification_length = CUSTOM_QUERY_VENDOR.len() as u64;

    QUERY_CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
    0
}

unsafe extern "C" fn test_query_callback_v1(
    _user_data: *mut core::ffi::c_void,
    _item_view: *const kmipkit_server_request_item_view_v1_t,
    query_function_flags: u32,
) -> u32 {
    assert!(query_function_flags & 0x01 != 0); // Operations
    assert!(query_function_flags & 0x04 != 0); // ServerInformation

    QUERY_V1_CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
    0
}

#[test]
#[allow(clippy::too_many_lines)] // One scenario verifies the callback ABI and payload round trips together.
fn test_s2c_put_and_query_callbacks_with_real_payloads() {
    // 1. Build Put request payload
    let mut put_payload = Vec::new();
    put_payload.extend_from_slice(&ttlv_test_item(0x0042_0070, 0x05, &2u32.to_be_bytes())); // PutFunction Replace
    put_payload.extend_from_slice(&ttlv_test_item(0x0042_0094, 0x07, b"sym-key-uid"));
    put_payload.extend_from_slice(&ttlv_test_item(0x0042_0076, 0x07, b"replaced-sym-uid"));
    // Managed Object: SymmetricKey (0x0042_008F)
    let sym_obj = ttlv_test_item(0x0042_0057, 0x05, &2u32.to_be_bytes()); // ObjectType: SymmetricKey
    put_payload.extend_from_slice(&ttlv_test_item(0x0042_008F, 0x01, &sym_obj));
    // Attributes (0x0042_0125)
    let attr_inner = ttlv_test_item(0x0042_0030, 0x07, b"TestKey");
    put_payload.extend_from_slice(&ttlv_test_item(0x0042_0125, 0x01, &attr_inner));

    let mut item1 = Vec::new();
    item1.extend_from_slice(&ttlv_test_item(
        0x0042_005C,
        0x05,
        &0x0000_001Cu32.to_be_bytes(),
    )); // Put
    item1.extend_from_slice(&ttlv_test_item(0x0042_0093, 0x08, b"item-1"));
    item1.extend_from_slice(&ttlv_test_item(0x0042_0079, 0x01, &put_payload));
    let batch_item1 = ttlv_test_item(0x0042_000F, 0x01, &item1);

    // 2. Build Query request payload
    let mut query_payload = Vec::new();
    query_payload.extend_from_slice(&ttlv_test_item(0x0042_0074, 0x05, &1u32.to_be_bytes())); // Operations
    query_payload.extend_from_slice(&ttlv_test_item(0x0042_0074, 0x05, &3u32.to_be_bytes())); // ServerInformation

    let mut item2 = Vec::new();
    item2.extend_from_slice(&ttlv_test_item(
        0x0042_005C,
        0x05,
        &0x0000_0018u32.to_be_bytes(),
    )); // Query
    item2.extend_from_slice(&ttlv_test_item(0x0042_0093, 0x08, b"item-2"));
    item2.extend_from_slice(&ttlv_test_item(0x0042_0079, 0x01, &query_payload));
    let batch_item2 = ttlv_test_item(0x0042_000F, 0x01, &item2);

    // 3. Header & Root
    let mut pv = Vec::new();
    pv.extend_from_slice(&ttlv_test_item(0x0042_006A, 0x02, &2i32.to_be_bytes()));
    pv.extend_from_slice(&ttlv_test_item(0x0042_006B, 0x02, &1i32.to_be_bytes()));
    let mut header = Vec::new();
    header.extend_from_slice(&ttlv_test_item(0x0042_0069, 0x01, &pv));
    header.extend_from_slice(&ttlv_test_item(0x0042_000D, 0x02, &2i32.to_be_bytes()));
    let request_header = ttlv_test_item(0x0042_0077, 0x01, &header);

    let mut root = Vec::new();
    root.extend_from_slice(&request_header);
    root.extend_from_slice(&batch_item1);
    root.extend_from_slice(&batch_item2);
    let request_message_bytes = ttlv_test_item(0x0042_0078, 0x01, &root);

    // Decode batch
    let mut limits = ptr::null_mut();
    assert_eq!(kmipkit_codec_limits_defaults(&raw mut limits), SUCCESS);
    let mut batch = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_incoming_server_batch_decode_ttlv(
            request_message_bytes.as_ptr(),
            request_message_bytes.len() as u64,
            limits,
            &raw mut batch,
        ),
        SUCCESS
    );
    assert!(!batch.is_null());

    // Test item_put_v2 getters directly
    let mut put_fn = 0u32;
    let mut uid_buf = [0u8; 32];
    let mut uid_len = 0u64;
    let mut rep_buf = [0u8; 32];
    let mut rep_len = 0u64;
    let mut obj_item = ptr::null_mut();
    let mut attr_st = ptr::null_mut();

    // SAFETY: `batch` is a live handle and every output points to writable
    // storage matching the supplied buffer capacity.
    assert_eq!(
        unsafe {
            client_abi::kmipkit_incoming_server_batch_item_put_v2(
                batch,
                0,
                &raw mut put_fn,
                uid_buf.as_mut_ptr(),
                uid_buf.len() as u64,
                &raw mut uid_len,
                rep_buf.as_mut_ptr(),
                rep_buf.len() as u64,
                &raw mut rep_len,
                &raw mut obj_item,
                &raw mut attr_st,
            )
        },
        SUCCESS
    );
    assert_eq!(put_fn, 2);
    assert_eq!(
        &uid_buf[..usize::try_from(uid_len).expect("test ABI length fits in usize")],
        b"sym-key-uid"
    );
    assert_eq!(
        &rep_buf[..usize::try_from(rep_len).expect("test ABI length fits in usize")],
        b"replaced-sym-uid"
    );
    assert!(!obj_item.is_null());
    assert!(!attr_st.is_null());
    kmipkit_ttlv_item_release(obj_item);
    kmipkit_ttlv_structure_release(attr_st);

    let mut dispatcher = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_create(&raw mut dispatcher),
        SUCCESS
    );

    // 1. Test dispatch_v1 with kmipkit_server_request_handler_v1_t
    let handler_v1 = kmipkit_server_request_handler_v1_t {
        struct_size: abi_u32(std::mem::size_of::<kmipkit_server_request_handler_v1_t>()),
        abi_version: 1,
        user_data: ptr::null_mut(),
        handle_discover_versions: None,
        handle_notify: None,
        handle_put: Some(test_put_callback_v1),
        handle_query: Some(test_query_callback_v1),
        handle_set_endpoint_role: None,
    };

    let mut response_v1 = ptr::null_mut();
    PUT_V1_CALLED.store(false, std::sync::atomic::Ordering::SeqCst);
    QUERY_V1_CALLED.store(false, std::sync::atomic::Ordering::SeqCst);

    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_dispatch_v1(
            dispatcher,
            batch,
            &raw const handler_v1,
            &raw mut response_v1,
        ),
        SUCCESS
    );
    assert!(PUT_V1_CALLED.load(std::sync::atomic::Ordering::SeqCst));
    assert!(QUERY_V1_CALLED.load(std::sync::atomic::Ordering::SeqCst));
    assert!(!response_v1.is_null());
    kmipkit_response_message_release(response_v1);

    // 2. Test dispatch_v2 with kmipkit_server_request_handler_v2_t
    let handler_v2 = kmipkit_server_request_handler_v2_t {
        struct_size: abi_u32(std::mem::size_of::<kmipkit_server_request_handler_v2_t>()),
        abi_version: 2,
        user_data: ptr::null_mut(),
        handle_discover_versions: None,
        handle_notify: None,
        handle_put: Some(test_put_callback),
        handle_query: Some(test_query_callback),
        handle_set_endpoint_role: None,
    };

    let mut response_v2 = ptr::null_mut();
    PUT_CALLED.store(false, std::sync::atomic::Ordering::SeqCst);
    QUERY_CALLED.store(false, std::sync::atomic::Ordering::SeqCst);

    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_dispatch_v2(
            dispatcher,
            batch,
            &raw const handler_v2,
            &raw mut response_v2,
        ),
        SUCCESS
    );
    assert!(PUT_CALLED.load(std::sync::atomic::Ordering::SeqCst));
    assert!(QUERY_CALLED.load(std::sync::atomic::Ordering::SeqCst));
    assert!(!response_v2.is_null());
    kmipkit_response_message_release(response_v2);

    // 3. Test dispatch_v3 with kmipkit_server_request_handler_v3_t (Success)
    let handler_v3 = kmipkit_server_request_handler_v3_t {
        struct_size: abi_u32(std::mem::size_of::<kmipkit_server_request_handler_v3_t>()),
        abi_version: 3,
        user_data: ptr::null_mut(),
        handle_discover_versions: None,
        handle_notify: None,
        handle_put: Some(test_put_callback_v3),
        handle_query: Some(test_query_callback_v3),
        handle_set_endpoint_role: None,
    };

    let mut response_v3 = ptr::null_mut();
    PUT_V3_CALLED.store(false, std::sync::atomic::Ordering::SeqCst);
    QUERY_V3_CALLED.store(false, std::sync::atomic::Ordering::SeqCst);

    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_dispatch_v3(
            dispatcher,
            batch,
            &raw const handler_v3,
            &raw mut response_v3,
        ),
        SUCCESS
    );
    assert!(PUT_V3_CALLED.load(std::sync::atomic::Ordering::SeqCst));
    assert!(QUERY_V3_CALLED.load(std::sync::atomic::Ordering::SeqCst));
    assert!(!response_v3.is_null());
    kmipkit_response_message_release(response_v3);

    // 4. Test dispatch_v3 with failure handler setting custom out_result (status, reason, message)
    let fail_handler_v3 = kmipkit_server_request_handler_v3_t {
        struct_size: abi_u32(std::mem::size_of::<kmipkit_server_request_handler_v3_t>()),
        abi_version: 3,
        user_data: ptr::null_mut(),
        handle_discover_versions: None,
        handle_notify: None,
        handle_put: Some(test_put_callback_v3_fail),
        handle_query: Some(test_query_callback_v3),
        handle_set_endpoint_role: None,
    };

    let mut fail_resp = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_dispatch_v3(
            dispatcher,
            batch,
            &raw const fail_handler_v3,
            &raw mut fail_resp,
        ),
        SUCCESS
    );
    assert!(!fail_resp.is_null());
    let resp_handle =
        reference_handle(fail_resp, Kind::ResponseMessage).expect("response message handle");
    let HandleValue::ResponseMessage(resp_msg) = &resp_handle.value else {
        panic!("expected ResponseMessage");
    };
    let items: Vec<_> = resp_msg.batch_items().collect();
    assert_eq!(items.len(), 2);
    assert_eq!(
        items[0].result_status(),
        Some(kmipkit_protocol::ResultStatus::from_raw(1))
    );
    assert_eq!(
        items[0].result_reason(),
        Some(kmipkit_protocol::ResultReason::from_raw(0x01))
    );
    items[0].with_result_message(|msg| assert_eq!(msg, "Custom Failure From C Handler"));
    kmipkit_response_message_release(fail_resp);

    // 5. Test ABI validation and version mismatch
    let mut invalid_handler_v1 = handler_v1;
    invalid_handler_v1.abi_version = 2;
    let mut resp = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_dispatch_v1(
            dispatcher,
            batch,
            &raw const invalid_handler_v1,
            &raw mut resp,
        ),
        ERROR_INVALID_INPUT
    );

    let mut invalid_handler_v2 = handler_v2;
    invalid_handler_v2.abi_version = 1;
    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_dispatch_v2(
            dispatcher,
            batch,
            &raw const invalid_handler_v2,
            &raw mut resp,
        ),
        ERROR_INVALID_INPUT
    );

    let mut invalid_handler_v3 = handler_v3;
    invalid_handler_v3.abi_version = 2;
    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_dispatch_v3(
            dispatcher,
            batch,
            &raw const invalid_handler_v3,
            &raw mut resp,
        ),
        ERROR_INVALID_INPUT
    );

    let mut invalid_handler_v3_size = handler_v3;
    invalid_handler_v3_size.struct_size = 0;
    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_dispatch_v3(
            dispatcher,
            batch,
            &raw const invalid_handler_v3_size,
            &raw mut resp,
        ),
        ERROR_INVALID_INPUT
    );

    client_abi::kmipkit_server_request_dispatcher_release(dispatcher);
    client_abi::kmipkit_incoming_server_batch_release(batch);
    kmipkit_codec_limits_release(limits);
}

#[test]
#[allow(clippy::too_many_lines)] // This test checks every invalid record shape against the same ABI contract.
fn test_s2c_capabilities_contract_validation() {
    let mut st = ptr::null_mut();

    // 1. NULL capabilities
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(0x01, ptr::null(), &raw mut st),
        ERROR_INVALID_INPUT
    );

    // 2. struct_size = 0
    let mut caps = kmipkit_s2c_capabilities_v1_t {
        struct_size: 0,
        abi_version: 1,
        operations: ptr::null(),
        operations_count: 0,
        object_types: ptr::null(),
        object_types_count: 0,
        vendor_identification_data: ptr::null(),
        vendor_identification_length: 0,
        extension_tags: ptr::null(),
        extension_tags_count: 0,
        reserved: 0,
    };
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x01,
            &raw const caps,
            &raw mut st
        ),
        ERROR_INVALID_INPUT
    );

    // 3. struct_size != sizeof
    caps.struct_size = 99;
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x01,
            &raw const caps,
            &raw mut st
        ),
        ERROR_INVALID_INPUT
    );

    // 4. abi_version != 1
    caps.struct_size = abi_u32(std::mem::size_of::<kmipkit_s2c_capabilities_v1_t>());
    caps.abi_version = 99;
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x01,
            &raw const caps,
            &raw mut st
        ),
        ERROR_INVALID_INPUT
    );

    // 5. reserved != 0
    caps.abi_version = 1;
    caps.reserved = 1;
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x01,
            &raw const caps,
            &raw mut st
        ),
        ERROR_INVALID_INPUT
    );
    caps.reserved = 0;

    // 6. operations_count > 0 with NULL operations
    caps.operations_count = 5;
    caps.operations = ptr::null();
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x01,
            &raw const caps,
            &raw mut st
        ),
        ERROR_INVALID_INPUT
    );
    caps.operations_count = 0;

    // 7. object_types_count > 0 with NULL object_types
    caps.object_types_count = 5;
    caps.object_types = ptr::null();
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x02,
            &raw const caps,
            &raw mut st
        ),
        ERROR_INVALID_INPUT
    );
    caps.object_types_count = 0;

    // 8. extension_tags_count > 0 with NULL extension_tags
    caps.extension_tags_count = 5;
    caps.extension_tags = ptr::null();
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x08,
            &raw const caps,
            &raw mut st
        ),
        ERROR_INVALID_INPUT
    );
    caps.extension_tags_count = 0;

    // 9. vendor_identification_length > 0 with NULL data
    caps.vendor_identification_length = 10;
    caps.vendor_identification_data = ptr::null();
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x04,
            &raw const caps,
            &raw mut st
        ),
        ERROR_INVALID_INPUT
    );
    caps.vendor_identification_length = 0;

    // 10. Invalid UTF-8 in vendor_identification
    let bad_utf8 = [0xFFu8, 0xFEu8];
    caps.vendor_identification_data = bad_utf8.as_ptr();
    caps.vendor_identification_length = bad_utf8.len() as u64;
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x04,
            &raw const caps,
            &raw mut st
        ),
        ERROR_INVALID_INPUT
    );
    caps.vendor_identification_data = ptr::null();
    caps.vendor_identification_length = 0;

    // 11. Count > MAX_S2C_CAPABILITY_ITEMS (1024)
    caps.operations_count = 1025;
    let dummy_op = 1u32;
    caps.operations = &raw const dummy_op;
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x01,
            &raw const caps,
            &raw mut st
        ),
        ERROR_RESOURCE_LIMIT
    );
}

#[test]
fn test_s2c_query_deliberately_empty_capabilities() {
    let caps = kmipkit_s2c_capabilities_v1_t {
        struct_size: abi_u32(std::mem::size_of::<kmipkit_s2c_capabilities_v1_t>()),
        abi_version: 1,
        operations: ptr::null(),
        operations_count: 0,
        object_types: ptr::null(),
        object_types_count: 0,
        vendor_identification_data: ptr::null(),
        vendor_identification_length: 0,
        extension_tags: ptr::null(),
        extension_tags_count: 0,
        reserved: 0,
    };

    let mut st = ptr::null_mut();
    // Request Operations (0x01) and Objects (0x02)
    assert_eq!(
        client_abi::kmipkit_s2c_query_respond_with_capabilities_v1(
            0x03,
            &raw const caps,
            &raw mut st
        ),
        SUCCESS
    );
    assert!(!st.is_null());

    let mut view = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_structure_view(st, &raw mut view), SUCCESS);
    let mut count = 0u64;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(view, &raw mut count),
        SUCCESS
    );
    // Deliberately empty: structure should have exactly 0 items, not injected fallbacks!
    assert_eq!(count, 0);

    kmipkit_ttlv_structure_view_release(view);

    kmipkit_ttlv_structure_release(st);
}

#[test]
fn s2c_discover_versions_v2_preserves_order_and_signed_components() {
    use crate::kmipkit_protocol_version_v1_t as Version;
    let v21 = Version { major: 2, minor: 1 };
    let v30 = Version { major: 3, minor: 0 };
    let large = Version {
        major: 70_000,
        minor: 100_000,
    };
    let supported = [large, v30, v21];
    let mut count = 0;
    assert_eq!(
        client_abi::kmipkit_s2c_discover_versions_negotiate_v2(
            ptr::null(),
            0,
            supported.as_ptr(),
            3,
            ptr::null_mut(),
            0,
            &raw mut count
        ),
        SUCCESS
    );
    assert_eq!(count, 3);

    let requested = [v21, large];
    let mut output = [v21; 2];
    assert_eq!(
        client_abi::kmipkit_s2c_discover_versions_negotiate_v2(
            requested.as_ptr(),
            2,
            supported.as_ptr(),
            3,
            output.as_mut_ptr(),
            2,
            &raw mut count
        ),
        SUCCESS
    );
    assert_eq!(count, 2);
    assert_eq!(output, [large, v21]);
}

fn assert_query_extension_payload(payload: &kmipkit_ttlv::StructureView<'_>, map: bool) {
    use kmipkit_ttlv::{KmipTag, ValueView};
    let [information] = payload.children() else {
        panic!("expected one Extension Information record");
    };
    assert_eq!(information.tag(), KmipTag::ExtensionInformation.into());
    information.with_value(|value| {
        let ValueView::Structure(fields) = value else {
            panic!("expected structured Extension Information");
        };
        let fields = fields.children();
        assert_eq!(fields.len(), if map { 3 } else { 1 });
        assert_eq!(fields[0].tag(), KmipTag::ExtensionName.into());
        fields[0]
            .with_value(|value| assert!(matches!(value, ValueView::TextString("TenantLabel"))));
        if map {
            assert_eq!(fields[1].tag(), KmipTag::ExtensionTag.into());
            fields[1].with_value(|value| assert!(matches!(value, ValueView::Integer(&0x54_0001))));
            assert_eq!(fields[2].tag(), KmipTag::ExtensionType.into());
            fields[2].with_value(|value| assert!(matches!(value, ValueView::Enumeration(&7))));
        }
    });
}

unsafe extern "C" fn query_metadata_callback(
    user_data: *mut core::ffi::c_void,
    _item_view: *const kmipkit_server_request_item_view_v1_t,
    query_flags: u32,
    output: *mut crate::kmipkit_s2c_capabilities_v2_t,
    _result: *mut kmipkit_s2c_operation_result_v1_t,
) -> u32 {
    if query_flags != 0x30 {
        return 1;
    }
    // The test keeps the metadata array live for the duration of dispatch.
    let output = unsafe { &mut *output };
    output.extension_information = user_data.cast();
    output.extension_information_count = 1;
    0
}

#[test]
#[allow(clippy::too_many_lines)] // The fixture compares all supported protocol versions in one public C ABI scenario.
fn c_abi_extension_query_supports_v2_0_v2_1_and_v3_0() {
    use crate::{
        kmipkit_protocol_version_v1_t as Version, kmipkit_s2c_capabilities_v2_t as Capabilities,
        kmipkit_server_request_handler_v5_t as HandlerV5,
        kmipkit_server_request_handler_v6_t as HandlerV6,
    };
    let name = b"TenantLabel";
    let mut named = ptr::null_mut();
    let mut tagged = ptr::null_mut();
    let mut mapped = ptr::null_mut();
    assert_eq!(
        kmipkit_extension_information_create(name.as_ptr(), name.len() as u64, &raw mut named),
        SUCCESS
    );
    assert_eq!(
        kmipkit_extension_information_tag(named, 0x54_0001, &raw mut tagged),
        SUCCESS
    );
    assert_eq!(
        kmipkit_extension_information_type(tagged, 7, &raw mut mapped),
        SUCCESS
    );
    let mut descriptors = [mapped];

    let base = kmipkit_s2c_capabilities_v1_t {
        struct_size: abi_u32(std::mem::size_of::<kmipkit_s2c_capabilities_v1_t>()),
        abi_version: 1,
        operations: ptr::null(),
        operations_count: 0,
        object_types: ptr::null(),
        object_types_count: 0,
        vendor_identification_data: ptr::null(),
        vendor_identification_length: 0,
        extension_tags: ptr::null(),
        extension_tags_count: 0,
        reserved: 0,
    };
    let caps = Capabilities {
        struct_size: abi_u32(std::mem::size_of::<Capabilities>()),
        abi_version: 2,
        base,
        extension_information: descriptors.as_ptr(),
        extension_information_count: 1,
        reserved: 0,
    };
    let mut dispatcher = ptr::null_mut();
    let mut limits = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_create(&raw mut dispatcher),
        SUCCESS
    );
    assert_eq!(kmipkit_codec_limits_defaults(&raw mut limits), SUCCESS);
    let handler = HandlerV6 {
        struct_size: abi_u32(std::mem::size_of::<HandlerV6>()),
        abi_version: 6,
        base: HandlerV5 {
            struct_size: abi_u32(std::mem::size_of::<HandlerV5>()),
            abi_version: 5,
            user_data: descriptors.as_mut_ptr().cast(),
            handle_discover_versions: None,
            handle_notify: None,
            handle_put: None,
            handle_query: None,
            handle_set_endpoint_role: None,
        },
        handle_query: Some(query_metadata_callback),
    };

    for (major, minor) in [(2i32, 0i32), (2, 1), (3, 0)] {
        let version = Version { major, minor };
        for (flags, map) in [(0x10, false), (0x20, true), (0x30, true)] {
            let mut result = ptr::null_mut();
            assert_eq!(
                client_abi::kmipkit_s2c_query_respond_with_capabilities_v2(
                    flags,
                    version,
                    &raw const caps,
                    &raw mut result
                ),
                SUCCESS
            );
            {
                let owner = reference_handle(result, Kind::Structure).unwrap();
                let HandleValue::Structure(payload) = &owner.value else {
                    unreachable!()
                };
                assert_query_extension_payload(&payload.view(), map);
            }
            kmipkit_ttlv_structure_release(result);
        }

        let version_bytes = [
            ttlv_test_item(0x42_006a, 2, &major.to_be_bytes()),
            ttlv_test_item(0x42_006b, 2, &minor.to_be_bytes()),
        ]
        .concat();
        let mut header = ttlv_test_item(0x42_0069, 1, &version_bytes);
        if major != 3 {
            header.extend_from_slice(&ttlv_test_item(0x42_000d, 2, &1i32.to_be_bytes()));
        }
        let functions = [
            ttlv_test_item(0x42_0074, 5, &5u32.to_be_bytes()),
            ttlv_test_item(0x42_0074, 5, &6u32.to_be_bytes()),
        ]
        .concat();
        let item = [
            ttlv_test_item(0x42_005c, 5, &0x18u32.to_be_bytes()),
            ttlv_test_item(0x42_0079, 1, &functions),
        ]
        .concat();
        let bytes = ttlv_test_item(
            0x42_0078,
            1,
            &[
                ttlv_test_item(0x42_0077, 1, &header),
                ttlv_test_item(0x42_000f, 1, &item),
            ]
            .concat(),
        );
        let mut batch = ptr::null_mut();
        assert_eq!(
            client_abi::kmipkit_incoming_server_batch_decode_ttlv(
                bytes.as_ptr(),
                bytes.len() as u64,
                limits,
                &raw mut batch
            ),
            SUCCESS
        );
        let mut reply = ptr::null_mut();
        assert_eq!(
            client_abi::kmipkit_server_request_dispatcher_dispatch_v6(
                dispatcher,
                batch,
                &raw const handler,
                &raw mut reply
            ),
            SUCCESS
        );
        {
            let owner = reference_handle(reply, Kind::ResponseMessage).unwrap();
            let HandleValue::ResponseMessage(message) = &owner.value else {
                unreachable!()
            };
            assert_eq!(
                message.header().protocol_version(),
                kmipkit_protocol::ProtocolVersion::from_raw(major, minor)
            );
            let item = message.batch_items().next().unwrap();
            assert_eq!(
                item.result_status()
                    .map(kmipkit_protocol::ResultStatus::raw),
                Some(0)
            );
            item.with_response_payload(|payload| assert_query_extension_payload(&payload, true))
                .unwrap();
        }
        kmipkit_response_message_release(reply);
        client_abi::kmipkit_incoming_server_batch_release(batch);
    }

    client_abi::kmipkit_server_request_dispatcher_release(dispatcher);
    kmipkit_codec_limits_release(limits);
    kmipkit_extension_information_release(mapped);
}
