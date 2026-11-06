#![deny(unused_unsafe)]

use super::*;

// This helper is intentionally compiled but not invoked. Under `deny(unused_unsafe)`, every
// explicit unsafe block below proves that the corresponding raw-pointer entry point requires an
// unsafe Rust call. C callers retain the same ABI.
fn raw_pointer_ffi_call_sites() {
    let _ = unsafe {
        client_abi::kmipkit_client_batch_item_response_pkcs11_correlation_value(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_async_operation_outcome_metadata_v1(
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_execute_with_options_v1(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_execute_with_options_and_error_ttlv_v1(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_batch_item_response_metadata_v2(ptr::null_mut(), ptr::null_mut())
    };
    let _ = unsafe {
        client_abi::kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_batch_item_response_get_attribute_list(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
        )
    };
    let _ = unsafe {
        client_abi::kmipkit_client_batch_item_response_discover_versions_supported_versions(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
        )
    };
    let _ = unsafe { kmipkit_api_levels_get_v1(ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) };
    let _ = unsafe {
        kmipkit_client_batch_item_validate(
            ptr::null_mut(),
            0,
            ptr::null(),
            0,
            ptr::null_mut(),
            0,
            0,
            0,
            ptr::null_mut(),
        )
    };
}

#[test]
fn ffi_exports_reject_nulls_and_mismatched_handle_payloads() {
    raw_pointer_ffi_call_sites();
    null_pointer_export_matrix();
}

fn null_pointer_export_matrix() {
    let malformed_add_attribute_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::AddAttributeRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_adjust_attribute_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::AdjustAttributeRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_attestation_credential_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::AttestationCredential,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_attribute_set_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::AttributeSet, HandleValue::Structure(Structure::new()));
    let malformed_authentication_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::Authentication,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_batch_item_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::BatchItem, HandleValue::Structure(Structure::new()));
    let malformed_cancel_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::CancelRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_client_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Client, HandleValue::Structure(Structure::new()));
    let malformed_client_async_operation_outcome_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(
            Kind::ClientAsyncOperationOutcome,
            HandleValue::Structure(Structure::new()),
        );
    let malformed_client_batch_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::ClientBatch, HandleValue::Structure(Structure::new()));
    let malformed_client_batch_item_response_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_client_batch_response_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_codec_limits_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::CodecLimits, HandleValue::Structure(Structure::new()));
    let malformed_credential_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Credential, HandleValue::Structure(Structure::new()));
    let malformed_credential_value_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::CredentialValue,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_credential_view_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::CredentialView,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_delete_attribute_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::DeleteAttributeRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_get_attribute_list_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::GetAttributeListRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_hashed_password_credential_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::HashedPasswordCredential,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_hashed_password_timestamp_sequence_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(
            Kind::HashedPasswordTimestampSequence,
            HandleValue::Structure(Structure::new()),
        );
    let malformed_information_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Information, HandleValue::Structure(Structure::new()));
    let malformed_message_validation_error_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::MessageValidationError,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_modify_attribute_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::ModifyAttributeRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_poll_request_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::PollRequest, HandleValue::Structure(Structure::new()));
    let malformed_process_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::ProcessRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_profile_readiness_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::ProfileReadiness,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_profile_validation_error_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::ProfileValidationError,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_profile_validation_result_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::ProfileValidationResult,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_query_async_requests_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::QueryAsyncRequestsRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_register_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::RegisterRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_registry_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Registry, HandleValue::Structure(Structure::new()));
    let malformed_request_batch_item_view_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::RequestBatchItemView,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_request_header_view_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::RequestHeaderView,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_request_message_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::RequestMessage,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_response_batch_item_view_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::ResponseBatchItemView,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_response_header_view_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::ResponseHeaderView,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_response_message_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::ResponseMessage,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_revoke_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::RevokeRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_set_attribute_request_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::SetAttributeRequest,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_structure_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Structure, HandleValue::Value(Value::integer(0)));
    let malformed_usage_allocation_context_handle: *mut kmipkit_ttlv_structure_t = make_handle(
        Kind::UsageAllocationContext,
        HandleValue::Structure(Structure::new()),
    );
    let malformed_username_and_password_credential_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(
            Kind::UsernameAndPasswordCredential,
            HandleValue::Structure(Structure::new()),
        );
    let malformed_validated_handle: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Validated, HandleValue::Structure(Structure::new()));

    {
        let status = unsafe {
            super::kmipkit_api_levels_get_v1(ptr::null_mut(), ptr::null_mut(), ptr::null_mut())
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_api_levels_get_v1 malformed handle input must not panic"
        );
    }
    {
        let status =
            unsafe { super::kmipkit_last_credential_validation_error_get_v1(ptr::null_mut()) };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_last_credential_validation_error_get_v1 malformed handle input must not panic"
        );
    }
    {
        let status = unsafe {
            super::kmipkit_profile_validation_error_view_get_v1(
                malformed_profile_validation_error_handle.cast(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_error_view_get_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_error_view_get_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_profile_validation_error_view_get_v1_out_view =
            std::mem::MaybeUninit::<kmipkit_profile_validation_error_view_v1_t>::uninit();
        let status = unsafe {
            super::kmipkit_profile_validation_error_view_get_v1(
                malformed_profile_validation_error_handle.cast(),
                _kmipkit_profile_validation_error_view_get_v1_out_view.as_mut_ptr(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_error_view_get_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_error_view_get_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_error_view_get_v1 with valid output slots must not panic"
        );
    }
    {
        let status = unsafe {
            super::kmipkit_profile_readiness_evaluate_v1(
                ptr::null(),
                ptr::null(),
                0,
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_readiness_evaluate_v1 malformed handle input must not panic"
        );
    }
    {
        let status = unsafe {
            super::kmipkit_profile_readiness_view_get_v1(
                malformed_profile_readiness_handle.cast(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_readiness_view_get_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_readiness_view_get_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_profile_readiness_view_get_v1_out_view =
            std::mem::MaybeUninit::<crate::kmipkit_profile_readiness_view_v1_t>::uninit();
        let status = unsafe {
            super::kmipkit_profile_readiness_view_get_v1(
                malformed_profile_readiness_handle.cast(),
                _kmipkit_profile_readiness_view_get_v1_out_view.as_mut_ptr(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_readiness_view_get_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_readiness_view_get_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_readiness_view_get_v1 with valid output slots must not panic"
        );
    }
    {
        let status = unsafe {
            super::kmipkit_profile_readiness_blocker_at_v1(
                malformed_profile_readiness_handle.cast(),
                0,
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_readiness_blocker_at_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_readiness_blocker_at_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_profile_readiness_blocker_at_v1_out_view =
            std::mem::MaybeUninit::<crate::kmipkit_profile_readiness_blocker_view_v1_t>::uninit();
        let status = unsafe {
            super::kmipkit_profile_readiness_blocker_at_v1(
                malformed_profile_readiness_handle.cast(),
                0,
                _kmipkit_profile_readiness_blocker_at_v1_out_view.as_mut_ptr(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_readiness_blocker_at_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_readiness_blocker_at_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_readiness_blocker_at_v1 with valid output slots must not panic"
        );
    }
    {
        let status = unsafe { super::kmipkit_profile_catalog_row_at_v1(0, 0, ptr::null_mut()) };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_catalog_row_at_v1 malformed handle input must not panic"
        );
    }
    {
        let status =
            unsafe { super::kmipkit_profile_catalog_relation_count_v1(0, 0, 0, ptr::null_mut()) };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_catalog_relation_count_v1 malformed handle input must not panic"
        );
    }
    {
        let status =
            unsafe { super::kmipkit_profile_catalog_relation_at_v1(0, 0, 0, 0, ptr::null_mut()) };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_catalog_relation_at_v1 malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_extension_schema_structure(
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            0,
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_extension_schema_structure malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_client_extension_registry_create(
            ptr::null_mut(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_extension_registry_create malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_client_extension_registry_definition_at(
            malformed_registry_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_extension_registry_definition_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_extension_registry_definition_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_with_extension(
            malformed_batch_item_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_with_extension malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_with_extension mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_with_unique_batch_item_id(
            malformed_batch_item_handle.cast(),
            ptr::null(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_with_unique_batch_item_id malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_with_unique_batch_item_id mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_pkcs11(
            malformed_structure_handle.cast(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_pkcs11 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_pkcs11 mismatched handle payload must be rejected"
        );
    }
    {
        let status = unsafe {
            super::kmipkit_client_batch_item_pkcs11_with_extensions_v1(
                malformed_structure_handle.cast(),
                ptr::null_mut(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_pkcs11_with_extensions_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_pkcs11_with_extensions_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let status = unsafe {
            super::kmipkit_client_batch_item_validate(
                ptr::null_mut(),
                0,
                ptr::null(),
                0,
                ptr::null_mut(),
                0,
                0,
                0,
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_validate malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_reprovision(
            malformed_structure_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_reprovision malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_reprovision mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_derive_key(
            0,
            ptr::null(),
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_derive_key malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_join_split_key(
            0,
            ptr::null_mut(),
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_join_split_key malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_get_attributes(
            ptr::null(),
            ptr::null(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_get_attributes malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_delete_attribute(
            ptr::null(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_delete_attribute malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_locate(
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_locate malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_create_key_pair(
            malformed_structure_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_create_key_pair malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_create_key_pair mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_register_request_create(
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_register_request_create malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_revocation_reason_create(0, 0, ptr::null(), 0, ptr::null_mut());
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_revocation_reason_create malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_revoke_request_create(
            ptr::null(),
            ptr::null_mut(),
            0,
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_revoke_request_create malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_register_request_with_opaque_object_context(
            malformed_register_request_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_register_request_with_opaque_object_context malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_register_request_with_opaque_object_context mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_register_request_with_protection_storage_masks(
            malformed_register_request_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_register_request_with_protection_storage_masks malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_register_request_with_protection_storage_masks mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_register(
            malformed_register_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_register malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_register mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_revoke(
            malformed_revoke_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_revoke malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_revoke mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_extension_count(
            malformed_batch_item_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_extension_count malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_extension_count mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_extension_count_out_count =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = super::kmipkit_client_batch_item_extension_count(
            malformed_batch_item_handle.cast(),
            _kmipkit_client_batch_item_extension_count_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_extension_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_extension_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_extension_count with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_extension_identity_at(
            malformed_batch_item_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_extension_identity_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_extension_identity_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_client_batch_item_extension_criticality_indicator_at(
            malformed_batch_item_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_extension_criticality_indicator_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_extension_criticality_indicator_at mismatched handle payload must be rejected"
        );
    }
    {
        let mut
        _kmipkit_client_batch_item_extension_criticality_indicator_at_out_criticality_indicator =
            std::mem::MaybeUninit::<u8>::uninit();
        let status = super::kmipkit_client_batch_item_extension_criticality_indicator_at(
            malformed_batch_item_handle.cast(),
            0,
            _kmipkit_client_batch_item_extension_criticality_indicator_at_out_criticality_indicator
                .as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_extension_criticality_indicator_at with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_extension_criticality_indicator_at with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_extension_criticality_indicator_at with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_extension_information_attribute(
            malformed_information_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_extension_information_attribute malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_extension_information_attribute mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_validated_extension_value_value_at(
            malformed_validated_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_validated_extension_value_value_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_validated_extension_value_value_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_codec_limits_max_elements(
            malformed_codec_limits_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_codec_limits_max_elements malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_codec_limits_max_elements mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_ttlv_decode_with_limits(
            ptr::null(),
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_ttlv_decode_with_limits malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_request_message_from_ttlv(
            malformed_structure_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_message_from_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_message_from_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_request_message_ttlv(
            malformed_request_message_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_message_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_message_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_request_header_view_ttlv(
            malformed_request_header_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_header_view_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_request_header_view_protocol_version(
            malformed_request_header_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_header_view_protocol_version malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_protocol_version mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_request_header_view_protocol_version_out_version =
            std::mem::MaybeUninit::<crate::kmipkit_protocol_version_v1_t>::uninit();
        let status = super::kmipkit_request_header_view_protocol_version(
            malformed_request_header_view_handle.cast(),
            _kmipkit_request_header_view_protocol_version_out_version.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_header_view_protocol_version with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_protocol_version with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_protocol_version with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_request_header_view_batch_count(
            malformed_request_header_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_header_view_batch_count malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_batch_count mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_request_header_view_batch_count_out_count =
            std::mem::MaybeUninit::<i32>::uninit();
        let status = super::kmipkit_request_header_view_batch_count(
            malformed_request_header_view_handle.cast(),
            _kmipkit_request_header_view_batch_count_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_header_view_batch_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_batch_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_batch_count with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_request_header_view_effective_attestation_capable_indicator(
            malformed_request_header_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_header_view_effective_attestation_capable_indicator malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_effective_attestation_capable_indicator mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_request_header_view_effective_attestation_capable_indicator_out_indicator =
            std::mem::MaybeUninit::<u8>::uninit();
        let status = super::kmipkit_request_header_view_effective_attestation_capable_indicator(
            malformed_request_header_view_handle.cast(),
            _kmipkit_request_header_view_effective_attestation_capable_indicator_out_indicator
                .as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_header_view_effective_attestation_capable_indicator with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_effective_attestation_capable_indicator with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_effective_attestation_capable_indicator with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_request_header_view_effective_batch_error_continuation_option(
            malformed_request_header_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_header_view_effective_batch_error_continuation_option malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_effective_batch_error_continuation_option mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_request_header_view_effective_batch_error_continuation_option_out_option =
            std::mem::MaybeUninit::<u32>::uninit();
        let status = super::kmipkit_request_header_view_effective_batch_error_continuation_option(
            malformed_request_header_view_handle.cast(),
            _kmipkit_request_header_view_effective_batch_error_continuation_option_out_option
                .as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_header_view_effective_batch_error_continuation_option with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_effective_batch_error_continuation_option with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_header_view_effective_batch_error_continuation_option with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_request_message_batch_item_at(
            malformed_request_message_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_message_batch_item_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_message_batch_item_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_request_batch_item_view_ttlv(
            malformed_request_batch_item_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_batch_item_view_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_batch_item_view_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_request_batch_item_view_operation(
            malformed_request_batch_item_view_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_batch_item_view_operation malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_batch_item_view_operation mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_request_batch_item_view_operation_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_request_batch_item_view_operation_out_operation =
            std::mem::MaybeUninit::<u32>::uninit();
        let status = super::kmipkit_request_batch_item_view_operation(
            malformed_request_batch_item_view_handle.cast(),
            _kmipkit_request_batch_item_view_operation_out_present.as_mut_ptr(),
            _kmipkit_request_batch_item_view_operation_out_operation.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_batch_item_view_operation with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_batch_item_view_operation with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_batch_item_view_operation with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_response_message_ttlv(
            malformed_response_message_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_response_message_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_response_message_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_response_header_view_batch_count(
            malformed_response_header_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_response_header_view_batch_count malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_response_header_view_batch_count mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_response_header_view_batch_count_out_count =
            std::mem::MaybeUninit::<i32>::uninit();
        let status = super::kmipkit_response_header_view_batch_count(
            malformed_response_header_view_handle.cast(),
            _kmipkit_response_header_view_batch_count_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_response_header_view_batch_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_response_header_view_batch_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_response_header_view_batch_count with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_response_message_batch_item_at(
            malformed_response_message_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_response_message_batch_item_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_response_message_batch_item_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_request_batch_item_extension_count(
            malformed_request_batch_item_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_batch_item_extension_count malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_batch_item_extension_count mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_request_batch_item_extension_count_out_count =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = super::kmipkit_request_batch_item_extension_count(
            malformed_request_batch_item_view_handle.cast(),
            _kmipkit_request_batch_item_extension_count_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_request_batch_item_extension_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_batch_item_extension_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_request_batch_item_extension_count with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_response_batch_item_extension_count(
            malformed_response_batch_item_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_response_batch_item_extension_count malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_response_batch_item_extension_count mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_response_batch_item_extension_count_out_count =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = super::kmipkit_response_batch_item_extension_count(
            malformed_response_batch_item_view_handle.cast(),
            _kmipkit_response_batch_item_extension_count_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_response_batch_item_extension_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_response_batch_item_extension_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_response_batch_item_extension_count with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_response_batch_item_extension_at(
            malformed_response_batch_item_view_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_response_batch_item_extension_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_response_batch_item_extension_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_message_validation_error_kind(
            malformed_message_validation_error_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_message_validation_error_kind malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_message_validation_error_kind mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_message_validation_error_kind_out_kind =
            std::mem::MaybeUninit::<u32>::uninit();
        let status = super::kmipkit_message_validation_error_kind(
            malformed_message_validation_error_handle.cast(),
            _kmipkit_message_validation_error_kind_out_kind.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_message_validation_error_kind with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_message_validation_error_kind with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_message_validation_error_kind with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_message_validation_error_field_index(
            malformed_message_validation_error_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_message_validation_error_field_index malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_message_validation_error_field_index mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_message_validation_error_field_index_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_message_validation_error_field_index_out_index =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = super::kmipkit_message_validation_error_field_index(
            malformed_message_validation_error_handle.cast(),
            _kmipkit_message_validation_error_field_index_out_present.as_mut_ptr(),
            _kmipkit_message_validation_error_field_index_out_index.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_message_validation_error_field_index with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_message_validation_error_field_index with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_message_validation_error_field_index with valid output slots must not panic"
        );
    }
    {
        let status =
            super::kmipkit_credential_type(malformed_credential_handle.cast(), ptr::null_mut());
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_type malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_type mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_credential_type_out_raw = std::mem::MaybeUninit::<u32>::uninit();
        let status = super::kmipkit_credential_type(
            malformed_credential_handle.cast(),
            _kmipkit_credential_type_out_raw.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_type with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_type with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_type with valid output slots must not panic"
        );
    }
    {
        let status =
            super::kmipkit_credential_type_raw(malformed_credential_handle.cast(), ptr::null_mut());
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_type_raw malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_type_raw mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_credential_type_raw_out_raw = std::mem::MaybeUninit::<u32>::uninit();
        let status = super::kmipkit_credential_type_raw(
            malformed_credential_handle.cast(),
            _kmipkit_credential_type_raw_out_raw.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_type_raw with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_type_raw with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_type_raw with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_credential_value_from_ttlv(0, ptr::null_mut(), ptr::null_mut());
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_value_from_ttlv malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_credential_value_type_raw(
            malformed_credential_value_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_value_type_raw malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_value_type_raw mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_credential_value_type_raw_out_raw = std::mem::MaybeUninit::<u32>::uninit();
        let status = super::kmipkit_credential_value_type_raw(
            malformed_credential_value_handle.cast(),
            _kmipkit_credential_value_type_raw_out_raw.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_value_type_raw with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_value_type_raw with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_value_type_raw with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_authentication_create(ptr::null_mut(), 0, ptr::null_mut());
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_authentication_create malformed handle input must not panic"
        );
    }
    {
        let status = super::kmipkit_authentication_from_ttlv(
            malformed_structure_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_authentication_from_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_authentication_from_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_authentication_credential_count(
            malformed_authentication_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_authentication_credential_count malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_authentication_credential_count mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_authentication_credential_count_out_count =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = super::kmipkit_authentication_credential_count(
            malformed_authentication_handle.cast(),
            _kmipkit_authentication_credential_count_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_authentication_credential_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_authentication_credential_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_authentication_credential_count with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_authentication_credential_at(
            malformed_authentication_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_authentication_credential_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_authentication_credential_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_credential_view_type(
            malformed_credential_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_view_type malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_view_type mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_credential_view_type_out_raw = std::mem::MaybeUninit::<u32>::uninit();
        let status = super::kmipkit_credential_view_type(
            malformed_credential_view_handle.cast(),
            _kmipkit_credential_view_type_out_raw.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_view_type with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_view_type with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_view_type with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_credential_view_type_raw(
            malformed_credential_view_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_view_type_raw malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_view_type_raw mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_credential_view_type_raw_out_raw = std::mem::MaybeUninit::<u32>::uninit();
        let status = super::kmipkit_credential_view_type_raw(
            malformed_credential_view_handle.cast(),
            _kmipkit_credential_view_type_raw_out_raw.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_credential_view_type_raw with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_view_type_raw with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_credential_view_type_raw with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_attestation_credential_from_ttlv(
            malformed_structure_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_attestation_credential_from_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attestation_credential_from_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_attestation_credential_into_ttlv(
            malformed_attestation_credential_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_attestation_credential_into_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attestation_credential_into_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_device_credential_from_ttlv(
            malformed_structure_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_device_credential_from_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_device_credential_from_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_username_and_password_credential_from_ttlv(
            malformed_structure_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_username_and_password_credential_from_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_username_and_password_credential_from_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_username_and_password_credential_into_ttlv(
            malformed_username_and_password_credential_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_username_and_password_credential_into_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_username_and_password_credential_into_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_hashed_password_credential_from_ttlv(
            malformed_structure_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_hashed_password_credential_from_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_credential_from_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_hashed_password_credential_hashing_algorithm_raw(
            malformed_hashed_password_credential_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_hashed_password_credential_hashing_algorithm_raw malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_credential_hashing_algorithm_raw mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_hashed_password_credential_hashing_algorithm_raw_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_hashed_password_credential_hashing_algorithm_raw_out_algorithm =
            std::mem::MaybeUninit::<u32>::uninit();
        let status = super::kmipkit_hashed_password_credential_hashing_algorithm_raw(
            malformed_hashed_password_credential_handle.cast(),
            _kmipkit_hashed_password_credential_hashing_algorithm_raw_out_present.as_mut_ptr(),
            _kmipkit_hashed_password_credential_hashing_algorithm_raw_out_algorithm.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_hashed_password_credential_hashing_algorithm_raw with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_credential_hashing_algorithm_raw with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_credential_hashing_algorithm_raw with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_hashed_password_credential_effective_hashing_algorithm_raw(
            malformed_hashed_password_credential_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_hashed_password_credential_effective_hashing_algorithm_raw malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_credential_effective_hashing_algorithm_raw mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_hashed_password_credential_effective_hashing_algorithm_raw_out_algorithm =
            std::mem::MaybeUninit::<u32>::uninit();
        let status = super::kmipkit_hashed_password_credential_effective_hashing_algorithm_raw(
            malformed_hashed_password_credential_handle.cast(),
            _kmipkit_hashed_password_credential_effective_hashing_algorithm_raw_out_algorithm
                .as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_hashed_password_credential_effective_hashing_algorithm_raw with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_credential_effective_hashing_algorithm_raw with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_credential_effective_hashing_algorithm_raw with valid output slots must not panic"
        );
    }
    {
        let status = super::kmipkit_hashed_password_timestamp_sequence_record(
            malformed_hashed_password_timestamp_sequence_handle.cast(),
            0,
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_hashed_password_timestamp_sequence_record malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_timestamp_sequence_record mismatched handle payload must be rejected"
        );
    }
    {
        let status = super::kmipkit_hashed_password_timestamp_sequence_last_timestamp(
            malformed_hashed_password_timestamp_sequence_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_hashed_password_timestamp_sequence_last_timestamp malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_timestamp_sequence_last_timestamp mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_hashed_password_timestamp_sequence_last_timestamp_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_hashed_password_timestamp_sequence_last_timestamp_out_timestamp =
            std::mem::MaybeUninit::<i64>::uninit();
        let status = super::kmipkit_hashed_password_timestamp_sequence_last_timestamp(
            malformed_hashed_password_timestamp_sequence_handle.cast(),
            _kmipkit_hashed_password_timestamp_sequence_last_timestamp_out_present.as_mut_ptr(),
            _kmipkit_hashed_password_timestamp_sequence_last_timestamp_out_timestamp.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_hashed_password_timestamp_sequence_last_timestamp with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_timestamp_sequence_last_timestamp with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_hashed_password_timestamp_sequence_last_timestamp with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_log_ttlv(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_log_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_log_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_add_item(
            malformed_client_batch_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_add_item malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_add_item mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_count(
            malformed_client_batch_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_count malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_count mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_count_out_count = std::mem::MaybeUninit::<u64>::uninit();
        let status = client_abi::kmipkit_client_batch_item_count(
            malformed_client_batch_handle.cast(),
            _kmipkit_client_batch_item_count_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_count with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_at(
            malformed_client_batch_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_set_request_time_stamp(
            malformed_client_batch_handle.cast(),
            0,
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_set_request_time_stamp malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_set_request_time_stamp mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_request_time_stamp(
            malformed_client_batch_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_request_time_stamp malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_request_time_stamp mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_request_time_stamp_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_request_time_stamp_out_time_stamp =
            std::mem::MaybeUninit::<i64>::uninit();
        let status = client_abi::kmipkit_client_batch_request_time_stamp(
            malformed_client_batch_handle.cast(),
            _kmipkit_client_batch_request_time_stamp_out_present.as_mut_ptr(),
            _kmipkit_client_batch_request_time_stamp_out_time_stamp.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_request_time_stamp with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_request_time_stamp with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_request_time_stamp with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_add_error_continuation_value(
            malformed_client_batch_handle.cast(),
            0,
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_add_error_continuation_value malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_add_error_continuation_value mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_set_asynchronous_indicator(
            malformed_client_batch_handle.cast(),
            0,
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_set_asynchronous_indicator malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_set_asynchronous_indicator mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_set_client_correlation_value(
            malformed_client_batch_handle.cast(),
            ptr::null(),
            0,
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_set_client_correlation_value malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_set_client_correlation_value mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_execute(
            malformed_client_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_execute malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_execute mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_add_attribute_request_new_attribute(
            malformed_add_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_add_attribute_request_new_attribute malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_add_attribute_request_new_attribute mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_add_attribute_request_to_ttlv_payload(
            malformed_add_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_add_attribute_request_to_ttlv_payload malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_add_attribute_request_to_ttlv_payload mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_add_attribute_from_request(
            malformed_add_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_add_attribute_from_request malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_add_attribute_from_request mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_adjust_attribute_request_adjustment_value(
            malformed_adjust_attribute_request_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_adjust_attribute_request_adjustment_value malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_adjust_attribute_request_adjustment_value mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_adjust_attribute_from_request(
            malformed_adjust_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_adjust_attribute_from_request malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_adjust_attribute_from_request mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_delete_attribute_request_unique_identifier(
            malformed_delete_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_delete_attribute_request_unique_identifier malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_delete_attribute_request_unique_identifier mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_delete_attribute_request_unique_identifier_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let status = client_abi::kmipkit_delete_attribute_request_unique_identifier(
            malformed_delete_attribute_request_handle.cast(),
            _kmipkit_delete_attribute_request_unique_identifier_out_unique_identifier.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_delete_attribute_request_unique_identifier with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_delete_attribute_request_unique_identifier with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_delete_attribute_request_unique_identifier with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_delete_attribute_request_current_attribute(
            malformed_delete_attribute_request_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_delete_attribute_request_current_attribute malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_delete_attribute_request_current_attribute mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_delete_attribute_request_attribute_reference(
            malformed_delete_attribute_request_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_delete_attribute_request_attribute_reference malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_delete_attribute_request_attribute_reference mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_delete_attribute_request_attribute_reference_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_delete_attribute_request_attribute_reference_out_attribute_reference =
            std::mem::MaybeUninit::<crate::kmipkit_attribute_reference_v1_t>::uninit();
        let status = client_abi::kmipkit_delete_attribute_request_attribute_reference(
            malformed_delete_attribute_request_handle.cast(),
            _kmipkit_delete_attribute_request_attribute_reference_out_present.as_mut_ptr(),
            _kmipkit_delete_attribute_request_attribute_reference_out_attribute_reference
                .as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_delete_attribute_request_attribute_reference with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_delete_attribute_request_attribute_reference with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_delete_attribute_request_attribute_reference with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_delete_attribute_request_to_ttlv_payload(
            malformed_delete_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_delete_attribute_request_to_ttlv_payload malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_delete_attribute_request_to_ttlv_payload mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_delete_attribute_from_request(
            malformed_delete_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_delete_attribute_from_request malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_delete_attribute_from_request mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_modify_attribute_request_unique_identifier(
            malformed_modify_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_modify_attribute_request_unique_identifier malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_modify_attribute_request_unique_identifier mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_modify_attribute_request_unique_identifier_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let status = client_abi::kmipkit_modify_attribute_request_unique_identifier(
            malformed_modify_attribute_request_handle.cast(),
            _kmipkit_modify_attribute_request_unique_identifier_out_unique_identifier.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_modify_attribute_request_unique_identifier with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_modify_attribute_request_unique_identifier with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_modify_attribute_request_unique_identifier with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_modify_attribute_request_current_attribute(
            malformed_modify_attribute_request_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_modify_attribute_request_current_attribute malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_modify_attribute_request_current_attribute mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_modify_attribute_request_new_attribute(
            malformed_modify_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_modify_attribute_request_new_attribute malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_modify_attribute_request_new_attribute mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_modify_attribute_request_to_ttlv_payload(
            malformed_modify_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_modify_attribute_request_to_ttlv_payload malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_modify_attribute_request_to_ttlv_payload mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_modify_attribute_from_request(
            malformed_modify_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_modify_attribute_from_request malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_modify_attribute_from_request mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_set_attribute_request_unique_identifier(
            malformed_set_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_set_attribute_request_unique_identifier malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_set_attribute_request_unique_identifier mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_set_attribute_request_unique_identifier_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let status = client_abi::kmipkit_set_attribute_request_unique_identifier(
            malformed_set_attribute_request_handle.cast(),
            _kmipkit_set_attribute_request_unique_identifier_out_unique_identifier.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_set_attribute_request_unique_identifier with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_set_attribute_request_unique_identifier with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_set_attribute_request_unique_identifier with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_set_attribute_request_new_attribute(
            malformed_set_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_set_attribute_request_new_attribute malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_set_attribute_request_new_attribute mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_set_attribute_request_to_ttlv_payload(
            malformed_set_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_set_attribute_request_to_ttlv_payload malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_set_attribute_request_to_ttlv_payload mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_set_attribute_from_request(
            malformed_set_attribute_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_set_attribute_from_request malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_set_attribute_from_request mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_get_attribute_list_request_unique_identifier(
            malformed_get_attribute_list_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attribute_list_request_unique_identifier malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_get_attribute_list_request_unique_identifier mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_get_attribute_list_request_unique_identifier_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let status = client_abi::kmipkit_get_attribute_list_request_unique_identifier(
            malformed_get_attribute_list_request_handle.cast(),
            _kmipkit_get_attribute_list_request_unique_identifier_out_unique_identifier
                .as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attribute_list_request_unique_identifier with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_get_attribute_list_request_unique_identifier with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_get_attribute_list_request_unique_identifier with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_get_attribute_list_request_to_ttlv_payload(
            malformed_get_attribute_list_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attribute_list_request_to_ttlv_payload malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_get_attribute_list_request_to_ttlv_payload mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_get_attribute_list_from_request(
            malformed_get_attribute_list_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_get_attribute_list_from_request malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_get_attribute_list_from_request mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_get_attributes_request_create(
            ptr::null(),
            ptr::null(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attributes_request_create malformed handle input must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_get_attributes_request_unique_identifier(
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attributes_request_unique_identifier malformed handle input must not panic"
        );
    }
    {
        let mut _kmipkit_get_attributes_request_unique_identifier_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let status = client_abi::kmipkit_get_attributes_request_unique_identifier(
            ptr::null_mut(),
            _kmipkit_get_attributes_request_unique_identifier_out_unique_identifier.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attributes_request_unique_identifier with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_get_attributes_request_attribute_reference_count(
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attributes_request_attribute_reference_count malformed handle input must not panic"
        );
    }
    {
        let mut _kmipkit_get_attributes_request_attribute_reference_count_out_count =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = client_abi::kmipkit_get_attributes_request_attribute_reference_count(
            ptr::null_mut(),
            _kmipkit_get_attributes_request_attribute_reference_count_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attributes_request_attribute_reference_count with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_get_attributes_request_attribute_reference_at(
            ptr::null_mut(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attributes_request_attribute_reference_at malformed handle input must not panic"
        );
    }
    {
        let mut _kmipkit_get_attributes_request_attribute_reference_at_out_reference =
            std::mem::MaybeUninit::<crate::kmipkit_attribute_reference_v1_t>::uninit();
        let status = client_abi::kmipkit_get_attributes_request_attribute_reference_at(
            ptr::null_mut(),
            0,
            _kmipkit_get_attributes_request_attribute_reference_at_out_reference.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attributes_request_attribute_reference_at with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_get_attributes_request_to_ttlv_payload(
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_get_attributes_request_to_ttlv_payload malformed handle input must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_get_attributes_from_request(
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_get_attributes_from_request malformed handle input must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_cancel_request_asynchronous_correlation_value(
            malformed_cancel_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_cancel_request_asynchronous_correlation_value malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_cancel_request_asynchronous_correlation_value mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_cancel_request_asynchronous_correlation_value_out_value =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let status = client_abi::kmipkit_cancel_request_asynchronous_correlation_value(
            malformed_cancel_request_handle.cast(),
            _kmipkit_cancel_request_asynchronous_correlation_value_out_value.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_cancel_request_asynchronous_correlation_value with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_cancel_request_asynchronous_correlation_value with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_cancel_request_asynchronous_correlation_value with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_cancel_request_to_ttlv_payload(
            malformed_cancel_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_cancel_request_to_ttlv_payload malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_cancel_request_to_ttlv_payload mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_poll_request_asynchronous_correlation_value(
            malformed_poll_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_poll_request_asynchronous_correlation_value malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_poll_request_asynchronous_correlation_value mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_poll_request_asynchronous_correlation_value_out_value =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let status = client_abi::kmipkit_poll_request_asynchronous_correlation_value(
            malformed_poll_request_handle.cast(),
            _kmipkit_poll_request_asynchronous_correlation_value_out_value.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_poll_request_asynchronous_correlation_value with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_poll_request_asynchronous_correlation_value with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_poll_request_asynchronous_correlation_value with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_poll_request_to_ttlv_payload(
            malformed_poll_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_poll_request_to_ttlv_payload malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_poll_request_to_ttlv_payload mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_process_request_to_ttlv_payload(
            malformed_process_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_process_request_to_ttlv_payload malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_process_request_to_ttlv_payload mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_query_async_requests_request_with_correlation_values(
            malformed_query_async_requests_request_handle.cast(),
            ptr::null(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_query_async_requests_request_with_correlation_values malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_query_async_requests_request_with_correlation_values mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_query_async_requests_request_with_operations(
            malformed_query_async_requests_request_handle.cast(),
            ptr::null(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_query_async_requests_request_with_operations malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_query_async_requests_request_with_operations mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_query_async_requests_request_to_ttlv_payload(
            malformed_query_async_requests_request_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_query_async_requests_request_to_ttlv_payload malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_query_async_requests_request_to_ttlv_payload mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_execute_process(
            malformed_client_handle.cast(),
            ptr::null(),
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_execute_process malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_execute_process mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_execute_query_async_requests(
            malformed_client_handle.cast(),
            0,
            ptr::null(),
            0,
            0,
            ptr::null(),
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_execute_query_async_requests malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_execute_query_async_requests mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_execute_cancel_with_options_v1(
            malformed_client_handle.cast(),
            ptr::null(),
            0,
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_execute_cancel_with_options_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_execute_cancel_with_options_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_execute_process_with_options_v1(
            malformed_client_handle.cast(),
            ptr::null(),
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_execute_process_with_options_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_execute_process_with_options_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_execute_query_async_requests_with_options_v1(
            malformed_client_handle.cast(),
            0,
            ptr::null(),
            0,
            0,
            ptr::null(),
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_execute_query_async_requests_with_options_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_execute_query_async_requests_with_options_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_client_async_operation_outcome_metadata_v1(
                malformed_client_async_operation_outcome_handle.cast(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_async_operation_outcome_metadata_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_metadata_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_async_operation_outcome_metadata_v1_out_metadata =
            std::mem::MaybeUninit::<kmipkit_client_async_operation_metadata_v1_t>::uninit();
        let status = unsafe {
            client_abi::kmipkit_client_async_operation_outcome_metadata_v1(
                malformed_client_async_operation_outcome_handle.cast(),
                _kmipkit_client_async_operation_outcome_metadata_v1_out_metadata.as_mut_ptr(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_async_operation_outcome_metadata_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_metadata_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_metadata_v1 with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_async_operation_outcome_correlation_value(
            malformed_client_async_operation_outcome_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_async_operation_outcome_correlation_value malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_correlation_value mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_async_operation_outcome_correlation_value_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_async_operation_outcome_correlation_value_out_value =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let status = client_abi::kmipkit_client_async_operation_outcome_correlation_value(
            malformed_client_async_operation_outcome_handle.cast(),
            _kmipkit_client_async_operation_outcome_correlation_value_out_present.as_mut_ptr(),
            _kmipkit_client_async_operation_outcome_correlation_value_out_value.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_async_operation_outcome_correlation_value with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_correlation_value with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_correlation_value with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_async_operation_outcome_response_payload_ttlv(
            malformed_client_async_operation_outcome_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_async_operation_outcome_response_payload_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_response_payload_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_async_operation_outcome_extension_count(
            malformed_client_async_operation_outcome_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_async_operation_outcome_extension_count malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_extension_count mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_async_operation_outcome_extension_count_out_count =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = client_abi::kmipkit_client_async_operation_outcome_extension_count(
            malformed_client_async_operation_outcome_handle.cast(),
            _kmipkit_client_async_operation_outcome_extension_count_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_async_operation_outcome_extension_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_extension_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_extension_count with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_async_operation_outcome_extension_ttlv_at(
            malformed_client_async_operation_outcome_handle.cast(),
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_async_operation_outcome_extension_ttlv_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_async_operation_outcome_extension_ttlv_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_usage_allocation_context_usage_limits_unit(
            malformed_usage_allocation_context_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_usage_allocation_context_usage_limits_unit malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_usage_limits_unit mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_usage_allocation_context_usage_limits_unit_out_unit =
            std::mem::MaybeUninit::<u32>::uninit();
        let status = client_abi::kmipkit_usage_allocation_context_usage_limits_unit(
            malformed_usage_allocation_context_handle.cast(),
            _kmipkit_usage_allocation_context_usage_limits_unit_out_unit.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_usage_allocation_context_usage_limits_unit with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_usage_limits_unit with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_usage_limits_unit with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_usage_allocation_context_allocated_units(
            malformed_usage_allocation_context_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_usage_allocation_context_allocated_units malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_allocated_units mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_usage_allocation_context_allocated_units_out_units =
            std::mem::MaybeUninit::<i64>::uninit();
        let status = client_abi::kmipkit_usage_allocation_context_allocated_units(
            malformed_usage_allocation_context_handle.cast(),
            _kmipkit_usage_allocation_context_allocated_units_out_units.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_usage_allocation_context_allocated_units with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_allocated_units with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_allocated_units with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_usage_allocation_context_available_units(
            malformed_usage_allocation_context_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_usage_allocation_context_available_units malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_available_units mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_usage_allocation_context_available_units_out_units =
            std::mem::MaybeUninit::<i64>::uninit();
        let status = client_abi::kmipkit_usage_allocation_context_available_units(
            malformed_usage_allocation_context_handle.cast(),
            _kmipkit_usage_allocation_context_available_units_out_units.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_usage_allocation_context_available_units with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_available_units with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_available_units with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_usage_allocation_context_reserved_units(
            malformed_usage_allocation_context_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_usage_allocation_context_reserved_units malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_reserved_units mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_usage_allocation_context_reserved_units_out_units =
            std::mem::MaybeUninit::<i64>::uninit();
        let status = client_abi::kmipkit_usage_allocation_context_reserved_units(
            malformed_usage_allocation_context_handle.cast(),
            _kmipkit_usage_allocation_context_reserved_units_out_units.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_usage_allocation_context_reserved_units with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_reserved_units with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_reserved_units with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_usage_allocation_context_unique_identifier(
            malformed_usage_allocation_context_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_usage_allocation_context_unique_identifier malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_unique_identifier mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_usage_allocation_context_unique_identifier_out_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let status = client_abi::kmipkit_usage_allocation_context_unique_identifier(
            malformed_usage_allocation_context_handle.cast(),
            _kmipkit_usage_allocation_context_unique_identifier_out_identifier.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_usage_allocation_context_unique_identifier with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_unique_identifier with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_usage_allocation_context_unique_identifier with valid output slots must not panic"
        );
    }
    {
        let status =
            client_abi::kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator(
                malformed_client_handle.cast(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator mismatched handle payload must be rejected"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_client_execute_with_options_v1(
                malformed_client_handle.cast(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_execute_with_options_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_execute_with_options_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_profile_validation_selection_validate_v1(
                ptr::null(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_selection_validate_v1 malformed handle input must not panic"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_profile_id_count_v1(
                malformed_profile_validation_result_handle.cast(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_profile_id_count_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_profile_id_count_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_profile_validation_result_profile_id_count_v1_out_count =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_profile_id_count_v1(
                malformed_profile_validation_result_handle.cast(),
                _kmipkit_profile_validation_result_profile_id_count_v1_out_count.as_mut_ptr(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_profile_id_count_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_profile_id_count_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_profile_id_count_v1 with valid output slots must not panic"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_profile_id_at_v1(
                malformed_profile_validation_result_handle.cast(),
                0,
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_profile_id_at_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_profile_id_at_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_profile_validation_result_profile_id_at_v1_out_span =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_profile_id_at_v1(
                malformed_profile_validation_result_handle.cast(),
                0,
                _kmipkit_profile_validation_result_profile_id_at_v1_out_span.as_mut_ptr(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_profile_id_at_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_profile_id_at_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_profile_id_at_v1 with valid output slots must not panic"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_requirement_id_count_v1(
                malformed_profile_validation_result_handle.cast(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_requirement_id_count_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_requirement_id_count_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_profile_validation_result_requirement_id_count_v1_out_count =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_requirement_id_count_v1(
                malformed_profile_validation_result_handle.cast(),
                _kmipkit_profile_validation_result_requirement_id_count_v1_out_count.as_mut_ptr(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_requirement_id_count_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_requirement_id_count_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_requirement_id_count_v1 with valid output slots must not panic"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_requirement_id_at_v1(
                malformed_profile_validation_result_handle.cast(),
                0,
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_requirement_id_at_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_requirement_id_at_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_profile_validation_result_requirement_id_at_v1_out_span =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_requirement_id_at_v1(
                malformed_profile_validation_result_handle.cast(),
                0,
                _kmipkit_profile_validation_result_requirement_id_at_v1_out_span.as_mut_ptr(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_requirement_id_at_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_requirement_id_at_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_requirement_id_at_v1 with valid output slots must not panic"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_validate_transport_v1(
                malformed_profile_validation_result_handle.cast(),
                0,
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_validate_transport_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_validate_transport_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_validate_request_v1(
                malformed_profile_validation_result_handle.cast(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_validate_request_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_validate_request_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_validate_request_with_context_v1(
                malformed_profile_validation_result_handle.cast(),
                ptr::null_mut(),
                ptr::null(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_validate_request_with_context_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_validate_request_with_context_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_validate_response_v1(
                malformed_profile_validation_result_handle.cast(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_validate_response_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_validate_response_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_profile_validation_result_validate_response_with_context_v1(
                malformed_profile_validation_result_handle.cast(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_profile_validation_result_validate_response_with_context_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_profile_validation_result_validate_response_with_context_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_response_item_count(
            malformed_client_batch_response_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_response_item_count malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_response_item_count mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_response_item_count_out_count =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = client_abi::kmipkit_client_batch_response_item_count(
            malformed_client_batch_response_handle.cast(),
            _kmipkit_client_batch_response_item_count_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_response_item_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_response_item_count with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_response_item_count with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_response_item_at(
            malformed_client_batch_response_handle.cast(),
            0,
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_response_item_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_response_item_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_extension_ttlv_at(
            malformed_client_batch_item_response_handle.cast(),
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_extension_ttlv_at malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_extension_ttlv_at mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_metadata_v1(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_metadata_v1 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_metadata_v1 mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_metadata_v1_out_metadata =
            std::mem::MaybeUninit::<kmipkit_client_batch_item_metadata_v1_t>::uninit();
        let status = client_abi::kmipkit_client_batch_item_response_metadata_v1(
            malformed_client_batch_item_response_handle.cast(),
            _kmipkit_client_batch_item_response_metadata_v1_out_metadata.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_metadata_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_metadata_v1 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_metadata_v1 with valid output slots must not panic"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_client_batch_item_response_metadata_v2(
                malformed_client_batch_item_response_handle.cast(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_metadata_v2 malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_metadata_v2 mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_metadata_v2_out_metadata =
            std::mem::MaybeUninit::<kmipkit_client_batch_item_metadata_v2_t>::uninit();
        let status = unsafe {
            client_abi::kmipkit_client_batch_item_response_metadata_v2(
                malformed_client_batch_item_response_handle.cast(),
                _kmipkit_client_batch_item_response_metadata_v2_out_metadata.as_mut_ptr(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_metadata_v2 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_metadata_v2 with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_metadata_v2 with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_rekey(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_rekey malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_rekey mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_rekey_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_rekey_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let status = client_abi::kmipkit_client_batch_item_response_rekey(
            malformed_client_batch_item_response_handle.cast(),
            _kmipkit_client_batch_item_response_rekey_out_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_rekey_out_unique_identifier.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_rekey with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_rekey with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_rekey with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_rekey_key_pair(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_rekey_key_pair malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_rekey_key_pair mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_rekey_key_pair_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut
        _kmipkit_client_batch_item_response_rekey_key_pair_out_private_key_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let mut _kmipkit_client_batch_item_response_rekey_key_pair_out_public_key_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let status = client_abi::kmipkit_client_batch_item_response_rekey_key_pair(
            malformed_client_batch_item_response_handle.cast(),
            _kmipkit_client_batch_item_response_rekey_key_pair_out_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_rekey_key_pair_out_private_key_unique_identifier
                .as_mut_ptr(),
            _kmipkit_client_batch_item_response_rekey_key_pair_out_public_key_unique_identifier
                .as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_rekey_key_pair with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_rekey_key_pair with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_rekey_key_pair with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_reprovision(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_reprovision malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_reprovision mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_reprovision_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_reprovision_out_identifier_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_reprovision_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let status = client_abi::kmipkit_client_batch_item_response_reprovision(
            malformed_client_batch_item_response_handle.cast(),
            _kmipkit_client_batch_item_response_reprovision_out_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_reprovision_out_identifier_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_reprovision_out_unique_identifier.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_reprovision with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_reprovision with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_reprovision with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_reprovision_ttlv(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_reprovision_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_reprovision_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_obtain_lease(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_obtain_lease malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_obtain_lease mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_obtain_lease_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_obtain_lease_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let mut _kmipkit_client_batch_item_response_obtain_lease_out_lease_time =
            std::mem::MaybeUninit::<u32>::uninit();
        let mut _kmipkit_client_batch_item_response_obtain_lease_out_last_change_date =
            std::mem::MaybeUninit::<i64>::uninit();
        let status = client_abi::kmipkit_client_batch_item_response_obtain_lease(
            malformed_client_batch_item_response_handle.cast(),
            _kmipkit_client_batch_item_response_obtain_lease_out_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_obtain_lease_out_unique_identifier.as_mut_ptr(),
            _kmipkit_client_batch_item_response_obtain_lease_out_lease_time.as_mut_ptr(),
            _kmipkit_client_batch_item_response_obtain_lease_out_last_change_date.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_obtain_lease with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_obtain_lease with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_obtain_lease with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_join_split_key(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_join_split_key malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_join_split_key mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_join_split_key_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_join_split_key_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let status = client_abi::kmipkit_client_batch_item_response_join_split_key(
            malformed_client_batch_item_response_handle.cast(),
            _kmipkit_client_batch_item_response_join_split_key_out_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_join_split_key_out_unique_identifier.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_join_split_key with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_join_split_key with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_join_split_key with valid output slots must not panic"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
                malformed_client_batch_item_response_handle.cast(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_get_attributes(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_get_attributes malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_get_attributes mismatched handle payload must be rejected"
        );
    }
    {
        let status = unsafe {
            client_abi::kmipkit_client_batch_item_response_get_attribute_list(
                malformed_client_batch_item_response_handle.cast(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_get_attribute_list malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_get_attribute_list mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_get_attribute_list_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_get_attribute_list_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let mut _kmipkit_client_batch_item_response_get_attribute_list_out_references =
            std::mem::MaybeUninit::<crate::kmipkit_attribute_reference_v1_t>::uninit();
        let mut _kmipkit_client_batch_item_response_get_attribute_list_out_count =
            std::mem::MaybeUninit::<u64>::uninit();
        let status = unsafe {
            client_abi::kmipkit_client_batch_item_response_get_attribute_list(
                malformed_client_batch_item_response_handle.cast(),
                _kmipkit_client_batch_item_response_get_attribute_list_out_present.as_mut_ptr(),
                _kmipkit_client_batch_item_response_get_attribute_list_out_unique_identifier
                    .as_mut_ptr(),
                _kmipkit_client_batch_item_response_get_attribute_list_out_references.as_mut_ptr(),
                0,
                _kmipkit_client_batch_item_response_get_attribute_list_out_count.as_mut_ptr(),
            )
        };
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_get_attribute_list with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_get_attribute_list with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_get_attribute_list with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_set_constraints_ttlv(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_set_constraints_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_set_constraints_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_validate(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_validate malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_validate mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_validate_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_validate_out_validity_indicator =
            std::mem::MaybeUninit::<u32>::uninit();
        let status = client_abi::kmipkit_client_batch_item_response_validate(
            malformed_client_batch_item_response_handle.cast(),
            _kmipkit_client_batch_item_response_validate_out_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_validate_out_validity_indicator.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_validate with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_validate with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_validate with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_validate_ttlv(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_validate_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_validate_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_export(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_export malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_export mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_unique_batch_item_id(
            malformed_batch_item_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_unique_batch_item_id malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_unique_batch_item_id mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_unique_batch_item_id_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_unique_batch_item_id_out_value =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let status = client_abi::kmipkit_client_batch_item_unique_batch_item_id(
            malformed_batch_item_handle.cast(),
            _kmipkit_client_batch_item_unique_batch_item_id_out_present.as_mut_ptr(),
            _kmipkit_client_batch_item_unique_batch_item_id_out_value.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_unique_batch_item_id with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_unique_batch_item_id with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_unique_batch_item_id with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_encrypt(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_encrypt malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_encrypt mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_encrypt_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_encrypt_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let mut _kmipkit_client_batch_item_response_encrypt_out_data_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_encrypt_out_data =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let mut _kmipkit_client_batch_item_response_encrypt_out_iv_counter_nonce_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_encrypt_out_iv_counter_nonce =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let mut _kmipkit_client_batch_item_response_encrypt_out_correlation_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_encrypt_out_correlation_value =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let mut
        _kmipkit_client_batch_item_response_encrypt_out_authenticated_encryption_tag_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_encrypt_out_authenticated_encryption_tag =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let status = client_abi::kmipkit_client_batch_item_response_encrypt(
            malformed_client_batch_item_response_handle.cast(),
            _kmipkit_client_batch_item_response_encrypt_out_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_encrypt_out_unique_identifier.as_mut_ptr(),
            _kmipkit_client_batch_item_response_encrypt_out_data_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_encrypt_out_data.as_mut_ptr(),
            _kmipkit_client_batch_item_response_encrypt_out_iv_counter_nonce_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_encrypt_out_iv_counter_nonce.as_mut_ptr(),
            _kmipkit_client_batch_item_response_encrypt_out_correlation_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_encrypt_out_correlation_value.as_mut_ptr(),
            _kmipkit_client_batch_item_response_encrypt_out_authenticated_encryption_tag_present
                .as_mut_ptr(),
            _kmipkit_client_batch_item_response_encrypt_out_authenticated_encryption_tag
                .as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_encrypt with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_encrypt with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_encrypt with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_decrypt(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_decrypt malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_decrypt mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_client_batch_item_response_decrypt_out_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_decrypt_out_unique_identifier =
            std::mem::MaybeUninit::<kmipkit_unique_identifier_v1_t>::uninit();
        let mut _kmipkit_client_batch_item_response_decrypt_out_data_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_decrypt_out_data =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let mut _kmipkit_client_batch_item_response_decrypt_out_correlation_present =
            std::mem::MaybeUninit::<u8>::uninit();
        let mut _kmipkit_client_batch_item_response_decrypt_out_correlation_value =
            std::mem::MaybeUninit::<crate::kmipkit_byte_span_v1_t>::uninit();
        let status = client_abi::kmipkit_client_batch_item_response_decrypt(
            malformed_client_batch_item_response_handle.cast(),
            _kmipkit_client_batch_item_response_decrypt_out_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_decrypt_out_unique_identifier.as_mut_ptr(),
            _kmipkit_client_batch_item_response_decrypt_out_data_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_decrypt_out_data.as_mut_ptr(),
            _kmipkit_client_batch_item_response_decrypt_out_correlation_present.as_mut_ptr(),
            _kmipkit_client_batch_item_response_decrypt_out_correlation_value.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_decrypt with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_decrypt with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_decrypt with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_mac_ttlv(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_mac_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_mac_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_mac_verify_ttlv(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_mac_verify_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_mac_verify_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_sign_ttlv(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_sign_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_sign_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_client_batch_item_response_signature_verify_ttlv(
            malformed_client_batch_item_response_handle.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_client_batch_item_response_signature_verify_ttlv malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_client_batch_item_response_signature_verify_ttlv mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_attribute_set_create(
            malformed_structure_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_attribute_set_create malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attribute_set_create mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_attribute_set_try_push(
            malformed_attribute_set_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_attribute_set_try_push malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attribute_set_try_push mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_attribute_set_as_items(
            malformed_attribute_set_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_attribute_set_as_items malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attribute_set_as_items mismatched handle payload must be rejected"
        );
    }
    {
        let status = client_abi::kmipkit_attribute_set_len(
            malformed_attribute_set_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_attribute_set_len malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attribute_set_len mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_attribute_set_len_out_count = std::mem::MaybeUninit::<u64>::uninit();
        let status = client_abi::kmipkit_attribute_set_len(
            malformed_attribute_set_handle.cast(),
            _kmipkit_attribute_set_len_out_count.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_attribute_set_len with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attribute_set_len with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attribute_set_len with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_attribute_set_is_empty(
            malformed_attribute_set_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_attribute_set_is_empty malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attribute_set_is_empty mismatched handle payload must be rejected"
        );
    }
    {
        let mut _kmipkit_attribute_set_is_empty_out_is_empty =
            std::mem::MaybeUninit::<u8>::uninit();
        let status = client_abi::kmipkit_attribute_set_is_empty(
            malformed_attribute_set_handle.cast(),
            _kmipkit_attribute_set_is_empty_out_is_empty.as_mut_ptr(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_attribute_set_is_empty with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attribute_set_is_empty with valid output slots must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attribute_set_is_empty with valid output slots must not panic"
        );
    }
    {
        let status = client_abi::kmipkit_attribute_set_into_items(
            malformed_attribute_set_handle.cast(),
            ptr::null_mut(),
        );
        assert_ne!(
            status, ERROR_INTERNAL,
            "kmipkit_attribute_set_into_items malformed handle input must not panic"
        );
        assert_ne!(
            status, SUCCESS,
            "kmipkit_attribute_set_into_items mismatched handle payload must be rejected"
        );
    }

    release(
        malformed_add_attribute_request_handle,
        Kind::AddAttributeRequest,
    );
    release(
        malformed_adjust_attribute_request_handle,
        Kind::AdjustAttributeRequest,
    );
    release(
        malformed_attestation_credential_handle,
        Kind::AttestationCredential,
    );
    release(malformed_attribute_set_handle, Kind::AttributeSet);
    release(malformed_authentication_handle, Kind::Authentication);
    release(malformed_batch_item_handle, Kind::BatchItem);
    release(malformed_cancel_request_handle, Kind::CancelRequest);
    release(malformed_client_handle, Kind::Client);
    release(
        malformed_client_async_operation_outcome_handle,
        Kind::ClientAsyncOperationOutcome,
    );
    release(malformed_client_batch_handle, Kind::ClientBatch);
    release(
        malformed_client_batch_item_response_handle,
        Kind::ClientBatchItemResponse,
    );
    release(
        malformed_client_batch_response_handle,
        Kind::ClientBatchResponse,
    );
    release(malformed_codec_limits_handle, Kind::CodecLimits);
    release(malformed_credential_handle, Kind::Credential);
    release(malformed_credential_value_handle, Kind::CredentialValue);
    release(malformed_credential_view_handle, Kind::CredentialView);
    release(
        malformed_delete_attribute_request_handle,
        Kind::DeleteAttributeRequest,
    );
    release(
        malformed_get_attribute_list_request_handle,
        Kind::GetAttributeListRequest,
    );
    release(
        malformed_hashed_password_credential_handle,
        Kind::HashedPasswordCredential,
    );
    release(
        malformed_hashed_password_timestamp_sequence_handle,
        Kind::HashedPasswordTimestampSequence,
    );
    release(malformed_information_handle, Kind::Information);
    release(
        malformed_message_validation_error_handle,
        Kind::MessageValidationError,
    );
    release(
        malformed_modify_attribute_request_handle,
        Kind::ModifyAttributeRequest,
    );
    release(malformed_poll_request_handle, Kind::PollRequest);
    release(malformed_process_request_handle, Kind::ProcessRequest);
    release(malformed_profile_readiness_handle, Kind::ProfileReadiness);
    release(
        malformed_profile_validation_error_handle,
        Kind::ProfileValidationError,
    );
    release(
        malformed_profile_validation_result_handle,
        Kind::ProfileValidationResult,
    );
    release(
        malformed_query_async_requests_request_handle,
        Kind::QueryAsyncRequestsRequest,
    );
    release(malformed_register_request_handle, Kind::RegisterRequest);
    release(malformed_registry_handle, Kind::Registry);
    release(
        malformed_request_batch_item_view_handle,
        Kind::RequestBatchItemView,
    );
    release(
        malformed_request_header_view_handle,
        Kind::RequestHeaderView,
    );
    release(malformed_request_message_handle, Kind::RequestMessage);
    release(
        malformed_response_batch_item_view_handle,
        Kind::ResponseBatchItemView,
    );
    release(
        malformed_response_header_view_handle,
        Kind::ResponseHeaderView,
    );
    release(malformed_response_message_handle, Kind::ResponseMessage);
    release(malformed_revoke_request_handle, Kind::RevokeRequest);
    release(
        malformed_set_attribute_request_handle,
        Kind::SetAttributeRequest,
    );
    release(malformed_structure_handle, Kind::Structure);
    release(
        malformed_usage_allocation_context_handle,
        Kind::UsageAllocationContext,
    );
    release(
        malformed_username_and_password_credential_handle,
        Kind::UsernameAndPasswordCredential,
    );
    release(malformed_validated_handle, Kind::Validated);
}
