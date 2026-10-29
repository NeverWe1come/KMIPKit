//! The stable C application binary interface for `KMIPKit`.
#![deny(unsafe_code)]
#[allow(unsafe_code)]
mod extension_registry;
#[allow(dead_code)]
mod public_api_generated;

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
pub struct kmipkit_client_batch_item_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct kmipkit_register_request_t {
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
#[derive(Clone, Copy)]
pub struct kmipkit_byte_span_v1_t {
    pub data: *const u8,
    pub length: u64,
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
