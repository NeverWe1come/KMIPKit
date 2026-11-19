use super::*;
use std::ptr;
use crate::{KMIPKIT_UNIQUE_IDENTIFIER_KIND_IDENTIFIER, KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING};

#[test]
fn test_unique_identifier_v1_identifier_roundtrip() {
    let text = b"test-identifier-uid-42";
    let input = kmipkit_unique_identifier_v1_t {
        struct_size: std::mem::size_of::<kmipkit_unique_identifier_v1_t>() as u32,
        abi_version: 1,
        kind: KMIPKIT_UNIQUE_IDENTIFIER_KIND_IDENTIFIER,
        reserved: 0,
        text_data: text.as_ptr(),
        text_length: text.len() as u64,
        enumeration_value: 0,
        integer_value: 0,
    };

    let parsed = copy_unique_identifier(&input)
        .expect("valid identifier")
        .expect("some identifier");
    assert!(matches!(parsed, kmipkit_protocol::UniqueIdentifier::Identifier(ref s) if s == "test-identifier-uid-42"));

    let mut output: kmipkit_unique_identifier_v1_t = unsafe { std::mem::zeroed() };
    output.struct_size = std::mem::size_of::<kmipkit_unique_identifier_v1_t>() as u32;
    output.abi_version = 1;
    client_abi::write_unique_identifier_v1(&mut output, Some(&parsed)).expect("write succeeds");
    assert_eq!(output.kind, KMIPKIT_UNIQUE_IDENTIFIER_KIND_IDENTIFIER);
    assert_eq!(output.text_length, text.len() as u64);
    let output_slice = unsafe { std::slice::from_raw_parts(output.text_data, output.text_length as usize) };
    assert_eq!(output_slice, text);
}

#[test]
fn test_ttlv_v3_primitives_constructors_and_views() {
    let mut limits_handle = ptr::null_mut();
    assert_eq!(kmipkit_codec_limits_defaults(&mut limits_handle), SUCCESS);
    assert!(!limits_handle.is_null());

    let payload = b"ref-uid-100";
    let mut val_id = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_value_identifier(limits_handle, payload.as_ptr(), payload.len() as u64, &mut val_id),
        SUCCESS
    );
    assert!(!val_id.is_null());

    let mut val_ref = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_value_reference(limits_handle, payload.as_ptr(), payload.len() as u64, &mut val_ref),
        SUCCESS
    );
    assert!(!val_ref.is_null());

    let mut val_name_ref = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_value_name_reference(limits_handle, payload.as_ptr(), payload.len() as u64, &mut val_name_ref),
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
    assert_eq!(kmipkit_version_gate_validate_operation(2, 1, 60), ERROR_INVALID_INPUT);

    // KMIP 3.0 tag in 3.0 vs 2.1
    assert_eq!(kmipkit_version_gate_validate_tag(3, 0, 0x4201BF), SUCCESS);
    assert_eq!(kmipkit_version_gate_validate_tag(2, 1, 0x4201BF), ERROR_INVALID_INPUT);
}

#[test]
fn test_nist_key_lifecycle_ffi() {
    let mut can_orig = 0u8;
    let mut can_proc = 0u8;

    // Active
    assert_eq!(client_abi::kmipkit_nist_key_lifecycle_can_originate(2, &mut can_orig), SUCCESS);
    assert_eq!(client_abi::kmipkit_nist_key_lifecycle_can_process(2, &mut can_proc), SUCCESS);
    assert_eq!(can_orig, 1);
    assert_eq!(can_proc, 1);

    // Deactivated
    assert_eq!(client_abi::kmipkit_nist_key_lifecycle_can_originate(3, &mut can_orig), SUCCESS);
    assert_eq!(client_abi::kmipkit_nist_key_lifecycle_can_process(3, &mut can_proc), SUCCESS);
    assert_eq!(can_orig, 0);
    assert_eq!(can_proc, 1);

    // Destroyed
    assert_eq!(client_abi::kmipkit_nist_key_lifecycle_can_originate(5, &mut can_orig), SUCCESS);
    assert_eq!(client_abi::kmipkit_nist_key_lifecycle_can_process(5, &mut can_proc), SUCCESS);
    assert_eq!(can_orig, 0);
    assert_eq!(can_proc, 0);

    // Invalid enum
    assert_eq!(client_abi::kmipkit_nist_key_lifecycle_can_originate(999, &mut can_orig), ERROR_INVALID_INPUT);
}

#[test]
fn test_lease_context_ffi() {
    let uid_text = b"lease-key-test";
    let uid = kmipkit_unique_identifier_v1_t {
        struct_size: std::mem::size_of::<kmipkit_unique_identifier_v1_t>() as u32,
        abi_version: 1,
        kind: KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: uid_text.as_ptr(),
        text_length: uid_text.len() as u64,
        enumeration_value: 0,
        integer_value: 0,
    };

    let mut lease = ptr::null_mut();
    assert_eq!(kmipkit_lease_context_create(&uid, 100, 50, &mut lease), SUCCESS);
    assert!(!lease.is_null());

    let mut exp = 0i64;
    assert_eq!(kmipkit_lease_context_expires_at(lease, &mut exp), SUCCESS);
    assert_eq!(exp, 150);

    let mut is_exp = 1u8;
    assert_eq!(kmipkit_lease_context_is_expired(lease, 140, &mut is_exp), SUCCESS);
    assert_eq!(is_exp, 0);
    assert_eq!(kmipkit_lease_context_check_valid(lease, 140), SUCCESS);

    assert_eq!(kmipkit_lease_context_is_expired(lease, 151, &mut is_exp), SUCCESS);
    assert_eq!(is_exp, 1);
    assert_eq!(kmipkit_lease_context_check_valid(lease, 151), ERROR_INVALID_INPUT);

    let mut out_uid: kmipkit_unique_identifier_v1_t = unsafe { std::mem::zeroed() };
    out_uid.struct_size = std::mem::size_of::<kmipkit_unique_identifier_v1_t>() as u32;
    out_uid.abi_version = 1;
    assert_eq!(kmipkit_lease_context_unique_identifier(lease, &mut out_uid), SUCCESS);
    assert_eq!(out_uid.kind, KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING);
    assert_eq!(out_uid.text_length, uid_text.len() as u64);

    kmipkit_lease_context_release(lease);
}

#[test]
fn test_cryptographic_parameters_v2_ffi() {
    let mut fields: kmipkit_cryptographic_parameters_fields_v2_t = unsafe { std::mem::zeroed() };
    fields.struct_size = std::mem::size_of::<kmipkit_cryptographic_parameters_fields_v2_t>() as u32;
    fields.abi_version = 2;
    fields.kem_algorithm_present = 1;
    fields.kem_algorithm = 2; // DHKEM
    fields.deterministic_present = 1;
    fields.deterministic = 1;

    let mut params = ptr::null_mut();
    assert_eq!(
        unsafe { cryptographic_parameters::kmipkit_cryptographic_parameters_from_fields_v2(&fields, &mut params) },
        SUCCESS
    );
    assert!(!params.is_null());

    let mut out_fields: kmipkit_cryptographic_parameters_fields_v2_t = unsafe { std::mem::zeroed() };
    out_fields.struct_size = std::mem::size_of::<kmipkit_cryptographic_parameters_fields_v2_t>() as u32;
    out_fields.abi_version = 2;
    assert_eq!(
        unsafe { cryptographic_parameters::kmipkit_cryptographic_parameters_read_fields_v2(params, &mut out_fields) },
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
    assert_eq!(kmipkit_codec_limits_defaults(&mut limits), SUCCESS);

    // Build a batch item structure with Operation, ResultStatus, ResponsePayload
    let mut batch_item = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_structure_create(&mut batch_item), SUCCESS);

    let mut raw_tag = ptr::null_mut();
    let mut tag = ptr::null_mut();
    let mut val = ptr::null_mut();
    let mut item = ptr::null_mut();
    let mut s1 = ptr::null_mut();

    // 1. Operation = 1
    assert_eq!(kmipkit_ttlv_raw_tag_create(0x42005C, &mut raw_tag), SUCCESS);
    assert_eq!(kmipkit_ttlv_raw_tag_try_checked(raw_tag, &mut tag), SUCCESS);
    kmipkit_raw_tag_release(raw_tag);
    assert_eq!(kmipkit_ttlv_value_enumeration(1, &mut val), SUCCESS);
    assert_eq!(kmipkit_ttlv_item_create(tag, val, limits, &mut item), SUCCESS);
    kmipkit_tag_release(tag);
    assert_eq!(kmipkit_ttlv_structure_with_item(batch_item, item, limits, &mut s1), SUCCESS);
    kmipkit_ttlv_structure_release(batch_item);
    kmipkit_ttlv_item_release(item);

    // 2. ResultStatus = 0
    assert_eq!(kmipkit_ttlv_raw_tag_create(0x42007F, &mut raw_tag), SUCCESS);
    assert_eq!(kmipkit_ttlv_raw_tag_try_checked(raw_tag, &mut tag), SUCCESS);
    kmipkit_raw_tag_release(raw_tag);
    assert_eq!(kmipkit_ttlv_value_enumeration(0, &mut val), SUCCESS);
    assert_eq!(kmipkit_ttlv_item_create(tag, val, limits, &mut item), SUCCESS);
    kmipkit_tag_release(tag);
    let mut s2 = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_structure_with_item(s1, item, limits, &mut s2), SUCCESS);
    kmipkit_ttlv_structure_release(s1);
    kmipkit_ttlv_item_release(item);

    // 3. ResponsePayload = empty structure
    assert_eq!(kmipkit_ttlv_raw_tag_create(0x42007C, &mut raw_tag), SUCCESS);
    assert_eq!(kmipkit_ttlv_raw_tag_try_checked(raw_tag, &mut tag), SUCCESS);
    kmipkit_raw_tag_release(raw_tag);
    let mut empty_payload = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_structure_create(&mut empty_payload), SUCCESS);
    assert_eq!(kmipkit_ttlv_value_structure(empty_payload, limits, &mut val), SUCCESS);
    kmipkit_ttlv_structure_release(empty_payload);
    assert_eq!(kmipkit_ttlv_item_create(tag, val, limits, &mut item), SUCCESS);
    kmipkit_tag_release(tag);
    let mut s3 = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_structure_with_item(s2, item, limits, &mut s3), SUCCESS);
    kmipkit_ttlv_structure_release(s2);
    kmipkit_ttlv_item_release(item);

    let items = [s3];
    let mut resp = ptr::null_mut();
    assert_eq!(
        kmipkit_s2c_build_response_ttlv(
            2,
            1,
            items.as_ptr(),
            items.len() as u64,
            &mut resp,
        ),
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
            &mut item,
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
            &mut intro_maj,
            &mut intro_min,
            &mut depr_maj,
            &mut depr_min,
            &mut rem_maj,
            &mut rem_min,
        ),
        SUCCESS
    );
    assert_eq!(intro_maj, 2);
    assert_eq!(intro_min, 0);

    let supported = [((2u32) << 16) | 1, ((2u32) << 16) | 0];
    let mut neg_maj = 0;
    let mut neg_min = 0;
    assert_eq!(
        client_abi::kmipkit_s2c_discover_versions_negotiate(
            2,
            1,
            supported.as_ptr(),
            supported.len() as u32,
            &mut neg_maj,
            &mut neg_min,
        ),
        SUCCESS
    );
    assert_eq!(neg_maj, 2);
    assert_eq!(neg_min, 1);

    let mut eff = 0;
    assert_eq!(
        client_abi::kmipkit_s2c_put_effective_function(
            2,
            0,
            &mut eff,
        ),
        SUCCESS
    );
    assert_eq!(eff, 1);

    assert_eq!(
        client_abi::kmipkit_s2c_put_effective_function(
            2,
            1,
            &mut eff,
        ),
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
            &mut query_ttlv,
        ),
        SUCCESS
    );
    assert!(!query_ttlv.is_null());
    kmipkit_ttlv_structure_release(query_ttlv);

    let mut dispatcher = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_server_request_dispatcher_create(&mut dispatcher),
        SUCCESS
    );
    assert!(!dispatcher.is_null());
    client_abi::kmipkit_server_request_dispatcher_release(dispatcher);
}
