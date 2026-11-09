//! Pre-send validation gate enforcing version compatibility before network transport.

use std::error::Error;
use std::fmt;

use crate::{ProtocolVersion, VersionSpan};

/// A local error emitted when a requested operation, attribute, or structure
/// is incompatible with the effective KMIP protocol version.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionGateError {
    /// The protocol version is not catalogued or supported by the library.
    UnsupportedProtocolVersion(ProtocolVersion),
    /// The requested operation is not supported in the negotiated version.
    OperationNotSupportedInVersion {
        /// The KMIP operation enumeration.
        operation: u32,
        /// The effective negotiated or forced protocol version.
        negotiated: ProtocolVersion,
        /// The earliest protocol version where this operation was introduced.
        introduced: ProtocolVersion,
    },
    /// The operation value is outside the vendor-extension allocation and is
    /// not assigned by the selected KMIP edition.
    UnassignedOperationInVersion {
        /// The unassigned KMIP operation enumeration.
        operation: u32,
        /// The effective negotiated or forced protocol version.
        version: ProtocolVersion,
    },
    /// A requested attribute or tag is not supported in the negotiated version.
    AttributeNotSupportedInVersion {
        /// The raw TTLV tag of the attribute or field.
        tag: u32,
        /// The effective negotiated or forced protocol version.
        negotiated: ProtocolVersion,
        /// The earliest protocol version where this attribute was introduced.
        introduced: ProtocolVersion,
    },
    /// An attribute mutation (such as `SetAttribute`) is prohibited in the negotiated version.
    AttributeMutationProhibitedInVersion {
        /// The raw TTLV tag of the attribute.
        tag: u32,
        /// The effective protocol version.
        version: ProtocolVersion,
    },
}

impl fmt::Display for VersionGateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProtocolVersion(v) => {
                write!(
                    formatter,
                    "KMIP protocol version {v} is not catalogued or supported"
                )
            }
            Self::OperationNotSupportedInVersion {
                operation,
                negotiated,
                introduced,
            } => {
                write!(
                    formatter,
                    "operation 0x{operation:08X} requires KMIP {introduced} or newer, but session is {negotiated}"
                )
            }
            Self::UnassignedOperationInVersion { operation, version } => {
                write!(
                    formatter,
                    "operation 0x{operation:08X} is not assigned by KMIP {version}"
                )
            }
            Self::AttributeNotSupportedInVersion {
                tag,
                negotiated,
                introduced,
            } => {
                write!(
                    formatter,
                    "tag 0x{tag:08X} requires KMIP {introduced} or newer, but session is {negotiated}"
                )
            }
            Self::AttributeMutationProhibitedInVersion { tag, version } => {
                write!(
                    formatter,
                    "mutation of attribute tag 0x{tag:08X} is prohibited under KMIP {version} (rotate-only)"
                )
            }
        }
    }
}

impl Error for VersionGateError {}

/// Pre-send validation gate enforcing version spans against compiled catalog tables.
pub struct VersionGate;

impl VersionGate {
    /// Returns the normative version span for a known KMIP client-to-server operation.
    #[must_use]
    pub fn operation_span(operation: u32) -> Option<VersionSpan> {
        use crate::named_values_generated::values::{
            enumeration::operation as kmip_2_1,
            versions::{v2_1, v3_0},
        };

        if v3_0::enumeration::operation::name(operation).is_some() {
            Some(VersionSpan::since(ProtocolVersion::V3_0))
        } else if v2_1::enumeration::operation::name(operation).is_some() {
            Some(VersionSpan::since(ProtocolVersion::V2_1))
        } else if kmip_2_1::name(operation).is_some() {
            Some(VersionSpan::since(ProtocolVersion::V2_0))
        } else {
            None
        }
    }

    /// Validates whether `operation` is permitted to be executed under `version`.
    ///
    /// # Errors
    ///
    /// Returns [`VersionGateError`] if `version` is not catalogued, or if `operation`
    /// was introduced in a newer version or removed in `version`.
    pub fn validate_operation(
        version: ProtocolVersion,
        operation: u32,
    ) -> Result<(), VersionGateError> {
        if !version.is_catalogued() {
            return Err(VersionGateError::UnsupportedProtocolVersion(version));
        }

        match Self::operation_span(operation) {
            Some(span) if !span.is_supported_in(version) => {
                return Err(VersionGateError::OperationNotSupportedInVersion {
                    operation,
                    negotiated: version,
                    introduced: span.introduced,
                });
            }
            Some(_) => {}
            None if is_operation_extension(operation) => {}
            None => {
                return Err(VersionGateError::UnassignedOperationInVersion { operation, version });
            }
        }

        Ok(())
    }

    /// Returns the normative version span for a known KMIP tag.
    #[must_use]
    pub fn tag_span(tag: u32) -> Option<VersionSpan> {
        match tag {
            // KMIP 2.1 Rotate* attributes (§§4.48-4.54) and Constraints (§4.25)
            0x0042_014A..=0x0042_0150 | 0x0042_0152 => {
                Some(VersionSpan::since(ProtocolVersion::V2_1))
            }
            _ if KMIP_3_0_TAG_ASSIGNMENTS.binary_search(&tag).is_ok() => {
                Some(VersionSpan::since(ProtocolVersion::V3_0))
            }
            _ => None,
        }
    }

    /// Validates whether a TTLV tag is permitted to be sent under `version`.
    ///
    /// # Errors
    ///
    /// Returns [`VersionGateError`] if `version` is not catalogued or the tag is out of span.
    pub fn validate_tag(version: ProtocolVersion, tag: u32) -> Result<(), VersionGateError> {
        if !version.is_catalogued() {
            return Err(VersionGateError::UnsupportedProtocolVersion(version));
        }

        if let Some(span) = Self::tag_span(tag)
            && !span.is_supported_in(version)
        {
            return Err(VersionGateError::AttributeNotSupportedInVersion {
                tag,
                negotiated: version,
                introduced: span.introduced,
            });
        }

        Ok(())
    }

    /// Validates whether mutating the attribute represented by `tag` is permitted under `version`.
    ///
    /// In OASIS KMIP 2.0, the seven core cryptographic attributes are rotate-only and cannot be mutated
    /// via `SetAttribute`.
    ///
    /// # Errors
    ///
    /// Returns [`VersionGateError::AttributeMutationProhibitedInVersion`] if `tag` is a rotate-only
    /// attribute under KMIP 2.0.
    pub fn validate_attribute_mutation(
        version: ProtocolVersion,
        tag: u32,
    ) -> Result<(), VersionGateError> {
        if !version.is_catalogued() {
            return Err(VersionGateError::UnsupportedProtocolVersion(version));
        }

        if version == ProtocolVersion::V2_0 {
            match tag {
                0x0042_0028 // Cryptographic Algorithm
                | 0x0042_002A // Cryptographic Length
                | 0x0042_002B // Cryptographic Parameters
                | 0x0042_002C // Cryptographic Usage Mask
                | 0x0042_0032 // Digest
                | 0x0042_00BF // Key Value Present
                | 0x0042_0044 // Key Format Type
                => {
                    return Err(VersionGateError::AttributeMutationProhibitedInVersion {
                        tag,
                        version,
                    });
                }
                _ => {}
            }
        }

        Ok(())
    }
}

// OASIS KMIP Enumeration extensions use the allocated 8XXXXXXX range. Keep
// this exact extension allocation separate from the assigned standard table
// values so numeric gaps never become standard operations.
const fn is_operation_extension(operation: u32) -> bool {
    matches!(operation, 0x8000_0000..=0x8FFF_FFFF)
}

// Exact assignments from KMIP 3.0 §11.58, Table 605. Do not replace this list
// with an allocation range: adjacent tags are valid in KMIP 2.0/2.1.
const KMIP_3_0_TAG_ASSIGNMENTS: &[u32] = &[
    0x0042_0190,
    0x0042_0191,
    0x0042_0192,
    0x0042_0193,
    0x0042_0194,
    0x0042_0195,
    0x0042_0196,
    0x0042_0197,
    0x0042_0198,
    0x0042_0199,
    0x0042_019A,
    0x0042_019B,
    0x0042_019C,
    0x0042_019D,
    0x0042_019E,
    0x0042_019F,
    0x0042_01A0,
    0x0042_01A1,
    0x0042_01A2,
    0x0042_01A3,
    0x0042_01A4,
    0x0042_01A5,
    0x0042_01A6,
    0x0042_01A7,
    0x0042_01A8,
    0x0042_01A9,
    0x0042_01AA,
    0x0042_01AB,
    0x0042_01AC,
    0x0042_01AD,
    0x0042_01AE,
    0x0042_01AF,
    0x0042_01B0,
    0x0042_01B1,
    0x0042_01B2,
    0x0042_01B3,
    0x0042_01B4,
    0x0042_01B5,
    0x0042_01B6,
    0x0042_01B7,
    0x0042_01B8,
    0x0042_01B9,
    0x0042_01BA,
    0x0042_01BB,
    0x0042_01BC,
    0x0042_01BD,
    0x0042_01BE,
    0x0042_01BF,
    0x0042_01C0,
    0x0042_01C1,
    0x0042_01C2,
    0x0042_01C3,
    0x0042_01C4,
    0x0042_01C5,
    0x0042_01C6,
    0x0042_01C7,
    0x0042_01C8,
    0x0042_01C9,
    0x0042_01CA,
    0x0042_01CB,
];
