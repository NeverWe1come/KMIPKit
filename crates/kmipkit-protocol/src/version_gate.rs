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
        match operation {
            // Operations introduced in KMIP 2.1 (Set Defaults through Ping)
            0x0000_0036..=0x0000_003B => Some(VersionSpan::since(ProtocolVersion::V2_1)),

            // Operations introduced in KMIP 3.0 (Create User through ReEncrypt)
            0x0000_003C..=0x0000_0042 => Some(VersionSpan::since(ProtocolVersion::V3_0)),

            // Base operations existing in KMIP 2.0 (and 2.1)
            0x0000_0001..=0x0000_0035 => Some(VersionSpan::since(ProtocolVersion::V2_0)),

            _ => None,
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

        if let Some(span) = Self::operation_span(operation)
            && !span.is_supported_in(version)
        {
            return Err(VersionGateError::OperationNotSupportedInVersion {
                operation,
                negotiated: version,
                introduced: span.introduced,
            });
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

            // KMIP 3.0 System Objects & PQC attributes
            0x0042_0153..=0x0042_0172 => Some(VersionSpan::since(ProtocolVersion::V3_0)),

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
