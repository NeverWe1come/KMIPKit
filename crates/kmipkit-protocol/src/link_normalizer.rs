//! KMIP 2.x generic Link to KMIP 3.0 Discrete Typed Link Normalizer (KMIPKIT-0071).

use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

/// Supported link relationships between KMIP objects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LinkType {
    Certificate,
    PublicKey,
    PrivateKey,
    PreviousKey,
    NextKey,
    DerivationBaseObject,
    DerivedKey,
    SplitKey,
    WrappingKey,
    ParentObject,
    ChildObject,
}

impl LinkType {
    /// Returns the KMIP 2.x enumeration value for this link type.
    #[must_use]
    pub const fn v2_enumeration(self) -> u32 {
        match self {
            Self::Certificate => 0x0000_0101,
            Self::PublicKey => 0x0000_0102,
            Self::PrivateKey => 0x0000_0103,
            Self::PreviousKey => 0x0000_0104,
            Self::NextKey => 0x0000_0105,
            Self::DerivationBaseObject => 0x0000_0106,
            Self::DerivedKey => 0x0000_0107,
            Self::SplitKey => 0x0000_0108,
            Self::WrappingKey => 0x0000_0109,
            Self::ParentObject => 0x0000_010A,
            Self::ChildObject => 0x0000_010B,
        }
    }

    /// Returns the KMIP 3.0 discrete typed link TTLV tag for this link type.
    #[must_use]
    pub const fn v3_tag(self) -> u32 {
        match self {
            Self::Certificate => 0x0042_0153,
            Self::PublicKey => 0x0042_0154,
            Self::PrivateKey => 0x0042_0155,
            Self::PreviousKey => 0x0042_0156,
            Self::NextKey => 0x0042_0157,
            Self::DerivationBaseObject => 0x0042_0158,
            Self::DerivedKey => 0x0042_0159,
            Self::SplitKey => 0x0042_015A,
            Self::WrappingKey => 0x0042_015B,
            Self::ParentObject => 0x0042_015C,
            Self::ChildObject => 0x0042_015D,
        }
    }

    /// Resolves from KMIP 2.x enumeration value.
    #[must_use]
    pub const fn from_v2_enumeration(val: u32) -> Option<Self> {
        match val {
            0x0000_0101 => Some(Self::Certificate),
            0x0000_0102 => Some(Self::PublicKey),
            0x0000_0103 => Some(Self::PrivateKey),
            0x0000_0104 => Some(Self::PreviousKey),
            0x0000_0105 => Some(Self::NextKey),
            0x0000_0106 => Some(Self::DerivationBaseObject),
            0x0000_0107 => Some(Self::DerivedKey),
            0x0000_0108 => Some(Self::SplitKey),
            0x0000_0109 => Some(Self::WrappingKey),
            0x0000_010A => Some(Self::ParentObject),
            0x0000_010B => Some(Self::ChildObject),
            _ => None,
        }
    }

    /// Resolves from KMIP 3.0 discrete typed link TTLV tag.
    #[must_use]
    pub const fn from_v3_tag(tag: u32) -> Option<Self> {
        match tag {
            0x0042_0153 => Some(Self::Certificate),
            0x0042_0154 => Some(Self::PublicKey),
            0x0042_0155 => Some(Self::PrivateKey),
            0x0042_0156 => Some(Self::PreviousKey),
            0x0042_0157 => Some(Self::NextKey),
            0x0042_0158 => Some(Self::DerivationBaseObject),
            0x0042_0159 => Some(Self::DerivedKey),
            0x0042_015A => Some(Self::SplitKey),
            0x0042_015B => Some(Self::WrappingKey),
            0x0042_015C => Some(Self::ParentObject),
            0x0042_015D => Some(Self::ChildObject),
            _ => None,
        }
    }
}

/// Normalizer for KMIP link structures between 2.x and 3.0.
pub struct LinkNormalizer;

impl LinkNormalizer {
    /// Converts a KMIP 2.x generic `Link` Structure to a KMIP 3.0 discrete typed link.
    ///
    /// # Errors
    ///
    /// Returns an error if the tag is not 0x0042_0047, or if required fields are missing.
    pub fn normalize_to_v3(generic_link: &Item) -> Result<Item, String> {
        if generic_link.tag().raw() != 0x0042_0047 {
            return Err("Expected KMIP 2.x generic Link tag 0x0042_0047".into());
        }

        let mut link_type = None;
        let mut linked_id = None;

        generic_link.with_value(|view| {
            if let ValueView::Structure(s) = view {
                for child in s.children() {
                    match child.tag().raw() {
                        0x0042_0048 => {
                            child.with_value(|v| {
                                if let ValueView::Enumeration(val) = v {
                                    link_type = LinkType::from_v2_enumeration(*val);
                                }
                            });
                        }
                        0x0042_0049 => {
                            child.with_value(|v| {
                                if let ValueView::TextString(s) = v {
                                    linked_id = Some(s.to_string());
                                }
                            });
                        }
                        _ => {}
                    }
                }
            }
        });

        let lt = link_type.ok_or_else(|| "Missing or unrecognized Link Type in generic Link".to_string())?;
        let id = linked_id.ok_or_else(|| "Missing Linked Object Identifier in generic Link".to_string())?;

        let tag = RawTag::new(lt.v3_tag())
            .map_err(|e| e.to_string())?
            .try_checked()
            .map_err(|e| e.to_string())?;

        Item::new(tag, Value::text_string(id)).map_err(|e| e.to_string())
    }

    /// Converts a KMIP 3.0 discrete typed link to a KMIP 2.x generic `Link` Structure.
    ///
    /// # Errors
    ///
    /// Returns an error if the tag is not a supported discrete typed link tag.
    pub fn normalize_to_v2(typed_link: &Item) -> Result<Item, String> {
        let lt = LinkType::from_v3_tag(typed_link.tag().raw())
            .ok_or_else(|| format!("Tag 0x{:08X} is not a recognized KMIP 3.0 discrete typed link", typed_link.tag().raw()))?;

        let mut linked_id = None;
        typed_link.with_value(|view| {
            if let ValueView::TextString(s) = view {
                linked_id = Some(s.to_string());
            } else if let ValueView::Structure(s) = view {
                for child in s.children() {
                    if child.tag().raw() == 0x0042_0049 {
                        child.with_value(|v| {
                            if let ValueView::TextString(ts) = v {
                                linked_id = Some(ts.to_string());
                            }
                        });
                    }
                }
            }
        });

        let id = linked_id.ok_or_else(|| "Missing Linked Object Identifier in typed link".to_string())?;

        let mut link_struct = Structure::new();
        let tag_type = RawTag::new(0x0042_0048).map_err(|e| e.to_string())?.try_checked().map_err(|e| e.to_string())?;
        let item_type = Item::new(tag_type, Value::enumeration(lt.v2_enumeration())).map_err(|e| e.to_string())?;
        link_struct.try_push(item_type).map_err(|e| e.to_string())?;

        let tag_id = RawTag::new(0x0042_0049).map_err(|e| e.to_string())?.try_checked().map_err(|e| e.to_string())?;
        let item_id = Item::new(tag_id, Value::text_string(id)).map_err(|e| e.to_string())?;
        link_struct.try_push(item_id).map_err(|e| e.to_string())?;

        let link_tag = RawTag::new(0x0042_0047).map_err(|e| e.to_string())?.try_checked().map_err(|e| e.to_string())?;
        Item::new(link_tag, Value::structure(link_struct)).map_err(|e| e.to_string())
    }
}
