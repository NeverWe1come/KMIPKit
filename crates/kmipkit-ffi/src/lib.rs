//! The stable C application binary interface for `KMIPKit`.
#![deny(unsafe_code)]
#[allow(unsafe_code)]
mod extension_registry;

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
#[doc(hidden)]
#[allow(dead_code, clippy::struct_field_names)]
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
