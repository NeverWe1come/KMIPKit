//! Data-only models for registered KMIP vendor extensions.
//!
//! These types describe local extension identities, compatibility, schemas,
//! and metadata. They do not add a transport or an alternate message writer.

mod definition;
mod information;
mod limits;
mod schema;
mod value;

pub use definition::{
    Compatibility, Discriminator, ExtensionDefinition, ExtensionDefinitionAccounting,
    ExtensionIdentity, TtlvPath, accounting, clone_extension_definition,
    clone_extension_discriminator, compatibility, discriminator, extension_definition,
    extension_identity, identity, scalar_value_fingerprint, ttlv_path, validate_schema_limits,
    validate_schema_limits_with_aggregate, with_child_tag,
};
pub use information::{
    ExtensionInformation, extension_information, information, to_ttlv, with_attribute,
    with_description, with_enumeration, with_information, with_parent_structure_tag, with_tag,
    with_type,
};
pub use limits::{ExtensionRegistryLimits, defaults, with_values};
pub use schema::{
    ExtensionChildRule, ExtensionOrderConstraint, ExtensionSchema, extension_order_constraint,
    optional, repeated, required, scalar, structure, with_allowed_bit_mask,
    with_allowed_enumeration, with_maximum_length, with_minimum_length, with_required_bit_mask,
    with_signed_range, with_unsigned_range,
};
pub use value::{
    SchemaValidationOutcome, ValidatedExtensionValue, generic_value, validate,
    validate_schema_only, validated_extension_value_identity,
};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};

pub(crate) fn categorized_error(kind: ProtocolErrorKind) -> ProtocolError {
    ProtocolError::categorized(kind, ProtocolCauseCategory::InvalidValue)
}
