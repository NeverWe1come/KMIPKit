//! Lossless normalization for link relationships shared by KMIP 2.1 and 3.0.

use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

/// A link relationship with an exact KMIP 2.1 and KMIP 3.0 representation.
///
/// The numeric assignments are intentionally mapped explicitly. The KMIP 2.1
/// Enumeration values and KMIP 3.0 attribute tags are not contiguous semantic
/// ranges.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LinkType {
    /// Certificate Link.
    Certificate,
    /// Public Key Link.
    PublicKey,
    /// Private Key Link.
    PrivateKey,
    /// Derivation Base Object Link in KMIP 2.1; Derivation Base Object Link in 3.0.
    DerivationBaseObject,
    /// Derived Key Link in KMIP 2.1; Derived Object Link in 3.0.
    DerivedKey,
    /// Replacement Object Link.
    ReplacementObject,
    /// Replaced Object Link.
    ReplacedObject,
    /// Parent Link.
    Parent,
    /// Child Link.
    Child,
    /// Previous Link.
    Previous,
    /// Next Link.
    Next,
    /// PKCS#12 Certificate Link.
    Pkcs12Certificate,
    /// PKCS#12 Password Link.
    Pkcs12Password,
    /// Wrapping Key Link.
    WrappingKey,
}

impl LinkType {
    /// Returns the exact KMIP 2.1 §11.28, Table 465 Link Type value.
    #[must_use]
    pub const fn v2_enumeration(self) -> u32 {
        match self {
            Self::Certificate => 0x0000_0101,
            Self::PublicKey => 0x0000_0102,
            Self::PrivateKey => 0x0000_0103,
            Self::DerivationBaseObject => 0x0000_0104,
            Self::DerivedKey => 0x0000_0105,
            Self::ReplacementObject => 0x0000_0106,
            Self::ReplacedObject => 0x0000_0107,
            Self::Parent => 0x0000_0108,
            Self::Child => 0x0000_0109,
            Self::Previous => 0x0000_010A,
            Self::Next => 0x0000_010B,
            Self::Pkcs12Certificate => 0x0000_010C,
            Self::Pkcs12Password => 0x0000_010D,
            Self::WrappingKey => 0x0000_010E,
        }
    }

    /// Returns the matching KMIP 3.0 §4.35 attribute tag from §11.58.
    #[must_use]
    pub const fn v3_tag(self) -> u32 {
        match self {
            Self::Certificate => 0x0042_0190,
            Self::Child => 0x0042_0191,
            Self::DerivationBaseObject => 0x0042_0192,
            Self::DerivedKey => 0x0042_0193,
            Self::Next => 0x0042_0194,
            Self::Parent => 0x0042_0195,
            Self::Pkcs12Certificate => 0x0042_0196,
            Self::Pkcs12Password => 0x0042_0197,
            Self::Previous => 0x0042_0198,
            Self::PrivateKey => 0x0042_0199,
            Self::PublicKey => 0x0042_019A,
            Self::ReplacedObject => 0x0042_019B,
            Self::ReplacementObject => 0x0042_019C,
            Self::WrappingKey => 0x0042_019D,
        }
    }

    /// Resolves one assigned KMIP 2.1 §11.28, Table 465 Enumeration value.
    #[must_use]
    pub const fn from_v2_enumeration(value: u32) -> Option<Self> {
        match value {
            0x0000_0101 => Some(Self::Certificate),
            0x0000_0102 => Some(Self::PublicKey),
            0x0000_0103 => Some(Self::PrivateKey),
            0x0000_0104 => Some(Self::DerivationBaseObject),
            0x0000_0105 => Some(Self::DerivedKey),
            0x0000_0106 => Some(Self::ReplacementObject),
            0x0000_0107 => Some(Self::ReplacedObject),
            0x0000_0108 => Some(Self::Parent),
            0x0000_0109 => Some(Self::Child),
            0x0000_010A => Some(Self::Previous),
            0x0000_010B => Some(Self::Next),
            0x0000_010C => Some(Self::Pkcs12Certificate),
            0x0000_010D => Some(Self::Pkcs12Password),
            0x0000_010E => Some(Self::WrappingKey),
            _ => None,
        }
    }

    /// Resolves a KMIP 3.0 link attribute tag with a KMIP 2.1 equivalent.
    #[must_use]
    pub const fn from_v3_tag(tag: u32) -> Option<Self> {
        match tag {
            0x0042_0190 => Some(Self::Certificate),
            0x0042_0191 => Some(Self::Child),
            0x0042_0192 => Some(Self::DerivationBaseObject),
            0x0042_0193 => Some(Self::DerivedKey),
            0x0042_0194 => Some(Self::Next),
            0x0042_0195 => Some(Self::Parent),
            0x0042_0196 => Some(Self::Pkcs12Certificate),
            0x0042_0197 => Some(Self::Pkcs12Password),
            0x0042_0198 => Some(Self::Previous),
            0x0042_0199 => Some(Self::PrivateKey),
            0x0042_019A => Some(Self::PublicKey),
            0x0042_019B => Some(Self::ReplacedObject),
            0x0042_019C => Some(Self::ReplacementObject),
            0x0042_019D => Some(Self::WrappingKey),
            _ => None,
        }
    }
}

/// Converts only the link relationships with a source-defined counterpart.
pub struct LinkNormalizer;

impl LinkNormalizer {
    /// Converts a KMIP 2.1 generic `Link` Structure to its KMIP 3.0 attribute.
    ///
    /// The input must contain exactly one Link Type and one Linked Object
    /// Identifier. Unknown children and unsupported identifier encodings are
    /// rejected so an extension is never silently discarded.
    ///
    /// # Errors
    ///
    /// Returns a fixed, redacted error when the input shape or values cannot
    /// be represented without loss.
    pub fn normalize_to_v3(generic_link: &Item) -> Result<Item, String> {
        if generic_link.tag().raw() != 0x0042_0047 {
            return Err("Expected KMIP 2.1 generic Link tag 0x0042_0047".into());
        }

        let mut is_structure = false;
        let mut invalid_child = false;
        let mut link_type = None;
        let mut linked_identifier = None;
        generic_link.with_value(|view| {
            let ValueView::Structure(structure) = view else {
                return;
            };
            is_structure = true;
            for child in structure.children() {
                match child.tag().raw() {
                    0x0042_0048 if link_type.is_none() => {
                        child.with_value(|value| match value {
                            ValueView::Enumeration(raw) => link_type = Some(*raw),
                            _ => invalid_child = true,
                        });
                    }
                    0x0042_0049 if linked_identifier.is_none() => {
                        child.with_value(|value| match copy_identifier(&value) {
                            Ok(identifier) => linked_identifier = Some(identifier),
                            Err(()) => invalid_child = true,
                        });
                    }
                    _ => invalid_child = true,
                }
            }
        });

        if !is_structure || invalid_child {
            return Err("Generic Link contains fields that cannot be normalized losslessly".into());
        }
        let link_type = link_type
            .and_then(LinkType::from_v2_enumeration)
            .ok_or_else(|| "Missing or unrecognized KMIP 2.1 Link Type".to_string())?;
        let linked_identifier = linked_identifier
            .ok_or_else(|| "Missing KMIP 2.1 Linked Object Identifier".to_string())?;
        let tag = RawTag::new(link_type.v3_tag())
            .map_err(|_| "KMIP 3.0 Link tag is invalid".to_string())?
            .try_checked()
            .map_err(|_| "KMIP 3.0 Link tag is not allocated".to_string())?;

        Item::new(tag, linked_identifier).map_err(|_| "KMIP 3.0 Link value is invalid".to_string())
    }

    /// Converts a mapped KMIP 3.0 link attribute to a KMIP 2.1 `Link` Structure.
    ///
    /// KMIP 3.0-only link attributes and Reference/Name Reference values have
    /// no lossless KMIP 2.1 representation and return an error. The supplied
    /// item is borrowed and remains unchanged.
    ///
    /// # Errors
    ///
    /// Returns a fixed, redacted error when no exact semantic or identifier
    /// representation exists in KMIP 2.1.
    pub fn normalize_to_v2(typed_link: &Item) -> Result<Item, String> {
        let link_type = LinkType::from_v3_tag(typed_link.tag().raw())
            .ok_or_else(|| "KMIP 3.0 link tag has no KMIP 2.1 Link Type equivalent".to_string())?;
        let linked_identifier = typed_link
            .with_value(|value| copy_identifier(&value))
            .map_err(|()| "KMIP 3.0 link identifier has no KMIP 2.1 representation".to_string())?;

        let link_type_tag = checked_tag(0x0042_0048)?;
        let identifier_tag = checked_tag(0x0042_0049)?;
        let link_tag = checked_tag(0x0042_0047)?;
        let mut structure = Structure::new();
        structure
            .try_push(
                Item::new(
                    link_type_tag,
                    Value::enumeration(link_type.v2_enumeration()),
                )
                .map_err(|_| "KMIP 2.1 Link Type is invalid".to_string())?,
            )
            .map_err(|_| "KMIP 2.1 Link Structure is invalid".to_string())?;
        structure
            .try_push(
                Item::new(identifier_tag, linked_identifier)
                    .map_err(|_| "KMIP 2.1 Linked Object Identifier is invalid".to_string())?,
            )
            .map_err(|_| "KMIP 2.1 Link Structure is invalid".to_string())?;
        Item::new(link_tag, Value::structure(structure))
            .map_err(|_| "KMIP 2.1 Link Structure is invalid".to_string())
    }
}

fn copy_identifier(value: &ValueView<'_>) -> Result<Value, ()> {
    match value {
        ValueView::Enumeration(value) => Ok(Value::enumeration(**value)),
        ValueView::Integer(value) => Ok(Value::integer(**value)),
        _ => Err(()),
    }
}

fn checked_tag(raw: u32) -> Result<kmipkit_ttlv::Tag, String> {
    RawTag::new(raw)
        .map_err(|_| "KMIP tag is invalid".to_string())?
        .try_checked()
        .map_err(|_| "KMIP tag is not allocated".to_string())
}
