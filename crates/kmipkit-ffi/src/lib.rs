//! The stable C application binary interface for `KMIPKit`.
#![deny(unsafe_code)]
#[allow(unsafe_code)]
mod extension_registry;
#[allow(dead_code)]
mod public_api_generated;

#[cfg(test)]
#[allow(unsafe_code)]
#[path = "../tests/support/profile_readiness_identifier_parity_tests.rs"]
mod profile_readiness_identifier_parity_tests;

#[repr(C)]
pub struct kmipkit_extension_identity_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_extension_compatibility_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_ttlv_path_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_extension_discriminator_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_extension_schema_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_extension_information_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_extension_definition_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_client_extension_registry_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_client_configuration_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_transport_config_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_client_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_client_batch_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_client_batch_response_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_client_batch_item_response_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_client_async_operation_outcome_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_profile_validation_error_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_profile_validation_result_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_profile_readiness_t {
    _private: [u8; 0],
}

/// Common result fields for one explicit asynchronous follow-up operation.
#[repr(C)]
pub struct kmipkit_client_async_operation_metadata_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub operation: u32,
    pub outcome_kind: u32,
    pub result_status: u32,
    pub has_result_reason: u32,
    pub result_reason: u32,
    pub has_cancellation_result: u32,
    pub cancellation_result: u32,
    pub has_result_message: u32,
    pub reserved: u32,
    pub result_message_data: *const u8,
    pub result_message_length: u64,
}
#[repr(C)]
pub struct kmipkit_usage_allocation_context_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_split_key_reference_t {
    _private: [u8; 0],
}

/// Common operation outcome fields, versioned independently from opaque handles.
#[repr(C)]
pub struct kmipkit_client_batch_item_metadata_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    /// `1` for a completed outcome and `2` for a Pending outcome.
    pub outcome_kind: u32,
    pub result_status: u32,
    pub has_result_reason: u32,
    pub result_reason: u32,
}

/// Version 2 adds the source operation and exact optional Result Message while
/// leaving the version 1 metadata record unchanged.
#[repr(C)]
pub struct kmipkit_client_batch_item_metadata_v2_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub operation: u32,
    /// `1` for a completed outcome and `2` for a Pending outcome.
    pub outcome_kind: u32,
    pub result_status: u32,
    pub has_result_reason: u32,
    pub result_reason: u32,
    pub has_result_message: u32,
    pub reserved: u32,
    /// Borrowed UTF-8 bytes, valid while the item-response handle remains open.
    pub result_message_data: *const u8,
    pub result_message_length: u64,
}

/// One exact signed KMIP Protocol Version Major/Minor pair.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct kmipkit_protocol_version_v1_t {
    pub major: i32,
    pub minor: i32,
}

#[repr(C)]
pub struct kmipkit_validated_extension_value_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_registered_extension_value_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_client_request_message_extension_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_ttlv_structure_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_attribute_set_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_request_message_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_request_header_view_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_request_batch_item_view_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_response_message_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_response_header_view_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_response_batch_item_view_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_message_extension_view_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_message_validation_error_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_client_batch_item_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_register_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_add_attribute_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_adjust_attribute_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_delete_attribute_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_modify_attribute_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_set_attribute_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_get_attributes_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_get_attribute_list_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_cancel_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_poll_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_process_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_query_async_requests_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_revocation_reason_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_revoke_request_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_nonce_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_secret_bytes_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_secret_text_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_credential_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_credential_value_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_authentication_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_cryptographic_parameters_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_credential_view_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_attestation_credential_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_device_credential_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_username_and_password_credential_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_hashed_password_credential_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_hashed_password_timestamp_sequence_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_extension_child_rule_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_extension_order_constraint_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_ttlv_value_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_extension_recognition_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_raw_tag_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_tag_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_ttlv_item_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_codec_limits_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_ttlv_structure_view_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_ttlv_item_view_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_ttlv_value_view_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct kmipkit_extension_registry_limits_t {
    pub max_definitions: u64,
    pub max_schema_nodes: u64,
    pub max_child_rules_per_structure: u64,
    pub max_text_bytes_per_field: u64,
    pub max_registry_text_bytes: u64,
    pub max_discriminator_scalar_bytes: u64,
    pub max_total_discriminator_scalar_bytes: u64,
    pub max_constraint_members_per_rule: u64,
    pub max_total_constraint_members: u64,
    pub max_payload_index_records: u64,
    pub max_lookup_comparisons: u64,
    pub max_depth: u64,
}

#[repr(C)]
#[allow(clippy::struct_field_names)]
pub struct kmipkit_extension_registry_limits_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub max_definitions: u64,
    pub max_schema_nodes: u64,
    pub max_child_rules_per_structure: u64,
    pub max_text_bytes_per_field: u64,
    pub max_registry_text_bytes: u64,
    pub max_discriminator_scalar_bytes: u64,
    pub max_total_discriminator_scalar_bytes: u64,
    pub max_constraint_members_per_rule: u64,
    pub max_total_constraint_members: u64,
    pub max_payload_index_records: u64,
    pub max_lookup_comparisons: u64,
    pub max_depth: u64,
}

/// A non-owning byte span supplied for one ABI call.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct kmipkit_byte_span_v1_t {
    pub data: *const u8,
    pub length: u64,
}

/// Versioned, redacted profile validation details. UTF-8 pointer/length pairs remain valid while
/// the owning `kmipkit_profile_validation_error_t` handle remains open.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct kmipkit_profile_validation_error_view_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    /// 0 for no source, 1 for a clause ID, or 2 for a source-gap ID and locator.
    pub source_kind: u32,
    /// 1 when `batch_item_index` contains a zero-based request item index; otherwise 0.
    pub batch_item_index_present: u32,
    pub batch_item_index: u64,
    pub error_code_data: *const u8,
    pub error_code_length: u64,
    pub profile_id_data: *const u8,
    pub profile_id_length: u64,
    pub requirement_id_data: *const u8,
    pub requirement_id_length: u64,
    /// Clause ID for source kind 1, gap ID for source kind 2, otherwise an empty span.
    pub source_id_data: *const u8,
    pub source_id_length: u64,
    /// Catalogued locator for source kind 2, otherwise an empty span.
    pub source_locator_data: *const u8,
    pub source_locator_length: u64,
}

/// One local evidence record accepted by the profile readiness evaluator.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct kmipkit_profile_evidence_record_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub requirement_id: kmipkit_byte_span_v1_t,
    /// 0 reports failure and 1 reports pass. Other values are rejected.
    pub passed: u8,
    pub reserved_0: u8,
    pub reserved_1: u8,
    pub reserved_2: u8,
    pub reserved_3: u8,
    pub reserved_4: u8,
    pub reserved_5: u8,
    pub reserved_6: u8,
}

/// Versioned snapshot view. Its spans borrow from the readiness handle until release.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct kmipkit_profile_readiness_view_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub profile_id: kmipkit_byte_span_v1_t,
    pub effective_state: kmipkit_byte_span_v1_t,
    pub target_selected: u8,
    pub runtime_validation_available: u8,
    pub requirement_records_complete: u8,
    pub published_claim: u8,
    pub reserved_0: u8,
    pub reserved_1: u8,
    pub reserved_2: u8,
    pub reserved_3: u8,
    pub blocker_count: u64,
}

/// One redacted readiness blocker with presence-tagged stable identifiers.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct kmipkit_profile_readiness_blocker_view_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub kind: kmipkit_byte_span_v1_t,
    pub message: kmipkit_byte_span_v1_t,
    pub requirement_id_present: u8,
    pub profile_id_present: u8,
    pub test_id_present: u8,
    pub source_clause_id_present: u8,
    pub reserved_0: u8,
    pub reserved_1: u8,
    pub reserved_2: u8,
    pub reserved_3: u8,
    pub requirement_id: kmipkit_byte_span_v1_t,
    pub profile_id: kmipkit_byte_span_v1_t,
    pub test_id: kmipkit_byte_span_v1_t,
    pub source_clause_id: kmipkit_byte_span_v1_t,
}

/// Generic versioned view over one generated profile catalog row or nested string relation.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct kmipkit_profile_catalog_row_view_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub row_kind: u32,
    pub reserved: u32,
    pub profile_id: kmipkit_byte_span_v1_t,
    pub name: kmipkit_byte_span_v1_t,
    pub applicability: kmipkit_byte_span_v1_t,
    pub claim_state: kmipkit_byte_span_v1_t,
    pub selected_target: u8,
    pub has_accepted_deviation: u8,
    pub source_keyword_present: u8,
    pub source_force_present: u8,
    pub source_gap_id_present: u8,
    pub source_locator_present: u8,
    pub reserved_flag_0: u8,
    pub reserved_flag_1: u8,
    pub requirement_id: kmipkit_byte_span_v1_t,
    pub source_clause_id: kmipkit_byte_span_v1_t,
    pub source_keyword: kmipkit_byte_span_v1_t,
    pub source_force: kmipkit_byte_span_v1_t,
    pub source_gap_id: kmipkit_byte_span_v1_t,
    pub source_locator: kmipkit_byte_span_v1_t,
    pub verification_category: kmipkit_byte_span_v1_t,
    pub scope_state: kmipkit_byte_span_v1_t,
    pub test_id: kmipkit_byte_span_v1_t,
    pub official_case_id: kmipkit_byte_span_v1_t,
    pub mandatory_status: kmipkit_byte_span_v1_t,
    pub fixture_availability: kmipkit_byte_span_v1_t,
    /// Text payload for nested string relations; empty for compound relation rows.
    pub text_value: kmipkit_byte_span_v1_t,
}

/// Exact wire value for the three Unique Identifier forms accepted by KMIP
/// §§6.1.8–6.1.10. Text bytes are borrowed on input and output.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct kmipkit_unique_identifier_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    /// 1 `TextString`, 2 `Enumeration`, or 3 `Integer`.
    pub kind: u32,
    pub reserved: u32,
    pub text_data: *const u8,
    pub text_length: u64,
    pub enumeration_value: u32,
    pub integer_value: i32,
}

/// One lossless KMIP Attribute Reference returned by a successful operation.
/// Name-form spans are borrowed while the response handle remains open.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct kmipkit_attribute_reference_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub kind: u32,
    pub reserved: u32,
    pub tag_value: u32,
    pub reserved2: u32,
    pub vendor_identification_data: *const u8,
    pub vendor_identification_length: u64,
    pub attribute_name_data: *const u8,
    pub attribute_name_length: u64,
}

pub const KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG: u32 = 1;
pub const KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME: u32 = 2;

pub const KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING: u32 = 1;
pub const KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION: u32 = 2;
pub const KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER: u32 = 3;

/// One explicitly encoded transport certificate, key, or revocation input.
#[repr(C)]
pub struct kmipkit_transport_input_v1_t {
    pub source_kind: u32,
    pub encoding: u32,
    pub data: kmipkit_byte_span_v1_t,
}

/// One connect, write, read, or total timeout setting.
#[repr(C)]
pub struct kmipkit_timeout_limit_v1_t {
    pub kind: u32,
    pub reserved: u32,
    pub seconds: u64,
    pub nanoseconds: u32,
    pub reserved2: u32,
}

/// Versioned per-exchange timeout overrides for `kmipkit_client_execute_with_options_v1`.
/// A timeout with kind zero and zero duration inherits the client's configured policy.
#[repr(C)]
pub struct kmipkit_request_options_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub connect_timeout: kmipkit_timeout_limit_v1_t,
    pub write_timeout: kmipkit_timeout_limit_v1_t,
    pub read_timeout: kmipkit_timeout_limit_v1_t,
    pub total_timeout: kmipkit_timeout_limit_v1_t,
}

/// Borrowed, explicit profile selection for one client exchange.
#[repr(C)]
pub struct kmipkit_profile_validation_selection_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub profile_id_data: *const u8,
    pub profile_id_data_length: u64,
    pub profile_id_offsets: *const u64,
    pub profile_id_lengths: *const u64,
    pub profile_id_count: u64,
}

/// Versioned input for constructing KMIP 2.1 Cryptographic Parameters.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct kmipkit_cryptographic_parameters_fields_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub block_cipher_mode_present: u8,
    pub block_cipher_mode: u32,
    pub padding_method_present: u8,
    pub padding_method: u32,
    pub hashing_algorithm_present: u8,
    pub hashing_algorithm: u32,
    pub key_role_type_present: u8,
    pub key_role_type: u32,
    pub digital_signature_algorithm_present: u8,
    pub digital_signature_algorithm: u32,
    pub cryptographic_algorithm_present: u8,
    pub cryptographic_algorithm: u32,
    pub random_iv_present: u8,
    pub random_iv: u8,
    pub iv_length_present: u8,
    pub iv_length: i32,
    pub tag_length_present: u8,
    pub tag_length: i32,
    pub fixed_field_length_present: u8,
    pub fixed_field_length: i32,
    pub invocation_field_length_present: u8,
    pub invocation_field_length: i32,
    pub counter_length_present: u8,
    pub counter_length: i32,
    pub initial_counter_value_present: u8,
    pub initial_counter_value: i32,
    pub salt_length_present: u8,
    pub salt_length: i32,
    pub mask_generator_present: u8,
    pub mask_generator: u32,
    pub mask_generator_hashing_algorithm_present: u8,
    pub mask_generator_hashing_algorithm: u32,
    pub p_source_present: u8,
    pub p_source: *const u8,
    pub p_source_length: u64,
    pub trailer_field_present: u8,
    pub trailer_field: i32,
}

/// Caller-owned object state for one request batch item.
#[repr(C)]
pub struct kmipkit_profile_object_state_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub batch_item_index: u64,
    pub unique_identifier_data: *const u8,
    pub unique_identifier_length: u64,
    pub known_fields: u32,
    pub boolean_values: u32,
    pub usage_limits_total: i64,
    pub usage_limits_count: i64,
}

/// Borrowed caller-owned state supplied only for one profile-validated exchange.
#[repr(C)]
pub struct kmipkit_profile_validation_context_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub object_states: *const kmipkit_profile_object_state_v1_t,
    pub object_state_count: u64,
}

/// Versioned, payload-free details for one generic TTLV decoder failure.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct kmipkit_ttlv_decode_error_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub kind: u32,
    pub byte_offset: u64,
}

/// Versioned, payload-free details for one credential validation failure.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct kmipkit_credential_validation_error_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    /// Stable category code, or zero when the last failure was not credential validation.
    pub kind: u32,
}

/// Versioned production transport inputs. All spans are borrowed only for the
/// constructor call; the validated configuration owns its parsed material.
#[repr(C)]
pub struct kmipkit_transport_config_v1_t {
    pub struct_size: u32,
    pub abi_version: u32,
    pub endpoint_kind: u32,
    pub trust_kind: u32,
    pub flags: u32,
    pub reserved: u32,
    pub endpoint: kmipkit_byte_span_v1_t,
    pub endpoint_port: u16,
    pub reserved2: u16,
    pub client_certificates: *const kmipkit_transport_input_v1_t,
    pub client_certificate_count: u64,
    pub client_private_key: kmipkit_transport_input_v1_t,
    pub trust_certificates: *const kmipkit_transport_input_v1_t,
    pub trust_certificate_count: u64,
    pub revocation_lists: *const kmipkit_transport_input_v1_t,
    pub revocation_list_count: u64,
    pub tls_server_name: kmipkit_byte_span_v1_t,
    pub target_uri: kmipkit_byte_span_v1_t,
    pub connect_timeout: kmipkit_timeout_limit_v1_t,
    pub write_timeout: kmipkit_timeout_limit_v1_t,
    pub read_timeout: kmipkit_timeout_limit_v1_t,
    pub total_timeout: kmipkit_timeout_limit_v1_t,
    pub max_request_bytes: u64,
}
