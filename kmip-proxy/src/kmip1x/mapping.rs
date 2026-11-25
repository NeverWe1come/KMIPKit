//! Exhaustive bidirectional mapping tables between KMIP 1.0–1.4 and KMIP 2.0 / 2.1 / 3.0.
//!
//! Covers:
//! - All 77 KMIP 1.0–1.4 standard attributes (with cardinality and KMIP 3.0 read-only flags)
//! - Deprecated KMIP 1.0 certificate attributes normalization
//! - Custom `x-*` / `y-*` attributes `<->` KMIP 2.0+ Vendor Attributes
//! - KMIP 1.X `Link` (`0x42004A`) `<->` KMIP 3.0 typed link attributes (`0x420190..=0x42019D`)
//! - All KMIP 2.0 / 2.1 / 3.0 Result Reasons downgraded to valid KMIP 1.X Result Reasons
//! - Version-gated `Query` allowlists (Operations, Object Types, Algorithms, Query Functions)
#![forbid(unsafe_code)]

use crate::kmip1x::tags::*;

/// Target upstream KMIP version spoken with the modern KMIP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpstreamKmipVersion {
    /// OASIS KMIP 2.0.
    V2_0,
    /// OASIS KMIP 2.1 (default upstream version).
    #[default]
    V2_1,
    /// OASIS KMIP 3.0 (supports typed links and strict read-only create rules).
    V3_0,
}

impl UpstreamKmipVersion {
    /// Returns the numeric `(major, minor)` protocol version pair.
    #[must_use]
    pub const fn major_minor(self) -> (i32, i32) {
        match self {
            Self::V2_0 => (2, 0),
            Self::V2_1 => (2, 1),
            Self::V3_0 => (3, 0),
        }
    }

    /// Parses a textual version identifier (`"2.0"`, `"2.1"`, `"3.0"`, etc.).
    ///
    /// # Errors
    ///
    /// Returns an error string if `s` is not one of the supported KMIP 2.0, 2.1, or 3.0 formats.
    pub fn parse_str(s: &str) -> Result<Self, String> {
        match s.trim() {
            "2.0" | "2_0" | "v2.0" => Ok(Self::V2_0),
            "2.1" | "2_1" | "v2.1" => Ok(Self::V2_1),
            "3.0" | "3_0" | "v3.0" => Ok(Self::V3_0),
            other => Err(format!(
                "unsupported upstream KMIP version '{other}'; expected 2.0, 2.1, or 3.0"
            )),
        }
    }
}

/// Policy for handling KMIP 1.X `Operation Policy Name` (removed in KMIP 2.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OperationPolicyHandling {
    /// Silently drop `Operation Policy Name` (access control is managed by upstream RBAC/mTLS).
    #[default]
    Ignore,
    /// Preserve it as a KMIP 2.0+ Vendor Attribute (`kmip1x-proxy` / `Operation Policy Name`).
    StoreAsVendorAttribute,
    /// Reject requests containing `Operation Policy Name` with `Operation Not Supported`.
    Reject,
}

/// Cardinality of an attribute in KMIP 2.0 / 2.1 / 3.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// Single-instance: `Add Attribute` / `Modify Attribute` route to `Set Attribute` (`0x31`).
    SingleInstance,
    /// Multi-instance: `Add Attribute` stays `Add Attribute` (`0x0D`); `Modify`/`Delete` use RMW.
    MultiInstance,
}

/// Normative descriptor for a KMIP 1.0–1.4 standard attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeSpec {
    /// Canonical KMIP 1.X string attribute name.
    pub name_1x: &'static str,
    /// Corresponding KMIP 2.0 / 2.1 / 3.0 24-bit TTLV tag.
    pub tag_2x: u32,
    /// Single-instance vs multi-instance cardinality in KMIP 2.0+.
    pub cardinality: Cardinality,
    /// Minimum KMIP 1.X minor version where this attribute was introduced (`0..=4`).
    pub min_minor_1x: i32,
    /// `true` if server-managed/read-only (stripped from `Create`/`Register` when enabled).
    pub read_only_on_create: bool,
}

/// Complete normative table of KMIP 1.0–1.4 standard attributes.
pub const STANDARD_ATTRIBUTES: &[AttributeSpec] = &[
    // Core identity & classification (KMIP 1.0)
    AttributeSpec {
        name_1x: "Unique Identifier",
        tag_2x: TAG_UNIQUE_IDENTIFIER,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Name",
        tag_2x: TAG_NAME,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Object Type",
        tag_2x: TAG_OBJECT_TYPE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    // Cryptographic properties (KMIP 1.0)
    AttributeSpec {
        name_1x: "Cryptographic Algorithm",
        tag_2x: TAG_CRYPTOGRAPHIC_ALGORITHM,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Cryptographic Length",
        tag_2x: TAG_CRYPTOGRAPHIC_LENGTH,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Cryptographic Parameters",
        tag_2x: TAG_CRYPTOGRAPHIC_PARAMETERS,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Cryptographic Domain Parameters",
        tag_2x: TAG_CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Cryptographic Usage Mask",
        tag_2x: TAG_CRYPTOGRAPHIC_USAGE_MASK,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Key Format Type",
        tag_2x: TAG_KEY_FORMAT_TYPE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Key Part Identifier",
        tag_2x: TAG_KEY_PART_IDENTIFIER,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Split Key Method",
        tag_2x: TAG_SPLIT_KEY_METHOD,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Split Key Parts",
        tag_2x: TAG_SPLIT_KEY_PARTS,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Split Key Threshold",
        tag_2x: TAG_SPLIT_KEY_THRESHOLD,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Opaque Data Type",
        tag_2x: TAG_OPAQUE_DATA_TYPE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    // Certificate properties (KMIP 1.0 / 1.1)
    AttributeSpec {
        name_1x: "Certificate Type",
        tag_2x: TAG_CERTIFICATE_TYPE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Certificate Length",
        tag_2x: TAG_CERTIFICATE_LENGTH,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 1,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "X.509 Certificate Identifier",
        tag_2x: TAG_X509_CERTIFICATE_IDENTIFIER,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 1,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "X.509 Certificate Subject",
        tag_2x: TAG_X509_CERTIFICATE_SUBJECT,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 1,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "X.509 Certificate Issuer",
        tag_2x: TAG_X509_CERTIFICATE_ISSUER,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 1,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Digital Signature Algorithm",
        tag_2x: TAG_DIGITAL_SIGNATURE_ALGORITHM,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 1,
        read_only_on_create: true,
    },
    // Digest & Lifecycle State / Dates (KMIP 1.0)
    AttributeSpec {
        name_1x: "Digest",
        tag_2x: TAG_DIGEST,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "State",
        tag_2x: TAG_STATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Initial Date",
        tag_2x: TAG_INITIAL_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Activation Date",
        tag_2x: TAG_ACTIVATION_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Process Start Date",
        tag_2x: TAG_PROCESS_START_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Protect Stop Date",
        tag_2x: TAG_PROTECT_STOP_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Deactivation Date",
        tag_2x: TAG_DEACTIVATION_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Destroy Date",
        tag_2x: TAG_DESTROY_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Compromise Occurrence Date",
        tag_2x: TAG_COMPROMISE_OCCURRENCE_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Compromise Date",
        tag_2x: TAG_COMPROMISE_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Archive Date",
        tag_2x: TAG_ARCHIVE_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Last Change Date",
        tag_2x: TAG_LAST_CHANGE_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Revocation Reason",
        tag_2x: TAG_REVOCATION_REASON,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: true,
    },
    // Lease, Usage, Links, Grouping, Metadata (KMIP 1.0)
    AttributeSpec {
        name_1x: "Lease Time",
        tag_2x: TAG_LEASE_TIME,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Usage Limits",
        tag_2x: TAG_USAGE_LIMITS,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Link",
        tag_2x: TAG_LINK,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Object Group",
        tag_2x: TAG_OBJECT_GROUP,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Contact Information",
        tag_2x: TAG_CONTACT_INFORMATION,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Application Specific Information",
        tag_2x: TAG_APPLICATION_SPECIFIC_INFORMATION,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 0,
        read_only_on_create: false,
    },
    // KMIP 1.1 additions
    AttributeSpec {
        name_1x: "Fresh",
        tag_2x: TAG_FRESH,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 1,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Key Value Present",
        tag_2x: TAG_KEY_VALUE_PRESENT,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 1,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Key Value Location",
        tag_2x: TAG_KEY_VALUE_LOCATION,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 1,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Original Creation Date",
        tag_2x: TAG_ORIGINAL_CREATION_DATE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 1,
        read_only_on_create: false,
    },
    // KMIP 1.2 additions
    AttributeSpec {
        name_1x: "Alternative Name",
        tag_2x: TAG_ALTERNATIVE_NAME,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 2,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Random Number Generator",
        tag_2x: TAG_RANDOM_NUMBER_GENERATOR,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 2,
        read_only_on_create: true,
    },
    // KMIP 1.3 additions
    AttributeSpec {
        name_1x: "PKCS#12 Friendly Name",
        tag_2x: TAG_PKCS12_FRIENDLY_NAME,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 3,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Description",
        tag_2x: TAG_DESCRIPTION,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 3,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Comment",
        tag_2x: TAG_COMMENT,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 3,
        read_only_on_create: false,
    },
    // KMIP 1.4 additions
    AttributeSpec {
        name_1x: "Sensitive",
        tag_2x: TAG_SENSITIVE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 4,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Always Sensitive",
        tag_2x: TAG_ALWAYS_SENSITIVE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Extractable",
        tag_2x: TAG_EXTRACTABLE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 4,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Never Extractable",
        tag_2x: TAG_NEVER_EXTRACTABLE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Key Role Type",
        tag_2x: TAG_KEY_ROLE_TYPE,
        cardinality: Cardinality::SingleInstance,
        min_minor_1x: 4,
        read_only_on_create: false,
    },
    AttributeSpec {
        name_1x: "Certificate Subject CN",
        tag_2x: TAG_CERTIFICATE_SUBJECT_CN,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject O",
        tag_2x: TAG_CERTIFICATE_SUBJECT_O,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject OU",
        tag_2x: TAG_CERTIFICATE_SUBJECT_OU,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject Email",
        tag_2x: TAG_CERTIFICATE_SUBJECT_EMAIL,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject C",
        tag_2x: TAG_CERTIFICATE_SUBJECT_C,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject ST",
        tag_2x: TAG_CERTIFICATE_SUBJECT_ST,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject L",
        tag_2x: TAG_CERTIFICATE_SUBJECT_L,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject UID",
        tag_2x: TAG_CERTIFICATE_SUBJECT_UID,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject Serial Number",
        tag_2x: TAG_CERTIFICATE_SUBJECT_SERIAL_NUMBER,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject Title",
        tag_2x: TAG_CERTIFICATE_SUBJECT_TITLE,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject DC",
        tag_2x: TAG_CERTIFICATE_SUBJECT_DC,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Subject DN Qualifier",
        tag_2x: TAG_CERTIFICATE_SUBJECT_DN_QUALIFIER,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer CN",
        tag_2x: TAG_CERTIFICATE_ISSUER_CN,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer O",
        tag_2x: TAG_CERTIFICATE_ISSUER_O,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer OU",
        tag_2x: TAG_CERTIFICATE_ISSUER_OU,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer Email",
        tag_2x: TAG_CERTIFICATE_ISSUER_EMAIL,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer C",
        tag_2x: TAG_CERTIFICATE_ISSUER_C,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer ST",
        tag_2x: TAG_CERTIFICATE_ISSUER_ST,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer L",
        tag_2x: TAG_CERTIFICATE_ISSUER_L,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer UID",
        tag_2x: TAG_CERTIFICATE_ISSUER_UID,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer Serial Number",
        tag_2x: TAG_CERTIFICATE_ISSUER_SERIAL_NUMBER,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer Title",
        tag_2x: TAG_CERTIFICATE_ISSUER_TITLE,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer DC",
        tag_2x: TAG_CERTIFICATE_ISSUER_DC,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
    AttributeSpec {
        name_1x: "Certificate Issuer DN Qualifier",
        tag_2x: TAG_CERTIFICATE_ISSUER_DN_QUALIFIER,
        cardinality: Cardinality::MultiInstance,
        min_minor_1x: 4,
        read_only_on_create: true,
    },
];

/// Resolves a KMIP 1.X attribute name (including deprecated KMIP 1.0 aliases) to its spec.
#[must_use]
pub fn lookup_by_name_1x(name: &str) -> Option<&'static AttributeSpec> {
    let normalized = match name {
        "Certificate Identifier" => "X.509 Certificate Identifier",
        "Certificate Subject" => "X.509 Certificate Subject",
        "Certificate Issuer" => "X.509 Certificate Issuer",
        other => other,
    };
    STANDARD_ATTRIBUTES
        .iter()
        .find(|spec| spec.name_1x == normalized)
}

/// Resolves a KMIP 2.0/2.1/3.0 attribute tag back to its KMIP 1.X attribute spec.
#[must_use]
pub fn lookup_by_tag_2x(tag: u32) -> Option<&'static AttributeSpec> {
    STANDARD_ATTRIBUTES.iter().find(|spec| spec.tag_2x == tag)
}

/// Returns true if the attribute name follows the KMIP 1.X custom attribute convention (`x-*` or `y-*`).
#[must_use]
pub fn is_custom_attribute_1x(name: &str) -> bool {
    name.starts_with("x-") || name.starts_with("y-")
}

/// Maps a KMIP 1.X `Link Type` enumeration to its KMIP 3.0 typed link attribute tag.
#[must_use]
pub const fn link_type_to_v3_tag(link_type: u32) -> Option<u32> {
    match link_type {
        LINK_TYPE_CERTIFICATE => Some(TAG_CERTIFICATE_LINK),
        LINK_TYPE_PUBLIC_KEY => Some(TAG_PUBLIC_KEY_LINK),
        LINK_TYPE_PRIVATE_KEY => Some(TAG_PRIVATE_KEY_LINK),
        LINK_TYPE_DERIVATION_BASE => Some(TAG_DERIVATION_BASE_OBJECT_LINK),
        LINK_TYPE_DERIVED_KEY => Some(TAG_DERIVED_OBJECT_LINK),
        LINK_TYPE_REPLACEMENT => Some(TAG_REPLACEMENT_OBJECT_LINK),
        LINK_TYPE_REPLACED => Some(TAG_REPLACED_OBJECT_LINK),
        LINK_TYPE_PARENT => Some(TAG_PARENT_LINK),
        LINK_TYPE_CHILD => Some(TAG_CHILD_LINK),
        LINK_TYPE_PREVIOUS => Some(TAG_PREVIOUS_LINK),
        LINK_TYPE_NEXT => Some(TAG_NEXT_LINK),
        LINK_TYPE_PKCS12_CERT => Some(TAG_PKCS12_CERTIFICATE_LINK),
        LINK_TYPE_PKCS12_PASSWORD => Some(TAG_PKCS12_PASSWORD_LINK),
        LINK_TYPE_WRAPPING_KEY => Some(TAG_WRAPPING_KEY_LINK),
        _ => None,
    }
}

/// Maps a KMIP 3.0 typed link attribute tag back to its KMIP 1.X `Link Type` enumeration.
#[must_use]
pub const fn v3_link_tag_to_link_type(tag: u32) -> Option<u32> {
    match tag {
        TAG_CERTIFICATE_LINK => Some(LINK_TYPE_CERTIFICATE),
        TAG_PUBLIC_KEY_LINK => Some(LINK_TYPE_PUBLIC_KEY),
        TAG_PRIVATE_KEY_LINK => Some(LINK_TYPE_PRIVATE_KEY),
        TAG_DERIVATION_BASE_OBJECT_LINK => Some(LINK_TYPE_DERIVATION_BASE),
        TAG_DERIVED_OBJECT_LINK => Some(LINK_TYPE_DERIVED_KEY),
        TAG_REPLACEMENT_OBJECT_LINK => Some(LINK_TYPE_REPLACEMENT),
        TAG_REPLACED_OBJECT_LINK => Some(LINK_TYPE_REPLACED),
        TAG_PARENT_LINK => Some(LINK_TYPE_PARENT),
        TAG_CHILD_LINK => Some(LINK_TYPE_CHILD),
        TAG_PREVIOUS_LINK => Some(LINK_TYPE_PREVIOUS),
        TAG_NEXT_LINK => Some(LINK_TYPE_NEXT),
        TAG_PKCS12_CERTIFICATE_LINK => Some(LINK_TYPE_PKCS12_CERT),
        TAG_PKCS12_PASSWORD_LINK => Some(LINK_TYPE_PKCS12_PASSWORD),
        TAG_WRAPPING_KEY_LINK => Some(LINK_TYPE_WRAPPING_KEY),
        _ => None,
    }
}

/// Downgrades any KMIP 2.0 / 2.1 / 3.0 Result Reason to a valid KMIP 1.X Result Reason.
#[must_use]
pub const fn downgrade_result_reason(reason_2x: u32, client_minor_1x: i32) -> u32 {
    match reason_2x {
        // KMIP 1.0 core result reasons (0x01..=0x0F, 0x100)
        0x0000_0001..=0x0000_000F | 0x0000_0100 => reason_2x,
        // KMIP 1.1 additions (0x10..=0x13)
        0x0000_0010..=0x0000_0013 => {
            if client_minor_1x >= 1 {
                reason_2x
            } else {
                REASON_INVALID_FIELD
            }
        }
        // KMIP 1.2 additions (0x14..=0x15)
        0x0000_0014..=0x0000_0015 => {
            if client_minor_1x >= 2 {
                reason_2x
            } else {
                REASON_AUTHENTICATION_NOT_SUCCESSFUL
            }
        }
        // KMIP 1.4 addition: Sensitive (0x16)
        0x0000_0016 => {
            if client_minor_1x >= 4 {
                reason_2x
            } else {
                REASON_PERMISSION_DENIED
            }
        }
        // KMIP 2.0+ specific reasons mapped to closest 1.X semantics
        0x0000_0017 => REASON_PERMISSION_DENIED, // Not Extractable
        0x0000_0018 => REASON_INVALID_FIELD,     // Object Already Exists
        0x0000_0019 => REASON_AUTHENTICATION_NOT_SUCCESSFUL, // Invalid Ticket
        0x0000_001A => REASON_PERMISSION_DENIED, // Usage Limit Exceeded
        0x0000_001B..=0x0000_001C => REASON_INVALID_FIELD, // Numeric Range / Invalid Data Type
        0x0000_001D | 0x0000_0022 => REASON_PERMISSION_DENIED, // Read Only Attribute
        0x0000_001E | 0x0000_0023 => REASON_INVALID_FIELD, // Multi Valued / Single Valued
        0x0000_001F => REASON_FEATURE_NOT_SUPPORTED, // Unsupported Attribute
        0x0000_0020 | 0x0000_0021 => REASON_ITEM_NOT_FOUND, // Attribute (Instance) Not Found
        0x0000_0024 => REASON_CRYPTOGRAPHIC_FAILURE, // Bad Cryptographic Parameters
        0x0000_0025 => REASON_AUTHENTICATION_NOT_SUCCESSFUL, // Bad Password
        0x0000_0026 => REASON_INVALID_MESSAGE,   // Codec Error
        0x0000_0028..=0x0000_0029 => REASON_INVALID_FIELD, // Illegal Object Type / Incompatible Usage Mask
        0x0000_002A => REASON_GENERAL_FAILURE,             // Internal Server Error
        0x0000_002B..=0x0000_0030 => REASON_INVALID_FIELD, // Invalid Correlation / Attribute / CSR / Object Type
        0x0000_0032 => REASON_FEATURE_NOT_SUPPORTED,       // Key Wrap Type Not Supported
        0x0000_0034 => REASON_MISSING_DATA,                // Missing Initialization Vector
        0x0000_0035 => REASON_INVALID_FIELD,               // Non Unique Name Attribute
        0x0000_0036 | 0x0000_0037 => REASON_ITEM_NOT_FOUND, // Object Destroyed / Not Found
        0x0000_0039 => REASON_PERMISSION_DENIED,           // Not Authorised
        0x0000_003A => REASON_GENERAL_FAILURE,             // Server Limit Exceeded
        0x0000_003B..=0x0000_003D => REASON_INVALID_MESSAGE, // Unknown Enum / Extension / Tag
        0x0000_003E => REASON_FEATURE_NOT_SUPPORTED,       // Unsupported Cryptographic Parameters
        0x0000_003F => REASON_OPERATION_NOT_SUPPORTED,     // Unsupported Protocol Version
        0x0000_0040 => REASON_OBJECT_ARCHIVED,             // Wrapping Object Archived
        0x0000_0041 | 0x0000_0042 => REASON_ITEM_NOT_FOUND, // Wrapping Object Destroyed / Not Found
        0x0000_0043 => REASON_PERMISSION_DENIED,           // Wrong Key Lifecycle State
        0x0000_0044..=0x0000_0049 => REASON_FEATURE_NOT_SUPPORTED, // Protection Storage / PKCS#11
        0x0000_004A..=0x0000_004D => REASON_INVALID_FIELD, // Unknown Group / Constraint / Duplicate / Circular Link
        _ => REASON_GENERAL_FAILURE,
    }
}

/// Returns true if an operation enum value is valid for the given KMIP 1.X minor version.
#[must_use]
pub const fn is_operation_allowed_in_1x(op: u32, minor_1x: i32) -> bool {
    match op {
        // KMIP 1.0 operations (0x01..=0x1C)
        0x0000_0001..=0x0000_001C => true,
        // KMIP 1.1: Re-key Key Pair (0x1D), Discover Versions (0x1E)
        0x0000_001D..=0x0000_001E => minor_1x >= 1,
        // KMIP 1.2: Encrypt..Join Split Key (0x1F..=0x29)
        0x0000_001F..=0x0000_0029 => minor_1x >= 2,
        // KMIP 1.4: Import (0x2A), Export (0x2B)
        0x0000_002A..=0x0000_002B => minor_1x >= 4,
        _ => false,
    }
}

/// Returns true if an Object Type enum value is valid in the given KMIP 1.X minor version.
#[must_use]
pub const fn is_object_type_allowed_in_1x(obj_type: u32, minor_1x: i32) -> bool {
    match obj_type {
        // Certificate..Opaque Object (including Template)
        0x0000_0001..=0x0000_0008 => true,
        // PGP Key added in KMIP 1.2
        0x0000_0009 => minor_1x >= 2,
        _ => false,
    }
}

/// Returns true if a Query Function enum value is valid in the given KMIP 1.X minor version.
#[must_use]
pub const fn is_query_function_allowed_in_1x(qf: u32, minor_1x: i32) -> bool {
    match qf {
        0x0000_0001..=0x0000_0004 => true, // Operations, Objects, Server Info, App Namespaces
        0x0000_0005..=0x0000_0006 => minor_1x >= 1, // Extension List, Extension Map
        0x0000_0007 => minor_1x >= 2,      // Attestation Types
        0x0000_0008..=0x0000_000C => minor_1x >= 3, // RNGs, Validations, Profiles, Capabilities, Client Reg
        _ => false,
    }
}

/// Returns true if a Cryptographic Algorithm enum value is valid in KMIP 1.X
/// (filters out KMIP 2.0/3.0 PQC algorithms like ML-KEM, ML-DSA, SLH-DSA, X25519, etc.).
#[must_use]
pub const fn is_crypto_algorithm_allowed_in_1x(alg: u32, minor_1x: i32) -> bool {
    match alg {
        0x0000_0001..=0x0000_0019 => true, // DES..Twofish (KMIP 1.0)
        0x0000_001A..=0x0000_001B => minor_1x >= 2, // EC, One Time Pad
        0x0000_001C..=0x0000_0028 => minor_1x >= 4, // ChaCha20..SHAKE-256 (KMIP 1.4)
        _ => false,                        // PQC & KMIP 2.0+/3.0 additions filtered
    }
}

/// Returns true if the operation expects a `Unique Identifier` that can use KMIP 1.X implicit ID Placeholder.
#[must_use]
pub const fn operation_supports_id_placeholder(op: u32) -> bool {
    matches!(
        op,
        OP_REKEY
            | OP_REKEY_KEY_PAIR
            | OP_DERIVE_KEY
            | OP_CERTIFY
            | OP_RECERTIFY
            | OP_CHECK
            | OP_GET
            | OP_GET_ATTRIBUTES
            | OP_GET_ATTRIBUTE_LIST
            | OP_ADD_ATTRIBUTE
            | OP_MODIFY_ATTRIBUTE
            | OP_DELETE_ATTRIBUTE
            | OP_OBTAIN_LEASE
            | OP_GET_USAGE_ALLOCATION
            | OP_ACTIVATE
            | OP_REVOKE
            | OP_DESTROY
            | OP_ARCHIVE
            | OP_RECOVER
            | OP_ENCRYPT
            | OP_DECRYPT
            | OP_SIGN
            | OP_SIGNATURE_VERIFY
            | OP_MAC
            | OP_MAC_VERIFY
            | OP_EXPORT
            | OP_JOIN_SPLIT_KEY
    )
}
