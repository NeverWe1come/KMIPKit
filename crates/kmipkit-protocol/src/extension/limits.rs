//! Immutable limits for extension metadata and schema declarations.

use crate::ProtocolErrorKind;

use super::categorized_error;

const HARD_MAX_DEFINITIONS: u64 = 1_024;
const HARD_MAX_SCHEMA_NODES: u64 = 100_000;
const HARD_MAX_CHILD_RULES: u64 = 4_096;
const HARD_MAX_FIELD_TEXT_BYTES: u64 = 4_096;
const HARD_MAX_REGISTRY_TEXT_BYTES: u64 = 16 * 1024 * 1024;
const HARD_MAX_DISCRIMINATOR_BYTES: u64 = 4_096;
const HARD_MAX_TOTAL_DISCRIMINATOR_BYTES: u64 = 16 * 1024 * 1024;
const HARD_MAX_CONSTRAINT_MEMBERS: u64 = 4_096;
const HARD_MAX_TOTAL_CONSTRAINT_MEMBERS: u64 = 100_000;
const HARD_MAX_PAYLOAD_INDEX_RECORDS: u64 = 200_000;
const HARD_MAX_LOOKUP_COMPARISONS: u64 = 4_194_304;
const HARD_MAX_DEPTH: u64 = 64;

/// Immutable resource limits applied to one extension registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::struct_field_names)] // Names match the stable cross-language API manifest.
pub struct ExtensionRegistryLimits {
    pub(crate) max_definitions: u64,
    pub(crate) max_schema_nodes: u64,
    pub(crate) max_child_rules_per_structure: u64,
    pub(crate) max_text_bytes_per_field: u64,
    pub(crate) max_registry_text_bytes: u64,
    pub(crate) max_discriminator_scalar_bytes: u64,
    pub(crate) max_total_discriminator_scalar_bytes: u64,
    pub(crate) max_constraint_members_per_rule: u64,
    pub(crate) max_total_constraint_members: u64,
    pub(crate) max_payload_index_records: u64,
    pub(crate) max_lookup_comparisons: u64,
    pub(crate) max_depth: u64,
}

impl ExtensionRegistryLimits {
    /// Returns the default maximum definitions in one registry.
    #[must_use]
    pub const fn max_definitions(&self) -> u64 {
        self.max_definitions
    }

    /// Returns the default maximum aggregate schema node count.
    #[must_use]
    pub const fn max_schema_nodes(&self) -> u64 {
        self.max_schema_nodes
    }

    /// Returns the maximum child rules in one Structure schema.
    #[must_use]
    pub const fn max_child_rules_per_structure(&self) -> u64 {
        self.max_child_rules_per_structure
    }

    /// Returns the maximum UTF-8 bytes in one metadata text field.
    #[must_use]
    pub const fn max_text_bytes_per_field(&self) -> u64 {
        self.max_text_bytes_per_field
    }

    /// Returns the maximum aggregate identity and metadata text bytes.
    #[must_use]
    pub const fn max_registry_text_bytes(&self) -> u64 {
        self.max_registry_text_bytes
    }

    /// Returns the maximum bytes in one discriminator scalar.
    #[must_use]
    pub const fn max_discriminator_scalar_bytes(&self) -> u64 {
        self.max_discriminator_scalar_bytes
    }

    /// Returns the maximum aggregate discriminator scalar bytes.
    #[must_use]
    pub const fn max_total_discriminator_scalar_bytes(&self) -> u64 {
        self.max_total_discriminator_scalar_bytes
    }

    /// Returns the maximum constraint members in one schema rule.
    #[must_use]
    pub const fn max_constraint_members_per_rule(&self) -> u64 {
        self.max_constraint_members_per_rule
    }

    /// Returns the maximum aggregate schema constraint members.
    #[must_use]
    pub const fn max_total_constraint_members(&self) -> u64 {
        self.max_total_constraint_members
    }

    /// Returns the maximum temporary payload-index records per lookup.
    #[must_use]
    pub const fn max_payload_index_records(&self) -> u64 {
        self.max_payload_index_records
    }

    /// Returns the maximum discriminator tag comparisons per lookup.
    #[must_use]
    pub const fn max_lookup_comparisons(&self) -> u64 {
        self.max_lookup_comparisons
    }

    /// Returns the maximum schema and discriminator path depth.
    #[must_use]
    pub const fn max_depth(&self) -> u64 {
        self.max_depth
    }
}

/// Returns the registry resource limits specified as `KMIPKit` defaults.
#[must_use]
pub const fn defaults() -> ExtensionRegistryLimits {
    ExtensionRegistryLimits {
        max_definitions: 256,
        max_schema_nodes: 16_384,
        max_child_rules_per_structure: 256,
        max_text_bytes_per_field: 4_096,
        max_registry_text_bytes: 1_048_576,
        max_discriminator_scalar_bytes: 4_096,
        max_total_discriminator_scalar_bytes: 1_048_576,
        max_constraint_members_per_rule: 256,
        max_total_constraint_members: 16_384,
        max_payload_index_records: 200_000,
        max_lookup_comparisons: 1_048_576,
        max_depth: 64,
    }
}

/// Constructs immutable registry limits without exceeding their hard maxima.
///
/// # Errors
///
/// Returns `ResourceLimit` if any supplied value exceeds its specified hard
/// maximum. Zero and values below the defaults are valid limits.
#[allow(clippy::too_many_arguments)] // Mirrors the stable 12-field API manifest constructor.
pub fn with_values(
    max_definitions: u64,
    max_schema_nodes: u64,
    max_child_rules_per_structure: u64,
    max_text_bytes_per_field: u64,
    max_registry_text_bytes: u64,
    max_discriminator_scalar_bytes: u64,
    max_total_discriminator_scalar_bytes: u64,
    max_constraint_members_per_rule: u64,
    max_total_constraint_members: u64,
    max_payload_index_records: u64,
    max_lookup_comparisons: u64,
    max_depth: u64,
) -> Result<ExtensionRegistryLimits, crate::ProtocolError> {
    let requested = [
        max_definitions,
        max_schema_nodes,
        max_child_rules_per_structure,
        max_text_bytes_per_field,
        max_registry_text_bytes,
        max_discriminator_scalar_bytes,
        max_total_discriminator_scalar_bytes,
        max_constraint_members_per_rule,
        max_total_constraint_members,
        max_payload_index_records,
        max_lookup_comparisons,
        max_depth,
    ];
    let maximum = [
        HARD_MAX_DEFINITIONS,
        HARD_MAX_SCHEMA_NODES,
        HARD_MAX_CHILD_RULES,
        HARD_MAX_FIELD_TEXT_BYTES,
        HARD_MAX_REGISTRY_TEXT_BYTES,
        HARD_MAX_DISCRIMINATOR_BYTES,
        HARD_MAX_TOTAL_DISCRIMINATOR_BYTES,
        HARD_MAX_CONSTRAINT_MEMBERS,
        HARD_MAX_TOTAL_CONSTRAINT_MEMBERS,
        HARD_MAX_PAYLOAD_INDEX_RECORDS,
        HARD_MAX_LOOKUP_COMPARISONS,
        HARD_MAX_DEPTH,
    ];

    if requested
        .iter()
        .zip(maximum)
        .any(|(value, hard_maximum)| *value > hard_maximum)
    {
        return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
    }

    Ok(ExtensionRegistryLimits {
        max_definitions,
        max_schema_nodes,
        max_child_rules_per_structure,
        max_text_bytes_per_field,
        max_registry_text_bytes,
        max_discriminator_scalar_bytes,
        max_total_discriminator_scalar_bytes,
        max_constraint_members_per_rule,
        max_total_constraint_members,
        max_payload_index_records,
        max_lookup_comparisons,
        max_depth,
    })
}
